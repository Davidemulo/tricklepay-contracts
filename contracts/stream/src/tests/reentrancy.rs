#![cfg(test)]

use soroban_sdk::{contract, contractimpl, testutils::Address as _, token, Address, Env, String};

use crate::contract::{StreamContract, StreamContractClient};

use super::helpers::StreamTest;

/// A hostile token that attempts to re-enter the stream contract during transfer.
#[contract]
struct ReentrantToken;

/// Client generated for the hostile token.
#[allow(dead_code)]
pub struct ReentrantTokenClient<'a> {
    env: &'a Env,
    address: Address,
}

#[allow(dead_code)]
impl<'a> ReentrantTokenClient<'a> {
    pub fn new(env: &'a Env, address: &Address) -> Self {
        ReentrantTokenClient {
            env,
            address: address.clone(),
        }
    }

    pub fn set_reentrancy_target(&self, stream_contract: &Address) {
        self.env.invoke_contract(
            &self.address,
            &soroban_sdk::symbol_short!("set_reentrancy_target"),
            soroban_sdk::vec![self.env, stream_contract],
        )
    }
}

#[contractimpl]
impl ReentrantToken {
    /// Standard token transfer that attempts to re-enter the stream contract.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        // Store call context to detect re-entrancy attempt.
        let key = String::from_str(&env, "reentrancy_target");
        if let Some(stream_contract) = env.storage().temporary().get::<_, Address>(&key) {
            // Attempt to re-enter: try to withdraw or cancel during the transfer.
            let stream_client = StreamContractClient::new(&env, &stream_contract);

            // Try to call withdraw on stream id 0.
            let _ = stream_client.try_withdraw(&0);
        }

        // Complete the actual transfer (simplified - just for testing).
        // In a real token this would move balances, but for this test
        // we only care about the re-entrancy behavior.
    }

    pub fn set_reentrancy_target(env: Env, stream_contract: Address) {
        let key = String::from_str(&env, "reentrancy_target");
        env.storage().temporary().set(&key, &stream_contract);
        env.storage().temporary().extend_ttl(&key, 100_000, 100_000);
    }
}

/// Soroban normally rejects direct re-entry into a contract already on the call
/// stack. This test verifies what happens when a token attempts to call back
/// into the stream contract during a transfer.
///
/// The threat model states: "A token whose `transfer` invokes back into this
/// contract (for example, from a hook on the recipient) executes before this
/// contract's own state write completes for `create_stream` (transfer precedes
/// the write) but after it for `withdraw`/`withdraw_amount`/`cancel` [...] —
/// a reentrant call in the latter case would read `stream.withdrawn` already
/// updated for the in-flight withdrawal, so a second, concurrent withdrawal of
/// the same accrued balance is not possible. It could still interleave with the
/// schedule (e.g., trigger a `cancel`) in ways not exercised by the test suite."
#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_reentrancy_during_withdraw_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    // Register the stream contract and hostile token.
    let stream_contract_id = env.register(StreamContract, ());
    let stream_contract = StreamContractClient::new(&env, &stream_contract_id);

    let hostile_token_id = env.register(ReentrantToken, ());
    let hostile_token_address = hostile_token_id.clone();
    let hostile_token_client = ReentrantTokenClient::new(&env, &hostile_token_id);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Configure the hostile token to attempt re-entry into the stream contract.
    hostile_token_client.set_reentrancy_target(&stream_contract_id);

    env.ledger().set_timestamp(100);

    // Create a stream using the hostile token.
    // This succeeds because the transfer happens before state is written.
    let id = stream_contract.create_stream(
        &sender,
        &recipient,
        &hostile_token_address,
        &1_000,
        &100,
        &1_100,
        &100,
    );

    // Advance time so some tokens vest.
    env.ledger().set_timestamp(600);

    // Attempt to withdraw. The hostile token's transfer will try to call back
    // into the stream contract's withdraw method during the transfer.
    // Soroban should reject this re-entrancy with InvalidAction.
    let _ = stream_contract.withdraw(&id);
}

/// Similar test for cancel operation - token attempts re-entry during the refund transfer.
#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_reentrancy_during_cancel_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract_id = env.register(StreamContract, ());
    let stream_contract = StreamContractClient::new(&env, &stream_contract_id);

    let hostile_token_id = env.register(ReentrantToken, ());
    let hostile_token_address = hostile_token_id.clone();
    let hostile_token_client = ReentrantTokenClient::new(&env, &hostile_token_id);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    hostile_token_client.set_reentrancy_target(&stream_contract_id);

    env.ledger().set_timestamp(100);

    let id = stream_contract.create_stream(
        &sender,
        &recipient,
        &hostile_token_address,
        &1_000,
        &100,
        &1_100,
        &100,
    );

    env.ledger().set_timestamp(600);

    // Attempt to cancel. The hostile token's transfer (for the refund) will
    // try to call back into the stream contract during the transfer.
    let _ = stream_contract.cancel(&id);
}

/// Test that uses a standard token to verify normal operation still works.
/// This confirms the re-entrancy protection doesn't break legitimate use.
#[test]
fn normal_token_withdraw_and_cancel_work() {
    let t = StreamTest::setup(1_000);
    t.set_time(100);

    let id = t.contract.create_stream(
        &t.sender,
        &t.recipient,
        &t.token_address,
        &1_000,
        &100,
        &1_100,
        &100,
    );

    t.set_time(600);

    // Normal withdraw succeeds.
    let withdrawn = t.contract.withdraw(&id);
    assert_eq!(withdrawn, 500);
    assert_eq!(t.token.balance(&t.recipient), 500);

    // Normal cancel succeeds.
    let refund = t.contract.cancel(&id);
    assert_eq!(refund, 500);
    assert_eq!(t.token.balance(&t.sender), 500);

    // Verify no double-counting: contract balance is now zero.
    assert_eq!(t.token.balance(&t.contract.address), 0);
}
