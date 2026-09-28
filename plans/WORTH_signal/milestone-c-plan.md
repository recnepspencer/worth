# Milestone C Engineering Spec: Async Resource Policy Families

> **Status:** Closed
>
> **Closeout:** [milestone-c-closeout.md](./milestone-c-closeout.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Vision parents:**
> - [worth_signals2.md](./worth_signals2.md)
> - [worth_signal_vision.md](./worth_signal_vision.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-a-closeout.md](./milestone-a-closeout.md)
> - [milestone-b-closeout.md](./milestone-b-closeout.md)
>
> **Primary architectural driver:** make async/resource policy variation a
> descriptor-backed runtime substrate so retry, timeout, cancellation,
> supersession, revalidation, observation, output continuity, retention,
> diagnostics, and replay compatibility stop being adapter folklore.

## Summary

Milestone C completes the policy layer above the async/resource substrate closed
by Milestone B.

This milestone is not "add more knobs to resource nodes."

It is:

- deterministic async/resource policy registries
- frozen descriptor identity for every policy choice that can affect runtime
  behavior
- policy lowering before request admission, temporal wake allocation,
  completion admission, observation, retention, diagnostics, or replay
- built-in policy families for retry/backoff, timeout/deadline, cancellation,
  supersession, revalidation/freshness, observation, output continuity,
  retention, diagnostics, and replay compatibility
- typed denial for unknown, duplicate, incompatible, budget-exhausted, and
  semantically illegal policy decisions
- proof-bearing force, eligibility, compatibility, and budget tokens
- policy-specific boundary performance envelopes and certification artifacts
- replay-honest compatibility handling when policy versions or descriptors
  drift

The governing rule is:

`declare policy once, lower it once, execute only descriptor truth, replay or deny explicitly`

If route resources, query views, form actions, or browser adapters still need
to invent retry, timeout, visibility, retention, or freshness semantics above
the runtime after this milestone, the milestone is incomplete.

## 1. Goal

Make async/resource policy variation first-class in `worth-signal` so that:

- policy choices are deterministic runtime artifacts, not host callback
  behavior
- retry, timeout, cancellation, supersession, revalidation, observation,
  output-continuity, retention, diagnostics, and replay decisions are
  descriptor-backed
- policy variation cannot weaken hard lifecycle laws established by Milestone B
- replay, restore, branch re-entry, diagnostics, and certification can explain
  exactly which policy decision changed operational behavior
- higher-level resource products can consume one runtime policy substrate
  rather than defining parallel state machines

## 2. Why This Milestone Exists

Milestone B closed the async/resource lifecycle substrate. The runtime now owns
request identity, in-flight state, completion admission, cancellation, timeout,
retry, revalidation, branch restore, replay reconstruction, diagnostics
summaries, retained lifecycle history, and performance closeout evidence.

That substrate deliberately did not complete every policy product. It froze the
extension boundary so Milestone C could make policy variation rigorous instead
of letting it accrete in app layers.

The missing policy category is now the biggest risk to the next product layer.
Route resources, query-backed views, form actions, background refresh, long-
lived subscriptions, offline/reconnect flows, and operator-facing tools all
need richer behavior than one fixed retry strategy, one fixed timeout shape, one
visibility mode, and one retention posture.

Without Milestone C:

- route-resource and query layers will each invent separate retry/backoff,
  deadline, stale-after, and visibility semantics
- forms and background actions will invent separate cancellation and
  supersession semantics
- output continuity will become UI-local display state instead of runtime
  explanation
- retention truncation and diagnostics richness will drift into policy bugs
  instead of typed availability outcomes
- replay may silently reinterpret old async truth under whatever policy code is
  linked into the current process
- product code will own behavior that branch restore, certification, and
  diagnostics need to understand

Milestone C exists to prevent that drift while the policy surface is still small
enough to freeze honestly.

## 3. Hard Part

The hard part is not enumerating retry modes.

The hard part is freezing one exact truth-preserving relationship among:

- policy declaration
- policy registry identity
- semantic policy name and version
- parameter-sensitive descriptor digest
- policy selection basis
- compatibility posture
- lowered hot-path decision form
- temporal wake allocation
- request, generation, attempt, and branch epoch proof
- completion admission and denial
- observation and output continuity
- retained history and diagnostics richness
- replay compatibility or typed incompatibility
- boundary performance envelope

The design fails if:

- policy callbacks can decide runtime legality inside completion or retry hot
  paths
- changing a policy parameter does not change the descriptor digest
- policy variation can alter request identity, generation semantics, branch
  epoch matching, or stale-completion denial
- retry storms, timer churn, host cancellation fanout, observation delivery, or
  diagnostics expansion hide behind cheap-looking APIs
- replay silently reinterprets historical async state under missing or
  incompatible policy descriptors
- output visibility mutates lifecycle truth
- retention pruning drops history without a typed unavailable or omitted
  artifact
- diagnostics richness decides policy truth rather than materializing retained
  policy evidence

## 4. Explicit Assumptions

- `worth-relational` remains the owner of truth identity, mutation, history,
  diffs, and traversal.
- `worth-store` remains the owner of persistence when persistence is involved.
- `worth-signal` remains the owner of derived execution and async/resource
  lifecycle truth.
- Milestone A temporal semantics are available and remain the only runtime time
  substrate for retry, backoff, timeout, stale-after, and deadline decisions.
- Milestone B async/resource lifecycle law is closed and may be extended only by
  descriptor-backed policy decisions, not weakened.
- hosts may execute external work and may receive advisory cancellation signals,
  but hosts do not decide runtime completion legality.
- this milestone is core-only; wasm, React, route-resource, form, browser cache,
  and query product APIs consume this substrate later.
- policy descriptors are runtime artifacts. Arbitrary host closures may execute
  external work, but replay-critical policy meaning must be represented by
  deterministic descriptors.

## 5. Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is adversarial-constraint-first
  design. Policy work must begin from retry storms, timeout drift, cancellation
  races, incompatible replay, and hidden cost surfaces, not from ergonomic
  option builders.
- `arch_laws.md`
  The most important laws here are 17, 20, 21, 24, 27, 30, 34, 36, 37, 40, and
  41. Policy resolution must be isolated from execution, public APIs must
  reveal orchestration boundaries, diagnostics must not own truth, rejection
  must precede construction, executors must consume lowered plans, managed
  resources must stay framework-owned, replay must reconstruct from checkpoint
  plus bounded journal, invalid policy transitions must be unrepresentable,
  semantic categories must remain distinct, and proof-bearing policy types must
  encode what has been established.
- `perf_laws.md`
  The most important thing it protects is policy cost honesty. Retry/backoff,
  timeout, cancellation fanout, observation delivery, retention, diagnostics,
  and replay compatibility must expose counters at the boundary that performs
  the work rather than hiding broad scans or cold reconstruction behind facade
  reads.
- `domain_laws.md`
  The most important thing it protects is responsibility shape. Policy
  registry, retry/backoff, timeout/deadline, cancellation/supersession,
  revalidation/freshness, observation/output continuity, retention/diagnostics,
  replay compatibility, and certification need named subsystems instead of one
  broad resource-policy helper.
- `worth_signals2.md`
  The most important thing it protects is the runtime thesis: policy-aware,
  branchable, replayable, diagnosable derived computation is a core substrate
  responsibility, not adapter convenience.
- `worth_signal_vision.md`
  The most important thing it protects is the authority boundary:
  `worth-signal` owns derived execution semantics while remaining standalone,
  deterministic, transactional, auditable, and decoupled from truth storage.
- `signal_architecture2.md`
  The most important thing it protects is proof-bearing pipeline structure and
  cost-visible boundaries. Milestone C must lower policy before execution, carry
  proof types phase-to-phase, expose boundary envelopes, and preserve hot/cold
  separation for diagnostics and retained history.
- `worth_signal_temporal_async_roadmap.md`
  The most important thing it protects is sequencing and scope. Milestone C
  comes after temporal ownership and async/resource lifecycle ownership because
  policy families must consume canonical time, request identity, branch epoch,
  completion admission, replay, diagnostics, and performance envelopes rather
  than define them.
- `test-requirements.md`
  The most important thing it protects is certification under hostile async
  policy grammars. Milestone C is not closed until async policy family
  certification, retry/backoff, timeout/deadline, cancellation/supersession,
  revalidation/freshness, observation/output-continuity, and retention/replay
  compatibility all emit machine-checkable artifacts.
- `milestone-a-closeout.md`
  The most important thing it protects is the closed temporal substrate.
  Milestone C must consume runtime-owned clock basis, wake ordering, stale-after
  semantics, interval behavior, and diagnostics-visible temporal provenance.
- `milestone-b-closeout.md`
  The most important thing it protects is the closed async/resource lifecycle
  substrate. Milestone C must extend descriptor-backed policy variation without
  reopening request identity, generation, branch epoch, completion admission,
  denial classification, retained lifecycle history, or completion performance
  law.

## 6. Adversarial Constraint

Milestone C must survive the following hostile condition:

> A branchable, replayable runtime with deterministic execution, rollback-safe
> observation, runtime-owned time, and async/resource lifecycle truth must apply
> retry, timeout, cancellation, supersession, revalidation, observation, output
> continuity, retention, diagnostics, and replay policies from deterministic
> descriptors so equivalent compatible histories converge to the same resource
> truth and incompatible histories deny with typed compatibility evidence.

Concretely, the design must remain correct when all of the following are true:

- policies are registered in different orders but with the same semantic
  content
- duplicate policy ids and names attempt to enter a registry
- a resource declaration references unknown policy identity
- policy parameters drift between checkpoint capture and restore
- deterministic jitter is replayed after branch restore
- retry budgets are exhausted under concurrent retry pressure
- timeouts race success, cancellation, and revalidation
- cancellation host signalling fails but runtime-hard denial must still hold
- supersession permits overlapping generations under one policy and denies old
  completions under another
- observer-demand revalidation races dependency-change revalidation
- output is preserved or hidden while pending under different visibility
  policies
- retained history is pruned by policy before a later diagnostics or replay
  request
- diagnostics expansion is budget-denied after retention has dropped rich
  history
- replay sees missing, incompatible, or semantically drifted policy descriptors

If any supported path lets policy variation decide lifecycle law after request
admission, silently reinterpret historical state, hide broad policy work, or
collapse output visibility into lifecycle truth, the milestone has failed.

## 7. Product Decision Lock

- policy variation is runtime-owned descriptor truth, not adapter-local
  callback folklore
- all replay-relevant policy behavior must pass through frozen registries and
  lowered descriptors
- built-in and caller-registered policies use the same declaration, freeze,
  lower, execute, record, and version lifecycle
- unknown, duplicate, missing, incompatible, semantically illegal, and
  budget-exhausted policy decisions deny explicitly
- policy may alter eligibility, timing, host advisory signalling, visibility,
  retained richness, diagnostics, and compatibility posture
- policy may not alter request identity, generation identity, attempt lineage,
  branch epoch matching, stale completion denial, denied-completion non-apply,
  or rollback suppression law
- retry remains distinct from revalidation
- cancellation remains distinct from supersession
- timeout remains distinct from rejection and cancellation
- output continuity remains distinct from lifecycle state
- observation policy cannot mutate resource truth
- diagnostics policy cannot reconstruct or decide lifecycle truth unless an
  explicit cold-work budget admits diagnostics expansion
- replay compatibility is a first-class typed result, not a best-effort warning

Normative consequence:

- any implementation that stores policy meaning only in Rust closures is out of
  spec
- any implementation that lets policy callbacks run before descriptor lowering
  as the source of replay-critical truth is out of spec
- any implementation that changes retry/backoff behavior without changing a
  descriptor digest is out of spec
- any implementation that treats "cancel host work" as equivalent to
  runtime-hard completion denial is out of spec
- any implementation that collapses previous-output visibility and committed
  lifecycle truth into one `Result`-like value is out of spec
- any implementation that returns a normal replay summary after policy
  incompatibility is out of spec
- any implementation that hides diagnostics reconstruction behind an ordinary
  retained summary read is out of spec
- any implementation that makes policy registry order affect canonical
  descriptor identity is out of spec

## 8. Scope

### 8.1 In Scope

- async/resource policy registry completion
- deterministic policy ids, semantic names, versions, descriptors, digests,
  selection bases, and compatibility postures
- retry and backoff policy families
- timeout and deadline policy families
- cancellation and supersession policy families
- revalidation and freshness policy families
- observation and output-continuity policy families
- retention, diagnostics, and replay compatibility policy families
- policy denial classifications
- proof-bearing force, eligibility, budget, compatibility, and advisory tokens
- boundary performance envelopes for policy decisions
- canonical certification artifacts for every required policy family
- compile-fail fixtures for descriptor construction, force tokens, eligibility
  proofs, compatibility proofs, and registry/facade boundaries

### 8.2 Explicitly Out Of Scope

- wasm bindings
- React, Angular, or browser-store resource adapters
- route-resource APIs
- form/action APIs
- query replacement product facade work
- network transport implementation
- persistent cache storage beyond canonical runtime artifacts required for
  replay and compatibility proof
- domain-specific cache products above the generic policy families
- UI-specific loading, transition, optimistic rendering, or error display
  ergonomics

## 9. Current-State Assessment

The runtime is structurally ready for this milestone in several ways:

- Milestone A closed runtime-owned time, scheduled wakes, stale-after,
  interval, previous-value, temporal diagnostics, and temporal performance
  evidence
- Milestone B closed runtime-owned resource lifecycle, request identity,
  generation/attempt/branch epoch proof, completion admission, cancellation,
  timeout, retry, revalidation, branch restore, replay reconstruction,
  diagnostics expansion, retention compaction, and performance closeout
- resource descriptors and policy registry vocabulary already exist as the
  intended extension boundary
- public async/resource reports already carry boundary performance envelopes
- compile-fail fixtures already protect many proof constructors and facade
  boundaries
- closeout certification already proves first-ship lifecycle families and
  hostile completion denial lanes

The missing policy category is still real:

- policy registries are not yet the complete product surface for every resource
  policy family
- retry/backoff policies are not yet rich enough to cover budget scopes,
  deterministic jitter, failure classes, duplicate retry coalescing, and retry
  storm denial
- timeout/deadline policies are not yet rich enough to cover per-attempt,
  lifetime, inherited deadline, progress-heartbeat extension, and terminal vs
  revalidation-eligible timeout classifications
- cancellation/supersession policies are not yet rich enough to cover
  host-advisory signalling, grace periods, dependent resource propagation,
  overlapping generations, intent equivalence, and old-host-work posture
- revalidation/freshness policies are not yet rich enough to cover
  observer-demand, terminal-state, fulfilled-only, forced active-handle, and
  coalesced revalidation
- observation/output-continuity policies are not yet rich enough to cover
  lifecycle-only, denied-completion, retry-schedule, pending visibility, and
  terminal output continuity variants
- retention/diagnostics/replay policies are not yet rich enough to cover
  lifecycle history budgets, denied-completion retention, retry lineage,
  compacted terminal records, diagnostics budget posture, and policy version
  compatibility

This means the substrate is ready, but the policy layer is not yet complete
enough for higher-level resource products to inherit without inventing their
own semantics.

## 9.1 QA Findings Folded Into This Spec

This revision intentionally corrects the weaknesses that would make the plan
too abstract to implement safely.

### Finding 1: The Spec Named Policy Families But Not Their Physical Home

What was wrong:

- the prior plan said policy families needed named responsibilities, but did
  not give an implementation topology precise enough to prevent a larger
  `policy.rs` or `resource.rs` from absorbing everything

Why it matters:

- Milestone B already has dense resource files. Without a concrete topology,
  Milestone C would naturally accrete into existing broad modules and lose the
  single-responsibility structure the spec claims to require.

Authority violated:

- `domain_laws.md` single-responsibility and domain alignment
- `arch_laws.md` laws 1, 10, 13, and 16

Required correction:

- Milestone C must name the production and test module topology before feature
  implementation begins.

### Finding 2: Descriptor Truth Was Not Yet A Compile-Time Pipeline

What was wrong:

- the plan said declaration, descriptor, decision, and outcome are separate,
  but did not define a phase chain where output type `K` becomes input type
  `K+1`

Why it matters:

- an implementer could still validate descriptors at runtime, pass raw
  declarations into decision code, or defensively re-resolve policy inside the
  hot path.

Authority violated:

- `arch_laws.md` laws 24, 27, 30, 37, and 41
- `MENTALITY.md` mechanical enforcement over convention

Required correction:

- the spec must require phase-typed policy forms and compile-fail fixtures for
  every illegal shortcut.

### Finding 3: Custom Policy Extensibility Was Too Permissive

What was wrong:

- the plan allowed custom policy families where descriptors can represent
  replay meaning, but did not explicitly forbid executable closure identity,
  process-local function pointers, or opaque serializer blobs from becoming the
  practical custom-policy path

Why it matters:

- replay-compatible "custom policy" is the easiest place to smuggle
  non-deterministic behavior. If C allows arbitrary callbacks now, later product
  adapters will build on it and make compatibility enforcement impossible.

Authority violated:

- `arch_laws.md` laws 17, 26, 36, 40, and 41
- `perf_laws.md` no hidden path conflation or repeated rediscovery

Required correction:

- first-ship custom policy support must be descriptor-only unless a custom
  policy can lower into deterministic, versioned, data-only decision tables
  with certified compatibility.

### Finding 4: Replay Compatibility Needed A Default-Deny Matrix

What was wrong:

- the plan named compatibility posture but did not define a concrete default
  matrix for identical, compatible, missing, unknown, version-drifted,
  parameter-drifted, and semantics-drifted descriptors

Why it matters:

- compatibility checks are where "probably fine" becomes silent historical
  reinterpretation. Default allow is catastrophic.

Authority violated:

- `arch_laws.md` laws 7, 12, 36, and 40
- `perf_laws.md` explicit policy degradation

Required correction:

- policy compatibility must default to deny unless a typed compatibility proof
  explains the exact allowed drift.

### Finding 5: Certification Rows Needed Implementation-Useful Scenario Names

What was wrong:

- the prior plan listed hostile conditions but did not give enough named
  scenario rows for implementers to map directly into tests and closeout
  artifacts

Why it matters:

- broad hostile condition lists often collapse into one heroic test that proves
  less than it claims. Milestone C needs row-level proof like Milestone B.

Authority violated:

- `MENTALITY.md` test the architecture, not just behavior
- `arch_laws.md` laws 7, 8, and 32

Required correction:

- the spec must include named scenario and performance closeout rows with exact
  purposes and trap conditions.

## 10. Architecture Rules For This Milestone

### 10.1 Policy Is A Runtime Subsystem, Not A Bag Of Options

Async/resource policy must be modeled as a first-class runtime subsystem with
owned registries, descriptors, lowering, compatibility, diagnostics, and
certification.

It must not be implemented as:

- ad hoc enums attached directly to resource declarations with no registry
  identity
- host callbacks that decide replay-critical policy meaning
- per-product option structs that bypass the core resource policy surface
- serializer-only metadata with no hot-path proof form

Required consequence:

- resource declarations reference policy descriptors or named built-ins
- every descriptor has id, semantic name, version, digest, selection basis,
  compatibility posture, and cost-contract identity
- descriptor lowering happens before any execution work is constructed

### 10.2 Declaration, Descriptor, Decision, And Outcome Are Separate

Policy APIs must preserve four distinct categories:

- declaration: what a caller requested
- descriptor: the frozen canonical runtime identity of that request
- decision: the lowered decision made at a specific runtime boundary
- outcome: the lifecycle, observation, retention, diagnostics, or replay result
  after applying the decision

Required consequence:

- declarations may fail before descriptor construction
- descriptors are stable and replay-visible
- decisions carry boundary-specific proof and counters
- outcomes may not be used as proof that a policy decision was legal unless
  they carry the admitted decision evidence

### 10.3 Lifecycle Law Dominates Policy Preference

Policy variation may choose among legal runtime behaviors. It may not redefine
resource lifecycle law.

Required consequence:

- stale completions still deny
- superseded, cancelled, timed-out, retired, malformed, contradictory,
  duplicate, and unknown completions remain distinct denial classes
- generation, attempt, branch epoch, and ordinal categories remain distinct
- rollback and observation suppression law remains unchanged
- denied completions may be observed or retained by policy, but may not mutate
  committed resource state

### 10.4 Policy Resolution Must Precede Hot Execution

Retry, timeout, cancellation, supersession, revalidation, observation, output
continuity, retention, diagnostics, and replay compatibility decisions must be
resolved into lowered proof forms before the hot path consumes them.

Required consequence:

- completion admission does not call policy code to rediscover retry,
  visibility, or retention behavior
- temporal wake allocation consumes lowered retry/timeout/deadline decisions
- observation delivery consumes lowered observation/output-continuity decisions
- retention compaction consumes lowered retention policy decisions
- replay consumes descriptor compatibility artifacts rather than executing
  current policy callbacks against old history

### 10.5 Registry Construction Must Be Deterministic And Closed Before Use

Policy registries must behave like frozen runtime construction artifacts.

Required consequence:

- duplicate id and duplicate semantic-name registration fail before runtime use
- canonical registry digest is stable independent of registration order
- unknown policy references fail before request admission
- missing or incompatible descriptors on restore produce typed compatibility
  denials
- built-ins and caller-registered policies share the same registry path

### 10.6 Policy Budgets Are Runtime Truth

Retry budgets, elapsed retry windows, deadline budgets, diagnostics budgets,
retention budgets, and replay compatibility budgets must be proof-bearing
runtime objects.

Required consequence:

- budget exhaustion denies with typed evidence before expensive construction
- budget scope is explicit: request, node, family, runtime, or caller-declared
  scope
- budget counters are exposed at the decision boundary
- budget state is branch-, restore-, and replay-honest

### 10.7 Host Advisory Effects Are Separate From Runtime Denial

Some policies may ask a host to cancel external work, extend a heartbeat, or
interrupt transport. Those are advisory host effects, not runtime truth.

Required consequence:

- host cancellation failure cannot allow late completion to commit
- best-effort host signal count and runtime-hard denial count are separate
- host advisory width is measured separately from affected request footprint
- external work continuation after supersession is legal only if runtime denial
  still rejects stale completion later

### 10.8 Observation And Output Continuity Must Not Collapse Lifecycle

Policy may decide what observers see and whether previous output remains
visible while resource state is pending, rejected, timed out, cancelled, or
superseded. It may not decide what lifecycle state is committed.

Required consequence:

- lifecycle digest and output-continuity digest stay separate
- observer packets remain commit-bounded and rollback-safe
- visibility policies are replayable and diagnostics-visible
- hiding an output while pending does not erase the previous committed output
  history
- preserving an output while pending does not imply fulfilled lifecycle state

### 10.9 Retention And Diagnostics Must Be Availability-Aware

Policy may retain all lifecycle history, terminal summaries, denied
completions, retry lineage, or compacted terminal records. When history is not
available, the runtime must say so explicitly.

Required consequence:

- ordinary retained summary reads perform zero cold reconstruction
- diagnostics expansion is a named, budgeted cold-work API
- unavailable, omitted, pruned, compacted, retained, reconstructed, denied, and
  incompatible outcomes remain distinct
- retention policy loss may deny rich replay or diagnostics, but may not invent
  lifecycle facts

### 10.10 Replay Compatibility Is A Typed Boundary

Replay and restore must compare historical policy descriptors against current
registry descriptors.

Required consequence:

- compatible histories emit compatibility evidence
- incompatible histories deny with typed incompatibility artifacts
- missing descriptors deny with missing-policy artifacts
- semver-compatible policy evolution must still prove descriptor compatibility
  at the runtime boundary
- replay never silently upgrades or downgrades policy meaning

### 10.11 Policy Work Must Stay Breadth-Bounded

Every policy family must name the cost basis it introduces.

Required consequence:

- retry/backoff cost is stated in decision width, temporal wake footprint, and
  budget scope touches
- timeout/deadline cost is stated in temporal frontier width and affected
  request count
- cancellation/supersession cost is stated in affected request footprint and
  host-signal advisory width separately
- revalidation cost is stated in active-handle proof checks and coalescing width
- observation/output-continuity cost is stated in candidate width, coalesced
  width, and delivery width
- retention/diagnostics cost is stated in retained summary reads, pruned
  records, cold reconstruction, and diagnostics budget consumption
- replay compatibility cost is stated in descriptor count and retained policy
  history width, not total graph size

## 11. Required Architecture Changes

### 11.0 Required Production Module Topology

Milestone C must split policy work by domain responsibility before broadening
behavior.

The exact filenames may evolve if the surrounding module tree has already been
split differently, but the responsibilities may not collapse back into one
policy file or one runtime method.

Required data-module responsibilities under
`crates/worth-signal/src/data/resource/`:

- `policy/identity.rs`
  policy ids, semantic names, versions, descriptor digests, registry digests,
  and compatibility posture identities
- `policy/registry.rs`
  mutable builder, frozen registry, duplicate detection, built-in registration,
  canonical digest construction, and registry freeze reports
- `policy/declaration.rs`
  public declaration forms that callers can request but cannot treat as
  admitted proof
- `policy/descriptor.rs`
  frozen data-only descriptors and parameter digests
- `policy/lowering.rs`
  declaration-to-descriptor resolution and lowered bundle construction
- `policy/decision.rs`
  family-neutral decision envelope categories and decision digests
- `policy/denial.rs`
  typed denial topology for unknown, duplicate, missing, incompatible,
  malformed, budget-exhausted, semantically illegal, and unsupported decisions
- `policy/budget.rs`
  retry, retention, diagnostics, deadline, and replay budget scopes plus
  proof-bearing admissions and denials
- `policy/compatibility.rs`
  replay/restore compatibility matrix and compatibility proof artifacts
- `policy/families/retry.rs`
- `policy/families/timeout.rs`
- `policy/families/cancellation.rs`
- `policy/families/supersession.rs`
- `policy/families/revalidation.rs`
- `policy/families/observation.rs`
- `policy/families/output_continuity.rs`
- `policy/families/retention.rs`
- `policy/families/diagnostics.rs`
- `policy/families/replay.rs`
- `policy/certification.rs`
  Milestone C certification records, scenario rows, performance closeout rows,
  and final run artifacts

Required runtime-module responsibilities under
`crates/worth-signal/src/logic/transaction/runtime/state/resource/` or the
nearest existing resource runtime subtree:

- `policy_registry.rs`
  runtime-owned frozen registry access and descriptor lookup
- `policy_budget.rs`
  branch-, restore-, and replay-honest budget state
- `policy_decision.rs`
  lowered decision execution entrypoints that consume descriptor proofs
- `policy_retention.rs`
  retained policy history, pruning, compaction, and availability outcomes
- `policy_replay.rs`
  restore/replay compatibility classification from retained descriptors
- `policy_diagnostics.rs`
  budgeted policy explanation and cold reconstruction

Required test responsibilities:

- `resource_policy_registry.rs`
- `resource_policy_retry.rs`
- `resource_policy_timeout.rs`
- `resource_policy_cancellation.rs`
- `resource_policy_supersession.rs`
- `resource_policy_revalidation.rs`
- `resource_policy_observation.rs`
- `resource_policy_retention.rs`
- `resource_policy_replay.rs`
- `resource_policy_certification.rs`
- compile-fail fixtures under `crates/worth-signal/tests/ui` for every
  public/private boundary named in this spec

Anti-patterns:

- do not expand the existing resource runtime state file with all policy family
  decision logic
- do not put retry, timeout, cancellation, revalidation, observation, retention,
  diagnostics, and replay policy in one `policy.rs`
- do not model family-specific decisions as stringly policy names with
  catch-all payload maps
- do not expose policy family internals through the crate facade because tests
  need them

### 11.1 Complete The Resource Policy Registry

The existing resource policy registry surface must become the canonical policy
subsystem for all Milestone C families.

It should own:

- policy ids
- semantic names
- semver-like versions or direct canonical equivalents
- parameter digests
- descriptor digests
- compatibility postures
- selection bases
- cost-contract ids
- built-in and caller-registered descriptor catalogs

It must produce:

- registry freeze artifact
- canonical registry digest
- duplicate id/name denial
- unknown policy denial
- missing descriptor denial
- incompatible descriptor denial
- lowered descriptor bundle for resource declaration and replay

### 11.1.1 Required Policy Proof Pipeline

Milestone C must encode the policy lifecycle as proof-widening types. The
names may vary, but the pipeline shape may not.

Required phase chain:

```text
ResourcePolicyDeclaration
  -> ValidatedResourcePolicyDeclaration
  -> FrozenResourcePolicyDescriptor
  -> LoweredResourcePolicyBundle
  -> ResourcePolicyDecisionPlan
  -> AdmittedResourcePolicyDecision | DeniedResourcePolicyDecision
  -> CommittedResourcePolicyArtifact | RetainedResourcePolicyDenial
  -> ResourcePolicyInspection | ResourcePolicyReplayCompatibility
  -> ResourceMilestoneCPolicyCertificationRecord
```

Compile-time requirements:

- raw declarations cannot enter retry, timeout, cancellation, revalidation,
  observation, retention, diagnostics, or replay execution
- frozen descriptors cannot be constructed outside registry freeze/lowering
- lowered bundles cannot be constructed from arbitrary descriptor vectors
- decision plans cannot be built without a lowered bundle and concrete runtime
  boundary basis
- admitted decisions cannot be created by external callers or reused across
  branch epochs
- denied decisions cannot be spent as admitted decisions
- committed policy artifacts cannot be constructed without consuming an
  admitted decision or retained denial
- inspection artifacts cannot be WORTHd as certification records
- replay compatibility proof cannot be WORTHd from descriptor equality alone

Required trybuild fixtures:

- external code cannot construct `FrozenResourcePolicyDescriptor`
- external code cannot construct `LoweredResourcePolicyBundle`
- external code cannot construct `ResourcePolicyDecisionPlan`
- external code cannot construct `AdmittedResourcePolicyDecision`
- external code cannot construct `DeniedResourcePolicyDecision`
- external code cannot pass `ResourcePolicyDeclaration` to a decision executor
- external code cannot spend a retry eligibility token as a revalidation token
- external code cannot spend a host advisory cancellation token as runtime-hard
  cancellation proof
- external code cannot construct replay compatibility proof from raw digests
- external code cannot deserialize certification records as proof objects

Runtime assertions are allowed only at untrusted boundaries. Inside this chain,
defensive re-proof is a defect unless the proof crosses a trust, version,
branch, restore, or serialization boundary.

### 11.1.2 Custom Policy First-Ship Rule

Milestone C may expose caller-registered custom policy descriptors only if the
custom behavior is data-describable and replay-compatible.

Allowed first-ship custom forms:

- named descriptor aliases over built-in families
- parameterized built-in descriptors registered under caller-owned ids
- deterministic table-driven policies whose complete decision table is part of
  the descriptor digest
- compatibility adapters that only classify old descriptors as compatible,
  incompatible, missing, or unavailable and never execute runtime behavior

Disallowed first-ship custom forms:

- process-local function pointers as replay authority
- closures whose behavior is not fully represented by descriptor data
- opaque serialized blobs that the runtime cannot canonicalize and compare
- callbacks that perform IO, consult ambient clocks, inspect host mutable state,
  or allocate external work during policy resolution
- custom policies that can classify stale, superseded, cancelled, timed-out, or
  malformed completions differently from Milestone B lifecycle law

If a future custom policy needs executable host code, it must be modeled as a
separate later milestone with:

- deterministic descriptor identity
- explicit capability limits
- replay compatibility contracts
- sandboxed decision inputs
- no hot-path callback truth
- compile-fail proof boundaries

Milestone C is allowed to defer full executable custom policy support. It is
not allowed to ship a weak custom callback escape hatch.

### 11.2 Introduce Policy Decision Artifacts

Each family needs a decision artifact that records which descriptor controlled
the boundary and what work was admitted or denied.

Expected forms include direct canonical equivalents of:

- `ResourceRetryPolicyDecision`
- `ResourceTimeoutPolicyDecision`
- `ResourceCancellationPolicyDecision`
- `ResourceSupersessionPolicyDecision`
- `ResourceRevalidationPolicyDecision`
- `ResourceObservationPolicyDecision`
- `ResourceOutputContinuityDecision`
- `ResourceRetentionPolicyDecision`
- `ResourceDiagnosticsPolicyDecision`
- `ResourceReplayCompatibilityDecision`

Each decision artifact must expose:

- descriptor id/name/version/digest
- selection basis digest
- decision outcome
- denial class when denied
- boundary performance envelope
- canonical decision digest

### 11.3 Add Proof-Bearing Policy Tokens

Milestone C must introduce sealed tokens for policy decisions that later phases
consume.

At minimum, architecture should preserve distinct forms for:

- retry eligibility
- retry budget admission
- retry storm denial
- deterministic jitter basis
- timeout eligibility
- inherited deadline proof
- progress heartbeat extension proof
- runtime-hard cancellation proof
- host cancellation advisory proof
- supersession admission
- overlapping generation admission
- intent-equivalence coalescing proof
- active-handle revalidation proof
- forced revalidation token
- observation delivery posture
- previous-output visibility posture
- retention budget admission
- diagnostics expansion budget admission
- replay compatibility proof
- replay incompatibility denial

These tokens must be private-field and module-sealed. External callers may
request policy behavior; they may not synthesize the proof that behavior was
admitted.

### 11.4 Split Policy Storage By Hot, Retained, And Diagnostics Surfaces

Policy state must not become one all-purpose registry map.

Required storage categories:

- hot descriptor lookup for request admission and completion-time decisions
- temporal policy frontier data for retry/backoff and timeout/deadline wakes
- budget-scope state for retry, diagnostics, retention, and replay
  compatibility
- retained policy history for replay and diagnostics
- cold diagnostics detail for policy explanation
- certification artifacts for family closeout

The storage split must preserve Milestone B's separation between operational,
retained-history, diagnostics, and facade/report allocation lanes.

### 11.5 Extend Boundary Performance Envelopes For Policy Families

Milestone B introduced boundary performance envelopes. Milestone C must extend
them instead of inventing a separate cost surface.

Expected additions:

- policy family kind
- policy descriptor count
- decision width
- budget scope touches
- temporal wake footprint
- affected request footprint
- host advisory width
- coalescing width
- output-continuity classification width
- retained history write/prune/compact counts
- diagnostics cold reconstruction counts
- compatibility descriptor comparison counts
- policy-specific density strategy where relevant

### 11.6 Add Policy Certification Artifacts

Milestone C needs a closeout gate comparable to Milestone B's final
certification run.

Expected forms include direct canonical equivalents of:

- `ResourceMilestoneCPolicyCertificationBundle`
- `ResourceMilestoneCPolicyScenarioMatrix`
- `ResourceMilestoneCPolicyPerformanceCloseout`
- `ResourceMilestoneCCertificationRun`

The final run must require:

- complete policy family certification records
- hostile scenario rows for every required family
- replay compatibility and incompatibility evidence
- performance closeout claims for every policy family
- compile-fail boundary evidence for all proof constructors

### 11.7 Required Replay Compatibility Matrix

Policy compatibility must default to denial. Compatibility is admitted only
when a typed compatibility proof names the allowed drift.

Required classifications:

| Classification | Meaning | Required outcome |
| --- | --- | --- |
| `IdenticalDescriptor` | id, semantic name, version, parameters, digest, and family semantics match exactly | replay may proceed with identical proof |
| `CompatibleParameterExpansion` | new descriptor adds optional parameters whose default is provably identical to the historical behavior | replay may proceed with compatibility proof that names every defaulted parameter |
| `CompatibleRetentionNarrowing` | retained richness is lower, but canonical lifecycle, denial, and output-continuity conclusions remain provable | replay may proceed for canonical truth and must mark rich history unavailable |
| `CompatibleDiagnosticsRichnessChange` | diagnostics tier/richness changed without changing runtime truth | replay may proceed for truth and must expose diagnostics availability posture |
| `MissingDescriptor` | historical descriptor id/version is absent from the current registry and no retained descriptor is available | replay denies with missing-policy artifact |
| `UnknownFamily` | descriptor family is not supported by this runtime | replay denies with unknown-family artifact |
| `VersionIncompatible` | semantic version or explicit compatibility posture says behavior may differ | replay denies with incompatible-version artifact |
| `ParameterDigestDrift` | descriptor id/version match but parameter digest differs outside an admitted compatibility adapter | replay denies with parameter-drift artifact |
| `DecisionSemanticsDrift` | descriptor shape matches but compatibility adapter reports changed decision meaning | replay denies with semantics-drift artifact |
| `BudgetHistoryUnavailable` | replay needs budget history pruned by retention policy | replay denies rich policy replay or emits unavailable classification according to the access lane |

Default rule:

- any classification not explicitly admitted by a sealed compatibility proof is
  incompatible
- compatibility proof must bind both historical and current descriptor digests
- compatibility proof must expose which canonical conclusions remain valid
- compatibility proof must expose which retained or diagnostics details are
  unavailable
- compatibility proof may not be inferred from semantic version text alone

### 11.8 Minimum Built-In Descriptor Matrix

Milestone C must ship concrete built-ins rather than only an extension
framework.

Minimum first-ship built-ins:

| Family | Built-ins required for C closeout |
| --- | --- |
| Retry/backoff | `Disabled`, `FixedDelay`, `ExponentialBackoff`, `CappedExponentialBackoff`, `DeterministicJitter`, `MaxAttempts`, `MaxElapsedWindow`, `FailureClassRetry`, `BudgetScopedRetry`, `DuplicatePendingCoalescing` |
| Timeout/deadline | `Disabled`, `FixedTimeout`, `TransactionInheritedDeadline`, `RuntimeInheritedDeadline`, `PerAttemptTimeout`, `TotalRequestLifetimeTimeout`, `ProgressHeartbeatExtension`, `TerminalTimeout`, `RevalidationEligibleTimeout` |
| Cancellation | `RuntimeHardCancellation`, `HostAdvisoryCancellation`, `GracePeriodCancellation`, `DependentCancellationPropagation` |
| Supersession | `NewestGenerationWins`, `OverlappingGenerations`, `IntentEquivalenceCoalescing`, `LeaveOldHostWorkRunning`, `CancelOldHostWorkOnSupersession` |
| Revalidation/freshness | `ExplicitOnly`, `StaleAfter`, `DependencyChange`, `ObserverDemand`, `TerminalState`, `FulfilledOnly`, `ForcedWithActiveHandle`, `DedupeAndCoalesce` |
| Observation | `LifecycleOnly`, `LifecycleAndOutput`, `DeniedCompletion`, `RetrySchedule`, `PerTransactionCoalesced` |
| Output continuity | `PreserveWhilePending`, `HideWhilePending`, `PreserveAfterRejection`, `HideAfterRejection`, `PreserveAfterTimeout`, `HideAfterTimeout`, `PreserveAfterCancellation`, `HideAfterCancellation`, `SupersessionVisibility` |
| Retention | `RetainAllTransitions`, `TerminalSummariesOnly`, `DeniedCompletionsByBudget`, `RetryLineageByBudget`, `CompactSuperseded`, `CompactCancelled`, `CompactTimedOut` |
| Diagnostics | `RetainedOnly`, `BudgetedExpansion`, `DenyColdExpansion`, `ForensicExpansionBudget` |
| Replay compatibility | `IdenticalOnly`, `CompatibleParameterExpansion`, `CompatibleRetentionNarrowing`, `CompatibleDiagnosticsRichnessChange`, `DenyOnUnknownOrMissing` |

Each built-in must have:

- a stable semantic name
- a stable descriptor version
- a parameter digest
- a selection basis digest
- a cost-contract id
- a compatibility posture
- at least one focused unit test
- at least one certification or hostile row when it changes runtime behavior

### 11.9 Practical API Shape Requirements

Public APIs must make orchestration and cost visible.

Required API posture:

- registry construction uses a builder that produces `Result<Frozen...,
  ResourcePolicyRegistryDenial>` rather than panicking on duplicate or malformed
  entries
- runtime construction receives frozen policy registries or built-in policy
  presets; it must not accept mutable registries after build
- resource declaration builders accept named policy declarations and return
  lowering/admission results, not direct descriptor proof objects
- every API that can allocate temporal wakes, touch retry budgets, emit host
  advisories, perform retention compaction, or request diagnostics expansion
  must return a report with a boundary performance envelope
- ordinary summary and inspection APIs must state retained-only behavior in
  their return type or report name when they cannot perform cold work
- cold diagnostics and replay expansion APIs must require explicit budget input
  and return typed denial on budget failure

Forbidden API shapes:

- `set_policy(...)` on a live runtime after registry freeze
- `policy_named("...")` that stores only strings past declaration lowering
- `retry_after(...)` or `timeout_after(...)` surfaces that schedule temporal
  wakes without returning policy decision evidence
- getters that perform replay compatibility checks or diagnostics expansion as
  hidden lazy work
- APIs that return `Option` for policy incompatibility, budget denial, missing
  descriptors, or unavailable retained history

## 12. Milestone Phases

### Phase 1: Policy Registry And Descriptor Freeze

Deliver:

- complete frozen resource policy registry architecture
- the production module split named in `11.0`
- deterministic registry digest independent of declaration order
- policy descriptor vocabulary with id, semantic name, version, parameter
  digest, descriptor digest, selection basis, compatibility posture, and
  cost-contract id
- built-in policy registration through the same path as caller-registered
  policies
- typed denials for duplicate id, duplicate name, unknown policy, missing
  descriptor, incompatible descriptor, and malformed descriptor
- private descriptor constructors and facade-only registry access

Must prove:

- all policy declarations lower before execution work is constructed
- duplicate registrations deny before runtime construction or registry freeze
- unknown policy references deny before request admission
- registry digest is stable under equivalent registration order changes
- descriptor digests are parameter-sensitive
- external callers cannot synthesize descriptor proof objects

Phase 1 implementation rails:

- start by moving existing resource policy code into the module topology before
  adding new family behavior
- keep existing Milestone B policy declarations working through compatibility
  aliases only if the aliases lower into real descriptors
- add trybuild fixtures in the same patch that introduces each proof type
- do not add policy family variants until duplicate/unknown/missing/
  incompatible descriptor denials are real

### Phase 2: Retry And Backoff Policy Families

Deliver:

- disabled retry
- fixed delay retry
- exponential backoff
- capped exponential backoff
- deterministic seeded jitter
- max attempts
- max elapsed retry window
- failure-class-based retry
- retry by timeout, host failure, semantic rejection, or explicit manual intent
- retry budget scopes by request, resource node, resource family, runtime, and
  caller-declared scope
- duplicate pending retry coalescing
- retry-storm denial
- retry lineage decision artifacts

Must prove:

- every admitted retry preserves generation and attempt lineage
- retry remains distinct from revalidation
- retry eligibility denies before temporal wake allocation when the policy
  already proves the retry cannot admit
- deterministic jitter replays identically across branch restore and checkpoint
  replay
- retry budget exhaustion denies with typed, diagnostics-visible evidence
- duplicate pending retry coalescing does not suppress a semantically required
  retry
- retry cost reports decision width, temporal wake footprint, and budget scope
  touches

Phase 2 implementation rails:

- implement disabled retry and budget denial before implementing successful
  retry scheduling
- implement deterministic jitter from descriptor-bound seed material, not from
  process RNG, wall time, or request insertion order
- implement coalescing as a decision artifact that names losers and winner; do
  not silently drop duplicate retry candidates
- exact proof test must assert that a denied retry allocates zero temporal
  wakes

### Phase 3: Timeout And Deadline Policy Families

Deliver:

- disabled timeout
- fixed timeout
- transaction-inherited deadline
- runtime-inherited deadline
- per-attempt timeout
- total request-lifetime timeout
- progress-heartbeat extension
- timeout-as-terminal classification
- timeout-as-revalidation-eligible classification
- timeout-triggered retry eligibility
- deadline decision artifacts

Must prove:

- timeout admission always consumes Milestone A temporal wake truth
- timeout policy cannot invent a second clock model
- changing timeout scope changes descriptor digest
- progress-heartbeat extension has proof-bearing admission and cannot extend an
  already terminal request
- terminal timeout and revalidation-eligible timeout remain distinct lifecycle
  classifications
- timeout-triggered retry consumes retry policy proof rather than ad hoc
  timeout code
- timeout cost reports temporal frontier width and affected request count

Phase 3 implementation rails:

- timeout decisions must consume Milestone A wake proofs rather than direct
  clock comparisons in resource code
- inherited deadlines must carry the authority that supplied them:
  transaction, runtime, or descriptor
- progress-heartbeat extension must be rejected for terminal, cancelled,
  superseded, and timed-out requests before constructing extension evidence
- exact proof test must assert timeout racing success produces one winner and
  one typed denial, not two lifecycle transitions

### Phase 4: Cancellation And Supersession Policy Families

Deliver:

- runtime-hard cancellation
- best-effort host cancellation signalling
- cancellation grace period
- cancellation after supersession
- dependent-resource or child-resource cancellation propagation where declared
- newest-generation-wins supersession
- overlapping-generation policy
- intent-equivalence coalescing
- leave-old-host-work-running with runtime-hard late-completion denial
- cancel-old-host-work-on-supersession with host advisory evidence
- cancellation and supersession decision artifacts

Must prove:

- host cancellation failure cannot allow a late completion to commit
- runtime-hard cancellation proof and host advisory signal proof stay separate
- superseded completion denial remains stable across replay
- overlapping-generation policy cannot erase request identity or branch epoch
- intent-equivalence coalescing preserves canonical lineage and digest evidence
- dependent cancellation only touches declared dependent footprint
- cancellation and supersession cost report affected request footprint and host
  advisory width separately

Phase 4 implementation rails:

- implement runtime-hard denial first; host advisory signalling is an effect
  derived from cancellation policy, not the cancellation authority
- model host advisory evidence as optional and non-authoritative even when the
  host reports success
- overlapping generations must require an explicit admission token and must
  still prevent older generations from masquerading as current winner
- exact proof test must assert failed host cancellation plus late completion
  remains denied

### Phase 5: Revalidation And Freshness Policy Families

Deliver:

- explicit revalidation only
- stale-after revalidation
- dependency-change revalidation
- observer-demand revalidation
- terminal-state revalidation
- fulfilled-only revalidation
- forced revalidation with active-handle proof
- deduped and coalesced revalidation
- revalidation freshness decision artifacts

Must prove:

- revalidation remains distinct from retry
- stale-after revalidation consumes runtime temporal truth
- dependency-change revalidation consumes runtime invalidation truth
- observer-demand revalidation consumes commit-bounded observation truth
- active request overwrite requires expected-active proof or a policy-produced
  force token
- forced revalidation cannot be WORTHd by external callers
- coalescing does not suppress a semantically required refresh
- revalidation cost reports active-handle proof checks and coalescing width

Phase 5 implementation rails:

- implement active-handle proof before forced revalidation
- implement stale-after revalidation through temporal substrate evidence, not
  direct resource-local timestamps
- observer-demand revalidation must consume committed observation evidence; it
  may not read mutable transaction state
- exact proof test must assert retry and revalidation produce different
  decision artifacts for equivalent-looking refresh work

### Phase 6: Observation And Output Continuity Policy Families

Deliver:

- lifecycle-only observation
- output-continuity observation
- denied-completion observation
- retry-schedule observation
- per-transaction coalesced observation
- preserve previous output while pending
- hide previous output while pending
- preserve or hide output after rejection
- preserve or hide output after timeout
- preserve or hide output after cancellation
- supersession visibility policies
- observation and output-continuity decision artifacts

Must prove:

- observation policy cannot mutate lifecycle truth
- output visibility digest stays separate from lifecycle digest
- observer packets remain commit-bounded and rollback-safe
- visibility choices replay identically
- preserving previous output while pending does not imply fulfilled lifecycle
- hiding previous output while pending does not erase retained output history
- denied-completion observation may report denial evidence without applying the
  denied completion
- observation cost reports candidate width, coalesced width, delivery width,
  and output-continuity classification width

Phase 6 implementation rails:

- lifecycle digest and output-continuity digest must be generated by different
  canonical functions
- pending visibility changes must not update committed output identity
- denied-completion observation must consume denied completion evidence and
  remain incapable of calling completion apply
- exact proof test must assert preserved pending output and fulfilled lifecycle
  are distinguishable by public accessors and certification digests

### Phase 7: Retention, Diagnostics, And Replay Compatibility Policies

Deliver:

- retain all lifecycle transitions
- retain terminal summaries only
- retain denied completions by budget
- retain retry lineage by budget
- compact superseded, cancelled, and timed-out records
- retained-history unavailable classifications
- diagnostics expansion budgets
- policy version compatibility rules
- policy incompatibility denials
- missing-policy replay denials
- retained/reconstructed/omitted/denied/unavailable availability outcomes
- retention, diagnostics, and replay compatibility decision artifacts

Must prove:

- retained summary reads perform zero cold reconstruction
- diagnostics expansion is explicitly budgeted cold work
- retention policy loss is classified, not silently ignored
- policy-compatible restore emits compatibility proof
- policy-incompatible restore emits typed denial
- missing policy descriptor restore emits typed missing-policy denial
- replay does not silently reinterpret old async truth under current policy
  code
- retention and diagnostics cost report retained-summary reads, pruned records,
  compacted records, cold reconstruction, and diagnostics budget consumption
  separately

Phase 7 implementation rails:

- implement compatibility denials before compatibility approvals
- ordinary retained summary reads must be tested before diagnostics expansion
  so the zero-cold baseline cannot move
- retention compaction must create availability artifacts for what was dropped;
  it may not rely on absence as the signal
- exact proof test must assert incompatible descriptor restore denies without
  calling current policy decision code on historical records

### Phase 8: Policy Certification Surface And Closeout Gate

Deliver:

- `async_resource_policy_family_certification`
- `async_retry_budget_and_backoff_certification`
- `async_timeout_deadline_certification`
- `async_cancellation_supersession_policy_certification`
- `async_revalidation_freshness_certification`
- `async_observation_output_continuity_certification`
- `async_retention_replay_policy_certification`
- policy scenario matrix
- policy performance closeout
- final Milestone C certification run
- final adversarial closeout suites for the crucial mixed temporal and async
  runtime grammars
- compile-fail fixtures for private descriptor constructors, policy decision
  constructors, force tokens, budget proofs, compatibility proofs, and facade
  boundaries

Must prove:

- every policy family has at least one passing certification record
- every policy family has at least one hostile scenario row
- every policy decision is traceable to descriptor id/name/version/digest
- policy variation does not alter hard lifecycle laws
- replay compatibility and incompatibility are both certified
- performance envelopes report policy-specific cost surfaces
- external callers cannot WORTH policy proof objects

Phase 8 implementation rails:

- closeout certification must consume production reports from phases 1-7; it
  may not synthesize passing rows from raw digests
- every row must expose accessors for the counters it uses to pass
- performance closeout must bind scenario row digest plus boundary envelope
  digest, not only family name
- trybuild must prove final run, scenario matrix, hostile rows, and performance
  closeout are proof objects, not deserializable truth

### Phase 9: Adversarial Confidence Prioritization And Final Runtime Torture

Deliver:

- one unified async nightmare grammar certification lane
- one strong async branch-restore-and-replay equivalence lane
- one strong temporal parity-and-boundedness lane
- explicit spec language distinguishing crucial closeout coverage from
  non-blocking exhaustive certification expansion

This phase exists to keep Milestone C honest against the full
`test-requirements.md` ambition without pretending every enumerated sub-bullet
needs its own heavyweight standalone suite before the milestone is production
grade.

The governing prioritization for final closeout is:

- **crucial and blocking**
  - the unified async nightmare grammar from `19A` / `19B`
  - async branch restore and replay equivalence from `18`
  - async resource lifecycle parity from `15`
  - async rollback and observation equivalence from `17`
  - temporal wake boundedness from `13`
- **important, but acceptable to satisfy through stronger combined suites**
  - temporal eligibility replay parity from `11`
  - temporal branch restore equivalence from `12`
  - previous-value and time-gated node equivalence from `14`
  - out-of-order completion supersession from `16`
  - broad async policy family workload parity from `20`
- **non-blocking for Milestone C closeout unless a concrete gap appears**
  - exhaustive one-suite-per-sub-bullet coverage for every clause in sections
    `11` through `20A`
  - full policy-permutation matrices where sampled hostile families and the
    unified nightmare grammar already prove the same invariant
  - new certification artifact layers that do not buy a missing behavioral
    guarantee

Must prove:

- the runtime survives one mixed hostile async grammar that instantiates
  completion-ordering, completion-integrity, request-identity, liveness, and
  async-pressure failures together
- equivalent async branch histories converge to identical canonical lifecycle,
  denial, inflight, and explanation digests after restore and replay
- temporal parity and boundedness remain honest under mixed timing policies,
  branch restore, long idle gaps, sparse frontiers, and threshold oscillation
- the milestone does not claim exhaustive certification where a smaller number
  of stronger workload suites already provide the production-grade guarantee

Phase 9 implementation rails:

- prefer fewer mixed hostile suites with stronger invariants over a larger
  number of decorative scenario-specific tests
- any suite used to satisfy multiple requirements must assert the exact digests,
  denial classes, and boundedness counters those requirements care about
- if a requirement from sections `11` through `20A` is treated as indirectly
  covered, the owning suite must name that coverage explicitly in the test body
  or certification helper
- do not add new proof artifacts in this phase unless an existing artifact
  cannot honestly carry the required runtime truth
- the milestone is not allowed to call itself production-grade while the
  `19A` nightmare grammar is still absent

### 12.1 Phase Ordering Rationale

The ordering is intentionally strict.

- `Phase 1` freezes registry and descriptor identity before any family-specific
  policy work, so later phases cannot smuggle behavior through loose callback
  paths.
- `Phase 2` lands retry/backoff first because retry interacts with temporal
  wake allocation, budget state, attempt lineage, and later timeout-triggered
  decisions.
- `Phase 3` lands timeout/deadline after retry because timeout may classify
  terminal lifecycle or produce retry eligibility, and both paths need runtime
  time plus retry proof.
- `Phase 4` lands cancellation/supersession after timeout because timeout,
  cancellation, and supersession all retire completion authority but must remain
  distinct denial and visibility classes.
- `Phase 5` lands revalidation/freshness after cancellation/supersession
  because revalidation can overwrite active intent only with proof that doing so
  is legal under the active policy.
- `Phase 6` lands observation/output continuity after lifecycle-affecting
  policies so visibility can be derived from committed lifecycle truth rather
  than defining it.
- `Phase 7` lands retention/diagnostics/replay compatibility after every
  policy family has real decision artifacts worth retaining, explaining, and
  comparing during replay.
- `Phase 8` comes last because certification must assemble proof from every
  prior phase rather than defining the policy model itself.
- `Phase 9` follows certification assembly because the final production-grade
  bar is not "all artifact types exist", it is "the hardest mixed temporal and
  async failure grammars are actually survived by real workload suites."

If a future edit tries to merge non-adjacent phases, it must prove that no real
structural dependency, proof boundary, or performance boundary is being hidden
by the compression.

## 13. Must Ship

Milestone C is not done because policy enums exist.

It is done only when `worth-signal` ships:

- complete frozen registries for all Milestone C async/resource policy families
- deterministic policy descriptor identity with id, semantic name, version,
  descriptor digest, selection basis, compatibility posture, and cost-contract
  identity
- duplicate, unknown, missing, incompatible, malformed, budget-exhausted, and
  semantically illegal policy denials
- built-in retry and backoff policy families
- built-in timeout and deadline policy families
- built-in cancellation and supersession policy families
- built-in revalidation and freshness policy families
- built-in observation and output-continuity policy families
- built-in retention, diagnostics, and replay compatibility policy families
- proof-bearing policy decision artifacts for every family
- sealed force, eligibility, budget, advisory, visibility, retention,
  diagnostics, compatibility, and incompatibility tokens
- branch-, restore-, and replay-honest policy budget state
- policy-specific boundary performance envelopes
- retained policy history and diagnostics-visible policy provenance
- public core APIs for declaring policy, using built-in policy descriptors,
  registering custom descriptor families where allowed, and inspecting policy
  decisions without exposing internal constructors
- compile-fail fixtures for policy proof constructors and facade boundaries
- final Milestone C certification run
- explicit mapping from the crucial remaining requirements in
  `test-requirements.md` to either a direct owning suite or a stronger combined
  suite that satisfies them

### 13.1 Required Named Test Families

- `async_resource_policy_family_certification`
- `async_retry_budget_and_backoff_certification`
- `async_timeout_deadline_certification`
- `async_cancellation_supersession_policy_certification`
- `async_revalidation_freshness_certification`
- `async_observation_output_continuity_certification`
- `async_retention_replay_policy_certification`

These families are the owning implementation lanes for the corresponding
policy requirements declared in [`test-requirements.md`](./test-requirements.md),
especially:

- `20. The async resource policy family certification test`
- `20A. The async policy registry boundary test`
- `19A. The worst async nightmare grammar`
- `19B. The regulated-system adversarial rule`

### 13.2 Hostile Conditions Required In Certification

- duplicate policy id registration
- duplicate policy semantic-name registration
- unknown policy reference during declaration lowering
- missing policy descriptor during restore
- incompatible policy descriptor during restore
- deterministic jitter replay after branch restore
- retry budget exhaustion under retry storm pressure
- duplicate pending retry coalescing
- timeout racing success
- timeout racing cancellation
- timeout-triggered retry with exhausted retry budget
- progress-heartbeat extension of a non-terminal request
- invalid heartbeat extension after terminal state
- host cancellation signal failure followed by late completion
- cancellation racing completion
- supersession with old host work left running
- overlapping-generation completion under an admitted overlap policy
- intent-equivalence coalescing with distinct request identities
- stale-after revalidation after temporal restore
- observer-demand revalidation racing dependency-change revalidation
- forced revalidation without active-handle proof
- pending output preserved under one policy and hidden under another
- denied-completion observation without denied-completion apply
- retained history pruned before diagnostics expansion
- diagnostics expansion budget denial
- compatible policy restore
- incompatible policy restore

### 13.3 Required Scenario Matrix Rows

`ResourceMilestoneCPolicyScenarioMatrix` must contain at least these rows.

| Scenario | Evidence kind | Purpose |
| --- | --- | --- |
| `RegistryOrderCanonicalization` | registry freeze report | proves equivalent policy sets produce one registry digest regardless of registration order |
| `DuplicatePolicyIdentityRejected` | policy denial | proves duplicate id and duplicate semantic-name deny before runtime construction |
| `UnknownPolicyReferenceRejected` | policy denial | proves unknown declarations deny before request admission or temporal wake allocation |
| `RetryBudgetExhaustionRejected` | retry decision | proves budget exhaustion denies with zero wake allocation |
| `DeterministicJitterReplayParity` | retry decision plus replay compatibility | proves jitter is descriptor-seeded and branch/replay stable |
| `RetryStormCoalescingBounded` | retry decision | proves duplicate retry candidates coalesce with explicit winner/loser evidence |
| `TimeoutSuccessRaceClassified` | timeout decision plus completion evidence | proves timeout racing success creates one committed winner and one typed denial |
| `HeartbeatExtensionTerminalDenied` | timeout decision denial | proves heartbeat extension cannot revive terminal requests |
| `HostCancellationFailureLateCompletionDenied` | cancellation decision plus completion denial | proves host advisory failure does not weaken runtime-hard denial |
| `OverlappingGenerationIdentityPreserved` | supersession decision | proves overlap policy does not collapse request/generation/attempt/branch identity |
| `IntentEquivalenceCoalescingPreservesLineage` | supersession decision | proves equivalent intent coalescing records winner/loser lineage |
| `RetryAndRevalidationRemainDistinct` | retry and revalidation decisions | proves same-looking refresh work cannot share one proof type |
| `ForcedRevalidationRequiresActiveHandle` | revalidation denial | proves external callers cannot force refresh without active-handle proof |
| `ObserverDemandUsesCommittedObservation` | revalidation decision | proves observer-demand revalidation reads committed observation evidence only |
| `PendingVisibilityDoesNotMutateLifecycle` | output-continuity decision | proves preserve/hide while pending changes visibility digest but not lifecycle digest |
| `DeniedCompletionObservationCannotApply` | observation decision | proves denied-completion observation cannot mutate committed resource state |
| `RetentionCompactionReportsUnavailableHistory` | retention decision | proves pruned/compacted history creates typed availability artifacts |
| `DiagnosticsExpansionBudgetDeniedZeroCold` | diagnostics denial | proves denied diagnostics expansion performs zero cold reconstruction |
| `CompatibleDescriptorRestoreAdmitted` | replay compatibility proof | proves explicitly compatible descriptor drift emits compatibility evidence |
| `IncompatibleDescriptorRestoreDenied` | replay incompatibility denial | proves descriptor drift denies before current policy code executes |
| `MissingDescriptorRestoreDenied` | replay incompatibility denial | proves missing historical descriptor cannot be silently ignored |

The final certification run must require every row exactly once. Duplicate,
missing, or wrong-evidence-kind rows must reject before final run construction.

### 13.4 Required Performance Closeout Rows

`ResourceMilestoneCPolicyPerformanceCloseout` must contain at least these
claims.

| Claim | Evidence bound | Contract checked |
| --- | --- | --- |
| `RegistryFreezeOrderBounded` | `RegistryOrderCanonicalization` | registry freeze cost is descriptor count plus duplicate-index width, not resource graph width |
| `RetryBudgetDenialZeroWake` | `RetryBudgetExhaustionRejected` | denied retry performs zero temporal wake allocation and reports budget scope touches |
| `RetryStormCoalescingBounded` | `RetryStormCoalescingBounded` | coalescing reports candidate width, winner count, loser count, and zero duplicate execution |
| `DeterministicJitterReplayBounded` | `DeterministicJitterReplayParity` | jitter cost is admitted retry decision width and descriptor seed proof |
| `TimeoutRaceFrontierBounded` | `TimeoutSuccessRaceClassified` | timeout race cost is temporal frontier width plus affected request count |
| `HostCancellationAdvisorySeparated` | `HostCancellationFailureLateCompletionDenied` | host advisory width and runtime-hard denial footprint are separate counters |
| `SupersessionOverlapIdentityBounded` | `OverlappingGenerationIdentityPreserved` | overlap policy reports affected request footprint without identity collapse |
| `RevalidationActiveHandleBounded` | `ForcedRevalidationRequiresActiveHandle` | active-handle proof checks are counted and denied force performs no admission work |
| `ObservationVisibilityRollbackBounded` | `PendingVisibilityDoesNotMutateLifecycle` | visibility classification width is separate from lifecycle transition count |
| `DeniedCompletionObservationNonApplying` | `DeniedCompletionObservationCannotApply` | denied observation reports delivery width but zero committed completion transitions |
| `RetentionCompactionAvailabilityBounded` | `RetentionCompactionReportsUnavailableHistory` | retention reports retained/pruned/compacted/unavailable counts separately |
| `DiagnosticsBudgetDenialZeroCold` | `DiagnosticsExpansionBudgetDeniedZeroCold` | denied diagnostics expansion reports zero cold reconstruction |
| `ReplayCompatibilityDescriptorBounded` | compatible/incompatible/missing descriptor rows | replay compatibility cost is descriptor comparison width plus retained policy history width |

The final performance closeout must bind:

- scenario row digest
- policy decision digest
- boundary performance envelope digest
- cost contract id
- cost posture
- allocation lane counts
- density strategy where applicable

No performance claim may pass from family name, descriptor digest, or scenario
digest alone.

## 14. Must Preserve

- deterministic execution remains a product contract
- commit-bounded observation remains unchanged
- rollback remains hard rewind rather than best-effort cleanup
- authority stays outside `worth-signal`
- async/resource lifecycle remains runtime-owned derived truth
- request identity, generation, attempt, branch epoch, and ordinal categories
  remain distinct
- stale, superseded, cancelled, timed-out, malformed, contradictory, duplicate,
  unknown, partial, retained-history-unavailable, and impossible completions
  remain distinct denial classes where applicable
- completion admission remains transactional
- denied completions do not mutate committed resource state
- temporal meaning remains owned by the Milestone A temporal substrate
- Milestone B retained summary reads remain zero-cold unless an explicit
  diagnostics or replay API admits cold work
- diagnostics richness may vary by policy, but lifecycle truth may not
- policy compatibility may deny replay, but may not silently reinterpret old
  runtime history

## 15. Performance Contracts

The milestone must expose named counters for at least:

- policy registry freeze count
- policy descriptor count
- policy descriptor comparison count
- policy descriptor incompatibility count
- duplicate policy id denial count
- duplicate policy name denial count
- unknown policy denial count
- missing policy descriptor denial count
- malformed policy descriptor denial count
- policy budget exhaustion denial count
- retry policy decision count
- retry admitted count
- retry denied count
- retry coalesced count
- retry storm denial count
- retry budget scope touch count
- deterministic jitter decision count
- retry temporal wake allocation count
- timeout policy decision count
- timeout admitted count
- timeout denied count
- timeout temporal frontier width
- deadline inherited count
- progress heartbeat extension count
- cancellation policy decision count
- runtime-hard cancellation count
- host cancellation advisory count
- cancellation grace period count
- dependent cancellation propagation count
- supersession policy decision count
- overlapping generation admission count
- intent-equivalence coalescing count
- old host work retained count
- old host work advisory-cancelled count
- revalidation policy decision count
- observer-demand revalidation count
- dependency-change revalidation count
- forced revalidation count
- active-handle proof check count
- revalidation coalesced count
- observation policy decision count
- observation candidate width
- observation coalesced width
- observation delivered width
- denied-completion observation count
- retry-schedule observation count
- output-continuity decision count
- previous-output-preserved count
- previous-output-hidden count
- terminal-output-preserved count
- terminal-output-hidden count
- retention policy decision count
- retained lifecycle transition count
- retained terminal summary count
- retained denied completion count
- retained retry lineage count
- compacted terminal record count
- retained history unavailable count
- diagnostics policy decision count
- diagnostics expansion request count
- diagnostics expansion admitted count
- diagnostics expansion denied count
- diagnostics cold reconstruction count
- replay compatibility decision count
- replay compatible count
- replay incompatible count
- replay missing policy count
- policy boundary envelope count
- policy operational allocation count
- policy retained-history allocation count
- policy diagnostics allocation count
- policy facade/report allocation count

The milestone must also declare named complexity contracts for:

- policy registry freeze
- policy descriptor lookup
- policy descriptor compatibility comparison
- resource declaration policy lowering
- retry eligibility and budget admission
- deterministic jitter calculation
- retry/backoff temporal wake allocation
- retry-storm coalescing and denial
- timeout/deadline admission
- progress heartbeat extension
- runtime-hard cancellation
- host cancellation advisory emission
- dependent cancellation propagation
- supersession admission
- overlapping generation admission
- intent-equivalence coalescing
- revalidation admission
- active-handle proof validation
- observation policy delivery
- output-continuity classification
- retention policy application
- diagnostics expansion admission
- replay compatibility classification
- policy certification bundle construction

Each contract must name its real cost bases explicitly. At minimum:

- registry freeze cost must be stated in terms of registered policy descriptor
  count and duplicate detection work, not total resource graph size
- descriptor lookup cost must be stated in terms of descriptor key lookup and
  compatibility posture checks
- retry cost must be stated in terms of retry decision width, budget scope
  touches, and temporal wake footprint
- deterministic jitter cost must be stated in terms of admitted retry decisions,
  not elapsed time or historical retry count
- retry storm coalescing cost must be stated in terms of pending retry
  candidates and coalesced winners
- timeout/deadline cost must be stated in terms of temporal frontier width and
  affected request count
- cancellation cost must be stated in terms of affected request footprint
- host advisory cost must be stated separately from runtime-hard denial cost
- supersession cost must be stated in terms of superseded active request
  footprint and admitted replacement width
- revalidation cost must be stated in terms of active-handle proof checks and
  coalescing width
- observation cost must be stated in terms of candidate, matching, coalesced,
  and delivered observation width
- output-continuity cost must be stated in terms of classification width and
  retained output references
- retention cost must be stated in terms of retained, pruned, compacted, and
  unavailable lifecycle records
- diagnostics expansion cost must be explicitly separated from operational
  policy decision cost
- replay compatibility cost must be stated in terms of descriptor comparison
  width and retained policy history width

### 15.1 Named Policy Performance Failure Modes

Milestone C should name the failure modes it intends to prohibit so later work
cannot reintroduce them under nicer names.

At minimum:

- `PolicyRegistryOrderDrift`
  Canonical registry identity changes because equivalent policies were
  registered in a different order.
- `CallbackPolicyTruthLeak`
  Replay-critical policy meaning lives only in callback code rather than a
  deterministic descriptor.
- `PolicyDescriptorBlindness`
  A policy parameter change alters behavior without changing descriptor digest.
- `RetryStormAmplification`
  Retry scheduling creates work proportional to historical attempts or elapsed
  time rather than admitted retry decisions and budget scopes.
- `TimeoutClockLeak`
  Timeout or deadline policy consults a non-runtime clock or host wall-clock
  path as eligibility authority.
- `HostCancellationAuthorityLeak`
  Best-effort host cancellation signal is treated as runtime-hard completion
  denial.
- `SupersessionIdentityCollapse`
  Supersession or overlapping-generation policy erases request, generation,
  attempt, or branch epoch identity.
- `RevalidationRetryCollapse`
  Retry and revalidation share one path that loses the distinction between
  continuing failed intent and admitting fresh refresh intent.
- `OutputLifecycleCollapse`
  Output visibility policy mutates lifecycle truth or lets preserved output
  masquerade as fulfilled state.
- `ObservationPolicyRollbackLeak`
  Observation policy delivers packets from failed or rolled-back completion
  transactions.
- `RetentionSilentDrop`
  Policy compaction or pruning removes lifecycle or denial history without a
  typed unavailable or omitted outcome.
- `HiddenDiagnosticsColdWork`
  Ordinary summary, observation, or policy inspection reads perform diagnostics
  reconstruction without an explicit budget.
- `ReplayPolicyReinterpretation`
  Restore or replay applies current policy code to old history without proving
  descriptor compatibility.
- `PolicyAllocationChurn`
  Policy decisions allocate per request or per retained record without
  lifecycle-bounded allocation posture or counted debt.

## 16. Acceptance Evidence

Milestone C is complete only when `worth-signal` can certify all of the
following with canonical machine-checkable artifacts:

- the `Async Resource Policy Family Certification Test`
- the `Async Retry Budget And Backoff Certification Test`
- the `Async Timeout Deadline Certification Test`
- the `Async Cancellation Supersession Policy Certification Test`
- the `Async Revalidation Freshness Certification Test`
- the `Async Observation Output Continuity Certification Test`
- the `Async Retention Replay Policy Certification Test`
- the `Async Policy Registry Boundary Test`

The final closeout gate must be a sealed `ResourceMilestoneCCertificationRun`
or direct canonical equivalent that requires:

- a complete policy certification bundle
- every required scenario matrix row from `13.3`
- every required performance closeout row from `13.4`
- replay compatibility and incompatibility evidence
- compile-fail boundary evidence for every proof constructor class
- exact row uniqueness by scenario, family, hostile condition, and performance
  claim

The certification bundle must include canonical digests for:

- policy registries
- policy descriptors
- policy selection bases
- policy compatibility postures
- registry freeze artifacts
- retry lineage
- retry budget decisions
- deterministic jitter decisions
- retry storm coalescing or denial
- timeout and deadline decisions
- inherited deadline proofs
- progress heartbeat extension proofs
- cancellation decisions
- host cancellation advisory evidence
- supersession decisions
- overlapping generation admission
- intent-equivalence coalescing
- revalidation and freshness decisions
- active-handle and force-token proofs
- observation decisions
- output-continuity decisions
- lifecycle digests separate from output-visibility digests
- retention decisions
- diagnostics budget decisions
- retained, pruned, compacted, unavailable, reconstructed, denied, and
  incompatible availability outcomes
- replay compatibility and incompatibility artifacts
- boundary performance envelopes
- cost contract ids and postures
- allocation posture counters
- diagnostics/explanation artifacts

Required compile-fail fixture classes:

- external code cannot construct frozen policy descriptors
- external code cannot construct lowered policy bundles
- external code cannot construct admitted or denied policy decisions
- external code cannot mutate policy decision private fields
- external code cannot spend retry proof as revalidation proof
- external code cannot spend host advisory proof as runtime cancellation proof
- external code cannot construct active-handle revalidation proof
- external code cannot construct diagnostics expansion budget admission
- external code cannot construct replay compatibility proof
- external code cannot deserialize final certification rows as proof objects
- resource policy internals are not publicly reachable except through the
  facade-approved exports

Required focused implementation tests before closeout:

- registry digest remains stable when equivalent descriptors are registered in
  different orders
- policy descriptor digest changes for every parameter that affects behavior
- built-in aliases lower through the same path as caller-registered
  descriptors
- custom descriptor aliases cannot carry executable callbacks
- denied retry allocates zero temporal wakes
- timeout decisions consume temporal wake evidence
- failed host advisory cancellation cannot commit late completion
- output visibility changes leave lifecycle digest unchanged
- retained summary read stays zero-cold after retention policy churn
- incompatible replay denies before executing current policy decision code

## 17. Architectural Notes

- Milestone C should use deterministic descriptors for policy meaning even
  where an implementation internally calls Rust code to calculate a decision.
  The descriptor, not the closure identity, is the replay authority.
- Built-in policies are not special execution branches. They must register and
  lower through the same descriptor path as later caller-registered policies.
- Custom policy families may exist only where the runtime can represent their
  replay-relevant meaning by descriptor identity and versioned compatibility.
  A custom callback that cannot be described cannot own replay-critical truth.
- Retry and revalidation should feel similar to consumers but stay different in
  the runtime. Retry continues an admitted intent after eligible failure;
  revalidation admits fresh intent to refresh resource truth.
- Cancellation and supersession both retire completion authority, but they are
  not the same event. Cancellation may be user/system intent to stop work;
  supersession admits newer intent and classifies older authority accordingly.
- Timeout and rejection are different. Timeout is temporal policy reaching a
  deadline; rejection is resource completion outcome or host/semantic failure.
- Output continuity is a presentation and observation contract over committed
  resource lifecycle; it must not redefine the lifecycle itself.
- Observation policy belongs to the runtime because it affects committed
  derived-state delivery and replay explanation. UI subscription ergonomics
  remain later product-layer work.
- Retention policy may make rich diagnostics unavailable. It may not fabricate
  diagnostics from missing history.
- Replay compatibility should be conservative. A typed denial is better than a
  plausible replay under uncertain policy meaning.

## 18. Explicit Deferrals

Milestone C intentionally does not include:

- wasm resource bindings
- React, Angular, or browser adapter APIs
- route loaders
- form submit/action products
- query replacement product surfaces
- transport, fetch, websocket, RPC, or background worker implementation
- persistent storage products for resource caches
- domain-specific cache eviction beyond generic retention policy
- optimistic UI transition products
- app-level loading-state design
- full merge semantics for resource histories beyond the branch/replay
  compatibility needed here

Those remain later roadmap or product-layer work. They can only be considered
honest once they reduce to the runtime-owned policy substrate defined here.

## 19. Sequencing Notes

This milestone belongs immediately after Milestone B because Milestone B closes
the substrate that policy variation must consume.

Milestone B established:

- resource declarations and lowered descriptors
- request identity
- generation, attempt, branch epoch, and ordinal categories
- in-flight ownership
- cancellation, timeout, retry, revalidation, and supersession substrate
- completion admission and denial
- transaction apply and rollback
- branch restore and replay reconstruction
- retained lifecycle history
- diagnostics expansion budgets
- boundary performance envelopes

Milestone C consumes that foundation to define:

- production retry/backoff policy families
- production timeout/deadline policy families
- production cancellation/supersession policy families
- production revalidation/freshness policy families
- production observation/output-continuity policy families
- production retention/diagnostics/replay compatibility policy families
- policy-specific certification and performance closeout

It belongs before wasm, route-resource, form, query, and browser adapter product
layers claim resource behavior because those layers must consume runtime policy
truth instead of defining parallel state machines.

If product adapters ship first, their policy defaults will become de facto
runtime semantics and Milestone C will become a migration problem instead of a
substrate design problem.

## 20. Required Self-Check

- Does this milestone solve a real structural problem or just package work
  cosmetically?
  Yes. It closes the policy substrate required for async/resource lifecycle to
  become product-usable without adapter-local state machines.
- Is the adversarial constraint precise and load-bearing?
  Yes. Descriptor drift, retry storms, timeout races, host cancellation failure,
  output/lifecycle collapse, retention loss, diagnostics cold work, and replay
  incompatibility all directly shape the architecture and tests.
- Does the milestone preserve crate authority boundaries?
  Yes. `worth-signal` owns derived lifecycle and policy truth; hosts execute
  external work; relational/store remain truth and persistence authorities.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Registry freeze, descriptor identity, decision artifacts, budget tokens,
  replay compatibility, denial artifacts, performance envelopes, and compile-
  fail boundaries are all required proof surfaces.
- Is performance encoded into architecture rather than left as observability?
  Yes. Every policy family has named counters, complexity contracts, boundary
  envelopes, allocation lanes, and prohibited failure modes.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names policy subsystem boundaries, proof tokens, decision
  artifacts, storage categories, phases, counters, hostile cases, certification
  families, and closeout artifacts.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It follows temporal and async/resource substrate closeout, and it must
  precede product-layer resource APIs that would otherwise invent policy truth.

## 21. Milestone Done When

Milestone C is done only when `worth-signal` can support async/resource policy
variation through a frozen, typed, replay-honest substrate that:

- preserves authority boundaries
- makes policy descriptors runtime-owned truth
- keeps lifecycle law stronger than policy preference
- denies unknown, duplicate, incompatible, budget-exhausted, and illegal policy
  decisions explicitly
- keeps retry, timeout, cancellation, supersession, revalidation, observation,
  output continuity, retention, diagnostics, and replay compatibility
  descriptor-backed
- keeps observation commit-bounded and rollback-safe
- exposes bounded, measurable policy work
- integrates with temporal policy, async/resource lifecycle, branch, restore,
  replay, diagnostics, and certification without inventing a second semantic
  story
- proves the crucial mixed temporal and async nightmare workloads directly,
  while allowing less critical requirement bullets to close through stronger
  combined suites instead of one bespoke suite per clause

At that point, higher-level wasm, route-resource, form, query, and app resource
surfaces can finally inherit one trustworthy policy model instead of turning
runtime async truth back into product folklore.
