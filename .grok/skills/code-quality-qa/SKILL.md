---
name: code-quality-qa
description: Review and correct WORTH code composition and domain topology within the current task's causal scope. Use when auditing responsibility, placement, naming, facades, module boundaries, future insertion, or repository size and function advisories without absorbing unrelated dirty-worktree debt.
---

# Code Quality QA

Review whether the change's physical structure preserves its meaning. Judge
against `composition_laws.md`, `domain_structure_laws.md`, the governing
specification, and the Composition and Architecture sections of
`docs/coding-guidelines/qa_review_guide.md`.

## Establish task scope

Identify files changed by the current task and the directly affected parent,
child, facade, consumer, and dependency modules needed to judge ownership.
Distinguish that scope from unrelated pre-existing staged, unstaged, renamed, or
untracked user work. Do not expand the task merely because the worktree is large
or dirty.

Run repository composition, function, and line-cap tools required by local
instructions. Classify their findings by causal scope. A structural violation
introduced, modified, or made relevant by the current task blocks completion;
an unrelated pre-existing violation is reported as repository debt.

In WORTH Rust worktrees, use
`python scripts/quality/scrutinize_rust_functions.py --dirty .` when available,
then inspect every reported candidate that belongs to the task's causal scope.
Apply the 400-line limit as required by repository rules. Functions over 60
lines or with five or more explicit arguments require judgment but are not
defects by size alone.

## Independent review

Use the structural reviewer selected by the user or repository. Do not hard-code
a model or require a new instance after every edit. Give the reviewer the
relevant laws, specification, current diff, task-scoped file inventory,
destination topology, and enforcement results. A persistent reviewer must
inspect the final diff before clearing the change.

Verify reviewer findings directly. If required outside review is unavailable,
report that limitation without treating the primary pass as independent.

## Review lenses

- **Destination topology:** Expected successors enter without authority
  reversal, bucket growth, or unrelated relocation.
- **Semantic responsibility:** Each directory, file, type, and function owns a
  predictable idea at its visibility radius.
- **Composition:** Orchestrators read as named semantic steps; decisions,
  effects, and failure paths are not hidden in mixed-level bodies.
- **Placement:** Logic lives at the narrowest honest semantic radius, and stable
  meaning owns volatile mechanisms.
- **Naming:** Names predict contents, exclusions, authority, phase, and truth
  status without archaeology.
- **Boundary honesty:** Facades aggregate rather than implement, and dependency
  direction preserves authority.
- **Deletion and extension:** Adding or removing a responsibility has an obvious
  location and does not require unrelated edits.
- **Size discipline:** Task-scoped files satisfy repository limits and are not
  split into dishonest fragments merely to satisfy a number.

Keep the pass structural. Follow semantic, performance, or test defects only
when structure conceals or causes them.

## Correct and complete

Correct responsibility and topology rather than shuffling lines. Search the
affected semantic family for the same structural defect and verify compilation
and required repository enforcement after the structure stabilizes.

Review is complete when the task-scoped structure is coherent, required limits
and checks pass, the assigned reviewer has inspected the final diff, and no
known in-scope structural defect remains. Report unrelated repository debt as a
caveat rather than silently absorbing or fixing it.
