# Same-directory reopen: direct default-stack reproduction

Candidate HEAD: `b8d9b28608f434c4c21bedb22dbb6da96d7b20d8`, clean source.

The queued failure did not reproduce on this branch. The exact court
`host::application_home::reopening::same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key`
passes directly: **1 passed, 0 failed, 0 ignored**, exit 0.

Full evidence: `queue1-same-directory-direct-unset.log`. It includes the owned
nextest build/selection paths and the exact executable invocation. The runner
selects exactly one non-ignored test, invokes that test executable directly,
and removes `RUST_MIN_STACK` from the execution environment. It sets no stack
size, creates no worker thread, changes no request policy and selects no suite.
The committed Cargo toolchain/profiles and existing build-identity runner are
used. Compiler/build logs remain at the paths printed in the full lane log.

No product edit, boxing or phase split was made. This is evidence that the
named reproducer passes at this HEAD, not a claim that all default-stack paths
or the still-unmerged integration tree are qualified. The older integration
failure remains valid historical evidence for its own HEAD.

Boundary review: Journal-only original-frame readmission remains in Native's
`canonical_commit_artifact/readmission.rs::decode_payload`. Reproduction comes
before changing that owner. Since this reproducer passes, the user's explicit
instruction is to record the result and move to D13. No frame-size diagnosis
or speculative source repair is claimed.
