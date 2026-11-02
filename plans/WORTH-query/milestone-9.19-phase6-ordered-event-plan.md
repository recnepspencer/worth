# Phase6 Native ordered-event membership implementation plan

The [boundary review](milestone-9.19-phase6-ordered-event-boundary-review.md)
precedes this plan. Implement the real Native recipe/source/changed-row receiver,
not a private live-set cache. Query descriptive authoring/lowering follows later.

Native callers will register four nodes: three Stored leaves followed by
`OrderedEvents { predecessor: 0, introduced: 1, retired: 2 }`. Existing exact
registry handles and managed Source programs consume it. These indices describe
meaning, never current physical chronology or read authority.

1. Append the portable node and typed InvalidEventChain denial. Extend normal
   grammar validation/reachability, catalog leaves and canonical tag9, preserving
   all existing tags/bytes. Registration rejects general/nonphysical inputs and
   endpoint gaps. Recovered meaning uses the same validator.
2. Add derived_relation/ordered_events as the semantic owner. Its acquisition
   owns per-call funded physical-leaf support; chain.rs uses the existing Native
   frontier and actual raw rosters; membership.rs owns newest-first fold and the
   private validated inverse-candidate handoff. Evaluation integrates every
   entity/relation/empty adjacency into its existing support and memo result.
3. Add ordered_events/affected.rs for both-image nonmonotone scheduling and full
   changed-row cohorts. The existing affected/expression owner dispatches this
   case explicitly. Do not fake Compose or mint a temporary registered handle.
4. Extend the existing managed-source Native court with a small generic event
   chain (no CAD), authored rosters and actual provider/committed/proposed reads.
   Keep actual runtime/request construction in its registered existing root.
   Cover ordered retirement/reintroduction/same-step semantics, inverse agreement,
   complete negative support, before/after deletion/rewire and removed rows, plus
   malformed parallel/branch/cycle refusal and original cancellation/exhaustion.
   Focused pure mechanics may use literal physical records; observation custody
   claims require genuine installed Native issuance. Include literal old encoding
   and malformed/recovered definition fences at their existing owner if needed.
5. Run cheap formatting/caps/source topology first, then build one existing Native
   court target and run the selected managed-source cases. Use an existing owner
   binary only if local algorithm evidence cannot honestly fit that court. No new
   root, flag, profile, workspace sweep or unchanged heavy growth repetition.
   Run actual boundary/context tools and strict text/new-link/default diff checks.
   Resolve concrete independent qa-loop/qa-tests/code-quality-qa findings, record
   actual evidence and commit the coherent slice.

Resource meaning stays with original SourceRead controls. Prefix union caching
is per immutable-side invocation, not persistent sharing or a locality claim.
All support survives until successful whole acquisition or disposal. Refusal
returns no partial data. Exact original faults retain their categories.

Out of scope: Query authoring/archives/lowering, private codec/profile/COW,
SDK adoption, actual CAD contact/topology/face-scale/Acceptance8 and final clean
private loop gate. They remain mandatory subsequent9.19 work.

Implemented receiving refinements: actual optional passive changed-row recording
is enabled only in the new existing-root Source fixture, proving removed and
continuing rows reach warm maintenance cohorts. Native rule groups are explicitly
schema plus relation integrity as resolved in the boundary review. A real
32,768-work short/129-event twin reaches the bounded reader and refuses incomplete
data without moving the head. Strict Native identity cannot issue parallel
same-kind/same-endpoint records; no forged parallel runtime fixture or parallel
court pass is claimed. Raw-record multiplicity checks remain in the implementation.

Canonical identity is exercised through the ordinary registered registry digest:
swapping only introduction/retirement indices changes identity with identical
physical leaves. Legacy Stored JSON has a literal wire fence and decoded bad
recipes use normal registration denial. The standalone snapshot digest writer is
not an exported facade; no test-only export or extra owner binary was added to
reach it. Existing canonical tags are unchanged; new OrderedEvents tag is9.
