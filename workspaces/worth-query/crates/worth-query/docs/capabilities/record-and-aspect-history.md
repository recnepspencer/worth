# Installed Record And Aspect History

Application code declares history through `worth-query-decl` and executes it
through `worth-query-host::facade::primary_graph`. The compiled facade audit
journeys and Native growth court linked below qualify this bounded capture and
paging route.

## Declare the subject and metadata meaning

Implement `ApplicationHistoryQueryBinding<Schema>` with a genuine installed
`SubjectRead: ApplicationQueryBinding<Schema>`. Add an
`ApplicationHistoryQueryDefinition::<Schema, History>::entity(..)` or
`::relation(..)` and its typed `.aspect(..)` references to the schema. The relation
target names the exact SubjectRead relation slot; it selects physical occurrences
that survive that query's filtering, ordering and disclosure. Endpoint equality
does not identify a physical relation.

History has its own `ApplicationQueryDisclosureContract`. Public metadata must be
explicitly authored Public. Governed metadata must cover every
`ApplicationHistoryMetadataFacet::ALL` value through
`ApplicationHistoryMetadataRef::<History>::new(facet)` and
`.disclose_history_metadata_by(..)`: occurrence existence, selected ancestry, and
continuation/completion. Payload permission alone grants none of these facets.
The installed contract and the ordinary SubjectRead contract are admitted as two
separate decisions on the same genuine graph session.

## Capture disclosed provenance

Select the product, resolve its principal and scope, and admit the ordinary
SubjectRead plan through the supported Host advancement phase. Pass that plan to
`runtime.admit_application_history_capture(..)` or
`runtime.admit_governed_application_history_capture(..)`, with the actual checked
request from `phase.checked_execution_request_for(runtime.product_runtime())`.
Execute the capsule with `runtime.execute_application_history_capture(..)`.
Its `into_parts()` returns the completed ordinary read and opaque
`WorthQueryCapturedApplicationHistorySubjects<Schema, History>`.

Capture sees only the final disclosed tree. Its identity is bound to that exact
completion's runtime, schema, query, original parameters, scope generation and
retained product. It resolves genuine Native record generations while the basis
is live. It cannot be constructed from row IDs, endpoint pairs or receipt display
fields. Ordinary reads continue to return their usual completion without a hidden
history pin.

Capture currently accepts complete one-shot results. Partial ordered continuation
pages and live targeted collections do not supply a history-capture route.

`WorthQueryApplicationHistoryCaptureLimits` bounds subjects, selected aspects,
source visits and newly retained backing. A source visit is either one disclosed
node examination or one exact Native record-resolution probe. Temporary subject
array and retained selection overlap is included in the storage cap. Existing
read-tree, parameter and immutable canonical-envelope custody is shared rather
than counted as newly allocated capture backing.

## Open and continue at one exact product

Retain a real `WorthQueryApplicationReadObservation` and select it on a fresh
advancement phase. Supply this selected truth and a separately selected current
security product from the current head of the **same branch** as the selected
truth to `runtime.open_application_history(..)`. Use
`open_governed_application_history(..)` when both contracts require actual typed
capability access. The corresponding `read_application_history(..)` and
`read_governed_application_history(..)` methods take the opaque captured subjects,
the same selected truth and a mutable owner-created cursor.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// The executable helpers below include principal/scope admission.
let page = runtime.read_governed_application_history(
    &phase, &history, &external, &subjects, &mut cursor,
    current_security, &selected_truth, payload_access, metadata_access,
    &request, limits,
)?;
for event in page.entries() {
    inspect(event.commit_id(), event.version_id(), event.branch_id(), event.record());
}
let complete = page.is_complete();
```

Every call reissues original-source permission against the selected truth and
current security. Revoke either payload or metadata access and the call refuses
before capture or page-limit diagnostics. Regrant and freshly admit access to
continue. Captured provenance grants no read authority. A stable container scope
can admit history for an already deleted selected subject; a replaced scope
generation or later record occupant cannot lend identity or disclosure.

`WorthQueryApplicationHistoryPageLimits` bounds entries, source visits and newly
owned page/continuation backing. Each page performs one World parent step **or**
one bounded Native canonical-record chunk, never both. An empty page may be
incomplete. Continue until `is_complete()`; total work can grow with retained
ancestry and examined canonical records. This API promises no constant-time or
record-local historical lookup.

World supplies exact raw product ancestry and component publication evidence.
Forks inherit their actual parent, unchanged retained components add no duplicate
Native event, and a genuine World root includes its one imported canonical
component. Traversal does not walk Native ancestors before that World root.
Missing raw parents return the actual
`World(ProductExactHistoryDenial::History(CompositeAncestorSelectionDenial::RetiredHistoryGap))`
cause rather than splicing to an older protected commit.

Entries borrow genuine commit/version/branch and record identity, selected changed
aspects, structural change and opacity facts. An equal-value authored Updated
fragment with no surviving canonical aspect operations remains an occurrence with
an empty `changed_aspects()` iterator. It does not attribute an aspect write that
the canonical fragment no longer records. Actor attribution, before-images,
inverse meaning and automatic successor traversal are not inferred.

Wrong runtime, captured source, filter or pinned root refuses. Cancellation and
disclosure refusal preserve the last accepted cursor; only the final original
request handoff installs a successor. Drop the page, cursor and captured subjects
to release their actual Native/World custody. These owners are memory-resident;
they are not serialized restart capabilities.

## Executable audit panel

The existing `application_graph` target contains the facade-only
[physical relation and deleted generation journey](../../../worth-query-court-host-certification-fast/tests/worth_query_court_host_certification_fast/fast/certification/application_graph/relation_laws/history/court/mod.rs)
and [authored sequence, independent revocation and retired gap journeys](../../../worth-query-court-host-certification-fast/tests/worth_query_court_host_certification_fast/fast/certification/mod.rs).
Their helpers show complete public and governed admission, bounded continuation,
independent committed payload/receipt oracles, cancellation and final pin release.

The [Native growth court](../../../../../../crates/worth-runtime-world/tests/runtime_world_certification/court/record_history/growth.rs)
compares ten and 10,000 unrelated retained record changes with identical finite
page limits, backing and actual per-page request-work maxima. It traverses every
empty incomplete prefix before the one independently selected matching record.
Total traversal still grows with the retained source.

The [correction history](../execution/recorded-source-correction.md) remains the
owner of inverse, pre-image and reapplication meaning.
