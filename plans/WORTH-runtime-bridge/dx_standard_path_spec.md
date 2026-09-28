# WORTH Runtime Bridge DX Standard Path Spec

## Purpose

This document defines what the bridge should feel like in the standard path.

It is intentionally more opinionated than the broader DX docs.

The other bridge DX documents answer questions like:

- what belongs in the public boundary
- what should remain advanced or specialist
- what docs should exist

This document answers a different question:

- what should ordinary bridge code actually look like when the API is good

That distinction matters.

A bridge can have correct layering, careful boundaries, and strong diagnostics
and still feel harder to use than it should.

This document exists to prevent that outcome.

---

## Product Standard

The standard path should feel boringly obvious.

An engineer should be able to do normal bridge work by remembering only these
verbs:

- `builder`
- `route`
- `evaluate`
- `speculate`
- `discard`
- `promote`
- `diagnostics`

If normal work requires remembering substantially more public verbs than that,
the bridge is still too complicated.

---

## Governing Rule

The standard path is not merely:

- possible

It must be:

- obvious
- short
- typed
- safe by default
- diagnostic by default

The bridge should not require ordinary callers to think in terms of:

- validation phases
- admission phases
- lowering phases
- canonicalization phases
- replay phases
- family registry mechanics
- record-family inventory

Those remain real.
They do not belong in the everyday memory model.

---

## Golden Path Sketch

This is the target feel for ordinary bridge usage:

```rust
use worth_runtime_bridge::facade::*;

let bridge = RuntimeBridge::builder()
    .with_truth_source(relational_source)
    .with_compute_sink(signal_sink)
    .build()?;

let route = bridge.route(change)?;
let result = bridge.evaluate_current(route.target())?;

let session = bridge.speculate(spec_request)?;
let comparison = session.compare_to_main()?;

session.discard()?;
// or
session.promote(promotion_request)?;

let diag = bridge.diagnostics().explain_last()?;
```

This is not a final signature lock.

It is a feel lock.

If the shipped bridge cannot produce code that feels approximately this direct,
the DX work is not done.

---

## Standard Path Jobs

The standard path must optimize for these exact jobs:

1. build a bridge
2. route a truth change
3. evaluate against the default truth view
4. open a speculative session
5. compare speculative outcome to main
6. discard or promote
7. inspect what happened

These are the jobs the first examples, first tests, and first docs should all
demonstrate.

---

## Required Ergonomic Properties

### 1. One Obvious Setup Door

The setup path should be:

- `RuntimeBridge::builder()`
- attach truth source
- attach compute sink
- optionally refine policy and diagnostics
- `build()`

Users should not need to choose among multiple equally-primary constructors.

### 2. One Obvious Route Door

The route path should be one method family:

- `bridge.route(...)`

This may accept:

- a committed change
- a route request object

But it should not require users to manually orchestrate:

- ingest
- plan
- deliver
- prepare

for ordinary routing.

### 3. One Obvious Evaluation Door

The default evaluation path should be one method family:

- `bridge.evaluate_*`

At minimum it should support a simple default like:

- `evaluate_current(...)`

Advanced truth-view selection may exist, but the standard path should not force
callers to construct a historical or branch selector just to get started.

### 4. Speculation Should Feel Like Entering A Session

Speculation should return a session-shaped object.

That session should own:

- comparison to main
- discard
- promote
- speculative diagnostics readback

The standard path should not require callers to thread speculative session ids
through unrelated top-level methods unless they are doing explicitly advanced
work.

### 5. Diagnostics Must Attach To The Main Flow

Ordinary diagnostics should feel like:

- `bridge.diagnostics().explain_*()`

not:

- searching across record families to discover what object to inspect first

The first diagnostics questions should be job-shaped:

- explain this route
- explain this evaluation
- explain this speculative session
- explain this promotion
- explain the last bridge action

### 6. Safe Defaults Are Mandatory

The standard path should default to:

- current truth view
- standard diagnostics tier
- canonical routing mode
- ordinary promotion path

Callers may override those choices.
They should not have to specify them just to be correct.

### 7. Request Types Must Earn Their Keep

Use request objects only where they add real value:

- preserving typed intent
- bundling several meaningful options
- preventing invalid combinations

Do not force ceremonial request wrappers around trivial calls.

### 8. Advanced Power Must Stay Off The Happy Path

These remain real and important:

- family-aware writeback detail
- replay bundle authoring
- structural comparison control
- stream protocol control
- mapper containment and adapter authoring

But standard-path examples and tests should not need them.

---

## Standard Path Surface Targets

These are the public API shapes the bridge should aim to expose.

The exact names may refine slightly during implementation, but the structure
should remain stable.

### Setup

```rust
RuntimeBridge::builder()
RuntimeBridgeBuilder::with_truth_source(...)
RuntimeBridgeBuilder::with_compute_sink(...)
RuntimeBridgeBuilder::with_policy(...)
RuntimeBridgeBuilder::with_diagnostics(...)
RuntimeBridgeBuilder::build()
```

### Route

```rust
RuntimeBridge::route(...)
RuntimeBridge::route_request(...)
```

The first form should exist if ordinary routing can be inferred directly from
the arguments.

### Evaluate

```rust
RuntimeBridge::evaluate_current(...)
RuntimeBridge::evaluate(...)
```

The simple form should be the default memory path.
The richer form may support explicit truth-view requests.

### Speculate

```rust
RuntimeBridge::speculate(...)
BridgeSpeculativeSession::compare_to_main()
BridgeSpeculativeSession::discard()
BridgeSpeculativeSession::promote(...)
BridgeSpeculativeSession::diagnostics()
```

### Diagnostics

```rust
RuntimeBridge::diagnostics()
BridgeDiagnostics::explain_last()
BridgeDiagnostics::explain_route(...)
BridgeDiagnostics::explain_evaluation(...)
BridgeDiagnostics::explain_session(...)
BridgeDiagnostics::explain_promotion(...)
```

---

## What The Standard Path Must Hide

For ordinary usage, the standard path must hide direct exposure to:

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`
- raw family registry admission
- raw mapper-envelope assembly
- raw record lookup sequences

Those surfaces may remain public.

But if ordinary integration tests need them, the standard path has failed.

---

## Test And Docs Rule

The first public docs and the first end-to-end integration tests should all use
the standard path.

Specifically:

- `README`
- `QUICKSTART`
- `DAILY_WORKFLOWS`
- the first Milestone 13 end-to-end certification lanes

should all read like the golden path above.

If those materials need caveats like:

- "under the hood you actually need three more facade calls"
- "for now, use the lower-level admission methods"
- "this is not really the public path yet"

then the implementation is not ready to certify.

---

## Relationship To Milestone 12b And Milestone 13

Milestone 12b matters here because the standard path must remain simple even
though the bridge now has family-aware writeback depth under the hood.

Milestone 13 matters here because the pricing-shock reference workload should
be implemented through this standard path, not through raw subsystem
orchestration.

That is the real test:

- can the bridge stay powerful internally
- while still feeling obvious externally

If not, the DX work is incomplete.

---

## Bottom Line

The bridge standard path should feel like a well-designed framework, not a
protocol lab bench.

Ordinary users should be able to think:

- build
- route
- evaluate
- speculate
- discard or promote
- inspect

and be productive immediately.
