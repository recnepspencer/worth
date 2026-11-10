# Static gates on the master-merge repair batch

Log: `fable-repair-static-gates.log` (HEAD `171d380829` + uncommitted repair batch, 2026-11-10).

- Lockfile alignment: PASS (18 lockfiles).
- Line cap: PASS (no changed or added Rust file over 400 lines).
- agent-context check: was RED, `worth-query-host/AGENT_CONTEXT.md` stale (its declared facade exports lacked the
  Application Home surface that earlier integration work added; nothing in this batch touches that facade).
  Regenerated with `agent-context generate` (tool output, one line); included in the repair commit.
- boundary-check: RED, exit 1. Not fixed in this commit. Findings:

| Finding | Files | Cause | Disposition |
|---|---|---|---|
| BC9002 court has a build script ("court cannot contain production targets or Rust outside tests/") | `worth-query-court-decl-host-authored-ownership/Cargo.toml` + `build.rs` | INTRODUCED by the pre-approved PDB v2 patch (`497f5f6f`): the court's Windows test PDB is 6.57 GB, over the 4 GiB link limit, and the patch passes `/PDBPAGESIZE:8192` through a build script | FOLLOW-UP: the law-compliant fix is to split the court by dependency set (iteration_laws) so each binary stays under the limit, then delete `build.rs`. Coordinator to confirm: the patch was approved without a boundary run. |
| BC9002 source test rooted on `ExecutionRequest`, not in `src-court-baseline.json` | `field_authoring_candidate/tests/storage_footprint.rs` (gained a root), `record_arena/values/tests/optional_output_funding.rs` (new), `commit_semantic_result/validation/tests/proposed_families.rs` (new) | INTRODUCED by the pre-approved runtime batch (`e67c0992`) | FOLLOW-UP: move these bodies into the owning court crate (never grow the baseline) |
| BC9002 same, files untouched or with unchanged roots | about 20 `crates/worth-relational/src/**` test files plus one Query `historical_retention_tests.rs` | PRE-EXISTING (came with master; listed in the BC9002 queue in earlier evidence) | repository debt, reported |
| BC7014 reconstruction lane seal | `durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs` | PRE-EXISTING (untouched file) | repository debt, reported |

The repair commit is a safe point with these boundary findings recorded, not a boundary-green HEAD.
