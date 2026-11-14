---
name: cad-qa-loop
description: Review and correct a WORTH CAD kernel implementation against its
  governing specification or plan, with the CAD kernel addendum.
---

# CAD QA Loop

The governing specification or plan is the only authority for behaviour. Do
not consult other documents to find requirements. "No material findings" is
a fully successful outcome.

## Check in two directions

Missing: for each behaviour the specification states, read the code path
that implements it. Then trace what happens when it fails: invalid input,
an error partway through, an empty or maximal case, an out-of-order or
repeated call. Read the path; do not infer it from names, comments or
passing tests.

Excess: for each mechanism in the diff, find the requirement it serves. If
there is none, it is a finding: abstractions, configuration, counters,
budgets, defensive checks for states that cannot occur, and generality
nobody asked for. Resource accounting needs a stated requirement or a
measured failure.

## What counts as a finding

A finding names four things:

1. the specification clause affected;
2. a concrete trigger reachable through the public interface;
3. the consequence;
4. the smallest coherent correction.

Without a concrete trigger it is not a finding. Unfamiliar or complex code is
not a defect. Optional improvements are reported and do not block.

If the specification is silent on a failure the code can reach, report a
specification gap with the trigger. Do not invent the requirement.

## Correcting

- Fix the cause at the code that owns it. A guard, a special case or a green
  reproducer does not close a cause that survives.
- Scope follows causal necessity. Leave unrelated code alone and list
  unrelated debt separately.
- A correction that removes excess also removes its tests and contracts. Do
  not keep it under a new name.

## Stopping

- Make one pass. Review again only what a correction touched.
- Do not add rounds to look for more.
- Escalate a contradiction in the specification, a product choice that
  cannot be discovered, or an action beyond your authority, with evidence
  and options.

## Cost

- Use cheap source checks and the smallest meaningful build and test scope.
  Count compile, link and fixture construction.
- Full gates, broad matrices and reverse-dependency builds are milestone or
  nightly work, not a condition of clearing a review.
- Reuse valid evidence; rerun only what a correction affects.

## Report

Findings with their four parts, corrections made, commands actually run,
specification gaps, and deferred checks.

## CAD kernel

The kernel is a consumer of the platform. The platform owns snapshot
consistency, dependency tracking, currentness, invalidation, safe
publication and recovery.

- Kernel code that re-establishes any of those is excess: a private
  freshness check, dependency tracker, invalidation, publication validator
  or recovery certifier. Remove it.
- If a platform guarantee is missing or broken, report it against the
  platform. Do not compensate in the kernel.

When tracing failures, add three CAD cases: degenerate input, an unsupported
geometry combination, and a numerical result that cannot be decided. Each
must end in a typed outcome, never a guessed sign, a silent approximation or
a partial body.

Findings regardless of what the specification says:

- a private epsilon, decimal conversion, solver or precision escalation
  outside its numerical owner;
- an authored value passed through floating point;
- proximity used to weld, connect, merge or declare tangency;
- validation that moves, closes, flips or discards anything;
- a decision, stored value, name or hash that depends on hash iteration
  order, thread completion order or a library transcendental.
