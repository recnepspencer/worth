# Installed Operation Lineage And Promotion

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Installed operation lineage records how an effect preserved, generated,
replaced, split, merged, retired, or failed to establish identity. Query binds
that result to the exact workflow run, stage, and mutation receipt. Persistent
naming and sparse graph promotion consume this bound evidence instead of
guessing continuity from labels, geometry, coordinates, or array position.

## Why You Use It

- Keep identity continuity attached to the mutation that established it.
- Preserve split, merge, ambiguity, retirement, and continuity-break meaning.
- Admit persistent naming only when an executed naming mutation targets the
  same authoritative lineage subject.
- Give a derived subelement graph identity only when a durable reference asks
  for it and the exact carrying publication contains it.

## Stable Entry Points

Installed meaning and execution use:

- `WorthQueryOperationLineageContract::{NotRequired, Preserve, Evolve}`
- `WorthQueryOperationPromotionContract::OnDurableReference`
- `WorthQueryWorkflowStageExecutionContext::execute_identity_evolution(...)`
- `WorthQueryWorkflowStageExecutionContext::execute_identity_correspondence(...)`
- `WorthQueryWorkflowStageMaterial::with_lineage_outcomes(...)`
- `WorthQueryCompletedWorkflowTrace::lineage_report()`
- `WorthQueryTraceLineageEvidence`

Persistent naming uses:

- `WorthQueryPersistentNameIntent`
- `trace.admit_persistent_name(...)`

Sparse promotion uses:

- `WorthQueryDurableReferenceIntent`
- `published.admit_promotion_on_reference(...)`
- `WorthQueryPromotedGraphIdentity`
- `WorthGraphDurableReferenceKind`

## Core Mental Model

The domain executor does not construct lineage from identities. It first
executes a Query-owned mutation and receives `WorthQueryWorkflowEffectEvidence`.
The stage context then runs Query's existing identity-evolution engine against
that exact effect. The resulting outcome is already bound to the operation,
run, stage, effect receipt, basis, and authoritative continuity evidence.

At workflow completion, Query validates the installed lineage contract and
materializes one trace lineage report. Foundational lineage and provenance
attachments describe that result across boundaries, but the stronger Query
trace evidence remains the operational authority.

Schema Graph owns the closed promotion grammar. Query lowers that grammar and
admits the operational promoted identity through Foundational authority only
after publication and lineage checks pass.

## How It Executes

```text
execute a declared mutation through the stage context
  -> derive identity evolution from the exact mutation receipt
  -> attach the outcome to stage material
  -> validate run, stage, effect, basis, and lineage contract
  -> bind one trace lineage report
  -> optional: bind an executed persistent naming mutation
  -> optional: publish the result and admit one durable-reference promotion
```

No-change, deferred, suppressed, and reverted-clean conditional stages cannot
claim fresh lineage. Ambiguous, advisory, retired, and broken outcomes remain
inspectable but cannot authorize persistent naming or promotion.

## Small Example

An executor derives lineage from the effect it just caused:

```rust
let effect = context.execute_mutation(command, workspace)?;
let lineage = context.execute_identity_evolution(&effect)?;

Ok(
    domain::WorthQueryWorkflowStageMaterial::new(output)
        .with_lineage_outcomes(vec![lineage])
        .with_result_state(domain::WorthQueryOperationResultState::Ready),
)
```

Passing raw entity identities to `with_lineage_outcomes` is impossible. The
vector can contain only outcomes returned by Query's admitted identity-
evolution path.

## Real Example

After publication, a durable reference can request sparse promotion:

```rust
let intent = domain::WorthQueryDurableReferenceIntent::new(
    domain::WorthGraphDurableReferenceKind::PersistentSelection,
    domain::WorthQueryOperationProjectionRole::new("vertex")?,
    0,
    0,
);

let capability = published
    .admit_promotion_on_reference(intent)
    .unwrap();

let identity = capability.promoted_graph_identity();
assert_eq!(
    identity.carrying_artifact_identity().as_str(),
    capability.publication_identity(),
);
```

Admission proves that the installed operation declared promotion, the
publication role matches, the selected lineage evidence belongs to the
publication stage, continuity is authoritative, and the exact entity is
present in the carrying publication.

Plural lineage does not imply a positional authority-to-entity mapping. Until
an owner supplies exact correspondence for each successor, split or merge
promotion fails with `LineageSubjectEntityBindingUnavailable`.

## How It Relates To Other Features

- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
  compares the same trace-bound lineage under a fresh execution identity.
- [Authoritative Mutation Evidence](../capabilities/authoritative-mutation-evidence.md)
  supplies the mutation, continuity, and naming receipts lineage consumes.
- [Projection Consumption](../capabilities/projection-consumption.md) is the
  ordinary downstream fact path; it does not itself promote identities.

## Inspection And Debugging

Inspect:

- `trace.lineage_report().identity()`
- each evidence row's stage receipt and effect receipt identities
- the typed `InstalledIdentityEvolutionKind`
- engine artifact, continuity evidence, and Foundational lineage attachment
- lineage counters, including indexed trace stages and effect receipts
- promotion counters and exact denial kind

The trace report is safe to compare and inspect. Copying its fields, identity,
or Foundational attachment cannot satisfy persistent naming or promotion.

## Anti-Patterns

- Constructing continuity from coordinates, topology summaries, labels, or
  debug output.
- Pairing successor identities with entity vectors by ordinal.
- Treating advisory correspondence as authoritative continuity.
- Running a second identity-evolution engine inside an executor.
- Promoting every projected subelement eagerly.
- Treating Schema Graph's pure `GraphPromotionIdentityBasis` as operational
  authority.

## Current Limits

- Plural split and merge outcomes remain plural. Promotion requires an exact
  owner-proved subject-to-entity correspondence and otherwise fails closed.
- Persistent naming policy remains domain-owned. Query verifies an already
  executed typed naming mutation; it does not author product naming grammar.
- Promotion mints sparse Query-admitted graph identity. It does not copy dense
  geometry or artifact payload into graph storage.
- Foundational attachments are descriptive transport and readmission evidence,
  not replacements for the trace-bound Query outcome.

## Related Docs

- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
- [Lineage And Correspondence](../capabilities/lineage-and-correspondence.md)
- [Authoritative Mutation Evidence](../capabilities/authoritative-mutation-evidence.md)
