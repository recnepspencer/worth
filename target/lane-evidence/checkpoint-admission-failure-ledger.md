# Failure attribution and resolution

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
