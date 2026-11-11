# Application Home graceful oracle

The normal Store **process** lane owns the bounded H3 oracle:

- `process::application::host_oracle::graceful::graceful_home_oracle_seed_zero_preserves_relational_and_world_prefixes`
- `process::application::host_oracle::graceful::graceful_home_oracle_seed_one_preserves_relational_and_world_prefixes`

The default seed range is `0..2`. Each seed has twelve generated keyed operations,
with fresh keys, same-key retries and changed-intent retries. Every completed
operation is followed by a graceful close or quiescent process kill, then a fresh
Host child reopens. One ordinary MemoryOnly reference and one run-wide identity
correspondence compare every prefix and recovered state. This follows the M2
`relational_home` prefix/reopen structure through the production Host.

Subjects are Relational graph state and World heads, occurrence ancestry and
custody, including application idempotency and semantic receipt correspondence.
The court's resolved path dependencies reach these owners, Query execution and
Host; the affected-lane planner therefore includes Store when these inputs
change. `scripts/test_cost/nextest_build.py` registers `process::` tests in its
normal process lane; no additional oracle runner or ignored mode exists.

Signal and changed-build dimensions extend this oracle in H5 and H6. This is not
the C4 fault oracle: it has no incomplete-operation kills, media sweep, I/O
injection or exhaustive mode. MemoryOnly has no Directory close proof; its
reference is disposed after checking the documented retained lifecycle refusal.
