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

"Guide" links to the section that shows the exact calls. The "Never build"
column names the substitute that this capability makes wrong.

| I want to… | Use (facade path / call) | Guide | Never build |
|---|---|---|---|
| Keep a business value unique | Schema `unique_field(entity, field)` with an equality-queryable field, then the normal authored body/owning action. Query retains intrinsic indexed dependencies for actual unique writes before signing/sealing; native lowering and publication enforce collisions and concurrent claims. | [Unique business values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#unique-business-values) | Host duplicate checks, granting supplier read access merely to enforce uniqueness, removing the constraint to permit creation, or treating a request key as business identity |
| Conditionally emit the exact operation input | `body.emit_external_input(effect, Some(native_condition))`, the operation's `.external_effect(effect, correlation_family)`, and normal `setup.operation_body::<Binding>()`. Met emits; Unmet/NotApplicable suppresses only that effect. Guard facts remain publication obligations. | [Authored conditional effects](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md) | Host-side guard/payload builders, a second business commit, global optional-payload fallbacks, decoded metadata as codec authority, or claiming outbox creation proves external delivery |
| Emit graph data absent from the command or public DTO | `body.emit_external_query_context::<Binding, _, _, _, _, _>(effect, exact_query_definition, condition)`, a payload binding with Value `ApplicationQueryExpressionSnapshot`, `execute_with_expression_context()` and the actual `expect_source`. The query is parameter-free and ExactlyOne. Query funds/prepares once and publishes the ordinary outbox with all native source facts. | [Native query-backed effect payloads](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md#real-example) | Copied DTOs or publicly constructed snapshots as source authority, another graph reader, source-free execution, re-encoding in candidate construction, or treating a committed outbox as completed delivery |
| Share one computed revision across business, audit and payload | `ApplicationOperationBodyValue::admit`, `body.assign_value`, `creation.initialize_value`, and `emit_external_evaluation` with `ApplicationOperationBodyEvaluationSnapshot`. Native Foundation evaluation prepares once and validates every installed destination codec. Unitless UInt64 results; at least one field sink. Computed scope bookkeeping may accompany signing, with old revision captured as AuditContext; computed signed-artifact initializers remain refused. | [Shared Computed Body Values](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-body-values.md) | Host/codec arithmetic, caller-supplied computed answers, copied expressions per sink, treating observed query data as the postimage, fabricating snapshots as authority, or rewriting a lifecycle Subject source while signing |
| Retain correlated native query values for expressions | `request.query(intent).execute_with_expression_context()`; inspect `observed_source.expression_context()`. Exactly-one one-shot result; native disclosed values remain attached to recursive source dependencies before any domain projector. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md) | Reconstructing values from DTOs, zipped parallel lists, treating omitted data as empty/absent, or treating descriptive contexts as mutation/read authority |
| Require correlated native query information | `operation.requirement_operands().query_context::<Binding, _, _, _, _>(name, query_definition)` consumes the exact parameter-free, exactly-one query declaration. Supply its sealed source with `expect_source` to guidance and execution; native facts remain publication obligations. | [Authored context requirements](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#real-example) | Host-built expression records or readiness booleans, a second validation loop, weakening source parameters through a callback, or treating a descriptive Met finding as workflow authority |
| Require values across a related collection | `operation.requirement_operands().outgoing_field_values(name, relation, child_field)` binds a tracked `List<Value>` or `List<Option<Value>>`; use the existing Foundation expression for length and all/any. Same requirement drives guidance and mutation admission, with membership/liveness/field publication dependencies | [Related-item values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#require-values-on-every-related-item) | Caller-supplied lists, host validation loops, count-only readiness, filtering away missing values, or treating independently collected lists as a correlated row API |
| Return the scope, one selected outgoing subject or a created row from an authored body | `decl::application_schema`: `body.preserve_scope_output::<Binding, Role>()`, `body.preserve_related_output::<Binding, Role>(relation)` (Preserve/ExactlyOne), `creation.create_output::<Binding, Role>()`; read the real receipt's `outputs_of::<Contract>()?.entity::<Role>()?`. Related selection is tracked once and retained through publication; replay preserves original identity. | [Scope-bound operation bodies](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#stable-entry-points) | Caller-selected output IDs, post-commit pointer lookups as correspondence, dummy writes, or treating a preserved identity as current approval/workflow authority |
| Supply original typed inputs for native workflow needs | `WorthQueryWorkflowOperationInputs::new(program_runtime).with::<InstalledAction, _>(description)` then `select(workflow, &required)`. One entry per binding; native need selects order and existing native owners retain all progress/refusal/completion/cleanup custody. Source-completion actions only. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) | Host stage/name dispatch, recapturing transferred input/key, treating a finite pack as a durable or atomic whole-command runner |
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build §3](build-an-application.md#3-declare-features-and-the-program) | [Consumer prohibition](#prohibition-declare-features-and-a-program) |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build §4](build-an-application.md#4-install-the-application-graph) | [Consumer prohibition](#prohibition-install-the-application-graph) |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) | Rebuilding original fields; treating resource custody as permission; charging registry bytes as total descriptor heap |
| Compose one complete action | `ApplicationFeatureSpec::root::<S, F>()` or `at::<S, I, F>()`, then `.action::<Binding, Delivery>()`, optional `.locality_and_change::<Scope, Shape>()`, `.correspondence::<C>()`, `.evaluated_requirement::<Rule>()`, `.external_input::<Provider>()`, and `.finish_action()` before feature completion. Each clause is once-only; delivery is explicit. `ApplicationActionReference<S, I, F, B, D>` selects complete actual installed meaning without manufacturing it. | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) | Separate declarations for delivery and metadata; mutable declaration bags; duplicate portable action keys; a second registry or policy engine |
| Store fixed or discovered work beyond construction | `WorthQueryFixedOperation<S, Intent, Program, Root, Action>` and `WorthQueryDiscoveredOperation<S, Intent, Program, Root, Action>` retain the authentic runtime allocation and complete installed action. Keep an independent genuine runtime handle for fresh requests on `owner.branch()`. All refusals return actual custody; runtime close refuses `RuntimeKeepaliveRetained` while operation keepalive remains. | [Owning actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#retained-runtime-and-configuration-custody) | Fabricated lifetimes, a replacement runtime, operation runtime access as admission, cloned native obligations, Drop as cleanup or durable continuation |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build §5.3](build-an-application.md#53-execute-through-the-program) | [Consumer prohibition](#prohibition-run-a-mutation) |
| Run a declared fixed action whose source feeds required outputs | Declare `.action::<Binding, Delivery>() ... .finish_action()` and its exact `ApplicationActionReference`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Construction reuses the native governed root-artifact/source guard before effects; invalid locality/change/dependency metadata returns the original description with `WorthQueryOperationConstructionDenial::OutputSource`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports finite `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis; eligible Required fixed-leaf terminal `close` invokes native unavailable completion, preserving actual negative evidence across cleanup refusal in `output_unavailable()` on terminal or closed custody. Source saved with no current output is not positive settlement | [Build §5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/…/application_program/required_unpublished.rs` | [Consumer prohibition](#prohibition-run-a-declared-fixed-action-whose-source-feeds-required-outputs) |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) | [Consumer prohibition](#prohibition-declare-discovered-output-delivery-meaning) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) | [Consumer prohibition](#prohibition-own-a-discovered-action-through-recovery-fanout-delivery-and-cleanup) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` | [Consumer prohibition](#prohibition-validate-an-explicitly-driven-discovered-aggregate-at-one-retained-observation) |
| Own a declaration-authorized no-output action | Declare `.action::<Binding, Delivery>() ... .finish_action()` with `ApplicationSourceCompletion`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode requires `advance(&fresh_request, maximum_history_work)` and delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) | [Consumer prohibition](#prohibition-own-a-declaration-authorized-no-output-action) |
| Retry a mutation safely | Retain the move-only operation returned by `Progress` or `Refused`, and supply fresh request authority to its next `advance`. Delivery refusal retains the settled owner; retry its declared read. Terminal custody exposes evidence and close, without advance. After process loss, checkpoint replay requires the full stable original native intent to be re-expressible under fresh authority (tested lawful no-source/unpinned cases, with all other intent material unchanged). Row/result-set source expectations bind the old runtime; a new observation is different intent, not promised historical replay. Do not serialize live owners, substitute a source posture the binding forbids, or change key/source to force replay. A current read alone cannot rule out an ambiguous external effect: reconcile its authoritative outcome through the existing native/domain/Store recovery owner before choosing a new operation; never automatically reissue an uncertain effect under a new key. Store durable restart and process-local owning custody remain distinct. | [Owning entry](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) | [Consumer prohibition](#prohibition-retry-a-mutation-safely) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) | [Consumer prohibition](#prohibition-control-who-may-do-what) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` | [Consumer prohibition](#prohibition-require-different-people-for-different-duties) |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) | Comparing an input actor, treating absence as different, or reauthorizing the historical actor as the current caller |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build §6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) | [Consumer prohibition](#prohibition-run-a-multi-step-process) |
| Start a fresh episode after a completed workflow | Scope/Preserve body `.succeed_workflow::<Spec, Binding, Role>()`; install `.succession_operation::<Binding, Role>()`, then native `prepare_workflow_succession` / `for_workflow_succession` with the same logical input/key. Preparation accepts the runtime-owned published reference or original pure start descriptor; after checkpoint restore, the latter requires fresh command admission and tracked native lookup. | [Completed-chain succession](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#completed-chain-succession) | Reviving the old instance, two host commits, inheriting approvals or effects, selecting an arbitrary historical definition, using an older predecessor to fork a chain, carrying old runtime handles across restore, or treating a descriptor as permission |
| Retain the actual outputs of a performed workflow operation | Native guarded settlement records the binding and actual role/entity/posture/reference inventory atomically with the transition; Existing, Created and Retire identities survive audit retention. No additional supplier flag or host write. | [Retained operation correspondence](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-operation-correspondence) | A receipt digest or current business lookup as historical identity; treating retained output as current-chain proof; rebinding foreign runtime instance handles |
| Require succession to continue the selected round or revision | Body `require_predecessor_output::<Binding, SelectedRole, Producer, ProducedRole>()` compares its sealed Preserve/ExactlyOne output with the latest performed occurrence of the declared guarded producer; both roles must name the same entity. Exact replay precedes fresh mismatch refusal. | [Succession output affinity](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-succession-output-affinity.md) | Caller-supplied historical IDs, root identity as round identity, fallback to an earlier occurrence, or a host-current-chain table |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `…for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build §6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-no-signature) |
| Certify shared and row-local facts without copying their source truth | A native signed creation uses `capture_from_path(ApplicationQueryRootPath, source, snapshot, Subject/AuditContext)` with v3 graph provenance. The existing lifecycle derives source-identity and selection-scoped supersession; `bound_subject_impacts` bounds the union of affected subjects. | [Graph-certified subjects](../workspaces/worth-query/crates/worth-query/docs/authoring/graph-certified-subjects.md) | Denormalizing shared facts to imitate scope-only signing; host dependency lists or invalidators; first/latest selection; treating decoded provenance as authority |
| Own capability delegation or revocation across fresh requests | `request.retain_capability_delegation_in_program(..)` / `retain_capability_revocation_in_program(..)` own original input, key, conditions and principal association. `advance(&fresh_request)` returns progress, same-owner refusal, native completion or terminal custody; `close_with_evidence` discharges cleanup before moving the receipt | [Owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands) | Flat keyed reissue loops; storing a request/grant; cloning a command; using an observation as permission; treating Drop as cleanup |
| Own a guarded workflow effect across fresh requests | Effect-free bound construction refusal returns the original binding; consuming `into_parts()` extracts its original description and requirement beyond the runtime borrow. `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) | Cached prepared admission; recreating an outbox handle after refusal; reissuing a spent slot; treating effect completion as workflow terminality |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) | Taking cleanup from a diagnostic denial clone; equating source save, observer close, evidence acceptance or approval |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event. For a compiled signed body on that approval control, use `prepare_workflow_approval_with_body(..)` to co-publish its attestation and approval under one event; exact settled replay needs no new event. Its evidence join accepts one or more distinct assessment inputs; empty applicable evidence never grants approval. | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Approval with attestation](../workspaces/worth-query/crates/worth-query/docs/authoring/signed-operation-creations.md#one-approval-action-with-a-business-attestation) | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-a-signature) |
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
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build §3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | [Consumer prohibition](#prohibition-produce-derived-or-computed-outputs) |
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
Native declarations have no output-source toggle. Configure initial workflow
vocabulary before cloning runtime handles or retaining operations. A refused
`retain_workflow_spec` preserves runtime, installed spec and signing owner in
`WorthQueryWorkflowRuntimeBindingRefusal::into_parts()`; refused
`support_workflow_spec` returns the original spec with its denial.
`WorthQueryWorkflowApplicationRuntime::close_conditional_runtime(&mut self)`
delegates native close while retaining the configured wrapper and its installed
workflow/signing companions on refusal. Close the actual operation owners and
retry the same wrapper; consuming it into a program runtime is unnecessary.

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

## Known gaps

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
