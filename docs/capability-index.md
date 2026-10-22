# Capability Index

WORTH is built to be used by AI agents as well as people. The most damaging
mistake a consumer can make is to conclude that the platform lacks a capability
and then build a second way to do it: a status table, a scheduler, a copied
permission check, a private registry. The second way does not carry the
platform's authority, so it is wrong in exactly the cases that matter.

This page answers two questions: is this supported, and what is the one way to
do it?

An application imports two crates only:

| Facade | Role | Shorthand on this page |
|---|---|---|
| `worth-query-decl` | Declare meaning: schema, operations, queries, capabilities, programs, workflows | `decl::` = `worth_query_decl::facade::` |
| `worth-query-host` | Install and run: requests, mutations, outputs, branches, workflows, recovery | `host::` = `worth_query_host::facade::` |

`host::declaration` re-exports the declaration modules, so a host crate can
reach `host::declaration::application_program::*` without a second import. Some
declaration macros (`worth_query_mutation_binding!`, `worth_query_capability!`,
`worth_query_policy!`) are on `decl::` only.

---

## Before you conclude a capability is missing

Do all five steps, in order.

1. **Search the facade exports.** `tools/boundary-check/snapshots/facades.toml`
   snapshots the top-level exports of both facades and the full contents of
   `host::application_entry`, `host::primary_graph`, and
   `host::provisional_aftermath`. Other modules (for example `host::runtime`,
   `host::application_installation`, and every `decl::` module) appear there by
   name only, so also read their rustdoc:

   ```bash
   cargo doc --manifest-path workspaces/worth-query/Cargo.toml --no-deps -p worth-query-decl -p worth-query-host
   ```

   Facade source: `workspaces/worth-query/crates/worth-query-decl/src/facade.rs`
   and `workspaces/worth-query/crates/worth-query-host/src/facade.rs`.
2. **Look up the task** in the [task index](#task-index) below.
3. **Search for a worked use** in the runnable examples and the reference
   application, which both use only the two facades:

   ```bash
   grep -rn "<name or concept>" workspaces/worth-query/crates/worth-query-certification/examples
   grep -rn "<name or concept>" workspaces/worth-query-bank-world
   ```

4. **Check the [known gaps](#known-gaps).** Follow the current support posture
   and linked consumer prohibitions before reporting a missing capability.
5. **Only then report the gap** ([Reporting a gap](#reporting-a-gap)). Do not
   build a substitute: no parallel registry, copied authority, host executor,
   host scheduler, or host persistence of platform truth.

---

## Task index

Application installation and reopen require one
`host::application_installation::ApplicationBuildIdentity`, embedded by the
application build pipeline. Query binds it to every producer, including
`INPUT_REUSE=None`; callers do not maintain a second producer roster.
A new build conservatively invalidates derived output. This identity describes
producing code and grants no execution, publication or recovery authority.
See [build identity](build-an-application.md#application-build-identity).

Server protocol projections preserve the catalog's exact result schema identity
and numeric version. Generated TypeScript/Python decoders reject a successful
envelope when either differs; they validate protocol correspondence, not product
payload validity or native operation authority. Use `WorthServer::product_protocol_catalog()`
and `worth-server-client-generation` rather than a consumer-maintained route or
version table.

External-input capture and original mutation description are distinct custody.
For resumable provider-revision admission, use
`WorthQueryCapturedExternalInput::admit_preserving(&provider)` and retain its
returned `(original_capture, provider_denial)` on refusal. Retry that capture;
do not refetch current data or reconstruct an admitted token from serialized
values. The one-shot `admit` discards capture on refusal. Both admit data
correspondence, never operation authority. See [external input on an action](build-an-application.md#32-attach-actions-to-features).

Program validation admits finite canonicalization work through
`ApplicationProgramDefinition::CANONICAL_REVISION_BUDGET` and the program facade's
`CanonicalDigestWorkBudget`. The 4096-entry/1 MiB default is unchanged; override
the declaration's finite ceiling for larger actual meaning. A sufficient budget
does not alter canonical revision identity or grant execution authority. See
[program validation](build-an-application.md#35-validate).

Ordinary mutations carry their prepared decision into `CandidateWriter` without
candidate quotes or dependency-byte ceilings. Installed mutation authority,
required output roles/families, current reads, cancellation and atomic publication
remain enforced. See [prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects).

Platform owners can borrow the existing targets of a validated native mutation
with `ValidatedMutationFootprint::records()`. The iterator preserves target
multiplicity and performs no graph read or allocation. Consumers still apply
the footprint's field and scope checks; enumeration grants no publication or
currentness authority.

Platform runtime owners retaining actual native map results use
`ExecutionMap::run_retained` with a fresh `ExecutionRequest` and declared work
ceiling. Its opaque `ExecutionRetainedResult` holds the accepted value and its
funding; `try_carry` uses the native meter's original checkpoint prefix or
requires actual execution when context observations were not reusable. This
grants no model, source, session or edition authority. Arbitrary checked maps
still perform full admission and dispatch; this is not sparse map settlement.
Use `RetainedMapInput` with `run_retained_inputs` for a mixed current plan, or
`run_reduce_retained_holding` for genuine leaf results beside the canonical
tree. See [retained native results](../crates/worth-execution/README.md#retained-native-results).

Platform-owned ordered retained metadata uses `ExecutionOrderedMap<K, V>`:
shared-root cloning, canonical iteration and policy-admitted changed paths retain
their own grants. `rank_of`/`entry_at` use cached counts rather than prefix scans.
Native reduction shape-cost inspection accepts exact removals/insertions through
`checked_full_build_cost_after_edits`, reusing native edit traversal without
combining values. This storage/cost information supplies no result, source or request authority.
See [funded ordered storage](../crates/worth-execution/README.md#funded-ordered-storage).

For a platform owner's keyless map, `run_retained_run` issues an opaque
`ExecutionRetainedRun` after real native completion. `prior.advance` accepts
changed inputs and canonical removed identities only; native settlement
consumes cached unchanged ranges and executes a tight slot's original native
checkpoint prefix. The caller must independently qualify unchanged inputs.
Stopped prefixes cannot advance, and context-unqualified slots require real
execution. This does not make Query roster, source validation, publication or
collection membership processing sparse. See
[persistent native runs](../crates/worth-execution/README.md#persistent-native-runs).

For nested persisted values, use the existing value-binding macro's
`scalar: String, canonical,` arm with Query-free
`WorthValue` derivation. The installed value-binding catalog retains the derived
shape and any explicit domain representation policy. Ordinary storage needs no
guessed byte, depth or list ceilings. Fixed arrays, checked indices, `Box`, exact
pairs and tagged named variants use the same checked seed path. Tag-first
canonical objects never require an arbitrary
object buffer. Complete closure admission rejects an unsupported opaque child,
including an unselected variant. Pure schema owners need no Execution or Query
dependency for these descriptions. Native canonical projection reads dispatch the exact
installed codec under the original request's cumulative execution allowance; ignored
admission failures veto completion. Opaque owners must supply admitted seeds.
Use an explicit field `worth_value(delegate = ...)` for a real domain-owned
constructor. For sealed original-text documents, `native_envelope: Envelope`
delegates the pure owner's `CanonicalDocumentCodec` readers and exact failures.
For derived readiness, use `WorthQueryOutputReadinessContractBuilder::for_field`
and typed `field_dependency` entries, then `build_typed`. The generated contract
is sealed against the actual installed field and complete aspect catalogs before
configuration. Its `delivery_ordinal` distinguishes delivery selection from
eligibility and uses the actual canonical node order. See
[canonical values and derived readiness](build-an-application.md#canonical-nested-values-and-derived-readiness).
Continuation and live delivery retain decode custody through their native final
handoff. For cumulative reads of a retained invariant snapshot, use
`snapshot.read_fields(&execution, |read| ...)`; its scoped `read.field(...)`
calls borrow the original pin and refuse the entire output after an ignored
decode failure or late cancellation.
The installed `value_meaning()` roster checks executable and native field
contracts; native canonical codec digests carry the same emitted meaning.
Use the retained catalogue's `same_installation_as(&other)` to compare its actual
immutable backing. Clones match; independent installation or restore does not,
even with equal field meaning. This grants no request, snapshot or settlement authority.
Producer readiness uses the actual output-role carrier and its emitted meaning.
Predecessor comparison requires explicit semantic aspect revision advancement
for changed meaning and grants no recovered-transition permission.
The existing [checkpoint transition](build-an-application.md#44-checkpoint-program-transitions)
supports whole-output derived-codec retirement when authenticated native
Create/Preserve publication succession still covers every present field.
Complete membership is independent of individual data writes: unchanged
companions and genuinely executed no-data Preserve retain their original
identities, while no-contact reuse retains its performed provenance. Dynamic
families may allocate and retire members through actual native operations;
each member retains its own birth and exact predecessor write positions.
An intervening ordinary write, including equal bytes, revokes continuity.
Only a genuine complete native codec retirement can close that generation for
fresh target creation; ordinary deletion or producer Retire cannot free it.
It preserves authored bytes and identities on
the same carrier and requires the installed target producer to rebuild output.
A closed set of native producer-owned internal relations can retire with its
output records when both endpoints are independently proven in that same
publication. Related ordering can survive when its key codec stays unchanged.
A producer owns every relation it actually allocates/creates touching at least
one output, including new boundary edges during Preserve; external endpoints
remain external entities. Before/candidate/cold association authenticates the
complete relation roster and its birth/write succession. Independently committed
crossing references may survive in either direction without being adopted; exact
native creation/allocation distinguishes them from omitted producer ownership.
Internal and claimed-owned relations require complete native provenance.
Destructive retirement remains closed: external or rewritten relations, protected
or partly classified fields, affected invariants, changed ordering keys and join
consequences refuse before publication; ordinary restore remains exact. Surviving application relations
are selected with the same bounded native reads as entities and rejudged through
`RelationMutationIntent::Revalidate(RevalidateRelationIntent { relation_id })`.
This native read demand schedules installed rules against the actual preimage;
it changes no endpoint, field, record version or lineage and claims no write or
rollback restoration. Rules that overlap retired relations remain refused. Independently produced
surviving output requires complete native original association and exact present
identity/kind/payload plus compatible installed roles. It retains historical
correspondence without readiness facts, through normal capture and the same
funded deferred/acknowledged repair capsule. Every disposition first validates complete original native publication membership,
including present optional outputs. Mixed or incomplete outputs refuse;
only real installed producer publication can establish target currentness.
An existing continuation resumes
through its original runtime and retained index generation, while a new target
request reads only genuinely regenerated target output. See the
[historical query lifecycle](build-an-application.md#44-checkpoint-program-transitions).
See [canonical nested values and derived readiness](build-an-application.md#canonical-nested-values-and-derived-readiness)
for supported grammar, native checkpoint examples and the current package
archive refusal. Representation limits do not grant request resources,
currentness or codec-transition permission.

Managed computation values derive resident accounting with Host's existing
`application_contribution::ChargedBytes` trait and derive. Every resident field
counts, independently of Serde visibility; named opaque/shared owners retain
their native contracts. Pure schema owners use the same trait and derive from
`worth_foundational::facade` without an Execution dependency. See
[resident value accounting](build-an-application.md#resident-value-accounting).

`WorthQueryInvariantEntityIdentity` has its own native resident contract for the
shared entity-key allocation. Clones preserve that backing and its original
reader authority; resident description grants no read, currentness or mutation
authority and introduces no process-wide deduplication promise.

Platform storage owners separately admit newly copied nested payload through
`ExecutionAllocationPolicy::reserve_memory(bytes)` before copying, retaining
the returned reservation with that payload. Fixed arrays still admit inline
backing only. See [Execution payload custody](../crates/worth-execution/README.md)
for the ownership and refusal contract; this grants no graph or publication authority.

"Guide" links to the section that shows the exact calls. The "Never build"
column names the substitute that this capability makes wrong.

| I want to… | Use (facade path / call) | Guide | Never build |
|---|---|---|---|
| Declare a concrete operation's access once | `decl::worth_query_operation!(... input Input; access { ... })` generates Rust capabilities and the schema projection; `schema.declared_operation::<Operation>(definition)` installs it with explicit effect/aftermath policy. Declare the complete read/effect/precondition set. | [Operation access](build-an-application.md#21-declare-operation-access-once) | Duplicating capability macros and installed member lists; inferring access from handler bodies; using a projection-work invoice or compile capability as native authority |
| Fence a locator to one live Server assembly | Use `WorthServer::instance_identity()`, retain session discovery's `server_instance`, and install `WorthServerInstanceSelectionPolicy::RequiredForProductCalls` where product calls must carry it. Guard and generated catalog share the policy; session IDs share the same instance identity. | [Server instance selection](../crates/worth-server/docs/connect-another-backend.md#fence-a-live-server-instance) | Clock/counter namespaces, silently rebinding an old locator after restart, a late adapter-only check, or treating the selector as Query authority or persistence |
| Preserve product preflight/currentness refusals over HTTP | Server retains `WorthServerTransportDenial::product_operation_denial()` and projects `ProductOperationDenied` with `worth-server-product-surface-denial.v1` owner codes, basis facts and execution/recovery posture. | [Product-surface refusals](../crates/worth-server/docs/connect-another-backend.md#product-surface-refusals) | Parsing diagnostic text, reporting a stale basis as a missing route, inventing a completed product envelope, or retrying uncertain performed work |
| Attach a native program to Server readiness without extracting its runtime | Transport entry uses `worth-server`'s `WorthServerProductOperationDeclaration::with_program_application(program.clone())` and returns a finished adapter registration. Readiness reuses the authentic installed allocation and grants no operation authority; native requests still authorize execution. | [Connect another backend](../crates/worth-server/docs/connect-another-backend.md) | Reinstalling a graph, copying readiness/basis state, exposing raw mutation access to the host, or falling back to unleased work in a checked packet |
| Re-enter an application's genuine operation custody on product retries | Runtime-backed product mutation declarations use `.with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody)`. Every command passes fresh Server admission/readiness and enters the adapter; Server neither caches responses nor issues retry receipts. The adapter must retain actual native custody and freshly authorize its advancement/delivery. Reads always enter fresh authorization, even with a retry key. Eligible default `ServerReceipt` mutations and durable-executor replay keep their existing contracts. | [Application-owned retries](../crates/worth-server/docs/connect-another-backend.md#application-owned-command-retries) | Treating the flag as custody, permission, idempotency or crash durability; replaying protected cached responses instead of native readmission; recapturing original intent on retry |
| Complete an effectful business Back or global ending | Retain the original guarded owner; after real external completion, call `completed.accept_local_action(&fresh_request, workflow)`. The original binding's native admission—required abilities/policies or its exact installed capability, never Advance—admits exact route/custody settlement. Inspect returned progress and retain cleanup; exact replay never repeats business effects. Suppressed effects confirm their original joint settlement. | [Local action completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#complete-a-local-action-with-an-external-effect) | Omitting a required effect; inventing a second role capability; restoring spent authority; using a callback as permission; bypassing another pending owner; a second host cursor update |
| Attach a fresh record to the admitted creator | `creation.link_to_principal::<Binding, Relation>(relation, key)` binds the exact mutation principal and carries its observed target through publication. Root and peer creations only; fresh schema and typed installation validate the correspondence. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) | Client-supplied actor IDs, organization-member searches, host target resolution, or treating the link as a grant |
| Atomically create a retained audit beside a disposable selection | `creation.with_peer(entity, key, authoring_closure)` reuses typed initializers; `creation.link_from_path(path, relation, key)` binds a retained existing anchor. Reusing one site does not suppress independent peers. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) | Two commits, an audit owned by a disposable snapshot, host anchor lookup/effect planning, or skipping independent effects on snapshot reuse |
| Assign or clear an input-selected existing entity reference | `body.replace_relation_from_input(relation, required_readonly_identity, member, key)` admits an original Option member, resolves through the native equality owner, and retains current/selected endpoints into one candidate. Nullable outgoing max-one relations only; same endpoint preserves its edge. | [Existing-reference inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/existing-reference-inputs.md) | Host key-to-ID resolvers, two-commit replacement, deleting an assigned target, a dummy scope write, or equating target existence with eligibility/authority |
| Replace a disposable current snapshot and retire its predecessor | `creation.replace_and_retire_from_scope(relation, key)` reuses the tracked native predecessor selection for atomic Unlink/Delete/Create/Link. Requires explicit Delete and one-to-one relation cardinality. Ordinary `replace_from_scope` still preserves the old entity. | [Retiring selection replacement](../workspaces/worth-query/crates/worth-query/docs/authoring/retiring-selection-replacement.md) | A host delete after commit, global ownership inferred from one relation, deleting signed history, or claiming content-equivalent signature preservation |
| Preserve an unchanged disposable selection's actual identity | `creation.reuse_current_if_unchanged()` compares every declared field, including optional absence, through the native canonical-equivalence owner after input admission. Independent body effects still run. | [Complete-content selection reuse](../workspaces/worth-query/crates/worth-query/docs/authoring/complete-content-selection-reuse.md) | Host tuple equality, partial projections, caller-provided reuse decisions, ignoring provenance, or claiming unrelated writes cannot invalidate signatures |
| Create a record and its required child topology atomically | `creation.with_child(relation, key, authoring_closure)` declares flat prior-parent sites; `child.link_from_parent(relation, key)` adds an association from that same fresh parent. Existing native owners allocate all sites in the original admitted context and link retained created handles. | [Atomic child creation](../workspaces/worth-query/crates/worth-query/docs/authoring/atomic-child-creation.md) | Multi-commit default creation, duplicate children for current/history, runtime effect callbacks, allocating from a newly created context, implicit child outputs, or counting kinds instead of instances |
| Atomically write a computed value to a selected parent | `body.assign_value_at_path(path, required_field, &value)` retains the exact guard-free native path target and its destination-field dependency. Existing checked evaluation feeds the same candidate as scope/audit writes and signed creation. Captured AuditContext remains a preimage; certified Subject loci cannot be rewritten, including through aliases. | [Computed graph-target writes](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-path-writes.md) | A second host commit, copied parent revision input, target IDs as authority, last-wins aliased sinks, rewriting a certified Subject while minting evidence, or a second arithmetic evaluator |
| Write an exact Decimal from captured input | Use input assignments/initializers with a readable `ScalarAspectType::Decimal` field codec and its explicit native input profile; a transparently serialized string is borrowed exactly and validated before effects. Required and optional members are supported. | [Exact Decimal inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#exact-decimal-inputs) | Floating-point conversion, integer truncation, graph String fields posing as quantities, or treating selected text as admitted domain validity |
| Require captured input to match a parent's field | `body.expect_input_from_path(ApplicationQueryRootPath, field, member)` uses native forward/reverse/identity selection and tracked field comparison without changing the row's scope. Optional graph absence cannot match. | [Path-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/path-input-guards.md) | Copied parent truth on every row, first/latest parent selection, or host equality checks before mutation |
| Retire dependent current snapshots when a source input changes | `body.retire_owned_selections_on_input_change(field, member, owners, selection, maximum_owners)` coupled to the exact body input write. Native observation retains every selection and publication unlinks/deletes atomically, preserving holders. | [Owned-selection retirement](../workspaces/worth-query/crates/worth-query/docs/authoring/owned-selection-retirement.md) | Host cascade loops, partial-prefix cleanup, copied parent state, signature invalidation counters, or using deletion for retained historical evidence |
| Require a stored field to equal captured input | `body.expect_input(field, "member")`, ordinary native body installation and owning execution. Required or optional field must be present; the original captured flat scalar is admitted through the field's installed codec. Query retains the comparison fact through publication. | [Captured-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#compare-a-field-with-captured-input) | Host equality checks, re-resolving provider data during replay, a query invented solely to carry this guard, or treating a guarded input as authorization |
| Keep a business value unique | Schema `unique_field(entity, field)` with an equality-queryable field, then the normal authored body/owning action. Query retains intrinsic indexed dependencies for actual unique writes before signing/sealing; native lowering and publication enforce collisions and concurrent claims. | [Unique business values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#unique-business-values) | Host duplicate checks, granting supplier read access merely to enforce uniqueness, removing the constraint to permit creation, or treating a request key as business identity |
| Conditionally emit the exact operation input | `body.emit_external_input(effect, Some(native_condition))`, the operation's `.external_effect(effect, correlation_family)`, and normal `setup.operation_body::<Binding>()`. Met emits; Unmet/NotApplicable suppresses only that effect. Guard facts remain publication obligations. | [Authored conditional effects](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md) | Host-side guard/payload builders, a second business commit, global optional-payload fallbacks, decoded metadata as codec authority, or claiming outbox creation proves external delivery |
| Emit graph data absent from the command or public DTO | `body.emit_external_query_context::<Binding, _, _, _, _, _>(effect, exact_query_definition, condition)`, a payload binding with Value `ApplicationQueryExpressionSnapshot`, `execute_with_expression_context()` and the actual `expect_source`. The query is parameter-free and ExactlyOne. Query funds/prepares once and publishes the ordinary outbox with all native source facts. | [Native query-backed effect payloads](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md#real-example) | Copied DTOs or publicly constructed snapshots as source authority, another graph reader, source-free execution, re-encoding in candidate construction, or treating a committed outbox as completed delivery |
| Share one computed revision across business, audit and payload | `ApplicationOperationBodyValue::admit`, `body.assign_value`, `creation.initialize_value`, and `emit_external_evaluation` with `ApplicationOperationBodyEvaluationSnapshot`. Native Foundation evaluation prepares once and validates every installed destination codec. Unitless UInt64 results; at least one field sink. Computed scope bookkeeping may accompany signing, with old revision captured as AuditContext; computed signed-artifact initializers remain refused. | [Shared Computed Body Values](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-body-values.md) | Host/codec arithmetic, caller-supplied computed answers, copied expressions per sink, treating observed query data as the postimage, fabricating snapshots as authority, or rewriting a lifecycle Subject source while signing |
| Retain correlated native query values for expressions | `request.query(intent).execute_with_expression_context()`; inspect `observed_source.expression_context()`. Exactly-one one-shot result; values stay attached to recursive dependencies before any domain projector. The original row or `result_set_observation()` supplies them through `expect_source` or `expect_result_set`; these source contracts remain distinct. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md) | Reconstructing values from DTOs, zipped parallel lists, treating omitted data as empty/absent, or treating descriptive contexts as mutation/read authority |
| Inspect nested snapshot data in a pure payload codec | `ApplicationQueryExpressionSnapshot::field_path(&["owner", "name"])` follows actual nominal record types and schema field order, unwraps intermediate optional-one records, and preserves a final option. `ExpressionValue::record_fields()` distinguishes records from lists/entries. Lists are never implicitly indexed. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#inspection-and-debugging) | Generated record-name/position guesses, taking the first collection row, treating untrusted snapshots as query or mutation authority |
| Require correlated native query information | `operation.requirement_operands().query_context::<Binding, _, _, _, _>(name, query_definition)` consumes the exact parameter-free, exactly-one query declaration. Supply its sealed source with `expect_source` to guidance and execution; native facts remain publication obligations. | [Authored context requirements](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#real-example) | Host-built expression records or readiness booleans, a second validation loop, weakening source parameters through a callback, or treating a descriptive Met finding as workflow authority |
| Require values across a related collection | `operation.requirement_operands().outgoing_field_values(name, relation, child_field)` binds a tracked `List<Value>` or `List<Option<Value>>`; use the existing Foundation expression for length and all/any. Same requirement drives guidance and mutation admission, with membership/liveness/field publication dependencies | [Related-item values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#require-values-on-every-related-item) | Caller-supplied lists, host validation loops, count-only readiness, filtering away missing values, or treating independently collected lists as a correlated row API |
| Return the scope, one selected outgoing subject or a created row from an authored body | `decl::application_schema`: `body.preserve_scope_output::<Binding, Role>()`, `body.preserve_related_output::<Binding, Role>(relation)` (Preserve/ExactlyOne), `creation.create_output::<Binding, Role>()`; read the real receipt's `outputs_of::<Contract>()?.entity::<Role>()?`. Related selection is tracked once and retained through publication; replay preserves original identity. | [Scope-bound operation bodies](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#stable-entry-points) | Caller-selected output IDs, post-commit pointer lookups as correspondence, dummy writes, or treating a preserved identity as current approval/workflow authority |
| Supply original typed inputs for native workflow needs | `WorthQueryWorkflowOperationInputs::new(program_runtime)`, then `.with::<InstalledAction, _>(description)` for a binding-wide entry or `.with_at::<InstalledAction, _>(&qualified_input_node, description)` for an exact node, and `select(workflow, &required)`. Distinct qualified nodes may retain the same binding; ambiguous overlap is refused. Native needs select order and existing owners retain all custody. Source-completion actions only. Accept an opaque completion with `prepared_advance.accept_selected_operation(completed_selection)`; the occupied native owner retains fresh admission, exact receipt checks and cleanup on every refusal. This roster does not supply heterogeneous proposal target inputs: ordinary `OperationInput` edges still require the same portable type and exact source-proposal input identity. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) | Host stage/name dispatch, recapturing transferred input/key, homogenizing unrelated input contracts, treating a finite pack as a durable or atomic whole-command runner |
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build §3](build-an-application.md#3-declare-features-and-the-program) | [Consumer prohibition](#prohibition-declare-features-and-a-program) |
| Require a producer feature's Initial and Preserve roles once | Host `WorthQueryRequiredFeaturePair<S, Feature, Initial, InitialReadiness, Preserve, PreserveReadiness>`; program `.required_pair::<Roles>()`, `contracts.feature::<Roles>()`, `setup.feature::<Roles>(WorthQueryFeatureRuntime::new(initial, preserve))`. Both explicit role values use `WorthQueryFeatureRoleRuntime::new(handler, provider, readiness_configuration)`. Exact readiness producer association is derived; native predicates and output binding stay with their owners. | [Required output roles](build-an-application.md#required-initial-and-preserve-output-roles), runnable `topology_entry/examples/required_feature.rs` | Inferring required roles from `SUPPORTED`, a second producer roster, erasing requirements by omitting callbacks, or replacing native currentness/access with configuration booleans |
| Declare distinct captured workflow target inputs | `graph.captured_operation_input(&proposal, &target)` and the component equivalent declare exact control/guarded correspondence. `prepare_workflow_proposal_with_inputs(workflow, instance, &inputs)` publishes complete original target contracts through native proposal/replay/first-effect owners. Archive profile 8 carries the new edge; unchanged definitions keep profile 7. Recorded-actor admission and recovery retain the original proposal correspondence and require fresh grants. In-memory owning execution, not a durable whole-command runner. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) | Homogenizing unrelated inputs, treating archive or roster data as authority, weakening final identity checks, or using a host registry to bypass missing native support |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build §4](build-an-application.md#4-install-the-application-graph) | [Consumer prohibition](#prohibition-install-the-application-graph) |
| Identify a rejected operation declaration dependency | `decl::application_schema::ApplicationSchemaDeclarationDenial` preserves `MissingOperationProgramDependency`, `MissingOperationDecisionReadDependency` and `MissingOperationMutationPreconditionDependency` with owned `operation` and `ApplicationOperationDependencyLocus`. Target loci carry the original typed field/relation/precondition; whole-body failures remain explicitly `Body`. | [Declaration failures](build-an-application.md#45-declaration-failures) | Reconstructing a field from a test selector, replacing the native cause with a generic installation string, or treating diagnostic data as authority |
| Omit intrinsic invalidation preparation and retained-byte ceilings | `host::runtime::WorthQueryInvalidationResourceInstallation::new(None, None, positions)`. `Some(bytes)` supplies an explicit finite ceiling; `bounded(work, preparation, retained, positions)` supplies finite work and byte ceilings. Ordinary Memory Workspace and Consumer Kit installations also select no marking-work ceiling. Accounting, retained tickets, checked counters and request interruption remain active; absence grants no RAM or process-memory guarantee. Other application hosts retain their independently supplied policies. | [Build: invalidation byte ceilings](build-an-application.md#invalidation-byte-ceilings) | Use the original installation and resource owner; never substitute maximum-integer sentinels or release credit while old roots remain held. |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) | Rebuilding original fields; treating resource custody as permission; charging registry bytes as total descriptor heap |
| Compose one complete action | `ApplicationFeatureSpec::root::<S, F>()` or `at::<S, I, F>()`, then `.action::<Binding, Delivery>()`, optional `.locality_and_change::<Scope, Shape>()`, `.correspondence::<C>()`, `.evaluated_requirement::<Rule>()`, `.external_input::<Provider>()`, and `.finish_action()` before feature completion. Each clause is once-only; delivery is explicit. `ApplicationActionReference<S, I, F, B, D>` selects complete actual installed meaning without manufacturing it. | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) | Separate declarations for delivery and metadata; mutable declaration bags; duplicate portable action keys; a second registry or policy engine |
| Store fixed or discovered work beyond construction | `WorthQueryFixedOperation<S, Intent, Program, Root, Action>` and `WorthQueryDiscoveredOperation<S, Intent, Program, Root, Action>` retain the authentic runtime allocation and complete installed action. Keep an independent genuine runtime handle for fresh requests on `owner.branch()`. All refusals return actual custody; runtime close refuses `RuntimeKeepaliveRetained` while operation keepalive remains. | [Owning actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#retained-runtime-and-configuration-custody) | Fabricated lifetimes, a replacement runtime, operation runtime access as admission, cloned native obligations, Drop as cleanup or durable continuation |
| Observe a required output without driving its producer | Candidate `DecisionReader::output_availability(&native_producer, typed_demand)` and checked `WorthQueryComputationReader` equivalent retain the fresh source Query and native completion dependency in the same attempt; Current or positively qualified Pending, with typed refusal otherwise. Neutral public native source/completion and original-image retry are qualified; private CAD and Fresh-on-restore remain open | [Source-aligned availability](../workspaces/worth-query/crates/worth-query/docs/capabilities/source-aligned-output-availability.md), compiling `topology_entry/.../output_availability/collection.rs` | Missing-head or denial-to-Pending mapping; a copied observation, boolean currentness cache or separate watcher |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build §5.3](build-an-application.md#53-execute-through-the-program) | [Consumer prohibition](#prohibition-run-a-mutation) |
| Omit a request-memory ceiling without disabling accounting | `ExecutionBudget::new(workers, None, work)` and `SerialMemoryBudget::new(None)`. `Some(bytes)` is an explicit finite request ceiling. Serial descendants and leases charge the original shared ledgers, including any finite ancestor or process ceiling; checked live-byte overflow remains a distinct `MemoryLimitDenial::CounterOverflow`. Worker counts, concrete operation declarations, backing ownership, cancellation and deadlines remain enforced. | [Request-memory policy](build-an-application.md#optional-request-memory-policy) | No maximum-integer sentinel, host-RAM heuristic, substituted request, or early release of a live charge. |
| Retain a result whose heap size is not known before evaluation | Original `ExecutionRequest::run_measured_result(ceiling, known_bytes, &mut reservation, kernel)` reuses Scope admission and lends `ExecutionResultMemory` on that same reservation for before-growth claims. It measures opaque callback results/errors before framework retention and leaves their actual heap charged until release. This is post-materialization admission unless the source cooperates before allocation; declared Scan/Map maxima remain separate real contracts. | [Request-memory policy](build-an-application.md#optional-request-memory-policy) | No fabricated prior quote, zero-heap wrapper, callback replay, or unreserved ownership transition. |
| Author ordinary work without an intrinsic operation-count ceiling | Omit `.operation_projection_work_budget(..)`; use `RelationalInvariantWorkBudget::new(None)`. Original `ExecutionBudget::new(workers, memory, None)` carries absence through serial/leased scopes and children; Signal attempt, waiter-resolution and upstream-dependency work are absent in presets while slots/bytes stay physical. Compiled contracts, archives, default reads, output demand, currentness, Fixed delivery and child invalidation preserve absence. Explicit finite limits still narrow; physical custody, validity and authority remain enforced. Invariant `Context::check_live()` / `ScopePlanner::check_live()` expose the original serial/leased request interruption as native `CustomInvariantInterruption::{Cancelled, DeadlineElapsed}` without charging work | [Optional invariant work and original interruption](build-an-application.md#optional-invariant-work-and-original-interruption) | MAX/zero absence sentinels; substituted requests; ignoring validity or physical admission; charging numerical work in a pure interruption callback |
| Retain ordinary World populations without an application quota | `host::runtime::WorthQueryProductWorldResources::new(RuntimeWorldBudgets::without_policy_limits(), clock, invalidation, execution_policy)` with `WorthQueryInvalidationResourceInstallation::new(None, None, None)`; ordinary Relational profiles omit published/active snapshot and prepared-candidate quotas; explicit finite policies remain available | [World policy ceilings](build-an-application.md#runtime-world-policy-ceilings) | MAX/zero sentinels; workload-fitted caps; dropping original observations to admit work; bypassing actual storage or ownership |
| Omit ordinary Query host quotas while retaining native storage accounting | Default query/output-demand profiles omit host quotas, including registry/lineage retention and settlement iterations. Signal conditional evaluation/temporal and Bridge retention presets likewise use `None`; finite setters/`bounded` constructors and explicit `Some` fields narrow. Original request funding and checked native retained-storage accounting remain. Result-buffer evidence reports absent policy as `None`; concrete buffer claims stay counted. Convenience settlement follows native `Progressed` on the same request, parks on `Pending`, and honors explicit finite step boundaries. Bounded parallel pair kernels use genuine finite per-read/batch result bounds, never estimate-derived ceilings | [Ordinary Query policies](build-an-application.md#ordinary-query-resource-policies) | Workload-fitted defaults, MAX/zero sentinels, dropped originals, uncharged backing, or repeated blocked settlement |
| Run a declared fixed action whose source feeds required outputs | Declare `.action::<Binding, Delivery>() ... .finish_action()` and its exact `ApplicationActionReference`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Construction reuses the native governed root-artifact/source guard before effects; invalid locality/change/dependency metadata returns the original description with `WorthQueryOperationConstructionDenial::OutputSource`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports optionally bounded `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis; eligible Required fixed-leaf terminal `close` invokes native unavailable completion, preserving actual negative evidence across cleanup refusal in `output_unavailable()` on terminal or closed custody. Source saved with no current output is not positive settlement | [Build §5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/…/application_program/required_unpublished.rs` | [Consumer prohibition](#prohibition-run-a-declared-fixed-action-whose-source-feeds-required-outputs) |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) | [Consumer prohibition](#prohibition-declare-discovered-output-delivery-meaning) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) | [Consumer prohibition](#prohibition-own-a-discovered-action-through-recovery-fanout-delivery-and-cleanup) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` | [Consumer prohibition](#prohibition-validate-an-explicitly-driven-discovered-aggregate-at-one-retained-observation) |
| Own a declaration-authorized no-output action | Declare `.action::<Binding, Delivery>() ... .finish_action()` with `ApplicationSourceCompletion`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode uses `advance(&fresh_request, maximum_history_work: Option<NonZeroUsize>)`; `None` adds no history work ceiling, while a finite bound remains a caller choice. It delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) | [Consumer prohibition](#prohibition-own-a-declaration-authorized-no-output-action) |
| Retry a mutation safely | Retain the move-only operation returned by `Progress` or `Refused`, and supply fresh request authority to its next `advance`. Delivery refusal retains the settled owner; retry its declared read. Terminal custody exposes evidence and close, without advance. After process loss, checkpoint replay requires the full stable original native intent to be re-expressible under fresh authority (tested lawful no-source/unpinned cases, with all other intent material unchanged). Row/result-set source expectations bind the old runtime; a new observation is different intent, not promised historical replay. Do not serialize live owners, substitute a source posture the binding forbids, or change key/source to force replay. A current read alone cannot rule out an ambiguous external effect: reconcile its authoritative outcome through the existing native/domain/Store recovery owner before choosing a new operation; never automatically reissue an uncertain effect under a new key. Store durable restart and process-local owning custody remain distinct. | [Owning entry](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) | [Consumer prohibition](#prohibition-retry-a-mutation-safely) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) | [Consumer prohibition](#prohibition-control-who-may-do-what) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` | [Consumer prohibition](#prohibition-require-different-people-for-different-duties) |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) | Comparing an input actor, treating absence as different, or reauthorizing the historical actor as the current caller |
| Create a subject and enroll it atomically with owning recovery | Declare `creation.enroll_in_workflow::<Spec, Binding, Role>()`, install `birth_operation`, and prepare native `RequiredWorkflowBirth`. Capture one original descriptor, then `for_workflow_birth::<Spec, Role>(&workflow, &required)?.retain_operation::<Program, Action>(program)` selects the declared SourceOnly owner. Frozen input/key and actual one-shot birth authority survive refusal, keyed/unpublished recovery and cleanup; `into_parts` recovers effect-free bound custody. | [Workflow-enrolled creations](../workspaces/worth-query/crates/worth-query/docs/authoring/workflow-enrolled-creations.md) | Separate create/start commits, a host retry state machine, re-encoding original input at binding, treating a published definition as permission, or consuming the birth slot again during recovery |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build §6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) | [Consumer prohibition](#prohibition-run-a-multi-step-process) |
| Start a fresh episode after a completed workflow | Scope/Preserve body `.succeed_workflow::<Spec, Binding, Role>()`; install `.succession_operation::<Binding, Role>()`, then native `prepare_workflow_succession` / `for_workflow_succession` with the same logical input/key. Preparation accepts the runtime-owned published reference or original pure start descriptor; after checkpoint restore, the latter requires fresh command admission and tracked native lookup. | [Completed-chain succession](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#completed-chain-succession) | Reviving the old instance, two host commits, inheriting approvals or effects, selecting an arbitrary historical definition, using an older predecessor to fork a chain, carrying old runtime handles across restore, or treating a descriptor as permission |
| Retain the actual outputs of a performed workflow operation | Native guarded settlement records the binding and actual role/entity/posture/reference inventory atomically with the transition; Existing, Created and Retire identities survive audit retention. No additional supplier flag or host write. | [Retained operation correspondence](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-operation-correspondence) | A receipt digest or current business lookup as historical identity; treating retained output as current-chain proof; rebinding foreign runtime instance handles |
| Require succession to continue the selected round or revision | Body `require_predecessor_output::<Binding, SelectedRole, Producer, ProducedRole>()` compares its sealed Preserve/ExactlyOne output with the latest performed occurrence of the declared guarded producer; both roles must name the same entity. Exact replay precedes fresh mismatch refusal. | [Succession output affinity](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-succession-output-affinity.md) | Caller-supplied historical IDs, root identity as round identity, fallback to an earlier occurrence, or a host-current-chain table |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `…for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build §6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-no-signature) |
| Certify shared and row-local facts without copying their source truth | A native signed creation uses `capture_from_path(ApplicationQueryRootPath, source, snapshot, Subject/AuditContext)` with v3 graph provenance. The existing lifecycle derives source-identity and selection-scoped supersession; `bound_subject_impacts` bounds the union of affected subjects. | [Graph-certified subjects](../workspaces/worth-query/crates/worth-query/docs/authoring/graph-certified-subjects.md) | Denormalizing shared facts to imitate scope-only signing; host dependency lists or invalidators; first/latest selection; treating decoded provenance as authority |
| Own capability delegation or revocation across fresh requests | `request.retain_capability_delegation_in_program(..)` / `retain_capability_revocation_in_program(..)` own original input, key, conditions and principal association. `advance(&fresh_request)` returns progress, same-owner refusal, native completion or terminal custody; `close_with_evidence` discharges cleanup before moving the receipt | [Owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands) | Flat keyed reissue loops; storing a request/grant; cloning a command; using an observation as permission; treating Drop as cleanup |
| Own a guarded workflow effect across fresh requests | Effect-free bound construction refusal returns the original binding; consuming `into_parts()` extracts its original description and requirement beyond the runtime borrow. `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) | Cached prepared admission; recreating an outbox handle after refusal; reissuing a spent slot; treating effect completion as workflow terminality |
| Inspect a workflow source independently of dispatch recovery | `WorthQueryWorkflowOperation::inspect_source(&fresh_request)` freshly admits the original binding and resolves native custody, then borrows its retained source evidence with `Committed`, `DispatchPending` or `ExternallySettled` posture. No recovery handle is minted and no delivery, settlement or cleanup occurs. `SourceEvidenceUnavailable` means completion has not been established, not that no effect happened. | [Source inspection](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#inspect-source-without-advancing-dispatch) | Reconstructing a receipt from business rows; interpreting pending dispatch as rejection or delivery; skipping fresh authorization; discarding the original owner |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) | Taking cleanup from a diagnostic denial clone; equating source save, observer close, evidence acceptance or approval |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event. For a compiled signed body on that approval control, use `prepare_workflow_approval_with_body(..)` to co-publish its attestation and approval under one event; exact settled replay needs no new event. Its evidence join accepts one or more distinct assessment inputs; empty applicable evidence never grants approval. | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Approval with attestation](../workspaces/worth-query/crates/worth-query/docs/authoring/signed-operation-creations.md#one-approval-action-with-a-business-attestation) | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-a-signature) |
| Prepare and publish an actual mutation | `OperationHandler::decide` / `prepare_decision` reads through `DecisionReader` and carries typed targets, owned keys and prepared values in its `Decision`; `build_candidate` consumes them through `CandidateWriter`. The framework opens the effect program directly and preserves checked sizes, cancellation, fallible growth, invariant validation and atomic commit. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects) | Untracked existing-entity targets, effects outside declared authority, fabricated output correspondence or a second mutation path |
| Publish typed fields, links and emissions | Use the original `CandidateWriter` under the operation's declared create/write/relation/`OperationEmits` authority, then bind its actual fixed output roles or family members. Payload validation and output cardinality remain enforced. | [Build: mutation bindings](build-an-application.md#52-mutations-intent-binding-handler) | Treating data or diagnostic text as authority; publishing external projections as ordinary emissions; bypassing output validation |
| Act on time or a signal | A conditional operation: `.conditional_operation::<Op>()` on the feature spec, `host::worth_query_conditional_node!`, and a host-supplied predicate and named clock | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Conditional installed operations](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md) | [Consumer prohibition](#prohibition-act-on-time-or-a-signal) |
| Complete work from an external system | An installed inbound source: `runtime.inbound_occurrences().receive(source, bytes, scope).execute()` (`host::application_entry::WorthQueryApplicationInboundOccurrencesExt`); in a workflow, `await_inbound::<Effect>(..)` | [Aftermath: authenticated inbound completion](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#authenticated-inbound-completion), [Workflows: waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion) | [Consumer prohibition](#prohibition-complete-work-from-an-external-system) |
| Survive a restart | The app owns the artifact. Capture: inside `runtime.with_host_advancement(..)`, call `runtime.capture_application_checkpoint_with_sections(&phase, policy)` with a `host::application_installation::WorthQueryCheckpointCapturePolicy`. Restore: `in_memory_program_from_checkpoint(..)` or `in_memory_rostered_program_from_checkpoint(..)` | [Front door: capture and reopen](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#capture-and-reopen-cost-boundary), [Build §4.4](build-an-application.md#44-checkpoint-program-transitions) | [Consumer prohibition](#prohibition-survive-a-restart) |
| Work on a branch or move it to a new program under its declared capability | `runtime.branches().fork(runtime.current_world()).components(..).create()`; `request(..).on_branch(branch)`; `.programs()` then `compare(&target)`, `adopt(&requirements)?.prepare(max)`, `programs.publish(prepared)` | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Build §7](build-an-application.md#7-adopt-a-new-program-on-a-branch) | [Consumer prohibition](#prohibition-work-on-a-branch-or-move-it-to-a-new-program-under-its-declared-capability) |
| Own Bank deposit, withdrawal and send-money transport custody | Typed money builders use `retain_operation()` and explicit retained history bounds. Strict v4 create/resume/read/close commands store actual native owners under finite capacity; retain `BankHttpMutationContinuation` before activating the prepared listener. Fresh commands provide authority, while listener loss and shutdown refusal preserve independent resumption | [Owning HTTP money](../workspaces/worth-query-bank-world/docs/http-owning-money.md), [Bank consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) | [Consumer prohibition](#prohibition-own-bank-deposit-withdrawal-and-send-money-transport-custody) |
| Own plain Bank account, access or payment mutation | Bank builders for personal/business account creation, access grant/revocation, initiation and either rejection form use `retain_operation()` and native SourceCompletion; supply fresh authority to each `advance` and `close`. Fresh results and recovered receipt evidence remain distinct | [Bank owning completion guidance](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-account-access-and-payment-completion) | [Consumer prohibition](#prohibition-own-plain-bank-account-access-or-payment-mutation) |
| Own Bank estate source or retained funding/reversal | Typed Bank mutation builders use exact declared SourceCompletion or existing RetainedCommit; `close_with_evidence` transfers original completion only after native cleanup | [Bank source and publication handoff](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-estate-sources-funding-reversal-and-deployment-revocation) | [Consumer prohibition](#prohibition-own-bank-estate-source-or-retained-funding-reversal) |
| Retain Bank HTTP native cleanup and business evidence | The listener-independent mutation continuation stores actual source and elevation cleanup owners under finite custody. Fresh `release_cleanup` retries on the original execution thread; the local `take_released_cleanup` handoff preserves actual uncommitted evidence and the installed runtime after physical release | [HTTP cleanup custody](../workspaces/worth-query-bank-world/docs/http-owning-money.md#estate-and-native-cleanup-custody) | [Consumer prohibition](#prohibition-retain-bank-http-native-cleanup-and-business-evidence) |
| Issue or revoke Bank program deployment permission | Explicit `BankProgramDeploymentGrantSeed::for_program::<Source>(..)` in `BankWorldSeed`; recorded issuer calls `revoke_program_deployment_grant(..)` | [Build §7.1](build-an-application.md#71-the-call-chain) | [Consumer prohibition](#prohibition-issue-or-revoke-bank-program-deployment-permission) |
| Read current data | `request(..).query(intent).execute()`; exact reads with `retain_read()` and `at(&observation)`; live reads with `query(intent).subscribe(..)` | [Host README: ordinary typed query entry](../workspaces/worth-query/crates/worth-query-host/README.md#ordinary-typed-query-entry), [Host README: output demand and live reads](../workspaces/worth-query/crates/worth-query-host/README.md#output-demand-exact-observation-and-live-reads) | [Consumer prohibition](#prohibition-read-current-data) |
| Read the original workflow instance at a retained observation | `request.at(&observation).read_original_workflow::<Spec, SubjectRead, AuditRead>(workflow.vocabulary(), &original_intent, subject_input, audit_input)` uses fresh installed read bindings before lookup. `Live`, `Historical` and authorized `Missing` retain exact basis data; audit is independently admitted and otherwise explicitly withheld. | [Governed original-intent reads](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#governed-original-intent-reads) | Supplying an actor/principal entity as read authority; querying a latest basis behind a retained observation; treating a view or live locator as a command grant |
| Read bounded workflow history and reacquire proposal references | `request.at(&data).read_workflow_history::<Spec, SubjectRead, AuditRead>(vocabulary, target, subject_input, audit_input, width, indexes, after)` returns exact-basis pages with independently withheld audit. `subscribe_workflow_history` / `poll_workflow_history` use native envelope dependency marks and fresh disclosure. | [Retained history](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-proposal-and-approval-history) | Treating a recorded locator, decision or actor as permission; refreshing a retained basis on a mark; retaining a revoked read grant |
| Read one workflow definition's metadata | Retained `discover_workflow_definition` with explicit typed `ApplicationWorkflowDefinitionReadTarget` and the separately installed `DefinitionMetadata` binding; fresh admission precedes selected-index lookup. | [Governed definition metadata](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#discover-and-retire-definitions) | Borrowing subject permission for metadata or whole-inventory disclosure; execution-only lookup |
| Retry committed discovered-source retention | Ordinary `advance` retains and retries the native owner. For deliberate native control, retain the opaque `WorthQueryDiscoveredOutputRetentionFailure`; use a fresh original `advanced()` native request's `retry_discovered_output_retention(failure, application)`. Refusal returns the original owner; success promotes its actual carrier before starting native outputs | [Committed discovered retention](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#committed-discovered-retention) | [Consumer prohibition](#prohibition-retry-committed-discovered-source-retention) |
| Retain an application preview through readmission, discard or promotion refusal | `runtime.begin_preview(&phase, &observation, request)`; `preview.readmit(&phase, &runtime)`; `readmitted.promote()`; either phase's `discard()` returns its same owner on refusal. Fresh readmission validates the original source and runtime; promotion finalizes retained preview evidence, separately from source/output publication. Rejected preview remains explicitly discardable. Name its native result vocabulary through `worth_query_host::facade::application_installation::{BridgeSpeculationError, BridgeSpeculationErrorKind, BridgeSpeculativePromotionOutcome}` | [Preview custody](build-an-application.md#56-retain-preview-custody-across-refusal) | Auto-discarding on an admission stop; reconstructing a preview from its identity or receipt; treating Drop/liveness as successful cleanup; promoting stale source as fresh output |
| Own a complete no-source output tree | `request.demand(original).controls(controls).retain_program_outputs::<Program, Root>(&program)` retains the actual installed full tree. Owned construction refusal supports fresh `retry` without replacing an issued basis. `advance` returns pending/refused/settled/terminal custody. Settled `read_current` checks all original outputs and discovery before reading at the same native observation and phase. Every phase can `close`; an owned close refusal supports `retry_close`, including after source supersession. Native unpublished and uncertain obligations remain owned; close delegates lawful original native cleanup/recovery and otherwise refuses | [Complete no-source trees](build-an-application.md#55-own-a-complete-no-source-output-tree) | Treating `start_in_program` root-only settlement as a full tree; fake no-op source actions; receipt reconstruction; dropping native recovery custody; claiming current output from read-request close |
| Close original complete output custody before fresh final delivery | Settled Fixed and complete no-source parents expose `close_for_delivery(&fresh_request)`. Actual native cleanup/interest close precedes sealed `WorthQueryClosedFixedDelivery` or `WorthQueryClosedProgramOutputDelivery`. Their existing-shape `deliver_current`/`read_current` methods validate the original full tree and installed meaning at a fresh native observation. To show several closed carriers current at one common basis, each no-projection Fixed and no-source carrier's `deliver_current_at(&request.at(&observation), policy)` runs that same installed-action or selected-root gate and full lineage at the caller's observation. Fixed close refusal returns the same parent; no-source `WorthQueryProgramOutputDeliveryCloseRefusal` supports close-only `retry_close` | [Delivery after native close](build-an-application.md#delivery-after-native-close), Host `closed_delivery_contract` construction/phase/shape family | Reopening released interests, replaying source/producer work, treating root evidence, commit equality or bare `require_current_program_output` lineage as complete currentness, retaining readonly history through whole-runtime shutdown |
| Inspect original root output roles without a producer receipt | `host::application_entry::WorthQueryApplicationProgramOutputSettlement::root_settlement()` borrows the actual immutable root demand settlement; use its `outputs_of::<Contract>()` typed correspondence even when `root_receipt()` is `None`. Parent operation retains all tree/close custody; this root-only evidence does not establish fresh complete-tree currentness | [Original root output evidence](build-an-application.md#original-root-output-evidence) | Reconstructing receipts, querying keys as output-role proof, starting a replacement demand, or treating a root borrow as aggregate currentness/discharge |
| Poll cancellation/deadline in a decision handler | `DecisionReader::checkpoint()` polls the real request. Preparation retains its first failure and refuses before candidate authoring even if caught. Successful polls preserve completed-decision input-reuse eligibility under the existing producer/context contract. | [Build §5.1](build-an-application.md#51-who-is-asking-principal-and-request-scope) | Treating a caught interruption as publication permission or declaring untracked request data trusted |
| Bind a derived collection to checked computation | `ApplicationDerivedCollection` requires `Computation` and `Completeness`; declare the computation beside the collection and install with `setup.derived_collection::<Feature, Collection, _>(owner)`. The owner's output and stop types must match the completeness contract. Initial/Create and Preserve/Preserve producers may share that installed owner and operation with a common no-intent input: implement `ApplicationMutationInputScope<Schema, Binding>` for each selected mutation binding. Ordinary intent inputs delegate their existing single-binding scope. Scope selection grants no execution authority. Complete installed operation/workflow-stage target coverage is checked before initialization, separately from exact per-binding route inventory; `installed_conditional_targets_for_schema` exposes read-only full locations, and conditional declarations use `in_workflow_stage` for stage selection. Missing or misbound installation refuses before bootstrap effects; the existing keyed computation checks its completed value before publication. Native partition reuse selects retained facts from the exact settlement frontier and consumes validation at the same pinned reader snapshot; applications provide no dirty hints. Successful native reduction retains its Execution-owned run; a fresh descendant request advances only changed inputs and actual removals, sharing unchanged native results and fitting cost ranges. Fixed-membership tree updates need no full shape plan or deletion scan. Execution owns first-stop and tight-prefix charging; applications cannot supply a result handle or checkpoint tape. Context-sensitive or nested kernels execute rather than retaining unsupported custody. Fixed membership with unchanged keys also carries checked reader-call ranges and immutable item/call bounds; ordinary preflight/charges and exact reached-entity scope still apply. Qualified native postcommit rebase consumes paired original/postcondition custody, the complete snapshot-aligned frontier and actual candidate touches; fresh suffixes and changed leaf sizing remain charged. Missing association or continuity reconstructs in full. Qualified completed original physical reads share checked locator/effect routing through a genuinely fresh registered session. Both comparisons retain native source and registration checks; changed or unqualified source uses funded full original-fact comparison. Principal and source-role observations remain individual. Posting registration and membership processing remain separate obligations; key/membership changes retain full preparation. | [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | An identity-only declaration, unchecked computation registration, or a presentation view as aggregate authority |
| Inspect original producer publication or admission stops | Borrow the native demand denial inside the original advance result. `publication_companion_stop()` identifies a commit resource stop; `producer_admission_stop()` identifies native preparation/readmission stops, including structural causes. Both retain the original `host::primary_graph::CompanionPreflightStop` and supplied quote. `output_verification_capacity_stop()` retains the original `VerificationCapacityStop` for output verification, including precommit consumed edges and preparation/readmission: the meter stop, scratch-size overflow or allocation refusal remains distinct. Demand capacity evidence may overlap producer admission; the commit getter specifically identifies precommit verification. All three `None` leaves the cause unknown. Keep the original operation/cleanup custody | [Producer publication diagnostics](build-an-application.md#producer-publication-diagnostics), [Host borrowing example](../workspaces/worth-query/crates/worth-query-host/src/facade.rs) | Inferring a capacity from the producer name; treating diagnostic data as retry, publication or cleanup authority |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build §3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | [Consumer prohibition](#prohibition-produce-derived-or-computed-outputs) |
| Produce from authentic sources and declare artifact lifecycle | `WorthQueryApplicationProducerProvider` supplies semantic identity, operation input and idempotency from the original source. `ApplicationDerivedArtifact` declares output/locality, retention, succession, required posture, producer, dependencies, reuse and stopped outcome. The installed mutation path enforces source/currentness and publication contracts. | [Producer source and publication](build-an-application.md#producer-source-and-publication) | Fabricating source or current output; replaying a producer for work already reused; treating lifecycle metadata as mutation authority |
| Undo or redo a change | **Provisional.** `host::provisional_aftermath` exists, but undo and redo are experiments; Milestone 9.18 owns any accepted contract | [Front door: current limits](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#current-limits) | [Consumer prohibition](#prohibition-undo-or-redo-a-change) |

Action metadata retains its existing capability scope: locality/change and
delivery belong to the exact occurrence; correspondence lookup is binding-type
scoped; rule/provider lookup is operation-type plus rule/provider-type scoped.
A declaration neither evaluates a rule nor admits an external revision. The
installed evaluator supports guidance and submission enforcement, not a new
unbypassable native authorization gate. External values must be resolved and
revision-admitted before being used by the source action.

Deliberately specialized native declarations use `.native_action::<Binding>()`
and the same optional clauses followed by `.finish_action()`. Ordinary product
mutations, including retries formerly driven manually, require real delivery.
Native declarations have no output-source toggle. For a concrete binding issued
only by an installed conditional, use `.conditional_native_action::<Binding>()`
and `.finish_action()` (DSL `conditional_native_action(Binding) {};`). It carries
real binding membership with conditional ownership and no ordinary delivery;
installation still checks the exact conditional operation set and real binding,
handler and producer contracts. Configure initial workflow
vocabulary before cloning runtime handles or retaining operations. A refused
`retain_workflow_spec` preserves runtime, installed spec and signing owner in
`WorthQueryWorkflowRuntimeBindingRefusal::into_parts()`; refused
`support_workflow_spec` returns the original spec with its denial.
`WorthQueryWorkflowApplicationRuntime::close_conditional_runtime(&mut self)`
delegates native close while retaining the configured wrapper and its installed
workflow/signing companions on refusal. Close the actual operation owners and
retry the same wrapper; consuming it into a program runtime is unnecessary.

Before disclosing retained SourceOnly status, active, completed and terminal
owners expose `assess_current_authorization(&fresh_request)`. It checks the exact
installed action through the binding-selected conventional/capability owner,
using frozen input and original workflow context without staging, claiming a key
or advancing custody. Success is descriptive, not execution permission. See the
[owning entry guide](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions)
and the native SourceOnly authorization tests in `native_workflow_operations`.
Authentication or a cached command-owner match is
not sufficient for current disclosure authorization.

Completed source owners also expose `original_input()` as a read-only borrow of
the exact frozen encoded input, including recovered completion. It grants no
read, publication or reconstruction authority. Server's typed `CompletionData`
projection uses this only after completion and fresh authorization; active and
terminal custody cannot claim a created locator through this metadata. See
[source command custody](../crates/worth-server/docs/query-source-custody.md).

Generic hosts constrain bindings with `WorthQuerySourceOperationBinding<Schema>`
and call those ordinary owners directly. The sealed interface carries native
dispatch, not an authorization witness or a consumer-replaceable callback.
For bounded HTTP custody, `worth-server::mount_query_source(adapter, limits)`
retains real Query owners, rejects duplicate/key/capacity admission before
preparation, stores returns before replies and freshly governs inspection.
See [Query source custody](../crates/worth-server/docs/query-source-custody.md).
It is process-local and currently selects SourceCompletion, not required outputs
or an asynchronous signed-owner transport. Never serialize a live owner or copy
the worker into a supplier.

---

## Consumer prohibitions

<!-- consumer-prohibitions begin -->

This is the consumer prohibition source. Task and gap rows link here; local
lifecycle explanations describe behavior, not another permission policy.

### Prohibition: Declare features and a program

A second feature or action registry; runtime feature flags that decide which operations exist

### Prohibition: Install the application graph

A host-side schema or program catalog

### Prohibition: Run a mutation

A rebuilt mutation request per retry; a second effect/recovery owner; manual native execution as the ordinary path

### Prohibition: Run a declared fixed action whose source feeds required outputs

Rebuilding the intent or key after a partial; reading at a different observation from currentness validation; dropping a refused cleanup owner

### Prohibition: Declare discovered output delivery meaning

Treating a declaration or decoded description as runtime authority; replacing a concrete installed selector with a lookalike

### Prohibition: Own a discovered action through recovery fanout delivery and cleanup

Rebuilding requests or discovery after retention refusal; treating a receipt as an original carrier; running a replacement selector or pumping closed outputs

### Prohibition: Validate an explicitly driven discovered aggregate at one retained observation

Rediscovering roots at delivery; checking only surviving roots; treating `observation()` or an empty settlement iterator as complete-set proof

### Prohibition: Own a declaration authorized no output action

Rerunning an unpublished handler; treating uncertain `Unseen` as permission to execute; using a receipt as a retained World observation; substituting no-output completion for required outputs

### Prohibition: Retry a mutation safely

A new key per retry; a did-it-commit table; reexecuting effects after a lost reply; treating cleanup refusal as completion

### Prohibition: Control who may do what

Role booleans checked in a handler, UI, or transport; treating authentication as permission

### Prohibition: Require different people for different duties

Comparing actor ids from input in a handler. See [recorded workflow actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors)

### Prohibition: Run a multi step process

A host state machine or status column that decides the next step

### Prohibition: Run a workflow step that needs no signature

A dummy approval or auto-approving principal. Guarded bindings must declare workflow requirements explicitly

### Prohibition: Run a workflow step that needs a signature

A signature-shaped row or checkbox; one capability for several meanings of a signature

### Prohibition: Act on time or a signal

A host scheduler, cron job, timer loop, or wake registry

### Prohibition: Complete work from an external system

A callback that writes truth or resumes an instance; a correlation token as authority

### Prohibition: Survive a restart

Serializing live handles or references; mirroring platform truth in your own store as authority

### Prohibition: Work on a branch or move it to a new program under its declared capability

Copying data into a scratch table; a process-wide "latest program" switch

### Prohibition: Own bank deposit withdrawal and send money transport custody

Treating a custody ID as authority; queued timeout as not applied; dropping pending custody with a response; hidden pumping or reconstructed retained observations

### Prohibition: Own plain bank account access or payment mutation

Treating a payment continuation as authority; rebuilding a partial operation; manufacturing a recovered handler result

### Prohibition: Own bank estate source or retained funding reversal

Cloning a performed witness; treating published-effect recovery as source retry; replacing capability admission with an employee role

### Prohibition: Retain bank http native cleanup and business evidence

Treating physical release as business completion; expiring unique cleanup; moving retry to a foreign thread; restoring consumed elevation authority from tokens

### Prohibition: Issue or revoke bank program deployment permission

Reusing estate/payment permission; granting every authenticated actor; selecting the target’s permission

### Prohibition: Read current data

Reading lower runtimes directly; a cache treated as truth

### Prohibition: Retry committed discovered source retention

Receipt reconstruction, handler rerun or rediscovery to replace unregistered custody

### Prohibition: Produce derived or computed outputs

Recomputing in the app and storing the result as authoritative

### Prohibition: Undo or redo a change

A generic undo engine over committed history. A business reversal is an ordinary declared operation

### Application authoring constraints

- Do not import `worth_query_declaration`, `worth_query_installation`,
  `worth_query_admission`, `worth_query_execution`, `worth_query_publication`,
  or `worth_query`. Their consumer names are re-exported through the two
  facades (for example `worth_query_host::facade::domain` and
  `worth_query_host::facade::admission`).
- Do not install a program-less runtime as an ordinary application or coordinate
  fixed/discovered/source-only retries manually. Deliberate native control is described under
  [Advanced](build-an-application.md#advanced-native-mutation-control).
- Do not substitute an action or program meaning on retry. Native admission compares the retained action with the branch's selected program.
- Do not expect adoption to change sibling branches, or a branch set to adopt
  atomically.
- Do not register a handler for a workflow control binding.
- Do not wait for a background scheduler to run work for you. There is none,
  and no timer or callback advances a workflow instance: the caller pumps.
  Time- and signal-driven work is declared with `conditional_operation`: the
  host submits clock or predicate observations, Signal decides whether a wake
  is eligible, and Query freshly admits the installed operation. Do not build
  a host scheduler or wake registry.
- Do not treat a validated program or validated workflow definition as
  permission to do anything.
- Do not treat names found only in `plans/` as real. Check [§9](build-an-application.md#9-use-the-consumer-vocabulary).
- Do not build a substitute for a capability you could not find: no parallel
  registry, copied authority, host executor or scheduler, or host persistence
  of platform truth. Follow the missing-capability procedure in the
  [Capability Index](capability-index.md).
- Do not present `compare_and_commit_program_undo` / `_redo` as an accepted
  correction contract. They are provisional ([§5.2](build-an-application.md#52-mutations-intent-binding-handler)).

### Host boundary constraints

- create a second operating-world or application-authority root;
- call operation executors directly;
- expose raw Relational, Bridge, or Signal handles to application code;
- reconstruct Query authority from receipts, reports, projections, or digests;
- import certification replay through the host facade;
- construct lineage outcomes or promoted graph identities from raw identity
  material.
- treat acknowledgement, timeout, disconnect, or lost response as external
  completion;
- serialize a recovery handle or treat its opaque wire identity as live
  authority;
- teach `facade::provisional_aftermath` as accepted undo/redo support.
- schedule temporal work locally, return raw Signal decisions, or invoke a
  conditional operation directly.
- invent a host-local temporal binding or idempotency hash, or derive either
  again during commit.

### Ordinary entrance constraints

- importing internal Query authority crates from application code;
- treating a role, relationship, token claim, or Signal decision as permission;
- executing a provider or operation callback directly;
- reading protected fields and redacting them after projection;
- rebuilding cursors, continuation, or recovery authority from wire values;
- maintaining a local temporal scheduler or conditional wake registry;
- retrying an indeterminate mutation as though it definitely failed;
- teaching provisional undo or redo as an accepted application contract.

### Application authority constraints

- Never pass a program to a mutation. The branch decides which program runs.
- Never treat a report, token, digest, receipt clone, or discovery answer as
  authority.


### Prohibition: workflow-state

Do not mirror kernel state into your schema as truth or use a status table to decide whether to advance. Advance and match the typed outcome.

### Prohibition: workflow-references

Do not serialize, forge, or re-derive references. Do not build an instance registry and treat it as authority.

### Prohibition: workflow-actor

Do not treat a caller-supplied "actor" field in input as admitted identity.

### Prohibition: workflow-proposal-wait

Handle `AwaitingProposal(RequiredWorkflowProposal)` as a typed need: inspect its node, operation and input type, then use fresh `prepare_workflow_proposal` admission. Waiting grants no authority.

<!-- consumer-prohibitions end -->

For installation and checkpoint-transition work inside an already admitted original
execution request, use `host::application_contribution::with_bootstrap_advancement_from_request`.
Its bootstrap phase borrows the same cancellation/deadline and cumulative native work.
Call `host::application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement`
with that phase; capture funding derives from its request. The capture-bearing path
requires leased backing and refuses unsupported serial backing with the existing
`ExecutionDenied(ResourceEnvelopeMismatch)` vocabulary, before effects. Bootstrap
custody cannot admit installed-runtime execution. A performed recovery capsule's
`repair_to_checkpoint_with_request(original)` returns that same capsule on opening,
settlement or capture refusal. Existing convenience constructors still own their
explicit bootstrap request and capture policy. See [original-request transitions](build-an-application.md#44-checkpoint-program-transitions).

## Known gaps

**Selective compiler fingerprints.** The supported ordinary guard is the
whole-application `ApplicationBuildIdentity`: the build pipeline embeds one UUID,
and installation binds it to every producer automatically. A changed build may
conservatively invalidate all derived outputs. Follow the
[build identity pipeline contract](build-an-application.md#application-build-identity);
consumers do not need the standalone compiler observer or a second edition roster.
Per-crate/selective code fingerprints are not promised by this API. The identity
is descriptive; native authority, source-format compatibility and currentness
remain governed by their existing owners.

Check the support posture in the capability table before concluding that a
capability is missing. Follow the linked current owner and consumer guidance.

**Domain concerns that are not platform gaps.** Document these as patterns in
your application:

- The meaning of a signature is its approval capability. Declare one
  capability per meaning.
- A reason for a change is domain data in the proposed input. The signature
  covers it through its input identity.
- Worklists and queues are derived views of domain state.

---

## Reporting a gap

If the five steps above leave you without a way, report it. Write:

1. **The task**, in domain-neutral terms (for example "a document must be
   signed by two different reviewers").
2. **What you searched**: the facade modules, rustdoc, examples, and this page.
3. **The nearest existing API**, with its facade path.
4. **Why it does not fit**, in one or two sentences.
5. **The known-gap row** it matches, if any.

Open an issue in this repository with that content, or give it to the platform
owners.

Meanwhile:

- Stop the part of the work that needs the capability. Finish the rest.
- Do not ship a substitute: no parallel registry, copied authority, host
  executor, host scheduler, or host persistence of platform truth.
- An application may keep domain data in its own schema. It must not mirror
  platform truth (workflow state, grants, commit outcomes) as authority.

---

## Which source wins

When two sources disagree, the higher one wins. Report the disagreement.

1. **Compiled source and its rustdoc.** The facades and their exports are the
   API.
2. **Blocks marked `<!-- compiled: X -->`** in
   [Build an Application](build-an-application.md). CI checks each one byte for
   byte against a fixture that compiles.
3. **The [Glossary](glossary.md)**, for the meaning of terms.
4. **Guides**: this page, the build guide's other blocks, the Query guides, and
   crate READMEs. Their names are checked by review, not by a compiler.
5. **Plans** under `plans/`. A plan is intent, not API. Never treat a plan as
   proof that a capability exists.

---

## Keeping this page current

Platform contributors who add a consumer capability add its row here in the
same change. Before adding one, check this page and the known gaps, so the
platform never gains a second way to do the same thing.

Governed workflow disclosure source now separates definition-selected retained
pages (`read_workflow_instances`, explicit Subject/Definition scope) from private
complete adoption safety inventory. Adoption accepts opaque-reference choices
or explicit native carry-compatible intent, and refuses with non-disclosing
workflow denials. Metadata grants do not authorize instance enumeration or whole-program inventory.

### Explicit optional aspects

Applications declare whole-aspect absence with
`ApplicationSchemaDeclarationBuilder::optional_aspect` through `worth-query-decl`.
`aspect` remains Required independently of each field's presence. Installation,
value meaning, predecessor comparison and current package archive record protocol8
retain this law. Native producer births with absent payload require every actual
binding to be a supported Optional EntityField struct; Required and Defaulted
aspects do not acquire absence permission. Tracked reads retain Optional whole-aspect absence independently of field absence; another field materializing that aspect invalidates the read. See the compiled `optional_aspects.rs`
example in [build guidance](build-an-application.md#whole-aspect-absence).

Ordinary Query reads use installed result cardinality and fresh per-read admission;
`request.query(intent).limit_results(n).execute()` narrows that cardinality.
`request.query_batch(intents).execute(&phase)` issues exact installed read slots
without caller work or byte estimates. Slots carry no authentication or currentness.
Live declarations name cause/binding/effect/scope/target meaning without a resource
contract. Request Live controls retain channel buffer capacity and materialized-record
cardinality; their managed queue occupancy is released on delivery or normal close.
For ordinary derived readiness, use the
[typed field builder](build-an-application.md#canonical-nested-values-and-derived-readiness)
against the completed declaration. The lower-level
`WorthQueryOutputReadinessContractBuilder::new` accepts a native projection for
families that own that projection directly. Readiness is synchronous, effect-free
native execution with no work/byte quotation.


### Graph-read admission in the default Query workspace

`worth-query-admission::integration::review_graph_read_access(requirements, inventory)`
reviews native index support without a caller budget or cost estimate. The legacy
`worth-query` receiver preserves the requirement set, inventory match, authority
receipt and typed async, persistent-index, capability-registration or unsupported
refusal. Its execution receipt retains the admitted plan and actual executor
observations. It no longer presents estimated index bytes as an allocation receipt
or creates materialized row proofs from estimated result counts. The retired
`graph_read_materializations` simulator is not an asynchronous executor; an async
requirement still refuses ordinary inline execution until its native support exists.
