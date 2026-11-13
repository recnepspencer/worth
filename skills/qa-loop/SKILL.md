---
name: qa-loop
description: Review and correct a WORTH implementation against its intent, specification, engineering laws, and credible evidence.
---

# QA Loop

For CAD kernel implementations, use cad-qa-loop and cad-qa-tests with the shared
code-quality-qa instead of this generic QA pair. Public platform work uses this
skill with qa-tests and code-quality-qa.

Read the repository instructions, governing specification, and `docs/coding-guidelines/arch_laws.md`, `docs/coding-guidelines/perf_laws.md`, and `docs/coding-guidelines/MENTALITY.md` before reviewing. Inspect the implementation, affected owners, consumers, and tests. Scope follows causal necessity: include boundaries and prerequisites needed to make the requested behavior correct, preserve unrelated work, and report unrelated debt separately.

Check consequential implementation claims using plausible inputs, boundary conditions, and execution sequences. Build a compact risk matrix connecting claims and invariants to failure cases and distinguishing evidence. Prioritize authority, correctness, recovery, and performance risks, including interactions between them. Reuse adequate evidence and investigate concrete uncertainty; avoid exhaustive combinations, speculative concerns, and repeated review without new evidence.

## Review purpose and scope

Read the task repository's REVIEW.md. Review is an assessment, not a requirement
to produce corrections. "No material in-scope findings" is a fully successful
outcome. Report supported defects; do not manufacture improvements to
demonstrate thoroughness or suppress a real defect to return a clean verdict.

A blocking finding must identify the affected requirement, credible trigger,
concrete consequence and smallest coherent correction. Optional improvements
do not block completion. Follow additional owners only when needed to establish
or correct that consequence; stop when the assigned review is complete.

## Proportionate engineering

Would an experienced engineer choose this design to solve the actual
requirement if no AI-generated plan, review checklist or inherited test
demanded it? Prefer established facilities, clear ownership and proportionate
complexity. Additional mechanisms must earn their implementation, maintenance
and iteration cost through a concrete need.

Judge the actual responsibility, consumer and tradeoff rather than a company's
reputation or an "enterprise grade" label. Unfamiliar or complex code is not
itself a defect. Explain what is unnecessary and why; do not turn this question
into speculative redesign, cosmetic cleanup or another mandatory framework.

## Remove speculative resource accounting

Ordinary modeling and publication must not require work counters, candidate
budgets, retained-byte estimates, mutation quotes, reservation ledgers, or
per-publisher accounting of fields, links, copies, and allocations. Reject new
mandatory resource bookkeeping and remove existing bookkeeping in the affected
path when it has no demonstrated product need. Moving it into a typed carrier,
shared calculator, or framework wrapper does not justify keeping it. Replacing
an arbitrary ceiling with exact pre-accounting can preserve the same unnecessary
obstacle.

Resource accounting is a low-priority optimization, not a correctness foundation
or a prerequisite for completing an authoring milestone. Prioritize working UI
authoring, correct and local invalidation, reuse, partitioning, and useful
parallelism. Retain or introduce resource admission only for an explicit product
requirement or a measured resource failure whose correction needs it. Name the
requirement or reproducer and explain why a simpler remedy is insufficient;
hypothetical exhaustion, generic adversarial pressure, and more complete
accounting are not findings or reasons to expand scope.

Preserve geometry/topology validity, mutation authority, currentness, atomic
publication, cancellation, checked size arithmetic, and ordinary allocation
failure handling. These guarantees do not imply pre-budgeting. Semantic counts
such as cardinality, revisions, and algorithm termination are not resource
budgets merely because they use integers. Judge their actual purpose.

An existing safeguard, test, or specification clause is not by itself evidence
that accounting is worth retaining. When the user directs its removal, update
causally affected contracts and tests coherently; do not preserve it under a new
name or launch a replacement accounting framework. Keep unrelated work and
accepted functional evidence. Close the smallest coherent removal and return
to product behavior; do not turn this instruction into a repository-wide purge.

## Findings and closure

Each finding must state the defect and consequence, concrete evidence, root cause, proposed correction, and required verification. Pin it to an exact law or numbered principle in `arch_laws.md`, `perf_laws.md`, or `MENTALITY.md`, explaining the violation. Use **N/A** when none applies and name the violated specification, contract, or expected behavior; these findings carry equal weight. Close findings by restoring the underlying invariant at its owning boundary. Use typed progression, carried authority, or required infrastructure when the governing laws call for them. A guard, special case, or green reproducer does not close a surviving root cause. Complete causally necessary foundations; difficulty or a larger diff alone does not justify deferral.

Use the reviewer arrangement selected by the user or repository. Run focused
verification during correction and inspect the final diff and affected evidence
before clearance. Required cheap landing checks remain required; expensive
gates are milestone, release or nightly work under the policy below. Escalate
genuine specification contradictions, undiscoverable product choices and actions
beyond granted authority with evidence and options. Investigate discoverable
uncertainty. Broaden review only when new evidence or a correction invalidates
an earlier conclusion. Stop when focused verification and review are complete
and no supported material in-scope finding remains. Report findings, corrections,
actual evidence, deferred checks and limitations; do not add rounds merely to
discover more issues.

## Compile time is sacred

During ordinary development, slice review and routine landing, use cheap source
checks and the smallest meaningful compilation and test scope. Include compile,
link and fixture construction cost when selecting evidence; test runtime alone
does not describe its cost.

Expensive gates belong at milestone completion, release verification or nightly
jobs. They are not prerequisites for every edit, review, commit or slice landing.
Do not expand automatically to all reverse dependencies or workspace CI.

A concrete suspected defect may require an expensive focused reproducer. Name
the uncertainty and why cheaper evidence cannot resolve it; this does not
authorize a full gate. Do not invent a scheduling job by invoking this policy.

Reuse valid evidence and rerun checks when a correction affects what they
establish. Three QA lenses do not imply three builds, test runs or reviewers.
Report deferred verification honestly; a focused pass is not a full gate pass.
