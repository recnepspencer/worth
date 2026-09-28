---

# Code cleanup pipeline contract

This runner pipeline is for architectural cleanup, proof-flow consolidation,
and domain-structure repair. It closes phases on structural evidence: clearer
ownership, better topology, narrower facades, named transitions, honest helper
placement, and mechanical proof where a boundary or behavior changes.

## Load before acting

Read the spec, the current phase scope, relevant public APIs, and project
context:

{project.context_files}

Also read the coding laws that govern cleanup work:

- `docs/coding-guidelines/MENTALITY.md`
- `docs/coding-guidelines/arch_laws.md`
- `docs/coding-guidelines/composition_laws.md`
- `docs/coding-guidelines/domain_structure_laws.md`
- `docs/coding-guidelines/perf_laws.md`

Read `docs/coding-guidelines/dx_laws.md` when public caller experience,
facades, examples, or ergonomic capability flow changes.

## Cleanup posture

Move the code toward this shape:

- directories reflect lifecycle, authority, and responsibility
- public facades teach the valid lifecycle order
- proof flows read as named transitions
- overloaded functions become auditable orchestration plus named semantic steps
- helpers live at the narrowest responsibility they serve
- certification/test support has clear authority status
- receipts and counters are tied to verified transition outcomes
- construction boundaries are mechanically guarded when construction order or
  authority matters

## Evidence posture

Choose evidence that matches the cleanup:

- directory skeletons and public API diffs for topology cleanup
- removed exports and visibility diffs for facade cleanup
- named classifiers, decision tables, and transition functions for proof-flow
  cleanup
- compile-fail coverage for construction and visibility boundaries
- runtime tests for changed behavior or preserved hostile behavior
- focused commands for touched crates/modules

Keep JSON payloads small. Put plans, findings, explanations, and verification
summaries in chat. The JSON is only progress state.

## Runner state

Use only the event requested by the prompt. If the runner cursor appears stale,
repair it from phase rows before continuing.

