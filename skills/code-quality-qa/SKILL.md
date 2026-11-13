---
name: code-quality-qa
description: "Review and correct the structure of a change: ownership,
  placement, naming, module boundaries and size. Use after behaviour and
  test QA, on platform or CAD work."
---

# Code Quality QA

Review whether the structure of this change matches its meaning. "No
material findings" is a fully successful outcome. Structure serves the code
that exists and the successors the specification names, nothing further.

## Scope

The files this task changed, plus the parents, facades and consumers needed
to judge them. A violation this task introduced or touched is a finding. A
violation that was already there is reported as debt and left alone.

Run `python scripts/quality/scrutinize_rust_functions.py --dirty .` where it
exists and inspect the candidates inside this scope.

## What counts as a finding

A finding names the rule broken and one concrete cost: an edit that would
have to touch unrelated places, or something a reader would get wrong. With
no concrete cost it is taste, and it is not a finding. Unfamiliar or complex
code is not a defect.

## Rules

- A module, file or type owns one idea, and its name says which. A name that
  hides what is inside, or promises more than is there, is a finding.
- Logic lives in the narrowest place that owns it. Stable meaning does not
  depend on volatile mechanism.
- A facade re-exports. It does not implement or decide.
- An orchestrating function reads as named steps. Decisions, effects and
  failure handling are not buried in a body that mixes levels.
- A responsibility can be added or removed in one obvious place.
- Files stay within 400 lines. A function over 60 lines or with five or
  more arguments gets a look; size alone is not a defect. Do not split a
  file into fragments just to meet the number.
- A change does not widen the compile or link closure of normal edits:
  no heavy dependency, generic instantiation or test world pulled into a
  crate that did not need it.

## Remove

- A trait, interface or generic with one implementation and no named
  successor.
- A layer that only forwards.
- Configuration nobody sets, and extension points for unnamed future work.
- Resource bookkeeping in ordinary domain APIs (counters, budgets, quotes,
  ledgers) without a stated requirement or a measured failure. Moving or
  renaming it does not justify it.

## Correcting

- Correct by moving, renaming, merging or deleting. Do not add structure to
  fix structure.
- Corrections do not change behaviour and do not go beyond the findings.
- Build once, after the structure has settled, at the smallest meaningful
  scope.

## Report

Findings with their rule and cost, corrections made, debt left alone,
commands actually run.
