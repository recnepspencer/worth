# Cold funding resume lane evidence

All lane headers identify 852154d4c4 plus a dirty worktree, the exact command,
start/end UTC and exit code. No claim is made for a clean source commit.
Compile/format/static-tool rows have no test cases (0/0/0). Selected-test
ignored count is0; nextest also lists unselected ignored inventory entries,
which were not executed. Owned build component logs are linked by each probe
log under FULL_OWNED_LOG_PATH.

| Lane log | Exit | Passed/failed/selected ignored |
|---|---:|---:|
| [cold-funding-agent-context-generate.log](cold-funding-agent-context-generate.log) | 0 | 0/0/0 |
| [cold-funding-agent-context.log](cold-funding-agent-context.log) | 1 | 0/0/0 |
| [cold-funding-boundary.log](cold-funding-boundary.log) | 1 | 0/0/0 |
| [cold-funding-cost-root-check.log](cold-funding-cost-root-check.log) | 0 | 0/0/0 |
| [cold-funding-cost-root-format.log](cold-funding-cost-root-format.log) | 0 | 0/0/0 |
| [cold-funding-cost-root-growth.log](cold-funding-cost-root-growth.log) | 0 | 1/0/0 |
| [cold-funding-final-agent-context.log](cold-funding-final-agent-context.log) | 0 | 0/0/0 |
| [cold-funding-final-boundary.log](cold-funding-final-boundary.log) | 1 | 0/0/0 |
| [cold-funding-final-line-caps.log](cold-funding-final-line-caps.log) | 0 | 0/0/0 |
| [cold-funding-line-caps-git-bash.log](cold-funding-line-caps-git-bash.log) | 0 | 0/0/0 |
| [cold-funding-line-caps.log](cold-funding-line-caps.log) | 1 | 0/0/0 |
| [cold-funding-partial-home-reopen.log](cold-funding-partial-home-reopen.log) | 0 | 3/0/0 |
| [cold-funding-query-check.log](cold-funding-query-check.log) | 0 | 0/0/0 |
| [cold-funding-query-clippy.log](cold-funding-query-clippy.log) | 0 | 0/0/0 |
| [cold-funding-query-format.log](cold-funding-query-format.log) | 0 | 0/0/0 |
| [cold-funding-query-owner-check.log](cold-funding-query-owner-check.log) | 0 | 0/0/0 |
| [cold-funding-query-owner-format.log](cold-funding-query-owner-format.log) | 0 | 0/0/0 |

The first Query check predates the guard's actual-owner-frontier correction;
cold-funding-query-owner-check is the final guard check. Both are retained.
The first Bash invocation selected WSL without /bin/bash. Explicit Git Bash
is the corrected successful line-cap lane, not a changed guard.

Boundary first reports 5 findings: two nested-cost category locations plus the
three queued Native BC9002 locations. The growth test's assertions/workload and
finite budgets were preserved while moving to the existing top-level cost tree.
Generated-context check requested regeneration; the tool's generate command
ran, changing only the Host context among 6 generated files.

Cold owner implementation and its memory/work controls are not run: exact
private-file extension remains pending. Native3/Host2/Store cold recapture,
cold filesystem zero-covered-read and final CDC/recovery selections are unrun
this resume; their prior failure evidence remains funding-finish-lanes.md.
Full Root/Store Clippy and full funding certification are also unrun. No
ignored tests, query-scale, exhaustive Home oracle or scenario suite ran.

Final state: top-level growth PASS 1/0/0; Host selected3/0/0; final dirty line
cap PASS, 63 files. Final boundary FAIL only the 3 queued Native BC9002 sites,
with no remaining new cost-category finding. Final agent-context check PASS
after tool generation. Query Clippy PASS with recorded warnings. No build or
test process remains running. No budgets, cadences, timeouts, assertions or
production formats were changed by the topology correction.
