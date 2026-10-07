# worth-query-installation

`worth-query-installation` owns portable Query installation meaning. It is the
small package to compile when changing domain package, installed operation,
workflow, conditional-node, or canonical installation contracts.

Use its facade:

```rust
use worth_query_installation::facade::*;
```

Application and domain consumers use the same types through
`worth_query_host::facade::domain`. Import this crate directly only when
working on installation meaning itself.

## Portable Meaning Only

This crate owns:

- domain package definitions and canonical package identity
- typed domain operation definitions and semantic closures
- workflow DAG declarations
- portable conditional-node declarations
- installed external-effect protocol, correlation, and payload-bound contracts
- installed aftermath correction authority, correction mechanism, pre-image,
  next-action, and published-posture contracts
- semantic truth dependencies
- validation, admission inputs, conflict denials, and rebuildable installed
  indexes

It does not own:

- executor callbacks
- graph providers
- Runtime Bridge instances
- Signal graphs, nodes, or aspect allocations
- runtime support profiles
- execution receipts or Query consequences

Portable definitions must remain callback-free and runtime-independent.

## Installed Operation Contract

`WorthQueryDomainOperationDefinition<D, O, F>` binds exact domain, operation,
and family marker types to `WorthQueryDomainOperationSemanticClosure`.

The closure states parameters, canonical query and result shape, graph reads,
touches/effects, external-effect and aftermath meaning, workflow, conditional
nodes, replay, lineage, promotion, publication, support, terminal states,
failure classes, cost, and deterministic lowering identity.

Use typed `NotRequired` variants for absent capabilities. Provider absence and
empty labels are not semantic declarations.

## Conditional Authoring

`WorthQueryPortableConditionalNodeDeclaration::declare(...)` requires
dependencies, outputs, context, condition, trigger, comparison, artifact
policy, maintenance, and output relationship before `finish()` succeeds.

Dependencies carry Foundational aspect contracts and masks plus Relational
bindings, locality, relevant change kinds, and graph-read role. They never
contain runtime-local Signal allocation.

## Canonicality

Equivalent declaration order converges to one canonical identity. One-field
semantic drift produces a conflict. Derived lookup indexes must be rebuildable
from portable installed artifacts without changing identity or denial outcomes.

Package and application-schema identity derivations each have a fixed 32 MiB
canonical encoded-byte ceiling. This provides headroom for the composed House
component application, whose package exceeded the former 16 MiB ceiling after
its capabilities were narrowed to the read vocabulary it consumes. This
allowance belongs to Query installation; consumers cannot raise it.

Package validation checks entry breadth before member sorting and canonical
basis construction, using the entry-storage bound derived from that finite
byte ceiling. Canonical encoding checks each append before accepting bytes
beyond the ceiling and reports typed exhaustion. The ceiling bounds encoded
material, not total installation memory; basis storage and canonical allocation
remain separate costs. Raising it changes admission headroom without changing
canonical rules, ordering, versions, or identities of previously admitted
packages and schemas.

## Related Docs

- [Runtime-Installed Domains And Operations](../worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](../worth-query/docs/domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed Stops And Remediation Guidance](../worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed Operation Lineage And Promotion](../worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Application Aftermath, External Effects, And Recovery](../worth-query/docs/execution/application-aftermath-and-recovery.md)
- [Worth Query Orientation](../worth-query/docs/AI_README.md)
