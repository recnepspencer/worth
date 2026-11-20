# Semantic Correspondence And Conditional Execution

## What This Feature Is

Semantic correspondence binds a portable Query truth dependency to the exact
Signal targets used by one runtime. Conditional execution then lowers Query’s
portable condition contract into that installed Signal node and returns
Signal-minted decision evidence.

Application and domain code normally enters through Query. Use the Bridge
facade directly only while constructing a runtime integration or implementing
the Bridge-owned boundary.

## Stable Entry Points

- `BridgeSemanticDependencyCandidate`
- `BridgeSignalAspectTargetDeclaration`
- `BridgeSemanticCorrespondenceRegistration`
- `BridgeInstalledSemanticCorrespondence`
- `CorrespondenceAdmissionOutcome`
- `BridgeConditionalInstallationRequest`
- `BridgeConditionalProviderSet`
- `BridgeInstalledConditionalLowering`
- `BridgeOwnedSignalRuntime`
- `BridgeConditionalExecutionRequest`
- `BridgeConditionalDecisionEvidence`

## Semantic Input

The dependency candidate retains the Query installation, basis, graph role and
adapter authority, Foundational aspect contract and mask, Relational binding
and relevant change meaning, locality, and authoritative source profile.

It does not retain a caller-selected Signal slot. Signal targets are volatile
runtime allocation selected during construction.

## Admission

Correspondence admission resolves the candidate against the exact Query
dependency registry, maps registered targets, allocates slots, and asks Signal
to admit the actual graph/node/aspect capabilities.

The outcome keeps denial, deferment, stale, rebind-required, and failure
separate. Only success mints `BridgeInstalledSemanticCorrespondence`.

Successful precision is:

- `Exact`
- `DeclaredWidening`

Widening must be declared. Unsupported precision and ambiguous target ownership
are denials, not reasons to fall back to whole-graph invalidation.

## Conditional Provider Sets

`BridgeConditionalProviderSet` carries volatile implementations for the
provider families declared by the portable Query node:

- typed domain condition
- temporal wake
- typed on-demand trigger
- dependency comparator
- output comparator
- artifact-reuse comparator

Exact built-in conditions require no duplicate provider. Typed custom families
require exactly the matching provider. Missing or extra providers fail runtime
construction.

Providers supply mechanics; they do not redefine portable identity or choose a
different condition policy.

## Execution

`BridgeOwnedSignalRuntime` retains one Bridge and one Signal graph. Execution:

1. verifies the installed lowering, runtime, graph, snapshot, and attempt
2. materializes the precompiled semantic observation plan
3. resolves the installed condition through Signal
4. contacts compute only when Signal admits it
5. lets Signal compare dependency, output, and artifact reuse posture
6. returns Signal decision evidence without restamping

Ineligible, suppressed, and deferred decisions contact compute zero times.
Reverted-clean decisions retain compute cost but carry no semantic delivery.

## Authoritative Delivery

Relational changes arrive with aspect identity, revision, binding, field path,
change kind, source identity, and precision. Delivery must match the installed
correspondence before updating a Signal target.

The following cannot authorize delivery:

- a mapping label
- an equal numeric Signal aspect
- a diagnostic digest
- a copied Query dependency
- a target from another graph or runtime

## Rebuild And Inspection

Use `RuntimeBridge::rebuild_correspondence_allocation_index()` to reconstruct
derived allocation indexes from authoritative registrations. Exact parity must
preserve target allocation and counters.

Inspect installed basis, target count, precision, admission counters, lowering
identity, semantic observations, and returned Signal evidence. Reports explain
the boundary but cannot satisfy it.

## Related Docs

- [`README.md`](./README.md)
- [`API_OVERVIEW.md`](./API_OVERVIEW.md)
- [Query Conditional Installed Operations](../../workspaces/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Query Aspects And Authority Lanes](../../workspaces/worth-query/docs/modeling/aspects-and-authority-lanes.md)
