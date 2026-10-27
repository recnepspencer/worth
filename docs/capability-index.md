# Capability Index

Find the API for an application task, follow its usage guide, and compare a
worked example. Each entry identifies the facade that owns the operation and
the inputs, results or retained values needed to use it.

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

<a id="before-you-conclude-a-capability-is-missing"></a>

## Find a capability

Use these entry points to find an API and its worked use.

1. **Open the facade's API documentation.** Generate local rustdoc for
   declarations and runtime calls:

   ```bash
   cargo doc --manifest-path workspaces/worth-query/Cargo.toml --no-deps -p worth-query-decl -p worth-query-host
   ```

   Browse the [Decl facade](../workspaces/worth-query/crates/worth-query-decl/src/facade.rs)
   and [Host facade](../workspaces/worth-query/crates/worth-query-host/src/facade.rs)
   to find the module that exports a name.
2. **Look up the task** in the [task index](#task-index) below.
3. **Search for a worked use** in the runnable examples and the reference
   application, which both use only the two facades:

   ```bash
   grep -rn "<name or concept>" workspaces/worth-query/crates/worth-query-certification/examples
   grep -rn "<name or concept>" workspaces/worth-query-bank-world
   ```

4. **Check the [known gaps](#known-gaps)** for the task's support posture.
5. **[Report a gap](#reporting-a-gap)** with the task, the facade names you
   checked and the specific behavior you need.

---

## Task index

For a new installed feature, use the
[complete feature definition](build-an-application.md#complete-feature-definitions)
first: `host::worth_query_application_contribution! { pub feature ... }` projects
schema members, action meaning, typed runtime configuration, required roles and
delivery roots. Supply actual handlers and domain configuration through the
generated runtime constructor. Use lower-level builders for specialized composition.
For tracked numerical work, use the
[managed/partitioned computation lifecycle](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation):
declare semantic membership, gather tracked inputs, compute with the original
checkpoint and complete on the owning operation. Canonical identities and
association preserve results across admitted placements; tracked changes refresh
affected partitions while unchanged partitions retain their results.
For application execution, compose
[one process execution service](build-an-application.md#one-process-execution-service):
retain an `Arc<host::runtime::ExecutionAuthority>` above applications and sessions,
pass clones through installation's `with_execution_authority`, and select
`ExecutionRequestPolicy::for_workers` with Serial as the ordinary default.
Choose Automatic placement for independent compute partitions when workload
measurements support it. Propagate typed cancellation, deadline, allocation and
domain failures from the original request.
For submission and recovery, retain the
[original owning action](build-an-application.md#53-execute-through-the-program).

Application installation and reopen require one
`host::application_installation::ApplicationBuildIdentity`, embedded by the
application build pipeline. Query binds it to every producer, including
`INPUT_REUSE=None`. Repeated launches of one artifact retain its embedded identity;
a new build causes derived output to regenerate while preserving authored source
and references.
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
returned `(original_capture, provider_denial)` on refusal. Retry that original
capture with the provider. The one-shot `admit` discards capture on refusal.
Pass the admitted capture into the authorized action. See
[external input on an action](build-an-application.md#32-attach-actions-to-features).

Program validation admits finite canonicalization work through
`ApplicationProgramDefinition::CANONICAL_REVISION_BUDGET` and the program facade's
`CanonicalDigestWorkBudget`. The default covers 4096 entries and 1 MiB; override
the declaration's finite ceiling for a larger program. Handle a validation
refusal before installation. See
[program validation](build-an-application.md#35-validate).

For mutations, prepare the typed decision through `DecisionReader`, then consume
its values and native targets through `CandidateWriter`. Propagate preparation
or publication denials; a successful commit publishes the validated candidate
atomically. See [prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects).

Platform owners can borrow the existing targets of a validated native mutation
with `ValidatedMutationFootprint::records()`. Iterate its borrowed records to
inspect each target, preserving multiplicity and the footprint's field and scope
checks.

Platform runtime owners use native partitions with `run_retained_run` or
`run_reduce_retained_native` to retain values accepted by canonical settlement.
Retain the returned owner for as long as its immutable values are needed. Reuse
checks a fresh request's controls; the caller qualifies input equality and
currentness. `accounted_cost()` is optional: `None` means cost was not measured.
For explicit payload and cost contracts, use lower-level methods
including `run_retained`, `run_retained_inputs` and
`run_reduce_retained_holding`.
Their genuine receipts carry original checkpoint prefixes; a value that lacks
the required cost qualification must execute again. See
[retained native results](../crates/worth-execution/README.md#retained-native-results).

Platform-owned ordered retained metadata uses `ExecutionOrderedMap<K, V>`:
shared-root cloning and canonical iteration preserve original custody.
Retain the returned root while reading or applying later edits. `accounted_bytes()`
returns the available byte measurement; handle `None` as unmeasured.
`rank_of`/`entry_at` use actual structural counts rather than prefix scans.
Deliberately accounted reduction shape-cost inspection accepts removals/insertions through
`checked_full_build_cost_after_edits`, reusing native edit traversal without
combining values.
See [funded ordered storage](../crates/worth-execution/README.md#funded-ordered-storage).

For a platform owner's keyless map, `run_retained_run` issues an opaque
`ExecutionRetainedRun` after real native completion. `prior.advance_native`
accepts changed values and canonical removed identities only; native settlement
keeps qualified unchanged ranges. Qualify unchanged inputs before calling advance;
propagate stopped-prefix or missing-qualification refusals. Accounted reuse follows
the original receipt at its work boundary. See
[persistent native runs](../crates/worth-execution/README.md#persistent-native-runs).

For nested persisted values, use the existing value-binding macro's
`scalar: String, canonical,` arm with Query-free
`WorthValue` derivation. The installed value-binding catalog retains the derived
shape and any explicit domain representation policy. Fixed arrays, checked
indices, `Box`, exact
pairs and tagged named variants use the checked seed path. Complete closure
admission checks every child, including unselected variants. Keep these pure
descriptions in the schema owner. Projection reads use the installed codec and
original request controls. Propagate decoder refusals: an ignored failure vetoes
completion.
Opaque owners supply admitted seeds.
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

For managed and partitioned computation, declare domain values and use the
installed owner's `decision_plan` to prepare, compute and complete them under
the original request. See
[managed computation](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).

`WorthQueryInvariantEntityIdentity` has its own native resident contract for the
shared entity-key allocation. Clones preserve that backing and its original
reader authority; resident description grants no read, currentness or mutation
authority and introduces no process-wide deduplication promise.

Platform storage owners use `ExecutionAllocationPolicy::reserve_memory(bytes)`
before copying when their contract requires admission. Accounted leases return
a reservation to retain with that payload. A worker-only lease checks request
status and returns `None`; the allocation owner still handles layout and allocator
failures. Retain an accounted reservation until the payload is released. See
[Execution payload custody](../crates/worth-execution/README.md)
for the ownership and refusal contract; this grants no graph or publication authority.

Choose a task and follow its guide for the exact calls and result handling.

| I want to… | Use (facade path / call) | Guide |
|---|---|---|
| Declare a concrete operation's access once | `decl::worth_query_operation!(... input Input; access { ... })` generates Rust capabilities and the schema projection; `schema.declared_operation::<Operation>(definition)` installs it with explicit effect/aftermath policy. Declare the complete read/effect/precondition set. | [Operation access](build-an-application.md#21-declare-operation-access-once) |
| Fence a locator to one live Server assembly | Use `WorthServer::instance_identity()`, retain session discovery's `server_instance`, and install `WorthServerInstanceSelectionPolicy::RequiredForProductCalls` where product calls must carry it. Guard and generated catalog share the policy; session IDs share the same instance identity. | [Server instance selection](../crates/worth-server/docs/connect-another-backend.md#fence-a-live-server-instance) |
| Preserve product preflight/currentness refusals over HTTP | Server retains `WorthServerTransportDenial::product_operation_denial()` and projects `ProductOperationDenied` with `worth-server-product-surface-denial.v1` owner codes, basis facts and execution/recovery posture. | [Product-surface refusals](../crates/worth-server/docs/connect-another-backend.md#product-surface-refusals) |
| Attach a native program to Server readiness without extracting its runtime | Transport entry uses `worth-server`'s `WorthServerProductOperationDeclaration::with_program_application(program.clone())` and returns a finished adapter registration. Readiness reuses the authentic installed allocation and grants no operation authority; native requests still authorize execution. | [Connect another backend](../crates/worth-server/docs/connect-another-backend.md) |
| Re-enter an application's genuine operation custody on product retries | Runtime-backed product mutation declarations use `.with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody)`. Every command passes fresh Server admission/readiness and enters the adapter; Server neither caches responses nor issues retry receipts. The adapter must retain actual native custody and freshly authorize its advancement/delivery. Reads always enter fresh authorization, even with a retry key. Eligible default `ServerReceipt` mutations and durable-executor replay keep their existing contracts. | [Application-owned retries](../crates/worth-server/docs/connect-another-backend.md#application-owned-command-retries) |
| Complete an effectful business Back or global ending | Retain the original guarded owner; after real external completion, call `completed.accept_local_action(&fresh_request, workflow)`. The original binding's native admission—required abilities/policies or its exact installed capability, never Advance—admits exact route/custody settlement. Inspect returned progress and retain cleanup; exact replay never repeats business effects. Suppressed effects confirm their original joint settlement. | [Local action completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#complete-a-local-action-with-an-external-effect) |
| Attach a fresh record to the admitted creator | `creation.link_to_principal::<Binding, Relation>(relation, key)` binds the exact mutation principal and carries its observed target through publication. Root and peer creations only; fresh schema and typed installation validate the correspondence. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) |
| Atomically create a retained audit beside a disposable selection | `creation.with_peer(entity, key, authoring_closure)` reuses typed initializers; `creation.link_from_path(path, relation, key)` binds a retained existing anchor. Reusing one site does not suppress independent peers. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) |
| Assign or clear an input-selected existing entity reference | `body.replace_relation_from_input(relation, required_readonly_identity, member, key)` admits an original Option member, resolves through the native equality owner, and retains current/selected endpoints into one candidate. Nullable outgoing max-one relations only; same endpoint preserves its edge. | [Existing-reference inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/existing-reference-inputs.md) |
| Replace a disposable current snapshot and retire its predecessor | `creation.replace_and_retire_from_scope(relation, key)` reuses the tracked native predecessor selection for atomic Unlink/Delete/Create/Link. Requires explicit Delete and one-to-one relation cardinality. Ordinary `replace_from_scope` still preserves the old entity. | [Retiring selection replacement](../workspaces/worth-query/crates/worth-query/docs/authoring/retiring-selection-replacement.md) |
| Preserve an unchanged disposable selection's actual identity | `creation.reuse_current_if_unchanged()` compares every declared field, including optional absence, through the native canonical-equivalence owner after input admission. Independent body effects still run. | [Complete-content selection reuse](../workspaces/worth-query/crates/worth-query/docs/authoring/complete-content-selection-reuse.md) |
| Create a record and its required child topology atomically | `creation.with_child(relation, key, authoring_closure)` declares flat prior-parent sites; `child.link_from_parent(relation, key)` adds an association from that same fresh parent. Existing native owners allocate all sites in the original admitted context and link retained created handles. | [Atomic child creation](../workspaces/worth-query/crates/worth-query/docs/authoring/atomic-child-creation.md) |
| Atomically write a computed value to a selected parent | `body.assign_value_at_path(path, required_field, &value)` retains the exact guard-free native path target and its destination-field dependency. Existing checked evaluation feeds the same candidate as scope/audit writes and signed creation. Captured AuditContext remains a preimage; certified Subject loci cannot be rewritten, including through aliases. | [Computed graph-target writes](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-path-writes.md) |
| Write an exact Decimal from captured input | Use input assignments/initializers with a readable `ScalarAspectType::Decimal` field codec and its explicit native input profile; a transparently serialized string is borrowed exactly and validated before effects. Required and optional members are supported. | [Exact Decimal inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#exact-decimal-inputs) |
| Require captured input to match a parent's field | `body.expect_input_from_path(ApplicationQueryRootPath, field, member)` uses native forward/reverse/identity selection and tracked field comparison without changing the row's scope. Optional graph absence cannot match. | [Path-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/path-input-guards.md) |
| Retire dependent current snapshots when a source input changes | `body.retire_owned_selections_on_input_change(field, member, owners, selection, maximum_owners)` coupled to the exact body input write. Native observation retains every selection and publication unlinks/deletes atomically, preserving holders. | [Owned-selection retirement](../workspaces/worth-query/crates/worth-query/docs/authoring/owned-selection-retirement.md) |
| Require a stored field to equal captured input | `body.expect_input(field, "member")`, ordinary native body installation and owning execution. Required or optional field must be present; the original captured flat scalar is admitted through the field's installed codec. Query retains the comparison fact through publication. | [Captured-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#compare-a-field-with-captured-input) |
| Keep a business value unique | Schema `unique_field(entity, field)` with an equality-queryable field, then the normal authored body/owning action. Query retains intrinsic indexed dependencies for actual unique writes before signing/sealing; native lowering and publication enforce collisions and concurrent claims. | [Unique business values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#unique-business-values) |
| Conditionally emit the exact operation input | `body.emit_external_input(effect, Some(native_condition))`, the operation's `.external_effect(effect, correlation_family)`, and normal `setup.operation_body::<Binding>()`. Met emits; Unmet/NotApplicable suppresses only that effect. Guard facts remain publication obligations. | [Authored conditional effects](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md) |
| Emit graph data absent from the command or public DTO | `body.emit_external_query_context::<Binding, _, _, _, _, _>(effect, exact_query_definition, condition)`, a payload binding with Value `ApplicationQueryExpressionSnapshot`, `execute_with_expression_context()` and the actual `expect_source`. The query is parameter-free and ExactlyOne. Query funds/prepares once and publishes the ordinary outbox with all native source facts. | [Native query-backed effect payloads](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md#real-example) |
| Share one computed revision across business, audit and payload | `ApplicationOperationBodyValue::admit`, `body.assign_value`, `creation.initialize_value`, and `emit_external_evaluation` with `ApplicationOperationBodyEvaluationSnapshot`. Native Foundation evaluation prepares once and validates every installed destination codec. Unitless UInt64 results; at least one field sink. Computed scope bookkeeping may accompany signing, with old revision captured as AuditContext; computed signed-artifact initializers remain refused. | [Shared Computed Body Values](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-body-values.md) |
| Retain correlated native query values for expressions | `request.query(intent).execute_with_expression_context()`; inspect `observed_source.expression_context()`. Exactly-one one-shot result; values stay attached to recursive dependencies before any domain projector. The original row or `result_set_observation()` supplies them through `expect_source` or `expect_result_set`; these source contracts remain distinct. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md) |
| Inspect nested snapshot data in a pure payload codec | `ApplicationQueryExpressionSnapshot::field_path(&["owner", "name"])` follows actual nominal record types and schema field order, unwraps intermediate optional-one records, and preserves a final option. `ExpressionValue::record_fields()` distinguishes records from lists/entries. Lists are never implicitly indexed. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#inspection-and-debugging) |
| Require correlated native query information | `operation.requirement_operands().query_context::<Binding, _, _, _, _>(name, query_definition)` consumes the exact parameter-free, exactly-one query declaration. Supply its sealed source with `expect_source` to guidance and execution; native facts remain publication obligations. | [Authored context requirements](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#real-example) |
| Require values across a related collection | `operation.requirement_operands().outgoing_field_values(name, relation, child_field)` binds a tracked `List<Value>` or `List<Option<Value>>`; use the existing Foundation expression for length and all/any. Same requirement drives guidance and mutation admission, with membership/liveness/field publication dependencies | [Related-item values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#require-values-on-every-related-item) |
| Return the scope, one selected outgoing subject or a created row from an authored body | `decl::application_schema`: `body.preserve_scope_output::<Binding, Role>()`, `body.preserve_related_output::<Binding, Role>(relation)` (Preserve/ExactlyOne), `creation.create_output::<Binding, Role>()`; read the real receipt's `outputs_of::<Contract>()?.entity::<Role>()?`. Related selection is tracked once and retained through publication; replay preserves original identity. | [Scope-bound operation bodies](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#stable-entry-points) |
| Supply original typed inputs for native workflow needs | `WorthQueryWorkflowOperationInputs::new(program_runtime)`, then `.with::<InstalledAction, _>(description)` for a binding-wide entry or `.with_at::<InstalledAction, _>(&qualified_input_node, description)` for an exact node, and `select(workflow, &required)`. Distinct qualified nodes may retain the same binding; ambiguous overlap is refused. Native needs select order and existing owners retain all custody. Source-completion actions only. Accept an opaque completion with `prepared_advance.accept_selected_operation(completed_selection)`; the occupied native owner retains fresh admission, exact receipt checks and cleanup on every refusal. This roster does not supply heterogeneous proposal target inputs: ordinary `OperationInput` edges still require the same portable type and exact source-proposal input identity. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) |
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build §3](build-an-application.md#3-declare-features-and-the-program) |
| Select complete feature schema, runtime, contracts, spec and output projections once | Host `worth_query_application_contribution! { pub feature Definition<S: Binding> for Feature => Runtime { ... } }`; typed `action name(Binding, Delivery, HandlerType)` and `required_pair name(Roles, InitialHandler, PreserveHandler)` members drive every projection. `ApplicationFeatureDefinition` composes specs/schema/actual roots; `WorthQueryFeatureDefinition` composes explicit configuration and existing installation. The existing required pair and role runtimes preserve native lifecycle ownership. Repeated identical delivery roots deduplicate before flattening; distinct-root connection conflicts still refuse. | [Complete feature definitions and required roles](build-an-application.md#required-initial-and-preserve-output-roles), runnable `topology_entry/examples/required_feature.rs` |
| Declare distinct captured workflow target inputs | `graph.captured_operation_input(&proposal, &target)` and the component equivalent declare exact control/guarded correspondence. `prepare_workflow_proposal_with_inputs(workflow, instance, &inputs)` publishes complete original target contracts through native proposal/replay/first-effect owners. Archive profile 8 carries the new edge; unchanged definitions keep profile 7. Recorded-actor admission and recovery retain the original proposal correspondence and require fresh grants. In-memory owning execution, not a durable whole-command runner. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build §4](build-an-application.md#4-install-the-application-graph) |
| Identify a rejected operation declaration dependency | `decl::application_schema::ApplicationSchemaDeclarationDenial` preserves `MissingOperationProgramDependency`, `MissingOperationDecisionReadDependency` and `MissingOperationMutationPreconditionDependency` with owned `operation` and `ApplicationOperationDependencyLocus`. Target loci carry the original typed field/relation/precondition; whole-body failures remain explicitly `Body`. | [Declaration failures](build-an-application.md#45-declaration-failures) |
| Configure invalidation retention | `host::runtime::WorthQueryInvalidationResources::new()` takes no policy. The subscription retains every delivered position, so no reader falls outside a window or reconstructs to catch up. | [Build: invalidation retention](build-an-application.md#invalidation-retention) |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) |
| Compose one complete action | `ApplicationFeatureSpec::root::<S, F>()` or `at::<S, I, F>()`, then `.action::<Binding, Delivery>()`, optional `.locality_and_change::<Scope, Shape>()`, `.correspondence::<C>()`, `.evaluated_requirement::<Rule>()`, `.external_input::<Provider>()`, and `.finish_action()` before feature completion. Each clause is once-only; delivery is explicit. `ApplicationActionReference<S, I, F, B, D>` selects complete actual installed meaning without manufacturing it. | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) |
| Store fixed or discovered work beyond construction | `WorthQueryFixedOperation<S, Intent, Program, Root, Action>` and `WorthQueryDiscoveredOperation<S, Intent, Program, Root, Action>` retain the authentic runtime allocation and complete installed action. Keep an independent genuine runtime handle for fresh requests on `owner.branch()`. All refusals return actual custody; runtime close refuses `RuntimeKeepaliveRetained` while operation keepalive remains. | [Owning actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#retained-runtime-and-configuration-custody) |
| Observe a required output without driving its producer | Candidate `DecisionReader::output_availability(&native_producer, typed_demand)` and checked `WorthQueryComputationReader` equivalent retain the fresh source Query and native completion dependency in the same attempt; Current or positively qualified Pending, with typed refusal otherwise. Neutral public native source/completion and original-image retry are qualified; private CAD and Fresh-on-restore remain open | [Source-aligned availability](../workspaces/worth-query/crates/worth-query/docs/capabilities/source-aligned-output-availability.md), compiling `topology_entry/.../output_availability/collection.rs` |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build §5.3](build-an-application.md#53-execute-through-the-program) |
| Configure application compute placement | `host::runtime::ExecutionRequestPolicy::for_workers(posture, determinism, workers)` selects placement, determinism and width; start with Serial. Pass the policy through application resources. Platform stages use `ExecutionRequest::run_native`, `run_native_with_checkpoint` and `KernelContext::poll`; propagate typed request and allocation failures. | [One process execution service](build-an-application.md#one-process-execution-service) |
| Share workers across applications and checkpoint reopen | Construct `ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(workers))` once at process startup, retain its `Arc` above sessions, and pass clones through installation's `with_execution_authority`. A final handle drop does not authorize constructing another physical authority in that process. | [Shared service and runnable example](build-an-application.md#one-process-execution-service) |
| Configure explicit request-memory accounting | `ExecutionBudget::new(workers, memory, work)` and `SerialMemoryBudget::new(memory)` accept optional memory ceilings. `Some(bytes)` selects a finite ceiling. Retain each reservation with its payload and handle finite-capacity or counter-overflow denials. | [Explicit request-memory policy](build-an-application.md#optional-request-memory-policy) |
| Retain a result whose heap size is not known before evaluation | Original `ExecutionRequest::run_measured_result(ceiling, known_bytes, &mut reservation, kernel)` reuses Scope admission and lends `ExecutionResultMemory` on that same reservation for before-growth claims. It measures opaque callback results/errors before framework retention and leaves their actual heap charged until release. This is post-materialization admission unless the source cooperates before allocation; declared Scan/Map maxima remain separate real contracts. | [Request-memory policy](build-an-application.md#optional-request-memory-policy) |
| Check invariant interruption and configure work policy | Call `Context::check_live()` or `ScopePlanner::check_live()` and propagate `CustomInvariantInterruption::{Cancelled, DeadlineElapsed}`. Configure optional domain work through `RelationalInvariantWorkBudget::new(limit)`; supply an explicit accounted execution budget when work measurement is needed. | [Optional invariant work and original interruption](build-an-application.md#optional-invariant-work-and-original-interruption) |
| Return a typed refusal from an installed invariant | Declare the rule's `Refusal` and canonical `RefusalBinding` with a finite wire maximum. Return `Violation(refusal)`; the original commit denial decodes through `decode_invariant_refusal::<Binding>()` and retains exact bytes. Installed binding order determines the first violation; encoding failure and cancellation remain distinct outcomes. | [Typed invariant refusals](build-an-application.md#typed-refusal-from-installed-invariants) |
| Enforce application relation laws | Declare `.acyclic()`, `.symmetric()`, `.unique(scope)`, `.same_partition_endpoints()`, `.reaches_at_least(n, kinds)` or `.orbit()` on the registered `ApplicationRelationIntegrity`; native commit validation judges proposed edges, physical multiplicity and distinct reachable targets, including creations and removals. The stored orbit law requires homogeneous endpoints and exactly one predecessor and successor per live member. It also enables private committed aggregate identity and lineage under one Native owner; ordinary orbit/lineage inspections, tracked orbit decisions and complete declared orbit queries are available; scoped and tracked lineage reads remain unfinished. Generic Native branch reconciliation retains its separate typed limitation. A native failure that blocks the current execution point prevents custom invariant preparation; administrative admission still comes first. Inspect original typed law fields through the commit denial's `relation_law()` view. | [Relation laws](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Return a typed domain denial from an output producer | Declare the provider's `type DenialValue` as a canonical binding of the operation's `Denial` with a finite wire maximum, as `WorthQueryProjectedProducerDenial<P>` projecting each denial onto such a binding or an explicit absence, or as `WorthQueryUndeclaredProducerDenial`. Installation refuses an unbounded binding. A `ProducerDomainDenied` demand denial decodes through `decode_producer_denial::<Binding>()` and keeps exact bytes in diagnostic clones. Mismatched, undeclared and unencodable values are refused by type. `domain_reason()` stays diagnostic text. | [Typed producer domain denial](build-an-application.md#typed-producer-domain-denial) |
| Require an invariant of every producer publication | Declare `REQUIRED_INVARIANTS` on the producer binding. Each publication is admitted only when the rule passes or Relational proves the candidate's footprint touches no kind in the rule's declared closure; the receipt's `non_applicability()` is then `CandidateOutsideClosure`. A scoped-out rule (`NoApplicableTouch`, as for a raw delete), refused preparation, or a candidate with no intents (no receipt) refuses the publication, and the demand denial's `producer_requirement_denial()` names the receipt's verdict and basis. | [How it works §9.8](how-it-works.md#98-publication-live-reads-and-output-demand) |
| Observe reads by an installed invariant | `host::application_invariants::WorthQueryInvariantProjectionWork` describes reads. `ScopePlanner::reads()` observes preparation; `Context::reads()` includes preparation plus this evaluation, across every proposed and committed view. Counts describe Query reads and grant no access. | [Invariant reads and observations](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md) |
| Follow an invariant entity between the two views of one proposal | `target.reobserve_entity(&entity_binding, &issued_selector)` freshly observes the same physical entity under the target view's declared kind, original controls and prepared scope. The selector must belong to the same actual proposal and installation; it grants no read authority. A removed target returns `None`, while stale/foreign selectors or undeclared access refuse. The returned handle has the target posture; ordinary cross-view reads remain refused. | [Invariant views](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md) |
| Read complete reachability inside an installed invariant | `view.walk(&binding, RelationDirection::Forward, &root)`, `closure(.., &roots)` and `reaches(.., &from, &to)` consume one installed endorelation. Native-backed results retain the original payer and every visited entity/physical edge enters prepared admission. Cancellation and resource refusal return typed errors. | [Invariant walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md) |
| Read a persistent orbit inside an installed invariant | Use `view.orbit(&relation_binding, &member)` on the proposed or committed view. `members()` is successor order from the current anchor, `identity()` is the persistent typed name, and `len()` is the orbit size. `orbit_of(..)` preserves the same complete entity/edge footprint. Both calls obey installed binding, view affinity, declared access and prepared scope; a cyclic relation without the declared orbit law refuses. | [Orbit law and invariant reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Inspect a persistent orbit at a selected snapshot | Projection and raw operation readers expose `reader.orbit(Relation::reference(), &member)` and `orbit_of(..)`. The complete owned result exposes successor-ordered `members()`, physical `relations()`, `member(index)`, persistent `identity()` and `len()`. Operation readers require the relation's `OperationReads` permission; all members enter the original realized scope. These inspection reads record no publication decision dependencies. A retained identity grants no read, write or history-disclosure authority. | [Ordinary orbit inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Track an orbit as a mutation or computation decision | `DecisionReader::orbit` / `orbit_of` and computation readers retain original Native aggregate identity/membership and complete successor-walk facts. Lower-level admitted operation readers expose `decision_orbit` / `decision_orbit_of`. Both original bodies are sealed under the actual admission, contracts and realized scope. Fresh comparison detects identity or topology changes; retained aggregate inputs currently require full verification and a fresh computation. Live bodies have no checkpoint reconstruction door. | [Tracked orbit reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Query one complete persistent orbit | Add a typed `ApplicationQueryOrbitRef` with `.orbit(slot)` to a `Many` query whose root and scope are the same kind. `result.orbit(slot)` exposes identity, full length and borrowed member rows in Native anchor-forward order, including after disclosed/output-demand conversion. Governed queries require one exact `orbit_by` metadata rule in addition to member-field rules; omission refuses before Native group selection. Filters, root-path unions, explicit ordering, continuation and live lanes are refused. | [Declared orbit queries](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Inspect aggregate lineage under privileged snapshot authority | `reader.lineage(Relation::reference(), &name)` on an installation-owned projection reader acquires real stored incident history and merge-only latest resolution at its exact snapshot; the operation inspection wrapper also requires `OperationReads`. The owned result retains actual Native values and typed related names. A retained name selects the read but grants no history disclosure, member scope or decision/publication authority. An absent name refuses; a retired terminal with no successor succeeds with `latest_identity() == None`. Scoped/governed lineage, tracked lineage decisions and restoring portable names into a fresh installation remain unfinished. | [Lineage inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Read complete reachability in a snapshot projection | In an invariant projection callback, call `reader.walk(Relation::reference(), direction, &root)`, `closure(.., &roots)` or `reaches(.., &from, &to)` with one endorelation and locally resolved identities. One pinned Native traversal retains result funding; the snapshot separately retains its reached scope. `reader.work()` reports actual reads so far. | [Invariant walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#snapshot-projections) |
| Inspect complete reachability in a raw operation projection | The raw operation reader accepts `walk`, `closure` and `reaches` for a relation implementing `OperationReads<Operation>`, using the selected snapshot and installed work budget. These inspection calls do not record publication dependencies. | [Raw operation projections](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#raw-operation-projections) |
| Record complete reachability as an operation decision dependency | Handler and computation readers accept `walk`, `closure` and `reaches`; the raw operation reader uses their `decision_*` forms. Original issued Native values pass through sealing and complete current-snapshot comparison. Structural computation inputs currently force a fresh run; live carried-value and checkpoint reuse remain unsupported. | [Recorded decision walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#recorded-decision-walks) |
| Select complete reachability in a declared query | Compose `ApplicationQueryRootPath::forward_closure` or `reverse_closure` over an endorelation with ordinary hops and guards. Original Native complete bodies support row/set currentness; `receipt().root_path_steps()` reports actual per-step reads. | [Declared query closure](../workspaces/worth-query/crates/worth-query/docs/foundations/declared-query-closure.md) |
| Observe reads by a declared query | `result.receipt().reads()` returns the same `WorthQueryInvariantProjectionWork` on one-shot results, continuation pages and live refreshes. It reports actual field, adjacency, endpoint and index reads separately from projection capacity and authorization work. Reused computation facts add no historical reads; fallback preserves reads already performed. | [Read observations](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md#inspection-and-debugging) |
| Declare a foreign domain value through its entry facade | `decl::application_schema::ValueSeedDelegate` supplies the root grammar via `ApplicationCanonicalValueContract::delegated`, `encode_delegated_canonical_value`, and the bounded or admitted delegated decoder. The existing canonical codec borrows the foreign value without requiring a platform trait on its pure owner. | [Typed invariant refusals](build-an-application.md#typed-refusal-from-installed-invariants) |
| Configure World resources | Pass `RuntimeWorldBudgets`, a clock, invalidation resources and an execution policy to `host::runtime::WorthQueryProductWorldResources::new`. Use `RuntimeWorldBudgets::without_policy_limits()` for standard installation, or install deliberate finite policies. | [World resources](build-an-application.md#runtime-world-policy-ceilings) |
| Configure Query reads and output settlement | Use `WorthQueryApplicationQueryResourceProfile::new(concurrency)` for graph-work concurrency and `WorthQueryOutputDemandResourceProfile::standard()` for settlement. Convenience settlement continues on `Progressed`, parks on `Pending`, and returns `Settled` when complete. Retain returned owners for later advancement or close. | [Query resource policies](build-an-application.md#ordinary-query-resource-policies) |
| Run a declared fixed action whose source feeds required outputs | Declare `.action::<Binding, Delivery>() ... .finish_action()` and its exact `ApplicationActionReference`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Construction reuses the native governed root-artifact/source guard before effects; invalid locality/change/dependency metadata returns the original description with `WorthQueryOperationConstructionDenial::OutputSource`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports optionally bounded `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis; eligible Required fixed-leaf terminal `close` invokes native unavailable completion, preserving actual negative evidence across cleanup refusal in `output_unavailable()` on terminal or closed custody. Source saved with no current output is not positive settlement | [Build §5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/…/application_program/required_unpublished.rs` |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` |
| Own a declaration-authorized no-output action | Declare `.action::<Binding, Delivery>() ... .finish_action()` with `ApplicationSourceCompletion`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode uses `advance(&fresh_request, maximum_history_work: Option<NonZeroUsize>)`; `None` adds no history work ceiling, while a finite bound remains a caller choice. It delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) |
| Retry a mutation safely | Retain the move-only operation returned by `Progress` or `Refused` and supply a fresh authorized request to its next `advance`. Retry delivery through the returned settled owner; terminal custody supports evidence inspection and close. After process loss, replay requires the original stable intent, key and supported source posture under fresh authority. Row/result-set source expectations belong to their original runtime. Reconcile an ambiguous external effect through its native, domain or Store recovery owner before choosing a new operation. | [Owning entry](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) |
| Correct a recorded source operation | Declare `ApplicationCorrectionBinding` with its recorded-inverse or compensation mechanism in the contribution's `corrections`; call `request.corrections().select(receipt, limits)`, then `reverse(&source).idempotency(key).limits(limits).prepare()` and publish the candidate. Handle equivalent replay, stale targets and native custody through the returned outcome. | [Recorded source correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md), [Compiled example](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction.rs) |
| Reapply retained source input from an explicit history choice | Enable `visit_reapplication` and `reapplication_key` on the correction binding. Read `corrections().alternatives(&source, limits, maximum_entries)` through its continuation, select the chosen reversal, then call `reapply(&alternative).idempotency(key).limits(limits)` with the source action's ordinary preparation and delivery. | [Explicit historical alternatives](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#reapply-an-explicit-historical-alternative), [Tree journey](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/tree_reapplication.rs) |
| Keep a correction chooser across authenticated requests | Retain the native page returned by `into_page()` in the server's session lifecycle. Complete traversal, then use a fresh request's `corrections().select_alternative_coordinates(&page, &chosen_source, limits)` before preparing reapplication. Selection validates the exact target and current disclosure; refresh a stale chooser and retain the same command key for an equivalent retry. | [Chooser request lifecycle](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#carry-a-chooser-across-requests) |
| Correct an authored conditional definition | Capture the authored parameters in the recorded inverse. The ordinary handler prepares the installed definition with `DecisionReader::prepare_conditional_definition`, carries its owner permit in the decision, and joins it to authored field writes with `CandidateWriter::advance_conditional_definition`. Publication exposes the complete Relational and Signal successor together. | [Conditional-definition correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#correct-an-authored-conditional-definition), [Ordinary handler](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/composite_definition_handler.rs) |
| Select a correction source after normal operation close | Keep the completed receipt's `correction_source()` locator, close its operation, then call `request.corrections().select_recorded(&locator, limits)` under the current caller. For transport, serialize `locator.coordinates()` and use `select_coordinates(&coordinates, limits)`. Native evidence lookup, World ancestry and disclosure validate the selection. | [Closed source selection](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#select-a-source-after-closing-its-operation) |
| Keep a corrective action's installed delivery owner | Use `prepare_operation::<Correction, Program, Action>(&application)`. Publish a prepared candidate or continue the returned original action, then advance, read and close its native owner under fresh requests. | [Corrective action delivery](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#keep-the-corrective-actions-delivery-owner) |
| Compensate a completed external workflow operation | Declare the actual source effect, compensation and exact `completed_external_reconciliation()` procedure. Preparation checks the original dispatch's authenticated inbound completion and originating workflow settlement; the new compensation preserves the original remote effect and workflow disposition. | [Completed external compensation](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#compensate-a-completed-external-workflow-operation) |
| Derive compensation from an original created output | Bind the source's actual exactly-one Create role, declare its fields with `ApplicationCorrectionOutput::<Source, Role>::field(...)` in `source_output_fields()`, and decode the borrowed original values from `ApplicationCorrectionSourceMaterial::Compensation(fields)`. | [Original created-output fields](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#read-original-fields-of-a-created-output), [Compiled binding](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/source_output_correction.rs) |
| Regenerate required outputs after a recorded correction | Use `prepare_performed::<Correction, Program, Root>(&application)` and retain its performed-source owner. Start its required outputs under a fresh request. An unpublished owner resumes through `recover_recorded_correction` and `promote_recorded_correction`; a retention failure resumes through `retry_recorded_correction_retention`. These owners retain the actual captured inverse and transformed key. | [Required outputs and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#required-outputs-and-recovery), [Compiled recovery](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/required_output_recovery.rs) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) |
| Create a subject and enroll it atomically with owning recovery | Declare `creation.enroll_in_workflow::<Spec, Binding, Role>()`, install `birth_operation`, and prepare native `RequiredWorkflowBirth`. Capture one original descriptor, then `for_workflow_birth::<Spec, Role>(&workflow, &required)?.retain_operation::<Program, Action>(program)` selects the declared SourceOnly owner. Frozen input/key and actual one-shot birth authority survive refusal, keyed/unpublished recovery and cleanup; `into_parts` recovers effect-free bound custody. | [Workflow-enrolled creations](../workspaces/worth-query/crates/worth-query/docs/authoring/workflow-enrolled-creations.md) |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build §6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) |
| Start a fresh episode after a completed workflow | Scope/Preserve body `.succeed_workflow::<Spec, Binding, Role>()`; install `.succession_operation::<Binding, Role>()`, then native `prepare_workflow_succession` / `for_workflow_succession` with the same logical input/key. Preparation accepts the runtime-owned published reference or original pure start descriptor; after checkpoint restore, the latter requires fresh command admission and tracked native lookup. | [Completed-chain succession](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#completed-chain-succession) |
| Retain the actual outputs of a performed workflow operation | Native guarded settlement records the binding and actual role/entity/posture/reference inventory atomically with the transition; Existing, Created and Retire identities survive audit retention. No additional supplier flag or host write. | [Retained operation correspondence](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-operation-correspondence) |
| Require succession to continue the selected round or revision | Body `require_predecessor_output::<Binding, SelectedRole, Producer, ProducedRole>()` compares its sealed Preserve/ExactlyOne output with the latest performed occurrence of the declared guarded producer; both roles must name the same entity. Exact replay precedes fresh mismatch refusal. | [Succession output affinity](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-succession-output-affinity.md) |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `…for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build §6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` |
| Certify shared and row-local facts without copying their source truth | A native signed creation uses `capture_from_path(ApplicationQueryRootPath, source, snapshot, Subject/AuditContext)` with v3 graph provenance. The existing lifecycle derives source-identity and selection-scoped supersession; `bound_subject_impacts` bounds the union of affected subjects. | [Graph-certified subjects](../workspaces/worth-query/crates/worth-query/docs/authoring/graph-certified-subjects.md) |
| Own capability delegation or revocation across fresh requests | `request.retain_capability_delegation_in_program(..)` / `retain_capability_revocation_in_program(..)` own original input, key, conditions and principal association. `advance(&fresh_request)` returns progress, same-owner refusal, native completion or terminal custody; `close_with_evidence` discharges cleanup before moving the receipt | [Owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands) |
| Own a guarded workflow effect across fresh requests | Effect-free bound construction refusal returns the original binding; consuming `into_parts()` extracts its original description and requirement beyond the runtime borrow. `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) |
| Inspect a workflow source independently of dispatch recovery | `WorthQueryWorkflowOperation::inspect_source(&fresh_request)` freshly admits the original binding and resolves native custody, then borrows its retained source evidence with `Committed`, `DispatchPending` or `ExternallySettled` posture. No recovery handle is minted and no delivery, settlement or cleanup occurs. `SourceEvidenceUnavailable` means completion has not been established, not that no effect happened. | [Source inspection](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#inspect-source-without-advancing-dispatch) |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event. For a compiled signed body on that approval control, use `prepare_workflow_approval_with_body(..)` to co-publish its attestation and approval under one event; exact settled replay needs no new event. Its evidence join accepts one or more distinct assessment inputs; empty applicable evidence never grants approval. | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Approval with attestation](../workspaces/worth-query/crates/worth-query/docs/authoring/signed-operation-creations.md#one-approval-action-with-a-business-attestation) |
| Prepare and publish an actual mutation | `OperationHandler::decide` / `prepare_decision` reads through `DecisionReader` and carries typed targets, owned keys and prepared values in its `Decision`; `build_candidate` consumes them through `CandidateWriter`. The framework opens the effect program directly and preserves checked sizes, cancellation, fallible growth, invariant validation and atomic commit. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects) |
| Delete an authored entity | Declare the entity read and delete grant; observe `reader.entity(&identity, Entity::reference())`, carry its mutation target into the candidate, then delete and bind its actual Retire output role. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects), [Compiled handler](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/source_output_handlers.rs) |
| Publish typed fields, links and emissions | Use the original `CandidateWriter` under the operation's declared create/write/relation/`OperationEmits` authority, then bind its actual fixed output roles or family members. Payload validation and output cardinality remain enforced. | [Build: mutation bindings](build-an-application.md#52-mutations-intent-binding-handler) |
| Act on time or a signal | A conditional operation: `.conditional_operation::<Op>()` on the feature spec, `host::worth_query_conditional_node!`, and a host-supplied predicate and named clock | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Conditional installed operations](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md) |
| Complete work from an external system | An installed inbound source: `runtime.inbound_occurrences().receive(source, bytes, scope).execute()` (`host::application_entry::WorthQueryApplicationInboundOccurrencesExt`); in a workflow, `await_inbound::<Effect>(..)` | [Aftermath: authenticated inbound completion](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#authenticated-inbound-completion), [Workflows: waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion) |
| Survive a restart | The app owns the artifact. Capture: inside `runtime.with_host_advancement(..)`, call `runtime.capture_application_checkpoint_with_sections(&phase, policy)` with a `host::application_installation::WorthQueryCheckpointCapturePolicy`. Restore: `in_memory_program_from_checkpoint(..)` or `in_memory_rostered_program_from_checkpoint(..)` | [Front door: capture and reopen](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#capture-and-reopen-cost-boundary), [Build §4.4](build-an-application.md#44-checkpoint-program-transitions) |
| Work on a branch or move it to a new program under its declared capability | `runtime.branches().fork(runtime.current_world()).components(..).create()`; `request(..).on_branch(branch)`; `.programs()` then `compare(&target)`, `adopt(&requirements)?.prepare(max)`, `programs.publish(prepared)` | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Build §7](build-an-application.md#7-adopt-a-new-program-on-a-branch) |
| Run a transaction on a fork of the current head and adopt it | `runtime.branches().fork(head).components(..).create()`, step with `request(..).on_branch(fork)`, then adopt with `runtime.on_branch(head).close()`; discard a refused fork by closing it instead. The fork keeps its parent's history, so corrections reach across the adoption and retained history grows linearly | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Fork-head adoption court](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/fork_head_adoption.rs) |
| Own Bank HTTP mutation and correction custody | Create/resume/read/close commands retain native operations across requests. Money actions supply retained activity reads; estate freeze, recorded restoration and capability revocation supply receipt completion. Retain `BankHttpMutationContinuation` before activating the listener; fresh authenticated commands advance and close each original owner | [Owning HTTP mutations](../workspaces/worth-query-bank-world/docs/http-owning-money.md), [Recorded estate restoration](../workspaces/worth-query-bank-world/docs/http-owning-money.md#correcting-an-estate-freeze), [Bank consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) |
| Own plain Bank account, access or payment mutation | Bank builders for personal/business account creation, access grant/revocation, initiation and either rejection form use `retain_operation()` and native SourceCompletion; supply fresh authority to each `advance` and `close`. Fresh results and recovered receipt evidence remain distinct | [Bank owning completion guidance](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-account-access-and-payment-completion) |
| Own Bank estate source or retained funding/reversal | Typed Bank mutation builders use exact declared SourceCompletion or existing RetainedCommit; `close_with_evidence` transfers original completion only after native cleanup | [Bank source and publication handoff](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-estate-sources-funding-reversal-and-deployment-revocation) |
| Retain Bank HTTP native cleanup and business evidence | The listener-independent mutation continuation stores actual source and elevation cleanup owners. Fresh `release_cleanup` retries on the original execution thread; the local `take_released_cleanup` handoff preserves actual uncommitted evidence and the installed runtime after physical release | [HTTP cleanup custody](../workspaces/worth-query-bank-world/docs/http-owning-money.md#estate-effects-and-native-cleanup-custody) |
| Issue or revoke Bank program deployment permission | Explicit `BankProgramDeploymentGrantSeed::for_program::<Source>(..)` in `BankWorldSeed`; recorded issuer calls `revoke_program_deployment_grant(..)` | [Build §7.1](build-an-application.md#71-the-call-chain) |
| Read current data | `request(..).query(intent).execute()`; exact reads with `retain_read()` and `at(&observation)`; live reads with `query(intent).subscribe(..)` | [Host README: ordinary typed query entry](../workspaces/worth-query/crates/worth-query-host/README.md#ordinary-typed-query-entry), [Host README: output demand and live reads](../workspaces/worth-query/crates/worth-query-host/README.md#output-demand-exact-observation-and-live-reads) |
| Read the original workflow instance at a retained observation | `request.at(&observation).read_original_workflow::<Spec, SubjectRead, AuditRead>(workflow.vocabulary(), &original_intent, subject_input, audit_input)` uses fresh installed read bindings before lookup. `Live`, `Historical` and authorized `Missing` retain exact basis data; audit is independently admitted and otherwise explicitly withheld. | [Governed original-intent reads](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#governed-original-intent-reads) |
| Read bounded workflow history and reacquire proposal references | `request.at(&data).read_workflow_history::<Spec, SubjectRead, AuditRead>(vocabulary, target, subject_input, audit_input, width, indexes, after)` returns exact-basis pages with independently withheld audit. `subscribe_workflow_history` / `poll_workflow_history` use native envelope dependency marks and fresh disclosure. | [Retained history](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-proposal-and-approval-history) |
| Read one workflow definition's metadata | Retained `discover_workflow_definition` with explicit typed `ApplicationWorkflowDefinitionReadTarget` and the separately installed `DefinitionMetadata` binding; fresh admission precedes selected-index lookup. | [Governed definition metadata](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#discover-and-retire-definitions) |
| Retry committed discovered-source retention | Ordinary `advance` retains and retries the native owner. For deliberate native control, retain the opaque `WorthQueryDiscoveredOutputRetentionFailure`; use a fresh original `advanced()` native request's `retry_discovered_output_retention(failure, application)`. Refusal returns the original owner; success promotes its actual carrier before starting native outputs | [Committed discovered retention](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#committed-discovered-retention) |
| Retain an application preview through readmission, discard or promotion refusal | `runtime.begin_preview(&phase, &observation, request)`; `preview.readmit(&phase, &runtime)`; `readmitted.promote()`; either phase's `discard()` returns its same owner on refusal. Fresh readmission validates the original source and runtime; promotion finalizes retained preview evidence, separately from source/output publication. Rejected preview remains explicitly discardable. Name its native result vocabulary through `worth_query_host::facade::application_installation::{BridgeSpeculationError, BridgeSpeculationErrorKind, BridgeSpeculativePromotionOutcome}` | [Preview custody](build-an-application.md#56-retain-preview-custody-across-refusal) |
| Own a complete no-source output tree | `request.demand(original).controls(controls).retain_program_outputs::<Program, Root>(&program)` retains the actual installed full tree. Owned construction refusal supports fresh `retry` without replacing an issued basis. `advance` returns pending/refused/settled/terminal custody. Settled `read_current` checks all original outputs and discovery before reading at the same native observation and phase. Every phase can `close`; an owned close refusal supports `retry_close`, including after source supersession. Native unpublished and uncertain obligations remain owned; close delegates lawful original native cleanup/recovery and otherwise refuses | [Complete no-source trees](build-an-application.md#55-own-a-complete-no-source-output-tree) |
| Close original complete output custody before fresh final delivery | Settled Fixed and complete no-source parents expose `close_for_delivery(&fresh_request)`. Actual native cleanup/interest close precedes sealed `WorthQueryClosedFixedDelivery` or `WorthQueryClosedProgramOutputDelivery`. Their existing-shape `deliver_current`/`read_current` methods validate the original full tree and installed meaning at a fresh native observation. To show several closed carriers current at one common basis, each no-projection Fixed and no-source carrier's `deliver_current_at(&request.at(&observation), policy)` runs that same installed-action or selected-root gate and full lineage at the caller's observation. Fixed close refusal returns the same parent; no-source `WorthQueryProgramOutputDeliveryCloseRefusal` supports close-only `retry_close` | [Delivery after native close](build-an-application.md#delivery-after-native-close), Host `closed_delivery_contract` construction/phase/shape family |
| Inspect original root output roles without a producer receipt | `host::application_entry::WorthQueryApplicationProgramOutputSettlement::root_settlement()` borrows the actual immutable root demand settlement; use its `outputs_of::<Contract>()` typed correspondence even when `root_receipt()` is `None`. Parent operation retains all tree/close custody; this root-only evidence does not establish fresh complete-tree currentness | [Original root output evidence](build-an-application.md#original-root-output-evidence) |
| Poll cancellation/deadline in a decision handler | `DecisionReader::checkpoint()` polls the real request. Preparation retains its first failure and refuses before candidate authoring even if caught. Successful polls preserve completed-decision input-reuse eligibility under the existing producer/context contract. | [Build §5.1](build-an-application.md#51-who-is-asking-principal-and-request-scope) |
| Bind a derived collection to checked computation | `ApplicationDerivedCollection` requires `Computation` and `Completeness`; declare the computation beside the collection and install with `setup.derived_collection::<Feature, Collection, _>(owner)`. The owner's output and stop types must match the completeness contract. Initial/Create and Preserve/Preserve producers may share that installed owner and operation with a common no-intent input: implement `ApplicationMutationInputScope<Schema, Binding>` for each selected mutation binding. Ordinary intent inputs delegate their existing single-binding scope. Scope selection grants no execution authority. Complete installed operation/workflow-stage target coverage is checked before initialization, separately from exact per-binding route inventory; `installed_conditional_targets_for_schema` exposes read-only full locations, and conditional declarations use `in_workflow_stage` for stage selection. Missing or misbound installation refuses before bootstrap effects; the existing keyed computation checks its completed value before publication. Native partition reuse selects retained facts from the exact settlement frontier and consumes validation at the same pinned reader snapshot; applications provide no dirty hints. Successful native reduction retains its Execution-owned run; a fresh descendant request advances only changed inputs and actual removals, sharing unchanged native results and fitting cost ranges. Fixed-membership tree updates need no full shape plan or deletion scan. Execution owns first-stop and tight-prefix charging; applications cannot supply a result handle or checkpoint tape. Context-sensitive or nested kernels execute rather than retaining unsupported custody. Fixed membership with unchanged keys also carries checked reader-call ranges and immutable item/call bounds; ordinary preflight/charges and exact reached-entity scope still apply. Qualified native postcommit rebase consumes paired original/postcondition custody, the complete snapshot-aligned frontier and actual candidate touches; fresh suffixes and changed leaf sizing remain charged. Missing association or continuity reconstructs in full. Qualified completed original physical reads share checked locator/effect routing through a genuinely fresh registered session. Both comparisons retain native source and registration checks; changed or unqualified source uses funded full original-fact comparison. Principal and source-role observations remain individual. Posting registration and membership processing remain separate obligations; key/membership changes retain full preparation. | [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) |
| Inspect original producer publication or admission stops | Borrow the native demand denial inside the original advance result. `publication_companion_stop()` identifies a commit resource stop; `producer_admission_stop()` identifies native preparation/readmission stops, including structural causes. Both retain the original `host::primary_graph::CompanionPreflightStop` and supplied quote. `output_verification_capacity_stop()` retains the original `VerificationCapacityStop` for output verification, including precommit consumed edges and preparation/readmission: the meter stop, scratch-size overflow or allocation refusal remains distinct. Demand capacity evidence may overlap producer admission; the commit getter specifically identifies precommit verification. All three `None` leaves the cause unknown. Keep the original operation/cleanup custody | [Producer publication diagnostics](build-an-application.md#producer-publication-diagnostics), [Host borrowing example](../workspaces/worth-query/crates/worth-query-host/src/facade.rs) |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build §3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) |
| Produce from authentic sources and declare artifact lifecycle | `WorthQueryApplicationProducerProvider` supplies semantic identity, operation input and idempotency from the original source, and declares its typed domain denial value. `ApplicationDerivedArtifact` declares output/locality, retention, succession, required posture, producer, dependencies, reuse and stopped outcome. The installed mutation path enforces source/currentness and publication contracts. | [Producer source and publication](build-an-application.md#producer-source-and-publication) |
| Reverse a business change | Declare a domain action with its required source, authorization and effects, then execute it through the ordinary owning request path. | [Owning actions](build-an-application.md#53-execute-through-the-program) |

For managed or partitioned handler computation, use the installed
`decision_plan` entrance described in [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).
It owns prepare/compute/complete and propagates the original typed failures.
Use `checkpoint.poll()` for request controls and
`HandlerResult::from_result`/`into_result` with `HandlerFailure<Domain>` to
propagate failures. Supply domain inputs, partition keys and computed values;
return the completed decision to the installed handler lifecycle.

Action metadata retains its existing capability scope: locality/change and
delivery belong to the exact occurrence; correspondence lookup is binding-type
scoped; rule/provider lookup is operation-type plus rule/provider-type scoped.
Use the installed evaluator for guidance and submission enforcement. Resolve
external values and admit their provider revisions before passing them to the
authorized source action.

Deliberately specialized native declarations use `.native_action::<Binding>()`
and the same optional clauses followed by `.finish_action()`. Declare delivery
for product mutations. For a concrete binding issued
only by an installed conditional, use `.conditional_native_action::<Binding>()`
and `.finish_action()` (DSL `conditional_native_action(Binding) {};`). It carries
binding membership with conditional ownership;
installation still checks the exact conditional operation set and real binding,
handler and producer contracts. Configure initial workflow
vocabulary before cloning runtime handles or retaining operations. A refused
`retain_workflow_spec` preserves runtime, installed spec and signing owner in
`WorthQueryWorkflowRuntimeBindingRefusal::into_parts()`; refused
`support_workflow_spec` returns the original spec with its denial.
`WorthQueryWorkflowApplicationRuntime::close_conditional_runtime(&mut self)`
delegates native close while retaining the configured wrapper and its installed
workflow/signing companions on refusal. Close the actual operation owners and
retry the same wrapper.

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
This process-local transport selects SourceCompletion bindings. Retain the live
Query owners in the mounted adapter through inspection, recovery and close.

---

## Use requests and retained operations

Declare schema and program meaning through `worth-query-decl`, then install and
execute it through `worth-query-host`. Bind workflow vocabulary before retaining
operations or cloning the configured runtime.

Use the branch's selected program and an authenticated application request to
prepare an action. Retain its original input, key and operation across
`Progress`, `Pending` and `Refused` results. Supply a fresh authorized request for
the next advance, inspect its typed outcome, and close the returned owner when
its cleanup obligations are complete.

Drive workflows by matching their returned needs. For
`AwaitingProposal(RequiredWorkflowProposal)`, inspect the node, operation and
input type, then use fresh `prepare_workflow_proposal` admission. Use admitted
principal identity for actor checks and reacquire live workflow references from
the installed runtime after reopen.

Declare time- and signal-driven work with `conditional_operation`. Submit clock
or predicate observations through the installed conditional runtime and advance
the eligible operation through Query.

For business reversal, declare a domain operation with the required source,
authorization and effects. For branch adoption, retain the target branch's
native adoption result and handle its typed preparation and publication outcomes.

For installation and checkpoint-transition work inside an already admitted original
execution request, use `host::application_contribution::with_bootstrap_advancement_from_request`.
Its bootstrap phase borrows the same cancellation/deadline and cumulative native work.
Call `host::application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement`
with that phase and a required capture policy naming the request's funding lease or
a descendant of it, which caps the capture; any other policy refuses at entry with
`ExecutionDenied(ExecutionResource { NestedLeaseMisuse })`. The capture-bearing path
requires leased backing and refuses unsupported serial backing with the existing
`ExecutionDenied(ResourceEnvelopeMismatch)` vocabulary, before effects. Bootstrap
custody cannot admit installed-runtime execution. A performed recovery capsule's
`repair_to_checkpoint_with_request(original)` returns that same capsule on opening,
settlement or capture refusal. Existing convenience constructors still own their
explicit bootstrap request and capture policy. See [original-request transitions](build-an-application.md#44-checkpoint-program-transitions).

## Known gaps

For implementation identity, use the whole-application
[`ApplicationBuildIdentity`](build-an-application.md#application-build-identity).
The build pipeline embeds one UUID; installation binds it to every producer.
Each new build regenerates derived outputs as required by the installed program.

Use the capability table to find a supported task. Describe additional domain
meaning in the application:

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

---

<a id="which-source-wins"></a>

## API reference and examples

- Read the facade's rustdoc for function signatures, type bounds and failures.
- Use [Build an Application](build-an-application.md) for the declaration,
  installation and execution sequence.
- Browse [runnable examples](../workspaces/worth-query/crates/worth-query-certification/examples)
  for complete installations and request handling.
- Use the [Glossary](glossary.md) for platform terms.

---

<a id="keeping-this-page-current"></a>

## Workflow and value reads

Use `read_workflow_instances` with an explicit Subject/Definition scope to read
retained workflow pages. Supply opaque-reference choices or native
carry-compatible intent to adoption, and handle its typed workflow denials.
Request instance-enumeration authorization separately from definition metadata.

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
`request.query_batch(intents).execute(&phase)` runs installed read slots under
fresh per-read authorization. Supply intents in the batch's declared order.
Live declarations name cause/binding/effect/scope/target meaning without a resource
contract. Request Live controls retain channel buffer capacity and materialized-record
cardinality; their managed queue occupancy is released on delivery or normal close.
For ordinary derived readiness, use the
[typed field builder](build-an-application.md#canonical-nested-values-and-derived-readiness)
against the completed declaration. The lower-level
`WorthQueryOutputReadinessContractBuilder::new` accepts a native projection for
families that own that projection directly. Supply an effect-free synchronous
predicate over the declared field dependencies.


### Graph-read admission in the default Query workspace

`worth-query-admission::integration::review_graph_read_access(requirements, inventory)`
checks read requirements against the available native index inventory. Handle
typed async, persistent-index, capability-registration or unsupported refusals.
The execution receipt retains the admitted plan and executor observations.
Use inline execution for supported synchronous reads; an asynchronous requirement
returns its typed refusal when native asynchronous support is unavailable.
