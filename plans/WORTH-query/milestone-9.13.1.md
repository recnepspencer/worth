# Milestone 9.13.1: Query Iteration Foundation

Status: Phases 1-3 closed on 2026-07-18. Phases 4-8 are open.

## Goal

Make Query iteration livable before Milestone 9.14 by removing obvious test
waste, dismantling the giant library-test binary, isolating cold certification,
and extracting the declaration and installation authorities as the first
permanent production packages.

Milestone 9.13.1 establishes a useful package-selection boundary now.
Milestone 9.13.2 completes the higher-quality authority decomposition and
deletes the remaining monolith.

## Why This Milestone Exists

The first cut removed 38 repeated trybuild harnesses and reduced warm compiler
certification from roughly 399 seconds to roughly 4 seconds. That was real, but
it exposed the next floor: `cargo test -p worth-query --lib` still compiles and
runs one approximately 2,981-test binary in roughly 118 seconds warm. A
production edit can also require roughly a minute merely to rebuild that test
binary. One package-validation case has independently consumed more than a
minute because it reconstructs broad state repeatedly.

That is not a sharding problem. `worth-query` manually injects dozens of
integration suites into its library test target, hundreds of local test
modules share the same package artifact, and cold certification still lives
beside ordinary behavior. Cargo cannot select semantic work that the package
and target graph do not separate.

This milestone therefore continues past the trybuild rescue. It makes the
obvious target and cost cuts first, creates the permanent cold certification
leaf, and extracts the two upstream production authorities that Milestone
9.14 will build upon. It does not wait for a comprehensive inventory and it
does not build a test-selection platform.

## Governing Summaries

- `MENTALITY.md`: solve the actual adversarial constraint first and use AI to
  accelerate judgment rather than manufacture code around an untested premise.
- `arch_laws.md`: autonomous authorities and cold certification need physical
  boundaries; a facade or test selector cannot substitute for them.
- `composition_laws.md`: the giant library-test binary and its manual suite
  aggregator collapse unrelated responsibilities into one compilation unit.
- `domain_structure_laws.md`: tests must falsify their production owner without
  constructing unrelated authorities; package and target topology are the
  proof of that ownership.
- `perf_laws.md`: structural breadth and repeated reconstruction dominate
  constant tuning; ordinary edits must not inherit certification or unrelated
  authority cost.
- `WORTH_query_roadmap.md`: 9.13.1 establishes the fast upstream foundation;
  9.13.2 completes the authority graph before 9.14.

## Adversarial Constraint

A change to Query declaration or installation meaning must compile and test
that authority without compiling the remaining monolith, publication, replay,
or certification. Ordinary Query behavior must not share one manually
assembled library-test binary with public journeys, exhaustive convergence
matrices, source audits, or cert-only reconstruction. One compiler invocation
must retain the selected compiler-owned denials without repeated harness setup.

No slice may begin with a repository-wide test or dependency inventory. Each
slice inventories only the boundary it will immediately change, removes or
extracts the structural cost, verifies that boundary once, and records one
before/after observation. An inventory with no same-slice deletion, move, or
package cut is not milestone progress.

## Product Decision Lock

- Retain a compile-fail fixture only when it uniquely protects authority
  minting, type or family substitution, phase ordering, move-only ownership, or
  a load-bearing facade denial. Its first compiler error must exercise the
  named invariant rather than an incidental missing import or stale API.
- Delete historical API tombstones, generic report/counter/manifest privacy
  probes, certification-topology fixtures, redundant per-type privacy probes,
  orphan baselines, and any fixture that fails before reaching its claimed
  boundary.
- Use Cargo and trybuild directly. Add no custom runner crate, inventory,
  manifest language, cache manager, compiler wrapper, receipt schema, shard
  scheduler, timing sampler, or migration framework.
- The direct trybuild target is compile-fail only. Positive facade journeys use
  ordinary integration tests or doctests so Cargo can compile them as normal
  test targets without trybuild's per-program rebuild loop.
- Production Query code may not own fixture paths, fixture counts, golden
  transcript registries, or compiler-test manifests. Test registration is test
  code and carries no product authority.
- Query owns Query behavior. Worth UI owns proof that Worth UI adopts Query.
- Verification exercises product tests and the real compiler portfolio. It
  does not add tests that inspect, count, replay, or certify other tests.
- Each remaining phase is a complete slice: narrow inventory, implementation,
  local verification, and one elapsed observation. No separate audit or
  inventory phase may be inserted ahead of the obvious work.
- Test placement has four destinations only: beside a private production
  invariant, in a responsibility-named public integration target, in cold
  certification, or deletion. Test-only public production APIs are forbidden.
- `worth-query-certification`, `worth-query-declaration`, and
  `worth-query-installation` are permanent packages, not transitional shards.
  Their names and dependency edges must be added to the machine constitution
  in the slice that creates them.
- A timing observation is a decision gate, not a new timing-test framework.
  The milestone aims for authority-local loops measured in tens of seconds;
  merely getting below five minutes is not closure.

## Phase Plan

### Phase 1: Select The Load-Bearing Compiler Denials

Move retained trybuild registration out of the ordinary library harness and
feed it into one direct compile-fail integration-test target. Rationalize the
portfolio before registering it:

- keep phase-order, move-only, authority-forgery, cross-kind substitution, and
  load-bearing facade counterexamples
- keep only representative opaque-construction probes where private
  construction is itself the authority boundary
- move unique positive public journeys to ordinary integration tests or
  doctests, and delete duplicate golden transcripts
- delete all other fixtures and their expected diagnostics

The compiler target may use small responsibility-named registration modules to
stay under the workspace line cap. Those modules contain fixture declarations
only; they do not discover paths from production registries or mint proof
identities, ownership records, plans, or receipts.

**Relevant subsystems**

- the 38 inherited trybuild harness files
- `workspaces/worth-query/crates/worth-query-certification/tests/ui`
- `workspaces/worth-query/crates/worth-query/src/integration_tests.rs`
- the direct Cargo test target for compiler certification

**Warnings**

- A strong filename does not make a strong fixture. If the first error is an
  unresolved import but the fixture claims to prove wrong-phase substitution,
  the fixture is false confidence and must be deleted or rewritten against the
  real public seam.
- Privacy is not independently valuable for every report, row, counter, and
  receipt. Retain privacy proof only where public construction would mint or
  counterfeit authority.
- One counterexample may cover a family of equivalent implementation-private
  shapes. Do not preserve one probe per field or historical type.

**Verification**

- Run the ordinary library lane once after the cutover and confirm its existing
  product tests pass without invoking trybuild.
- Run the direct compiler-certification target once and require every retained
  compile-fail fixture to fail for its intended reason.
- Run migrated positive journeys through their ordinary integration or doctest
  owner; do not recreate a positive-fixture inventory.
- Do not add a test that scans source files, counts harness constructors,
  compares inventories, or otherwise proves the test topology.

**Adversarial tests**

- A retained fixture whose first diagnostic no longer reaches its named
  authority, substitution, phase, ownership, or facade boundary must fail the
  review and be repaired or deleted rather than accepted by filename.
- Two representative same-family privacy probes must demonstrate that one
  load-bearing authority-construction denial covers the family; retaining both
  requires a distinct compiler-visible product invariant.

**Engineering decisions**

- Cargo owns target reuse, locking, invalidation, and job count.
- trybuild owns negative-fixture compilation and diagnostic comparison.
- A direct integration-test target is sufficient process isolation. No child
  process protocol or custom executable is required.
- The compile portfolio is deliberately not sharded. Revisit parallelism only
  after the authority crate split changes the natural Cargo package graph.

**Open questions**

- None.

### Phase 2: Cut Consumer Dependencies And Adopt The Direct Workflow

Delete Query-side source audits and certification rows that read Worth UI.
Consumer adoption belongs to the consumer workspace and may use Query only
through its public facade. Then make the smallest honest commands the supported
workflow: ordinary Query behavior during development and explicit compiler
certification when public type boundaries change.

**Relevant subsystems**

- Query reference-consumer and product-boundary certification
- installed-domain and native-value certification manifests
- Worth UI's existing Query binding tests
- local and CI verification commands

**Warnings**

- Moving a Query-owned semantic test into Worth UI would invert authority in
  the other direction. Only consumer-adoption evidence moves or disappears;
  Query behavior and public-boundary fixtures remain in Query.
- Do not retain a compatibility registry of former Worth UI paths. Deleting the
  dependency means Query no longer knows those paths exist.
- Do not add authority-local selection, impact analysis, proof manifests, or
  cost receipts here. Package boundaries in Milestone 9.13.2 will provide the
  natural selection boundary.

**Verification**

- Run the affected Query behavior tests after deleting consumer-path scans.
- Run Worth UI's existing Query binding tests from the Worth UI workspace; no
  Query command may be required to certify Worth UI adoption.
- Record one before/after wall-clock observation for the real developer command
  and one for compiler certification. These are observations, not golden tests
  or a repeated benchmark suite.

**Adversarial tests**

- Removing or renaming the Worth UI checkout must not change any Query test
  discovery, compilation, or outcome.
- Worth UI must prove its Query binding through Query's public facade while a
  Query-side attempt to inspect Worth UI source has no supported test seam.

**Engineering decisions**

- `cargo test -p worth-query --lib` is the ordinary Query iteration command.
- `cargo test -p worth-query --test compile_certification` is the compiler
  certification command.
- Full workspace CI may run both ordinary and compiler-boundary tests, but it
  may not reconstruct or shard the compiler fixture list through production
  code.
- Timing budgets are decision inputs. They do not justify a performance-testing
  subsystem inside Query.

**Open questions**

- None.

### Phase 3: Establish The Cold Certification Leaf

Inventory only the current cert-only entry points: the retained compiler
target, replay/reconstruction suites, source-heavy hostile certification, and
the harness APIs they consume. Create `workspaces/worth-query` and move the
Query engine plus its declaration, host, and replay audience facades into that
workspace without changing their public type identity. Create
`worth-query-certification` there as a cold leaf that may depend on the current
monolith during migration but is depended on by no ordinary Query path. Move
only tests and support that are truly certification-owned; do not sweep ordinary
behavior into the leaf to make the monolith look smaller. The repository root
remains an orchestrator and consumes Query through explicit path dependencies
rather than owning Query package membership.

Amend `NAMING.md`, `BOUNDARIES.md` where required, `road1.toml`, workspace
membership, and generated agent contexts in this slice. The final package is
already the certification node of the 9.13.2 authority graph, even though its
temporary upstream dependency is replaced as later authorities extract.

**Relevant subsystems**

- the direct `compile_certification` target and retained UI corpus
- cert-only replay and reconstruction entry points
- hostile cross-authority fixtures and certification-only harness surfaces
- root, Worth UI, Make, CI, line-cap, and constitution paths that consume or
  enforce the Query workspace
- Road 1 naming, boundary, and agent-context enforcement

**Warnings**

- Certification may consume public product facades and explicitly cert-only
  seams. It may not require ordinary packages to expose private constructors or
  already-solved receipts for test setup.
- Do not turn every slow test into certification. A test that owns ordinary
  declaration, installation, or runtime behavior stays with that product
  authority even if it is inconvenient.
- Do not introduce a certification prelude, shared fixture crate, manifest,
  registry, inventory, or runner protocol.

**Adversarial tests**

- Removing `worth-query-certification` from an ordinary Query command must
  leave that command's package closure and behavior unchanged.
- Entry, derived, app, UI, declaration, and installation packages must be
  mechanically unable to depend on certification or replay surfaces.
- A retained replay or hostile fixture must execute a real public journey;
  replacing its setup with a pre-solved receipt must make the test incapable
  of passing review.

**Verification**

- Run the new certification package directly once.
- Run the ordinary library target once and confirm certification is absent
  from its dependency and execution path.
- Run Cargo metadata for the root, Query, and Worth UI workspaces and confirm
  each resolves the one moved Query engine rather than a stale path or duplicate
  package.
- Run boundary-check and agent-context for the new package edge.
- Record one before/after observation; do not repeatedly sample.

**Closure evidence (2026-07-18)**

- `workspaces/worth-query` owns the engine, declaration, host, replay, and cold
  certification packages. Its default members are only `worth-query`,
  `worth-query-decl`, and `worth-query-host`; replay and certification require
  explicit selection.
- The ordinary library target passed all 2,941 tests without trybuild output.
  A fully warm run completed in 32.5 seconds, including 8.0 seconds of test
  execution. The remaining giant-target cost belongs to Phase 4.
- The inherited 230-fixture compiler portfolio took 286.3 seconds in the moved
  cold leaf and produced 106 path-sensitive mismatches. After applying Phase
  1's selection rule, the leaf owns ten explicit load-bearing denials and its
  clean warm certification run completed in 2.5 seconds.
- Root, Query, and Worth UI Cargo metadata resolve the same moved engine.
  Root consumers and `worth-ui-query-binding` compile against that engine.
- `boundary-check` and `agent-context check` accept the workspace and cold
  dependency edge. The Query slice introduces no new line-cap violation; the
  repository-wide line-cap guard remains red on unrelated inherited files.

**Engineering decisions**

- The Query workspace and cold leaf are permanent. Only certification's
  temporary dependency on the monolithic engine package is transitional.
- Compiler fixtures remain one selected compile-fail target inside the cold
  leaf unless a later authority-local compiler denial is demonstrably cheaper
  and clearer with its production owner.

**Open questions**

- None.

### Phase 4: Dismantle The Giant Library-Test Binary

Inventory only the tests manually injected by `src/lib.rs`,
`src/integration_tests.rs`, and the support modules those injected suites
directly require. Delete the aggregator. Classify each injected suite into one
of the four allowed destinations and move it in the same slice:

1. private invariant beside its production owner
2. public behavior in a responsibility-named integration target
3. cross-authority, replay, or reconstructive certification in the cold leaf
4. deletion when it proves history, ordinary privacy, source topology, or
   another proof

Do not inventory every `#[cfg(test)]` in the crate before starting. Local unit
tests that are not pulled through the manual aggregator remain untouched until
their production authority is extracted or a measured hotspot reaches them.

**Relevant subsystems**

- `workspaces/worth-query/crates/worth-query/src/lib.rs`
- `workspaces/worth-query/crates/worth-query/src/integration_tests.rs`
- `workspaces/worth-query/crates/worth-query/tests/support`
- explicit `[[test]]` registrations in
  `workspaces/worth-query/crates/worth-query/Cargo.toml`, because
  automatic integration-test discovery is disabled
- the directly included public-journey, runtime, installation, publication,
  and certification suites

**Warnings**

- Do not create one integration binary per source file. Use the smallest
  responsibility-named target set that preserves useful Cargo selection
  without recreating hundreds of link steps.
- Do not retain `pub mod integration_tests`, test support in the product
  facade, or production visibility widened solely for moved tests.
- A broad public journey does not become a unit test merely because it once
  lived inside the library target.

**Adversarial tests**

- A production edit confined to a public journey's owner must not relink or
  execute cold certification or unrelated journey targets.
- Deleting `src/integration_tests.rs` and its `lib.rs` registration must leave
  no orphan suite, duplicate test, hidden `#[path]` reinjection, or public
  test-support export.
- A moved test that previously used private access must either falsify the
  private invariant beside its owner or use the real public contract; a
  test-only public constructor is a phase failure.

**Verification**

- Run each new responsibility-named target once, then the remaining library
  target once.
- Compare the test identities once to ensure behavior was moved or deliberately
  deleted, not silently lost. This is a manual migration check, not a permanent
  inventory test.
- If the remaining library target still dominates ordinary iteration, record
  which production authority owns that breadth and carry its local tests into
  Phase 6 or 7. Do not compensate with visibility widening or another runner.
- Record rebuild and execution observations for the old and new ordinary
  targets on the same machine.

**Closure evidence (2026-07-18)**

- The removed manual aggregator injected 54 suites and 262 tests into the
  library binary. The migration retained 200 behavioral tests: 73 downstream
  journeys in three explicit targets, 122 private graph-read invariants beside
  `runtime::tests`, and five native-predicate invariants beside validation.
  The other 62 tests were source or documentation scans, milestone residue
  manifests, API-signature pointer checks, or assertions over pre-certified
  bundles and were deleted as proof-of-proof evidence.
- The downstream targets are `public_declarative_journeys` (37 tests),
  `runtime_public_journeys` (21 tests), and `graph_obligation_journeys` (15
  tests). Each passed independently. No retained suite qualified as archive
  reconstruction or cross-package replay certification; replay-named graph
  tests exercise ordinary deterministic runtime identity and remain with that
  private authority.
- The remaining library target passed 2,806 tests. Identity reconciliation is
  exact: 2,679 pre-existing local tests plus 122 graph-read and five validation
  tests. Across the library and three targets, 2,879 tests remain. No suite is
  registered through `#[path]`, `src/integration_tests.rs`, a public test
  support export, or a test-only public constructor.
- Before the cut, a public-journey edit inherited the 2,941-test library link;
  the Phase 3 observation was 32.5 seconds fully warm and more than two minutes
  after a production edit. After the cut, an actual public target-root edit
  rebuilt only `public_declarative_journeys` in 15.4 seconds. The runtime and
  graph-obligation executable timestamps were unchanged. Initial independent
  runs completed in 18.0, 7.4, and 4.6 seconds respectively; their test
  execution itself was 0.03, 0.41, and 0.18 seconds.
- Full-library certification is still broad rather than fast: the corrected
  run took 37.8 seconds, including 36.8 seconds of execution. Its largest test
  owners are runtime (992), harness (266), application (258), subscription
  (147), consumer kit (136), domain capabilities (123), and projection
  consumption (109). Those owner-local tests stay honest in Phase 4 and become
  Phase 6/7 package boundaries instead of being routed through another runner.
  After all new binaries were linked, the final fully warm package run took
  10.9 seconds, including 6.0 seconds of test execution.
- Every Phase 4 code and test file is within the 400-line cap. The global guard
  remains red on the repository's unrelated inherited over-cap baseline; it
  reported no `workspaces/worth-query` Phase 4 path.

**Engineering decisions**

- The library target may retain genuinely local unit tests while the monolith
  exists. The prohibited object is the manual cross-responsibility integration
  aggregator.
- Cargo target selection is the temporary selection boundary until production
  packages take ownership.
- Shared downstream fixture breadth remains visible in `tests/support`; Phase
  4 does not create a fixture crate or duplicate private/public backends to
  conceal it. Private graph-read tests reuse the canonical runtime test backend
  through an owner-local adapter.

**Open questions**

- None.

### Phase 5: Remove Reconstructive Test Hotspots

After Phase 4 exposes honest target timings, inventory only tests or setup
paths observed to dominate an ordinary target. Start with
`every_package_semantic_family_is_order_independent` and any sibling path that
rebuilds the same installed package, runtime, index, or canonical artifact for
each assertion. Name the authoritative input and the derived state once, then
either reuse the proved setup inside the ordinary scenario or move genuinely
exhaustive convergence/rebuild coverage to certification.

This phase removes repeated work by logic. It does not add parallel workers,
timeouts, ignored tests, sampling infrastructure, or a faster fake runtime.

**Relevant subsystems**

- installed-domain package convergence and conflict matrices
- package-validation setup and derived-index rebuild scenarios
- any ordinary test whose single observed execution dominates its target

**Warnings**

- Moving an O(n^2) reconstruction loop to certification does not repair it.
  First remove repeated derivation or collapse equivalent cases; only the
  residual exhaustive cross-authority proof belongs cold.
- Do not reuse mutable authoritative state across cases whose isolation is the
  property under test. Reuse canonical declarations or immutable admitted
  setup only where semantic equivalence is explicit.
- Elapsed time identifies the hotspot. Structural counters and scenario shape
  explain and repair it; no timing threshold becomes a product test.

**Adversarial tests**

- Declaration-order convergence must still include reversed and permuted
  representative inputs while proving the same canonical identity, outcome,
  and exact structural counters.
- Destroying and rebuilding derived installation indexes must preserve lookup,
  denial, and identity outcomes without reinstalling authoritative packages for
  every assertion.
- One-field semantic conflict cases must still fail atomically with zero
  residue after equivalent cases are collapsed.

**Verification**

- Run only the repaired target once, followed by its owner package target once.
- Inspect exact structural counters or construction counts to confirm repeated
  broad setup was removed.
- Record one before/after elapsed observation for the hotspot and target.

**Closure evidence (2026-07-18)**

- `every_package_semantic_family_is_order_independent` was replaced by a
  representative convergence proof. It now installs the canonical package and
  two load-bearing order masks instead of reconstructing the same authority for
  all 31 non-empty masks.
- The retained masks prove reverse-order and interleaved-order convergence with
  identical package identity, normalized family cardinalities, and installed
  snapshots. Independent sibling tests
  retain one-field conflicts and atomic zero-residue denial.
- The focused scenario's observed execution fell from 0.07 seconds to 0.01
  seconds. The final warm owner package completed 2,794 tests in 10.86 seconds;
  no timeout, parallel runner, ignored case, or fake runtime was introduced.

**Engineering decisions**

- Exhaustive matrices certify equivalence classes; ordinary tests prove the
  representative product rule. Neither needs to repeat identical authority
  construction per assertion.
- Test performance is repaired at the same authority/derivation boundary as
  production performance.

**Open questions**

- None.

### Phase 6: Extract Declaration Authority

Inventory only canonical intent, authoring, canonicalization, binding grammar,
schema-visible validation, identity, result-shape, collection declaration, and
view declaration meaning plus their direct consumers. Create the permanent
`worth-query-declaration` package and move those responsibilities with their
owned tests. Split mixed files at the boundary; do not import runtime types
back into the declaration package to preserve their current location.

`worth-query-decl` becomes a narrow audience facade over this package.
`worth-query` may temporarily depend on and re-export the new package for
uncut consumers, but the compatibility edge is deleted in 9.13.2 and may not
contain behavior.

**Relevant subsystems**

- canonical query declarations, identities, native value typing, and result
  shapes
- authoring, canonicalization, collection, and view declarations
- `worth-query-decl` and direct declaration consumers
- declaration-local unit, integration, and selective compiler tests

**Warnings**

- `application`, `runtime`, `domain_capabilities`, and similarly broad folders
  are not ownership units. Move only declaration meaning.
- `syn`, `quote`, and `proc-macro2` may not remain normal dependencies unless a
  real declaration feature, rather than source auditing, requires them.
- The compatibility re-export is migration plumbing, not a second facade or an
  excuse to keep downstream types in public signatures.

**Adversarial tests**

- The declaration package must build with relational, runtime bridge, signal,
  workspace, live, replay, and certification packages absent from its graph.
- Equivalent public construction paths must preserve canonical identity,
  native value meaning, result shape, and exact counters.
- Declaration code must be unable to mint installation, admitted, executed,
  publication, consumption, settlement, or replay authority.

**Verification**

- Run `cargo check -p worth-query-declaration` and its owned tests without the
  remaining monolith or later Query authorities selected.
- Compile declaration consumers through `worth-query-decl`.
- Run boundary-check and agent-context, then record one warm check and test
  observation.

**Closure evidence (2026-07-18)**

- Permanent `worth-query-declaration` now owns authored intent,
  canonicalization, binding grammar, collection and view declarations,
  schema-visible validation, declaration identity, result shapes, and the
  typed schema macro. `worth-query-decl` consumes it directly; the monolith's
  edge is compatibility-only.
- Its direct dependency graph is only `sha2` and `worth-foundational`.
  Runtime bridges, relational machinery, signals, workspace,
  live, replay, certification, installation, and the Query monolith are absent.
- Declaration-owned tests cover authoring rejection, binding, typed schema,
  native value contracts, normalization counters, warnings, equivalence, and
  query/result-shape identity separation. The former monolith owner tests now
  run in this package, while `worth-query-decl` proves a real facade-only
  authoring-to-validation journey.
- Canonical and schema artifacts retain the exact authorities minted by their
  production journeys. Digest-to-authority reconstruction and the external
  schema-token admission shortcut were removed instead of being preserved as
  migration plumbing.
- Observed warm commands were 0.15 seconds for `cargo check` and 0.48 seconds
  for the owned test suite. `worth-query-host` remained warm at 1.57 seconds
  immediately after the full Query suite, proving there is no alternate
  declaration feature graph forcing a broad rebuild.

**Engineering decisions**

- Declaration is the first production node in the frozen 9.13.2 DAG.
- A type shared with later authorities remains here only when declaration owns
  its meaning, not merely because placing it upstream avoids a cycle.

**Open questions**

- None.

### Phase 7: Extract Installation Authority

Inventory only portable domain packages, installation admission inputs,
runtime generation identity, installed domain and operation identity, conflict
semantics, rebuildable installed indexes, and their direct declaration
dependencies. Create permanent `worth-query-installation` above declaration
and move the owned tests. Separate portable definitions from volatile runtime
providers; execution callbacks and workspaces remain in the monolith until
9.13.2 extracts execution.

**Relevant subsystems**

- `domain_installation` package definitions, validation, conflict admission,
  generation affinity, and installed identity
- installation-owned portions of `domain_capabilities`, `application`, and
  `runtime`
- derived installation indexes and rebuild paths

**Warnings**

- Do not move execution, projection, workflow, provider, or certification code
  into installation merely to avoid a dependency edge.
- Portable definitions remain callback-free and non-operational until
  installed. Derived indexes are disposable and rebuildable from installed
  authority.
- If a mixed type crosses the boundary, split its meanings; do not create a
  bag-shaped shared crate.

**Adversarial tests**

- Equivalent packages converge across declaration order while one-field
  conflicts fail atomically with zero installed residue.
- Foreign-runtime, stale-generation, copied-semantic-key, and locally
  reconstructed handles cannot resolve installed authority.
- Destroying and rebuilding every installation index preserves exact lookup,
  denial, identity, and normalized installed cardinalities. Construction-work
  and lookup counters restart because they describe work performed by the new
  index, not portable authority.

**Verification**

- Run `cargo check -p worth-query-installation` and its owned tests without
  admission, execution, publication, replay, or certification selected.
- Compile the temporary monolith consumer and the host facade against the new
  package without introducing a reverse edge.
- Run boundary-check and agent-context, then record one warm check and test
  observation.

**Closure evidence (2026-07-18)**

- Permanent `worth-query-installation` now owns callback-free portable domain
  packages, typed validation and admission denials, runtime/generation
  affinity, installed package and operation authority, exact conflict
  semantics, and a rebuildable installed index. It has no Query package
  dependency; the monolith performs the one-way typed lowering into this
  callback-free contract.
- The monolith's installed-domain authority retains the exact installation
  package proof, and every volatile execution registration retains the exact
  installed-operation proof minted by the core index. Registry and execution
  lookup validate those proofs before provider work. Runtime-bound operation
  selection travels in a Query-owned wrapper beside the portable declaration,
  so declaration remains runtime-agnostic without falling back to semantic-key
  correlation.
- Package-owned tests prove order convergence, atomic conflicts, malformed and
  duplicate-definition denial, capability/configuration/operating admission,
  foreign runtime, stale generation, copied-key rejection, package-relevant
  support-profile convergence, indexed lookup, and exact rebuild identity plus
  normalized installed cardinalities. Monolith
  installed-domain tests prove the temporary typed-lowering boundary.
- Observed warm commands were 0.13 seconds for `cargo check`, 0.28 seconds for
  the owned suite, and 0.36 seconds for the focused monolith integration suite.
  All new and touched extraction files remain within the 400-line cap.

**Engineering decisions**

- Installation is the second production node in the frozen 9.13.2 DAG and the
  direct substrate for 9.14 installed-operation semantics.
- Runtime provider registration remains volatile execution mechanics and is
  not part of portable installation identity.

**Open questions**

- None.

### Phase 8: Freeze The Livable Inner Loops

Inventory only the commands a developer will actually use after the preceding
cuts. Remove accidental package or target edges discovered by `cargo tree` and
Cargo build output; do not produce a persistent dependency inventory. Document
the smallest owner commands and run each once after a representative owner-
local edit or equivalent invalidation.

**Relevant subsystems**

- declaration and installation package commands
- remaining monolith behavior targets
- cold certification and compiler-certification commands
- CI command ordering and contributor documentation

**Warnings**

- A no-op command alone is weak evidence because it can hide the cost of
  rebuilding an oversized artifact. Observe at least one representative
  owner-local invalidation.
- Full workspace CI remains a closeout gate, not the inner loop.
- Do not encode elapsed thresholds as tests. If the observed loop is still
  measured in minutes, repair the package or target boundary before closure.

**Adversarial tests**

- Touching declaration-owned production code must not compile installation,
  the monolith, certification, replay, or publication-shaped code.
- Touching installation-owned production code may compile declaration but must
  not compile later authorities or cold certification.
- Selecting ordinary remaining-monolith behavior must not execute compiler,
  replay, source-audit, or Worth UI work.

**Verification**

- Record one same-machine observation for warm check, owner test, and one
  representative owner-local rebuild for declaration and installation.
- Record one observation for remaining ordinary behavior and one for cold
  certification.
- Run full Query, boundary-check, agent-context, line-cap, and workspace gates
  only after all owner-local commands are honest.

**Closure evidence (2026-07-18)**

- `cargo tree` confirms that declaration reaches only `sha2` and Foundational,
  while installation reaches only `sha2`. Neither owner command can select the
  remaining engine, replay, certification, Worth UI, or a later Query
  authority. The remaining engine closure contains no replay, certification,
  or Worth UI package.
- The Makefile and workspace guide now expose separate one-command declaration,
  installation, remaining-engine check/test, compiler-certification, and cold-
  closeout loops. The former `query-fast` sequence no longer performs check,
  test compilation, and test execution back-to-back. The stale monolith-local
  compiler command and redundant declaration dev-dependency are removed.
- Same-machine warm observations were `0.74 s` check and `0.33 s` owner tests
  for declaration, and `0.36 s` check and `0.22 s` owner tests for
  installation. Package-scoped invalidation followed by an owner-only check
  took `1.07 s` for declaration and `0.35 s` for installation.
- After those real upstream invalidations, the complete remaining-engine
  package run passed 2,768 tests in `43.55 s` wall time, including `30.53 s`
  compilation and roughly `5.2 s` test execution. It emitted no trybuild,
  replay, source-audit, or Worth UI work.
- The explicitly selected cold lane passed all 10 retained compile-fail cases
  plus replay doctests in `62.98 s` after the upstream invalidation. CI no
  longer prebuilds the entire Query workspace before its ordinary/cold split,
  so that separation remains observable rather than cache-masked.
- Final closeout reran the complete ordinary Query workspace and the Worth UI-
  owned Query-binding package successfully. Boundary-check, generated agent-
  context validation, and diff integrity also passed. Every changed Rust file
  remains within the 400-line cap; the repository-wide line-cap and formatting
  gates remain red only on inherited files outside this milestone. The root
  workspace test attempt exhausted the development volume during compilation
  (`os error 112`) before tests could run, so it produced no code-level failure
  and must be rerun after sufficient build-artifact space is available.

**Engineering decisions**

- Decision targets are warm declaration checks around 15 seconds or less,
  declaration/installation owner tests around 30 seconds or less, and
  remaining ordinary monolith behavior around 60 seconds or less on the same
  development machine. These are milestone review budgets, not portable CI
  assertions.
- Failure to reach those budgets reopens the responsible slice. It does not
  justify a runner, cache, shard, ignored test, or relaxed definition of
  livable.

**Open questions**

- None.

## Must Ship

- ordinary Query library tests with no trybuild execution
- one direct compile-fail certification target over the selected load-bearing
  denial portfolio
- no positive trybuild execution; unique public journeys live in ordinary
  integration tests or doctests
- no historical tombstones, generic certification-artifact privacy probes,
  orphan diagnostics, or production-owned fixture registries
- no Query test or certification source that reads Worth UI
- permanent cold `worth-query-certification` package with no ordinary reverse
  dependency
- deletion of the manual `src/integration_tests.rs` library-test aggregator and
  classification of every directly injected suite
- repaired reconstructive test hotspots, beginning with installed-package
  validation and derived-index rebuild matrices
- permanent `worth-query-declaration` and `worth-query-installation` packages,
  narrow legal audience access, and machine-enforced dependency direction
- short documented commands for declaration, installation, remaining ordinary
  behavior, and compiler/cold certification iteration

## Must Preserve

- every distinct compiler-enforced authority-minting, substitution,
  phase-ordering, ownership, move-only, and facade invariant
- every unique positive ordinary facade journey, through ordinary test
  ownership rather than a trybuild transcript
- Query product semantics, public API, runtime authority, and Store handoffs
- Worth UI's ownership of its own consumer behavior and adoption tests
- canonical declaration identity, native value meaning, package installation,
  runtime generation affinity, conflict semantics, and rebuildable-index truth
- the final 9.13.2 authority DAG: declaration precedes installation, ordinary
  authorities never depend on certification, and facades do not implement
  behavior

## Acceptance Evidence

- one successful ordinary library run with no trybuild output
- one successful direct compiler-certification run covering the retained
  denial set, with each fixture failing at its named boundary
- one successful ordinary run covering migrated positive journeys
- one successful Worth UI Query-binding run owned by the Worth UI workspace
- one successful cold-certification run absent from ordinary package closures
- one successful responsibility-target run for every suite moved out of the
  former library aggregator, with the aggregator and public test registration
  absent
- exact convergence, conflict, and rebuild outcomes from the repaired package-
  validation scenarios without repeated broad reconstruction
- declaration and installation owner commands that build no later or cold
  Query authority
- one same-machine before/after observation per slice and representative
  owner-local invalidation observations showing the ordinary loops are measured
  in tens of seconds rather than minutes
- repository review showing historical compiler tombstones, orphan baselines,
  production-owned fixture registries, custom proof runners, receipts, shards,
  cache systems, compiler wrappers, meta-tests, and compatibility paths are
  absent

## Allowed Debt

- Admission, execution, and publication remain inside the shrinking monolith
  until Milestone 9.13.2 extracts them.
- The cold certification package may temporarily depend on the monolith; no
  ordinary package may depend back on certification, and 9.13.2 must replace
  the temporary edge before deleting the monolith.
- The remaining library target may retain genuinely local unit tests. It may
  not retain the manual integration-suite aggregator or any cert-only work.
- No inventory platform, test selector, custom runner, shared fixture package,
  compatibility facade, reverse dependency, or timing-test framework may
  remain debt.

## Transition Line-Cap Exception

This milestone removes dead source-audit rows and the test-suite certification
gate from the following inherited Milestone 9.6 source-topology implementation
without preserving deleted targets:

- `workspaces/worth-query/crates/worth-query/src/application/support/closure.rs`
- `workspaces/worth-query/crates/worth-query/src/application/support/identity_boundary_inventory.rs`
- `workspaces/worth-query/crates/worth-query/src/application/support/identity_boundary_inventory_sources.rs`

Those files remain temporarily exempt from the 400-line cap only until the
slice that removes their source-audit responsibility. They are inherited
oversized code and are not a pattern to copy. Phase 3 or the first later slice
that touches this responsibility deletes the source-audit authority and its
tests rather than splitting it into smaller proof-of-code catalogs. No new row,
scan, or consumer may be added while the exception exists.

## Store Dependency

This milestone is not blocked on Store. It changes test ownership and Query's
internal package graph without changing provider, durability, replay, or
Store-facing semantic contracts. Store integration must consume the extracted
declaration and installation authorities through the final facades established
by 9.13.2.

## Sequencing Notes

Milestone 9.13.1 follows 9.13 because it preserves Query meaning while repairing
the development loop around the authority surface 9.13 established. Its first
two phases remove obvious test waste; its later slices establish cold
certification and the two upstream production packages needed for useful Cargo
selection.

Milestone 9.13.2 follows immediately, extracts admission, execution, and
publication, retargets certification to the completed graph, cuts consumers,
and deletes the monolith. Milestone 9.14 begins only after that completion, so
installed operation semantics land in permanent authority packages rather than
expanding the monolith.
