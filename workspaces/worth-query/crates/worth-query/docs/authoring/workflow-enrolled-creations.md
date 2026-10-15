# Workflow-Enrolled Creations

## What This Feature Is

A workflow-enrolled creation publishes a new application subject and its fresh
native workflow instance in one guarded mutation. The creation body declares
which exact Create output is enrolled. Query resolves that candidate-owned
output, starts the instance and commits both together; the host does not create
an object and then separately attempt enrollment.

## Why You Use It

Use this when a newly created request, issue or document must never be visible
without its workflow run. Use ordinary `prepare_workflow_instance_start` when
the subject already exists and independent enrollment is intentional.

## Stable Entry Points

Declare with `worth_query_decl::facade::application_schema` and install with
`worth_query_host::facade::domain`:

- `creation.enroll_in_workflow::<Spec, Binding, Role>()` also declares the body's
  fixed Create output. Do not separately declare the same output again.
- `WorthQueryApplicationWorkflowSpecInstallation::birth_operation::<Binding, Role>()`
  binds the installed body, workflow spec, output and binding-associated capability.

Execute through `worth_query_host::facade::application_entry`:

- `request.prepare_workflow_birth::<Spec, Role>(&workflow, definition)` returns
  `RequiredWorkflowBirth`; it writes nothing.
- `request.for_workflow_birth::<Spec, Role>(&workflow, &required)` binds the exact
  input, logical key, workflow, role and branch.
- The ordinary `execute_capability_in_program` or prepared-program commit funnel
  then executes the installed creation body and native enrollment together.
- `required.instance_in(&receipt)` projects the actual `PublishedWorkflowInstanceRef`.

The creator must be a program-required, non-control mutation with its own
capability. It does not set `REQUIRES_WORKFLOW_AUTHORITY`: no current occurrence
of the unborn subject exists to authorize creation.

## Core Mental Model

The request scope is an existing container, such as a company. Its admitted
capability authorizes this creator operation on that container. The body's
created output is a different subject, identified by the candidate's actual
Create reference, not a predicted entity ID or the container ID.

Enrollment metadata is authored meaning, not permission. A published definition
reference is also not permission. Query admits the exact creator capability,
selects the branch's current definition and retains the definition, capacity and
deadline observations needed for publication.

`RequiredWorkflowBirth` holds a shared one-shot native authority slot. Cloning
the requirement does not mint another slot. A receipt only describes a performed
commit; projecting an instance from it grants no execution authority.

## How It Executes

Preparation validates the installed membership and admitted input, reconstructs
the current definition and observes its lineage capacity and deadline. Execution
freshly authorizes the creator, resolves idempotency before consuming the slot,
and executes the native body. The native owner selects the actual created output,
merges its observations without contradicting handler facts, and stages the
instance plus subject relation alongside the business effects.

The normal compare-and-publish boundary checks the complete candidate. A losing
candidate publishes neither half. Ordinary and advanced publication cannot
commit an enrollment-bearing body without native birth proof; program migration
explicitly refuses enrollment rather than dropping it.

## Small Example

Assume the ordinary typed body, fixed exactly-one Create role and mutation binding
already exist:

```rust,ignore
let body = creation
    .enroll_in_workflow::<RequestWorkflow, CreateRequestBinding, CreatedRequest>()?
    .finish()?;

// During vocabulary installation, beside the ordinary control capabilities:
let installation = installation
    .birth_operation::<CreateRequestBinding, CreatedRequest>()?;
```

This is not an ordinary workflow operation node. The vocabulary rejects
`.operation::<CreateRequestBinding>()` for an enrollment-bearing creator.

## Real Example

A purchase-request creator can initialize its code and admitted requester and
link company membership before attaching enrollment. These are existing typed
body operations, not a second host implementation:

```rust,ignore
let body = CreateRequestOperation::reference().body()
    .in_scope(Company::reference())?
    .create_in_scope(PurchaseRequest::reference(), "request")?
    .initialize_input(RequestCode::reference(), "code")?
    .initialize_from_principal_identity::<CreateRequestBinding, _, _, _, _, _>(
        RequestedBy::reference(),
    )?
    .link_from_scope(CompanyRequests::reference(), "company-request")?
    .enroll_in_workflow::<RequestWorkflow, CreateRequestBinding, CreatedRequest>()?
    .finish()?;
```

After installing that body, its program action, own company-scoped capability
and vocabulary membership, use the same intent and logical key at both calls:

```rust,ignore
let birth = runtime.request(&principal, &scope)
    .on_branch(definition.branch())
    .mutate(intent.clone()).without_source().idempotency(&key)
    .prepare_workflow_birth::<_, CreatedRequest>(&workflow, definition)?;

let outcome = runtime.request(&principal, &scope)
    .on_branch(birth.branch())
    .mutate(intent).without_source().idempotency(&key)
    .for_workflow_birth::<_, CreatedRequest>(&workflow, &birth)?
    .execute_capability_in_program(&program, allocation)?;

let instance_observation = outcome.receipt().map(|receipt| birth.instance_in(receipt));
// Keep the complete outcome even if descriptive projection is unavailable.
(outcome, instance_observation)
```

An exact retry may return `AlreadyCommitted` even after the instance advances or
the original slot is consumed. An independently prepared same-key request also
resolves that original publication. Changed intent is not a retry. Capability
revocation or changed definition observations refuse a fresh publication.

## How It Relates To Other Features

Use normal proposals, assessments, approvals and guarded operations after birth;
each freshly admits its own authority. Birth does not sign the new subject or
impose approval evidence on information gathering. It starts a fresh run, not a
migration successor inheriting another instance's spent budgets or custody.

Existing v2 coordinate-bound instance records retain their prior protocol. New
joint births use a versioned native subject relation resolved in the same commit.
Enrollment selects body archive v17; un-enrolled bodies keep their earlier
encoding. The workflow, binding and role change schema and program identity.

## Inspection And Debugging

Observe the returned mutation outcome and use normal authorized application
queries for the subject. `instance_in` requires a receipt for this exact birth
and exactly one recorded native instance. Projection failure does not undo a
commit: retain its outcome and receipt. Do not flatten `PreviouslyCommitted`,
unpublished, unresolved or deferred native outcomes into ordinary denial or
permission to execute again.

## Anti-Patterns

Do not predict a future ID, enroll the container as a placeholder, seed a subject,
copy a receipt into a permit, or publish creation and start as two independent
mutations while claiming atomic enrollment. Do not use an ended instance's
current-operation authority to create a new one. Never relax a capability policy
to make an invalid creator declaration install.

## Current Limits

The supported join is one local unsigned body creation and one fresh instance
under the current definition, with a fixed exactly-one Create output. Signed or
external-effect creation, workflow-approval-bearing birth, arbitrary resume targets,
multiple created subjects and enrollment through program migration are refused.

This is not a complete Reopen command. Closed-source pin admission, a fresh
approval round, preserved historical signature validity and linkage to prior
rounds remain separate domain and native joins. A retained requirement is live
authority, not a restart-persistable permit. No receipt-free historical instance
recovery or durable Store restart guarantee is added by this surface.

## Related Docs

- [Workflows](../foundations/workflows.md)
- [Executable Scope-Bound Bodies](./scope-field-operation-bodies.md)
- [Signed Operation Creations](./signed-operation-creations.md)
- [Application Authorization](../capabilities/application-authorization-and-emergency-elevation.md)
- [Aftermath and Recovery](../execution/application-aftermath-and-recovery.md)
