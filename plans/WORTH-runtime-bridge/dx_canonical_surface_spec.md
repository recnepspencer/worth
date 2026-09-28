# WORTH Runtime Bridge DX Canonical Surface Spec

## Purpose

This document defines the minimum canonical public shape `worth-runtime-bridge`
must converge toward before Milestone 13 implementation becomes the primary
focus.

It is intentionally narrow.

The goal is not to productize every bridge subsystem at once.
The goal is to decide what users and tests are supposed to memorize first so the
reference workload and certification suites can target the real bridge.

The everyday feel target for that surface is defined in
[`dx_standard_path_spec.md`](./dx_standard_path_spec.md).

---

## Canonical Import Path

Primary path:

- `worth_runtime_bridge::facade`

Target property:

- external users and integration tests should not need to reach past the facade
  to perform ordinary bridge work

---

## Canonical Product Memory Model

An engineer or AI agent should be able to remember the bridge as six jobs:

1. build bridge
2. route truth change
3. evaluate against truth view
4. open speculative session
5. discard or promote session
6. inspect bridge result

If the bridge requires a seventh mental anchor for ordinary work, that anchor
should be viewed suspiciously.

---

## Canonical Setup Flow

The normal setup story should revolve around:

- `RuntimeBridgeBuilder`
- explicit source registration
- explicit sink registration
- explicit policy and diagnostics configuration
- `build()`

Target property:

- one obvious production setup path
- no ambient host magic
- no hidden adapter assumptions

---

## Canonical Truth-Routing Flow

The normal routing story should revolve around:

- one route request
- one execution through `RuntimeBridge`
- one route result or delivery result

Target property:

- truth change ingestion should feel like one bridge job, not a pile of routing,
  lowering, delivery, and explanation nouns

---

## Canonical Truth-View Evaluation Flow

The normal evaluation story should revolve around:

- one truth-view selection
- one evaluation request
- one evaluation result

Target property:

- current, historical, and branch-aware evaluation should feel like variations
  of one coherent bridge job rather than unrelated subsystem doors

---

## Canonical Speculation Flow

The normal speculative story should revolve around:

- one preview or speculative declaration
- one session identity
- one discard or promote decision

Target property:

- branch-local work should feel explicit and typed
- preview and authoritative outcomes should never blur

---

## Canonical Diagnostics Flow

The normal diagnostics story should revolve around:

- one diagnostics entrypoint
- explanation, comparison, replay, and certification access beneath that

Target property:

- users should start from one diagnostics door, not guess between stream,
  merge, source, policy, replay, writeback, structural, and speculation modules

---

## Canonical Milestone 13 Testing Rule

The Milestone 13 reference workload should call only canonical bridge flows for:

- setup
- routing
- evaluation
- speculation
- discard or promotion
- diagnostics capture

If a Milestone 13 test needs to reach for internal subsystem details to perform
one of those ordinary jobs, the canonical surface is incomplete.
