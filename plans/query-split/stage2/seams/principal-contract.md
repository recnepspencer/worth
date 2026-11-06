# Stage 2 seam: the admitted principal and scope contract (W2-06)

Design for packet W2-06 after its ruling (option 3). This document replaces the lowering that
`merges.md` §2 planned for the principal family (Migration 3). Source state is `e200ba0e00`.
Counts come from the done-check simulation: `vocabulary_simulation.py` at domain level, with
`--impl-limit 600`, the draft stack and the designed exclusions. Each reference is a distinct
pair of source file and item. Paths are relative to
`workspaces/worth-query/crates/worth-query-execution/src/domain_computation/`, and `pg/` means
`primary_graph/`.

## Why the values cannot be lowered

`merges.md` §2 planned to move `authenticated_principal`, the two freshness evidence types and the
entity identity values down into a new `application_principal` module. Checking the code
showed that each value is built by primary-graph code that stays in the primary graph:

| construction door | caller in the primary graph |
|---|---|
| `WorthQueryAuthenticatedPrincipal::mint` | `pg/resolution/selected_product.rs`, `…/admitted/native.rs` |
| `WorthQueryResolvedPrincipalEvidence` literal | `pg/resolution.rs` |
| `WorthQueryPrincipalFreshnessEvidence::new` | `pg/resolution.rs`, `pg/resolution/admitted.rs` |
| `WorthQueryResolvedEntity::from_lookup` | `pg/entity_resolution/{admitted_scope.rs, mod.rs}` |
| `WorthQueryEntityResolutionSubject::new` | `pg/entity_resolution/{admitted_scope.rs, mod.rs}` |

Rust lets `pub(in path)` name only an ancestor of the item's own module. If these values moved
down, each of these doors would have to become `pub(in crate::domain_computation)`, which would
open the principal mint to every domain sibling. The ruling therefore keeps each value, its
producer and its seal in the primary graph. The cut is made at the authorization side instead:
authorization reaches these values only through a contract that it owns.

## What authorization needs (198 references in 67 items before the cut)

| item | refs | what authorization does with it |
|---|---|---|
| `WorthQueryAuthenticatedPrincipal` | 11 | reads binding, binding identity, external identity, clock, validity, entity id and runtime authority; asks the graph to re-validate its freshness at a snapshot; captures a currentness dependency |
| `WorthQueryApplicationEntityIdentity` | 9 | reads entity id, kind, name, runtime authority and binding identity; retains its scope selector (plain or funded); asks the graph to re-validate it at a snapshot; holds a resolved one by value in capability request resolution |
| `freshness::validate_freshness_at_snapshot` | 2 | the freshness re-validation above, called with the concrete principal |
| `freshness::WorthQueryDurablePrincipalCurrentness` | 2 | decoded with serde from a durable workflow approval and compared later |
| `entity_resolution::resolved::WorthQueryResolvedEntity` | 2 | returned by the entity resolution that capability admission drives |
| `freshness::WorthQueryPrincipalFreshnessEvidence` | 1 | stored by value in `WorthQueryPrincipalCurrentnessDependency` |

Every accessor authorization reads returns a type defined outside the primary graph: external
crates, `vocabulary::*` or authorization's own types. The graph-reading operations (freshness at a
snapshot, entity validation, selector retention) take and return only those same types.

## The contract

Two traits live in the new module `authorization::admission_subject`. Each is object-safe and
used as `&dyn` everywhere.

- `WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>`:
  - accessors: `binding`, `binding_identity`, `external_identity`, `clock`, `valid_until`,
    `is_expired`, `principal_entity_id`, `runtime_authority`;
  - `capture_currentness(session, layout) -> WorthQueryPrincipalCurrentnessDependency`;
  - `validate_freshness_at_snapshot(execution, relational, snapshot, layout, expected) ->
    Result<(), WorthQueryPrincipalResolutionDenial>`.
- `WorthQueryAdmissionScope<Schema, Scope>`:
  - accessors: `entity_id`, `entity_kind`, `entity_name`, `runtime_authority`,
    `binding_identity`;
  - `retain_scope_selector`, `retain_scope_selector_funded(execution)`;
  - `validate_freshness_at_snapshot(execution, runtime, relational, snapshot) ->
    Result<(), WorthQueryEntityResolutionDenialKind>`. Here `runtime` is
    `&dyn AuthorizationHost<Schema>`; the graph obtains its own installed resolution context
    through it.

**Placement.** The host-supertrait order puts authorization below the primary graph: every
primary-graph host extends `AuthorizationHost`. Authorization is the only consumer of these
traits, so it owns them, the same way it owns `AuthorizationHost`. A lower existing module such as
`vocabulary` would have to name `WorthQueryPrincipalCurrentnessDependency` and
`AuthorizationHost`, which are authorization's types, and that would point upward. The trait
visibility, `pub(in crate::domain_computation)`, equals the widest admission function that takes
them. No inherent method widens: each trait method is as narrow as, or narrower than, the
`pub(crate)` or `pub(in crate::domain_computation)` accessor it forwards to.

**Implementors.** Both implementations sit beside their types, as child modules that can see the
seal but never touch it:
- `pg/authenticated_principal/admission_subject.rs` implements the principal trait for
  `WorthQueryAuthenticatedPrincipal`. It calls `validate_freshness_at_snapshot` and
  `WorthQueryPrincipalCurrentnessDependency::capture` with `self`.
- `pg/entity_resolution/resolved/admission_subject.rs` implements the scope trait for
  `WorthQueryApplicationEntityIdentity`. It calls `retain_entity_resolution_context`,
  `at_snapshot` and `validate_entity_freshness` with `self`, and returns the denial's kind.

**How authorization obtains the value.** Callers already hand authorization a reference to the
principal and the scope. That reference now coerces to the trait object at the call. Callers
include `product_operation`'s `authorize_operation_with_request`, the selected conventional
admission, the query access context and the capability admission entry points. No call site has
to name a new type.

**Why the traits are generic.** The traits keep `Schema`, `Principal` and `PrincipalIdentity` (or
`Scope`) as parameters even though no method uses them. These parameters keep the admitted subject
typed by its schema and principal, as the concrete types were. They also keep every admission
phase's existing type parameters inferable from its arguments. Without them, every phase and every
caller would need a new signature.

**Typed refusals.** The refusals are unchanged. The entity re-validation keeps authorization's
mapping (`Stopped` to the relational read denial, `Refused` to `StaleScope`) and now matches on the
kind it receives. The principal re-validation keeps its `principal_currentness` and
`stale_principal` mappings at the same call sites.

## What stays, and why

- `decision_facts/principal_currentness.rs` keeps `WorthQueryAuthenticatedPrincipal` and
  `WorthQueryPrincipalFreshnessEvidence`.
  - `WorthQueryPrincipalCurrentnessDependency::capture` is the dependency's seal. It is called by
    the primary graph (three sites) and now by the contract's implementation. It keeps taking the
    concrete principal so that binding, external identity and evidence cannot come from different
    principals.
  - The dependency stores the evidence by value. It exposes the evidence as a serde view that is
    encoded by the attempt's workflow actor encoding, and as `WorthQueryApplicationObservedFact`
    lists. A contract for it would have to name those primary-graph types or erase serialization.
- `operation_admission/capability_admission/mod.rs` keeps `WorthQueryAuthenticatedPrincipal`.
  `admit_encoded_capability_access` is `pub` and reachable through the court doors. A
  `pub(in crate::domain_computation)` trait in its signature would be a private interface. Its
  sibling entry point stays alongside it, and both coerce into the contract one call later.
- `workflow_approval_authority{,/decoding}.rs` keep `WorthQueryDurablePrincipalCurrentness`.
  Authorization deserializes the concrete value from durable bytes, and a trait object cannot be
  deserialized.
- `capability_admission/preparation/observation/{request_resolution.rs,
  admitted_access/exact_observation.rs}` keep `WorthQueryResolvedEntity` and one
  `WorthQueryApplicationEntityIdentity`. Capability admission drives the primary graph's entity
  resolution (`WorthQueryEntityResolutionTruth`) and holds the resolved values by value. That is
  the resolver seam, which belongs to a later contract, not to this value contract.

## Measured effect

The done-check simulation and `module_coupling.py report` were run on snapshots of `e200ba0e00`
and of the implementation.

| measure | before | after |
|---|---|---|
| authorization → primary_graph (host-order seam) | 198 refs in 67 items | 179 refs in 66 items |
| `WorthQueryAuthenticatedPrincipal` | 11 | 2 (`principal_currentness.rs`, `capability_admission/mod.rs`) |
| `WorthQueryApplicationEntityIdentity` | 9 | 1 (`request_resolution.rs`) |
| `freshness::validate_freshness_at_snapshot` | 2 | 0 |
| `WorthQueryDurablePrincipalCurrentness`, `WorthQueryResolvedEntity`, `WorthQueryPrincipalFreshnessEvidence` | 2, 2, 1 | 2, 2, 1 (kept, see above) |
| primary_graph → authorization | 556 refs in 105 items | 560 refs in 107 items (downward: the two impl files) |
| domain level, upward references against a fresh layering | 396 | 377 |
| domain level, against host order after the vocabulary simulation | 455 | 438 |
| domain level, upward edges | 14 | 14 (same set; only authorization → primary_graph shrinks) |
| primary-graph level, upward edges and references | 126, 1,141 | 126, 1,141 (same set) |

Neither level gains an upward edge. The new module `authorization::admission_subject` lives inside
authorization, so the domain layering gains no module.
