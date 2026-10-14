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
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build §3](build-an-application.md#3-declare-features-and-the-program) | A second feature or action registry; runtime feature flags that decide which operations exist |
| Install the application graph | `host::application_installation::in_memory_program(..)`, or `in_memory_rostered_program(..)` with a `WorthQueryApplicationProgramRoster` | [Build §4](build-an-application.md#4-install-the-application-graph) | A host-side schema or program catalog |
| Run a mutation | `runtime.request(&principal, &scope).mutate(intent).without_source().idempotency(&key).execute_in_program(&runtime, allocation)`, where `allocation` is a `host::runtime::ExecutionAllocationPolicy` (for example `SystemAllocation`) | [Build §5.3](build-an-application.md#53-execute-through-the-program), [Front door: branch-resolved lanes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#branch-resolved-mutation-lanes) | Direct graph writes; a host executor that calls handlers; the non-program `execute(allocation)` on a program runtime (refused with `ApplicationProgramRequired`) |
| Run a declared fixed action whose source feeds required outputs | Declare `fixed_output_action::<Binding, Delivery>()` and its exact `ApplicationActionReference`; `…idempotency_owned(key).retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. Settlement supports `read_current(&fresh_request)` and refusal-preserving `close(&fresh_request)` | [Build §5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/…/application_program/required_unpublished.rs` | Rebuilding the intent or key after a partial; reading at a different observation from currentness validation; dropping a refused cleanup owner |
| Retry a mutation safely | Send the same intent with the **same** idempotency key. Match `host::application_entry::WorthQueryApplicationMutationOutcome`: `Committed`, `AlreadyCommitted`, `PreviouslyCommitted`, `IdempotencyIntentDrift`, `DomainDenied`, `Cancelled`, `DeadlineExceeded`, `Commit(..)`. For an unpublished commit: `recover_unpublished_in_program(..)`; to read the keyed outcome only: `resolve_idempotency_in_program(..)` | [Build §5.2](build-an-application.md#52-mutations-intent-binding-handler), [Front door: branch-resolved lanes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#branch-resolved-mutation-lanes), [Aftermath and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md) | A new key per retry; a "did it commit?" table; treating `Commit(..)` as a plain failure |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding that implements `ApplicationCapabilityMutationBinding` runs through `execute_capability_in_program(&runtime, allocation)` | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) | Role booleans checked in a handler, UI, or transport; treating authentication as permission |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..))` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` | Comparing actor ids from input in a handler. See the known gap on workflow proposers |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build §6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) | A host state machine or status column that decides the next step |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::new()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `…for_workflow_operation(&workflow, &required)?.execute_in_program(..)` | [Build §6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` | A dummy approval or auto-approving principal. Omitting `workflow_requirements` means exactly one approval, not none |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations) | A signature-shaped row or checkbox; one capability for several meanings of a signature |
| Act on time or a signal | A conditional operation: `.conditional_operation::<Op>()` on the feature spec, `host::worth_query_conditional_node!`, and a host-supplied predicate and named clock | [Build §3.2](build-an-application.md#32-attach-actions-to-features), [Conditional installed operations](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md) | A host scheduler, cron job, timer loop, or wake registry |
| Complete work from an external system | An installed inbound source: `runtime.inbound_occurrences().receive(source, bytes, scope).execute()` (`host::application_entry::WorthQueryApplicationInboundOccurrencesExt`); in a workflow, `await_inbound::<Effect>(..)` | [Aftermath: authenticated inbound completion](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#authenticated-inbound-completion), [Workflows: waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion) | A callback that writes truth or resumes an instance; a correlation token as authority |
| Survive a restart | The app owns the artifact. Capture: inside `runtime.with_host_advancement(..)`, call `runtime.capture_application_checkpoint_with_sections(&phase, policy)` with a `host::application_installation::WorthQueryCheckpointCapturePolicy`. Restore: `in_memory_program_from_checkpoint(..)` or `in_memory_rostered_program_from_checkpoint(..)` | [Front door: capture and reopen](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#capture-and-reopen-cost-boundary), [Build §4.4](build-an-application.md#44-checkpoint-program-transitions) | Serializing live handles or references; mirroring platform truth in your own store as authority |
| Work on a branch or move it to a new program under its declared capability | `runtime.branches().fork(runtime.current_world()).components(..).create()`; `request(..).on_branch(branch)`; `.programs()` then `compare(&target)`, `adopt(&requirements)?.prepare(max)`, `programs.publish(prepared)` | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Build §7](build-an-application.md#7-adopt-a-new-program-on-a-branch) | Copying data into a scratch table; a process-wide "latest program" switch |
| Issue or revoke Bank program deployment permission | Explicit `BankProgramDeploymentGrantSeed::for_program::<Source>(..)` in `BankWorldSeed`; recorded issuer calls `revoke_program_deployment_grant(..)` | [Build §7.1](build-an-application.md#71-the-call-chain) | Reusing estate/payment permission; granting every authenticated actor; selecting the target’s permission |
| Read current data | `request(..).query(intent).execute()`; exact reads with `retain_read()` and `at(&observation)`; live reads with `query(intent).subscribe(..)` | [Host README: ordinary typed query entry](../workspaces/worth-query/crates/worth-query-host/README.md#ordinary-typed-query-entry), [Host README: output demand and live reads](../workspaces/worth-query/crates/worth-query-host/README.md#output-demand-exact-observation-and-live-reads) | Reading lower runtimes directly; a cache treated as truth |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()` | [Build §3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build §3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) | Recomputing in the app and storing the result as authoritative |
| Undo or redo a change | **Provisional.** `host::provisional_aftermath` exists, but undo and redo are experiments; Milestone 9.18 owns any accepted contract | [Front door: current limits](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#current-limits) | A generic undo engine over committed history. A business reversal is an ordinary declared operation |

---

## Known gaps

These are real, consumer-visible limits. Each has an answer for today.

The four workflow gaps below are scheduled platform work, owned by
[Milestone 9.17.6.5](../plans/WORTH-query/milestone-9.17.6.5.md). Do not build a
substitute while you wait: the "Do not" column is the rule until a row is
removed. A plan describes intent, not API; use only what the facades export.

| Gap | What exists today | Do not |
|---|---|---|
| No facade read of a workflow instance's state or history (current node, transitions, proposals, approvals) | Only the typed outcomes your own requests return: `PerformedWorkflowTransition`, `PerformedWorkflowApproval` (`approver()`), `Awaiting*`. Idempotent replay of the same key returns the recorded outcome. | Do not mirror kernel state into your schema as truth or use a status table to decide whether to advance. Advance and match the typed outcome. |
| Instance and proposal references cannot be rebuilt in another process (`PublishedWorkflowInstanceRef` / `PublishedWorkflowProposalRef` have no public constructor or serialization; no instance discovery) | Keep the references in the process that owns the instance. | Do not serialize, forge, or re-derive references. Do not build an instance registry and treat it as authority. |
| Proposal and transition records do not retain the acting principal (approvals do retain the approver) | Capability separation-of-duty and distinct-actor rules exist but cannot see the proposer. | Do not treat a caller-supplied "actor" field in input as admitted identity. |
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
