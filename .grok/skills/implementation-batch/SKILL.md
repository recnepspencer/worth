---
name: implementation-batch
description: Continue a milestone by reviewing the next unfinished slice's real authority and integration boundaries, planning from that boundary review, implementing a coherent vertical slice, and verifying the milestone evidence. Use when Codex is asked to implement the next phase, batch, or unfinished milestone work rather than only review or plan it.
---

# Implementation Batch

Complete one honest milestone batch through four ordered stages:

1. select the next unfinished slice
2. review its boundaries
3. plan from the review
4. implement and verify the plan

Do not edit code before the boundary review and implementation plan are both
complete. Perform all four stages in the same turn unless the user asks to stop
at a checkpoint.

## Select the Slice

Start from the governing milestone or specification. Read the repository's
local instructions and the architecture, performance, composition, domain
structure, and DX guidance relevant to the slice when those documents exist.

Determine the next unfinished slice by comparing:

- declared phase order and dependencies
- required behavior and acceptance evidence
- current production code and public APIs
- current tests, fixtures, and machine-checkable evidence

Do not infer completion from names, stubs, checked boxes, or passing tests alone.
Do not skip earlier unfinished work merely because a later slice is easier.

Choose a coherent, proof-bearing vertical slice. Include adjacent work when it
is required to make the claimed behavior and authority boundary honest. Avoid
both cosmetic micro-patches and unrelated cleanup. If the milestone is nearly
closed and no meaningful implementation batch remains, perform a closeout-
readiness pass and report that fact instead of inventing work.

## Stage 1: Boundary Review

Review before planning. Inspect the spec, scoped source, public entry points,
tests, and the upstream substrate and downstream consumers that participate in
the behavior. Follow the real data, control, identity, evidence, and authority
paths rather than trusting directory names or intended architecture.

Produce a concise boundary brief that identifies:

- the semantic truth entering the slice and its authoritative sources
- what this slice may own, create, validate, preserve, or deny
- what adjacent components continue to own
- weaker or proxy representations that must become insufficient
- existing paths, fallbacks, wrappers, or competing authorities that must be
  cut over or removed
- the downstream handoff that must consume the result
- failure modes at dirty edges, including mixed cutovers, copied truth,
  identity inferred from proxies, mutable-field leakage, synthetic proof, and
  tests that bypass production derivation
- unresolved facts that must be verified before implementation

Treat required substrate or consumer changes as part of the boundary problem;
do not dismiss them as outside the current directory. Include them when they
are necessary for the milestone claim, but do not absorb unrelated ownership.

Do not write the implementation plan until this brief is complete.

## Stage 2: Implementation Plan

Plan from the boundary brief and verified source evidence. Do not assume facts
that the review did not establish. Resolve material unknowns before editing.

Write a specific plan that includes:

- slice name and milestone obligations covered
- boundary findings that constrain the implementation
- the intended user-facing or developer-facing result, with a concrete code
  example when an API or DX surface changes
- directory and module shape, including each artifact's responsibility
- ordered implementation steps, each stating the change and what proves it
- cutover and deletion work needed to leave one ordinary authority path
- tests, compile-time fences, diagnostics, or other acceptance evidence
- focused and broader verification commands appropriate to the risk
- explicit out-of-scope work and blockers

Order the plan by dependency and authority: repair dirty boundaries and
competing authority before building behavior that would depend on them. Keep
the plan proportional to the slice; use judgment instead of forcing a fixed
number of files, layers, or tests.

## Stage 3: Implement

Implement the plan immediately.

- Follow local patterns and repository rules.
- Prefer the principled production path over adapters, compatibility shims,
  duplicated truth, or fixture-only proof.
- Make invalid states unrepresentable when the codebase supports a reasonable
  type or construction boundary.
- Complete the mechanical cutover before broad polishing or certification.
- Remove displaced ordinary paths when the plan requires a single authority.
- Update production behavior and its acceptance evidence together.
- Keep files within local size limits and split touched oversized files unless
  an explicit exemption applies.

If implementation reveals a boundary fact that invalidates the plan, pause
editing, revise the boundary brief and plan, then continue. Expand only to work
required for the slice's claim to be honest.

Build the structure that makes committed future work easier to add, test,
replace, and scale. Keep harmless mechanics local when sharing would add
cognitive load, but consolidate authority, policy, lifecycle, encoding, and
effect decisions that must evolve together. If the next known addition would
require duplication, bucket growth, or unrelated edits, establish its honest
boundary now.

## Stage 4: Verify and Close

Run formatting and the narrowest checks that prove the changed behavior. Add
broader package or workspace checks when shared APIs, authority, persistence,
identity, replay, protocol, or other cross-cutting surfaces changed. Run
compile-fail or equivalent negative checks when construction or type boundaries
changed.

Compare the result directly with every covered milestone obligation and the
boundary brief. Confirm that:

- the production path, not only a fixture, derives the claimed result
- old and new authority paths do not coexist accidentally
- upstream and downstream handoffs are integrated
- denial and failure behavior is covered where meaningful
- evidence is strong enough to expose a false implementation

Report:

- the boundary reviewed and slice built
- material files or artifacts changed
- cutover or competing-authority paths removed
- verification results and any unrun checks
- remaining milestone work and the best next QA target

Do not call the slice complete when required evidence is missing or a competing
ordinary authority remains.
