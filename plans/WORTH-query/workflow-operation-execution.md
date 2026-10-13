# Generic native workflow-operation execution

Accepted correction to Query's workflow-operation join. This specification governs
the public platform change; its existence does not establish implementation or
consumer acceptance. It extends the completed program/workflow foundations rather
than reopening their milestones or creating another executor.

## Claim and boundary

Query binds a business mutation to its exact current workflow operation occurrence.
The supplier declares ordinary permissions, business-state preconditions, and
additional required evidence. Signed approval is one possible prerequisite, not
the universal source of workflow execution authority.

Keep `REQUIRES_WORKFLOW_AUTHORITY`, ordinary operation authorization, and the
existing `for_workflow_operation(...).execute_in_program(...)` funnel. Do not add
signed/unsigned authorization modes, dummy approvals, caller permit flags,
arbitrary proof plugins, or a host state machine.

## Adversarial proof

Through the public declaration/host facades, install an information-recording
binding and an approval-dependent binding over a real domain operation. Establish
their resources, permissions, proposals, and progression through existing owners.

- Execute the information step without signing. Independently read its business
  result and advance to the declared successor: local effect and native operation
  settlement must publish atomically.
- An editable definition cannot remove/substitute the approval required by an
  installed binding. Preserve both legacy required approval and explicit new
  required approval. An approval edge added to an otherwise empty evidence floor
  is still an obligation; revoking its real grant must deny execution.
- Issue independent requirements for the same occurrence; commit one, then try
  the other under a different key. No repeated effect or settlement may land.
  Retrying the original exact intent/key recovers the performed outcome instead.
- A direct mutation, wrong input/binding/branch/runtime, stale occurrence, or
  unauthorized principal must not bypass admission. Permission to pump is not
  permission to execute the business operation.
- Destroy derived compilation/progress and reconstruct from retained facts.
  Explicit empty/new approval contracts retain their meaning, and execution needs
  freshly issued authority. An old live authority is never serialized or restored.

Use the existing document-retention public-entry target and approval/recovery
proofs. Add cases for changed joins, not duplicate suites. This is a native API
boundary proof, not HTTP, browser, external-provider, or Store durability evidence.

## Declaration and installed meaning

`ApplicationMutationBinding::workflow_execution_requirements()` returns an optional
`ApplicationWorkflowExecutionRequirements` declaration. The optional macro clause
`workflow_requirements` projects to that same method; it is not a separate registry.

```rust,ignore
// A new occurrence-bound information step:
workflow_authority required,
workflow_requirements ApplicationWorkflowExecutionRequirements::new(),

// An operation requiring a specific native review capability instead:
workflow_authority required,
workflow_requirements ApplicationWorkflowExecutionRequirements::requiring_approval::<ReleaseReviewCapability>(),
```

The approval constructor is immutable: chained calls cannot silently replace a
previous requirement. The product carries descriptive meaning only. It names the
native approval contract through a typed capability and resolves in the installed
schema/vocabulary, not through labels or a signature-shaped record.

An absent declaration preserves historical guarded semantics: exactly one native
approval is required. Explicit empty is a distinct contract, permitting no extra
evidence while still requiring exact occurrence authority. New declarations on
unguarded or kernel-control bindings are invalid.

The installed binding establishes the evidence floor. Branch-local definitions
wire exact sources and provenance but cannot weaken it. A definition may add a
permitted approval obligation; it remains mandatory even with an empty installed
floor. Resolve zero or one compatible approval source, using existing composed
review/evidence mechanisms for multiple subjects. Missing, conflicting, malformed,
unavailable, foreign, stale, or revoked required evidence never becomes permission.

## Native runtime progression

Issue common operation authority only after validating native definition/instance
membership, current occurrence, exact operation and input provenance, subject,
branch/runtime affinity, liveness/deadline, and observed currentness facts. Add the
real approval authority basis when the installed contract or authored connection
requires it. Explicitly distinguish satisfied-without-approval from unavailable
approval; neither an optional-value default nor a caller choice can decide this.

The public requirement and candidate binding carry the installed evidence contract
so reconstructed metadata cannot independently authorize another typed binding.
Fresh operation permission and currentness are checked before the handler and at
the existing commit boundary. Independent freshly issued slots still compete on
native progress and idempotency, not just a shared in-process single-use slot.

Preserve atomic local mutation/settlement and external-owner custody. Recovery
repairs the original performed occurrence; it never authorizes another effect.
Keep stable logical keys, intent-drift denial, typed stale/cancelled/deferred/
indeterminate outcomes and retained recovery paths. A stale lifecycle command is
reconciled, not automatically retried.

Business readiness remains supplier-owned. A passed historical condition does
not assert eternal operand truth; preconditions required at execution use existing
tracked decision/invariant/requirement contracts. Do not invent universal condition
reevaluation or substitute a cached readiness flag for a required current check.

## Identity, persistence, and compatibility

Carry the declaration through mutation descriptors and schema meaning, canonical
program action records and semantic comparison, workflow references/definitions,
installed support identity, adoption coverage, persisted compilation, and
reconstruction. Otherwise-identical omitted/empty/approval declarations must have
distinct canonical program meaning and observable operation changes.

Preserve all legacy absent canonical-meaning encodings and pinned identities. New portable mutation
records use a distinct archive member tag; new workflow operation records use a
distinct node tag and a named optional requirements field. Old readers reject new
unsupported tags instead of interpreting them as legacy operations. New readers
retain the old required-approval interpretation. Unknown/malformed new encodings
deny before effects. Do not turn schema versions into build counters.

WQWD draft transport version 4 retains the exact optional operation evidence
contract and compares it with installed meaning during reauthoring. Versions 1–3
remain readable as absent evidence contracts; they cannot silently acquire either
new explicit declaration. Draft bytes remain compatibility expectations, never
execution authority. This transport version does not change legacy definition
content identities.

Occurrence-only execution requires a new installed supplier/program contract;
editing an existing release or dropping a graph edge cannot enable it. Checkpoint
reconstruction remains distinct from future Store process-crash durability.

## Destination and dependency direction

```text
worth-query-declaration/src/
  application_operation/binding/workflow_requirements.rs      [new: pure evidence contract]
  application_operation/binding/{intent,descriptor}.rs        [extend: one binding source]
  application_program/{action,manifest}.rs                    [extend: canonical program meaning]
  application_program/evolution/semantic_description.rs       [extend: semantic comparison]
  application_program/workflow/                              [extend: references/validation/identity]
worth-query-installation/src/
  application_operation/binding/catalog/workflow_requirements.rs [new: installed admissibility]
  application_program/workflow/                              [extend: floor correspondence/coverage]
worth-query-execution/src/domain_computation/primary_graph/
  workflow/{schema,definition}/                              [extend: facts/reconstruction/adoption]
  application_attempt/workflow_transition_program/operation/
    evidence.rs                                             [new: additional-evidence resolution]
    authority_binding.rs                                    [extend: exact candidate binding]
  application_attempt/workflow_transition_program/publication/operation.rs [extend: authority carrier]
  application_attempt/{read_set,provider_execution}/          [extend: commit evidence handoff]
worth-query-publication/src/application_entry/workflow/       [preserve funnel; exact checks]
worth-query-package-archive/src/record/application_schema/    [extend: versioned descriptive codec]
worth-query-certification/tests/application_graph/
  workflow_execution.rs                                    [new: native execution cases]
  workflow_execution/review.rs                             [new: additional review obligations]
```

Declaration owns meaning, Installation binds it, Execution issues/consumes/settles
authority, and Publication exposes the existing application funnel. Approval
revalidation remains with its authorization owner. Only declaration and host
audience facades serve applications. Future suppliers add their own contracts
through those facades, not runtime authorization categories or copied executors.

## Delivery and continuing documentation

First establish declaration/installation/canonical compatibility with focused
evidence. Then complete persisted native execution and its public-entry adversarial
cases. Finally close documentation, compiling examples, affected lint/type checks,
and required repository enforcement before local master integration. Every slice
is reviewed and committed locally; an intermediate slice does not claim capability
completion.

`docs/foundations/workflows.md` explains supplier evidence, the single execution
funnel, legacy interpretation, direct/stale denial, and local/external settlement.
`docs/AI_README.md` explains ownership without prescribing invented consumer
permits. The existing `authored_workflow` example must demonstrate information
recording and signed review through that same funnel. The Query roadmap links
this correction and its real handoff; it is not a second status ledger.

Use focused owner/consumer tests and existing guards for unchanged boundaries.
Run formatting, affected compilation/lint, line limits, and facade/dependency
enforcement. Commands have a 290-second bound; report cold builds separately and
never label a timeout or filtered-out case a pass. Tests beyond five minutes need
redesign; tests beyond ten minutes are prohibited. Ordinary work stays local to
the selected operation's requirements, not unrelated instances or full history.

The upstream handoff is this supported public join. Consumer adoption must still
prove its own business transition, signatures, and release behavior; this platform
correction does not implement NCR lifecycle rebuilding, automatic dispatch, new
effect protocols, Store durability, or cross-version impact analysis.
