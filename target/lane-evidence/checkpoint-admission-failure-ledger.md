# Failure attribution and resolution

## Resume3d7ccf16b0 through b8c3d86727

Correction through d61b52fe28 adds these resolved failures; the current index
contains28 terminal lanes:

| Failure/evidence | Root cause and wrong side | Fix / final state |
|---|---|---|
| admission_data_final_review REJECT, P2 | Whole-string UTF-8 validation, comparison and copying could exceed the image's declared <=64KiB request polling interval; product algorithm was wrong | Use the existing raw slice cursor with bounded UTF-8 validation/copy and bounded comparison windows, preserving family wire grammar and admitting grants before allocation. New public long-family control and all nine image courts pass9/0/0 |
| native-image-bounded-family-check FAIL101 | New fixture treated ExecutionRequest::run's optional lease argument as MapKernelContext; fixture implementation was wrong | Follow the existing nested run_native_with_checkpoint pattern and retain the exact typed inner stop plus bounded-slack assertion. Corrected check passes; bounded-family public courts pass9/0/0 |

The current-resume lane list is admission-data-lanes.md (23 terminal lanes).

| Failure/evidence | Root cause and wrong side | Fix / final state |
|---|---|---|
| native-image-selector-check FAIL101 | Our new selector owner incorrectly passed OriginalResultMemory where reserve_in_scope demanded a resource lease; product implementation was wrong | Corrected scoped check passed; final implementation uses ExecutionRequest::reserve_retained_memory, retaining its classified inner stops and explicit native-only accounting mode. Final native-image-probe-check passes |
| native-image-inorder-selector-check FAIL101 | Our new public fixture passed &&str into ClientKey::raw, whose Into<String> contract takes &str; fixture implementation was wrong | Pass *family without changing the original assertions; corrected check and affected public courts pass2/0/0 |
| admission_binary_encoding_review REJECT, medium | Our byte probe inherited Serde's human-readable mode instead of the actual MessagePack mode; product codec was wrong | Carry the actual flag; fixed_byte_probe_obeys_the_underlying_serializer_mode passes in corrected codec lane3/0/0 |
| admission_selector_data_review REJECT, medium | Our borrowed-field recursion scanned complete child subtrees again at each ancestor, causing n-log-n marker work and missing traversal charges; product algorithm was wrong | Replaced with exact framing plus streaming DeserializeSeed; bounded outer roster scans are explicitly charged. Streaming public lane passes7/0/0; final in-order affected courts pass2/0/0 |
| Self-review: ancestor-bound text charging | Carrying ancestor key bounds could repeatedly charge a long ancestor's text despite the streaming marker fix; product cost model was wrong | Use one in-order predecessor. Each family is compared at most twice; three genuine original publications exercise both sides of the balanced selector |
| Self-review: Serde collect_str default | A rejected probe could format/allocate a temporary string via the default trait method; product adapter was wrong | Reject collect_str without formatting and forward the actual serializer's method; fixed_byte_probe_does_not_format_nonbyte_tuple_values passes1/0/0 |

All executed test cases in these resume lanes passed. This is not a full-task
green claim: operational admission and the remaining queue are unfinished.

All lane paths/counts are in checkpoint-admission-lane-index.md. This ledger
distinguishes product, test and orchestration faults; open work remains P-A's.

| Failure/evidence | Root cause and wrong side | Fix / final state |
|---|---|---|
| admission-start-root-check-tests | Three World court calls omitted the actual Native argument required by the merged retirement contract; court was stale |2fb3d638e1 supplies that actual owner; issued Root check passes |
| admission-start-query-check-tests | Raw command omitted WORTH_IMPLEMENTATION_BUILD_ID; orchestration was wrong |Use existing scripts/application_build.py issuer; issued Query check passes; product env! contract is unchanged |
| held-native-courts-check | New court used unwrap on a custody-preserving refusal without Debug; test formatting was wrong |Print the actual reason via unwrap_or_else; corrected Store check passes |
| Initial missing-initial-input / WorldSectionRequired cases | Fixtures attempted existing Home admission without genuine completed initialization, or attempted Native-only publication where a coupled World owner is required; fixtures were wrong |503e31d904 initializes real Homes and uses the genuine coupled owner; two construction cases and original-budget catalog case pass |
| held-native-courts-first stale-capture case | Fixture retained a prepared Store publication inventory fence while trying a second preparation; fixture staging was wrong |766cf768e7 captures Native first and frames it through the actual coupled owner later; stale-capture court passes; the real fence is unchanged |
| held-native-courts-first foreign-runtime case | Expected ordinary capture denial although Complete rejects original authority earlier; expectation was stale |Assert exact Native Capture OriginalAuthorityUnavailable and unchanged selection; release substituted owner before source close; court passes |
| held-native-courts-first ACK case | Coupled reopen performed reads; the later Native-only probe still sees five member reads where the unchanged assertion requires zero; Native image admission remains absent |Unfixed. held-native-courts-three-rerun is2/1/0. Preserve the assertion, counters and custody. This is P-A's reopen work |
| native-image-producer-check-tests | Producer attempted Clone on ExecutionArray and omitted a new capture-denial arm; implementation was wrong |Share the actual funded parsed backing through Arc and preserve typed image stops through callers; corrected Root check passes |
| First independent producer review | New producer used consult-only serialization/copy/scans; product design was wrong for original-request charging |4215bd9518 adds explicit charges before bounded units; work court passes |
| Second/third independent producer reviews | New image reused runtime-affine predecessor/root/output DTO representations; new format was wrong for D17 |Logical durable references replace predecessor/root/Empty references and output-origin/retirement sources; independent cross-instance, genuine publisher and pure codec observations pass |
| native-image-query-callers-check | One exhaustive capture-denial conversion was missed; caller migration was incomplete |8e06ce9e12 preserves boxed typed AdmissionImage denial; corrected Query and Store checks pass |

Canonical originals use reserved canonical-frame identities. The corrected
projection issue is the runtime-affine representation in the new image, not a
proved leak of a live runtime allocation ordinal. The landed canonical format
is unchanged. No failure is dismissed as someone else's debt.

Untested/unimplemented requirements remain in the lane index and progress note.
The bounded-tail historical156-body result is still not closed by this producer
step. Genuine Home codec-retirement image admission remains unrun; its pure unit
does not claim an actual retirement happened or that recovered custody exists.
