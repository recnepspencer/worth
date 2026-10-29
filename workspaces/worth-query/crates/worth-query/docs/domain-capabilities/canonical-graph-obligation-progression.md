# Canonical Graph Obligation Progression

## What This Feature Is

An installed application query or operation carries the graph work it requires.
Query selects that installed meaning, admits one bounded plan, executes it in
one managed session, and publishes only what the real terminal proves. Use this
model when application work reads graph relationships, checks access, mutates
graph state, or runs installed invariants.

The caller declares application meaning. The caller does not register runtime
obligations, choose lower owners, construct plans, or run validators directly.

## Why You Use It

- Give every application query and operation one inspectable graph-work
  contract.
- Keep graph-read planning, authorization, invariant execution, and commit in
  one affinity-bound progression.
- Preserve exact ownership: Relational owns graph truth, Runtime Bridge owns
  installed correspondence, Signal owns policy evidence, and Query composes
  their evidence.
- Prevent a read receipt or publication object from becoming execution
  authority.

## Stable Entry Points

Application authors use the declaration and host facades:

- `worth_query_decl::facade` application-schema, query, and operation macros;
- `worth_query_host::facade::domain` for installed query or operation
  inspection;
- `installed_schema.native_contracts()` for the sealed application aspect
  catalog;
- `installed_operation.contracts().graph_reads()` and `.touches()` for exact
  typed graph scopes, plus `.emissions()` for declared application effects;
- `worth_query_host::facade::inspect_installed_graph_obligations(...)`
  for read-only downstream adoption evidence;
- the ordinary application runtime for query and operation execution; and
- `worth_query_host::facade::publication` for non-authoritative terminal
  receipts and inspection.

Raw selection, plan review, capacity reservation, session progression, and
terminal construction are integration boundaries. They are not caller
authoring APIs.

## Core Mental Model

An obligation is installed meaning, not a callback. For one exact installed
query or operation, Query derives a sealed set containing the required work,
the lower owners that must contribute evidence, the selection basis, resource
posture, effect posture, and required terminal.

```text
typed application declaration
  -> sealed installed native aspect catalog
  -> exact typed operation read and touch contracts
  -> sealed installed obligation set
  -> selected obligation rows
  -> requirement and cost review
  -> capacity-reserved graph-work plan
  -> branch- and basis-bound provider session
  -> actual lower-owner work
  -> execution terminal
  -> application aftermath and optional runtime-local recovery
  -> non-authoritative publication and inspection
```

The typed branch is part of the authority. A version number from another
branch is not interchangeable, even when its numeric value is equal. Plan,
session, decision facts, proposal, invariant evidence, commit, dispatch outbox,
recovery handle, receipt, and publication all retain the same branch-qualified
basis.

## How It Executes

### Derived relation meaning

A derived endpoint set is named with
`ApplicationDerivedRelationRef` and builds its recipe with
`ApplicationDerivedRelation`. Recipes compose actual stored or named-derived
references, reverse direction, and optionally exclude self edges. Completed
schema validation checks the named dependency DAG and exact endpoint kinds.
`ApplicationRelationGraphLaws` describes endpoint-set graph obligations; stored
edge deletion and physical uniqueness remain with stored relations.

`rule_readable_collection` records the existing Feature, Collection, Computation,
output artifact and completeness identities. `operation_read_derived_relation`
records an operation's declared source read. These associations grant no read,
publication or capability authority. Installed endpoint-set laws are enforced by
the actual Native source evaluator, including proposed-state changes to physical
support. The closed managed count requires a genuine Entity-rooted input.

Ordinary invariant setup resolves a typed derived relation against its actual
installed physical leaf roster. Preparation admits the complete source under
the genuine proposed or committed observation; evaluation consumes that frozen
proof with the same observation, root generation, posture, definition and access
contract. Proposed reads cannot borrow a committed proof or cached endpoint set.
Decision readers additionally require the actual operation's declared derived
read and issued physical scopes. They record every physical entity and explored
adjacency, including empty adjacency and distinct paths with equal endpoints;
a failed source read vetoes the decision even if a handler swallows its error.

For a count projection, contribution setup uses `derived_relation_count` with
the same declared recipe and actual collection traits. Its computation declares
`ApplicationDerivedRelationCountPartition`, the built-in deterministic one-bucket
key, and `DeterministicPartitioned` execution. The closed framework owner feeds
the existing partitioned computation and collection completion.
Installation refuses a custom owner that only repeats the descriptive collection
association. The existing derived-view registry retains the disposable count and
its reverse dependency index under the actual Product branch/incarnation/commit.
Physical changes invalidate it even when the derived endpoint set stays equal.
Every count first acquires its Native source, so warm retention is not a promise
of avoiding physical proof work. `discard_rule_readable_collections()` removes
these projections; later reads can reconstruct them without changing truth or
ordinary Query views.

The [facade journey](../../../worth-query-court-host-application-graph/tests/worth_query_court_host_application_graph/certification/application_graph/relation_laws/derived/court/mod.rs)
uses an authenticated action, real proposed-state rule, negative-adjacency edit,
physical rewire and discard/rebuild. The [Native audience court](../../../worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/derived_relation/native_audience.rs)
checks that a broad proof cannot lend its physical source access to a narrower
reader on the same observation. The [logical composition journey](../../../worth-query-court-host-application-graph/tests/worth_query_court_host_application_graph/certification/application_graph/relation_laws/logical/court/mod.rs)
uses the same source owners for laws, walks, orbits and components. The
[grant-revocation journey](../../../worth-query-court-host-certification-fast/tests/worth_query_court_host_certification_fast/fast/certification/mod.rs)
keeps a derived value available while actual grant revocation denies the action
before provider access; a derived value cannot replace that grant.

### Finite projected sources and composition

`ApplicationProjectedMemberDomainDefinition` declares a real carrier kind and
finite named cases. `ApplicationDerivedRelation::carrier_case`, reversal,
composition, union and intersection use the same registered evaluator. Complete
physical and empty-adjacency support survives endpoint deduplication. A logical
coordinate includes domain, full carrier generation and case; it grants no access.

In the [public logical fixture](../../../worth-query-testkit-certification-application-graph/src/certification/application_graph/relation_laws/logical.rs),
Uses has Negative and Positive cases over Node. The `successor()` helper is the
forward Negative-lift inverse -> Precedes -> Positive-lift leg. These actual
builder-chain lines register its union with the reverse as an orbit, then bind
an explicit component set:

<!-- compiled: logical_relation_source.rs -->
```rust
        .derived_relation(
            UseSuccessor::reference(),
            successor::<L>().union(successor::<L>().reversed()),
            ApplicationRelationGraphLaws::unbounded().orbit(),
        )
        .derived_components(ApplicationDerivedComponentSetDefinition::new(
            UseComponents::reference(),
            UseSuccessor::reference(),
        ))
```

The real Precedes permutation induces two-member cycles Negative(i) and
Positive(successor(i)); physical support-record count is not orbit length. This
fixture demonstrates logical composition, not the CAD shell model. CAD must
supply its actual incidence and qualification recipe.

An invariant resolves `typed_projected_case`, `typed_derived_relation` and
`typed_component_set` against its actual read and applicability closure. A genuine
carrier observation supplies `view.projected_member(..)`. Here `negative` is that
issued member and `self.source` is the resolved UseSuccessor binding:

<!-- compiled: logical_relation_rule.rs -->
```rust
        let walk = view.derived_walk(&self.source, RelationDirection::Forward, &negative)?;
```

Preparation and evaluation call the same APIs on their own proposed or committed
view. `graph_component_of` retains the actual Native identity and complete cohort;
`derived_orbit` uses genuine permutation order. `graph_components` consumes a
whole-census proof even with no carriers. That explicit enumeration can grow with
the universe; local rules need not request it. Prepared adjacency, direction,
reachability, component and census interpretations are distinct.

Keep changing context in the source's actual incidences, outside the carrier/case
coordinate. A whole-cohort context transfer with the same complete member keys
retains its Native identity without split or merge events. Actual membership
changes follow the existing surviving-member heir and lineage rules. The
[logical lifecycle court](../../../../../../crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/lifecycle.rs)
exercises these cases. A retired carrier generation cannot select a replacement
member; retained observations keep their original selected source. See the
[domain compatibility contract](../../../worth-query-decl/README.md#projected-member-domain-meaning)
for portable meaning and fresh admission.

Operations declare `operation_read_derived_relation` plus the actual typed
`OperationReads` marker. Extended recipes and derived component/orbit consumers
receive an installed composite DerivedSource scope beside their full physical
and carrier scopes. `DecisionReader::entity_member` and
`derived_member_relations_from` issue local observations; `derived_walk` and
`graph_component_of(set, relation, member)` retain the actual source and answer.
An equal result does not erase source dependencies: rewriting and restoring
physical support must stale an older candidate. A projected graph member cannot
be passed to `mutation_target`; the Host [compile-time fence](../../../worth-query-host/src/logical_member_contract.rs)
keeps the genuine Entity-identity counterpart explicit.

The closed managed count retains its Entity-identity input. Entity-to-Projected
recipes are supported; logical-root counts refuse installation until a genuine
input owner exists. Arbitrary cached collection values remain descriptive.
Every count acquires fresh Native proof. These rule and operation APIs add no
ordinary disclosed logical query rows, capability, approval or publication grant.

Read-only application work follows one path:

```text
installed query
  -> graph-read obligation selection
  -> specialized graph-read requirement and budget review
  -> managed session
  -> session-owned graph read
  -> read terminal and resource-release evidence
  -> disclosed result and publication receipt
```

Mutation work continues from the same session spine:

```text
installed operation
  -> complete authorization decision read-set
  -> proposed state
  -> every selected installed invariant runs through its real owner
  -> sealed Relational validated candidate
  -> compare-and-commit
  -> idempotency and optional co-committed dispatch outbox
  -> external-effect observation
  -> fresh or recovered commit terminal
  -> aftermath and publication receipt
  -> optional receipt-bound runtime recovery
```

Selection never counts as execution. A selected invariant becomes successful
only after the installed provider executes it against the exact proposed state.
Publication consumes a terminal; it cannot execute, retry, commit, dispatch,
or recover work.

Likewise, the installed touch contract is a legal ceiling, not evidence that a
mutation happened. Execution compares actual Relational touched-graph evidence
with that declared ceiling. An `Emit` program target is not a graph touch: it
has a sealed installed application-effect emission contract. Only an effect
also declared external receives the separate escaping correlation and protocol
contract.

## Small Example

Given an installed application query, downstream code may inspect its sealed
obligation set without acquiring planning or execution authority:

```rust
use worth_query_host::facade::inspect_installed_graph_obligations;

let proof = inspect_installed_graph_obligations(
    "reporting-service",
    installed_query.graph_obligations(),
)?;

assert_eq!(proof.subject_name(), "account_activity");
assert_eq!(proof.rows().len(), 1);
```

`WorthQueryGraphObligationAdoptionProof` exposes installed identities, kinds,
required owners, terminal requirements, and selector-index size. It exposes no
registration, selection, planning, execution, invariant, or commit transition.

For an installed operation, inspect exact semantic meaning through the same
Host domain facade rather than decoding diagnostic text:

```rust
use worth_query_host::facade::domain::{
    WorthQueryOperationGraphReadScope, WorthQueryOperationTouchScope,
};

for role in installed_operation.contracts().graph_reads().roles() {
    for scope in role.read_scopes() {
        match scope {
            WorthQueryOperationGraphReadScope::Entity(scope) => {
                inspect_entity(scope.schema(), scope.semantic_key());
            }
            WorthQueryOperationGraphReadScope::NativeProjection(scope) => {
                inspect_projection(
                    scope.entity().semantic_key(),
                    scope.aspect().as_str(),
                    scope.projection().mask(),
                );
            }
            WorthQueryOperationGraphReadScope::Relation(scope) => {
                inspect_relation(scope.schema(), scope.relation(), scope.from(), scope.to());
            }
        }
    }
}

for scope in installed_operation.contracts().touches().scopes() {
    match scope {
        WorthQueryOperationTouchScope::WriteField(field) => {
            inspect_field(field.schema(), field.entity(), field.contract(), field.field_path());
        }
        _ => inspect_structural_touch(scope),
    }
}
```

The installed native catalog is the one application-schema truth source used
to compile those scopes. A projection mask preserves either explicit
whole-aspect posture or its exact canonical field paths; an empty vector is
not a substitute for either meaning.

## Real Example

The external host-consumer test installs a normal declarative query, obtains
its installed query handle, and compares the graph-read obligation's bound
planning identity with the installed query's canonical read graph. It also
proves the read-only adoption view reports one `GraphRead` row owned by
Relational and requiring a `GraphReadProduct` terminal.

The executable example is
`worth-query-court-host-fast/tests/worth_query_court_host_fast/fast/host/canonical_graph_progression/mod.rs`. It deliberately lives
in the host crate, whose normal dependency graph contains destination packages
and no `worth-query` monolith dependency.

For application execution, use the ordinary typed host API. Do not reproduce
the admission sequence shown above in application code; Query performs it as
part of preparing and executing the installed query or operation.

## How It Relates To Other Features

- [Graph Read Access Planning](../authoring/graph-read-access-planning.md)
  explains the requirement, budget, and support review inside application-query
  admission.
- [Provider Sessions And Decision Read-Sets](./provider-sessions-and-decision-read-sets.md)
  explains session affinity and complete authorization facts.
- [Provisional State And Invariant Execution](./provisional-state-and-invariant-execution.md)
  explains real installed invariant execution before commit.
- [Graph Composition Authoring](../authoring/graph-composition-authoring.md)
  is the preserved generic workspace mutation engine. It is not an alternate
  application-obligation executor.
- The generic workspace-read engine remains available for its existing
  non-application responsibilities. It is not a second application-query plan.

## Inspection And Debugging

Inspect the progression at its stable boundaries:

- installed obligation identity and rows;
- selected-row counters and exact owner requirements;
- graph-read requirement, cost, budget, and inventory review;
- plan, session, managed-run, branch, and basis identities;
- complete authorization decision facts;
- invariant execution counts and dispositions;
- terminal cleanup and resource-release evidence; and
- publication work counters and fresh-versus-recovered posture.

When identities disagree, treat the result as affinity drift. Do not compare
rendered strings or reconstruct an identity from a version number.

## Anti-Patterns

- Registering graph obligations in a consumer crate.
- Treating selection, support, or a no-op callback as execution.
- Passing a manual invariant pack to generic graph or read composition.
- Importing admission internals to construct or review a raw plan.
- Calling Relational directly from an application-query lane.
- Parsing a rendered read or touch string to recover authority.
- Reconstructing an application aspect contract outside the installed native
  catalog.
- Treating a declared touch as performed mutation evidence.
- Defaulting a missing branch to `"main"`.
- Publishing a receipt before terminal cleanup is proved.
- Keeping a compatibility alias that can mint a retired authority product.

## Current Limits

- Runtime-local receipt-bound recovery is supported. Durable restart recovery
  and store-backed checkpointing remain deferred.
- It does not implement multiple branch heads, branch creation, merge, rebase,
  or concurrent branch writers.
- Read-only adoption is inspection, not a consumer extension system.
- Generic monolith read and graph-composition features remain preserved until a
  later named migration proves their parity and authorizes their retirement.

## Compatibility And Migration Map

| Inherited surface | Preserved guarantee | Current owner and integration | Retirement posture |
|---|---|---|---|
| 9.9 graph-obligation meaning | Kinds, owner routes, selection basis, resource/effect posture, terminal requirements, denials, and bounded lookup | Installation owns the sealed set; admission selects and plans; execution obtains actual owner terminals | The monolith obligation tree, fake executor, dispatch envelopes, and authority-capable Consumer Kit registration/execution are retired; generic graph composition remains |
| 9.9 manual invariant callbacks | Lawful success and violation behavior for installed invariant families | The application-operation session invokes the installed provider; Relational seals the validated candidate after all real invariant phases | Callback and no-op composition authority is retired; contributed capability denial inspection remains non-executable |
| 9.10 graph-read access planning | Requirement derivation, inventory matching, cost, budget, typed denial, and inspection | Admission owns the specialized application-query review; execution consumes it through the session-owned read port | Public raw planning-input/review construction and the application-query monolith admitted-plan export are retired; generic workspace reads remain |
| 9.11 downstream adoption | Honest public-facade adoption and exact consumer residue | Host exposes read-only installed-obligation inspection; certification and consumer suites prove source residue | Local graph-obligation registration, execution proof workspaces, and support-pin authority are retired; the unrelated generic Consumer Kit remains |
| 9.15 provider sessions | Provider lifecycle, complete decision read-sets, provisional state, installed invariant execution, and cleanup | Execution binds the existing managed run as the exact worker inside the application graph-work session | Integrated unchanged; it is a lower worker, not a competing application session or receipt authority |
| 9.16 Phase 6 application queries | Identity, parameters, basis, ordering, projection, continuation, history, preview, live, result shaping, and publication behavior | Every lane consumes the destination graph-read plan and session; semantic result shaping remains with application query | Parallel direct Relational execution and parallel application-query plan authority are retired |
| 9.16 Phases 7.1-7.2 access | Installed capability meaning, purpose, request binding, trusted time, exact grant/path, policy evidence, and revalidation | The complete evidence set is observed and revalidated inside the same branch-bound session | Pre-session observation and any reconstructed authorization scope are retired |
| Query and operation publication | Existing result/commit identity, omission, work, cleanup, response-loss, and retry behavior | Publication derives read-only receipts from execution terminals and distinguishes fresh execution from recovered prior commit | Public constructors and any selected-versus-executed synthetic outcome path are retired |
| 9.16 Phase 8 application aftermath | Explicit correction and external-effect meaning, co-committed outbox/idempotency facts, exact dispatch posture, receipt-bound recovery, and weaker publication | Declaration and installation own static meaning; execution owns commit, dispatch, and live recovery; publication owns closed descriptions | Generic recovery slots, guessed completion, and provisional undo/redo as an accepted product are forbidden |
| Facades and downstream consumers | Ordinary source behavior and full Query, Bank, and Worth UI features | Declaration and host facades lower one way into destination packages; Bank and Worth UI consume those facades | Only aliases that reopen retired authority are removed; unrelated adapters and monolith behavior remain |

## Successor Contract

Later disclosure and noninterference work begins from the same installed query,
typed branch, parameter binding, graph-read plan, managed session, access
decision, result shaping, continuation, history, preview, live, and publication
contracts. It must not add a disclosure-specific selector, planner, raw graph
executor, authorization lane, receipt, or branch default.

## Related Docs

- [Graph Touch Obligation Authority](../authoring/graph-touch-obligation-authority.md)
- [Graph Read Access Planning](../authoring/graph-read-access-planning.md)
- [Provider Sessions And Decision Read-Sets](./provider-sessions-and-decision-read-sets.md)
- [Provisional State And Invariant Execution](./provisional-state-and-invariant-execution.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
