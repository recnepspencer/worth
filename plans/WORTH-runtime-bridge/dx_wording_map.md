# WORTH Runtime Bridge DX Wording Map

## Purpose

This document defines the preferred public wording for the bridge.

The bridge now has enough real power that wording drift will create real DX
debt if left alone.

This map exists to prevent the bridge from sounding like:

- several subsystems speaking at once
- milestone history leaking into the product surface
- multiple near-synonyms competing for the same job

The goal is one stable public vocabulary.

---

## Governing Rule

For ordinary bridge work, one bridge job should have one dominant public word.

Avoid:

- multiple equally plausible verbs
- subsystem-local terminology in everyday docs
- protocol-stage wording becoming the main product wording

---

## Canonical Everyday Verbs

These are the words the standard path should teach first:

- `build`
- `route`
- `evaluate`
- `speculate`
- `compare`
- `discard`
- `promote`
- `diagnostics`
- `explain`
- `replay`

These should dominate the first-read docs and the first-read API stories.

---

## Wording By Job

### Setup

Prefer:

- build bridge
- builder
- truth source
- compute sink

Avoid as first-read language:

- adapter graph
- source capability surface
- consumer shape

Those are real, but specialist or advanced.

### Routing

Prefer:

- route truth change
- route
- invalidation target
- delivery

Avoid as first-read language:

- ingest
- lower
- publish
- packet reduction

These are internal or specialist phase terms.

### Evaluation

Prefer:

- evaluate against truth view
- evaluation
- truth view
- current
- branch head
- historical commit
- branch snapshot

Avoid as first-read language:

- historical materialization declaration
- source packet admission

These are too phase-shaped for ordinary usage.

### Speculation

Prefer:

- speculative session
- preview session
- compare to main
- discard
- promote

Avoid as first-read language:

- admit preview declaration
- activate preview session
- terminal lifecycle transition

These are real but too protocol-forward for the happy path.

### Diagnostics

Prefer:

- diagnostics
- explain
- inspect
- compare
- replay
- certification evidence

Avoid as first-read language:

- retained record families
- canonical record inventory
- diagnostics state query

### Writeback And Promotion

Prefer:

- promotion
- authoritative handoff
- writeback outcome
- family-aware writeback

Avoid as first-read language:

- mapper envelope
- mapped family input
- family admission proof

These are specialist details unless the user is doing host-authoring or
specialist writeback work.

---

## Canonical Nouns

Prefer these as the dominant public nouns:

- bridge
- truth
- compute
- truth view
- route
- evaluation
- session
- diagnostics
- replay bundle
- certification bundle

Use with caution:

- artifact
- declaration
- contract
- proof
- admission
- lowering

These are appropriate in advanced or specialist docs, but they should not
overrun Tier 1 docs.

---

## Acceptable Public Synonyms

These are acceptable but should not become competing top-level brands.

### `truth change` and `committed patch`

Rule:

- use `truth change` in first-read docs
- use `committed patch` in API and protocol detail

### `speculative session` and `preview session`

Rule:

- use `speculative session` for the everyday mental model
- keep `preview session` where it is the exact type/protocol term

### `main` and `authoritative`

Rule:

- use `main` when talking about branch comparison stories
- use `authoritative` when talking about promotion, writeback, and causal
  boundary claims

---

## Surface Wording Rules

### Docs

Tier 1 docs should sound like:

- build
- route
- evaluate
- speculate
- inspect

Tier 2 docs may introduce:

- policy
- replay
- writeback
- structural comparison

Tier 3 docs may use the full protocol vocabulary.

### API

The API should follow the same hierarchy.

Primary methods should use:

- route
- evaluate
- speculate
- discard
- promote
- diagnostics

Raw methods may keep:

- validate
- admit
- lower
- canonicalize
- replay

But their specialist status should be explicit in docs and examples.

### Tests

Ordinary integration and end-to-end tests should read like the product docs.

If an ordinary workflow test reads like a protocol-lab transcript, that is a
warning sign.

---

## Bridge Narrative Wording

The bridge should be described publicly as:

- the causal protocol boundary between `worth-relational` truth and
  `worth-signal` computation

Prefer that over:

- synchronization layer
- middleware
- adapter toolkit
- event bus

Those all undersell or distort the bridge vision.

---

## Red Flag Phrases

These phrases should trigger a wording review when they appear in public docs
or the happy-path API story:

- subsystem
- admission
- lowering
- canonicalization
- retained record
- family mapper envelope
- structural reduction
- ontology mapping

They are not banned.
They are red flags for ordinary-path overexposure.

---

## Immediate Naming Review Targets

Based on the current bridge surface, these are the main areas that still need
active wording review:

- diagnostics methods around evaluation versus route symmetry
- speculation wording where preview/propose/compare concepts overlap
- keeping family-aware writeback power visible but not noisy
- ensuring docs talk about truth and compute jobs before protocol stages

---

## Completion Test

The wording map is doing its job when:

1. Tier 1 docs all sound like one product
2. the standard path code reads like the standard path spec
3. ordinary tests use job words more often than phase words
4. specialist words still exist, but are clearly contained

If the bridge still sounds like several milestones talking at once, the wording
work is not done.
