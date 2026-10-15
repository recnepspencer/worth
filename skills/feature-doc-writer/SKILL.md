---
name: feature-doc-writer
description: Write or revise product-facing feature documentation for a real implementation surface. Use when creating docs for framework features, runtime capabilities, APIs, or developer-facing concepts where the goal is usage guidance and mental-model clarity rather than milestone planning or engineering-spec prose.
---

# Feature Doc Writer

Use this skill when the deliverable is product documentation for a feature that
already exists or is being stabilized.

Do not use this skill for milestone specs, roadmap edits, or implementation
closeouts. Those belong to `spec-designer` or the local engineering docs.

## What Good Docs Must Do

A good feature doc lets a capable engineer or AI:

- identify the right surface to use
- understand the feature's mental model
- combine it with adjacent features correctly
- avoid authority and lifecycle mistakes
- learn the stable boundary without spelunking tests or milestone history

If using the feature correctly still requires reading certification harnesses,
closeout docs, or lower-runtime internals, the documentation is incomplete.

## External Patterns Worth Copying

Use these patterns deliberately:

- Angular: concept-first mental model, small examples early, explicit "when not
  to use this"
- Laravel: practical examples, caveats near the feature, real code over
  abstract theory
- OpenAI docs: task-first framing, clear capability boundaries, direct request
  or call shape near the top
- Next.js: convention and constraint clarity, "good to know" callouts, honest
  defaults and conflict notes

Avoid these traps:

- giant megadocs that hide the feature in endless options
- API signatures without a real mental model
- concept pages that are too thin to stand on their own
- milestone-history storytelling in place of usage guidance
- parameter tables that explain syntax but not system behavior

## Required Workflow

1. Identify the feature boundary exactly.
2. Read the public surface that actually exposes it.
3. Read the support/admission posture if the feature can be deferred, gated, or
   vocabulary-only.
4. Read the strongest relevant tests to see what behavior is real.
5. Write the doc around user intent, not around internal module names.
6. Include one small example and one realistic example.
7. Name anti-patterns and non-goals explicitly.
8. QA the doc against code, tests, and support posture before finishing.

## Mandatory Source Set

Before writing, read:

- the public facade or API entry points for the feature
- the strongest focused tests for the feature
- any support matrix / admission / closeout boundary that changes whether the
  feature is stable, deferred, or unsupported

Read milestone or roadmap docs only when you need future-boundary context.

## Required Document Shape

Every feature doc should use this shape unless the local docs system already
has a stronger convention:

1. `What This Feature Is`
2. `Why You Use It`
3. `Stable Entry Points`
4. `Core Mental Model`
5. `How It Executes`
6. `Small Example`
7. `Real Example`
8. `How It Relates To Other Features`
9. `Inspection And Debugging`
10. `Anti-Patterns`
11. `Current Limits`
12. `Related Docs`

Use the full template in [references/doc-template.md](references/doc-template.md).

## WORTH-Specific Rules

- Preserve crate authority boundaries. Never let docs imply that a facade owns
  semantics that belong to a lower runtime.
- Distinguish stable, deferred, unsupported, and vocabulary-only surfaces.
- Prefer the stabilized public facade over lower-runtime plumbing.
- Explain authority lanes, retained state, preview/branch semantics, and
  inspection boundaries when they are part of correct usage.
- Do not mix engineering-spec language with feature guidance. The reader should
  leave knowing how to use the feature, not how the milestone was sequenced.

## Style Rules

- Lead with the concept and the code the reader writes.
- Keep prose concrete and calm.
- Put caveats near the relevant example instead of dumping them all at the end.
- Use callouts sparingly for "good to know", "deferred", or "do not use this
  for..." moments.
- Prefer worked examples that touch adjacent real features.

## QA Checklist

Run the checklist in [references/qa-checklist.md](references/qa-checklist.md)
before considering the doc done.

## Source-derived consumer guidance

In WORTH, the capability index owns the consumer prohibition list. Other guides
link to that list and keep their local lifecycle and refusal explanations.
Generated `source-facts` regions come from owning declarations: edit the owner
and regenerate with boundary-check `--write-guidance`, then check the diff.
Do not hand-maintain a second variant list or claim an unresolved API is absent.
A `known-gap` reference needs its checked canonical absence claim; Unsupported
source knowledge is not an absence proof. Keep new native API claims pending
until their owner and compiled caller are accepted. Required executable examples
must bind to real compiled/CI-run code, not be relabeled illustrative.

Repository `skills/` is authoritative. `.grok/skills` is generated from it by the
same explicit guidance command and is checked for drift. Installed user skills
are outside that generated zone and must not be overwritten by repository sync.

A checked assertion is a standalone `<!-- facade-absence: gap-id -->` in the
capability index followed by the unique heading `### Gap: gap-id`. A standalone
`<!-- known-gap: gap-id -->` reference is followed by a Markdown link to that
index's `#gap-gap-id` heading. Both require an explicit supported source selector;
markers do not turn incomplete knowledge into proof.

Guidance writes require a quiescent local checkout. Only configured consumer
Markdown regions and owned derived skill files are writable. Links, Windows
reparse points, changed destinations and unsupported file kinds refuse. Complete
files publish by same-directory rename; this is not a hostile-filesystem sandbox
or a claim of race-safe confinement against concurrent directory replacement.
The generated notice lists individual owned paths: an unknown extra refuses,
while only a previously listed generated path may be removed as stale.
