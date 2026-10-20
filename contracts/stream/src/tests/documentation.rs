#![cfg(test)]

use crate::StreamError;
use std::string::{String, ToString};

const README: &str = include_str!("../../../../README.md");

#[test]
fn readme_error_table_matches_stream_error_variants_and_codes() {
    let error_heading = README
        .lines()
        .position(|line| line == "## Error code reference")
        .expect("README must contain the Error code reference section");
    let actual: std::vec::Vec<(String, u32)> = README
        .lines()
        .skip(error_heading + 1)
        .skip_while(|line| {
            let mut columns = line.split('|').map(str::trim);
            columns.next() != Some("")
                || columns.next() != Some("Code")
                || columns.next() != Some("Variant")
        })
        .skip(1)
        .take_while(|line| line.trim_start().starts_with('|'))
        .filter(|line| !line.contains("---"))
        .map(|line| {
            let columns: std::vec::Vec<_> = line.split('|').map(str::trim).collect();
            let code = columns[1]
                .parse()
                .expect("README error code must be an integer");
            let variant = columns[2].trim_matches('`').to_string();
            (variant, code)
        })
        .collect();
    let expected = [
        ("StreamNotFound", StreamError::StreamNotFound as u32),
        ("InvalidTimeRange", StreamError::InvalidTimeRange as u32),
        ("InvalidAmount", StreamError::InvalidAmount as u32),
        ("InvalidCliff", StreamError::InvalidCliff as u32),
        ("AlreadyCancelled", StreamError::AlreadyCancelled as u32),
        ("NothingToWithdraw", StreamError::NothingToWithdraw as u32),
        (
            "InsufficientBalance",
            StreamError::InsufficientBalance as u32,
        ),
        (
            "StreamAlreadyCompleted",
            StreamError::StreamAlreadyCompleted as u32,
        ),
        ("AmountTooLarge", StreamError::AmountTooLarge as u32),
        ("StreamWindowInPast", StreamError::StreamWindowInPast as u32),
        (
            "StreamCountExhausted",
            StreamError::StreamCountExhausted as u32,
        ),
        ("InvalidParticipant", StreamError::InvalidParticipant as u32),
    ];

    for (variant, expected_code) in expected {
        let Some((_, actual_code)) = actual.iter().find(|(name, _)| name.as_str() == variant)
        else {
            panic!("README error table is missing StreamError::{variant}");
        };
        assert_eq!(
            *actual_code, expected_code,
            "README error table has the wrong code for StreamError::{variant}"
        );
    }
    for (variant, _) in &actual {
        assert!(
            expected.iter().any(|(name, _)| *name == variant.as_str()),
            "README error table has an unknown variant: {variant}"
        );
    }
    assert_eq!(
        actual.len(),
        expected.len(),
        "README error table must list every StreamError variant exactly once"
    );
}
