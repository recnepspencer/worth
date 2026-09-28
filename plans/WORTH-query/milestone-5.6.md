# Milestone 5.6 Engineering Spec: Unified Application Facade And Unified Runtime Configuration

> **Status:** Completed. Closeout: [milestone-5.6-closeout.md](./milestone-5.6-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-5.5.md](./milestone-5.5.md)
>
> **Prior closeout:** [milestone-5.5-closeout.md](./milestone-5.5-closeout.md)
>
> **Next roadmap step:** Milestone 6 is not yet closed and must compose with,
> not redefine, the facade and configuration ownership frozen here.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Implementation guardrail:** [milestone-5.6-build-checklist.md](./milestone-5.6-build-checklist.md)
>
> **Primary architectural driver:** make `worth-query` the explicit daily-driver import and runtime configuration surface for ordinary application code without flattening lower-crate ownership, capability boundaries, or configuration structure into one bag-shaped convenience API
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.6-build-checklist.md](./milestone-5.6-build-checklist.md)
> - [milestone-5.4.md](./milestone-5.4.md)
> - [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
> - [milestone-5.5.md](./milestone-5.5.md)
> - [milestone-5.5-closeout.md](./milestone-5.5-closeout.md)

## Goal

Make `worth-query` the explicit, application-facing facade and runtime
configuration surface for ordinary domain developers so they can use one
coherent daily-driver import while lower-runtime authority, capability
admission, and subsystem-shaped configuration remain explicit, typed, and
honest.

## Why This Milestone Exists

Milestone 5 made live query meaning survive time. Milestone 5.1 made
locality-bearing live narrowing and stream contracts explicit. Milestone 5.2
made preview-session basis identity explicit. Milestone 5.3 made route posture
planner-owned. Milestone 5.4 made correspondence and historical-path honesty
explicit. Milestone 5.5 made workflow declaration and lower-authority lowering
query-native without turning `worth-query` into a second mutation engine.

Those milestones solved the deep semantics. They did not yet solve the everyday
product boundary where developers decide what they import and how they boot the
runtime.

Without Milestone 5.6, the platform still fractures at the application layer:

- domain developers must shop among `worth-query`, `worth-relational`,
  `worth-signal`, and bridge-owned setup/configuration surfaces to do normal
  work
- configuration truth risks collapsing into a flat bag of toggles that no
  longer mirrors subsystem ownership
- capability support becomes implied by one broad facade type instead of
  advertised and proven explicitly
- lower-authority boundaries that earlier milestones carefully preserved get
  blurred by "convenience" pass-throughs

Milestone 5.6 therefore exists to freeze:

- that `worth-query` is the endorsed application-facing import for admitted
  daily-driver capabilities
- that a unified facade must still preserve subsystem ownership and typed
  capability witnesses
- that unified runtime configuration must remain sectioned by subsystem
  responsibility rather than becoming one bag-shaped `Config`
- that support metadata and executable capability admission must remain in sync
- that unsupported or deferred composed capabilities must fail typed and early
  instead of degrading into implied support

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "put one nice API on top." It is
  preserving architectural truth while making the product feel unified. The
  design must solve the flat-bag and pass-through-glue failure mode first.
- `arch_laws.md`: Laws 1, 7, 9, 10, 13, 17, 20, 27, 30, 32, 40, and 41 dominate
  this milestone. The facade must expose explicit orchestration boundaries,
  configuration must mirror subsystem architecture, and proof-bearing witness
  types must encode exactly which composed capabilities were admitted.
- `perf_laws.md`: facade calls and unified config resolution must not hide broad
  capability discovery, runtime probing, or cost-dishonest fallback. Capability
  lookups, section resolution, and unsupported-composition denials must be
  mechanically visible.
- `domain_laws.md`: application facade orchestration, capability witness types,
  support registry, support reporting, configuration sections, configuration
  validation, and certification are separate responsibilities and must not
  collapse into one god module.
- `worth_query_vision.md`: `worth-query` only fully lands as the developer
  surface if ordinary reads, live promotion, preview/workflow composition, and
  historical/query basis work can be reached through one coherent import
  without erasing the truth/runtime ownership model.
- `worth_query_roadmap.md`: Milestone 5.6 is explicitly about the unified
  application facade and unified runtime configuration, not new deep semantics.
  It belongs after workflow lowering because the facade must compose real
  admitted capabilities rather than speculate about them.
- `test-requirements.md`: the `Unified Facade And Configuration Boundary Test`
  is the closeout proof. It requires authority-preserving facade composition,
  sectioned configuration, typed unsupported-composition denial, and support
  metadata synchronization with actual admission behavior.
- `milestone-5.4.md` and `milestone-5.4-closeout.md`: correspondence and
  historical-path posture are already explicit and must remain capability-owned
  surfaces inside the new facade rather than being flattened into generic
  "history mode" flags.
- `milestone-5.5.md` and `milestone-5.5-closeout.md`: workflow lowering and
  authority boundaries are already frozen. Milestone 5.6 must expose those
  capabilities coherently without weakening their proof boundaries or collapsing
  them into one "do workflow" shortcut.

## Adversarial Constraint

Milestone 5.6 must survive the following hostile condition:

> A developer configures and uses the platform entirely through one `worth-query`
> application-facing surface while mixing admitted read, live, preview,
> workflow, and historical capability families; the facade and config must stay
> coherent for the developer without flattening subsystem ownership, hiding
> orchestration boundaries, inventing support, or turning sectioned runtime
> configuration into one ambiguous bag.

Concretely, the design must remain correct when all of the following are true:

- application code imports one primary `worth-query` facade and expects it to
  be the daily-driver surface
- the admitted capability mix includes capabilities with different authority
  owners and different failure topologies
- some capability families are admitted, some are deferred debt, and some are
  unsupported
- runtime-backed capability composition is available before later durable or
  store-backed milestones are complete
- a naive implementation would be tempted to:
  - re-export everything broadly and let developers discover support by trial
    and error
  - collapse configuration into a flat bag with mixed ownership
  - expose one broad capability handle with boolean flags
  - imply support for future composition classes because one facade type exists
  - silently route unsupported combinations into best-effort or lower-crate
    escape hatches

If any supported path:

- hides which subsystem owns the underlying authority
- turns configuration into a mixed-ownership property bag
- allows unsupported compositions to look admitted until late failure
- exposes cheap-looking getters or methods that actually cross heavy
  orchestration boundaries
- makes support metadata drift from executable admission behavior
- lets the unified facade become semantics-erasing pass-through glue

then Milestone 5.6 has failed.

## Product Decision Lock

- `worth-query` owns the unified application facade, capability advertisement,
  and unified runtime configuration surface
- `worth-query` does not erase lower-runtime ownership; it composes and exposes
  admitted capabilities through typed witnesses and explicit entrypoints
- unified configuration must be sectioned by subsystem ownership; flat bag
  configuration is out of spec
- the application-facing facade must be coherent, but coherence does not mean
  one broad capability handle or one broad "platform runtime" type
- support advertisement is a first-class artifact and must distinguish:
  - admitted capability families
  - deferred debt capability families
  - unsupported capability families
- capability admission must be executable, typed, and synchronized with support
  metadata
- deferred durable/store-backed capability families may remain explicit debt
  until later milestones close them honestly
- the legacy broad re-export wall may coexist temporarily for compatibility, but
  it is not the normative daily-driver surface for new work

Normative consequence:

- any implementation path that exposes one giant `WORTHQueryConfig` with mixed
  subsystem-owned fields is out of spec
- any implementation path that exposes one broad "application capability"
  object plus booleans for support is out of spec
- any implementation path that advertises a capability family without a typed
  admission or typed denial path is out of spec
- any implementation path that widens unsupported compositions into lower-crate
  escape hatches or silent fallback is out of spec
- any implementation path that redefines lower-crate semantics instead of
  exposing them through typed witnesses is out of spec

## Compile-Time Enforcement Policy

Milestone 5.6 must classify which unified-facade and unified-configuration
guarantees become unrepresentable, uncompilable, or construction-time
rejection.

`Unrepresentable` in public types:

- externally constructing admitted capability witnesses
- externally constructing support-report or capability-registry artifacts in
  ways that can drift from executable admission
- representing unified configuration as one flat property bag instead of
  subsystem-owned sections
- mutating one capability witness into another capability family after
  admission
- treating one config section as an untyped map or generic options bag

`Uncompilable` at the API boundary:

- using a query-read witness to call live-only, preview-only, workflow-only, or
  historical-only operations
- using a preview-only or historical-only capability witness to bypass the
  query/workflow entry surface it does not own
- reaching past the unified facade into internal application/capability/support
  modules from other crates
- constructing composed-capability shortcuts through bool flags or untyped
  section probes
- adding a new owning config section without updating every construction site
  and every validated-config propagation site

`Construction-time rejection` with typed failures:

- invalid unified configuration where required owning sections are absent or
  contradictory
- unsupported composed capability requests
- deferred capability families presented as if they were admitted
- configuration/admission pairs whose support metadata and executable admission
  disagree

`Counter-visible debt`:

- later durable/store-backed capability families may remain explicit debt, but
  only if support metadata, typed admission, and counters expose them as
  deferred rather than silently available

## Architectural Shape

Milestone 5.6 is not a "pub use everything" milestone. It is an application
subdomain with at least these separate responsibilities:

- `application/config`
  - unified config root
  - subsystem-owned config sections
  - config validation and section-resolution artifacts
- `application/capability`
  - capability witnesses
  - facade entrypoints
  - capability admission/resolution
  - typed capability errors
- `application/support`
  - capability family taxonomy
  - capability registry
  - support matrix
  - support report shaping
- `harness/unified_facade_certification`
  - certification lane definitions
  - matrix construction
  - row catalog
  - suite tests

This milestone must not ship as:

- one giant `application.rs`
- one giant `config.rs` that mixes ownership with admission logic
- one support module that also performs runtime admission
- one facade file that contains config shape, capability taxonomy, support
  metadata, counters, and execution routing together

## Capability Taxonomy

Milestone 5.6 freezes the requirement that the daily-driver facade be composed
from explicit capability families rather than one broad application bag.

Required family taxonomy:

- `QueryReadCapability`
- `LiveQueryCapability`
- `PreviewQueryCapability`
- `WorkflowQueryCapability`
- `HistoricalQueryCapability`

Required support-status taxonomy:

- `Admitted`
- `DeferredDebt`
- `Unsupported`

Required support artifacts:

- `WORTHQueryCapabilityFamily`
- `WORTHQueryCapabilitySupportStatus`
- `WORTHQueryCapabilityDescriptor`
- `WORTHQueryCapabilityRegistry`
- `WORTHQuerySupportMatrix`
- `WORTHQuerySupportReport`
- `CapabilityAdmissionError`
- `CapabilityAdmissionFailureClass`
- `ConfigurationAdmissionError`
- `ConfigurationAdmissionFailureClass`
- `ValidatedWORTHQueryConfig`

Required support-report content:

- admitted capability families
- deferred capability families
- unsupported capability families
- the config-section posture used to derive that support result
- canonical digests or equivalent machine-checkable identity for the registry
  and support matrix consumed

Required application-facing surface:

- one explicit facade root, such as `WORTHQueryApplicationFacade`
- sectioned config root, such as `WORTHQueryConfig`
- one witness-acquisition method per admitted capability family, such as:
  - `query_read_capability(...)`
  - `live_query_capability(...)`
  - `preview_query_capability(...)`
  - `workflow_query_capability(...)`
  - `historical_query_capability(...)`
- typed capability witness acquisition methods that return either:
  - an admitted witness
  - or a typed denial/deferred result carrying a failure class

Forbidden shapes:

- `PlatformCapability { query: bool, live: bool, preview: bool, ... }`
- `fn capability(family: WORTHQueryCapabilityFamily) -> ...`
- `WORTHQueryConfig { flat list of every subsystem field }`
- `fn do_everything(...)`
- support metadata inferred only from documentation or from the presence of a
  facade method

## Unified Configuration Contract

The unified runtime configuration must be architecture-shaped.

Minimum section ownership:

- `config.query`
- `config.relational`
- `config.signal`
- `config.runtime_bridge`
- `config.store` only when a later admitted capability actually depends on
  store-owned behavior
- additional sections only when they correspond to real subsystem ownership
  rather than a capability family or product marketing label

Critical rule:

- capability families and config sections are not the same taxonomy
- capability witnesses are application-facing composition surfaces
- config sections are owner-facing subsystem surfaces
- capability admission composes across validated subsystem sections; it must not
  be modeled as one config section per capability family

Configuration rules:

- each section owns only its subsystem-shaped fields
- cross-section composition is validated by the facade/admission layer rather
  than by smuggling foreign fields into one section
- config construction must be lifecycle-enforced: adding a new owning section
  must create compile errors at every `WORTHQueryConfig` construction and
  `ValidatedWORTHQueryConfig` propagation boundary until the new section is
  handled explicitly
- missing required sections for an admitted composition must fail typed and
  early
- deferred/store-backed/durable settings may exist only as explicitly gated
  sections or typed deferred fields, never as optimistic booleans that imply
  support

Forbidden configuration patterns:

- mixed-ownership fields in the root config
- "misc", "advanced", or "runtime" sections that collapse multiple subsystem
  responsibilities
- `config.preview`, `config.workflow`, or `config.historical` when those names
  are being used as bags for cross-subsystem settings rather than true owning
  subsystems
- implicit capability admission inferred from non-empty fields
- host-only config repair that silently invents required sections

Required validated output:

- unified config validation must produce a proof-bearing artifact such as
  `ValidatedWORTHQueryConfig`
- capability acquisition must consume the validated config artifact rather than
  raw root config fields

## Facade Boundary Contract

The unified facade must make `worth-query` the endorsed import while remaining
honest about boundary crossings.

Required facade behavior:

- capability acquisition is explicit
- capability witnesses remain narrow
- composed operations keep the lower-authority seam visible
- support reporting is query-owned and available from the same facade family
- unsupported and deferred capability acquisition fails typed and early

Required honesty rules:

- any method that crosses a real orchestration boundary must look like one
- the facade may expose admitted lower-crate capabilities, but it may not
  redefine their authority or merge their failure surfaces into one vague
  error
- the facade must not imply that all capability families compose just because
  they are reachable from one import
- witness acquisition must stay statically named by family rather than routing
  through one runtime-selected family parameter
- support reporting must be derived from the same registry/matrix that
  admission consumes, not from a second independently-maintained summary path

Examples of in-spec shapes:

- `facade.query_read_capability() -> Result<QueryReadCapability, FacadeError>`
- `facade.workflow_capability() -> Result<WorkflowQueryCapability, FacadeError>`
- `facade.support_report() -> WORTHQuerySupportReport`

Examples of out-of-spec shapes:

- `facade.capability() -> EverythingCapability`
- `facade.capability(family) -> CapabilityWitness`
- `facade.run(query, options)` where `options` hides live/preview/workflow
  composition and ownership
- `facade.config().set_enable_history(true)` when history support is actually a
  composed capability with separate ownership and deferred debt classes

Required failure topology:

- `ConfigurationAdmissionFailureClass` must distinguish at least:
  - `MissingRequiredSection`
  - `ContradictorySectionPosture`
  - `DeferredStoreBackedSection`
- `CapabilityAdmissionFailureClass` must distinguish at least:
  - `UnsupportedCapabilityFamily`
  - `DeferredCapabilityFamily`
  - `MissingOwningSection`
  - `InvalidComposedSupportPosture`

The facade must not collapse those failures into one generic "not supported"
string.

## Performance Architecture

Milestone 5.6 is algorithmically lighter than 5.5, but it still has named
performance obligations because facade/config dishonesty often hides as
"harmless" convenience.

Required counters:

- `capability_lookup_count`
- `config_section_resolution_count`
- `support_report_generation_count`
- `unsupported_composition_denial_count`
- `deferred_capability_denial_count`
- `config_validation_denial_count`
- `legacy_escape_attempt_denial_count` if the facade explicitly denies legacy
  escape routes through the new surface

Counter rules:

- capability acquisition must increment lookup counters exactly once per
  attempted acquisition
- config section resolution must be counted by section-aware resolution, not by
  generic "some work happened" counters
- unsupported and deferred composed capability denials must have distinct
  counters
- no facade call may conceal broad support rediscovery or ad hoc runtime probing
  behind a cheap-looking getter
- raw-root-config validation must happen once per admission posture and produce
  a validated config artifact consumed downstream rather than repeated field
  probing at every witness acquisition

Cost posture rules:

- support reporting must derive from owned registry/matrix artifacts rather than
  rediscovering support by probing behavior repeatedly
- configuration resolution must stay section-local and must not scan unrelated
  sections to decide capability admission
- the facade must deny unsupported compositions before building rich execution
  artifacts for them
- capability acquisition must not trigger lower-runtime behavior probes as a
  substitute for support metadata

## Implementation Phases

Milestone 5.6 should not be implemented as one broad "application facade"
batch. It has four real phases, each with its own proof obligations and
forbidden shortcuts.

### Phase 1: Unified Configuration Foundation

Purpose:

- freeze the unified config root
- freeze subsystem-owned config sections
- make validated configuration a proof-bearing prerequisite for later facade
  work

Must ship:

- `WORTHQueryConfig`
- subsystem-owned config sections
- `ValidatedWORTHQueryConfig`
- `ConfigurationAdmissionError`
- `ConfigurationAdmissionFailureClass`

Must prove:

- config is subsystem-shaped, not capability-shaped
- adding a new config section forces compile errors at construction and
  propagation boundaries
- missing and contradictory section postures fail typed and early

Must not ship:

- capability witnesses
- dynamic capability routing
- bag-shaped root config with placeholder sections

Phase 1 closeout evidence:

- config-section explicitness tests
- compile-fail boundaries around raw/unvalidated config misuse
- exact counters for config validation and section resolution

### Phase 2: Capability Registry And Support Truth

Purpose:

- freeze the capability-family taxonomy
- freeze support-status taxonomy
- make support advertisement derive from one authority path instead of docs or
  facade method presence

Must ship:

- `WORTHQueryCapabilityFamily`
- `WORTHQueryCapabilitySupportStatus`
- `WORTHQueryCapabilityDescriptor`
- `WORTHQueryCapabilityRegistry`
- `WORTHQuerySupportMatrix`
- `WORTHQuerySupportReport`

Must prove:

- support metadata is machine-checkable and registry-owned
- admitted, deferred, and unsupported families are distinguished explicitly
- support reporting does not rediscover or probe lower-runtime behavior

Must not ship:

- one broad "supported_features" bag
- support inference from facade method presence
- human-only support narratives with no executable authority artifact

Phase 2 closeout evidence:

- registry/matrix synchronization tests
- support-report content assertions
- certification rows for support-metadata synchronization

### Phase 3: Typed Capability Witnesses And Facade Admission

Purpose:

- make the application facade real
- expose one explicit acquisition path per capability family
- keep each witness narrow and compiler-enforced

Must ship:

- `WORTHQueryApplicationFacade`
- one admitted witness type per capability family
- one acquisition method per capability family
- `CapabilityAdmissionDecision`
- `CapabilityAdmissionError`
- `CapabilityAdmissionFailureClass`

Must prove:

- witness acquisition consumes `ValidatedWORTHQueryConfig`
- unsupported and deferred families fail typed and early
- witness misuse across families is uncompilable
- the facade is the normative new-work surface

Must not ship:

- `capability(family)` dynamic routing
- `EverythingCapability`
- bool-driven witness shortcuts
- new 5.6 composition-first APIs only on the legacy broad facade

Phase 3 closeout evidence:

- compile-fail witness-boundary tests
- unit tests for admitted acquisition and typed denial
- certification rows for admitted and denied capability families

### Phase 4: Unified Facade Certification And Legacy-Wall Containment

Purpose:

- prove the facade/config story is actually safe to treat as the daily-driver
  surface
- prevent the old broad re-export wall from remaining the operational default

Must ship:

- milestone-native unified facade certification slice
- legacy-wall regression guardrails
- exact counter assertions for lookup, resolution, and denial behavior

Must prove:

- support claims and executable admission behavior stay in sync
- unsupported compositions fail before deep probing or fallback
- the application facade, not the legacy wall, is the normative path for new
  5.6 composition work

Must not ship:

- legacy-only 5.6 composition surfaces
- row-presence-only certification
- hand-wavy "supported in practice" claims

Phase 4 closeout evidence:

- full named certification suite required by `test-requirements.md`
- legacy broad-facade shortcut rejection coverage
- closeout review showing no remaining bag/fallback/legacy dominance trap

## Phase Shape

Milestone 5.6 should implement a proof-bearing chain roughly shaped like:

1. `ConfigurationDeclaration`
2. `ValidatedConfigurationSections`
3. `CapabilityRegistryAndSupportMatrix`
4. `CapabilityAdmissionDecision`
5. `AdmittedCapabilityWitness`
6. `FacadeExposedOperation`
7. `SupportReportOrTypedDenial`

Meaning:

- configuration declaration names the available sectioned shape
- validation proves section presence and section-local legality
- registry/matrix artifacts declare what the product admits, defers, or rejects
- capability admission decision binds one validated config plus one family to a
  typed admitted/deferred/unsupported posture
- capability witnesses prove what this concrete facade instance may do
- facade operations consume the narrow witness rather than rediscovering support
- support reports and denials emit machine-checkable proof of the admitted or
  rejected composition

The executor/facade path must not re-decide support posture after capability
admission.

Required phase-owned artifacts:

- `WORTHQueryConfig`
- `ValidatedWORTHQueryConfig`
- `WORTHQueryCapabilityRegistry`
- `WORTHQuerySupportMatrix`
- `CapabilityAdmissionDecision`
- one admitted witness type per capability family
- `WORTHQuerySupportReport`

## Named Contracts

Milestone 5.6 must name at least these contracts:

- `unified_config_section_resolution`
- `capability_registry_honesty`
- `support_matrix_synchronization`
- `facade_capability_admission`
- `unsupported_composition_denial`

Each contract must name:

- the authority boundary it protects
- the explicit counters it emits
- whether the contract is `Verified` or `Debt`

## Certification Requirements

Milestone 5.6 closes only when the named certification suite from
[test-requirements.md](./test-requirements.md) is implemented as a real
milestone-native slice and proves adversarial behavior rather than row presence.

Required canonical rows:

- unified query-read capability admission
- unified live capability admission
- unified preview capability admission
- unified workflow capability admission
- unified historical capability admission
- unified configuration section explicitness
- support-metadata and executable-admission synchronization

Required rejection rows:

- unsupported composed capability denied
- deferred capability denied explicitly as deferred debt
- invalid unified configuration denied
- cross-capability witness misuse denied
- bag-shaped or bool-driven shortcut construction forbidden
- legacy broad-facade composition shortcut forbidden for new 5.6 capability
  surfaces

Required compile-fail coverage:

- external construction of capability witnesses
- external construction of support-registry/report artifacts that should remain
  authority-owned
- witness misuse across capability families
- bool-driven shortcut surfaces
- direct use of internal application modules outside the facade boundary
- runtime-selected `capability(family)` or equivalent dynamic witness routing
- new 5.6 composition surfaces exposed only through the legacy broad facade

Required adversarial assertions:

- equality assertions where equivalent capability acquisition paths claim parity
- inequality assertions where intentionally different capability families must
  not collapse into one witness/result
- zero assertions for forbidden fallback and forbidden silent widening
- exact counter assertions for capability lookup, config resolution, and denial
  classes

## Representative Scenario Matrix

The milestone should be reviewed against at least these scenarios:

- a developer boots only query-read capability through the unified facade and
  receives an admitted read witness plus honest support metadata
- a developer boots preview plus workflow composition and receives two typed
  capability witnesses rather than one broad application bag
- a developer changes only `config.runtime_bridge` posture and sees preview or
  workflow admission change without any mutation to `config.query`, proving the
  config sections are owner-shaped rather than capability-shaped
- a developer requests historical capability while only runtime-backed support
  is admitted and receives honest admitted or deferred posture, not an ambient
  history flag
- a developer presents invalid unified config where workflow is enabled without
  its owning section and receives a typed config denial before capability
  acquisition
- a developer requests an unsupported composition and the facade denies before
  any deep lower-runtime probing occurs
- a developer attempts to discover capability support by method presence on the
  legacy broad facade and cannot obtain a 5.6-only composed witness that way

## Explicit Non-Goals

Milestone 5.6 does not:

- introduce new deep query semantics comparable to Milestone 5.5
- erase lower-runtime ownership into one synthetic platform runtime
- close store-backed durability, durable artifacts, or later historical/diff
  completion
- justify adding convenience methods that bypass capability witnesses
- bless the legacy broad re-export wall as the normative future-facing design

## Allowed Debt

- deferred durable/store-backed capability families may remain explicit debt
- richer human-facing diagnostics narratives may remain later work if support
  metadata and typed denials are already machine-checkable and honest

Disallowed debt:

- flat bag-shaped unified config
- semantics-erasing facade shortcuts
- support metadata that outruns executable admission
- best-effort fallback for unsupported composed capabilities
- shipping new 5.6 application-composition entrypoints only on the legacy broad
  facade wall

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- one explicit application-facing facade posture exists and is the normative
  new-work surface
- unified configuration remains sectioned by subsystem ownership
- capability witnesses are typed, narrow, and compile-time protected
- support metadata and executable admission behavior remain synchronized
- unsupported and deferred compositions fail typed and early
- new 5.6 application-composition entrypoints exist on the application facade
  and are not introduced solely on the legacy broad facade wall
- the milestone-native certification suite emits the canonical artifacts
  required by [test-requirements.md](./test-requirements.md):
  - `query_digest`
  - `plan_digest`
  - `support_matrix_digest`
  - `capability_registry_digest`
  - `counter_snapshot`

## Closeout Rule

Milestone 5.6 is not closed because a facade type exists or because configuration
was gathered under one root struct.

It is closed only when:

- developers can use `worth-query` as the daily-driver import for the admitted
  application-facing capability mix
- lower-crate authority boundaries are still visible and typed
- configuration still teaches subsystem ownership instead of hiding it
- support claims, executable behavior, and certification artifacts agree
- the legacy broad facade no longer receives new 5.6 composition-first API
  growth that would keep it operationally dominant
- the new facade/configuration posture is architecturally cleaner than the
  legacy broad import wall rather than merely coexisting beside it
