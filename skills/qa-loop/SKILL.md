---
name: qa-loop
description: Review and correct a WORTH platform implementation against its
  governing specification or plan. Not for CAD kernel work.
---

# QA Loop

The governing specification or plan is the only authority for behaviour. Do
not consult other documents to find requirements. "No material findings" is
a fully successful outcome.

## Preproduction

WORTH is not in production. Do not preserve obsolete APIs, stored formats,
fixtures or golden bytes for backward compatibility. Within the change's
scope, delete displaced implementations, adapters, migrations and
compatibility-only tests; use one current representation. Recreate development
artifacts instead of migrating them. Preserve required current-version replay,
undo and recovery behavior. Add backward compatibility only when the user
explicitly requests it.

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
