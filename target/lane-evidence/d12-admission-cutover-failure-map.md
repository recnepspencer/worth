# D12 operational cutover failed attempts

All rows describe this dirty slice based at15730e6b76. Full logs and exact counts
are indexed in d12-admission-cutover-lanes.md. No failed result is relabeled green.

| Failed lane | Root cause / wrong side | Correction and focused result |
|---|---|---|
| d12-root-evidence-check | Missing internal evidence export in product; rustc also reported an incremental fingerprint ICE. | Export corrected; subsequent owner check green without cache deletion or compiler/profile overrides. |
| d12-region-owner-format | A child of an existing path-mounted module needed an explicit source path; product module mount wrong. | Explicit checkpoint_root path; later formatting and owner checks green. |
| d12-region-retention-check | Standalone expanded checkpoint still used kind binding after it became test-only; product caller wrong. | Moved caller to the actual request-funded binding; later checks green. |
| d12-region-expanded-check | New court treated a public basis as a receipt; fixture wrong. | Observe the public descriptor target instead; no product behavior changed for this failure. |
| d12-symbol-owner-check | Fixture glob imports made foundational and Native PartitionId ambiguous; fixture wrong. | Explicit Native imports; corrected check green. |
| d12-native-operational-public-court | Branch-cell restore still demanded the original envelope for an image root; product wrong. | Root evidence cutover, preserving actual cell roots and schema. Final operational court green. |
| d12-native-operational-stable-court | History synchronization still demanded an envelope instead of the actual image root/catalog linkage; product wrong. | Preserve typed checkpoint root evidence and source linkage. Final operational court green. |
| d12-operational-and-memory-courts | Field-patch fixture was incompatible with the public whole-struct creation reader; fixture wrong. | Use an actual whole-struct authored schema and creation; value assertion retained. Memory court already passed; final pair green. |
| d12-funded-creation-courts | Complete recapture treated source metadata as Journal-only and lacked a cold indexed head; product wrong. | Use exact source placement and actual indexed receipt facts. The unchanged second-Complete assertion passes in the final operational court. |
| d12-native-image-consumer-courts | Reconstructed publication routes lost their durable position floor and proposed position1; product wrong. | Restore floor from the actual indexed head. The final operational court commits again successfully; the other9 courts had passed. |
| d12-store-selection-and-ack-courts | Owned prerequisite validation required an existing feature absent from selected native package roots; runner selection wrong. No tests executed. | Select recovery package alongside application court, letting committed manifests derive the same feature graph. d12-store-selection-and-ack-graph-courts passes2/0/0. No feature override or new feature. |
| d12-package-refusal-court-check | New fixture called unwrap_err on a success owner without Debug; fixture wrong. | Match the public Result explicitly; corrected check passes. |

Independent-review design/funding findings and their resolutions are recorded in
d12-operational-review-record.md. Final qualification/gates and full-slice review
are still pending; this document is not completion certification.

| Later failed lane | Root cause / wrong side | Correction and focused result |
|---|---|---|
| d12-host-filesystem-reopen-courts: old-key pair | Receipt repair demanded resident original after actual checkpoint receipt reconstruction; product wrong. | Actual bound CheckpointReadmitted metadata repairs the receipt without original-frame demand; d12-host-reopen-failed-controls-corrected passes the pair. |
| d12-host-filesystem-reopen-courts: bounded writer | One60-second authentication scope was reused across37 distinct requests; fixture wrong. | Fresh scope per request, no timeout/workload/bound increase. Deadline failure cleared; checkpoint-work refusal remains below. |
| Native index compile | RecordRef was assumed Copy; product caller wrong. | Borrow and dereference, without trait weakening; later owner check green. |
| New incremental-index court | Cold prior generation had not been admitted before a zero-cold-work incremental refresh; fixture wrong. | Admit prior through actual public pin/read; zero-cold allowance and assertions preserved. d12-final-image-consumer-courts passes4/0/0. |
| d12-phase-dirty-line-caps | Windows Python resolved a WSL bash lacking/bin/bash; launcher wrong. | Explicit Git Bash runs the unchanged guard; d12-phase-dirty-line-caps-git-bash passes125 files. |
| d12-host-reopen-failed-controls-corrected, d12-host-bounded-maintenance-diagnostic, d12-owned-image-bounded-host-court | Checkpoint preparation exhausts the actual8m work budget; product work remains too high. | Removed actual duplicate Store emission and Native repeated hashes/parsing. Latest bounded result is still red; current-source acceptance not yet run. No grant or assertion widened. |
| d12-single-emission-public-courts | Accounting court required repeated operations removed by the single-emission design; test expectation wrong. | Count every actual serializer/hash/copy/scan/replacement, retain all tampering assertions and ceilings. Initial6/1/0; corrected accounting rerun passes1/0/0. |

The latest d12-single-emission-bounded-host-court remains0/1/0 at the same
C12/head41/WorkExhausted observation. d12-bounded-capture-work-measurement is
also intentionally0/1/0: after the assertion fails, same-owner capture succeeds
under its existing bootstrap lease and reports16,883,822 charged units and
1,323,394 package bytes. No resource policy change has been made. The fixture's
reused8m identity policy is insufficient; its legitimate finite replacement is
not yet derived. The Native stop itself is correct; silently completing would
violate request funding. Coordinator resource-contract question is pending.

Additional final-phase observations:
- Full source review P2: mandatory predecessor section copy before Home framing.
  Product boundary wrong. Separate original SectionSelectionCustody from final
  framed DATA; owning CapturedHomeRelationalSection consumes into final prepared
  owner. Fresh source review APPROVE; d12-single-framing-store-courts6/0/0.
- d12-single-framing-host-courts8/1/0: only bounded automatic maintenance fails
  at the unchanged8m request policy; explicit same-owner bill16,242,077 units.
  This remains unresolved; current typed stop is correct. No passing backfill.
- d12-phase-boundary-current: four missing exact certification callers and three
  adapter topology findings. Own caller registry omission corrected by four
  exact rows; d12-phase-boundary-exact-callers leaves only the three BC9002
  files explicitly deferred by latest task. No baseline or pattern expansion.

Later corrections (supersede the pending resource question above):
- Full review exposed envelope-only explicit historical-index constructor.
  Product wrong: use immutable catalog facts plus actual admitted basis/pins.
- New historical case first refused UnavailableRetainedTarget: receiver never
  performed original head retirement. Product owner lookup wrong; snapshot the
  existing Weak registry and project its actual same-owner pin. Case then green.
- Mixed lib/court target selection refused before tests: harness assumption
  wrong; split selected library and court targets. No tests ran in failed lane.
- Snapshot review found cardinality shrink misclassified as AllocationFailed
  and physical HashMap capacity underpayment. Product wrong; exact cardinality,
  paid bucket capacity, typed Changed. Historical/sparse public cases2/0/0.
- Source review found unfunded copied fork/name state. Product wrong; root Arc
  access avoids irrelevant state, remaining actual name copies separately paid.
  Source re-review APPROVE; latest check passes, focused reruns pending.
- Inherited ordinary8m fixture policy did not fund finite Home history capture.
  Fixture allocation wrong for this model; Home-only capacity-derived finite
  policy uses existing owner/request and preserves every automatic/read bound.
  Independently reviewed, compile green; latest nine-case proof remains due.

Final focused corrections now qualified:
- d12-retained-original-and-shape-courts4/0/0; d12-retained-public-final-courts2/0/0.
- d12-home-final-reopen-and-bounded-courts9/0/0. The capacity-derived Home-only
  fixture policy funds actual automatic capture; unchanged<=40 assertion passes.
  Ordinary8m policy unchanged. No manual checkpoint/backfill or larger deadline.
