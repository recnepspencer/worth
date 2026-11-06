# Structural measures: worth-query-execution

- production lines (non-test, own text): 454268
- in-crate test families: 505 (2266 tests, 204611 lines)
- Rust source lines under src/: 659993

| Workspace | Edit reach (packages) | Fan-in x production lines | Library test target |
|---|---:|---:|---|
| `Cargo.toml` | 9 | 4088412 | yes |
| `workspaces/worth-query/Cargo.toml` | 25 | 11356700 | yes |
| `workspaces/worth-query-bank-world/Cargo.toml` | 18 | 8176824 | yes |
| `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml` | 6 | 2725608 | yes |

Query workspace: 21 test targets, 13 court crates, 14 testkit crates (ceiling 15000 lines).

| Court or testkit | Lines | Over ceiling |
|---|---:|---|
| worth-query-court-decl | 40 |  |
| worth-query-court-decl-host | 31448 | yes |
| worth-query-court-decl-host-package-archive-replay-application-graph | 15610 | yes |
| worth-query-court-decl-host-package-archive-replay-workflow-approval | 4541 |  |
| worth-query-court-host-replay | 663 |  |
| worth-query-court-installation | 1503 |  |
| worth-query-court-package-archive | 3401 |  |
| worth-query-court-publication | 108 |  |
| worth-query-court-query | 3167 |  |
| worth-query-court-query-decl-host | 15971 | yes |
| worth-query-court-query-installed-operating-world | 10262 |  |
| worth-query-court-query-installed-operations | 6495 |  |
| worth-query-court-source | 9 |  |
| worth-query-testkit-certification | 8472 |  |
| worth-query-testkit-certification-application-graph | 13777 |  |
| worth-query-testkit-certification-canonical | 10284 |  |
| worth-query-testkit-certification-correction | 19631 | yes |
| worth-query-testkit-certification-granular-invalidation | 7056 |  |
| worth-query-testkit-certification-operation | 8199 |  |
| worth-query-testkit-decl | 73 |  |
| worth-query-testkit-document-retention | 14127 |  |
| worth-query-testkit-host | 10084 |  |
| worth-query-testkit-installation | 340 |  |
| worth-query-testkit-installed-operation-fixture | 11667 |  |
| worth-query-testkit-package-archive | 1943 |  |
| worth-query-testkit-publication | 1135 |  |
| worth-query-testkit-query | 9333 |  |
