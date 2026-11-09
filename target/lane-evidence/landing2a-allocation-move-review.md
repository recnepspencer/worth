# Landing 2a allocation court move review

Verdict: APPROVE

Scope: independent read-only review of the dirty patch over 2a51552fc5058c7f2ed2ad18ca3bd6e580acb568 in request.rs and milestone-3.md. Reviewed AGENTS.md instructions, all docs/coding-guidelines documents, qa-loop and code-quality-qa, the named 2026-11-09 ruling, and target/lane-evidence/bc-final-root-proof-root.log. No Cargo commands, builds, tests, product edits, or specification edits were performed by this reviewer.

## Findings

1. [P3, CLOSED] The initial moved-court prose contained a literal question mark in the allocation threshold, obscuring the required minus-one boundary. This was a transcription defect against the explicit ruling and MENTALITY section 4, not a product defect. The coordinator corrected the threshold to `controller + size_of(root) - 1` and corrected the C2 heading punctuation. Re-read the corrected spec hunk and confirmed closure. No open findings.

## Evidence and reasoning

- The final dirty inventory has exactly the two authorized tracked files. The source patch deletes only `request_root_refusal_preserves_original_and_releases_partial_backing` and its now-unused `RelationalBranchRootIdentityIssuer` import. A direct source comparison confirms that removing precisely those elements from the base produces the current file byte-for-byte after newline normalization. The surviving visibility test and all its assertions are unchanged. No production behavior, facade, authority, dependency, feature, or other assertion changes are present.
- The moved-court entry is under `Moved courts` / `C2 - Cost and bounds: backing and work budgets`. It explicitly says moved, not retired or weakened, preserves the refusal, partial backing release, original preservation, and cancellation claims, and requires a public allocation-refusal observation before restoration.
- `2a51552fc5:crates/worth-relational/src/branch/root_tests/request.rs` resolves to the exact current pre-deletion HEAD and contains the complete named court, including the minus-one threshold and both zero-charge assertions. It is therefore a real last-containing restoration reference, not an invented or already-deleted source location.
- The supplied historical log records HEAD e7f2490cb1a4f874189d6104849cad09e0917ff7, a dirty worktree, and a 2026-11-07 root proof run with five tests passed, zero failed, zero ignored, and exit code zero. It explicitly records the named allocation court passing in 2.984 seconds. This establishes historical execution evidence; it is not represented here as a fresh final-tree run or as public-boundary coverage.
- The scoped Rust file is 48 lines, below the 400-line limit. `git diff --check` passes. Deletion introduces no structural or authority surface. The public observation prerequisite keeps the future C2 court aligned with the evidence boundary required by Testing Laws 2, 3, and 20.

## Limits

Per the review instruction, no builds, tests, or repository enforcement lanes were run. This approval is for the authorized court move and its preserved restoration contract. It does not claim C2 implementation, a public allocation-refusal observer, or final landing gate completion.
