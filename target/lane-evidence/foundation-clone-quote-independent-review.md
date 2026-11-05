# Independent review: Foundation clone allocation quote

Verdict: REJECT for this two-file component pending finding 1.

Scope: HEAD ff0e8e88cad6be1b54790ff60357eb5f58deea8f to the inspected dirty candidate in `crates/worth-foundational/src/values/aspect_value.rs` and `crates/worth-foundational/src/values/aspect_value/tests.rs` only. Read-only source review; no builds, tests, or source edits performed. This is not approval or rejection of the complete funding slice, Root Clippy, or other dirty work.

## Numbered findings

1. **P2: reduce the ordinary unit-test fixture below the cost-category threshold.** `crates/worth-foundational/src/values/aspect_value/tests.rs:44` sets `length = 65536` in the ordinary `values::aspect_value::tests` module. Iteration Law 15 explicitly requires a `2^14+` size constant to live under `cost`. The current test needs only unequal length/capacity, so the large fixture adds no distinguishing evidence. Root cause: selecting a payload scale unrelated to this local clone contract. Correction: use a small nonzero length such as 8 while preserving the capacity, quote, and actual-clone assertions. Verification: rerun `inline_bytes_clone_quote_counts_length_without_spare_capacity` for the corrected candidate and refresh its recorded evidence. Governing principle: MENTALITY section 3 (mechanically enforce consequential rules), with the direct requirement in Iteration Law 15; Testing Laws 2 and 13 also favor the narrow local proof. This is an in-scope constitutional issue, not a functional failure of the quote implementation.

## Review observations

- Production logic is sound for the requested change. `InlineBytes` wraps `Vec<u8>` and derives `Clone`; `as_bytes().len()` measures copied bytes, excluding the original spare capacity. The changed match is exhaustive across every `AspectValue` variant and both `InternedString` variants. The zero arms contain only fixed-storage primitives, Copy scalar wrappers, inline UUID, and fixed identifier/reference types. Decimal, BigInt, Rational, and Raw string length accounting is preserved, including checked addition for Rational.
- The test independently observes the actual clone's owned capacity as well as the quote, and therefore distinguishes both the old zero result and an incorrect source-capacity quote. Its source-capacity assertion confirms the intended spare-capacity setup.
- The existing portable patch quote and application bound consumers delegate to this scalar quote with checked accumulation. No competing quote implementation or new caller surface is introduced by this component.
- The Native controlled-patch guard in `crates/worth-relational/src/authority/mutation/intents/record_aspect_patch/controlled.rs:54` remains unchanged from HEAD: if retained output exceeds `memory.bytes()`, the path returns `ChargedBytesOverflow` before resizing/returning the controlled patch. The file is clean against HEAD.
- The Foundation dirty inventory contains exactly the two reviewed files. The patch changes only the existing method body and adds the local pure assertion. No feature, manifest, public signature, facade/export, authority, or boundary expansion occurs. Files remain below 400 lines (169 and 51 lines respectively).

## Compact risk matrix

| Claim | Relevant failure | Distinguishing evidence |
| --- | --- | --- |
| Clone quote matches InlineBytes copy | Zero quote or quoting source spare capacity | Source length/capacity divergence plus actual-clone capacity assertion |
| Zero quote is fixed storage only | Heap-bearing variant silently falls through | Exhaustive variant match plus inspection of scalar/reference representations |
| Native refusal remains intact | Underfunded output accepted after construction | Clean guard file and comparison with HEAD guard hunk |
| Local test stays ordinary | Unnecessary large fixture in ordinary category | Finding 1; small fixture preserves the same oracle |

## Evidence and limitations

- `target/lane-evidence/funding-final-foundation-quote.log` records HEAD ff0e8e88cad6be1b54790ff60357eb5f58deea8f, `git_status_short_empty: false`, exit code 0, and 1 passed / 0 failed / 0 ignored. Its referenced raw `target/test-cost/root/funding-final-foundation-quote/run-selection.log` confirms execution of the exact new test and successful completion.
- `target/lane-evidence/funding-final-check.log` records the same HEAD and dirty-tree header, with `cargo check --tests` covering Foundation, Relational, and its certification court, exit code 0. It contains warnings in other owners; this review does not assess their closure.
- Supplemental `target/lane-evidence/funding-final-byte-controls.log` records the same HEAD and dirty-tree header, exit code 0, and the two cold InlineBytes head/history cases passing. These support adjacent consumer behavior but do not replace the focused assertion or approve those other components.
- Existing logs were inspected rather than rerun as instructed. Full Root Clippy and complete scoped enforcement remain the parent's separate responsibility. No whole-slice clearance is implied.
