# Query9.19 Phase4: historical candidate relevance boundary review

Status: boundary review in progress; no Rust changes for this slice. Previous
conservative correction slice is committed as56bfbd8639 and the worktree was clean.
The implementation-batch stages remain ordered: complete this review, save its
plan, implement, verify, obtain all three independent gates and commit.

## Next obligation

Conservative correction currently requires every Native Source-scope candidate
to cancel. This safely preserves no-ops/transient entrants but falsely refuses
uncancelled edits to rows outside the admitted provider predicate. Original and
target candidate intersection cannot solve that problem: an intermediate row
may enter and leave the range. The next vertical must consume actual per-commit
before/after Source material and the existing installed forward predicate under
the original controls. It must not infer history absence from current equality.

This is candidate-level historical relevance. Full candidate bodies (including
exact-refinement rejects) remain the influence unit. It does not promise minimal
field-level tracing, removal of every shared-row neighboring-scope false positive,
public historical READ or closure of all Phase4 profiles.

## Verified owners and actual material

Native authored_history/preparation.rs retains real authored record provenance,
relation incidence and complete before/after inverse impact row sets by actual
frozen Source catalog slot/full-generation scope. It includes authored no-ops,
whole-aspect operations, physical changes and transient membership. The packet is
bound to the final canonical envelope with its actual self-Weak; deep cloning,
serialization and cold decoding cannot recreate it. Cursor/page issuance already
requires that genuine packet and exact selected-prefix continuity.

NativeManagedSourceInterpretation owns the actual admitted physical_basis and
PreparedManagedIndexSources from the same applied proposal. Each actual
PreparedManagedIndexSourceRoot already contains program, full-generation scope,
committed/proposed Arc<dyn Any> payloads and physical availability. Native checks
the complete set before rules/publication. Query's TypedSourceService constructs
those payloads from actual Native Source loans and the installed adapter. They
are pure immutable ManagedSourceRoot images: persistent rows, provider nodes,
actual installed index/adapter, binding catalog and original allocation. Their
Entry occurrences carry actual program/scope/full-generation rows and projected
one-hop bodies; they do not retain a Native branch snapshot or public READ grant.

The actual selected Native root also owns a ManagedSourceRootImage containing
its actual cell affinity and ManagedIndexBranchImage. The branch image retains
opaque scoped payloads by real SourceKey. This can supply unchanged Source
material for authored no-ops when semantic Source preparation has no changed
records. Scope birth/retirement, absent payload and cold material must remain
explicitly distinguishable. A missing prepared root alone does not prove absence.

The final canonical freeze currently receives only the prepared impact packet.
A justified next handoff must receive the actual interpretation owner as well,
retain genuine scoped before/after payloads plus actual cell/program/scope binding,
and bind them within the same live envelope sidecar. Do not keep the entire
physical basis/branch root: that would risk a canonical-envelope/root retention
cycle. Do not copy a canonical plan, invent a nonce/digest, add a global registry,
or issue a new Source audience. No new public source/root constructor is allowed.

Query CapturedIndexInfluence retains the real Native observation, installed
adapter, original complete parameter set/Parent, passive audience and verified
coverage. Its erased retained-influence owner is the consumer for narrowing;
a generic correction history helper must not interpret provider values.
PersistentSourceRows can look up actual full-generation affected entries, and
source_rows already creates scoped typed provider loans through controlled decode.
The actual installed provider has matches_source(row, admitted_probe, control),
the same forward predicate used by complete candidate traversal. Extend that
installed adapter rather than introducing a second predicate implementation.

## Required handoff and failure meaning

A genuine Native observation plus canonical occurrence can ask the Native owner
for its own scoped historical impact. The owner must recheck live envelope,
actual frozen catalog, runtime and actual cell affinity/program/full-generation
scope. A packet that has no impact for this selector is genuinely Unaffected.
An impacted packet with unavailable material is Conservative; it remains selected
for cancellation, never becomes empty/irrelevant. A complete loan owns no public
READ and supplies actual affected rows and immutable before/after material only.
Foreign correspondence and original control failures retain typed denials.
No reconstructed receipt/ID tuple may create this loan.

Query may discard a managed occurrence only when every corresponding retained
managed fact proves no actual affected entry on either real side matches its
actual installed forward predicate. Test both sides separately with original
controls and retain all matching candidates even when final exact results are
empty. A matching authored no-op remains relevant despite equal bodies. An
intermediate entrant must be caught from its own historical after/before image.
Unsupported/cold/missing historical material takes the visible conservative path;
control errors must propagate, not be swallowed into relevance/irrelevance.

Ordinary authored occurrences remain an independent stream. Filtering managed
history must not erase an occurrence already selected by ordinary field/aspect/
adjacency facts or by another managed fact. Deduplicate only after this union.
Unchanged performed-cancellation and strict-original/complete-target comparison
follow selection. Current disclosure/principal and exact target publication stay
in their existing owners. Governance passive influence supplies consistency only.

## Facts still to resolve before the implementation plan

- Define the narrow actual-cell binding/retention helper inside the existing
  Native managed_source owner; no private affinity escape or branch-root cycle.
- Distinguish genuine scope absence from cold/unavailable payload on both sides,
  including no semantic Source preparation for authored no-ops. Prefer actual
  scope existence metadata captured during paid physical preparation.
- Establish a funded immutable live-material map attached at freeze from the
  actual interpretation; actual prepared roots win, unchanged selected material
  is usable only under complete Source-preparation closure.
- Verify every variable-width material-map lookup, affected row traversal and
  provider decode/match uses original work/memory and bounded map lookup costs.
- Verify SourceImageView/slot issuance for retained historical entries stays in
  the controlled passive consistency owner and creates no public source session.
- Select strong same-fixture courts: uncancelled irrelevant edit success; relevant
  raw restore/transient failure; matching authored no-op on an exact-rejected
  empty Parent remains blocked until actual recorded cancellation; ordinary
  disjoint-publication behavior survives. Preserve Native gap/foreign controls.

Use existing external Native/consumer targets, default Rust1.99.1/shared target
and sccache. No runtime src court, test feature, baseline exemption or new root
family. Settle APIs/fixture declarations before expensive builds. Current prior
receipts remain valid for56bfbd8639; no pass is claimed for this unimplemented slice.

## Review resolution before planning

The independent reviewer found no architectural blocker with the following
narrowing. Complete classification requires BOTH genuine payloads present and
available. Every missing side, cold cache, scope birth/retirement or unsupported
image remains Conservative. No missing side becomes an empty Source, and no new
scope-existence representation is needed in this slice. A completed global
PreparedManagedIndexSources stage plus exact program/scope closure exclusion is
required for unchanged selected material supplying both sides. Missing stage is
not evidence. Use actual prepared roots when present; reject inconsistent scope
closure, wrong typed/installed program/adapter or foreign affinity distinctly.

Only Public Parent posture may use this first predicate-relevance filter;
Governed retained facts retain the existing conservative history posture. Actual
Parent disclosure meaning supplies the gate, not a digest. Exact refinement is
never the history filter: matching coarser candidates remain selected even if
refinement rejects their final DTO.

The existing Native external owner currently publishes an opaque unit cell.
Extend that SAME owner to publish the real managed Source image through
prepare_managed_source_image and seal_managed_source_replacement, retaining its
actual admitted Source cell slot across publications. Its actual preparation
already derives Vec<EntityId> roots from Native membership loans. This allows
real observation/affinity/sidecar tests without a new SDK/root family. Preserve
first-publication pending/admitted custody and typed preflight refusals; no new
src runtime test or root factory.

Native selected-image borrowing can remain local to managed_source/history's
owner: copy only the existing affinity Arc, actual program/scope binding and
scoped payload Arc. The transient basis/image is dropped before the sidecar is
frozen. Query controlled decode already owns first/final original stop behavior;
use that existing invocation owner, ordered lookup accounting and exact native
full-generation row IDs for both historical SourceImageViews.

All before-plan material unknowns now have conservative explicit dispositions.
The resulting plan may proceed; this does not claim implementation or runtime
evidence. The reviewer specifically required unrelated nonmatching edit success,
matching no-op with exact-rejected empty DTO, intermediate entrant retention,
ordinary-stream preservation and unavailable-material Conservative evidence.

## Lifecycle discovery invalidating strong sidecar custody

Independent review identified actual canonical Arc release under the residency
mutex and branch-root reclamation reached through Query's Native runtime loan.
A strong opaque Any payload in the canonical sidecar could move its last destructor
under those gates. Unlocking one residency mutex is insufficient. Deferred
diagnostic artifacts are not a generic opaque disposer. The two new Native
material courts passed, but their Vec payloads do not qualify this destructor edge.

Revise before further Rust: canonical material must use SDK-funded weak receipts
only. Original Source image/prepared-root owners retain strong data. Weak control
allocation funding must survive expiration, accounting for exact dynamic outer
Arc layout/alignment rather than recursively freed heaps. Create the receipt from
an actual borrowed payload while its owner is alive; do not create an owned
selected-payload temporary that could become last strong during concurrent
cache eviction. The selected directory receiver must create the weak receipt
under its own guard, without moving opaque ownership out through that guard.

Native history acquisition must return sealed OWNED metadata/weak custody only,
never upgrade opaque payloads under its runtime call, including failure/final-poll
paths. An explicit historical-payload upgrade API must carry funding and require
its caller to hold/release strong shares outside runtime/attempt guards, as cache
retirement already requires. Expiration is Conservative. Warm actual selected
before custody is usable only after actual committed-side eligibility is proved;
committed=None remains Conservative.

This foundation can be qualified in the existing external Native target before
Query integration. The partial Query matcher is not valid until its outer
outside-guard custody boundary is resolved. No new global disposer or unowned
weak allocation is acceptable. The native and Query stages may be committed
separately if that is the coherent foundation-first boundary; Phase4 remains open.
