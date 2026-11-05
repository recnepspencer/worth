# Complete the retained-index image consumer

Fresh independent full-slice review found reconstruct_for_commit still insists
on resident recorded_commit_envelope and root.canonical_envelope. This is a real
consumer cutover omission; image roots permanently carry Checkpoint evidence.
Query primary_graph/index_refresh.rs calls this API for non-head commits. The
working analogue refresh_for_basis already accepts actual root evidence.

Use the actual immutable catalog artifact's receipt identity to select the same
retained root. Require requested commit/authoring branch/version and exact root
descriptor against that artifact, and root authoring evidence. No frame demand,
synthetic envelope, catalog-latest lookup or new authority. Pay branch scratch
and comparisons from the invoking maintenance work; preserve existing cold,
work and memory limits and atomic generation publication.

Add a public Native image court retaining an older root across genuine Complete
capture/recovery. Explicit reconstruct_for_commit must cold-build that root with
finite slot allowance, return the older field value and not newer-head entries,
refuse a foreign requested branch, and read zero covered canonical bodies.
Keep existing cold+incremental court unchanged. Run focused Native/court check,
new case plus original retained reconstruction and slot-reuse cases. No Query
caller edits, features, facades or new test doors. Then a fresh independent review.

The new court exposed a second real gap before cold reconstruction: recovered
pinned component roots need not appear in retired-head locators, because this
receiver never performed their original head transition. Do not fabricate a
retirement or add another live root table. Use the existing weak admitted-basis
registry for this explicit reconstruction only. Snapshot its Weak entries into
an intrinsically request-funded array, pay scan work before copying, release
the registry lock before any upgrade/drop/projection, and select only a matching
actual basis with the SAME current retention owner. Project its existing
observation; this clones its actual pin and grants no currentness. Preserve
live-head exclusion and existing retired-root path for other historical roots.
No ordinary exact-key lookup or retirement/fencing state changes. Scan cost is
explicit reconstructive work, bounded by maintenance work and original memory.
Private resource-accounting callback carries no business/authority decision.
Additional files: branch/{basis_registry,reference_root_access}.rs; maintenance/
retained_projection.rs and shared typed buffer-stop conversion in basis.rs.

Concurrency refinement before qualification: payment/array allocation occur
outside the registry lock; reacquire only for fixed-capacity Weak copies. If
concurrent growth exceeds the paid capacity, return typed Changed and map it
to SnapshotUnavailable. No retry/unbounded scan, owner callback, strong upgrade
or strong-owner drop under the registry lock. Private snapshot stop is exported
only within Native's branch facade; it is not a new public construction door.

Fresh source review rejected two real snapshot defects. Count shrink would
misclassify an incomplete fixed array as allocation failure, and len-based
payment underpaid HashMap high-water bucket traversal. Correct with separate
paid element/bucket shape: pay actual capacity, allocate exact len, reject any
cardinality change or unpaid capacity increase under the copy lock. Value-only
shape/seal predicate test covers shrink/growth/rehash with no runtime roots.
A public Native sparse-registry court creates65 actual simultaneous admissions,
releases all but one, and requires a32-unit request to refuse before cold work
or generation publication. No private observer or new authority/test door.
Historical reconstruction court moved along its responsibility into
checkpoint_admission/retained_indexes.rs to keep both files below400 lines.

Final source review found root() accidentally cloned unrelated mutable branch
and fork provenance names. Correct with narrow Arc-only root access under its
state guard. Separately pre-pay registry identity, fallback name, and exact
visibility-basis name before copying; no source-name cost enters a short fork.
Fresh source re-review APPROVE; focused qualification follows.
