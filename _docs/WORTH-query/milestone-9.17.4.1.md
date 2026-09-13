# Milestone 9.17.4.1: Unified Application Authoring Model

> **Status:** Planned priority milestone. Begin from the certified 9.17.4
> Phase 1 and Pre-M0 application foundation before expanding the CAD operation
> inventory or resuming the remaining 9.17.4 consumer migrations.
>
> **Product posture:** Application authors declare each semantic fact once.
> Query derives the existing typed contracts, installation, and runtime
> obligations from that declaration.

## Goal And Roadmap Placement

Replace hand-maintained application contract graphs with one typed semantic
authoring model. The generated result remains the strongly typed, explicit,
bounded, deterministic, fail-closed Query runtime completed by 9.17.4's
certified foundation.

The central rule is:

> Authors declare semantics. Query derives obligations.

This milestone is the corrective authoring slice inside
[Milestone 9.17.4](./milestone-9.17.4.md). It begins before further proprietary
CAD M0 operation families are added because the first extrusion proved that
manual contract repetition is already the dominant consumer cost and a source
of contradictory facts. It does not reopen 9.17.3's runtime certification.
The remaining 9.17.4 phases resume through the authoring surface delivered
here; [Milestone 9.18](./milestone-9.18.md) then consumes the same installed
operation and recovery contracts.

This is a full public Query milestone, not a CAD-only macro cleanup. CAD is the
first demanding consumer and decisive adoption proof. Query owns the generic
operation, output-protocol, derivation, invariant-dependency, module-membership,
lowering, and inspection contracts. CAD owns BREP meaning, topology grammar,
geometry algorithms, semantic topology roles, tolerance rules, and geometric
invariants.

## Current Boundary And Demonstrated Defect

The certified runtime has the correct owners, but its application contracts are
still the authoring interface. In the current CAD consumer:

- `RealizeRectangleOperation` declares read, write, create, link, and emission
  authority through operation traits, while `declare_realization` repeats the
  same inventory into the application schema;
- `RealizeRectangleBinding` separately declares input/result/denial bindings,
  scope, source query, identities, handler expectations, and candidate limits;
- rectangle and triangle independently enumerate 85 and 67 output roles, while
  candidate ceilings, creation code, prior-output lookup, and preservation code
  repeat consequences of those inventories;
- initial/preserve and rectangle/triangle produce parallel producer and
  readiness families;
- invariant target strings, typed fields and relations, affected-scope walks,
  and installation repeat related dependency facts;
- contribution `contracts()` and `configure()` paths repeat overlapping module
  membership.

The existing `AuthoredQueryBundleRequest -> CanonicalQueryBundle` pipeline
proves that authored meaning can lower into a canonical contract without
weakening execution. The application operation path lacks that same boundary.

The output problem is broader than fixed extrusion inventories. A boolean,
fillet, offset, intersection, split, or trim may produce cardinality that is
unknown until geometry runs. Query cannot require a compile-time list of every
output, and CAD may not replace governance with an unrestricted `Vec<Entity>`.
Query needs a protocol describing what a proposed output may contain, how each
member explains its existence, and which bounds and invariants it must satisfy.

## Adversarial Constraint And Decisive Proof

The plausible false implementation shortens syntax while retaining multiple
authoritative inventories. It can generate a happy-path rectangle, yet permit a
schema registration to disagree with trait authority, allow a runtime-generated
manifest to exceed its operation envelope, accept positional topology identity,
or keep the old manual installation path beside the generated path.

The decisive court uses the real public application entry and existing primary
candidate/publication owners with two consumers:

1. A fixed extruded-profile family declares profile topology algebraically.
   Triangle, rectangle, and pentagon variants lower from the same family. Exact
   role manifests, create counts, correspondence, candidate requirements,
   producer/readiness bindings, and installation membership are derived. Adding
   the pentagon changes the profile declaration and family membership only.
2. A runtime-generated split family receives an input with bounded variable
   cardinality. Its handler proposes typed output roles and lineage through a
   protocol-bound writer. The input-dependent estimate is admitted before
   construction; the resulting manifest is validated against the installed
   authority and resource envelopes, applied to an isolated candidate, checked
   by required structural and domain invariants, and published atomically.

The hostile sequence must include:

- declaration order permutations that must canonicalize to the same identity;
- duplicate semantic identities and duplicate fixed roles;
- a schema registration, trait projection, or installed binding omitted from
  the authored operation;
- a generated manifest containing one unauthorized entity kind, relation, or
  write;
- an estimator that understates actual items, retained bytes, or work;
- a role whose canonical text matches another role but whose typed meaning or
  entity kind differs;
- a preserved or split output with absent, ambiguous, or stale source lineage;
- a dependency path missing one required hop and a path whose lawful fan-out
  exceeds its admitted work;
- invariant failure after private candidate construction;
- a competing operation on a sibling branch while the hostile candidate is
  denied;
- cleanup after every denial and successful publication.

Required observations are independent: installed contract inspection, actual
candidate effects, committed World occurrence, retained readback, sibling
progress, resource counters, and managed-resource inventory. An unauthorized or
over-budget proposal produces a typed denial before publication, contributes no
partial authoritative graph, emits no publication event, and leaks no
reservation. A structurally valid but invariant-invalid proposal reaches the
private candidate and still publishes nothing. The sibling remains able to
progress.

The court must turn red if generated trait authority, schema lowering, protocol
validation, lineage validation, resource charging, invariant execution, or old-
path deletion is bypassed. A source snapshot of generated text is supporting
evidence only.

## Product Decision Lock

### One authored fact, one owner

The authoring model has five semantic definitions:

| Definition | Authoritative facts | Derived consequences |
| --- | --- | --- |
| `ApplicationOperationSpec` | Operation identity, input/result/denial, scope, source expectation, decision reads, non-output effects, handler binding, resource policy, output protocol reference | Operation marker traits, schema registrations, binding metadata, complete installed authority envelope |
| `ApplicationOutputProtocol` | Output create/preserve/retire roles, output writes/links/unlinks, allowed output language, semantic role type, lineage rules, structural rules, fixed or generated cardinality posture | Output authority projection, fixed manifests, actual-manifest validation, correspondence, candidate requirements, structural checks |
| `ApplicationDerivationSpec` | Producer trigger/source product, applicability variants, lifecycle modes, exact producing operation for each expanded route, required invariants, publication meaning | Operation source expectation and output protocol through the operation reference, producer contracts, readiness contracts, conditional bindings, lifecycle-specific routes |
| `ApplicationInvariantSpec` | Identity, execution point, typed dependency paths, affected scope, predicate binding, resource policy | Portable targets, installed invariant reads, impact closure, execution registration |
| `ApplicationModuleSpec` | Explicit membership and ordering constraints | Schema contract collection, contribution configuration, installation catalog, inspection inventory |

No derived consequence remains a separately editable ordinary declaration. If
a value needs independent policy rather than mechanical derivation, its owner is
named explicitly in the semantic definition.

Output effects are not restated in the operation. The installed operation
envelope combines the operation's decision reads and explicit non-output
effects with the output protocol's derived effect envelope. Invariant reads
derive from invariant dependency paths and are not copied into the operation.
`ApplicationFeatureSpec` is optional typed grouping syntax over these
definitions; it owns no additional copy of their facts. `ApplicationModuleSpec`
remains the sole owner of application membership.

### Authoring and lowering form

The first implementation uses ordinary Rust types plus exported declarative
macros in the existing Query declaration crates. It does not add a proc-macro
package, build script, filesystem scan, package manifest language, generated
source tree, or new package manager.

Macros exist only where Rust must emit implementations or associated types.
The semantic definitions themselves remain typed values that can be
canonicalized, validated, inspected, and tested. Syntax compression without
fact ownership does not satisfy this milestone.

The pipeline is:

```text
Authored application definitions
    -> canonical semantic program
    -> validated application program
    -> installed runtime contracts
    -> existing Query execution owners
```

Canonicalization normalizes order, resolves explicit typed references, and
computes stable identities. Validation rejects missing membership, duplicate or
conflicting facts, impossible lifecycle combinations, invalid dependency paths,
and mismatched output/resource contracts. Installation lowers the validated
program once. Execution consumes installed contracts and does not re-canonicalize
or rediscover authored meaning.

### Output protocols

`ApplicationOutputProtocol` describes a structural language, not necessarily a
static list. It has two postures under one public contract:

- `FixedOutputProtocol` expands a domain-owned typed generator into an exact
  manifest. Exact cardinality, output roles, correspondence, and the structural
  part of candidate requirements are consequences of that manifest.
- `GeneratedOutputProtocol` declares a typed authority envelope, structural
  grammar, semantic role and lineage contracts, and a bounded resource policy.
  Each execution produces a deterministic realization manifest whose exact
  cardinality is known only after governed computation.

Fixed and generated protocols share canonical role identity, manifest
validation, correspondence, candidate admission, invariant progression, and
publication. They are not separate mutation APIs.

Query's generic structural validation covers envelope containment, declared
entity/relation kinds, internal-reference closure, role uniqueness and entity
affinity, cardinality, and lineage presence. A domain-specific protocol may
bind an additional structural validator, but CAD remains the owner of BREP,
orientation, trim, tolerance, and manifold meaning. Query schedules that
validator under installed authority and work bounds; it does not interpret a
CAD grammar itself.

A domain protocol validator receives only the proposed effect/manifest view,
its declared source and prior-output projections, and installed policy values.
Every prerequisite field or relation must already be named by the producing
operation or an invariant dependency path. The validator cannot perform ambient
graph reads, discover dependencies, or widen its candidate scope.

`ApplicationSemanticRole` is a domain-owned typed value. Its canonical
serialization is stable descriptive identity; the string never substitutes for
the typed role or its entity affinity. CAD may define values such as preserved,
split, intersection, and blend roles with typed source identities and stable
component keys. Query treats their domain payload opaquely while validating
canonical uniqueness, entity affinity, and declared lineage requirements.

### One proposed delta

The realization manifest is not a second graph delta. A protocol-bound candidate
writer appends each existing Query application effect and its role/lineage
metadata as one operation. The finished proposal owns the existing effect
program and exposes its manifest as an immutable projection. It is impossible
to edit the effects and manifest independently.

The governed progression is:

```text
installed operation + admitted request
    -> reserved candidate resources
    -> proposed application graph delta
    -> protocol-validated delta
    -> isolated owner candidate
    -> invariant-validated candidate
    -> coordinated World publication
```

The manifest grants no authority. Installation supplies the maximum authority
envelope; request admission and candidate reservation narrow it. The writer can
propose only effects within the compile-time operation authority, and
finalization verifies the actual proposal is contained by the installed and
admitted envelopes before owner candidate application.

The new typestate names may wrap or refine existing effect-program and candidate
types. Implementation must reuse those owners rather than introduce a parallel
transaction, publication, graph, or candidate engine.

### Resources and cardinality

Fixed protocols derive exact structural quantities. Policy quantities such as
retained bytes, invariant work, deadlines, and numerical work remain explicit.

Generated protocols declare a hard installed `ApplicationResourceEnvelope` and
may provide an input-dependent estimator that requests a tighter reservation.
The estimator cannot widen the installed envelope. Failure to estimate within a
bounded contract denies before construction. Actual effects, bytes, and work are
charged while the proposal is built; any overrun denies and releases the
reservation. Background work cannot hide unbounded proposal state.

### Derivation, readiness, and lifecycle

An `ApplicationDerivationSpec` owns the Cartesian relationship currently
repeated across producer and readiness types:

```text
producer trigger/source product x applicability variant x lifecycle mode
    -> exact producing ApplicationOperationSpec
    -> that operation's source expectation
    -> that operation's output protocol
```

Producer identity, readiness identity, conditional identity, applicability,
required invariants, reuse posture, and publication meaning derive from that
relationship unless the semantic definition supplies an explicit policy.
The producer trigger/source product is derivation meaning; the selected-query or
occurrence expectation used by execution remains operation meaning. Every
expanded route contains one typed producing-operation reference, so the latter
is derived rather than restated.
Installation rejects a missing operation, an operation outside the enclosing
module, a source/protocol mismatch, or two operations claiming the same route.
Initial and preserve lifecycles remain different typed modes. Generation may
remove repeated wiring; it may not erase their different source, lineage,
correspondence, denial, or resource semantics.

### Invariant dependencies

`ApplicationInvariantSpec` declares dependencies as bounded typed paths. A path
names each entity, field or relation hop, direction, relevant cardinality, and
the scope it contributes. Query derives portable descriptive targets and
installed candidate reads from those typed paths.

The impact graph is compiled at installation. Candidate execution starts from
the actual touched set and traverses only the installed bounded paths needed by
the affected invariants. A path that cannot state an honest fan-out or work
bound is rejected or requires an explicitly broader resource policy. Query does
not infer dependencies from handler code and does not perform schema-wide
searches to compensate for an incomplete declaration.

### Module membership and inspection

`ApplicationModuleSpec` is the one explicit membership list. It derives the
contract collection and contribution configuration used by installation.
Registration remains explicit Rust composition; no filesystem or Cargo package
discovery is introduced.

Validated and installed programs expose a semantic explanation API showing the
authored fact, canonical identity, derived contracts, authority envelope,
output posture, resource posture, dependencies, producer/readiness routes, and
module membership. Explanation is a cold or diagnostic projection and adds no
per-request reconstruction, logging, or allocation.

This milestone exposes the library-level explanation model and one executable
example. It does not require a new general-purpose CLI framework before the
authoring cutover can ship.

## Required Public DX

Exact syntax may change during implementation planning, but the semantic
ownership and amount of authored information are fixed. A fixed family must be
expressible at approximately this level:

```rust
worth_query_application_feature! {
    pub ExtrusionRealization<Schema: CadSchemaBinding> {
        operations: [
            RealizeExtrusion {
                input: RealizeExtrusionInputBinding,
                result: RealizeExtrusionResultBinding,
                denial: RealizeExtrusionDenialBinding,
                scope: ExtrusionFeature by ExtrusionFeatureKey,
                source: ExtrusionReadQuery,
                reads: extrusion_source_reads(),
                output: ExtrudedProfileProtocol,
                emits: [BodySetPublished],
                handler: RealizeExtrusionHandler,
                resources: ExtrusionResourcePolicy,
            },
            PreserveExtrusion {
                input: PreserveExtrusionInputBinding,
                result: PreserveExtrusionResultBinding,
                denial: PreserveExtrusionDenialBinding,
                scope: ExtrusionFeature by ExtrusionFeatureKey,
                source: PriorExtrusionReadQuery,
                reads: preserved_extrusion_source_reads(),
                output: ExtrudedProfileProtocol,
                emits: [BodySetPublished],
                handler: PreserveExtrusionHandler,
                resources: ExtrusionPreservationResourcePolicy,
            },
        ],
        derivation: {
            variants: [Triangle, Rectangle, Pentagon],
            routes: [
                Initial => RealizeExtrusion,
                Preserve => PreserveExtrusion,
            ],
            invariants: [
                LoopCycleIntegrity,
                AnalyticBindingIntegrity,
                PlanarTrimIntegrity,
                ShellManifoldIntegrity,
            ],
        },
    }
}
```

The CAD-owned protocol describes topology algebraically and returns typed roles;
it does not author 67, 85, or 103-entry string arrays. Adding `Pentagon` must not
require a new Query binding, manual schema authority inventory, producer,
readiness route, or hand-counted candidate ceiling.

Membership remains explicit once:

```rust
worth_query_application_module! {
    pub CadContribution {
        features: [ExtrusionRealization],
        invariants: [
            LoopCycleIntegrity,
            AnalyticBindingIntegrity,
            PlanarTrimIntegrity,
            ShellManifoldIntegrity,
        ],
    }
}
```

A generated protocol must be expressible without listing future instances:

```rust
impl GeneratedOutputProtocol<CadSchema> for BooleanResultProtocol {
    type Role = BooleanTopologyRole;
    type Lineage = BooleanTopologyLineage;

    fn authority_envelope() -> CadTopologyAuthorityEnvelope { /* typed kinds */ }
    fn structure() -> CadStructuralValidatorBinding { /* domain-owned */ }
    fn resources() -> BooleanResourcePolicy { /* hard bounds + estimator */ }
}
```

The handler receives a protocol-bound writer and proposes actual topology with
typed roles and lineage. It never receives generic graph authority.

Invalid authoring errors name the semantic definition, conflicting fact,
derived obligation, and correction. Runtime denials distinguish authority,
structure, lineage, resource, and invariant failure without exposing internal
registry mechanics.

## Authority And Truth Ownership

| Product | Constructor and truth owner | What it proves | What it cannot authorize |
| --- | --- | --- | --- |
| Authored application definition | Domain entry crate | Claimed semantic intent and explicit membership | Installation validity or execution |
| Canonical application program | Query declaration canonicalizer | Stable normalized meaning and identity | Runtime authority or current policy |
| Validated application program | Query declaration/installation validation | Internal semantic closure and deterministic lowering | Principal or World authority |
| Installed operation envelope | Query installation under actual schema | Maximum permitted operation effects, protocol, handlers, dependencies, and resources | A particular request or candidate |
| Candidate reservation | Existing Query/owner resource admission | Bounded capacity for one admitted attempt | Effects beyond its narrowed envelope |
| Proposed application graph delta | Protocol-bound writer | One immutable effect program with role/lineage projection | Structural validity, commit, or publication |
| Protocol-validated delta | Query protocol validator | Actual proposal fits authority, structure, lineage, and resource contracts | Domain invariant truth |
| Invariant-validated candidate | Existing Relational/Query candidate owners | Required candidate predicates passed under concrete authority | World publication or future reuse |
| Published occurrence | Existing Runtime World owner | Coordinated authoritative graph occurrence | New admission or inferred currentness |
| Explanation | Derived from canonical/installed contracts | Why obligations exist and how they lowered | Disclosure or execution authority |

Concrete platform authority remains proof-carrying. Portable authoring records,
canonical identities, manifests, roles, lineage descriptions, and explanation
records are descriptive and mint no capability.

## Replacement And Cutover Contract

This milestone provides no compatibility API and leaves no manual/generated
ordinary pair. Each phase may use the frozen current contract as a temporary
test oracle inside the implementation branch. Before that phase is certified,
the migrated scope must:

1. lower exclusively from the new semantic definition;
2. delete the superseded manual trait, schema, binding, output, producer,
   readiness, invariant-target, or module-registration declaration;
3. remove its exports, documentation, fixtures, and feature switches;
4. prove no ordinary call site can select the old route.

Equivalence evidence compares normalized behavior and installed meaning at the
cutover boundary. It is not a promise to preserve an old public API. Tests must
describe certified behavior rather than naming a permanent old/new pairing.

Existing runtime contracts remain because they are the lowered execution
representation. They cease to be application-authored inputs where this
milestone assigns their fact to a semantic definition.

## Destination Directory And Module Skeleton

Paths are under `workspaces/worth-query/crates/` unless another root is shown.
`E` means existing, `N` new, `R` refactored or replaced, `D` removed, and `S`
marks a committed successor destination that is not created empty.

```text
worth-query-declaration/src/
  application_program/                         N: semantic authoring owner
    authored/
      operation.rs                             N: ApplicationOperationSpec
      output_protocol/{definition,fixed,generated}.rs N
      derivation.rs                            N: source/variant/lifecycle meaning
      invariant/{definition,dependency_path}.rs N
      module.rs                                N: explicit membership
    canonical/
      program.rs                               N: normalized program root
      operation.rs                             N
      output_protocol/{definition,manifest}.rs N
      derivation.rs                            N
      invariant.rs                             N
      module.rs                                N
    validation/
      identity.rs                              N: collisions and stable identity
      semantic_closure.rs                      N: reference/membership closure
      lifecycle.rs                             N: derivation compatibility
      output_protocol.rs                       N: static protocol consistency
      dependency_path.rs                       N: typed path validity/bounds
    explanation.rs                            N: derived cold inspection model
  application_operation_spec_macro.rs          N: emits required Rust types/impls
  application_feature_spec_macro.rs            N: family/module composition syntax
  application_operation/                       E/R: lowered contract vocabulary
  application_schema/                          E/R: consumes derived registration
  application_query/                           E: existing query authoring pipeline

worth-query-installation/src/
  application_program/
    compilation.rs                             N: validated program -> installed contracts
    operation.rs                               N: authority/binding lowering
    output_protocol/{fixed,generated}.rs        N
    derivation/{producer,readiness}.rs          N
    invariant/{dependencies,impact_graph}.rs    N
    module.rs                                  N: one membership lowering
    explanation.rs                            N: installed projection
  application_operation/                       E/R: receives compiled contracts
  application_schema/                          E/R: receives derived membership

worth-query-execution/src/domain_computation/primary_graph/
  application_attempt/
    effect_program/                            E/R: sole proposed-effect owner
    output_protocol/
      proposal/{writer,manifest,role,lineage}.rs N
      validation/{authority,structure,lineage,resources}.rs N
      denial.rs                                N: exact protocol failures
    provider_binding/                          E/R: consumes protocol-validated delta
  application_invariant/                       E/R: compiled dependency execution
  application_contribution/                    E/R: derived producers/readiness
  application_installation/                    E/R: one compiled module root
  provider/                                    E/R: existing candidate/resource owner

worth-query-decl/src/facade.rs                 R: semantic authoring exports
worth-query-host/src/facade.rs                 R: installed/runtime audience only
worth-query-certification/
  fixtures/consumer_entry/topology_entry/src/
    application_program.rs                     N/R: fixed + generated reference consumers
    output_protocol/{extruded_profile,split_family}.rs N
    invariants.rs                              R: typed dependency declarations
  tests/application_authoring.rs               N: one integration target/family

C:/forge_workspace/worth-proprietary/crates/worth-cad-entry/src/
  families/extrusion/realization/
    application_program.rs                     N: extrusion feature definition
    output_protocol/
      extruded_profile.rs                      N: algebraic fixed-profile topology
      role.rs                                  N: typed semantic topology roles
    derivation.rs                              N: variant/lifecycle declaration
    handler/                                   E/R: domain decision and output population
    declaration.rs                             D: repeated operation/schema inventory
    binding.rs                                 D/R: retain domain intent only where needed
    output_roles/{rectangle,triangle}.rs        D
    [parallel producer/readiness declarations] D
  invariants/
    application_program.rs                     N/R: typed dependencies + predicates
  contribution.rs                              R: one ApplicationModuleSpec membership
```

Declaration meaning, installation lowering, runtime proposal custody, and
domain topology are the dominant axes. Canonical and installed forms are
derived truth downstream of authored meaning. Runtime proposal state is live
custody and remains in execution. CAD's topology grammar cannot move into Query;
Query's authority, resource, and publication progression cannot move into CAD.

No `common`, `helpers`, `utils`, generic `compiler`, generated-source, package-
discovery, or cross-owner manager bucket is permitted. If implementation shows
that an existing file already owns a listed responsibility under a precise
name, extend or refine that owner instead of creating a duplicate module.

## Ordered Phase Plan

Each phase begins with `plan-implementation`, including the real owner boundary,
public DX, compile-cost posture, and exact directory population. Use
`implementation-batch` for coherent vertical slices. Certify and commit each
phase before the next. Reuse compiled binaries and accepted evidence; rerun only
checks invalidated by later changes.

### Phase 1: Operation Definition, Minimal Fixed Protocol, And Single Lowering

Establish the authored/canonical/validated/installed program pipeline and
`ApplicationOperationSpec` together with the smallest fixed output-protocol
contract needed to own rectangle output effects. Cut the public topology fixture
and CAD rectangle initial operation completely to them. The operation's declared
reads/non-output effects plus the protocol's output effects must emit the
required trait projections, schema registration, binding metadata, exact fixed
manifest/cardinality, and installed envelope. Delete the migrated manual
declarations.

This phase closes only when changing an authored read/non-output effect or a
protocol-owned output effect changes every derived projection, conflicting or
missing facts fail before installation, the existing real rectangle journey
passes, and no migrated call site can reach the old authoring route.

### Phase 2: Fixed Output Protocol And Typed Roles

Complete fixed `ApplicationOutputProtocol` expansion, typed semantic
roles, derived exact cardinality, output correspondence, and protocol-bound
writing. Express extrusion as an algebraic profile family; cut rectangle and
triangle initial/preserve output inventories to it. Prove a pentagon fixed
manifest in the protocol's focused tests without yet installing a pentagon
producer route. Delete hand-written 85/67 role arrays and independent structural
counts in the migrated family.

This phase closes when fixed create and preserve manifests are deterministic,
wrong entity affinity is denied, counts and correspondence cannot diverge, and
the pentagon protocol expansion requires no copied output inventory or count.

### Phase 3: Generated Output Protocol And Governed Manifest

Add generated protocols, resource envelopes and estimators, semantic lineage,
the protocol-bound proposed-delta typestate, and structural validation integrated
with the existing candidate pipeline. Prove it with the variable-cardinality
split family through real publication, retained read, denial, cleanup, and
sibling progress.

This phase closes when unauthorized, structurally invalid, ambiguous-lineage,
underestimated, and over-budget proposals cannot reach publication; valid
variable output publishes atomically without a second delta or candidate owner.

### Phase 4: Derivation Families And Runtime Wiring

Add `ApplicationDerivationSpec` and lower variant/lifecycle relationships into
producer, readiness, conditional, and publication contracts. Cut the extrusion
rectangle/triangle initial/preserve Cartesian product and the matching public
fixture. Install pentagon as the anti-copy acceptance case. Preserve lifecycle-
specific semantics while deleting parallel wiring.

This phase closes when each expanded route resolves one exact typed producing
operation, producer and readiness identities are deterministic, source/
operation/protocol/applicability/invariant mismatches fail installation, and
adding a fixed profile variant changes only domain profile semantics and family
membership.

### Phase 5: Invariant Dependencies And Impact Compilation

Add `ApplicationInvariantSpec`, bounded typed dependency paths, installation-
compiled impact graphs, and derived portable targets/access. Cut one local
invariant and one transitive CAD topology invariant, then migrate the remaining
Pre-M0 CAD invariants in coherent semantic families. Delete their independent
string target and traversal declarations.

This phase closes when touched-scope validation reaches the exact required
neighborhood, misses no affected entity, performs no schema-wide fallback, and
rejects absent hops, wrong directions, and unbounded lawful fan-out honestly.

### Phase 6: Module Membership, Explanation, And Closure

Add `ApplicationModuleSpec`, derive contract/configuration membership, expose
canonical and installed explanation, and cut all scoped public fixture and CAD
registration to the one module definition. Remove superseded exports, docs,
macros, fixtures, flags, and ordinary paths.

Run the complete fixed/generated courts, existing Pre-M0 public journey,
affected Bank regression, and actual proprietary CAD extrusion journey. Measure
cold and warm builds at declaration, entry, and consumer touch boundaries.
Update the roadmap and durable developer documentation to teach only the new
authoring model once it is real.

## Documentation Deliverables

| Audience | Authoritative document | Required change |
| --- | --- | --- |
| Query application authors | `workspaces/worth-query/docs/capabilities/declarative-query-experience.md` | Show operation, output protocol, derivation, invariant, module, denial, and explanation workflows from the real facade |
| Query architecture maintainers | `AI_README.md` and this specification | Explain authored, canonical, installed, and live runtime ownership using normal reference language and real paths |
| CAD contributors | `C:/forge_workspace/worth-proprietary/AI_README.md` and M0 plan | Show the extrusion program, fixed/generated protocol choice, typed roles, lineage, and handler boundary |
| Successor implementers | `WORTH_query_roadmap.md`, 9.17.4, and 9.18 | State the authoring prerequisite and contracts successors may consume |

Public snippets compile against the actual facade. Do not document proposed
types as shipped before their phase lands. Do not put migration history or
statements about removed APIs into either `AI_README.md`; those files remain
normal current-reference documents.

## Acceptance, QA, And Cost Discipline

Architecture review must confirm one owner per semantic fact, no authority from
descriptive manifests, no second delta/candidate/publication lane, and complete
deletion of each migrated manual route. DX review must confirm that fixed and
generated cases expose real semantic choices while keeping authority, resources,
lineage, lifecycle, and failures visible.

Focused canonicalization tests cover determinism, identity collision, semantic
closure, and invalid lifecycle composition. Compile-pass/fail cases are grouped
into the existing fixture targets and used only for valuable public type
boundaries. Runtime integration uses one public application-authoring target,
not one crate or binary per negative case. Existing candidate, invariant,
publication, retained-read, sibling-progress, and cleanup tests remain the
primary runtime evidence where their boundary is unchanged.

The fixed court must prove exact derived topology and the pentagon edit radius.
The generated court must prove variable cardinality, envelope containment,
typed role/entity affinity, lineage, estimator underflow, runtime overrun,
structural denial, invariant denial, atomic publication, and cleanup. The
private CAD journey proves the abstraction reduces real consumer work; public
certification never fabricates proprietary geometry authority.

Measure:

- cold compilation of affected Query declaration/installation/execution and
  the shared certification consumer;
- warm rebuild after touching only a semantic application declaration;
- warm rebuild after touching one domain output protocol;
- runtime work by proposed items, relations, retained bytes, dependency hops,
  invariant work, candidate applications, and publications.

Canonicalization, lowering, explanation, and full program validation are cold
installation work. The ordinary request path performs direct installed lookup,
resource admission, actual proposal charging, required candidate validation,
and publication only. It gains no schema-wide scan, source generation, dynamic
reflection, diagnostic construction, or repeated canonicalization.

For boundary-relevant implementation, run the repository's boundary and agent-
context checks, affected formatting and owner/integration tests, and the dirty
Rust line-cap guard. Preserve the requested quick line-check CI posture; this
milestone does not reinstate the old CI pipeline. Reuse compiled artifacts and
rerun expensive courts only when their boundary changes or final certification
requires them.

Review substantial batches. Separate certification blockers from optional
suggestions, preserve cleared findings and valid evidence, and reopen them only
when changed code or new evidence warrants it. A repeated failure at the same
boundary requires replanning the boundary and completing the replacement design
rather than adding another adapter.

## Must Ship And Must Preserve

The milestone ships:

- one typed authored-to-installed application program pipeline;
- single-source operation authority and binding;
- fixed and generated output protocols;
- typed semantic roles and lineage;
- one immutable proposed effect program with a manifest projection;
- fixed exact and generated bounded resource admission;
- derivation-generated producer/readiness wiring;
- typed invariant dependency paths and compiled affected-scope plans;
- explicit module membership and derived installation;
- cold explanation and provenance from each derived contract to its authored
  fact;
- complete cutover and deletion for the scoped public and CAD consumers.

It preserves:

- the certified 9.17.3 World/Signal/Relational/Bridge authority model;
- 9.17.4's concrete proof-carrying application request and candidate pipeline;
- typed runtime contracts, exact denials, idempotency, recovery, retained reads,
  output correspondence, and atomic coordinated publication;
- Query-free domain values and explicit entry composition;
- owner-local invariant truth and CAD-owned BREP/geometry semantics;
- bounded ordinary work and independent sibling progress;
- certification-only replay and existing lower-owner reconstruction boundaries.

## Successor Handoff

After 9.17.4.1, new application operations enter through
`ApplicationOperationSpec`; output-producing operations choose a fixed or
generated `ApplicationOutputProtocol`; producer/readiness families enter through
`ApplicationDerivationSpec`; invariants enter through typed dependency paths;
and application membership enters once through `ApplicationModuleSpec`.

The remaining 9.17.4 consumer migration uses these definitions rather than
adding more manual contracts. CAD M0 and later NURBS/BREP operators may add
domain protocols, roles, lineage, estimators, and predicates without changing
Query's authority or publication model. 9.18 adds correction semantics over the
same installed operations and published lineage. Later Query access and
correlated-execution milestones extend canonical operation meaning without
introducing another ordinary authoring language.

This milestone closes only when semantic complexity remains explicit, every
derived obligation has one authored cause, variable topology remains bounded
and explainable, and the old authoring path is absent from every migrated scope.
