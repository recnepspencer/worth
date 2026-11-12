---
name: cad-qa-loop
description: Review and correct WORTH CAD kernel behavior against CAD meaning, engineering laws and the public platform contract. Use for kernel implementation QA; public platform implementations use qa-loop.
---

# CAD QA Loop

Review the final slice against its governing specification, repository
instructions, CAD laws and relevant numerical, testing and iteration laws.
In worth-proprietary, start with AGENTS.md, REVIEW.md, AI_README.md and
cad/docs/cad-laws.md. Resolve current documents from that checkout; inspect
supersession notices rather than substituting a different worktree's contracts.

## The kernel does not reestablish trust

Public WORTH owns snapshot consistency, dependency tracking, currentness,
invalidation, safe publication and recovery guarantees. The kernel owns CAD
meaning: geometry, topology rules and domain invariants declared through the
platform.

Consume a successful public operation according to its declared contract.
Reject private reconstruction or revalidation of those guarantees, including
inside proprietary tests and reference oracles. Investigate an apparent public
failure at its supported boundary; fix the public contract or implementation
in public WORTH. Do not prescribe a private fallback or parallel infrastructure.

Review CAD invariants at the owner that constructs or admits their facts.
Successful publication must not be followed by a newly discoverable domain
refusal during Session adoption. Distinguish CAD meaning from platform safety;
moving a public recheck before the seal does not make it a CAD invariant.

## Trace CAD consequences

Follow authored inputs through their numerical owner, units, frames, parameter
domains, tolerance provenance, geometry, topology and affected consumers.
Shared arithmetic and numerical primitives stay at their established owners.
Do not promote tessellation, display approximations or legacy proxies into
authoritative geometry.

Inspect the real affected authoring transitions: create, edit, regenerate,
correct, delete or undo as relevant. Consider failed preparation, cancellation
and adoption where the change affects them. Test the kernel's use and CAD
consequences of the public contract without independently reproving the public
transaction or recovery machinery.

Scope follows causal necessity. Inspect the owners, consumers and tests needed
to judge the slice; preserve unrelated work and report unrelated debt separately.
A test-only audit path is not a production performance path. Trace callers
before assigning a performance defect or milestone blocker.

## Correct and close

Use plausible inputs and concrete failure sequences. Each material finding
names its location, trigger, consequence, evidence, owning contract or law,
correction and distinguishing verification. Separate observations from inference.

Restore the invariant at its rightful owner and remove displaced competing
mechanisms. Do not preserve speculative work budgets, reservation ledgers or
resource estimates in ordinary CAD APIs without an explicit product requirement
or a measured failure requiring that remedy. Preserve checked arithmetic,
semantic counts, authority, CAD validity and ordinary failure handling.

Apply cad-qa-tests and the shared code-quality-qa to the same final slice.
Do not also run generic qa-loop and qa-tests for the same kernel responsibility.
Follow the authorized independent reviewer arrangement; a read-only reviewer
reports findings and the implementer makes corrections.

Stop when no supported material in-scope finding remains and required focused
verification is complete. Report corrections, actual checks, unrun milestone
or nightly checks and limitations. Do not add rounds without new evidence.

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
