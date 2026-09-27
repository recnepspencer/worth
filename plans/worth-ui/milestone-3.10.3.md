# Milestone 3.10.3: Executable World Certification Foundation

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status

Status: Complete. Phases 1 through 5 closed on 2026-07-27. This corrective gate
after Milestone 3.10.2 is closed, and Milestone 3.11 may begin from the
permanent executable-world foundation.

## Placement

Milestone 3.10.2 closed the product capability it set out to build: the
checked-in Platform Pulse can be launched manually, reaches a real native egui
frame, visibly replaces blue with green through a real watched file edit,
preserves its predecessor after malformed source, recovers, and shuts down.
That real launch also exposed and caused repair of a Windows main-thread stack
overflow before first publication.

The evidence architecture did not deserve the same closure claim. Its automated
"native" and "product shell" tests create an `egui::Context` and drive public
application-shell methods in the certification process. The watched lifecycle
test uses a real operating-system watcher, but it also drives replacement
in-process. Neither automated world enters:

```text
worth-ui-platform-pulse::main
  -> eframe::run_native
  -> operating-system native event loop
  -> PlatformPulseNativeFrame
  -> application preparation
  -> watcher worker thread
  -> native first-frame publication
```

Those tests are useful integration evidence. They are not executable
end-to-end evidence under Testing Evidence Law 16, and their aggregate
greenness could not expose a defect in the product composition root or its real
thread and stack topology.

Milestone 3.11 must not attach pixel identity to a product world that automated
certification still does not enter. This milestone therefore establishes the
permanent executable-world foundation that Milestones 3.11 through 3.23 must
extend. Any future Milestone 3.24 enters a mature world and may concentrate on
polish, scenario richness, and quality thresholds rather than inventing its
first real product entry.

## Goal

Make product-entry reachability a permanent, separately classified, required
proof:

```text
exact Cargo-built pulse executable
  + isolated copy of canonical checked-in source
  + real operating-system process and native event loop
  + real watcher worker and filesystem delivery
  + real public application lifecycle and egui host
  + externally applied product actions
  + externally observed native consequences
  + product-issued typed lifecycle observations
  + typed executable-world progression and cleanup
```

The same executable-world target, world progression, source baseline, native
platform boundary, observation protocol, failure-artifact policy, and proof
ledger become cumulative 3.x infrastructure. Later milestones add semantic
world deltas and observations; they do not add a new binary, composition root,
integration target, generic harness, or product-entry mechanism.

This milestone does not replace focused, model, compile-contract, topology, or
in-process integration evidence. It prevents any one of those narrower forms
from closing a product-visible claim on behalf of the real executable.

## Central Claim

A Worth UI product-visible claim is closed only when the narrow evidence that
proves its local contracts is joined by a required executable-world scenario
that starts at the actual product binary and independently observes the
user-visible consequence through the production composition root.

No test count, in-process egui frame, direct application-shell call, source
scan, manual run, publication log, screenshot, or receipt is sufficient alone.
The executable world must relate product-issued causal observations to an
external process, native window, action, and consequence without promoting the
runner or its artifacts into product authority.

## Adversarial Constraint

Assume every local and in-process test is green while the real product can:

- overflow or panic on the native main thread before the first frame;
- fail only inside `eframe::run_native`, window creation, callback ownership,
  event-loop scheduling, or `PlatformPulseNativeFrame`;
- launch from the wrong source root or never start the real watcher worker;
- emit a publication-looking line and then exit before a stable visible frame;
- show pixels that came from application-local drawing rather than mounted host
  translation;
- emit a plausible lifecycle observation without a corresponding window or
  native consequence;
- show a plausible native consequence without the matching product-issued
  application generation and mounted frame;
- accept a file edit only because the runner injected a snapshot, candidate,
  event, identity, or late-stage runtime artifact;
- treat watcher timeout, process death, malformed-source denial, and successful
  replacement as the same empty or unchanged result;
- preserve green pixels while losing the predecessor generation or mounted
  frame that made them truthful;
- close the window while leaking the watcher thread, host surface, or child
  process; or
- pass on one machine because a required native lane silently compiled zero
  tests.

The permanent courtroom must make each of those implementations red.

## Decisive Executable Courtroom

### Real Entry Surface

The courtroom launches the exact `worth-ui-platform-pulse` binary Cargo built
for the integration target. It must obtain that path from Cargo's executable
test contract, not search `target`, invoke nested Cargo, rebuild the binary, or
call application code in-process.

The child process must execute its ordinary `main`, `eframe::run_native`, and
native callback lifecycle on the operating system's real process main thread.
The runner may not wrap product `main` in a larger-stack thread or replace the
event loop.

### Causally Valid World

The runner creates an isolated sandbox by copying the exact bytes of:

```text
workspaces/worth-ui/apps/platform-pulse/app/main.wui
```

The product is launched through a real `--source-root <absolute-directory>`
configuration that the ordinary executable owns. That configuration selects
source location only. It cannot accept a parsed source artifact, capability,
candidate submission, application generation, mounted frame, watcher event,
host receipt, or native shape.

The checked-in source is the immutable baseline. Each courtroom action applies
one named delta to its private sandbox. Tests do not mutate the checkout,
maintain a second hand-written canonical pulse, share mutable worlds, or copy
valid identities as literals.

### Hostile Sequence

One required ordinary-gate process performs this sequence:

1. Launch the exact product binary against the isolated canonical source.
2. Wait at most 5 seconds for the product-issued first-frame publication
   observation.
3. Locate the native window by child-process identity, not by an unscoped
   global title search.
4. Keep the process alive for at least 500 milliseconds after first
   publication and externally observe the 160-by-96 client area as admitted
   blue.
5. Apply one atomic blue-to-green source edit from outside the process.
6. Wait at most 5 seconds for a typed replacement publication relating
   predecessor generation, active generation, and successor frame.
7. Externally observe admitted green in the same native client area.
8. Apply one stable malformed source edit.
9. Wait at most 5 seconds for the typed DSL/source denial and explicit
   predecessor-preserved observation.
10. Verify that the process and window remain live and the same admitted green
    predecessor remains externally visible.
11. Restore the exact canonical bytes atomically.
12. Wait at most 5 seconds for a fresh blue successor and externally observe
    blue.
13. Request normal native-window close through the operating-system window
    boundary.
14. Wait at most 5 seconds for typed shutdown completion and successful child
    exit.
15. Prove that no pulse child, watcher registration, host surface, observation
    reader, or sandbox resource remains owned by the scenario.

There are no blind retries. Timeout, child exit, observation-stream closure,
window lookup failure, native capture failure, source-action failure, product
denial, and cleanup failure are distinct typed runner outcomes.

### Independent Observations

The courtroom joins two evidence classes without allowing either to certify
itself:

- **External consequence evidence** is owned by the runner's native-platform
  adapter: child-process liveness, process-bound native window identity,
  client-area pixel capture, externally applied source writes, native close,
  process exit, and resource cleanup.
- **Product causal evidence** is a versioned lifecycle-observation envelope
  derived from real public application, mounted-frame, replacement, source-
  denial, watcher-shutdown, and host-shutdown outcomes.

The external pixel expectation is authored independently as the canonical
blue and green client-area result. It may not be produced by the DSL compiler,
runtime paint projection, host adapter, product observation encoder, or an
egui shape generated by the process under test.

Milestone 3.10.3 correlates a visible native result with the process run and
published frame, but it does not claim Milestone 3.11's
`screen point -> mounted receipt identity` bridge.

### Required Survival and Cost

- The predecessor application generation, mounted frame, and visible pixels
  remain current after malformed source.
- A successful edit produces a distinct active generation and published frame.
- The native process stays live after first publication and throughout denial
  preservation.
- Observation publication is event-lane work only. Unchanged frames emit no
  lifecycle events and perform no pulse-certification allocation, encoding,
  filesystem write, or diagnostic materialization.
- One ordinary courtroom uses one child process, one native window, one source
  sandbox, one observation stream, and no retry.
- Failure artifacts are bounded to 64 MiB per scenario and contain only the
  minimized source snapshot, structured event trace, process output, native
  captures, environment posture, and teardown result needed for reproduction.
- A failed scenario still attempts bounded teardown; teardown failure remains a
  second typed fact and never replaces the primary defect.

### Mutation Sensitivity

The proof family must demonstrate that it turns red when:

- the product exits or the event stream closes after publishing but before the
  500-millisecond liveness hold and native capture;
- a syntactically valid publication envelope is supplied without the matching
  child process and process-bound window;
- expected pixels appear without a matching product-issued publication;
- the source edit is written but no operating-system watcher delivery reaches
  replacement;
- the malformed edit reports timeout, generic failure, or process death rather
  than the DSL/source denial;
- the predecessor generation or frame changes during denial preservation;
- application-local egui drawing is introduced outside the host adapter;
- the runner imports or constructs runtime, DSL, certification-support, or
  mounted-authority values;
- native close is replaced by process termination; or
- the required Windows executable lane is omitted, ignored, or reports zero
  executed courtroom scenarios.

These controls may combine hostile fixtures, dependency audits, source
topology audits, negative twins, and causal regressions. They must not require
shipping a deliberately broken product binary.

## Product Decision Lock

### Evidence Lanes Are Different Contracts

Worth UI has four named evidence lanes:

1. **Compile and topology evidence** proves invalid public programs and
   dependency shapes are unavailable.
2. **Focused semantic evidence** proves local algorithms, parsers, state
   machines, models, and bounded structures with independent local oracles.
3. **In-process integration evidence** proves real subsystem and authority
   interactions without claiming operating-system product entry.
4. **Executable-world evidence** proves a user-visible product claim through
   the actual binary and native composition root.

Every proof obligation records its required lane. A stronger lane does not
delete cheaper causal evidence, and a collection of weaker lanes does not
become an executable-world claim by accumulation.

Existing 3.10.2 egui-context and real-watcher tests remain in the consolidated
`application_contracts` target and are renamed or documented as in-process
integration. Their assertions stay valuable because they localize adapter and
replacement defects cheaply.

### One Permanent Executable-World Target

The pulse package gains exactly one explicit integration target:

```text
executable_world
```

It is disabled from the ordinary fast test lane by a test-membership feature
that must not appear in product `cfg` branches or change binary semantics. The
required native command is separate, named, and non-optional in merge
certification:

```powershell
cargo test `
  --manifest-path workspaces/worth-ui/Cargo.toml `
  -p worth-ui-platform-pulse `
  --features executable-world `
  --test executable_world `
  -- --test-threads=1
```

`autotests = false` remains. Later milestones add modules and proof obligations
inside this target; they may not add another executable-world integration
target or launch one process per assertion when one cumulative journey can
honestly own the sequence.

The feature controls only whether Cargo builds the certification target.
Boundary enforcement rejects any `cfg(feature = "executable-world")` in
product source and any product dependency or behavior that changes under it.

### Product Launch Configuration Is Ordinary

The product executable accepts:

```text
--source-root <absolute-directory>
```

The no-argument human command continues to use the checked-in canonical
`app/` directory. The explicit source root is a real product launch
configuration, not a test-only constructor or alternate composition root.

Source-root validation happens before native effects and returns a typed launch
denial. The selected directory still enters through
`WorthUiFilesystemSourceProvider`, `WorthUiFilesystemSourceWatcher`, the
DSL-owned sealed handoff, and the ordinary application lifecycle.

No environment variable, feature flag, fixture registry, or ambient global may
inject product meaning or authority.

### Lifecycle Observation Is Derived and Versioned

The pulse package exposes one narrow library facade containing only the
cross-process lifecycle observation contract. It does not export application
construction, source lowering, native-frame control, watcher control, host
access, or runtime types.

The canonical envelope is:

```rust
pub struct PlatformPulseLifecycleObservationEnvelope {
    protocol: PlatformPulseLifecycleObservationProtocol,
    run: PlatformPulseObservationRunIdentity,
    sequence: PlatformPulseObservationSequence,
    outcome: PlatformPulseLifecycleObservation,
}

pub enum PlatformPulseLifecycleObservation {
    ProcessStarted(PlatformPulseProcessStarted),
    FirstFramePublished(PlatformPulseFirstFramePublished),
    ReplacementPublished(PlatformPulseReplacementPublished),
    ReplacementDeniedPreserving(PlatformPulseReplacementPreserved),
    ShutdownCompleted(PlatformPulseShutdownCompleted),
    TerminalFailure(PlatformPulseTerminalFailure),
}
```

The exact private representation may differ, but these semantic variants,
protocol identity, schema version, run identity, monotonic sequence, and
outcome distinctions are fixed.

Observation constructors in the pulse contract consume the real public receipt
or denial type whose occurrence they project. The binary may not mint
publication, preservation, or shutdown observations from raw IDs, booleans,
debug strings, or copied values.

The protocol identity is
`worth-ui.platform-pulse.lifecycle-observation`, schema version `1`. Each
envelope is serialized as one stdout line beginning with the exact token
`WORTH_UI_PLATFORM_PULSE_EVENT`, followed by one ASCII space and canonical
JSON. Sequence starts at `1` and increases by exactly one for each event in the
product-issued run. Human-oriented context may render to
stderr, but strings do not encode the typed outcome. The runner parses the
shared versioned contract and rejects missing, duplicate, out-of-order,
foreign-run, unsupported-version, or trailing-post-shutdown events.

The canonical core carries only the facts needed to interpret the lifecycle
outcome: protocol, run, sequence, event kind, source snapshot where applicable,
predecessor and active generation where applicable, mounted frame where
applicable, typed denial family, actual native-effect count, and shutdown
resource disposition. Raw source text, credentials, arbitrary debug output, and
unbounded diagnostic structures are excluded.

The observation is not product authority. It cannot be fed back into Worth UI,
authorize replacement, reconstruct a receipt, or substitute for external
native evidence. It is a bounded derived view of truth already committed by
the owning product boundary.

The existing `WORTH_UI_PLATFORM_PULSE_PUBLISHED` and
`WORTH_UI_PLATFORM_PULSE_REPLACED` debug lines retire as machine-consumed
protocol when the versioned envelope lands. Equivalent human context may render
on stderr, but there is one canonical machine observation stream, one decoder,
and no compatibility parser that accepts both old marker prose and the new
contract as interchangeable truth. Historical 3.10.2 evidence continues to
record the markers that existed when it was captured.

### Executable-World Progression Is Compiler-Visible

The runner exposes one sealed typestate progression:

```rust
let installed: PulseExecutableWorld<Installed> =
    PulseExecutableWorld::install(canonical_pulse)?;

let awaiting: PulseExecutableWorld<AwaitingFirstFrame> =
    installed.launch(cargo_built_pulse)?;

let published: PulseExecutableWorld<Published> =
    awaiting.await_first_frame(first_frame_deadline)?;

let awaiting_replacement: PulseExecutableWorld<AwaitingReplacement> =
    published.apply_source_delta(blue_to_green)?;

let green: PulseExecutableWorld<Published> =
    awaiting_replacement.await_published_successor(replacement_deadline)?;

let awaiting_denial: PulseExecutableWorld<AwaitingPreservation> =
    green.apply_malformed_source(malformed_component)?;

let preserved: PulseExecutableWorld<PreservedPredecessor> =
    awaiting_denial.await_preserved_predecessor(denial_deadline)?;

let closed: PulseExecutableWorld<Closed> =
    preserved.close_native_window(shutdown_deadline)?;
```

The concrete names are binding; generic implementation mechanics are not.
Only the owning transition consumes one state and constructs the next.
Published, preserved, and closed states carry the exact evidence bundle earned
at that transition. Invalid ordering, assertion before observation, reuse of a
consumed world, and successful completion without teardown do not compile.

Runner typestate is certification authority over proof progression only. It
does not become application, source, mounted, host, Query, intent, service, or
inspection authority.

### Canonical Worlds and Deltas

The executable-world portfolio begins with:

- `CanonicalPlatformPulse`: exact checked-in ordinary baseline;
- `MalformedPulseSource`: explicit invalid-source delta;
- `MissingPulseSource`: explicit invalid-installation delta; and
- `InterruptedPulseProcess`: explicit product-process failure delta.

The baseline is immutable and reusable. Every scenario receives an isolated
sandbox copy and applies only the delta relevant to its claim. World
construction returns semantic handles and typed failures for installation,
launch, action, observation, and teardown.

Later milestones add narrow siblings rather than enlarging a universal
fixture:

- Milestone 3.13 adds Query installation and projection deltas.
- Milestone 3.14 adds native input and intent-result deltas.
- Milestone 3.15 adds portal, focus, motion, and service deltas.
- Milestone 3.16 adds theme and appearance deltas.
- Milestones 3.17 and 3.18 add expression and module-source deltas.
- Milestones 3.19 through 3.22 add diagnostic, inspection, replay, agent, and
  inspector observations.
- Milestone 3.23 adds the hostile Workflow Editor world as a second canonical
  product regime, not a replacement for the pulse baseline.

### Native Platforms Are Explicit Certification Postures

The native observation boundary is platform-neutral above per-operating-system
adapters. This milestone requires a Windows adapter because Windows is the
environment in which the escaped main-thread failure was observed and measured.
It must:

- bind a native window to the exact child process;
- report client-area bounds and native visibility;
- capture client-area pixels without calling product or egui capture APIs;
- request normal native close; and
- distinguish unsupported platform mechanics from product failure.

Windows becomes `CertifiedExecutable`. Linux and macOS remain explicitly
`CompileOnly` or `NotYetCertifiedExecutable` until their real window, pixel,
input, and close adapters run in required platform lanes. A `cfg` that compiles
zero scenarios cannot report `CertifiedExecutable`.

The stable platform contract anticipates committed Windows, macOS, Linux X11,
and Linux Wayland implementations. Only the Windows implementation is created
in this milestone; empty platform placeholders are forbidden.

### External Evidence Never Becomes Product Truth

Native captures, process handles, observation traces, timestamps, and runner
verdicts are certification artifacts. They may adjudicate a product claim but
cannot be imported by production runtime, host, source, Query, service, or
inspection code.

Later inspection and replay capabilities consume runtime-owned evidence, not
the executable runner's derived artifacts. The runner may compare public
inspection results to external observations; it may not become their source.

## Destination Topology

The milestone establishes this populated destination. Files marked
`committed successor` describe required insertion homes and are not created
empty.

```text
workspaces/worth-ui/
  Cargo.toml
    [modify: add shared serde_json beside existing serde]

  apps/platform-pulse/
    Cargo.toml
      [modify: one library facade, one explicit test target, serde/serde_json,
       bounded native-world dev dependencies]

    src/
      lib.rs
        [create: lifecycle-observation contract facade only]
      observation_contract/
        mod.rs
          [create: curated exports]
        envelope.rs
          [create: protocol/run/sequence envelope]
        lifecycle.rs
          [create: typed lifecycle observation variants]
        projection.rs
          [create: constructors consuming real public receipts and denials]
      main.rs
        [modify: parse launch configuration and enter eframe]
      launch_configuration.rs
        [create: default or explicit source-root admission]
      lifecycle_observation_publication.rs
        [create: bounded stdout protocol publication]
      application.rs
        [modify: consume admitted source-root configuration]
      native_frame.rs
        [modify: publish observations from actual outcomes]
      source_watch.rs
        [existing: watcher worker lifecycle]

    tests/
      executable_world.rs
        [create: sole integration-target root and courtroom registration]
      executable_world/
        courtroom/
          platform_pulse_lifecycle.rs
            [create: cumulative ordinary-gate journey]
          workflow_editor.rs
            [committed successor: Milestone 3.23 hostile product regime]
        installation/
          canonical_platform_pulse.rs
            [create: exact-source immutable baseline]
          isolated_source_sandbox.rs
            [create: per-scenario installation lifecycle]
          query_projection.rs
            [committed successor: Milestone 3.13]
        product_process/
          launch.rs
            [create: exact Cargo-built child process]
          progression.rs
            [create: sealed executable-world typestate]
          shutdown.rs
            [create: normal close, exit, and resource disposition]
        source_delta/
          atomic_replacement.rs
            [create: valid, malformed, and recovery edits]
          theme_appearance.rs
            [committed successor: Milestone 3.16]
          authored_expression.rs
            [committed successor: Milestone 3.17]
          authored_module.rs
            [committed successor: Milestone 3.18]
        external_observation/
          lifecycle_stream.rs
            [create: versioned child-pipe decoder and sequence validation]
          native_client_area.rs
            [create: process-bound window and pixel observation contract]
          process_liveness.rs
            [create: post-publication and exit observations]
          inspection.rs
            [committed successor: Milestones 3.19-3.22]
        native_platform/
          contract.rs
            [create: window, capture, input, and close port]
          windows.rs
            [create: current certified adapter]
          linux_x11.rs
            [committed successor: first required X11 execution lane]
          linux_wayland.rs
            [committed successor: first required Wayland execution lane]
          macos.rs
            [committed successor: first required macOS execution lane]
        host_action/
          viewport.rs
            [committed successor: Milestone 3.12]
          intent.rs
            [committed successor: Milestone 3.14]
          service.rs
            [committed successor: Milestone 3.15]
        adjudication/
          source_to_pixel.rs
            [create: independent first-frame and replacement verdict]
          predecessor_preservation.rs
            [create: denial identity and visible-state verdict]
          lifecycle_cleanup.rs
            [create: shutdown and residue verdict]
          identity_trace.rs
            [committed successor: Milestone 3.11]
          bounded_rebind.rs
            [committed successor: Milestone 3.12]
        evidence_artifact/
          failure_bundle.rs
            [create: bounded reproducible failure material]
          retention.rs
            [create: cleanup, expiry, and size policy]

  crates/worth-ui-certification/tests/
    suites/application_contracts.rs
      [modify: retain consolidated in-process proof only]
    application_contracts/platform_pulse.rs
      [modify: honest in-process egui adapter classification]
    application_contracts/platform_pulse_lifecycle.rs
      [modify: honest in-process watched-lifecycle classification]

  docs/application-lifecycle.md
    [modify: distinguish human, integration, and executable-world workflows]
```

### Structural Axes and Owners

- `observation_contract/` is owned by the pulse product boundary and contains
  only stable, derived, cross-process lifecycle observation meaning. It excludes
  runner mechanics and application implementation.
- `courtroom/` owns complete falsifiable product scenarios. It orchestrates
  named responsibilities but does not implement installation, process,
  platform, or adjudication mechanics.
- `installation/` owns causally valid immutable baselines and isolated
  sandboxes. It excludes actions after launch.
- `product_process/` owns child-process and typestate lifecycle. It excludes
  native-platform semantics and product authority construction.
- `source_delta/` owns externally authored filesystem changes and their exact
  source provenance, including later theme, expression, and module deltas.
- `external_observation/` owns facts observed outside the product. It never
  constructs product receipts.
- `native_platform/` owns volatile operating-system mechanics behind one
  platform-neutral certification port.
- `host_action/` is the committed destination for later externally applied
  viewport, input, and service actions. It does not interpret runtime meaning
  or own appearance.
- `adjudication/` joins independent evidence into certification verdicts. Its
  products remain derived and cannot enter production.
- `evidence_artifact/` owns bounded failure retention and disposal, not
  product logs, runtime diagnostics, or replay truth.

The tree forbids `helpers`, `common`, `util`, `shared`, a generic event bag, a
single universal world builder, a single platform switch file, app
implementation under the public library facade, and runner code inside
`worth-ui-certification` merely because certification already exists there.

The executable target belongs to the product package because Cargo must bind
it to the exact binary and because later pulse work extends that same
composition root. In-process platform integration remains in
`worth-ui-certification`; the two proof owners have different entry,
mechanism, lifecycle, cost, and replacement fate.

## Dependency and Visibility Enforcement

The pulse product library and binary may depend only on the curated Worth UI
facade, egui host adapter, eframe, serialization required by the observation
contract, and source-location/CLI mechanics justified by inventory.

The executable-world target may depend on:

- the pulse package's observation-contract facade;
- standard process, filesystem, and synchronization facilities;
- narrow serialization support;
- the current native-platform observation mechanism; and
- assertion libraries only when they do not compute the disputed semantics.

It must not depend on:

- `worth-ui-runtime`;
- `worth-ui-dsl`;
- `worth-ui-certification`;
- `worth-ui-test-support`;
- `worth-ui-query-binding` before Milestone 3.13 establishes a production
  installation boundary;
- application implementation modules; or
- host internals or egui paint/capture APIs.

Boundary-check gains explicit dependency and source-identifier denials for
these rules. Topology certification proves:

- `src/lib.rs` exports only the observation contract;
- product source contains no executable-world feature branch;
- the runner launches only `CARGO_BIN_EXE_worth-ui-platform-pulse`;
- no second executable-world target exists;
- no later milestone creates another pulse binary or canonical source; and
- platform certification cannot be reported from a zero-test or skipped lane.

## Public Contract and DX

### Human

The ordinary human command remains:

```powershell
cargo run --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui-platform-pulse
```

No arguments means the checked-in canonical source. Lifecycle events render as
bounded machine-readable observations while errors retain useful human
context. A human never supplies IDs, receipts, runtime phases, or test
features.

### External Product Installation

The honest configurable path is:

```powershell
cargo run `
  --manifest-path workspaces/worth-ui/Cargo.toml `
  -p worth-ui-platform-pulse `
  -- --source-root C:\absolute\pulse-installation
```

The source-root path is admitted once before watcher startup. Invalid,
relative, missing, non-directory, or missing-entry-source configurations fail
before native effects with distinct typed launch denials.

### Certification

The required Windows executable-world command is the single command shown
earlier. It builds the product binary once, starts one child, and owns teardown.
A failure reports:

- proof obligation and scenario identity;
- world-construction phase;
- last valid typestate;
- child exit/liveness posture;
- last accepted lifecycle sequence;
- expected and observed native consequence;
- primary failure;
- independent teardown failure, if any; and
- bounded retained artifact location.

It does not report a generic panic as the product outcome.

## Cost Lanes and Frozen Budgets

Phase 1 records an exact opening measurement before implementation. The
following ceilings govern the destination unless closing evidence explicitly
amends them with cause:

- exactly one added Cargo integration target and one added pulse library target;
- zero nested Cargo invocations;
- one child process and one native window in the ordinary cumulative courtroom;
- 5 seconds to first publication;
- 5 seconds for each watched replacement or typed denial;
- 5 seconds for normal shutdown and child exit;
- 20 seconds total execution time for the ordinary executable-world journey on
  the measured Windows posture;
- zero automatic retries and zero ignored or quarantined required scenarios;
- at most 256 lifecycle observations and 1 MiB encoded observation bytes in the
  ordinary journey;
- at most 64 MiB retained failure evidence per scenario;
- at most 240 seconds for clean build plus link of the pulse package and
  executable-world target on the measured baseline;
- at most 20 seconds for warm relink after package-only clean;
- at most 500 MiB of additional package-identifiable retained artifacts beyond
  the already measured pulse executable; and
- zero observation encoding, publication, polling, or runner-specific work on
  unchanged product frames.

Elapsed-time evidence names operating system, CPU, toolchain, renderer,
window-server posture, cold or warm state, and exact source revision. Structural
counters distinguish product events, decoded envelopes, native captures,
source writes, process launches, window lookups, close requests, retries, and
retained bytes.

Stress, repeated launch/close, edit storms, and later multi-platform matrices
may occupy scheduled lanes, but the core launch, visible first frame, valid
edit, malformed preservation, recovery, and clean shutdown claim runs in the
required merge lane.

## Phase 1: Evidence Reclassification and Courtroom Freeze

### What Becomes True

Every existing 3.10.2 pulse test and closure claim names the boundary it
actually exercises. The executable-world claim, plausible defects, world
provenance, platform posture, independent observations, mutation controls,
costs, and successor extensions are frozen before product or runner code
changes.

### Required Work

- Inventory the exact production and test entry paths from Cargo binary through
  shutdown.
- Classify each existing pulse proof as compile/topology, focused semantic,
  in-process integration, manual executable, or automated executable-world.
- Correct names and historical evidence wording that call an in-process shell
  or egui context an executable end-to-end world.
- Freeze the observation protocol, typestate transitions, Windows native
  observation boundary, canonical world, scenario deltas, courtroom sequence,
  mutation controls, cost lanes, and destination topology.
- Record the opening build, target, process, launch, artifact, and runtime
  posture.

### Mechanical Prohibitions

- No test count or aggregate suite result may imply a stronger lane.
- No manual launch may be recorded as automated executable proof.
- No in-process integration test may use "end-to-end", "executable world", or
  "product entry" in its closure claim.
- No implementation phase begins while a required edge or oracle remains
  ambiguous.

### Exit Gate

The evidence inventory has no unclassified product-visible claim, the
historical correction is explicit rather than silently rewritten, and the
courtroom would fail for the escaped stack defect even if every existing
in-process test stayed green.

The next phase may trust the fixed protocol and product-entry boundary, but not
yet the existence of automated executable evidence.

## Phase 2: Product Launch and Observation Contract

### What Becomes True

The real pulse executable can select an isolated source installation through an
ordinary configuration and emit a bounded versioned lifecycle observation
stream derived only from real public outcomes.

### Required Work

- Add the source-root launch configuration with default canonical behavior.
- Add the observation-contract-only pulse library facade.
- Project process start, first publication, replacement publication,
  predecessor-preserving denial, terminal failure, and shutdown completion
  from their real typed owners.
- Serialize monotonically sequenced envelopes on stdout.
- Preserve useful human diagnostics separately without making strings
  authoritative.
- Add protocol round-trip, ordering, unsupported-version, and constructor-
  provenance evidence at the narrowest honest boundary.

### Mechanical Prohibitions

- Product code cannot branch on the executable-world feature.
- Observation events cannot be constructed from raw IDs or booleans where a
  real receipt or denial exists.
- Observation publication cannot run on unchanged frames.
- Observation types cannot authorize or reconstruct product action.
- The library facade cannot export application implementation.

### Exit Gate

A normal human launch remains unchanged, an isolated real source root reaches
the same production pipeline, and the observation stream is typed, bounded,
versioned, monotonic, and causally derived.

The next phase may trust the product entry and observation protocol, but not
yet native-world adjudication.

## Phase 3: External World Progression and Windows Native Boundary

### What Becomes True

One compiler-visible external world can install the canonical source, launch
the exact binary, progress to a real first published native frame, observe a
process-bound Windows client area, and close with explicit teardown.

### Required Work

- Add the sole `executable_world` integration target and membership feature.
- Build the canonical installation and isolated-sandbox lifecycles.
- Launch only Cargo's exact executable path with stdout piped to the shared
  observation decoder.
- Implement the sealed world typestate and typed phase failures.
- Implement the Windows process-bound window, client-area capture, liveness,
  and native-close adapter.
- Join first-frame observation, 500-millisecond liveness, and independent blue
  pixels into the first executable adjudication.
- Add boundary and topology enforcement preventing private imports, product
  feature branches, second targets, and zero-test certification.

### Mechanical Prohibitions

- No application function, eframe callback, watcher, or host method is called
  by the runner.
- No target-directory search or nested Cargo invocation locates the binary.
- No title-only window match can satisfy process identity.
- No product screenshot or egui shape is the external pixel oracle.
- No emitted event alone progresses the world to `Published`.

### Exit Gate

The required Windows target fails if the product dies before a stable first
frame, if no process-bound native pixels exist, or if the causal publication
observation is absent or inconsistent.

The next phase may trust honest executable entry, world progression, and
external native observation.

## Phase 4: Watched Replacement, Denial Preservation, and Recovery

### What Becomes True

The same child process and native window survive the complete blue, green,
malformed-preservation, blue-recovery sequence through external source actions
and real watcher delivery.

### Required Work

- Add exact atomic valid, malformed, and recovery source deltas.
- Extend typestate through awaiting replacement, published successor, awaiting
  preservation, preserved predecessor, recovery, and closed states.
- Correlate source actions, watcher-derived product observations, generation
  and frame progression, process liveness, and independent client pixels.
- Distinguish watcher timeout, source denial, replacement denial, child exit,
  observation failure, and native observation failure.
- Add mutation-sensitive positive and negative twins for event-only, pixel-
  only, wrong-reason, premature-exit, direct-paint, and forced-termination
  shortcuts.
- Require normal native close, typed shutdown completion, successful exit, and
  residue-free teardown.

### Mechanical Prohibitions

- The runner cannot inject snapshots, watcher events, submissions, receipts,
  generations, frames, or shapes.
- A denial cannot progress through a success state.
- Preserved pixels without preserved product identity cannot pass.
- `Stop-Process`, task kill, or equivalent termination cannot satisfy normal
  shutdown.
- Teardown failure cannot be hidden by the primary scenario result.

### Exit Gate

One required executable journey proves the complete human-visible 3.10.2
lifecycle through the actual product composition root and fails for the named
shortcut mutations.

The next phase may trust the permanent executable-world foundation.

## Phase 5: Cost Closure and Successor Handoff

### What Becomes True

Executable-world certification is a durable, budgeted, documented part of the
3.x architecture, and every later Platform Pulse milestone has an additive
insertion path.

### Required Work

- Measure clean and warm build/link, first publication, full journey,
  observation volume, native captures, process launches, retries, retained
  artifacts, and teardown on the exact final source.
- Run the ordinary pulse integration lane and the new executable lane
  independently and report their distinct claims.
- Update `docs/application-lifecycle.md` with human, in-process integration,
  and executable-world workflows.
- Update the roadmap's permanent pulse contract and each 3.11 through 3.23
  handoff.
- Record Windows as certified executable and other platforms without
  overclaiming.
- Run boundary-check, agent-context, topology, line-cap, format, clippy,
  workspace, protocol, integration, executable, and documentation gates.
- Remove redundant, misnamed, stale, or duplicate pulse evidence made obsolete
  by the final classification.

### Mechanical Prohibitions

- No later milestone needs to move the executable-world facade or split a
  universal fixture to add its domain.
- No correctness claim depends only on a routinely skipped or ignored lane.
- No closeout reports a non-Windows platform as executable-certified without a
  required real lane.
- No failure artifacts, sandboxes, windows, or child processes remain after a
  passing run.

### Exit Gate

Milestone 3.11 can add point-to-mounted identity by extending
`adjudication/identity_trace.rs` and the same cumulative courtroom. It does not
need to invent process launch, source installation, window discovery, native
capture, action sequencing, observation transport, failure retention, or
teardown.

## Documentation Deliverables

### Governing Architecture

`plans/worth-ui/milestone-3.10.3.md` is the sole governing design for this
corrective gate. Implementation plans may sequence edits but cannot redefine
the evidence lanes, product entry, protocol, typestate, platform posture,
courtroom, topology, or budgets.

### Historical Evidence Correction

Revise:

- `plans/worth-ui/milestone-3.10.2.md`;
- `plans/worth-ui/milestone-3.10.2-phase-3-proof-ledger.csv`; and
- `plans/worth-ui/milestone-3.10.2-phase-4-proof-ledger.csv`.

The revision must preserve valid product and integration evidence while
explicitly identifying the later-discovered executable-world gap. It must not
pretend that 3.10.2 originally automated the composition root.

### Continuing Roadmap

Revise `plans/worth-ui/worth_ui_roadmap.md` so:

- 3.10.3 blocks 3.11;
- the permanent pulse contract owns both the consolidated in-process lane and
  the sole executable-world lane;
- every 3.11 through 3.23 pulse requirement extends the existing world;
- native platform certification is explicit; and
- a future 3.24 enters mature world infrastructure and may not introduce its
  first product-entry proof.

### Human and Operator Workflow

Revise `workspaces/worth-ui/docs/application-lifecycle.md` during
implementation. It must keep the no-argument human workflow, add the explicit
source-root workflow, explain the typed lifecycle stream, distinguish manual,
integration, and executable evidence, name failure artifacts and cleanup, and
remain executable against the real facade and binary.

## Must Ship

- an honest reclassification of existing pulse evidence;
- one ordinary source-root launch configuration with no test-only authority;
- one versioned, typed, derived lifecycle observation contract;
- one observation-contract-only pulse library facade;
- one explicit executable-world integration target in the pulse package;
- one sealed compiler-visible world progression;
- one immutable canonical pulse baseline and isolated scenario-delta model;
- one current Windows external window, client-area, liveness, and close adapter;
- one cumulative real-process blue/green/malformed/recovery/shutdown courtroom;
- independent native consequence and product causal observations;
- mutation-sensitive shortcut rejection;
- bounded failure artifacts and teardown;
- dependency, visibility, topology, platform, and test-membership enforcement;
- explicit build, execution, observation, artifact, and flake budgets; and
- durable 3.11 through 3.23 insertion homes.

## Must Preserve

- every actual product capability and authority guarantee closed by 3.10,
  3.10.1, and 3.10.2;
- the checked-in `app/main.wui` source and sole pulse executable;
- one public application lifecycle and one canonical runtime-to-host path;
- runtime ownership of visible meaning and host ownership of native mechanics;
- Query-free pulse operation before 3.13;
- whole-application replacement language before 3.12;
- 3.11 ownership of pixel-to-mounted identity;
- 3.12 ownership of bounded semantic rebind;
- 3.13 ownership of real Query projection consumption;
- later ownership of intents, services, appearance, expressions, modules,
  diagnostics, inspection, replay, and the Worth Inspector;
- consolidated compile-contract and in-process integration topology;
- no test-only production branch, alternate composition root, hidden
  constructor, or weakened validation;
- no ordinary-frame certification cost; and
- explicit non-certification for unexecuted native platforms.

## Acceptance Evidence

Milestone 3.10.3 is complete only when:

- historical 3.10.2 automated evidence is accurately classified without
  discarding its valid integration claims;
- the exact Cargo-built pulse binary enters the real Windows native event loop
  from an isolated copy of canonical checked-in source;
- no runner code imports or constructs product internals or authority;
- no runner source includes, path-imports, or recompiles product implementation
  modules outside the pulse package's observation-contract facade;
- a typed, versioned, monotonic lifecycle stream is derived only from real
  product outcomes and cannot authorize product work;
- compiler-visible world progression prevents out-of-order assertion and
  completion without teardown;
- first publication is followed by a stable live process, process-bound native
  window, independent blue client pixels, and matching product causal
  observation;
- one external atomic edit produces matching green successor identity and
  pixels;
- malformed source produces the owning typed denial while exact predecessor
  identity and green pixels remain current;
- exact canonical restoration produces a fresh blue successor;
- normal native close produces typed watcher/host shutdown evidence, successful
  child exit, and zero residue;
- event-only, pixel-only, wrong-reason, premature-exit, direct-paint, injected-
  source, forced-termination, and skipped-platform shortcuts turn named
  evidence red;
- the fast integration lane and executable lane remain separately runnable and
  separately claimed;
- all frozen build, runtime, observation, artifact, and retry budgets pass on
  exact final source;
- Windows is reported as executable-certified and no other platform is
  overclaimed;
- later Platform Pulse milestones have additive homes in the same target and
  require no new product-entry infrastructure; and
- boundary, agent-context, topology, line-cap, format, clippy, workspace,
  protocol, integration, executable, documentation, and cost gates are green.

## Successor Handoff

Milestone 3.11 inherits:

- `PulseExecutableWorld<Published>`;
- the exact process, window, source, generation, and mounted-frame correlation;
- the Windows external native observation boundary;
- the versioned lifecycle stream;
- the cumulative pulse courtroom;
- bounded failure artifacts and teardown; and
- a committed `adjudication/identity_trace.rs` insertion home.

It adds capture and point-to-mounted identity to that live world. It may not
substitute an in-process egui frame, loose PNG, reconstructed tree, or new
executable harness.

Every following milestone extends the same progression:

- 3.12 adds external edit to admitted bounded rebind;
- 3.13 adds causally installed Query projection state;
- 3.14 adds real host input to admitted intent;
- 3.15 and 3.16 add native service and appearance consequences;
- 3.17 and 3.18 add authored expression and module deltas;
- 3.19 through 3.22 add product diagnostics, inspection, replay, agent, and
  inspector observations; and
- 3.23 adds the hostile Workflow Editor regime through the same runner.

By the next milestone after 3.23, product entry, action, observation,
correlation, artifact retention, and teardown are established infrastructure.
New work should feel like refinement of a lived-in product world, not its first
inhabitation.
