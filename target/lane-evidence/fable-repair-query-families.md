# First permitted Query Tier 2 after the master-merge repair batch: red families

Lane: `fable-repair-query-tier2-b.log` (HEAD `171d380829` + uncommitted repair batch, 2026-11-10). Out dir
`fable-repair-tier2-b/query/`. Per-test list with panic site and message:
`fable-repair-query-tier2-b-families.txt` (69 groups). First attempt (`fable-repair-query-tier2.log`) died in
its build with "no space on device" and ran nothing.

Result: `run-query-execution.log` 619/619 passed; `run-query-rest.log` 7014 passed, 107 failed.
**Total 7633 passed, 107 failed.** There is no pre-repair Query run to compare with (the Query build could not
link before the PDB fix), so "new" versus "pre-existing" cannot be read from logs.

Confirmed green (the two Host courts that prove the `index_catalog.rs::readmit` routing fix):
`journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing` and
`close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen`.

| # | Family (cause as printed) | Tests | Track call | Next step |
|---|---|---|---|---|
| Q1 | Index unavailable or corrupt after checkpoint reopen: `IdentityIndexUnavailable`, `CorruptIdentityIndex`, `EqualityIndexUnavailable`, "installed index ... is current at the new commit", `RetainedBasisUnavailable` | 64 | HAPPY product (graceful reopen cannot resolve principals/entities) | P-C `2dce9c178c` (checkpoint index selections) is reported to cure it; take it in the next batch merge and rerun exactly these names. Same root as Root family F3. |
| Q2 | `semantic envelope seal: ConflictingCreateCorrespondence` on restore in authored-ownership correction courts (`correction_courts.rs:351`) | 8 | UNCLASSIFIED (Phase 9 corrections; may be the semantic-v2 seal refusing a real conflict in the fixture) | read the seal check against the restore batch's create bindings |
| Q3 | `History(Document(ReceiptMismatch))` in owned-outputs fixed-forest and generated-source courts | 8 | UNCLASSIFIED (Phase 6) | read the receipt comparison |
| Q4 | `injected unwind before product comparison` (`worth-runtime-world/src/lifecycle/operation_control.rs:164`) | 4 | CLEANUP (C3 owner crash) candidates | confirm they are unwind courts, then move out |
| Q5 | Cold capture refused, `UnsupportedLegacySemantics` (Root family F1) | 2 | as F1 | as F1 |
| Q6 | `DurableBranchPublicationRequired` creating an activation sibling | 2 | UNCLASSIFIED (program transition on a non-main branch) | read |
| Q7 | Other, one or two tests each (recovered publication evidence, checkpoint magic offset, retained roots, `WrongImmutableTarget`, memory/work exhaustion, separate-process reader exit 101) | 18 | UNCLASSIFIED | several are probably downstream of Q1; rerun after Q1 is cured before reading |
| Q8 | `...authority_cause::tests::authority_evidence_exposes_request_resources_by_value_through_the_commit_door` ended by hand after 46 CPU minutes | 1 | NOT a spin: a sweep of 9402 probes, each a fresh-world commit; it had reached probe 5800 and was progressing (log lines `authority probe=...`). The batch does not touch its crate; the body came with master (`b4ef380fa7`). | decide whether the probe set grew or the court belongs with the scale courts the permitted policy excludes |

No family here was analysed to root cause in this turn; none is claimed fixed by the repair commit.
