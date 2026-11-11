# Independent review: partial queue 4 source handoff

Verdict: **APPROVE the five-file partial source migration/handoff. Queue 4 remains IN PROGRESS.** This is not approval of full adapter removal, a zero-BC9002 claim, or completion of the whole queue.

Reviewed candidate HEAD `1f9f39b138bd5dcf92124317ec2e7ebe3f170153` plus the five dirty Rust files frozen in `queue4-source-state.json`. Read-only review; no build, test, suite, source edit, baseline edit or branch edit was performed. Only this review artifact was written. I read AGENTS.md, the coding guidelines, M3 D12/D17, test-convention v3, the applicable grants and private-assertion handoff, and applied the qa-loop skill. I did not consult earlier reviewer verdicts, the coordinator's migration report/plan, or the prior consumer/provider audit conclusions.

## Numbered findings

1. **No blocking defect found in the partial patch.** The original preserved case exactly matches the HEAD source case. Its behavior sentence is unchanged and occurs exactly once after migration. The complete multiset of the five lifetime test names is preserved across the old file and the new court; the other four bodies are unchanged by the diff. No private claim beyond the five named retirements was silently retired here.

2. **P1, outstanding full-queue blocker, not introduced by this patch:** full local Native adapter deletion crosses explicitly forbidden `branch/root_*` owners. Direct inspection of `branch/root_tests.rs`, `branch/root_authoritative_payload_tests.rs` and transitive `root_tests/request.rs` confirms concrete consumers, not merely provider syntax. `committed_root_and_envelope` constructs a durable local runtime, obtains the actual committed root from `runtime.history.branch_cell`, copies its original envelope, and tests private `relink_canonical_envelope`, committed axes, schema authority, visibility, request readmission, and retained owners. The authoritative-payload tests also pass local `TestDurableMedium::new()` into the persisted-intent fixture and create durable schema-transition commits. Their assertions require the local Native runtime and private root identity. The existing external testkit medium implements the normal external Native trait; it cannot satisfy the independently compiled unit-crate trait or replace those private roots. The two other local medium implementations found in Native source (`SlowFirstPublication`, `CountedTransitionReads`) delegate to a local medium and do not remove this dependency. Existing byte/snapshot sharing crosses only identity-neutral storage. I found no honest alternative that deletes the local Native identity adapter while leaving these files and their mounting unchanged, without a new local bridge or memory substitution. The correction requires the owning team's coordinated migration/permission for these root consumers. Required eventual verification is their preserved public or root-free-unit evidence plus adapter deletion and a boundary run with zero BC9002. Basis: iteration law 2, test-convention sections 1 and 7, grant 12, and the explicit `branch/root_*` prohibition. Complexity or provider count is not the blocker.

3. **P1, full completion remains unproved:** `queue4-boundary.log` exits 1 with exactly three BC9002 findings: `tests/durability/medium/cold_source.rs`, `cold_source/protected_reads.rs`, and `tests/support/durable_medium/runtime_attachment.rs`. None is touched by this five-file patch. The baseline is unchanged. This does not prevent approval of this coherent partial migration, but does prevent reporting queue 4 complete. Grant 12's ultimate zero-findings condition remains outstanding.

4. **Informational, permitted lint debt:** owner Clippy exits 101 at ten untouched, master-changed sites. The raw diagnostics match `queue4-clippy-diagnostics.json`; I independently confirmed each affected file differs between `7d0e5189bc` and `origin/master`, and is unchanged by this patch. The standing rule explicitly requires leaving and listing them. No new allow or suppression is added. The sites are listed below; Clippy must not be described as green.

## Source correctness and assertion preservation

I read the complete five candidate files and the actual three tracked diffs with three context lines, including both untracked new files as complete source. The new fixture creates a real durable Native producer, publishes one entity plus sixteen updates through ordinary transactions, releases the snapshots, captures a real external-member checkpoint, retains the exact produced members and canonical bytes, drops the producer, and recovers a fresh durable Native runtime. Locator identity uses SHA-256 and exact byte length over the original artifact bytes. No fake receipt, forged authority, memory runtime, raised limit, new Native observer/test door, alias, feature, or test mounting is introduced. Reusing the producer medium for the receiver changes the initial offer count, but the asserted invariant remains exact before/after equality around the read refusal; it does not replace publication with memory behavior.

The former private branch-head inequality is preserved through an admitted main-branch basis and its selected commit ID, which establishes the demanded receipt is nonhead. The basis is dropped before the read-lifetime experiment. `ReplayAccess::canonical_commit_envelope` calls the identical `HistoryAccess::demand_commit_envelope` owner path and only wraps its existing Arc in `SharedCanonicalCommitEnvelope`. `AsRef` exposes that same allocation, so pointer equality preserves the original shared-body identity assertion.

I traced envelope residency demand, original source readmission, the shared wrapper, readmission funding, control allocation and `RetainedCanonicalPosition`. The public wrapper owns only the actual Arc. Decoded backing and nested funding follow the decoded body; retained position owns only a Weak plus the separate fixed allocation ticket, which contains no body/source/runtime edge. Consequently the independent unlimited `SerialMemoryBudget::new(None)` isolates read funding without changing the original unlimited policy: charge remains while the second same-body reader exists, decreases only after the final reader drops, and the attempted reread must then reach the deliberately failing provider. The last requirement independently prevents a retained catalog body from passing by a charge-only accident. The fixed Weak control allocation can lawfully remain funded until its runtime owner drops; demanding immediate zero would be incorrect. Final runtime drop returning the read budget to zero covers that retained control fate as well.

The exact original receipt equality, provider read-count equality while shared, typed `StorageUnavailable { subject: Envelope(position), cause: SourceUnavailable }`, unchanged latest patch frontier, unchanged offered-artifact count, and successful same-receipt retry after clearing the failure are preserved. Holding an actual second public share and comparing its receipt preserves the second-reader custody observation. There is no assertion loss hidden in a fixture that preloads or permanently retains the queried body: the forced-provider refusal after dropping the readers would fail under that defect.

| Material risk | Distinguishing evidence |
| --- | --- |
| Counterfeit recovery or identity | Ordinary durable publication, native external-member capture, exact original bytes, fresh runtime recovery and original receipt |
| Second demand reconstructs instead of sharing | Same body address, forced provider failure and unchanged read count |
| Catalog retains the final reader's body | Intrinsic funding decrease and subsequent exact provider refusal |
| Failure publishes or poisons retry | Exact frontier/offers equality, cleared failure and successful same-receipt retry |

## Raw verification evidence

All named logs have the exact candidate HEAD above, `git_status_short_empty: false`, a command header, UTC start/end and explicit exit code. The logs do not claim per-command source hashes; the separately frozen hashes below match the current files. I inspected the raw output rather than treating the audit's assertions as authority.

- `queue4-envelope-check.log`: check of testkit and external court tests, exit 101, actual E0624 at private `branch_head`. This is failed evidence, corrected in the candidate.
- `queue4-envelope-public-basis-check.log`: same check after the public-basis correction, exit 0. Existing warnings remain outside the patch.
- `queue4-public-envelope-custody.log`: exactly the named external case passes, 1/0/0, nextest summary one run and 198 skipped. The JSON includes an ignored catalog entry for an unrelated cost test; the preselection rejects ignored selected tests, and the run summary confirms that entry was not executed. No ignored/query-scale/exhaustive-oracle/scenario test is claimed or needed.
- `queue4-native-unit-check.log`: Native `cargo check --tests`, exit 0; compilation evidence, not an executed unit-test suite.
- `queue4-owner-clippy.log`: exit 101 at the ten permitted skipped sites below; no error in the five-file patch.
- Both format logs: exit 0. First formats all three owners; the correction formats Native and the court. No later source mutation is in the reviewed diff.
- `queue4-line-caps.log`: exit 0, five dirty Rust files checked. Independently counted lengths are 99, 43, 261, 143 and 17, all within 400.
- `queue4-agent-context.log`: exit 0.
- `queue4-boundary.log`: exit 1, the exact three outstanding provider findings above, no zero-findings claim.
- `queue4-scope-audit.log`: exit 0. I also independently checked the actual changed/untracked inventory, name multisets, frozen hashes, Lifecycle eligibility, unchanged baseline/root/transferred files and all ten excluded Lifecycle files. No lane was rerun for this review.

The owned probe uses the existing Cargo/nextest capture and selection components; its added named court reservation is one Serial caller and does not change product resource limits or artifacts. The logs record removal of a stale linker override rather than use of an artifact-changing workaround. The probe's emitted 1/0/0 agrees with the real named PASS and nextest summary; the ignored catalog telemetry is not counted as an executed ignored test.

## Scope and frozen files

For every following file I independently ran `git diff --quiet 7d0e5189bc codex/m3-p3-lifecycle -- <file>`; all five returned 0. These are the only source changes, and all qualify under the conditional outside-area grant for queue item 4. SHA-256 matches `queue4-source-state.json`:

| File | SHA-256 |
| --- | --- |
| `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/cold_envelope_custody.rs` | `58c01444f02ecf045758c9d9ad77a2980c16217cbbf462e90ed2d954228e2d34` |
| `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/mod.rs` | `0333daa0190d9cd43beb9fece0a87fbcc0ed37e7b2171a54e87fcc7e6ef0284f` |
| `crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs` | `f37e37b7fbd5dd791e78f442553d65575a1c61d8f034ee34609740c95092ff02` |
| `crates/worth-testkit-relational/src/cold_recovery.rs` | `d49574380413a73e33f082f85b2b21295fcb09a8151c9b784985e0f557eaf80d` |
| `crates/worth-testkit-relational/src/lib.rs` | `f53d22b3810edb479a5c58316d2d3dc4a3b602194bd49f18e42388145f6a9f9f` |

The explicit `branch/root_*` exclusion controls even if an individual file passes the generic Lifecycle diff probe. Neither transferred checkpoint-complete file, any of the ten Lifecycle exclusions, generated context, manifest, baseline or enforcement file changes here.

Permitted Clippy skip sites, relative to `crates/worth-court-relational-certification/tests/worth_court_relational_certification/`:

1. `process/serial/relational/managed_source_work_account/public_only.rs:38`: `int_plus_one`.
2. `process/serial/relational/managed_source_work_account.rs:292`: `int_plus_one`.
3. `process/serial/relational/managed_source_work_account.rs:104`: `unnecessary_lazy_evaluations`.
4. `relational/durable_publication/index_catalog/recovery.rs:109`: `needless_borrow`.
5. `relational/durable_publication/index_catalog/recovery.rs:340`: `needless_borrow`.
6. `relational/durable_publication/index_catalog.rs:169`: `needless_borrow`.
7. `relational/durable_publication/index_catalog.rs:212`: `needless_borrow`.
8. `relational/managed_source/nested.rs:110`: `err_expect`.
9. `relational/managed_source/row_read.rs:48`: `err_expect`.
10. `relational/managed_source/owner.rs:267`: `collapsible_if`.

**Final decision: APPROVE for the partial five-file source handoff only. Full queue 4 remains IN PROGRESS with three BC9002 findings and the forbidden root-consumer dependency requiring owner coordination.**
