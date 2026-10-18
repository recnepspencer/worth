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
their own grants. This storage supplies no result, source or request authority.
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

For bounded nested persisted values, use the existing value-binding macro's
`scalar: String, canonical: ValueDecodeBounds::new(...)` arm with Query-free
`WorthValue` derivation. The installed value-binding catalog retains the derived
shape and constant bounds. Guarded recursion uses finite bound references;
`Box`, exact pair payloads and internally tagged unit/named variants use the
same bounded seed path. Tag-first canonical objects never require an arbitrary
object buffer. Complete closure admission rejects an unsupported opaque child,
including an unselected variant. Pure schema owners need no Execution or Query
dependency for these descriptions. Native canonical projection reads dispatch the exact
installed codec under the original request's cumulative execution allowance; ignored
admission failures veto completion. Opaque owners must supply admitted seeds.
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
External or rewritten relations, protected or partly classified fields, affected
invariants, changed ordering keys and join consequences refuse before
publication; ordinary restore remains exact. Surviving application relations
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
See [bounded canonical nested values](build-an-application.md#bounded-canonical-nested-values)
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

| I want toâ€¦ | Use (facade path / call) | Guide | Never build |
|---|---|---|---|
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()â€¦finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build Â§3](build-an-application.md#3-declare-features-and-the-program) | [Consumer prohibition](#prohibition-declare-features-and-a-program) |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build Â§4](build-an-application.md#4-install-the-application-graph) | [Consumer prohibition](#prohibition-install-the-application-graph) |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) | Rebuilding original fields; treating resource custody as permission; charging registry bytes as total descriptor heap |
| Compose one complete action | `ApplicationFeatureSpec::root::<S, F>()` or `at::<S, I, F>()`, then `.action::<Binding, Delivery>()`, optional `.locality_and_change::<Scope, Shape>()`, `.correspondence::<C>()`, `.evaluated_requirement::<Rule>()`, `.external_input::<Provider>()`, and `.finish_action()` before feature completion. Each clause is once-only; delivery is explicit. `ApplicationActionReference<S, I, F, B, D>` selects complete actual installed meaning without manufacturing it. | [Build Â§3.2](build-an-application.md#32-attach-actions-to-features), [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) | Separate declarations for delivery and metadata; mutable declaration bags; duplicate portable action keys; a second registry or policy engine |
| Store fixed or discovered work beyond construction | `WorthQueryFixedOperation<S, Intent, Program, Root, Action>` and `WorthQueryDiscoveredOperation<S, Intent, Program, Root, Action>` retain the authentic runtime allocation and complete installed action. Keep an independent genuine runtime handle for fresh requests on `owner.branch()`. All refusals return actual custody; runtime close refuses `RuntimeKeepaliveRetained` while operation keepalive remains. | [Owning actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#retained-runtime-and-configuration-custody) | Fabricated lifetimes, a replacement runtime, operation runtime access as admission, cloned native obligations, Drop as cleanup or durable continuation |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build Â§5.3](build-an-application.md#53-execute-through-the-program) | [Consumer prohibition](#prohibition-run-a-mutation) |
| Run a declared fixed action whose source feeds required outputs | Declare `.action::<Binding, Delivery>() ... .finish_action()` and its exact `ApplicationActionReference`; `â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Construction reuses the native governed root-artifact/source guard before effects; invalid locality/change/dependency metadata returns the original description with `WorthQueryOperationConstructionDenial::OutputSource`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports finite `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis; eligible Required fixed-leaf terminal `close` invokes native unavailable completion, preserving actual negative evidence across cleanup refusal in `output_unavailable()` on terminal or closed custody. Source saved with no current output is not positive settlement | [Build Â§5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/â€¦/application_program/required_unpublished.rs` | [Consumer prohibition](#prohibition-run-a-declared-fixed-action-whose-source-feeds-required-outputs) |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) | [Consumer prohibition](#prohibition-declare-discovered-output-delivery-meaning) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) | [Consumer prohibition](#prohibition-own-a-discovered-action-through-recovery-fanout-delivery-and-cleanup) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` | [Consumer prohibition](#prohibition-validate-an-explicitly-driven-discovered-aggregate-at-one-retained-observation) |
| Own a declaration-authorized no-output action | Declare `.action::<Binding, Delivery>() ... .finish_action()` with `ApplicationSourceCompletion`; `â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode requires `advance(&fresh_request, maximum_history_work)` and delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) | [Consumer prohibition](#prohibition-own-a-declaration-authorized-no-output-action) |
| Retry a mutation safely | Retain the move-only operation returned by `Progress` or `Refused`, and supply fresh request authority to its next `advance`. Delivery refusal retains the settled owner; retry its declared read. Terminal custody exposes evidence and close, without advance. After process loss, checkpoint replay requires the full stable original native intent to be re-expressible under fresh authority (tested lawful no-source/unpinned cases, with all other intent material unchanged). Row/result-set source expectations bind the old runtime; a new observation is different intent, not promised historical replay. Do not serialize live owners, substitute a source posture the binding forbids, or change key/source to force replay. A current read alone cannot rule out an ambiguous external effect: reconcile its authoritative outcome through the existing native/domain/Store recovery owner before choosing a new operation; never automatically reissue an uncertain effect under a new key. Store durable restart and process-local owning custody remain distinct. | [Owning entry](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) | [Consumer prohibition](#prohibition-retry-a-mutation-safely) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) | [Consumer prohibition](#prohibition-control-who-may-do-what) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` | [Consumer prohibition](#prohibition-require-different-people-for-different-duties) |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) | Comparing an input actor, treating absence as different, or reauthorizing the historical actor as the current caller |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build Â§6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) | [Consumer prohibition](#prohibition-run-a-multi-step-process) |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `â€¦for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build Â§6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-no-signature) |
| Own capability delegation or revocation across fresh requests | `request.retain_capability_delegation_in_program(..)` / `retain_capability_revocation_in_program(..)` own original input, key, conditions and principal association. `advance(&fresh_request)` returns progress, same-owner refusal, native completion or terminal custody; `close_with_evidence` discharges cleanup before moving the receipt | [Owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands) | Flat keyed reissue loops; storing a request/grant; cloning a command; using an observation as permission; treating Drop as cleanup |
| Own a guarded workflow effect across fresh requests | Effect-free bound construction refusal returns the original binding; consuming `into_parts()` extracts its original description and requirement beyond the runtime borrow. `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) | Cached prepared admission; recreating an outbox handle after refusal; reissuing a spent slot; treating effect completion as workflow terminality |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) | Taking cleanup from a diagnostic denial clone; equating source save, observer close, evidence acceptance or approval |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations) | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-a-signature) |
| Act on time or a signal | A conditional operation: `.conditional_operation::<Op>()` on the feature spec, `host::worth_query_conditional_node!`, and a host-supplied predicate and named clock | [Build Â§3.2](build-an-application.md#32-attach-actions-to-features), [Conditional installed operations](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md) | [Consumer prohibition](#prohibition-act-on-time-or-a-signal) |
| Complete work from an external system | An installed inbound source: `runtime.inbound_occurrences().receive(source, bytes, scope).execute()` (`host::application_entry::WorthQueryApplicationInboundOccurrencesExt`); in a workflow, `await_inbound::<Effect>(..)` | [Aftermath: authenticated inbound completion](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#authenticated-inbound-completion), [Workflows: waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion) | [Consumer prohibition](#prohibition-complete-work-from-an-external-system) |
| Survive a restart | The app owns the artifact. Capture: inside `runtime.with_host_advancement(..)`, call `runtime.capture_application_checkpoint_with_sections(&phase, policy)` with a `host::application_installation::WorthQueryCheckpointCapturePolicy`. Restore: `in_memory_program_from_checkpoint(..)` or `in_memory_rostered_program_from_checkpoint(..)` | [Front door: capture and reopen](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#capture-and-reopen-cost-boundary), [Build Â§4.4](build-an-application.md#44-checkpoint-program-transitions) | [Consumer prohibition](#prohibition-survive-a-restart) |
| Work on a branch or move it to a new program under its declared capability | `runtime.branches().fork(runtime.current_world()).components(..).create()`; `request(..).on_branch(branch)`; `.programs()` then `compare(&target)`, `adopt(&requirements)?.prepare(max)`, `programs.publish(prepared)` | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Build Â§7](build-an-application.md#7-adopt-a-new-program-on-a-branch) | [Consumer prohibition](#prohibition-work-on-a-branch-or-move-it-to-a-new-program-under-its-declared-capability) |
| Own Bank deposit, withdrawal and send-money transport custody | Typed money builders use `retain_operation()` and explicit retained history bounds. Strict v4 create/resume/read/close commands store actual native owners under finite capacity; retain `BankHttpMutationContinuation` before activating the prepared listener. Fresh commands provide authority, while listener loss and shutdown refusal preserve independent resumption | [Owning HTTP money](../workspaces/worth-query-bank-world/docs/http-owning-money.md), [Bank consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) | [Consumer prohibition](#prohibition-own-bank-deposit-withdrawal-and-send-money-transport-custody) |
| Own plain Bank account, access or payment mutation | Bank builders for personal/business account creation, access grant/revocation, initiation and either rejection form use `retain_operation()` and native SourceCompletion; supply fresh authority to each `advance` and `close`. Fresh results and recovered receipt evidence remain distinct | [Bank owning completion guidance](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-account-access-and-payment-completion) | [Consumer prohibition](#prohibition-own-plain-bank-account-access-or-payment-mutation) |
| Own Bank estate source or retained funding/reversal | Typed Bank mutation builders use exact declared SourceCompletion or existing RetainedCommit; `close_with_evidence` transfers original completion only after native cleanup | [Bank source and publication handoff](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-estate-sources-funding-reversal-and-deployment-revocation) | [Consumer prohibition](#prohibition-own-bank-estate-source-or-retained-funding-reversal) |
| Retain Bank HTTP native cleanup and business evidence | The listener-independent mutation continuation stores actual source and elevation cleanup owners under finite custody. Fresh `release_cleanup` retries on the original execution thread; the local `take_released_cleanup` handoff preserves actual uncommitted evidence and the installed runtime after physical release | [HTTP cleanup custody](../workspaces/worth-query-bank-world/docs/http-owning-money.md#estate-and-native-cleanup-custody) | [Consumer prohibition](#prohibition-retain-bank-http-native-cleanup-and-business-evidence) |
| Issue or revoke Bank program deployment permission | Explicit `BankProgramDeploymentGrantSeed::for_program::<Source>(..)` in `BankWorldSeed`; recorded issuer calls `revoke_program_deployment_grant(..)` | [Build Â§7.1](build-an-application.md#71-the-call-chain) | [Consumer prohibition](#prohibition-issue-or-revoke-bank-program-deployment-permission) |
| Read current data | `request(..).query(intent).execute()`; exact reads with `retain_read()` and `at(&observation)`; live reads with `query(intent).subscribe(..)` | [Host README: ordinary typed query entry](../workspaces/worth-query/crates/worth-query-host/README.md#ordinary-typed-query-entry), [Host README: output demand and live reads](../workspaces/worth-query/crates/worth-query-host/README.md#output-demand-exact-observation-and-live-reads) | [Consumer prohibition](#prohibition-read-current-data) |
| Read the original workflow instance at a retained observation | `request.at(&observation).read_original_workflow::<Spec, SubjectRead, AuditRead>(workflow.vocabulary(), &original_intent, subject_input, audit_input)` uses fresh installed read bindings before lookup. `Live`, `Historical` and authorized `Missing` retain exact basis data; audit is independently admitted and otherwise explicitly withheld. | [Governed original-intent reads](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#governed-original-intent-reads) | Supplying an actor/principal entity as read authority; querying a latest basis behind a retained observation; treating a view or live locator as a command grant |
| Read bounded workflow history and reacquire proposal references | `request.at(&data).read_workflow_history::<Spec, SubjectRead, AuditRead>(vocabulary, target, subject_input, audit_input, width, indexes, after)` returns exact-basis pages with independently withheld audit. `subscribe_workflow_history` / `poll_workflow_history` use native envelope dependency marks and fresh disclosure. | [Retained history](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-proposal-and-approval-history) | Treating a recorded locator, decision or actor as permission; refreshing a retained basis on a mark; retaining a revoked read grant |
| Read one workflow definition's metadata | Retained `discover_workflow_definition` with explicit typed `ApplicationWorkflowDefinitionReadTarget` and the separately installed `DefinitionMetadata` binding; fresh admission precedes selected-index lookup. | [Governed definition metadata](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#discover-and-retire-definitions) | Borrowing subject permission for metadata or whole-inventory disclosure; execution-only lookup |
| Retry committed discovered-source retention | Ordinary `advance` retains and retries the native owner. For deliberate native control, retain the opaque `WorthQueryDiscoveredOutputRetentionFailure`; use a fresh original `advanced()` native request's `retry_discovered_output_retention(failure, application)`. Refusal returns the original owner; success promotes its actual carrier before starting native outputs | [Committed discovered retention](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#committed-discovered-retention) | [Consumer prohibition](#prohibition-retry-committed-discovered-source-retention) |
| Retain an application preview through readmission, discard or promotion refusal | `runtime.begin_preview(&phase, &observation, request)`; `preview.readmit(&phase, &runtime)`; `readmitted.promote()`; either phase's `discard()` returns its same owner on refusal. Fresh readmission validates the original source and runtime; promotion finalizes retained preview evidence, separately from source/output publication. Rejected preview remains explicitly discardable. Name its native result vocabulary through `worth_query_host::facade::application_installation::{BridgeSpeculationError, BridgeSpeculationErrorKind, BridgeSpeculativePromotionOutcome}` | [Preview custody](build-an-application.md#56-retain-preview-custody-across-refusal) | Auto-discarding on an admission stop; reconstructing a preview from its identity or receipt; treating Drop/liveness as successful cleanup; promoting stale source as fresh output |
| Own a complete no-source output tree | `request.demand(original).controls(controls).retain_program_outputs::<Program, Root>(&program)` retains the actual installed full tree. Owned construction refusal supports fresh `retry` without replacing an issued basis. `advance` returns pending/refused/settled/terminal custody. Settled `read_current` checks all original outputs and discovery before reading at the same native observation and phase. Every phase can `close`; an owned close refusal supports `retry_close`, including after source supersession. Native unpublished and uncertain obligations remain owned; close delegates lawful original native cleanup/recovery and otherwise refuses | [Complete no-source trees](build-an-application.md#55-own-a-complete-no-source-output-tree) | Treating `start_in_program` root-only settlement as a full tree; fake no-op source actions; receipt reconstruction; dropping native recovery custody; claiming current output from read-request close |
| Close original complete output custody before fresh final delivery | Settled Fixed and complete no-source parents expose `close_for_delivery(&fresh_request)`. Actual native cleanup/interest close precedes sealed `WorthQueryClosedFixedDelivery` or `WorthQueryClosedProgramOutputDelivery`. Their existing-shape `deliver_current`/`read_current` methods validate the original full tree and installed meaning at a fresh native observation. To show several closed carriers current at one common basis, each no-projection Fixed and no-source carrier's `deliver_current_at(&request.at(&observation), policy)` runs that same installed-action or selected-root gate and full lineage at the caller's observation. Fixed close refusal returns the same parent; no-source `WorthQueryProgramOutputDeliveryCloseRefusal` supports close-only `retry_close` | [Delivery after native close](build-an-application.md#delivery-after-native-close), Host `closed_delivery_contract` construction/phase/shape family | Reopening released interests, replaying source/producer work, treating root evidence, commit equality or bare `require_current_program_output` lineage as complete currentness, retaining readonly history through whole-runtime shutdown |
| Inspect original root output roles without a producer receipt | `host::application_entry::WorthQueryApplicationProgramOutputSettlement::root_settlement()` borrows the actual immutable root demand settlement; use its `outputs_of::<Contract>()` typed correspondence even when `root_receipt()` is `None`. Parent operation retains all tree/close custody; this root-only evidence does not establish fresh complete-tree currentness | [Original root output evidence](build-an-application.md#original-root-output-evidence) | Reconstructing receipts, querying keys as output-role proof, starting a replacement demand, or treating a root borrow as aggregate currentness/discharge |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build Â§3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build Â§3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | [Consumer prohibition](#prohibition-produce-derived-or-computed-outputs) |
| Poll cancellation/deadline in a decision handler | `DecisionReader::checkpoint()` polls the real request. Preparation retains its first failure and refuses before candidate authoring even if caught. Successful polls preserve completed-decision input-reuse eligibility under the existing producer/context contract. | [Build Â§5.1](build-an-application.md#51-who-is-asking-principal-and-request-scope) | Treating a caught interruption as publication permission or declaring untracked request data trusted |
| Bind a derived collection to checked computation | `ApplicationDerivedCollection` requires `Computation` and `Completeness`; declare the computation beside the collection and install with `setup.derived_collection::<Feature, Collection, _>(owner)`. The owner's output and stop types must match the completeness contract. Initial/Create and Preserve/Preserve producers may share that installed owner and operation with a common no-intent input: implement `ApplicationMutationInputScope<Schema, Binding>` for each selected mutation binding. Ordinary intent inputs delegate their existing single-binding scope. Scope selection grants no execution authority. Complete installed operation/workflow-stage target coverage is checked before initialization, separately from exact per-binding route inventory; `installed_conditional_targets_for_schema` exposes read-only full locations, and conditional declarations use `in_workflow_stage` for stage selection. Missing or misbound installation refuses before bootstrap effects; the existing keyed computation checks its completed value before publication. Native partition reuse selects retained facts from the exact settlement frontier and consumes validation at the same pinned reader snapshot; applications provide no dirty hints. Successful native reduction retains its Execution-owned run; a fresh descendant request advances only changed inputs and actual removals, sharing unchanged native results and fitting cost ranges. Fixed-membership tree updates need no full shape plan or deletion scan. Execution owns first-stop and tight-prefix charging; applications cannot supply a result handle or checkpoint tape. Context-sensitive or nested kernels execute rather than retaining unsupported custody. Fixed membership with unchanged keys also carries checked reader-call ranges and immutable item/call bounds; ordinary preflight/charges and exact reached-entity scope still apply. Qualified native postcommit rebase consumes paired original/postcondition custody, the complete snapshot-aligned frontier and actual candidate touches; fresh suffixes and changed leaf sizing remain charged. Missing association or continuity reconstructs in full. Qualified completed original physical reads share checked locator/effect routing through a genuinely fresh registered session. Both comparisons retain native source and registration checks; changed or unqualified source uses funded full original-fact comparison. Principal and source-role observations remain individual. Posting registration and membership processing remain separate obligations; key/membership changes retain full preparation. | [Build Â§3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | An identity-only declaration, unchecked computation registration, or a presentation view as aggregate authority |
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

Reusing estate/payment permission; granting every authenticated actor; selecting the targetâ€™s permission

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
- Do not treat names found only in `plans/` as real. Check [Â§9](build-an-application.md#9-use-the-consumer-vocabulary).
- Do not build a substitute for a capability you could not find: no parallel
  registry, copied authority, host executor or scheduler, or host persistence
  of platform truth. Follow the missing-capability procedure in the
  [Capability Index](capability-index.md).
- Do not present `compare_and_commit_program_undo` / `_redo` as an accepted
  correction contract. They are provisional ([Â§5.2](build-an-application.md#52-mutations-intent-binding-handler)).

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

**Compiler-derived implementation editions.** Automatic producer/output/partition
edition integration is not yet a Decl/Host capability. The initial
[`implementation-edition` compiler observer](../tools/implementation-edition/src/lib.rs)
captures supported Rust/compiler inputs, verifies external bytes against locked
registry archives or Git trees, and checks finite generator effects against actual
execution. Serde Core 1.0.229 has a real compiler/generated-input proof; unqualified
proc-macro, native and generator contexts refuse. It does not yet qualify the CAD
closure or change installed output currentness. Consumers must await the native
cutover rather than construct editions from descriptive digests or reports.

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
