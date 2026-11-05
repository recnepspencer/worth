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

Package and application-schema identity derivations each have a fixed 64 MiB
canonical encoded-byte ceiling, matching portable package export and
reconstruction capacity. This allowance belongs to Query installation;
callers may narrow canonical work below it, but cannot raise the platform limit.

Package validation checks entry breadth before member sorting and canonical
basis construction, using the entry-storage bound derived from that finite
byte ceiling. Canonical encoding checks each append before accepting bytes
beyond the ceiling and reports typed exhaustion. The ceiling bounds encoded
material, not total installation memory; basis storage and canonical allocation
remain separate costs. Raising it changes admission headroom without changing
canonical rules, ordering, versions, or identities of previously admitted
packages and schemas.

Installed graph-obligation, authorization-path, authorization-policy and
operation-definition identities use an internal allowance derived from the
admitted declaration: at most **16 times its canonical entries and encoded
bytes** for each derivation. The allowance is carried from the freshly admitted
schema commitment, never supplied by an application or inferred from derived
output. There is no smaller independent installation ceiling. Declaration
meaning is admitted at package validation under the shared 64 MiB ceiling and
its byte-derived entry-storage bound. The 32,768-entry / 4 MiB budget in
`admission/identity.rs` governs support metadata, not declaration meaning.
A declaration-size refusal belongs at that validation door and retains the typed
kind and requested/maximum counts.

### Derived identity bound

Let E and B be the admitted schema commitment's entry and encoded-byte counts.
For any one operation (or query, inspection, resource or managed-index purpose),
the following source assignment bounds its identity by 16E entries and 16B bytes:

- Distinct native contracts are charged to their declaring aspects and fields
  once. A flat Struct field emits four native contract facts; its declaration
  already emits twelve field facts. Scalar, opaque and reference contracts have
  fixed framing charged to their aspect declaration. The two table framing
  entries are charged to that same declaration. No new nested shape is involved.
- Each projection/touch occurrence is charged to its own read/program member.
  It emits one contract ordinal, at most four binding entries and its kind,
  entity/aspect/relation and selected path facts. Installed paths are flat scalar
  fields. Complete explicit Struct masks emit one `complete-fields-v1` marker
  against the exact referenced contract; partial and whole-aspect masks retain
  their exact canonical material. No field inventory is multiplied by the
  number of references.
- Authorization paths reuse the declaration's exact canonical component emitter
  plus one family entry. A policy emits its three names, path count and one
  digest per declared path; operation authorization emits one digest per
  applicable declared ability/capability. Effects and invariant executions emit
  each applicable declaration once, including every state-load family and bound.
  There is no cross product of requirements, paths or fields within an identity.
- Subject, binding, role/count and resource framing is charged to schema headers
  and the subject declaration. Query and managed-index graph meanings are fixed
  digests, not expansions of their graphs; inspection meaning embeds its already
  admitted definition once. The operation-definition identity has four entries.

Under this assignment each source fact supplies at most four derived facts,
including framing. Values are copied or replaced by fixed integers/digests;
qualified locus/domain/framing overhead is at most four times its source encoding
(the source already frames domain, kind, locus and value). Thus even charging
both multiplicities gives 16E and 16B, including the smallest declarations and
all constant framing. E is at most floor(64 MiB / 30), so 16E fits u32; 16B is
at most 1 GiB and fits usize on supported 32-bit targets. Multiplication is checked.
This is a per-identity bound, not a claim about total installation allocation or
all operations' summed work.

The installed-index v4 identity commits the exact source package/admission table
and catalog cardinalities. All catalogs are deterministic projections of those
validated sources; repeating definition semantics or owner strings adds no
meaning. Its allowance sums the per-package derived allowances (saturating at
representable budget maxima); the empty index has eight fixed framing entries.
This removes the index's former 32,768-entry / 4 MiB installation gap without
multiplying long owner names by catalog breadth.

The v3 graph-obligation rule encodes every distinct aspect contract once in an
identity-local table and one ordinal per read projection or touch. Contract
version and entry count frame each table member; exact canonical entries decide
sharing. Masks, field paths, bindings, structural scopes and resources keep their
exact meaning. The v3 rule changes graph-obligation identities and dependent
seals; portable package/schema and authorization identity rules remain unchanged.
`canonical_denial()` still retains canonical derivation failures, but ordinary
admitted declaration breadth no longer encounters a second fixed cliff.

Courts install a 40-record / 12-field operation and an exactly 64 MiB admitted
package, refuse its one-byte-larger twin at validation, install authorization
paths/policies beyond the former fixed ceiling, preserve identity sensitivity
and linear growth, and reject replacement of the derived allowance by the old
fixed budget.

## Related Docs

- [Runtime-Installed Domains And Operations](../worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](../worth-query/docs/domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed Stops And Remediation Guidance](../worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed Operation Lineage And Promotion](../worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Application Aftermath, External Effects, And Recovery](../worth-query/docs/execution/application-aftermath-and-recovery.md)
- [Worth Query Orientation](../worth-query/docs/AI_README.md)
