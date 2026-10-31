# Branches and Previews

This page covers two separate concepts that share the word "branch":

| Concept | Entry point | Audience |
|---|---|---|
| Application product branch | `application.branches()` and `application.on_branch(...)` from `worth-query-host` | Application code |
| Workspace preview and branch session | `workspace.preview(...)` and `workspace.branch(...)` from `worth_query::facade` | Internal engine code |

A product branch is a Runtime World identity with its own history and program
revision. A Workspace preview or branch session is a lane-shifted context over
retained runtime surfaces and never mints product identity.

## Application product branches (host)

A **product branch** is a separate public concept. Runtime World owns its
identity, current composite commit, bounded single-parent history, and exact
Relational/Signal component composition. In a host advancement callback, use
the supplied `phase` for creation, selection and history traversal:

```rust,ignore
let source = application.current_world();
let branch = application
    .branches()
    .fork(source)
    .components(|components| {
        components.fork_relational().reuse_exact_signal_basis()
    })
    .create(&phase)?;
let selected = application.on_branch(branch).select(&phase)?;

// After one or more successful transactions advance this branch:
let page = application
    .branches()
    .history(&phase, branch, NonZeroUsize::new(1))?;
let prior = page.continue_ancestry(&phase, NonZeroUsize::new(1))?;
let entry = prior.entries().next().expect("the prior generation is retained");
let historical = prior.select(&phase, &entry)?;
```

The component selector has four complete postures: reuse both exact bases,
fork Relational and reuse Signal, reuse Relational and fork Signal, or fork both.
The product identity never collapses into either component identity. Branches
that share Signal may advance their Relational histories independently.

### Fork from an admitted selection

Use `fork_expected` when creation must use a selection already admitted by the
application. It consumes that selection and refuses if its source has moved.
The ordinary `fork` operation resolves its source during creation.

```rust,ignore
let selected = application.on_branch(source).select(&phase)?;
let candidate = application
    .branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
let branch = candidate.branch();
```

Run ordinary requests with `.on_branch(branch)`. Keep `candidate` while working:
it retains the exact source admission alongside the new branch.
`candidate.expected_source()` returns descriptive information about that source.

To select the candidate as a model's head, pass the model owner's actual mutable
branch slot to `adopt`. Run the authored requests before this call:

```rust,ignore
let previous = candidate.adopt(&mut model.head, &phase)?;
// Subsequent requests use model.head. Retire the previous branch separately.
application.on_branch(previous).close(&phase)?;
```

Adoption compares the slot with the original source and checks the source's exact
Native/World basis while replacing the slot. It refuses if the source moved during
candidate evaluation, even if the source's values later returned to their original
values. A retired candidate or a phase from another runtime also refuses. The
operation synchronizes with ordinary commits and protects both heads against
retirement during replacement. Cancellation or deadline expiry refuses before
the slot changes, including interruption while waiting for the guards.

A refused adoption leaves the supplied head unchanged and returns the candidate
and typed denial through `failure.into_parts()`. Its ordinary request effects
still exist on the candidate; retain it for inspection or discard it with `close`.
A refused close of the previous branch after successful adoption leaves the new
head selected. Handle that close's retry or pending cleanup separately.

Discard the candidate with `candidate.close(&phase)`. A refused close returns
the candidate and original denial through `failure.into_parts()`; inspect
`failure.denial()` to select the ordinary retry or pending-cleanup path. A
`ProductUnpublished` creation outcome retains its existing creation recovery.
Complete that recovery before issuing another creation attempt.

If the expected source is stale, obtain a fresh selection before creating a
replacement candidate. A selection from another application is refused.
The [compiled adoption cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/adoption.rs)
show ordinary candidate requests, head replacement, stale-source refusal and
cleanup.

### Own an application branch head

Consume a newly created expected-source candidate with `into_owned_head()` when
the application needs a move-only owner for its branch slot. Keep that owner in
the application state and use its descriptive `branch()` for ordinary requests.
A copied selector cannot construct another head owner.

```rust,ignore
let mut head = initial_candidate.into_owned_head();
let selected = application.on_branch(head.branch()).select(&phase)?;
let candidate = application.branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
// Run the ordinary candidate requests before adoption.
let previous = head.adopt(candidate, &phase)?;
previous.close(&phase)?;
```

`head.adopt` uses the same exact-source and currentness guards as candidate
adoption. Success returns the previous head as a separate owner; refusal leaves
the current head unchanged and preserves the candidate in the failure.
Converting the initial candidate releases its expected-source admission. The
head owner manages branch custody; ordinary request authorization still applies.

Close each head explicitly. A `Retained { head, denial }` failure returns a live
head after a refusal before retirement. A `Retired { branch, failure }` failure
returns the actual cleanup obligation after retirement. Retain
`failure.into_cleanup()` and call `retry()`; a further failure retains that same
cleanup through `into_cleanup()`. For example, a reader holding the retired
head can keep cleanup pending until that reader is released.

The [compiled head-owner cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/owned_head.rs)
show adoption, independent heads, wrong-source refusal, foreign-phase close
refusal and cleanup after releasing a held reader.

### Attach a model and read its owned head

Declare a history binding beside the model's existing ordinary READ and
mutation bindings. The READ binding's scope identifies the model type; the
definition names participating operations and their labels.

```rust,ignore
pub struct RetentionHistory;
worth_query_portable_type!(RetentionHistory => "example.retention-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for RetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DocumentRetentionHistory";
}
let schema = schema.application_history(
    ApplicationHistoryDefinition::<DocumentRetentionSchema, RetentionHistory>::for_binding(
        ApplicationQueryDisclosureContract::public(),
    )
        .operation::<SetRetentionBinding>("Set retention"),
);
```

Import history declaration types from `worth_query_decl::facade::application_history`.
Register the definition inside the same application contribution as its READ
and mutation bindings. Installation checks their actual typed contracts and
model scope. Matching identity text alone cannot substitute another binding.

To carry the declaration in a package archive, export the package's typed records
and pass its schema view to `encode_record_frame`. `WorthQueryPackageArchiveRecordDecoder`
preserves the full READ binding, metadata policy, model domain and authored
operation order and labels. Installation still resolves the original typed
bindings. Attach a live document from an admitted model head as below.

Transfer a genuine owned head through the ordinary request on that branch:

```rust,ignore
let head = initial_candidate.into_owned_head();
let document = host.with_host_advancement(|phase| {
    runtime.request(&principal, &scope).on_branch(head.branch())
        .attach_history::<RetentionHistory>(
            &phase, head, DocumentRetentionRead { identity: document_identity },
        )
})??;
let result = document.read(&runtime.request(&principal, &scope))?;
drop(result); // Release this read before attempting retirement.
host.with_host_advancement(|phase| document.close(&phase))??;
```

Attachment completes the declared READ and binds the actual model record,
including its generation, to the exact current head. A refusal returns the
original head through `failure.into_parts()`. The document is move-only.

`document.read` uses its own head and original model selector. The request
supplies fresh principal and execution scope; it cannot retarget the document.
Each call runs current ordinary READ admission. A foreign runtime, recreated
model or externally changed head refuses; equal field values do not repair a
changed head. No rows are issued on a divergence refusal.

A model retired and recreated under the same lookup name needs a new attached
document. Admit and own its new head through ordinary selection and forking;
copied names and step descriptors cannot transfer the old document's custody.
If the old document still owns the source branch, acquire the new head before
closing it. The [compiled model-recreation journey](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/model_lifecycle.rs)
shows ordinary retirement/creation, old-document refusal, a fresh attachment
and a successful edit on the replacement model.

For a retained no-output action, use `document.assess_edit(&request, &operation)`
to observe whether that original action currently belongs to this document.
The call completes the document's ordinary READ, freshly authorizes the action,
and checks its full model identity, declared operation membership and exact
selected head. An action for a different document or branch refuses even when
the model types and field values match. Both owners remain available after
success or refusal.

```rust,ignore
let request = runtime.request(&principal, &scope);
let eligibility = document.assess_edit(&request, &operation)?;
```

Use the observation to update action availability. It claims no idempotency key,
runs no handler and consumes no workflow grant. It grants no permission to
execute; ordinary action execution still requires fresh authorization.

Close under a phase from the owning host. `Retained { document, denial }` keeps
the live document after a refusal before retirement. `Retired { branch,
failure }` carries actual cleanup after retirement; retain and retry
`failure.into_cleanup()`. `Closing(owner)` carries unfinished disposal when
additional document-owned heads remain. Retry `owner.retry(&phase)` with a
phase from the owning runtime. A closing owner cannot read or edit. Close drops
document-owned history pins before retiring heads; independently held readers
can delay actual cleanup and must be released by their owners.

The [compiled declaration](../../../worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/document_history.rs)
and [compiled attachment cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/attachment.rs)
show two model heads, current reads, equal-value divergence and explicit close.
The [compiled edit-assessment cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/assessment.rs)
show ordinary action construction, model and membership refusals, fresh request
authority, unchanged model values and disposal of both owners.

### Use a governed payload READ

When the declared model READ requires capability governance, attach with
`request.attach_history_with_payload::<History, _, _, _>(&phase, head, access,
capability, operation, input)`. Supply the original query's payload purpose.
The returned document carries `WorthQueryGovernedHistoryPayload` as its final
type parameter; retain that mode when moving the document through edits,
builders, finish results and close refusals.

The stored references and cloneable input describe a purpose. Every read admits
that purpose again with the current request's principal and security context.
This includes page and detail reads, opening a preview, reading through an
existing preview, context refresh and internal edit reads. A retained preview
supplies its original data and grants no current payload permission.

Metadata still has its own contract and purpose. For example, a governed
payload document with governed metadata uses `current_step_governed(..)` with
the metadata capability while the stored payload recipe admits the payload
query independently. Revoking either grant refuses the corresponding access.
After a control write, restore lawful current authority and explicitly refresh
the document context before retrying an old holder. See the
[executable payload revocation journey](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/payload.rs).

### Read selected step metadata

Declare metadata policy explicitly in `ApplicationHistoryDefinition::for_binding`.
`ApplicationQueryDisclosureContract::public()` permits the public metadata entry.
For a governed contract, cover each `ApplicationHistoryMetadataFacet` with an
`ApplicationHistoryMetadataRef::<YourHistory>` rule under the declared capability.
The [installed example](../../../worth-query-testkit-document-retention/src/certification/application_graph/document_retention_model/document_history.rs)
uses the existing capability declaration and grant owner.

`document.current_step(&request)` requires public metadata. Governed metadata
uses `document.current_step_governed(&request, capability, operation, input)`.
The model's ordinary READ, including its body and filters, must also succeed.
Each call resolves current permission before looking up step metadata. A payload
grant or a mutation grant cannot substitute for the metadata purpose.

For fresh governed edit enrollment or an attempt retry, call
`edit.advance_with_metadata(&request, capability, operation, input)`. On refusal,
keep that returned edit and retry it with fresh authority. Once an occurrence
has been delivered, its step descriptor remains owned descriptive evidence.
See the [compiled metadata, retry and revocation cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/metadata.rs).

### Page completed document steps

Call `document.history_page(&request, limits, after.as_ref())` with positive
`WorthQueryHistoryPageLimits` bounds for returned entries, examined entries and
owned metadata bytes. `history_page_governed(&request, capability, operation,
input, limits, after.as_ref())` supplies the separate metadata purpose. These
calls execute the original model READ and current metadata policy before
checking cursors, storage limits or journal entries.

Rows arrive in creation order. Each sealed step identifies its parent, so the
consumer can derive child lists and layout without making that cache authority.
`row.retained_source_at_read()` describes exact source custody at assembly. It
does not retain historical payloads or grant payload READ, preview or mutation
permission. Dropping a page releases its own metadata backing; it does not close
the document or invalidate another page's descriptions.

Use `page.continuation()` for the next call. Foreign and changed-view cursors
refuse explicitly; they never restart silently. A successful edit changes the
view, as does selecting another history node. Private reservations and discarded
attempts leave it unchanged.
`edit.history_page(..)` continues to show that completed selected view.
`examined_entries()` counts visited existing map entries, not skipped ordinal
gaps. `admitted_backing_bytes()` reports a conservative buffer/label quote,
including shared label backing, rather than physical allocation traffic.

On allocation or request refusal, keep the original cursor and retry with fresh
permission and sufficient limits. Source movement or cancellation during
assembly discards the staged page. See the [compiled page journeys](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/mod.rs)
and [assembly handoff cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs).

### Read a historical step

To inspect a recorded step, open `document.preview(&request, node_id)` and call
`preview.read(&document, &request)`. The holder retains the exact historical
World source, while every read uses the supplied document's current principal,
scope, permissions and metadata purpose. Its query runs against the historical
Native/Signal correspondence through the original installed query executor.
`rows()`, `source()` and `basis()` describe that admitted result; they do not
select a writable branch. Closing a holder or result releases its own custody.
`required_root_posture()` reports the original qualified settlement for a
recorded fixed graph. The recorded endpoint retains one actual basis qualified
against the complete original output and discovery set. The source action,
root output and common endpoint can have different commits. `None` means no
required-root settlement was carried, rather than output unavailability.
Preview reads execute neither source commands nor producers.
For governed metadata, pass the current capability to `preview_governed(..)`
and `read_governed(..)`. An out-of-band change still causes document divergence;
restoring a grant does not silently replace its expected model head.
See the [compiled preview and independent-holder journeys](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/preview/mod.rs).

### Select a historical step for editing

Declare snapshot navigation explicitly on a history binding with
`.model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)` and
`.navigation::<Navigation>()`. Implement
`ApplicationHistoryNavigationBinding<Schema, History>` for `Navigation`. Supply
its stable `IDENTITY`, installed `Capability` and `Operation`, structured `Input`
and `InputBinding`, and a pure `input(&History::Access)` mapping. The capability
request must resolve the same model scope as the history READ. Install that
capability and operation through the ordinary schema declaration.
The model kind must be separate from installed principal, principal-mapping and
capability-grant roles; those live control records cannot be historical payload.

READ and navigation are separate permissions. Listing or previewing a node does
not authorize selecting it. Each preparation and advancement executes the
original current model READ, metadata disclosure and current NAV admission.
The navigation request reuses that READ's resolved principal; historical grants
never supply current permission.

Obtain a sealed description of the actual current selection before preparing:

```rust,ignore
let expected = document.selection(&request)?;
let navigation = document.navigate::<Navigation>()
    .checkout(node_id)
    .expected_selection(expected)
    .idempotency(click_key)
    .prepare(&request)?;

match navigation.advance(&request) {
    WorthQueryHistoryNavigationAdvance::Selected(done) => {
        show_selected(done.selection().step());
    }
    WorthQueryHistoryNavigationAdvance::Pending(owner) => retain_navigation(owner),
    WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
        retain_navigation_error(cause, owner);
    }
}
```

The node identifies an immutable authored occurrence. The expected selection
also identifies the actual current writable context. Checkout selects the
node's retained model record into a new writable branch while preserving current
records outside that model, including grants. It adopts the exact performed
endpoint through the original head owner. A competing selection or changed
candidate refuses rather than substituting a later equal-looking state.

Navigation adds no authored node. An ordinary edit after selecting S1 becomes a
child of S1; previously authored S2/S3 remain independently previewable and
selectable. `done.selection().physical_endpoint()` describes the new writable
context; `done.selection().step().endpoint()` still describes the original
authored source.

This entry supports an entity-record domain with an unchanged Signal basis,
compatible installed program, no storage medium, and no relevant required
outputs, workflow execution or external effects. Unsupported domain, program,
output, workflow, external-effect or Signal meaning returns a typed
`RequiresSemanticCorrection` eligibility refusal. Native source, schema and
storage-medium incompatibilities retain their owning-component refusal. Use the
ordinary semantic correction entry when the requested behavior requires
compensation or reconciliation.

Keep the owner on pending work or refusal. `discard(&request)` releases its real
candidate obligations and returns the owner if cleanup refuses; it can release
custody after NAV revocation. Dropping a handle leaves its obligations in the
document, whose explicit close drains them. Cancellation after adoption cannot
roll back the selected head: the owner reports `Selected` posture and a fresh
authorized retry resolves its original result. Retrying a successful key after
later navigation returns that original completion without moving the current
selection. Changed expected selection or node under the same key refuses.

For governed metadata, use `selection_governed(..)`,
`prepare_governed(..)` and `advance_governed(..)` with the current metadata
capability. Governed payload documents also repeat their original payload
purpose. See the [compiled checkout and fork journey](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/mod.rs)
and [owned retry cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/retries.rs).

### Refresh context after control changes

For a model consisting of exactly its scope entity record, declare
`.model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)` on its
`ApplicationHistoryDefinition`. This domain includes that record's full
generation. Related records, relations and Signal state are not added implicitly.
The default `WholeComposite` domain requires the original physical source.

After an outside control write, call
`document.refresh_context(&request, limits)` explicitly. For governed metadata,
use `refresh_context_governed(&request, capability, operation, input, limits)`.
Set positive `maximum_commits`, `maximum_records` and `maximum_evidence_bytes`
in `WorthQueryHistoryContextRefreshLimits`. These bound commit inspection,
examined record-patch and intent slots, and scratch or cold-evidence acquisition.
Resident evidence is borrowed. The original request's resource limits still apply.

Refresh runs the ordinary model READ and the current metadata decision first.
Native must prove that every intervening transition left the model record
untouched, with the exact Signal basis unchanged. An equal-value write or a
change followed by restoration still counts as model authorship and refuses.
Missing evidence, unsupported transitions, exhausted bounds, cancellation and a
source race refuse without replacing the document's expected context. Keep the
same document and retry with appropriate bounds after a transient refusal.

Success preserves the current step, its immutable endpoint and issued tree
cursors. Existing preview holders still read their original sources under fresh
current authority, and the next edit records a child of the same selected step.
Restoring a revoked grant therefore requires both the real current grant and a
successful explicit refresh; the old preview cannot authorize itself.

### Record an edit and retain its original outcome

Use the attached document to construct a fresh edit for a declared operation.
This entry accepts source-completion actions and fixed required-output graphs
whose bindings have no mutation source, workflow execution or external effect.
The document owns the actual
selected head throughout the edit; no separate application slot needs updating.

For a fixed graph, match `WorthQueryFixedHistoryEditOutcome` after `finish`.
`Recorded { step, delivery }` owns the original `WorthQueryClosedFixedDelivery`;
retain its settlement and close evidence as needed. Native qualifies the complete
original output and discovery set at one retained basis before selecting that
exact endpoint. Neither the latest candidate head nor the root's earlier
publication can substitute for it. A changed candidate refuses exact adoption
while the edit retains its original completed output owner.
`AlreadyRecorded { step, closed }` identifies the original step without moving
the current selection. `NotRecorded { terminal }` carries original source
diagnostics and completed cleanup through `advance_denial()`, `source_outcome()`, `source_result()`
and `close_evidence()`. A performed source followed by unavailable output is
distinct from a source that performed no effect. See the [executable fixed-leaf
journey](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/document_history/journeys.rs).
The [complete-output journey](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/document_history/aggregate.rs)
reads old dependent output after later edits. Terminal unavailability still
requires the original leaf eligibility; a partial nonleaf owner retains its
actual unresolved obligations.

```rust,ignore
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryEditAdvance, WorthQueryHistoryEditOutcome,
};

let mut edit = document
    .edit(SetRetentionIntent {
        input: SetRetentionInput { identity: document_identity, retention_days: 3 },
    })
    .idempotency_owned(&request, operation_key)?
    .retain_in_program::<RetentionProgramP0, SetRetentionAction>(host)?;

let edit = loop {
    match edit.advance(&request) {
        WorthQueryHistoryEditAdvance::Progress(next) => edit = next,
        WorthQueryHistoryEditAdvance::Ready(ready) => break ready,
        WorthQueryHistoryEditAdvance::Refused { edit, denial } => {
            // Keep this edit: it owns any candidate, completion and cleanup.
            return Err((edit, denial));
        }
    }
};
let finished = edit.finish(&request)?;
let document = finished.document;
match finished.outcome {
    WorthQueryHistoryEditOutcome::Recorded { step, completion, closed } => {
        // Use step.id()/parent()/label() for UI metadata and retain the
        // original completion for the action's result and commit receipt.
    }
    WorthQueryHistoryEditOutcome::AlreadyRecorded { step, closed } => {
        // This is the original occurrence; the current selection stays put.
    }
    WorthQueryHistoryEditOutcome::NotRecorded { terminal, closed } => {
        // Inspect terminal.native_outcome(); no completed step was appended.
    }
}
```

The [compiled recording journeys](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/journeys.rs)
and their [concrete action construction](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/mod.rs)
show the complete imports, installed action type, advancement, outcome handling
and disposal. Add typed expected facts with `.preconditions(conditions)` before
capturing the key when the action requires them. A capture refusal returns the
original builder, key and denial through `failure.into_parts()`.

Before opening a private candidate, admission completes a fresh model READ,
authorizes the original action and reserves the record's actual storage. The
business operation runs through its ordinary continuation on that candidate.
`edit.read(&request)` reads the selected document, while `completed_source()`
retains the candidate's original completion. Selection binds that receipt's
exact endpoint. An out-of-band candidate change refuses even when it restores
the same field values. Equal model values at different completed occurrences
have distinct step identities.

Use `posture()` when an advancement refuses. Cancellation after adoption can
leave `Selected` with its step and original completion still owned. It does
not undo selection. Call `finish` with a fresh request to transfer that evidence
after ordinary operation cleanup. A domain denial remains a native terminal
outcome and produces no completed user step. Inspect `terminal_source()` and
finish it to recover the document and its `NotRecorded` outcome.

For an equivalent retry in the same live document, keep the original key,
input and typed preconditions. Admission returns the original completed step,
including after later edits; it neither reruns the handler nor resets the
selected head. Changing any admitted intent under that key refuses. These
attempts belong to the process-local document and are not durable restart data.
`current_step(&request)` freshly admits the document READ before returning its
current sealed descriptor. A copied step or node ID grants no READ or write
permission and holds no source lease.

To abandon unselected work, call `discard(&request)`. On refusal, retain the
returned edit and resolve its actual continuation or cleanup before retrying.
An unsettled publication cannot be discarded as if it had no effects. A
selected edit must be finished. Close the returned document explicitly; release
independently retained reads before retrying a blocked close.

If `source_operation().posture()` reports `ProductUnpublished`, the business
operation has retained owner effects whose product publication still needs
recovery. The document keeps its previous selection. Keep the same edit and
call `advance` with a fresh request; do not submit a replacement edit or append
a receipt yourself. `discard` returns `NativeObligationRetained` while those
obligations remain. Successful recovery selects one step and returns the
original `Recovered` completion, including its actual publication evidence.
The [compiled recovery journey](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/recovery.rs)
shows cancellation after owner effects, refused discard, same-owner recovery
and final disposal.

### Store an expected basis and select it again

Keep records on a separately owned storage branch when saving them must leave
the authored model's basis current. Fork that branch's Relational component
with `parts.fork_relational()` and bind recording requests with
`.on_branch(storage_branch)`. Keep the storage branch's lifecycle owner and
close it explicitly when the records are no longer needed.

Use a declared canonical field binding for structured records so ordinary
queries restore their typed values. The compiled
[record schema](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/schema/data.rs)
and [storage example](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/storage.rs)
store model and target occurrences, ordered typed parameters and comparison
data through ordinary mutations and queries. Its collection query returns a
complete result or a typed limit refusal; exact-key selection follows the
successful read.

Retain one observation privately and perform the model's governed read at that
observation before describing it. Use `observation.select_on(&phase, runtime)`
to describe the same selection after admission. The
[compiled reader](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/model.rs)
uses current READ authority with retained data; storage and model reads have
independent policies over the actual grant facts on their respective branches.

Call `selected.describe_basis()` before releasing an admitted selection. Store
the returned `WorthQueryExpectedProductBasisDescription` with ordinary versioned
application data. It implements Serde serialization and contains comparison
coordinates only; keeping it does not retain a snapshot or keep a branch alive.

```rust,ignore
let selected = application.on_branch(model.head).select(&phase)?;
let bytes = serde_json::to_vec(&selected.describe_basis())?;
drop(selected);

// Load the application record under a fresh ordinary READ request first.
let description: WorthQueryExpectedProductBasisDescription =
    serde_json::from_slice(&bytes)?;
let selected = application.branches().select_expected(&description, &phase)?;
let candidate = application.branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
```

`select_expected` asks the installed World and Native owners to admit the exact
described basis. It refuses foreign owners, retired or replaced occurrences,
movement and unsupported description versions. Returning the values to their
previous contents does not restore the original basis. A retained historical
Native basis also cannot substitute for the current described basis. These
coordinates address their original live installation. Use them only within that
installation's lifetime; they are not portable branch restoration data.

Loading or editing the description grants no domain READ or mutation permission.
Use ordinary requests for those operations. Creation and adoption still check
the expected source, so movement after readmission can refuse either operation.
Keep and close candidate custody as described above. The
[compiled stored-basis cases](../../../worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/stored_basis.rs)
show serialization, fresh owner admission, ordinary candidate requests, adoption
and stale or mixed-basis refusal.

### Branch history and retirement

To correct a retained operation on a branch, bind an ordinary request with
`.on_branch(target)` and use its
[recorded correction entry](../execution/recorded-source-correction.md).
Selection proves the source belongs to the target's retained ancestry; preparation
checks current disclosure, program meaning and inverse authority. Publication
appends one child of that target and preserves its disjoint intervening edits.

Correction alternatives also belong to the selected branch's history. Use
`request.corrections().alternatives(...)`, follow its continuation to completion,
and select the historical reversal the user chose. The completed traversal
resolves the active reversal on that exact target. For example, after a source
is reversed, reapplied and reversed again, choosing the first reversal reapplies
the retained source input and consumes the currently active reversal. The
receipt records both identities. A sibling branch keeps its own active reversal
and continuation.

Each history continuation binds its target observation. If the branch advances
between pages or before selection, obtain a new traversal with a fresh request.
Keep the command's idempotency key when retrying a submitted correction: an
equivalent retry resolves its original performed occurrence. The
[history chooser example](../../../worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/tree_reapplication.rs)
shows explicit older-alternative selection, pagination and ordinary close.

A selected occurrence pins exact history for repeatable reads while the branch
head advances. Closing a product retires its World history first and then any
private component branches. History a live branch still reaches is not the
closed branch's to release: a live fork keeps its whole ancestry, including
history it inherited from the closed branch, and the close completes. A
publication that installed its successor without moving the head keeps that
history until its recovery is released, and the release then frees the closed
branch's history the same way. Retained
reads can defer cleanup; the returned pending cleanup retains the only retry
authority. A retired occurrence cannot be selected again, even if an
application later reuses the same display name.

Run isolated transaction steps on a fork of the selected branch. Keep the
application's selected branch in its owning model state, and have that owner
choose the branch used by subsequent requests after the steps succeed. Use
`close` to retire a discarded branch and release its resources. A refused step
on the fork leaves the parent's head and history unchanged.

The fork reads its parent's history as its own, so corrections can select a
source made on either branch. Retained history grows with the actual authored
operations across branch selection changes. The
[fork transaction example](../../../worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/fork_head_adoption.rs)
shows request execution, explicit branch selection and retirement.
History pages are branch-occurrence scoped and bounded by the caller's nonzero
limit. Their entries are descriptive until `select` asks World to issue an
exact historical observation; copied commit identities cannot perform that
admission. Drop history pages and historical reads before closing the branch
whose ancestry they protect.
See the executable
[advanced product branching example](../../../worth-query-certification/examples/advanced_product_branching.rs).

### Branch-local application programs

Each product occurrence carries one owner-admitted application-program
revision. Sibling branches may therefore remain on P0 and P1 simultaneously.
`application.request(...).on_branch(branch).programs().inspect()` reports the
program carried by that exact selection; the report is not adoption authority.
Ordinary queries, mutations, retained reads, security bases, pending producers,
and required-output work resolve through the same selected occurrence.

Adoption compares installed canonical meaning, derives migration and custody
requirements, validates existing state under the target rules, and reserves
one branch-bound publication. A component-local success does not activate the
target. Only the final World `Performed` outcome records the target program;
`ProductUnpublished` retains the original source, target, migration, owner
effects, and exact recovery route.

For multiple branches, call
`application.branches().program_adoption_coverage(&branches, maximum_targets)`
and then `request.on_branches(coverage, ordered_targets)`. The returned handle
is bounded and explicitly non-atomic: inspect `progress()`, resolve a
`NoEffect` or `ProductUnpublished` stop, and then resume, cancel, or close it.
Later-created branches never enter previously issued coverage. Handle each
branch's program adoption outcome before resuming the remaining set.

### Workflows on a forked branch

A fork copies the source branch's workflow definitions and instances as
history, not execution. A copied instance cannot act on the fork: its
approvals and receipts open nothing there. To keep that work going on the
fork, issue `prepare_workflow_fork_continuation(&workflow, instance, target,
resume_at)` on the fork. It ends the fork's copy and starts a successor on the
fork's current `target` definition, under the same law as migration. The
instance on its own branch is untouched. The instance-start capability
authorizes it. It is refused while an owner operation is unsettled and after
the instance's deadline. See
[Migration and fork continuation](workflows.md#migration-and-fork-continuation).

For the request lanes that run against a selected branch, see
[Branch-resolved mutation lanes](ordinary-application-front-door.md#branch-resolved-mutation-lanes).

## Workspace preview sessions (internal)

> **Internal engine surface.** This section documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

### What This Feature Is

Preview and branch sessions are isolated runtime contexts built from the same
workspace model. They let you reuse retained live, computed, and effect
surfaces while moving writes and intent-like work into preview-local or
branch-local authority lanes instead of mutating current truth directly.

### Why You Use It

- you want to try changes without touching authoritative truth
- you need isolated preview-local writes and closeout behavior
- you need branch-local intent staging that never silently executes against the
  authoritative lane

### Stable Entry Points

Stable session entry points:

- `workspace.preview(...)`
- `workspace.preview_with_options(...)`
- `workspace.branch(...)`
- `workspace.branch_with_options(...)`

Stable option constructors:

- `WorthQueryPreviewOptions::derive_only()`
- `WorthQueryPreviewOptions::muted()`
- `WorthQueryPreviewOptions::redirected_delivery()`
- `WorthQueryPreviewOptions::sandboxed_write_intent()`
- `WorthQueryBranchOptions::derive_only()`
- `WorthQueryBranchOptions::muted()`
- `WorthQueryBranchOptions::redirected_delivery()`
- `WorthQueryBranchOptions::sandboxed_write_intent()`

Ordinary preview and branch entry uses typed session labels:

- pass `WorthQuerySessionLabel`, not raw strings
- equivalent label identity re-entry stops through
  `WorthQueryStopClass::SessionLabelCollision`
- display rendering is presentation; basis admission and closeout identity use
  canonical label identity instead

Stable preview-local operations:

- bind live/computed/effect handles into a preview
- stage preview-local writes
- discard or promote a preview outcome

Stable ordinary branch-merge operations under
`worth_query::facade::workflow`:

- `declare_branch_merge(...)`
- `branch_merge(...)`
- `WorthQueryBranchMergeOutcome`
- `WorthQueryBranchMergeDurabilityUnresolved`
- `WorthQueryBranchMergeNextAction::ReopenRelationalOwner`
- `WorthQueryRuntime::repair_pending_branch_merge_settlement(...)`
- `WorthQueryWorkspace::repair_pending_branch_merge_settlement(...)`

Stable preview-live planning entry points live under
`worth_query::facade::policy`:

- `admit_scoped_preview_session_plan_binding(...)`
- `admit_scoped_preview_live_session_plan(...)`
- `execute_scoped_preview_live_session_plan(...)`
- `assess_preview_live_drift(...)`
- `ScopedPreviewSessionPlanBinding`
- `ScopedPreviewLiveSessionPlanBinding`

Support-gated neighbors:

- preview-local or branch-local intent execution still depends on admitted
  intent support and the correct sandboxed policy

### Core Mental Model

Preview and branch sessions are not separate products. They are lane-shifted
contexts over the same retained runtime surfaces.

Preview:

- binds existing handles into a preview lane
- keeps preview-owned active subscriptions separate from authoritative active
  subscriptions when their basis identity or checkpoint identity differs
- keeps preview-owned temporal wakes, async result-state, mixed-cause residue,
  and crossed preview or authoritative drift residue preview-local until
  discard or authoritative re-admission resolves them
- can stage preview-local writes
- can discard or promote staged work
- treats promotion as a rebinding boundary, not as structural reuse of the
  preview-owned basis
- defaults to `derive_only`

Branch:

- opens a branch-local lane
- is primarily about branch-local intent staging in the current public surface
- defaults to `derive_only`

The key idea is isolation by authority lane, not by reimplementing the whole
runtime.

Under that Query lane, Relational owns the actual branch reference. Each branch
has one mutable reference cell that selects an immutable root. An exact
reference observation records its selected target and branch-local truth
version together. Equal version numbers on different branches do not describe
the same state.

An owner-admitted branch basis pins the selected immutable root. Reads through
that basis are repeatable even when the live reference advances. A transaction
opened from the basis is detached from the runtime: it carries the exact basis,
overlay, and footprint without holding a general runtime borrow. Preparation
produces an opaque, single-use candidate. Publication compares that candidate
against only its branch cell, so unrelated branches have independent
linearization points.

Serialized basis descriptors are descriptive, not operational. Resolving one
checks its shape; only the owning Relational runtime can readmit it against the
live reference, retained root, lifecycle posture, and runtime identity. Query
consumes those admitted owner products through its facade rather than
reconstructing branch authority from IDs or digests.

For preview-live planning, isolation is also basis-scoped. A preview plan
binding and a scoped observation basis must agree structurally before Query
mints `ScopedPreviewSessionPlanBinding`. Live admission then consumes that
binding and returns `ScopedPreviewLiveSessionPlanBinding`, which is the only
artifact accepted by scoped execution and drift assessment. The sealed binding
keeps preview identity, live-plan identity, and basis authority together.

Graph touch obligations follow that same isolation rule. Branch and preview
graph touches must carry an operating world descriptor, and Query must select
registered obligations from that descriptor rather than pretending branch-local,
preview-local, and authoritative graph work share one validator table.

### How It Executes

Preview path:

1. Open a preview session from the workspace.
2. Bind live, computed, and optional effect handles.
3. Stage preview-local writes or preview-local intent work, depending on
   policy and support.
4. Discard or promote the preview.

Preview-live planning path:

1. Produce a preview session plan binding from an admitted preflight and active
   preview context.
2. Scope the matching observation world through `basis_lifecycle()`.
3. Admit both into `ScopedPreviewSessionPlanBinding`.
4. Admit the live plan into `ScopedPreviewLiveSessionPlanBinding`.
5. Execute or assess drift with that sealed scoped artifact.

Branch path:

1. Open a branch session from the workspace.
2. Use the declared branch effect policy.
3. Stage branch-local intent work when support and policy admit it.

An admitted Relational-backed mutation then follows a stricter owner path:

1. Observe or readmit one exact branch basis.
2. Open a detached transaction from that basis and stage work.
3. Validate and prepare one opaque branch-bound candidate without moving the
   reference.
4. Compare-and-publish under that branch's bounded critical section.
5. Settle durability and refresh any required Query indexes.

The performed and settled boundaries are distinct. If branch movement succeeds
but settlement fails, the operation returns typed settlement recovery. It must
not rerun the mutation or merge.

`derive_only` is the default for both preview and branch sessions. That means
derived behavior is allowed, but delivery and write-intent work are denied or
muted unless you explicitly choose a broader policy.

### Small Example

```rust
use worth_query::facade::runtime::{WorthQueryPreviewOptions, WorthQuerySessionLabel};

let mut workspace = runtime.workspace("preview").unwrap();
let label = WorthQuerySessionLabel::scoped_strs("workflow", ["draft-create"]).unwrap();

let mut preview = workspace
    .preview_with_options(
        label,
        WorthQueryPreviewOptions::sandboxed_write_intent(),
    )
    .unwrap();

preview
    .insert("Task", |task| {
        task.aspect("identity.id", "preview-1")
            .aspect("title.value", "Preview-only task")
    })
    .unwrap();

let outcome = preview.discard();
```

This is the smallest honest example because it shows preview-local staging with
an explicit closeout result instead of pretending preview writes are ordinary
truth writes.

### Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryBranchOptions, WorthQueryInspection, WorthQueryLiveView,
    WorthQueryPreviewOptions, WorthQuerySessionLabel,
};
use serde_json::json;
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("workflow").unwrap();
let preview_label =
    WorthQuerySessionLabel::scoped_strs("workflow", ["preview-execution"]).unwrap();
let branch_label =
    WorthQuerySessionLabel::scoped_strs("workflow", ["branch-local-intent"]).unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.preview-bind", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-preview")
    })
    .unwrap();

let preview_outcome = {
    let mut preview = workspace
        .preview_with_options(
            preview_label,
            WorthQueryPreviewOptions::redirected_delivery(),
        )
        .unwrap();
    preview.use_view(&live);
    preview
        .insert("Task", |task| {
            task.aspect("identity.id", "preview-task")
                .aspect("title.value", "Preview execution task")
        })
        .unwrap();
    preview.discard()
};

let branch_result = {
    let mut branch = workspace
        .branch_with_options(
            branch_label,
            WorthQueryBranchOptions::sandboxed_write_intent(),
        )
        .unwrap();
    branch.execute_intent(worth_query::facade::runtime::WorthQueryIntentDeclaration::strategy_commit(
        "branch-reconcile",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        json!({ "entity": "task-1", "title": "branch title" }),
    ))
};
```

An ordinary branch merge whose durable fate is unknown returns a typed
unresolved outcome from the workspace that owns the Relational runtime:

```rust
use worth_query::facade::workflow::{
    branch_merge, declare_branch_merge, WorthQueryBranchMergeNextAction,
    WorthQueryBranchMergeOutcome,
};

let declaration = declare_branch_merge("main", "candidate")?;
let context = branch_merge(&workspace, &declaration)?;

match declaration.using(context).run(&mut workspace) {
    WorthQueryBranchMergeOutcome::Completed(completion) => {
        inspect_merge(completion);
    }
    WorthQueryBranchMergeOutcome::DurabilityUnresolved(unresolved) => {
        assert_eq!(
            unresolved.next_action(),
            WorthQueryBranchMergeNextAction::ReopenRelationalOwner,
        );
        record_unresolved_merge(unresolved.commit_id());
        reopen_relational_owner();
    }
    other => handle_merge_stop(other),
}
```

Relational publishes the merge commit to its durable medium before the live
cutover. `DurabilityUnresolved` means nothing was installed live but the commit
may be durable; the Relational owner refuses later publications until it is
reopened, and reopening resolves the fate. The workflow counters report
`durability_unresolved_count()`. A performed merge commit whose witness was
lost is repaired by commit identity with
`repair_pending_branch_merge_settlement(...)` on the owning workspace or
runtime.

When a preview also carries a live plan, keep basis admission explicit:

```rust
use worth_query::facade::{
    foundation::basis_lifecycle,
    policy::{
        admit_scoped_preview_live_session_plan,
        admit_scoped_preview_session_plan_binding,
        execute_scoped_preview_live_session_plan,
    },
};

let scoped_basis = basis_lifecycle()
    .current_head()
    .observe()?;
let scoped_binding = admit_scoped_preview_session_plan_binding(
    scoped_basis,
    preview_binding,
)?;
let preview_live =
    admit_scoped_preview_live_session_plan(scoped_binding, live_plan)?;
let execution = execute_scoped_preview_live_session_plan(&preview_live)?;
```

Here `preview_binding` and `live_plan` are admitted planning artifacts produced
earlier in the preview pipeline. Consumer code moves the scoped binding
forward; it does not pass the basis digest and preview identity separately.

If you are authoring workflow capability evidence about a preview session, that
is a separate typed identity lane. Opening the preview uses
`WorthQuerySessionLabel`; binding workflow preview inspection or mutation
planning uses `worth_query::facade::runtime::BridgePreviewSessionIdentity`.
Do not collapse those two roles into one raw string.

Preview binding and preview outcome inspection stay on the
`WorthQuerySessionLabel` lane too. Use rendered labels for DX only; if replay
identity or collision posture matters, keep the typed session-label artifact.

What is authoritative:

- the workspace's current truth lane

What is isolated:

- preview writes land in `PreviewTruth`
- branch-local admitted intents land in `BranchLocalTruth`

What gets retained:

- binding evidence
- execution evidence
- closeout evidence
- preview or branch-local intent receipts when admitted
- exact branch observations and immutable-root retention for admitted
  Relational bases
- opaque settlement recovery when branch movement performed but durability did
  not settle

What closeout means:

- `discard()` proves staged work did not leak into authoritative truth
- `discard()` also retains preview-owned temporal, async, and mixed-cause
  residue counts so the runtime can prove what stayed preview-local
- `promote()` attempts an authoritative handoff, records a rebinding digest,
  and can fail typed and early when crossed preview residue requires
  authoritative re-admission first

### How It Relates To Other Features

- Use [Live Views](../runtime-surfaces/live-views.md) and [Computed](../runtime-surfaces/computed.md) as the
  retained handles you bind into previews.
- Use [Writes and Intent Boundaries](../execution/writes-and-intents.md) when deciding
  whether work belongs in direct writes, preview-local staging, or branch-local
  intent paths.
- Use [State and Readiness Surfaces](state.md) when you need typed posture
  snapshots around stable versus deferred families.

Preview and branch sessions are lane-control features, not alternate truth
engines.

### Inspection And Debugging

The runtime can explain:

- preview handle binding evidence
- preview execution evidence
- preview outcome and closeout residue
- preview-local and branch-local intent receipts

Look for:

- effect policy
- source lane versus target lane
- residue class counts
- temporal wake, async result-state, mixed-cause, and crossed-authoritative
  residue counts on preview closeout
- active checkpoint identity and future-bearing lane posture when a preview is
  bound to temporal or async live meaning
- rebinding digests on successful promotion
- basis snapshot tokens, rebinding digests, recovery posture, and denial
  digests on failed promotion
- branch-merge settlement message, counters, and `next_action()` without raw
  settlement access

### Anti-Patterns

- Treating preview writes as if they are already authoritative.
- Assuming `derive_only` allows delivery or write-intent work.
- Expecting branch-local or preview-local intents to bypass support admission.
- Executing or drift-checking preview-live work from an unscoped plan binding.
- Reconstructing preview-live authority from a basis digest, preview label, and
  live-plan digest.
- Treating preview or branch sessions as separate domain runtimes rather than
  authority-lane shifts over retained surfaces.
- Reconstructing a live branch basis from branch ID, commit ID, version, or a
  serialized descriptor instead of owner readmission.
- Assuming a reference-selected read follows later branch movement; an admitted
  basis is repeatable and remains pinned to its immutable root.
- Retrying a merge after `WorthQueryBranchMergeOutcome::DurabilityUnresolved`
  instead of reopening the Relational owner.

## Current Limits

- Preview and branch sessions are stable as runtime-backed isolation contexts.
- Preview-local writes, binding, discard, and typed promotion denials are part
  of the current surface.
- Temporal and async preview subscriptions are runtime-backed active objects,
  not wrapper-only observers, and they do not share authoritative active state
  when basis identity differs.
- Preview closeout now preserves preview-owned temporal, async, and mixed-cause
  residue explicitly, and promotion denies with typed rebinding recovery
  posture when crossed preview residue cannot be promoted honestly.
- Preview-local and branch-local intent work still depends on admitted intent
  support and the correct sandboxed policy.
- Relational branch-local MVCC, exact-basis readmission, detached transactions,
  and typed branch-merge settlement repair are implemented owner capabilities.
  They do not make every Query branch-session effect family supported.
- Signal owner services provide exact component bases and per-branch progress.
  [Runtime World](../../../../../../crates/worth-runtime-world/README.md) owns
  memory-resident composite history and coordinated Relational-plus-Signal
  publication. Query's product-branch facade carries that authority through
  reads, mutations, aftermath, delivery, and inspection. Existing Query
  preview and branch sessions remain lane controls and do not mint product
  identity.
- Durable preview replay and temporal/async branch-session behavior remain
  future work.

## Related Docs

- [Graph Touch Obligation Authority](../authoring/graph-touch-obligation-authority.md)
- [Workspace Overview](workspace-overview.md)
- [Writes and Intent Boundaries](../execution/writes-and-intents.md)
- [State and Readiness Surfaces](state.md)


