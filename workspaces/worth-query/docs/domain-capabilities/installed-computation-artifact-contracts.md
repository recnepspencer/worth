# Installed Computation Artifact Contracts

## What This Feature Is

Installed computation artifact contracts give domain-produced working data one
portable, validated meaning before a runtime handles it. Use them when an
operation produces candidate sets, solver state, compiled plans, checkpoints,
or other artifacts whose identity, ownership, evidence, cost, and disclosure
rules must remain consistent across providers.

The contract describes the artifact. It does not contain callbacks, own a
runtime value, or authorize execution.

## Why You Use It

- Prevent two providers from using the same artifact name for different
  semantics.
- Declare whether artifacts may be borrowed, moved, retained, transformed, or
  reconstructed.
- Make reproducibility, comparison, search, convergence, and loss claims
  explicit.
- Give counters declared units and aggregation rules.
- Govern decision evidence without turning explanatory records into authority.

## Stable Entry Points

Import these through `worth_query_host::facade::domain`:

- `WorthQueryPortableArtifactContract::declare::<Family>(...)`
- `WorthQueryPortableArtifactContractBuilder`
- `WorthQueryInstalledArtifactContractAuthority`
- `WorthQueryStructuralCounterContract`
- `WorthQueryStructuralCounterSchema`
- `WorthQueryDecisionRecordContract`
- `WorthQueryDecisionSchema`
- `WorthQueryArtifactGovernanceContract`
- `WorthQueryArtifactCompatibilityContract`

The installed authority is minted during package installation. Callers cannot
construct it from a portable contract or matching identity.

## Core Mental Model

A portable artifact contract answers five questions:

1. **What is this artifact?** Family, schema and protocol versions, semantic
   content identity, and occurrence identity.
2. **Who controls it?** Payload owner, producer and consumer roles, lifecycle,
   and carriage rules.
3. **What claims may be made about it?** Evidence, reproducibility, comparison,
   candidate search, convergence, and transformation loss.
4. **How may a runtime read it?** Declared native layouts, bulk paths, and
   scalar-fallback posture.
5. **How is it explained and governed?** Structural counters, decision schemas,
   classification, redaction, retention, compatibility, and retirement.

All semantic dimensions participate in canonical identity. Declaration order
does not. A changed retention rule, unit, occurrence policy, comparator, or
loss posture is therefore contract drift, not harmless metadata.

Structural counters report work; they never authorize a later phase. Required
foundation rows cannot be shadowed by optional domain rows, aggregation must be
acyclic, and source and destination units must be compatible.

Decision records explain choices such as pruning, ranking, or candidate
selection. Their schema fixes the decision identity, payload version, causal
parent shape, affected artifact family, and governance. A containing artifact
cannot disclose a decision more broadly than the decision's own classification
allows.

## How It Executes

```text
domain declares complete callback-free artifact meaning
  -> Query validates internal consistency
  -> canonical identity is derived from every semantic dimension
  -> package installation checks conflicts and closure
  -> Query mints installed artifact-contract authority
  -> runtime artifact owners and readers consume that authority
```

Validation occurs before provider allocation. A malformed or conflicting
contract cannot be repaired by a runtime callback.

## Small Example

```rust
use worth_query_host::facade::domain;

let contract = domain::WorthQueryPortableArtifactContract::declare::<CandidateSet>(
    schema_version,
    protocol_version,
)
.identity(content_identity)
.ownership(ownership)
.occurrence(occurrence)
.evidence(evidence)
.reproducibility(reproducibility)
.search(search)
.convergence(convergence)
.transformation(transformation)
.access_path(access_path)
.carriage(carriage)
.lifecycle(lifecycle)
.counters(counters)
.decisions(decisions)
.governance(governance)
.compatibility(compatibility)
.produced_by(["candidate-generator"])
.consumed_by(["candidate-validator"])
.finish()?;
```

The example is intentionally explicit. An omitted field is not interpreted as
`NotRequired`; the builder returns a validation denial.

## Real Example

A solver candidate artifact might declare:

```rust
let counters = domain::WorthQueryStructuralCounterContract::declare([
    generated_candidates,
    rejected_candidates,
    retained_candidates,
    retained_bytes,
]);

let decisions = domain::WorthQueryDecisionRecordContract::declared([
    candidate_rejection_schema,
    candidate_ranking_schema,
]);

let contract = domain::WorthQueryPortableArtifactContract::declare::<CandidateSet>(
    schema_version,
    protocol_version,
)
.identity(owner_canonical_identity)
.ownership(provider_owned_payload)
.occurrence(per_operation_attempt)
.evidence(candidate_evidence)
.reproducibility(seed_locked_reproducibility)
.search(bounded_search)
.convergence(explicit_convergence)
.transformation(loss_accounted_transformation)
.access_path(native_candidate_layout)
.carriage(move_and_borrow_only)
.lifecycle(explicit_disposal)
.counters(counters)
.decisions(decisions)
.governance(restricted_candidate_governance)
.compatibility(version_window)
.produced_by(["solver"])
.consumed_by(["invariant-validator"])
.finish()?;
```

The concrete schema constructors depend on the domain's installed vocabulary,
but the closure is fixed: counter units, decision governance, access paths, and
lifecycle are part of the artifact's meaning.

## How It Relates To Other Features

- [Managed Artifact Ownership And Native Access](./managed-artifact-ownership-and-native-access.md)
  consumes the installed contract to own and read runtime values.
- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
  reserves the capacity required by those contracts.
- [Provider Sessions And Decision Read-Sets](./provider-sessions-and-decision-read-sets.md)
  uses a different decision surface: exact facts that justify proposed work.

## Inspection And Debugging

Inspect:

- `contract.identity()`, `family()`, schema and protocol versions;
- content and occurrence identity policies;
- required and optional counter rows, units, aggregation, and reset boundaries;
- decision classification, redaction, retention, and audiences;
- installed package and admission identities;
- typed validation or installation denials.

Human-readable labels and reports are diagnostic. The installed authority is
the usable proof.

## Anti-Patterns

- Putting callbacks, `Any`, provider handles, pointers, or session tokens in a
  portable contract.
- Treating a contract ID as an installed contract authority.
- Using an empty collection to mean a capability is absent.
- Aggregating elements into comparisons or bytes without a declared compatible
  unit law.
- Letting artifact-level disclosure override a stricter decision schema.
- Treating candidate scores, counters, or decision records as approval.
- Silently changing a semantic field without changing contract identity.

## Current Limits

- Provider-specific values remain runtime-local.
- A declared native access path still requires a matching installed provider.
- Contracts describe reconstruction posture but do not themselves reconstruct
  artifacts.
- Evidence and reports remain descriptive and cannot be readmitted as
  execution authority.

## Related Docs

- [Managed Artifact Ownership And Native Access](./managed-artifact-ownership-and-native-access.md)
- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
