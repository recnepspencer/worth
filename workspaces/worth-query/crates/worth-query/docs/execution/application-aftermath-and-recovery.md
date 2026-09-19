# Application Aftermath, External Effects, And Recovery

## What This Feature Is

This feature lets an application declare what can happen after a mutation
commits, send one bounded typed effect to an external service, and recover when
durability, Query publication, or delivery is uncertain. Application code
declares the meaning. Query owns execution and its typed recovery surfaces.
Relational owns branch movement, commit history, and durable settlement. The
external service alone decides whether its effect completed.

Runtime World owns the canonical composite product publication. `Performed`
means it installed the exact product occurrence once. `ProductUnpublished`
means one or more component effects may already exist while the product
reference did not advance; its carrier preserves the only lawful settlement or
cleanup path. A late cancellation cannot erase a performed occurrence or turn
it into a pre-effect denial.

External dispatch admission is tied to that original performed occurrence.
Same-head losers, inherited idempotency rows, foreign branches, and descendants
cannot claim its outbox entry. Retry consumes retained aftermath and revalidates
the owning runtime; it does not rerun publication. A fresh composite aftermath
write begins from a newly selected and admitted product occurrence.

## Why You Use It

- Notify an external service without separating the local mutation from the
  dispatch record that makes delivery recoverable.
- Inspect or resolve an uncertain commit without accepting a caller-written
  recovery slot, posture, or boolean as authority.
- Complete durability and Query index publication when the Relational branch
  already moved but settlement did not finish.
- Retain an exact bounded pre-mutation field value when an installed contract
  requires it, while failing the commit if that exact truth is unavailable.
- Show callers a closed aftermath or delivery posture without exposing runtime
  handles, outbox bytes, protected decision facts, or execution internals.

## Stable Entry Points

Declaration consumers use `worth_query_decl::facade`:

- `application_schema::ApplicationExternalEffectPayload`
- `application_schema::ApplicationExternalEffectProtocol`
- `application_schema::WorthQueryExternalEffectCorrelationFamily`
- `application_schema::ApplicationOperationDefinitionBuilder::external_effect`
- `application_schema::ApplicationOperationDefinitionBuilder::no_external_effect`
- `application_aftermath::DeclaredApplicationAftermathContract`
- `application_aftermath::DeclaredPreImageDemand`
- `application_schema::ApplicationOperationDefinitionBuilder::aftermath`
- `application_schema::ApplicationOperationDefinitionBuilder::no_aftermath`

Hosts use `worth_query_host::facade`:

- `primary_graph` for sealed commit receipts and recovery progression
- `primary_graph::WorthQueryApplicationSettlementDeferred`
- `primary_graph::WorthQueryApplicationSettlementNextAction`
- `primary_graph::WorthQueryPrimaryGraphApplicationRuntime::recover_deferred_application_settlement`
- `publication::domain_computation::publish_application_commit`
- `publication::application_aftermath::publish_application_aftermath`
- `publication::application_aftermath::publish_recovery_support`

Application hosts should normally wrap the generic host surface in domain-named
operations, as Bank does with `BankCommitReceipt::aftermath()` and its recovery
methods.

The executable
[ordinary product workflow](../../../worth-query-certification/examples/ordinary_product_workflow.rs)
shows successful publication and exhaustively matches every typed commit
terminal. Its `ProductUnpublished` arm performs bounded catalog inspection,
continues owed owner settlement, and consumes the recovery carrier through
public cleanup. Its `SettlementDeferred` arm invokes the exact public repair
route, while cancellation and `NoEffect` remain explicit no-publication stops.
The executable does not manufacture an owner failure to enter those arms.

The `provisional_aftermath` facade contains the current undo/redo experiment.
It is not part of the stable feature described here.

## Core Mental Model

There are four independent owners:

1. Your declaration names the effect protocol and the operation's aftermath
   contract.
2. Query installs that declaration, executes the operation, co-commits the
   outbox data, and owns live recovery handles.
3. Relational assigns commit identity, parentage, branch head, and canonical
   history. Query does not keep a second lineage or history store.
4. The external service decides whether the external effect completed. Silence,
   a timeout, or a lost response is never treated as completion.

There are two different recovery boundaries:

- **Publication-settlement recovery** starts from
  `WorthQueryApplicationSettlementDeferred`. The mutation already crossed the
  Relational branch cutover, but durability acknowledgement or Query's derived
  index/head publication failed. Recovery completes that exact publication.
- **Application-aftermath recovery** starts from a sealed successful commit
  receipt and a `WorthQueryRecoveryHandle`. It inspects or resolves external
  effect and correction posture after commit.

Neither carrier is a serialized token. The application settlement carrier
hides Relational's raw `DeferredPublicationSettlement`. The aftermath handle is
a move-only runtime capability. An opaque wire identity can be logged or
transported, but must be re-admitted by the owning runtime before it can affect
anything.

Published aftermath is deliberately weaker. It describes accepted posture,
external observation, or recovery support, but cannot authorize a transition.

Every operation definition makes two explicit choices. It selects either one
external-effect contract or `no_external_effect()`, and either one aftermath
contract or `no_aftermath()`. Method presence is not a default. The builder
cannot finish until both choices are made.

An aftermath contract declares two independent facts:

- **correction authority** — whether Query can correct the operation alone,
  must coordinate with an external owner, or cannot correct it;
- **correction mechanism** — a recorded inverse, compensation, reconciliation,
  or no corrective mechanism.

Installation validates those facts and derives one closed published posture:
`Reversible`, `Compensatable`, `Reconcilable`, or `Irreversible`. An operation
with an escaping external effect cannot be published as reversible.

## How It Executes

1. The schema declares one typed emission and exactly chooses an external-effect
   slot and an aftermath slot for the operation. Either choice may explicitly
   be `none`.
2. The payload implementation supplies a stable version-free protocol family,
   an exact positive produced version, a maximum byte count, and the exact
   bytes to send.
3. Query installs the operation and its one aftermath contract.
4. During execution, Query derives the payload from the admitted typed emission.
   Callers cannot replace the effect name, protocol family, exact version,
   bound, or bytes.
5. Relational compares the prepared candidate with the exact branch reference,
   performs canonical branch movement, and settles durability.
6. If durability or Query index publication fails after movement, Query returns
   `SettlementDeferred`. The caller repairs that carrier; it does not retry the
   operation.
7. Query reads the committed outbox row through a fresh owner-sealed
   observation and dispatches it to the external owner.
8. The returned observation becomes `Completed`, `Acknowledged`, or
   `Unresolved(failure)`. Missing evidence never advances the posture.
9. If aftermath recovery is needed, the runtime opens one exact-handle
   lifecycle from the commit receipt. Inspection requires fresh disclosure
   authority; effectful progression requires fresh effect authority.
10. Publication projects the sealed commit or admitted inspection into closed
   consumer-facing values.

The local commit and the external consequence are different facts. Query may
know that the outbox row committed while the external result remains pending or
unresolved. Acknowledgement is not completion. Timeout, disconnect, and lost
response never let Query guess what the external owner did.

When an installed contract demands a pre-image, Query reads Relational's opaque
validated mutation footprint before commit. The exact record, aspect, and field
must be mutated by that attempt. A same-named field on another record, another
aspect, or merely-read field cannot satisfy the demand.

## Small Example

Define a stable external payload. The protocol family is application-owned,
version-free, and not a Rust module path; its exact produced version is a
separate typed value.

```rust
use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion,
};
use worth_query_decl::facade::application_schema::{
    ApplicationEffectPayload, ApplicationExternalEffectPayload,
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};

struct InvoiceReady {
    invoice_id: u64,
}

impl ApplicationEffectPayload for InvoiceReady {
    fn retained_bytes(&self) -> u64 {
        8
    }
}

impl ApplicationExternalEffectPayload for InvoiceReady {
    const PROTOCOL: ApplicationExternalEffectProtocol =
        ApplicationExternalEffectProtocol::new(
            BoundaryProtocolIdentity::new("billing.invoice-ready"),
            BoundaryProtocolVersion::new(1),
        );
    const MAX_EXTERNAL_BYTES: u64 = 8;

    fn external_effect_bytes(&self) -> Vec<u8> {
        self.invoice_id.to_be_bytes().to_vec()
    }
}
```

The schema must also declare the typed emission and bind that emission to an
external-effect correlation family with the operation definition's
`external_effect` transition. A payload implementation alone does not create a
dispatch lane.

Assume `InvoiceReadyEffect` is the schema's typed effect reference and
`declared_aftermath` is its installed declaration contract. The operation
builder then makes both static choices before `finish()` becomes available:

```rust
let correlation_family =
    WorthQueryExternalEffectCorrelationFamily::new("billing-rail")
        .expect("the application-owned correlation family is an atomic identity");

let definition = operation
    .definition()
    .external_effect(InvoiceReadyEffect::reference(), correlation_family)
    .aftermath(declared_aftermath)
    .finish();
```

An ordinary operation with neither feature states that just as explicitly:

```rust
let definition = operation
    .definition()
    .no_external_effect()
    .no_aftermath()
    .finish();
```

Hosts may inspect the exact installed contract through the read-only domain
facade:

```rust
let contracts = installed_operation.contracts();
let aftermath = contracts
    .aftermath()
    .expect("this operation declares aftermath meaning");

inspect_authority(aftermath.authority());
inspect_recovery(aftermath.recovery());
inspect_canonical(aftermath.canonical());

let procedure_slot = aftermath
    .reconciliation()
    .map(|procedure| procedure.procedure_slot());
let correlation_family = aftermath
    .external_effect()
    .correlation_family()
    .map(|family| family.as_str());

assert_eq!(
    aftermath.external_effect().correlation_family(),
    contracts.external_effect().correlation_family(),
);
```

These values retain correction authority, recovery posture, the exact
reconciliation procedure, correlation family, and canonical evidence. Reading
them grants no correction, recovery, dispatch, or external completion
authority.

## Real Example

Bank's death-notification operation declares one emission, one stable protocol
family with an exact version, and one rail:

```rust
schema
    .operation_emit(operation, EstateDeathNotificationEffect::reference())
```

The operation definition binds the escaping lane:

```rust
use worth_query_decl::facade::application_schema::WorthQueryExternalEffectCorrelationFamily;

operation
    .definition()
    .external_effect(
        EstateDeathNotificationEffect::reference(),
        WorthQueryExternalEffectCorrelationFamily::new(ESTATE_DEATH_NOTICE_RAIL)
            .expect("the Bank rail name is a stable atomic identity"),
    )
    .aftermath(declared_aftermath)
    .finish()
```

The payload encodes the exact estate, notice, and subject as three big-endian
`u64` values under family `bank.estate.death-notification`, version `1`. After a
real commit, an application consumer reads only the published posture:

```rust
use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedExternalEffectPosture,
    WorthQueryPublishedExternalEffectFailure,
};

match receipt.aftermath().external_effect() {
    WorthQueryPublishedExternalEffectPosture::Completed => {
        // The rail independently decoded and completed this effect.
    }
    WorthQueryPublishedExternalEffectPosture::Acknowledged => {
        // Accepted by the rail, but completion is not established.
    }
    WorthQueryPublishedExternalEffectPosture::Unresolved(
        WorthQueryPublishedExternalEffectFailure::LostResponse,
    ) => {
        // Open recovery through the host's receipt-bound recovery API.
    }
    _ => {}
}
```

The Bank rail is a separate process. It independently checks the effect name,
protocol family, exact version, declared bound, and bytes before admitting the
notice to its ledger.
A lost response can therefore leave Query unresolved even when the rail has
completed the effect; safe retry relies on the rail's idempotent correlation,
not on Query guessing what happened.

## Publication Settlement Recovery

Match the application commit outcome before entering aftermath handling:

```rust
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationSettlementNextAction,
};

match outcome {
    WorthQueryApplicationCommitOutcome::Committed(receipt)
    | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
        publish(receipt);
    }
    WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
        assert_eq!(
            deferred.next_action(),
            WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement,
        );
        let receipt = application_runtime
            .recover_deferred_application_settlement(&deferred)?;
        publish_repaired(receipt);
    }
    other => handle_unperformed_or_indeterminate(other),
}
```

Recovery runs under the application commit serialization boundary. It repairs
the exact Relational durability route, refreshes Query's primary indexes,
observes the current branch basis, proves the performed commit is still in the
current ancestry, binds the current Bridge head, and validates the exact
idempotency binding, readmitting it when required. A later legal application
head is preserved rather than rewound. Repeated repair returns the same commit
receipt.

This is not the `WorthQueryRecoveryHandle` flow below. No external effect is
redispatched and the application mutation is not executed again.

## Recovery Call Sequence

Aftermath recovery re-enters the ordinary admission path. Assume the host has re-admitted
the same operation under current truth and has obtained both
`admitted_operation` and its disclosure-admitted `capability_access`. The public
host sequence is:

```rust
use worth_query_host::facade::primary_graph::{
    inspect_recovery_handle, resolve_recovery_handle,
};
use worth_query_host::facade::publication::application_aftermath::{
    publish_recovery_support,
};

let handle = runtime.mint_recovery_handle(&commit_receipt)?;

let disclosure =
    runtime.admit_recovery_inspection_disclosure(&capability_access)?;
let inspect_authority = runtime.admit_recovery_inspect_authority(
    &handle,
    &admitted_operation,
    &disclosure,
)?;
let inspection = inspect_recovery_handle(&handle, &inspect_authority)?;
let published_support = publish_recovery_support(&inspection);

let effect_authority =
    runtime.admit_recovery_effect_authority(&handle, &admitted_operation)?;
let idempotency = runtime.resolve_admitted_application_idempotency(
    &admitted_operation,
    handle.binding().idempotency(),
)?;
let resolution =
    resolve_recovery_handle(handle, &effect_authority, idempotency)?;
```

Inspection borrows the live handle and returns descriptive, publishable state.
Resolution consumes the handle after a fresh owner-issued effect authority and
an admitted idempotency read agree with its exact binding. A domain host should
normally wrap this generic sequence in domain-named methods, as Bank does.

## How It Relates To Other Features

- Pair external effects with ordinary Query mutation and idempotency. The outbox
  row and mutation share one Relational commit.
- Use application settlement recovery when publication already performed but
  settlement or Query publication failed. Use aftermath recovery for an
  unresolved external observation or correction posture. Do not use replay:
  reconstruction and historical replay remain certification-only.
- Use publication for consumer descriptions. Use execution recovery authority
  only inside the host runtime that owns the handle.
- Relational remains the authority for commit history, entity lineage, branch
  head, and ancestry. Runtime Bridge may transport an already-admitted portable
  description, but it does not decide any of those facts.
- Exact pre-image retention is stable infrastructure. What an eventual undo or
  redo product may lawfully do with retained truth is still deferred.
- `worth-proof` supplies generic progression and proof-bearing carriers. Query
  operations still require the exact Query-owned authority values returned by
  the owning workflow; a caller-defined generic marker opens no recovery door.

## Inspection And Debugging

For a committed operation, inspect:

- `WorthQueryApplicationSettlementDeferred::stage()`, `detail()`, counters,
  optional Query publication failure detail, and `next_action()` before a
  successful commit receipt exists;

- the installed operation's `contracts().aftermath()` for correction
  `authority()`, `recovery()`, exact `reconciliation()`, canonical evidence,
  and the external effect's typed `correlation_family()`;
- the host receipt's commit identity and ordinary canonical-work counters;
- `aftermath().posture()` for the installed accepted posture;
- `aftermath().external_effect()` for external observation state;
- the host's committed-outbox observation when diagnosing transport ownership;
- `publish_recovery_support(&inspection)` after disclosure-admitted recovery
  inspection.

`Unresolved` includes typed failures such as timeout, disconnect, lost response,
duplicate acknowledgement, payload rejection, unsupported protocol version, or
an unknown provider outcome. Unsupported versions retain the exact produced
version and distinguish `PredatesWindow`, `ExceedsWindow`, and `Retired`.
These values explain what Query observed; they do not overwrite the external
owner's ledger.

## Anti-Patterns

- Do not derive a protocol family or version from `std::any::type_name`.
- Do not retain or parse a raw correlation-family string beside the typed
  `WorthQueryExternalEffectCorrelationFamily`.
- Do not accept payload bytes, effect names, completion flags, or recovery slots
  from the caller after operation admission.
- Do not treat acknowledgement, silence, timeout, or response loss as completion.
- Do not construct publication from copied enums or raw receipt fields.
- Do not serialize a recovery handle and treat the bytes as live authority.
- Do not extract, serialize, or accept a raw Relational settlement capability.
- Do not retry the application operation after `SettlementDeferred`; repair the
  typed application carrier.
- Do not use an aftermath recovery handle to repair publication settlement, or
  use a settlement carrier to redispatch an external effect.
- Do not treat `WorthQueryRecoveryBrief` or ordinary stop-remediation guidance
  as an application-aftermath recovery handle.
- Do not add Query-local lineage, parent selection, branch heads, or publication
  order. Those are Relational responsibilities.
- Do not use `provisional_aftermath` as a stable product contract.

## Current Limits

- Application settlement carriers and aftermath recovery handles are
  runtime-local and not restart-durable. Published aftermath recovery support
  reports when a Store capability is still required.
- Reconciliation and compensation currently produce exact owner-bound admission
  values. No accepted Query surface executes those corrective effects yet.
- The accepted external-effect lane carries one declared typed effect per
  operation contract.
- Durable cross-runtime recovery, branch-aware reversal, merge interaction,
  rebase, and restart-stable history navigation belong to the cross-runtime
  roadmap.
- Undo and redo behavior, eligibility, occurrence meaning, divergence policy,
  and public DX await the Query Undo/Redo Semantics milestone.

## Related Docs

- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Provider Sessions And Decision Read-Sets](../domain-capabilities/provider-sessions-and-decision-read-sets.md)
- [Authoritative Mutation Evidence](../capabilities/authoritative-mutation-evidence.md)
- [Query Operating Modes](../foundations/query-operating-modes.md)
- [Typed Stops And Remediation Guidance](../domain-capabilities/typed-stops-and-remediation-guidance.md)
- [worth-proof Authority And Workflow Contracts](../../../../../../crates/worth-proof/docs/features/authority-and-workflow-contracts.md)
