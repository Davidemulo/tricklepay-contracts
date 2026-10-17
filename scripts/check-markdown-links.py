#!/usr/bin/env python3
"""Validate relative Markdown file and heading links in the repository."""
from pathlib import Path
import re, sys
from urllib.parse import unquote, urlsplit
ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"!?(?:\[[^]]*\])\(([^)\s]+)(?:\s+[^)]*)?\)")
HEADING = re.compile(r"^#{1,6}\s+(.+?)\s*#*\s*$", re.MULTILINE)
def slug(text):
    text = re.sub(r"[`*_~]", "", text).lower()
    return re.sub(r"[^a-z0-9 -]", "", text).replace(" ", "-")
errors = []
for markdown in ROOT.rglob("*.md"):
    if any(part in {".git", "target", "node_modules"} for part in markdown.parts): continue
    content = markdown.read_text(encoding="utf-8")
    for raw in LINK.findall(content):
        parsed = urlsplit(raw)
        if parsed.scheme or parsed.netloc: continue
        target = unquote(parsed.path); fragment = unquote(parsed.fragment)
        if not target and not fragment: continue
        resolved = (markdown.parent / target).resolve() if target else markdown.resolve()
        if target and not resolved.is_file(): errors.append(f"{markdown.relative_to(ROOT)}: missing file {raw}")
        if fragment:
            target_content = resolved.read_text(encoding="utf-8") if resolved.is_file() else content
            if fragment.lower() not in {slug(h) for h in HEADING.findall(target_content)}:
                errors.append(f"{markdown.relative_to(ROOT)}: missing heading {raw}")
if errors: print("\n".join(errors), file=sys.stderr); sys.exit(1)
print("All relative Markdown links are valid.")
