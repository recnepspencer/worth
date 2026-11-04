# Checkpoint funding evidence (IN PROGRESS)

Logs retain exact command, HEAD, dirty flag, times, exit and counts. Source was dirty at fab7d6e943; executable source snapshots and compiler-unit records retain attribution. Diagnostic capture is not production qualification.

| Lane log (under target/lane-evidence) | Exit | Pass/fail/ignored |
|---|---:|---:|
| funding-diagnostic-check.log | 101 | 0/0/0 |
| funding-diagnostic-format-query.log | 0 | 0/0/0 |
| funding-diagnostic-format-root.log | 0 | 0/0/0 |
| funding-diagnostic-format-store.log | 0 | 0/0/0 |
| funding-diagnostic-owned-check.log | 0 | 0/0/0 |
| funding-host-agent-context.log | 0 | 0/0/0 |
| funding-host-boundary.log | 1 | 0/0/0 |
| funding-host-caller-stop-and-bounded.log | 0 | 2/0/0 |
| funding-host-clippy.log | 101 | 0/0/0 |
| funding-host-format-query.log | 0 | 0/0/0 |
| funding-host-format-store.log | 0 | 0/0/0 |
| funding-host-line-caps-git-bash.log | 0 | 0/0/0 |
| funding-host-line-caps.log | 1 | 0/0/0 |
| funding-host-owned-check.log | 0 | 0/0/0 |
| funding-host-public-control.log | 0 | 1/0/0 |
| funding-host-review-check.log | 0 | 0/0/0 |
| funding-host-review-format.log | 0 | 0/0/0 |
| checkpoint-funding-instrumented-head-forty-one.log | 0 | 1/0/0 |

No ignored tests executed. Nextest lists two pre-existing ignored cases as filtered metadata; selected named cases are nonignored and actual executed-test counts above exclude those cases.

Failures: bare diagnostic check omitted required application-build identity (fixed invocation); first bash selected absent WSL runtime (fixed Git Bash invocation). Scoped Clippy is stopped at exact Query producer needless borrow, awaiting owner handoff. Boundary has three queued BC9002 findings. No lowered lint or baseline installed.

UNRUN this funding slice: Native state-growth/covered-body-read proof (producer not implemented), post-funding decode/default-stack qualification, D13, raw-digest and BC9002 queue qualification, exhaustive/ignored/scenario/query-scale (forbidden). Prior D12 logs are not relabeled as current funding qualification.
