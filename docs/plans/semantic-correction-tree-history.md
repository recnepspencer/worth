# Retained reapplication and correction alternatives

This plan implements Phase 3 of [9.18](../../plans/WORTH-query/milestone-9.18.md)
in the public repository. Phase 2 is certified and closed in `420489ac1c`;
the existing independent source and evidence reviewers have cleared this plan.
Phase 3 is certified and closed after the native, public tree, required-output,
compile-contract and affected Bank checks passed and independent source and
evidence review cleared the implementation. Required repository enforcement
passes. The governing specification linked above owns current milestone progress.
Private consumer work is excluded.

## Boundary review

The existing correction source owns an exact World ancestry selection and the
performed Query occurrence. Its original authority carrier retains the actual
encoded source input, installed interpretation and disclosure parameters.
Selection and preparation already perform current disclosure; ordinary mutation
admission, handler execution, World publication, required outputs and recovery
remain their respective owners' responsibilities.

The existing Relational aftermath fact records the correction role, exact source
World and Relational coordinates, actual parent and performed child. Its key
index resolves a caller's correction command. That index establishes retry
identity, not whether another command already reversed or reapplied the same
occurrence. No current public alternative selection or retained reapplication
entrance supplies the missing checks.

The provisional redo policy requires the old undo head and principal lineage.
It cannot implement fresh admission after intervening ordinary edits. Its useful
authority, input-integrity and custody tests must move to the accepted path;
its linear-head policy must be removed. Final facade/caller cutover belongs to
Phase 4.

World owns ancestry, exact target comparison and composite publication. Its
whole-catalog publication page is a reconstruction surface, not an ordinary
correction lookup. Relational owns canonical causal facts and their derived
indexes. The current exact entity-field index lookup has a finite result limit
and overflow indication but no continuation. Pagination therefore needs a small
extension in that existing native index owner, not a Query history scan or a
second causal registry.

## Exact alternative and active reversal

Keep four meanings distinct:

- **Lineage root:** the original ordinary source occurrence, inherited through
  validated committed correction facts.
- **Operated source:** the exact occurrence being reversed or whose retained
  input is being reapplied.
- **Selected alternative:** the historical performed reversal chosen by the
  caller, supplying its original source meaning and provenance.
- **Active reversal:** the latest unconsumed reversal on the selected target
  ancestry, supplying the one-use applicability condition for reapplication.

For source S, reversal U, reapplication R and reversal V, both U and V remain
historical alternatives. Choosing U under active V records U and V separately
in the new reapplication fact. R's earlier consumption of U stays immutable.
The new command consumes V, regardless of which alternative was selected.
Two distinct choices cannot each consume V once. Reversing a performed
reapplication names that new occurrence, not the old source S again.

An exact retry resolves the selected alternative and active reversal recorded
with its original command. It never substitutes today's active reversal into
that command's meaning. First-use eligibility and replay resolution remain
distinct, while both require fresh request admission.

Consumption follows exact inherited target ancestry. A fork made after
consumption inherits it; a fork made before consumption may independently use
the reversal. An ordinary edit neither opens a new reversal period nor clears
alternatives. Same-key retry resolves its existing command before first-use
eligibility checks; a different key cannot repeat the effect in the same period.

These are immutable canonical relationships. The runtime derives their current
applicability from native facts at the selected target. No mutable redo cursor,
global consumed flag or caller-supplied lineage identity participates.

Index reversal facts by lineage root and by exact operated source. Index
reapplication facts by the active reversal they consumed. During bounded root
iteration, an exact consumption lookup classifies each reversal. A complete
traversal must establish at most one active reversal; two indicate inconsistent
canonical relationships. Before completion the active state remains unresolved,
and preparation cannot proceed. A source's existing reversal prevents reversing
that old occurrence again even after its reversal has been consumed. The next
reversal must name the new performed reapplication occurrence.

Posting order is native entity identity order, not causal order. Pages must not
infer the active reversal from their last row. Relevant-lineage traversal is
explicitly bounded and may continue; unrelated lineage rows are excluded by
the root index. This requires no mutable root-state row or new authority.

## Result for callers and authors

The same correction declaration may explicitly enable reapplication of its
original `Source` binding. It supplies typed conventional/capability dispatch
and conversion of the new command key to that source binding's key type.
The original input has no replacement setter or transformation callback.
Installation resolves and retains the source action's complete supported
meaning, including its delivery, and canonical encoding includes that support.

The caller progression is:

```rust,ignore
let request = application.request(&principal, &scope).on_branch(branch);
let source = request.corrections().select(&source_receipt, history_limits)?;
let mut page = request.corrections()
    .alternatives(&source, history_limits, maximum_entries)?;
let mut alternatives = page.entries().to_vec();
while let Some(continuation) = page.continuation() {
    let next = request.corrections().continue_alternatives(
        &source, history_limits, maximum_entries, continuation,
    )?;
    alternatives.extend_from_slice(next.entries());
    page = next;
}
let alternative = page.select(choose_alternative(&alternatives), history_limits)?;
let preparation = request.corrections().reapply(&alternative)
    .idempotency(client_key)
    .limits(history_limits)
    .prepare()?;
```

The compiled tree example exercises this progression. As with reversal,
source-delivering and required-output actions retain their ordinary typed
preparation and completion owners. Historical selection supplies meaning;
fresh admission supplies current authority. Missing support, expired material,
foreign ancestry and consumed applicability return distinct typed refusals.

Every page freshly checks disclosure. A continuation binds its native owner,
exact basis, index generation and selector. Expiry or changed support is an
explicit stop, never an empty successful list. Dropping pages and selections
releases optional pins without erasing canonical relationships or mandatory
publication/output recovery obligations.

Native history retirement supplies the existing optional-source availability
boundary. Test its actual unavailable outcome and surviving permitted causal
metadata. Index reconstruction restores derived access only; it cannot restore
expired source material. This phase adds no age policy, physical-erasure claim
or new retention service.

## Owner changes and implementation order

1. Extend declaration's `application_aftermath/correction_binding` with typed
   source reapplication support. Extend installation validation, canonical
   meaning and the existing package archive together. Unsupported bindings stay
   unavailable; malformed or mismatched support refuses installation.
2. Extend the existing canonical aftermath fact and schema layout with exact
   lineage, selected-alternative and active-reversal relationships. Register,
   bootstrap and incrementally maintain the relevant native indexes through
   the ordinary inventory. Establish first-use reversal and active-reversal
   consumption before effects; World comparison closes competing preparations.
3. Extend Relational's exact entity-field posting lookup with bounded pages
   and an opaque native continuation. Query's correction history projection
   uses that owner API over relevant lineage facts. Missing indexes return an
   explicit stop; the existing separately admitted index reconstruction owner
   restores discarded indexes. No whole-history fallback is introduced.
4. In execution's `primary_graph/product_operation/correction_source`, select
   original retained input plus the exact historical and active reversal
   evidence. Resolve the source action in the admitted current program and
   compare retained supported meaning. Preserve current disclosure, workflow
   disposition and external custody requirements.
5. Factor publication's existing correction preparation over the selected
   action binding. Reversal uses the inverse binding; reapplication uses the
   original source binding. Both use ordinary fresh admission, candidate
   construction, publication and output/recovery ownership. Include action,
   source, selected alternative, active reversal and target incarnation in
   immutable command meaning; no input or target substitution on retry.
6. Extend the existing numeric certification example with real tree journeys
   and migrate useful provisional-policy tests. Update the authoritative
   correction guide and ordinary front door to teach explicit alternatives.
   Retain the existing inverse/composite/Bank evidence for unchanged owners.

Each module retains one responsibility: declaration describes support,
installation validates it, Relational indexes canonical facts, execution
selects/adopts exact meaning, and publication exposes legal caller progression.
Use child modules for posting pagination, canonical lineage access, typed
reapplication dispatch and public selection/preparation as needed. No new
runtime or broad correction service is required.

## Evidence and closure

The new working journey must perform S, reverse it, preserve a disjoint edit,
select a reversal, reapply the retained original input and reverse the new
occurrence. It then lists two alternatives and successfully chooses the older
eligible alternative. Independently calculated source values, World parents,
causal identities and ordinary source/output completion prove the result.

Focused controls cover distinct-key duplicates, different alternatives competing
for one active reversal, stale preparation, forks before and after consumption,
foreign siblings, fresh authorization, changed source support, unavailable input,
page continuation/invalidation and lawful retention expiry. Drop handles and
discard native derived indexes; use existing explicit reconstruction, then
recover the same permitted alternatives from canonical facts. Preserve typed
missing-generation stops in the ordinary lane.

Compile-time cases protect private selection construction, report-as-permit,
one-use preparation, replacement-input rejection and retained-read mutation.
Use the established grouped compile lane and current public example. Native
posting tests prove exact owner/basis/generation/selector binding and complete
bounded iteration; Query tests prove the downstream authorization and lineage
contract. Avoid repeating adequate unchanged recovery or process portfolios.

Coordinate one incremental Cargo build owner. Use the existing fast development
profiles and target directory, unrestricted build jobs, the existing immutable
build identity and `RUST_MIN_STACK=67108864`. Batch affected compile work and
run focused selectors from the resulting binaries. Finish required boundary,
agent-context, scoped formatting, line-cap and whitespace checks, then obtain
the existing independent reviewers' source and evidence clearance. Phase 3
closes only after every obligation above has actual evidence.
