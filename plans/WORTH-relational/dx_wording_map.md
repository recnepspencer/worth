# WORTH Relational DX Wording Map

## Purpose

This document maps the current Relational vocabulary to a more usable guided
vocabulary without lying about the architecture.

Rule:

- guided docs and guided entrypoints should sound direct, casual, and obvious
- raw formal surfaces can stay formal where exactness actually matters

This is not about sanding the runtime down into baby talk.

It is about stopping the public story from sounding like internal subsystem
bookkeeping.

---

## Core Tone Rule

Prefer:

- direct job words
- verbs that say what happens
- names an AI agent can pick correctly on first contact

Avoid:

- academic framing
- architecture-first nouns as the first thing users see
- names that require insider context before they make sense

Bad vibe:

- "computation spec"

Good vibe:

- "recipe"

That same rule should govern Relational.

---

## Preferred Guided Wording

| Formal / Current | Guided / Preferred | Scope |
| --- | --- | --- |
| runtime construction | build runtime | docs / examples / guided APIs |
| mutation | write truth / make changes | docs / examples |
| committed truth | truth now / current truth | docs / examples |
| visibility reads | read truth | docs / guided naming direction |
| inspection | inspect what happened | docs / guided naming direction |
| diagnostics | what is wrong / diagnostics | docs |
| publication | what published / published output | docs |
| invariant validation | validation / truth checks | docs / guided naming direction |
| certification | certify current state / prove current state | docs |
| simulation | compiled artifacts | docs / guided naming direction |
| retention authority | retention | docs / guided naming direction |
| durability recovery | recover runtime / recovery | docs |
| historical reads | history | docs / guided entrypoint language |
| replay | replay / verify past commits | docs |
| commit strategy | commit strategy / commit pipeline | docs |
| batch mutation | batch change / write batch | docs / examples |
| payload policy / symbol policy / publication policy | setup knobs / policy knobs | docs |
| schema registry | schema | docs / examples when the longer formal term is noise |

---

## Canonical Guided Phrases

These are the phrases the docs should keep repeating until the product memory
gets sticky:

- build runtime
- write truth
- read truth
- inspect what happened
- inspect what published
- inspect what is wrong
- go to history
- go to replay
- go to merge
- go to validation
- go to compiled artifacts
- go to retention
- recover runtime

If a doc section title cannot be rewritten in that style, it is probably still
too internal.

---

## Surface-Level Naming Direction

These are not all mandatory immediate renames.

They are the wording direction the product should converge toward.

| Current Surface | Preferred Product Direction | Notes |
| --- | --- | --- |
| `RelationalRuntimeApi::builder()` | keep | already good |
| `visibility_reads()` | `read_truth()` | current implementation seam is fine for now, but the product story should teach `read_truth` |
| `inspection_access()` | `inspect_what_happened()` or inspection lane | method can stay for now; docs should teach the job first |
| `publication_access()` | publication lane / inspect what published | docs should lead with the job |
| `invariant_access()` | `validation()` | contained lane name should say the job |
| `simulation_access()` / `simulation_authority()` | `compiled_artifacts()` | "simulation" is weaker than the actual job |
| `retention_authority()` | `retention()` | shorter and more honest |
| `durability_access()` / `durability_authority()` | durability lane / recovery lane | docs should lead with recovery and checkpoint jobs |

---

## Wording Rules By Topic

## Setup

Use:

- build runtime
- runtime setup
- setup knobs
- runtime profile

Avoid leading with:

- configuration topology
- runtime assembly
- builder refinement graph

## Write Path

Use:

- write truth
- start a transaction
- push a batch
- commit changes

Avoid leading with:

- mutation admission pipeline
- provenance-complete batch admission
- naming-stable mutation shaping

Those deeper phrases can still appear in advanced reference material.

They just should not be the first story.

## Read Path

Use:

- read truth
- current truth
- query bigger slices
- use history for past truth

Avoid leading with:

- visibility materialization semantics
- storage-visible read fallback

Those are real architectural truths, but bad first-contact wording.

## Inspection And Diagnostics

Use:

- inspect what happened
- inspect what published
- inspect what is wrong
- inspect what is retained

Avoid leading with:

- artifact families
- diagnostic materialization buckets
- observer helper taxonomy

## Specialist Lanes

Use:

- merge
- replay
- recovery
- validation
- compiled artifacts
- retention

Avoid euphemisms that make the lane fuzzier than it is.

Specialist is fine.

We are not trying to hide power.

---

## Naming Guardrails

1. Do not rename precise core concepts just to sound friendlier.
2. Do not make guided wording contradict the architecture.
3. Do not hide specialist power behind vague words like "advanced tools."
4. Do not teach internal decomposition before teaching the user job.
5. Do use the guided words repeatedly in docs, examples, and overview material.

---

## Immediate Phase 4 Rewrite Targets

These are the places where wording should change first:

- crate landing docs should say "authoritative truth runtime"
- quickstart docs should say "build runtime", "write truth", and "read truth"
- operator docs should say "inspect what happened", "what published", and
  "what is wrong"
- advanced docs should say "history", "replay", "merge", "validation",
  "compiled artifacts", "retention", and "recovery"

---

## Bottom Line

The public story should sound like somebody trying to help you use the runtime,
not like the runtime explaining its own organs.
