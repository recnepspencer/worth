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

4. **Check the [known gaps](#known-gaps).** If your need is listed there, follow
   its "Do not" column.
5. **Only then report the gap** ([Reporting a gap](#reporting-a-gap)). Do not
   build a substitute: no parallel registry, copied authority, host executor,
   host scheduler, or host persistence of platform truth.

---

## Task index

"Guide" links to the section that shows the exact calls. The "Never build"
column names the substitute that this capability makes wrong.

| I want to… | Use (facade path / call) | Guide | Never build |
|---|---|---|---|
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build §3](build-an-application.md#3-declare-features-and-the-program) | [Consumer prohibition](#prohibition-declare-features-and-a-program) |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build §4](build-an-application.md#4-install-the-application-graph) | [Consumer prohibition](#prohibition-install-the-application-graph) |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) | Rebuilding original fields; treating resource custody as permission; charging registry bytes as total descriptor heap |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build §5.3](build-an-application.md#53-execute-through-the-program) | [Consumer prohibition](#prohibition-run-a-mutation) |
| Run a declared fixed action whose source feeds required outputs | Declare `fixed_output_action::<Binding, Delivery>()` and its exact `ApplicationActionReference`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports finite `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis | [Build §5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/…/application_program/required_unpublished.rs` | [Consumer prohibition](#prohibition-run-a-declared-fixed-action-whose-source-feeds-required-outputs) |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) | [Consumer prohibition](#prohibition-declare-discovered-output-delivery-meaning) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) | [Consumer prohibition](#prohibition-own-a-discovered-action-through-recovery-fanout-delivery-and-cleanup) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` | [Consumer prohibition](#prohibition-validate-an-explicitly-driven-discovered-aggregate-at-one-retained-observation) |
| Own a declaration-authorized no-output action | Declare `operation_action::<Binding, Delivery>()` with `ApplicationSourceCompletion`; `…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode requires `advance(&fresh_request, maximum_history_work)` and delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) | [Consumer prohibition](#prohibition-own-a-declaration-authorized-no-output-action) |
| Retry a mutation safely | Retain the move-only operation returned by `Progress` or `Refused`, and supply fresh request authority to its next `advance`. Delivery refusal retains the settled owner; retry its declared read. Terminal custody exposes evidence and close, without advance. After process loss, checkpoint replay requires the full stable original native intent to be re-expressible under fresh authority (tested lawful no-source/unpinned cases, with all other intent material unchanged). Row/result-set source expectations bind the old runtime; a new observation is different intent, not promised historical replay. Do not serialize live owners, substitute a source posture the binding forbids, or change key/source to force replay. A current read alone cannot rule out an ambiguous external effect: reconcile its authoritative outcome through the existing native/domain/Store recovery owner before choosing a new operation; never automatically reissue an uncertain effect under a new key. Store durable restart and process-local owning custody remain distinct. | [Owning entry](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) | [Consumer prohibition](#prohibition-retry-a-mutation-safely) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) | [Consumer prohibition](#prohibition-control-who-may-do-what) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` | [Consumer prohibition](#prohibition-require-different-people-for-different-duties) |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) | Comparing an input actor, treating absence as different, or reauthorizing the historical actor as the current caller |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build §6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) | [Consumer prohibition](#prohibition-run-a-multi-step-process) |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `…for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build §6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-no-signature) |
| Own a guarded workflow effect across fresh requests | `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) | Cached prepared admission; recreating an outbox handle after refusal; reissuing a spent slot; treating effect completion as workflow terminality |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) | Taking cleanup from a diagnostic denial clone; equating source save, observer close, evidence acceptance or approval |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations) | [Consumer prohibition](#prohibition-run-a-workflow-step-that-needs-a-signature) |
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
| Read the original workflow instance at a retained observation | `request.at(&observation).read_original_workflow::<Spec, SubjectRead, AuditRead>(workflow.vocabulary(), &original_intent, subject_input, audit_input)` uses fresh installed read bindings before lookup. `Live`, `Historical` and authorized `Missing` retain exact basis data; audit is independently admitted and otherwise explicitly withheld. This unreleased surface awaits joined compilation and certification | [Governed original-intent reads](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#governed-original-intent-reads) | Supplying an actor/principal entity as read authority; querying a latest basis behind a retained observation; treating a view or live locator as a command grant |
| Retry committed discovered-source retention | Ordinary `advance` retains and retries the native owner. For deliberate native control, retain the opaque `WorthQueryDiscoveredOutputRetentionFailure`; use a fresh original `advanced()` native request's `retry_discovered_output_retention(failure, application)`. Refusal returns the original owner; success promotes its actual carrier before starting native outputs | [Committed discovered retention](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#committed-discovered-retention) | [Consumer prohibition](#prohibition-retry-committed-discovered-source-retention) |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build §3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | [Consumer prohibition](#prohibition-produce-derived-or-computed-outputs) |
| Undo or redo a change | **Provisional.** `host::provisional_aftermath` exists, but undo and redo are experiments; Milestone 9.18 owns any accepted contract | [Front door: current limits](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#current-limits) | [Consumer prohibition](#prohibition-undo-or-redo-a-change) |

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

Comparing actor ids from input in a handler. See the known gap on workflow proposers

### Prohibition: Run a multi step process

A host state machine or status column that decides the next step

### Prohibition: Run a workflow step that needs no signature

A dummy approval or auto-approving principal. Omitting `workflow_requirements` means exactly one approval, not none

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

Do not treat it as a fault to retry blindly.

<!-- consumer-prohibitions end -->

## Known gaps

These are real, consumer-visible limits. Each has an answer for today.

The remaining workflow gaps below are scheduled platform work, owned by
[Milestone 9.17.6.5](../plans/WORTH-query/milestone-9.17.6.5.md). Do not build a
substitute while you wait: the linked canonical constraints apply until a row is
removed. A plan describes intent, not API; use only what the facades export.

| Gap | What exists today | Do not |
|---|---|---|
| Complete workflow observation and paged proposal/approval history remain unfinished | The unreleased original-intent read exposes exact state and bounded recorded transition paths under separate subject/audit admission. Other existing observations are the typed outcomes your own requests return: `PerformedWorkflowTransition`, `PerformedWorkflowApproval` (`approver()`), `Awaiting*`. Idempotent replay of the same key returns the recorded outcome. | Do not mirror kernel state into your schema as truth or use a status table to decide whether to advance. Advance and match the typed outcome. |
| Instance and proposal references cannot be rebuilt in another process (`PublishedWorkflowInstanceRef` / `PublishedWorkflowProposalRef` have no public constructor or serialization; definition/subject enumeration remains unfinished) | Retain the opaque `discovery_intent()` from the original prepared start for the governed exact lookup. Proposal reacquisition and enumeration remain unfinished. | Do not serialize, forge, or re-derive references. Do not build an instance registry and treat it as authority. |
| No outcome names "awaiting a proposal" | `PreparationDenied(WorkflowTransitionNodeUnsupported)` with the node path. | Do not treat it as a fault to retry blindly. |

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
