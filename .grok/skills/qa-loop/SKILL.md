---
name: qa-loop
description: Review and correct a WORTH implementation against its intent, specification, engineering laws, and credible evidence.
---

# QA Loop

Read the repository instructions, governing specification, and `docs/coding-guidelines/arch_laws.md`, `docs/coding-guidelines/perf_laws.md`, and `docs/coding-guidelines/MENTALITY.md` before reviewing. Inspect the implementation, affected owners, consumers, and tests. Scope follows causal necessity: include boundaries and prerequisites needed to make the requested behavior correct, preserve unrelated work, and report unrelated debt separately.

Check consequential implementation claims using plausible inputs, boundary conditions, and execution sequences. Build a compact risk matrix connecting claims and invariants to failure cases and distinguishing evidence. Prioritize authority, correctness, recovery, and performance risks, including interactions between them. Reuse adequate evidence and investigate concrete uncertainty; avoid exhaustive combinations, speculative concerns, and repeated review without new evidence.

## Budget review priority

Budget scrutiny must remain a small minority of ordinary QA and repair time. Make it a focus only when diagnosing a demonstrated performance/resource issue or investigating a concrete concerning resource risk, such as runaway work, excessive allocation, or an ineffective limit. Otherwise, budget findings should be rare, including advisories. Do not raise findings merely because accounting could be finer, counters could be richer, or an exhaustion matrix could be broader; do not pursue speculative budget refinements or repeatedly reopen adequately bounded paths.

Preserve existing safeguards and run explicitly required checks without turning them into a separate budget-hardening project. When a concerning defect warrants work, state its concrete consequence, fix the responsible boundary, and use focused evidence sufficient to close it. Reuse that evidence and return to the requested product behavior; budget accounting and instrumentation must not displace functional correctness, integration, or user-experience progress.

## Findings and closure

Each finding must state the defect and consequence, concrete evidence, root cause, proposed correction, and required verification. Pin it to an exact law or numbered principle in `arch_laws.md`, `perf_laws.md`, or `MENTALITY.md`, explaining the violation. Use **N/A** when none applies and name the violated specification, contract, or expected behavior; these findings carry equal weight. Close findings by restoring the underlying invariant at its owning boundary. Use typed progression, carried authority, or required infrastructure when the governing laws call for them. A guard, special case, or green reproducer does not close a surviving root cause. Complete causally necessary foundations; difficulty or a larger diff alone does not justify deferral.

Use the reviewer arrangement selected by the user or repository. Run focused verification during correction and required gates before completion. Reviewers must inspect corrections and affected evidence before clearance. Escalate genuine specification contradictions, undiscoverable product choices, and actions beyond granted authority with evidence and options; resolve discoverable uncertainty through investigation. Broaden review only when new evidence or a correction invalidates an earlier conclusion. Stop when required verification and review are complete and no supported, material in-scope finding remains. Report findings, corrections, evidence, and limitations; do not add rounds merely to discover more issues.
