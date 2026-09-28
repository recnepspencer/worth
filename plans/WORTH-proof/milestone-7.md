# Milestone 7 Engineering Spec: Certification And Cross-Crate Migration Closure

> **Status:** Planned
>
> **Roadmap parent:** [plans/worth-proof/worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [plans/worth-proof/worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [plans/worth-proof/test-requirements.md](./test-requirements.md)
>
> **Adjacent milestone:** [plans/worth-proof/milestone-6.md](./milestone-6.md)
>
> **Adjacent milestone closeout:** [plans/worth-proof/milestone-6-closeout.md](./milestone-6-closeout.md)
>
> **Primary architectural driver:** prove that `worth-proof` is actually
> migration-worthy for real WORTH flows rather than only locally elegant inside
> its own crate

## Goal

Prove that `worth-proof` is genuinely fit to replace bespoke progression
machinery in real WORTH crates by closing hostile cross-crate migration,
semantic-parity, and hot-path-honesty pressure.

## Why This Milestone Exists

Milestones 1 through 6 made the proof substrate internally coherent:

- phase-typed artifacts and proof-set composition are canonical
- proof minting and witness authority are sealed
- freshness, re-admission, and downgrade boundaries are explicit
- transitions and failure topology are typed
- lowered, ready, and executed forms are distinct
- fixed-arity fork/join and deterministic same-family lowering are canonical

That still leaves the final capstone question unanswered:

- does `worth-proof` actually survive contact with real WORTH code?

Without Milestone 7:

- `worth-proof` can look rigorous in isolation while still missing real
  migration pressure from `worth-signal`, `worth-relational`, `worth-query`,
  or `worth-store`
- hot-path zero-cost claims can remain representative but not migration-grade
- crates can continue carrying bespoke progression law because the shared
  substrate has never been forced to preserve their real semantics
- later engineers can mistake good local substrate tests for proof that the
  platform-level replacement is safe

Milestone 7 therefore exists to solve the last shared-law problem:

- migrate representative proof-heavy surfaces from real crates
- prove semantic parity instead of assuming it
- prove hot-path honesty against bespoke baselines instead of representative
  internal wrappers only
- leave explicit residual debt for whatever still cannot migrate honestly

## Hard Part

The hard part is not wiring `worth-proof` into one demo migration.

The hard part is preserving all of these at once:

- exact domain semantics across several different crates
- exact failure topology across migrated and non-migrated lanes
- exact stale, basis, authority, and trust-boundary behavior
- exact cost honesty on representative hot paths
- clean separation between proof-generic migration support and crate-specific
  migration adapters
- explicit residual debt instead of silent "close enough" migration claims

The design fails if:

- a migrated lane preserves type shape but changes semantics
- a migrated lane preserves success behavior but collapses denial topology
- a migration helper hides crate-specific pressure behind fake-generic support
- zero-cost claims remain internal-representative only and never face real
  crate-local hot paths
- the milestone closes by certifying one crate while leaving the roadmap claim
  of broad migration readiness implied rather than proven

## Explicit Assumptions

- Milestones 1 through 6 and their closeouts remain authoritative.
- `worth-proof` still owns proof-bearing progression law only; it does not take
  ownership of diagnostics, lineage, provenance, storage layout, or runtime
  execution policy from the migrating crates.
- `worth-foundational` remains the shared truth-vocabulary home; Milestone 7
  does not reopen that split.
- migration here means shared progression law replaces bespoke progression
  mechanics, not that all local crate semantics become generic.
- the migrated reference lanes may be representative rather than exhaustive,
  but they must be hard enough that later migration work can rely on them as
  real substrate proof.

## Governing Summaries

- `MENTALITY.md`
  The main protection here is refusing to mistake local cleanliness for
  platform readiness. Milestone 7 must solve the hostile migration problem
  directly before `worth-proof` is treated as finished infrastructure.
- `arch_laws.md`
  The main protection here is that migrated lanes must preserve exact
  proof-carrying meaning, exact phase ordering, exact denial topology, and
  exact facade boundaries. Laws 9, 20, 26, 29, 30, 32, 37, 40, and 41 apply
  most strongly.
- `perf_laws.md`
  The main protection here is that migration claims must be cost-honest at the
  real boundary being claimed, not only through internal wrapper examples.
  Boundary honesty, explicit equivalence contracts, and named measurement
  surfaces govern this milestone.
- `domain_laws.md`
  The main protection here is honest decomposition of migration harnesses and
  adapters. Proof-generic mechanics and crate-specific migration pressure must
  not collapse into one blurry support bucket.
- `worth_proof_vision.md`
  The main protection here is the ownership boundary: the crate must prove it
  can replace progression mechanics without turning into a second domain
  runtime.
- `worth_proof_roadmap.md`
  The main protection here is that Milestone 7 is the capstone proof of
  migration-worthiness, not another substrate feature milestone.
- `worth-proof` test requirements
  The main protection here is that Milestone 7 closes only through the named
  `Cross-Crate Migration And Hot-Path Honesty Test`, with real migration
  parity, hostile misuse lanes, and codegen or counter honesty.
- `milestone-6.md`
  The main protection here is that static fork/join and same-family lowering
  are already canonical before migration adapters consume them.
- `milestone-6-closeout.md`
  The main protection here is what Milestone 7 may assume: canonical
  fixed-arity composition, deterministic family lowering, and same-family
  symbolic/non-authoritative boundaries already exist and are certified.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several proof-heavy WORTH flows from different crates with different basis
> rules, denial topology, and hot-path execution shapes must migrate onto
> `worth-proof` so that semantic parity remains exact, stale and authority
> boundaries remain explicit, and the migrated hot paths introduce no hidden
> allocation, dynamic lookup, or dispatch relative to the bespoke baselines
> they replace.

The design fails if:

- a migrated lane can pass by preserving success output while changing failure
  or stale behavior
- migrated hot-path claims are not backed by named representative codegen or
  counter evidence
- one migration helper secretly bakes in crate-specific pressure while
  pretending to be proof-generic
- `worth-proof` adoption requires consumers to reach around the facade into
  internal modules
- the milestone closes without an explicit residual-debt inventory for
  remaining bespoke progression machinery

## Product Decision Lock

- Milestone 7 is a migration-proof milestone, not a feature-addition milestone
- at least three representative crate families must be migrated:
  - one from `worth-signal`
  - one from `worth-relational`
  - one from `worth-query` or `worth-store`
- the selected families must not all be low-pressure wrapper migrations
- at least one migrated family must materially pressure basis or staleness law
- at least one migrated family must materially pressure failure topology
- at least one migrated family must materially pressure hot-path honesty
- migration parity must preserve semantics, basis or staleness behavior, and
  failure topology, not only public success shape
- hot-path honesty must be certified at the migrated-lane boundary, not only at
  internal representative wrapper boundaries
- proof-generic migration harness logic and crate-specific migration adapters
  must stay structurally separate
- any remaining non-migrated progression law must be named as explicit debt,
  not implied completeness

Normative consequence:

- any migration that only rewrites type wrappers without parity proof is out of
  spec
- any migration support tree that hides domain pressure inside fake-generic
  helpers is out of spec
- any milestone closeout that claims platform readiness without explicit
  residual-debt inventory is out of spec

## Required Contracts

### Migration-Target Selection Rule

Milestone 7 must select families that actually pressure the substrate rather
than cherry-picking trivial migrations.

Required vocabulary:

- selected migration family
- selection rationale
- baseline semantic pressure
- baseline hot-path claim boundary

Rules:

- each selected family must name the proof-bearing law it exercises
- at least one selected family must materially exercise basis or stale law
- at least one selected family must materially exercise non-trivial failure
  topology
- at least one selected family must materially exercise a real hot-path claim
- helper-only, wrapper-only, or facade-only migrations do not satisfy the
  milestone by themselves

### Migration-Parity Rule

A migrated lane must preserve the real semantics of the bespoke progression
surface it replaces.

Required vocabulary:

- bespoke baseline lane
- migrated `worth-proof` lane
- semantic parity report
- explicit divergence lane

Rules:

- parity must be proven against independently produced baseline and migrated
  lanes
- parity must cover phase ordering, proof-bearing output meaning, and
  failure, stale, or trust-boundary topology where those are part of the
  contract
- if semantics intentionally differ, that divergence must be named and
  certified rather than hidden

### Hot-Path Honesty Rule

Any migration claim that names zero-cost or hot-path honesty must prove it at
the migrated boundary, not only through representative internal wrappers.

Required vocabulary:

- representative migrated hot path
- baseline cost surface
- migrated cost surface
- codegen or counter honesty report

Rules:

- the measurement boundary must match the claim boundary
- hidden allocation, dynamic lookup, or virtual dispatch must be forbidden in
  the certified representative scope
- if counters rather than codegen are used, the counters must explain the work
  performed structurally

### Migration-Adapter Separation Rule

The migration harness must distinguish proof-generic mechanics from
crate-specific domain pressure.

Required vocabulary:

- generic migration proof mechanics
- crate-specific migration adapter
- migrated fixture authority
- parity assertion surface

Rules:

- proof-generic harness logic must live in generic migration support
- crate-specific setup, naming, and semantic adaptation must live in explicit
  crate-specific homes
- no fake-generic helper may smuggle `signal`, `relational`, `query`, or
  `store` domain pressure

### Ownership-Routing Rule

Milestone 7 must leave behind clearer ownership boundaries, not wider crate
ambiguity.

Required vocabulary:

- proof-owned progression law
- foundational-owned boundary vocabulary
- domain-owned semantics or runtime behavior
- migration routing note

Rules:

- each migrated family must record which parts legitimately moved into
  `worth-proof`
- each migrated family must record which parts stay in `worth-foundational`
  or the domain crate
- no migration may "succeed" by silently moving diagnostics, lineage,
  provenance, storage, or runtime policy into `worth-proof`

### Residual-Debt Closure Rule

Anything still left outside `worth-proof` after Milestone 7 must be named
explicitly.

Required vocabulary:

- residual bespoke progression inventory
- non-migrated reason
- future migration gate

Rules:

- every intentionally non-migrated progression family must be inventoried
- the reason must distinguish "not yet worth migrating" from
  "architecturally blocked"
- later engineers must not have to rediscover why bespoke machinery remains
  after Milestone 7

## Scope

### In Scope

- reference migrations for at least three representative proof-heavy crate
  families
- migrated parity proof between bespoke and `worth-proof` lanes
- migrated hot-path honesty proof through named codegen or structural counter
  surfaces
- generic migration support ownership for parity, compile-fail, and
  codegen/counter proof mechanics
- explicit crate-specific migration adapters
- explicit residual-debt inventory for remaining bespoke progression families
- milestone-local routing guidance for when a remaining surface belongs in
  `worth-proof`, `worth-foundational`, or a domain crate

### Explicitly Out Of Scope

- migrating every proof-bearing surface in WORTH
- expanding `worth-proof` into diagnostics, lineage, provenance, or storage
  ownership
- adding new progression substrate features unless a migration gap proves the
  existing substrate is insufficient
- cross-language or runtime-plugin portability
- hiding unresolved migration pressure under "future cleanup"

## Phases

### Phase 1: Migration Target Selection And Baseline Capture

Select the representative crate families and freeze the bespoke baseline
surfaces before migration begins.

Must ship:

- one named migration family from `worth-signal`
- one named migration family from `worth-relational`
- one named migration family from `worth-query` or `worth-store`
- a short selection rationale for each family
- explicit baseline semantic contracts for each selected family
- explicit baseline failure, stale, authority, or trust-boundary topology for
  each selected family where applicable
- explicit baseline hot-path boundary or cost surface for each selected family

Implementation guidance:

- choose families that genuinely pressure different parts of the substrate
- do not choose three easy migrations just because they are faster
- freeze the baseline before writing migrated adapters so parity has a real
  comparison surface
- if a family cannot state a baseline contract clearly, it is not ready to be
  a certification lane yet

### Phase 2: Generic Migration Harness And Crate-Specific Adapters

Build the migration test architecture before migrating the selected lanes.

Must ship:

- generic support for parity comparison
- generic support for hostile compile-fail and misuse-lane execution
- generic support for codegen or counter-honesty capture
- explicit crate-specific adapter homes for selected families
- an explicit support split that makes proof-generic harness logic and
  crate-specific adapters impossible to confuse

Implementation guidance:

- separate generic proof mechanics from crate-specific semantics immediately
- keep the migration support tree small but future-shaped
- do not bury real migration logic in the root certification test
- do not let adapter code become a second substrate layer

### Phase 3: Migrated Lane Parity And Hostile Misuse Closure

Migrate the representative lanes and prove semantic parity plus hostile
boundary behavior.

Must ship:

- migrated `worth-proof` versions of the selected reference lanes
- exact semantic parity or explicit divergence proof
- hostile stale, basis, authority, trust-boundary, or phase-misuse coverage
  for migrated lanes
- explicit compile-fail coverage where the migrated lane claims compiler
  enforcement
- per-family routing notes recording what legitimately moved into
  `worth-proof` and what stayed outside it

Implementation guidance:

- prove one migrated family at a time instead of one giant bundle
- preserve failure topology as aggressively as success shape
- if a migration cannot preserve semantics honestly, stop and name the gap
- do not hide a routing mistake by broadening `worth-proof` ownership

### Phase 4: Hot-Path Honesty And Residual-Debt Closure

Close the milestone with the named suite, migrated hot-path honesty proof, and
explicit residual-debt inventory.

Must ship:

- machine-checkable evidence for the `Cross-Crate Migration And Hot-Path
  Honesty Test`
- `migration_parity_report`
- `failure_digest`
- `compile_fail_bundle`
- `codegen_honesty_report`
- `residual_debt_report`
- a milestone closeout that records migrated families, routing decisions, and
  remaining bespoke progression debt

Implementation guidance:

- certify the smallest honest set of real migrated lanes rather than bloating
  the suite into fake broad migration coverage
- if one migrated lane needs counters and another needs codegen, say that
  explicitly instead of pretending one proof shape fits all
- leave Milestone 7 with named debt, not implied future work

## Acceptance Evidence

Milestone 7 is not complete until the named suite required by
[plans/worth-proof/test-requirements.md](./test-requirements.md)
passes with a machine-checkable certification bundle for:

- `migration_parity_report`
- `failure_digest`
- `compile_fail_bundle`
- `codegen_honesty_report`
- `residual_debt_report`

At minimum, the hostile closure surface must prove:

- one `worth-signal` migration family preserves semantic parity
- one `worth-relational` migration family preserves semantic parity
- one `worth-query` or `worth-store` migration family preserves semantic parity
- the chosen families are not all low-pressure wrapper migrations
- migrated lanes preserve failure topology rather than only success shape
- migrated stale, trust-boundary, or authority misuse fails explicitly
- migrated hot paths remain free of hidden allocation, dynamic lookup, and
  virtual dispatch within the certified scope
- the migration harness keeps proof-generic support and crate-specific adapters
  structurally separate
- routing guidance for `worth-proof` versus `worth-foundational` versus the
  domain crate is explicit and reviewable
- any remaining bespoke progression families are named explicitly with reasons

## Why This Belongs Here

Milestone 7 belongs after Milestone 6 because migration parity only becomes
honest once the substrate already has:

- sealed progression law
- stale and trust-boundary honesty
- typed transition and failure topology
- lowered-versus-ready execution boundaries
- static fixed-arity fork/join progression
- deterministic same-family lowering

If Milestone 7 were attempted earlier, it would either:

- force the migration harness to paper over missing substrate law, or
- certify parity against bespoke local stopgaps that later milestones would
  reinterpret anyway

This milestone exists specifically to prevent that regression and to close the
roadmap with a real answer to the only question that matters at the platform
level:

- can real WORTH crates replace bespoke progression machinery with this shared
  substrate without semantic drift or hidden cost cliffs?
