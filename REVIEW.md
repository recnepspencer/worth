# Review instructions

Review the final change against the governing specification and repository
laws. Inspect affected owners, consumers and tests; the implementer's report
is a set of claims to verify.

Prioritize correctness, authority, architectural ownership, credible tests and
demonstrated performance or iteration costs. Report material defects introduced
or causally affected by the change, with location, triggering scenario,
consequence and evidence. Distinguish observation, inference and unverified
claims. Keep unrelated pre-existing debt separate.

Architectural ownership violations are substantive findings even when today's
tests pass. Avoid cosmetic preferences, speculative hardening and duplicate
reports of mechanical CI failures. Follow the review host's output format.

Local slice reviewers read the actual skills named below and apply them to the
same final diff. Three lenses do not require three reviewers or builds.
Hosted code review does not execute skills or commands; apply the substantive
review rules here and do not claim execution that did not occur.

Use [qa-tests](skills/qa-tests/SKILL.md) for test failures and test-quality
review during production changes. Leave unaffected tests alone; simplify or
replace an affected fixture when it obstructs the change or makes certification
fragile.

Use [test-suite-simplify](skills/test-suite-simplify/SKILL.md) when asked to
reduce the cost or fragility of an existing suite, fixture, snapshot or
verification step. This is a separate simplification workflow, not an additional
mandatory pass during production changes. Review removed test families against
the guarantees preserved and their replacements, or why no current guarantee
remained. Judge success by reduced recurring certification work.

## Necessity and scope

Judge whether the work is needed before judging how thoroughly it is built.

- Identify the user outcome, real consumer or explicit platform promise.
  An existing test or specification checkbox does not justify itself.
- For each proposed expansion, name the concrete failure, a credible trigger,
  and why existing facilities cannot satisfy the requirement.
- Prefer removing an unnecessary requirement or reusing an existing owner
  over adding another mechanism. Preserve required correctness and authority.
- Do not request speculative flexibility, unrelated cleanup, additional
  validation or broader testing without a concrete in-scope consequence.
- Separate necessary corrections from optional follow-ups. A suggestion
  does not become a landing blocker merely because a reviewer proposed it.
- "No material in-scope findings" is a fully successful outcome. Review is an
  assessment, not a requirement to produce corrections. Do not manufacture
  improvements or suppress a real defect to return a clean verdict.
- Stop when the required behavior and proportionate evidence are complete.

For each blocker, explain the consequence and smallest coherent correction.
If necessity is uncertain, identify the missing fact before prescribing work.

Would an experienced engineer choose this design to solve the actual
requirement if no AI-generated plan, review checklist or inherited test
demanded it? Prefer established facilities, clear ownership and proportionate
complexity. Additional mechanisms must earn their implementation, maintenance
and iteration cost through a concrete need. Judge the actual consumer and
tradeoff, not a company's reputation or an "enterprise grade" label. Unfamiliar
or complex code is not itself a defect; explain what is unnecessary and why.

## Required platform review skills

Read skills/qa-loop/SKILL.md, skills/qa-tests/SKILL.md and
skills/code-quality-qa/SKILL.md. Use implementation-batch for new implementation
slices. CAD kernel reviews use cad-qa-loop and cad-qa-tests instead of the generic
pair; code-quality-qa remains shared.

## Platform ownership

Public WORTH owns snapshot consistency, dependency tracking, currentness,
invalidation, safe publication and recovery guarantees. Apply the governing
laws in docs/coding-guidelines/.

Flag changes that weaken these contracts or require consumers to reconstruct
their guarantees. Missing capabilities are implemented at their public owner,
through supported consumer interfaces. Check docs/capability-index.md before
accepting a second implementation of an existing responsibility. New consumer
capabilities need a clear contract, appropriate facade and index entry.

## Authority and lifecycle

Authority must come from its issuing owner and survive the relevant boundaries.
Flag forged, unchecked or test-only authority paths. Ordinary operations must
not accidentally depend on replay, reconstruction or certification machinery.

Check refusals and partial effects at the owning boundary. Public tests
establish public guarantees with credible fixtures and independent observations;
consumers should not have to certify these guarantees again.

## Evidence and iteration

Inspect dependencies, generic instantiations, test worlds, harnesses and crate
placement for normal edit cost. Distinguish ordinary, cold, recovery and
certification paths in performance findings.

Mandatory resource accounting needs an explicit product requirement or a
measured failure requiring that remedy; hypothetical exhaustion is insufficient.

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
