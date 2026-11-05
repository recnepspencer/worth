# Admission DATA prerequisite lane evidence

Source through d61b52fe288f7a116530573dc19dc87c78167611; base3d7ccf16b0.
Fresh independent review APPROVE covers this source slice only; see
admission-data-independent-review.md. Full-task approval remains outstanding.
Each log retains its exact command, actual HEAD/dirty state, start/end times and
exit/count footer. These runs used an uncommitted source batch; their headers
honestly record that state. Historical successes are not claimed as reruns of
later source. Complete owned build and execution logs are embedded in all seven
runtime logs. No ignored test body executed.

| Lane log | Result (exit) | Pass/fail/ignored | Recorded HEAD | Clean |
|---|---|---|---|---|
| [native-image-fixed-binary-format.log](native-image-fixed-binary-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-fixed-binary-check.log](native-image-fixed-binary-check.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-fixed-binary-units.log](native-image-fixed-binary-units.log) | PASS (0) | 2/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-format.log](native-image-selector-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-check.log](native-image-selector-check.log) | FAIL (101) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-format-corrected.log](native-image-selector-format-corrected.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-check-corrected.log](native-image-selector-check-corrected.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-public-courts.log](native-image-selector-public-courts.log) | PASS (0) | 7/0/0 | 3d7ccf16b0 | false |
| [native-image-streaming-selector-format.log](native-image-streaming-selector-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-streaming-selector-check.log](native-image-streaming-selector-check.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-funded-format.log](native-image-selector-funded-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-selector-funded-check.log](native-image-selector-funded-check.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-fixed-binary-units-corrected.log](native-image-fixed-binary-units-corrected.log) | PASS (0) | 3/0/0 | 3d7ccf16b0 | false |
| [native-image-streaming-selector-public-courts.log](native-image-streaming-selector-public-courts.log) | PASS (0) | 7/0/0 | 3d7ccf16b0 | false |
| [native-image-inorder-selector-format.log](native-image-inorder-selector-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-inorder-selector-check.log](native-image-inorder-selector-check.log) | FAIL (101) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-inorder-selector-format-corrected.log](native-image-inorder-selector-format-corrected.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-inorder-selector-check-corrected.log](native-image-inorder-selector-check-corrected.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-inorder-selector-public-courts.log](native-image-inorder-selector-public-courts.log) | PASS (0) | 2/0/0 | 3d7ccf16b0 | false |
| [native-image-probe-format.log](native-image-probe-format.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-probe-check.log](native-image-probe-check.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-probe-format-unit.log](native-image-probe-format-unit.log) | PASS (0) | 1/0/0 | 3d7ccf16b0 | false |
| [native-image-source-whitespace.log](native-image-source-whitespace.log) | PASS (0) | 0/0/0 | 3d7ccf16b0 | false |
| [native-image-bounded-family-format.log](native-image-bounded-family-format.log) | PASS (0) | 0/0/0 | b8c3d86727 | false |
| [native-image-bounded-family-check.log](native-image-bounded-family-check.log) | FAIL (101) | 0/0/0 | b8c3d86727 | false |
| [native-image-bounded-family-format-corrected.log](native-image-bounded-family-format-corrected.log) | PASS (0) | 0/0/0 | b8c3d86727 | false |
| [native-image-bounded-family-check-corrected.log](native-image-bounded-family-check-corrected.log) | PASS (0) | 0/0/0 | b8c3d86727 | false |
| [native-image-bounded-family-public-courts.log](native-image-bounded-family-public-courts.log) | PASS (0) | 9/0/0 | b8c3d86727 | false |

The corrected codec logs cover four unique pure cases (3/0/0 and1/0/0).
The seven streaming public cases passed7/0/0; the later in-order change and
three-publication fixture passed its affected cases2/0/0. Public image cases
remain DATA/capture observations. The final bounded-family public lane passes9/0/0, including long
UTF-8 family copying, ordering work stops and cancelled invoking requests. These
results do not prove operational Home
admission, bounded-tail reopen, interrupted ACK or capture-origin restoration.

UNRUN this resume: current-HEAD Query/Store compilation; Host/Query same-directory
stack reproductions and compiler frame inventory; <=40-body filesystem court;
zero-member interrupted ACK; filesystem reopen/kill, CDC and recovery qualification;
D13 proof/refusal courts; BC9002 boundary completion; phase-completion line-cap,
boundary and generated-context gates. Formatting of both touched crates passed.
No integration merge or other-worktree edit occurred. No current development Home
format was retired. See the separate failure ledger for the two compiler failures.
