# Merge-history slice lanes

Counts:0cargo check,1Clippy,4focused builds,3selected test invocations.
Tool Cargo runs: boundary and context. No ignored test body ran.

| Lane log | Exit | Pass/fail/ignored |
|---|---:|---|
| landing2a-merge-owner-boundary.log | 1 | 0/0/0 |
| landing2a-merge-owner-clippy.log | 0 | 0/0/0 |
| landing2a-merge-owner-context.log | 0 | 0/0/0 |
| landing2a-merge-owner-fmt.log | 0 | 0/0/0 |
| landing2a-merge-owner-functions.log | 0 | 0/0/0 |
| landing2a-merge-owner-line-caps.log | 0 | 0/0/0 |
| landing2a-merge-owner-public-intermediate-cold.log | 0 | 1/0/0 |
| landing2a-merge-owner-public-record-ref-fix.log | 100 | 0/1/0 |
| landing2a-merge-owner-public.log | 1 | 0/0/0 |
| landing2a-merge-owner-validation.log | 0 | 1/0/0 |

Full commands, invocation HEAD, dirty status, times and exit in each log.
Owned build/test raw logs referenced by wrappers are preserved with
test-cost-landing2a-merge-owner* names in this same directory.

Failure roots: initial public build moved non-Copy RecordRef twice: fixture
fixed by cloning descriptive record at both uses. First run assumed all
recovery canonical reads were absent; actual claim is the nonhead intermediate
position. Corrected fixture asserts its exact absence then real demand.
Both were test errors; no product algorithm changed to hide either failure.
Boundary12remain:1ownednext runtime_attachment,11routed others; no finding
in completed slice. Clippy32lib/23duplicatetestwarnings remain next ownedgate.
