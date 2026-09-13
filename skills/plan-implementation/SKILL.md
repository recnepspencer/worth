---
name: plan-implementation
description: Create an implementation plan for a specified WORTH milestone phase or implementation slice. Use when the user wants a concrete, architecture-grounded plan before coding begins.
---

# Plan Implementation

Create an implementation plan for the requested phase or slice.

For each phase, prioritize the smallest working end-to-end production journey
that can provide meaningful feedback. Reach that MVP checkpoint as soon as
reasonably possible, then expand and harden the same journey against the full
acceptance requirements.

Do not implement the plan or edit files during this turn.

As you plan:

- Follow the repository's engineering mentality and its architectural,
  performance, composition, domain-structure, testing, and DX laws.
- Review the relevant context, including the governing specification, current
  implementation, relevant public APIs, tests, upstream authorities, and
  downstream consumers. Reuse established findings and investigate unresolved
  boundaries needed for the journey rather than repeating a broad review.
- Identify the adversarial constraint the work must survive.
- Plan the appropriate causal scope. Include required foundations,
  integrations, cutovers, and proof work even when they cross files, crates, or
  phases.
- Plan the destination directory and module skeleton explicitly. State what
  each proposed file or module owns.
- Plan the intended DX as an actual code-block target whenever the work affects
  a caller-facing surface.
- Make implicit requirements, authority boundaries, failure behavior,
  lifecycle obligations, and performance expectations explicit.
- Identify existing paths that must be cut over, removed, or made insufficient.
- Include the phase-relevant tests, structural counters, compiler boundaries,
  and verification needed to prove the result.
- Build the complete plan inline in the chat. Do not create a separate plan
  file unless the user asks for one.

Plan for low total construction cost across the current slice and committed
growth. Prevent duplicated authority or meaning, but do not abstract repeated
mechanics unless the organization makes expected additions, replacement,
testing, or scaling easier. Show where known successors enter and which existing
boundaries absorb growth without unrelated edits or ownership changes.

Write one coherent implementation path rather than a loose collection of
possibilities.

Start the plan with three concrete things:

- The first working journey: its real input, production path, observable result,
  and focused test or demonstration that will provide feedback.
- Its necessary prerequisites: which specific step each prerequisite blocks.
  Schedule each fix immediately before completing the behavior it enables.
- The remaining completion work: required scenarios, failure handling, lifecycle
  coverage, and performance evidence beyond the first working journey.

The MVP may narrow scenario coverage, polish, and optimization, but it must use
the real production owners and authority boundaries. A disconnected fixture,
stubbed internal integration, or isolated passing test cannot establish that the
journey works. Include focused verification with the first working version;
preserve every acceptance requirement for phase closure.

Use feedback from that journey and unmet acceptance requirements to order later
work. Defer general cleanup, speculative abstractions, and optimization unless a
concrete obstruction or governing requirement makes them necessary. Keep the
working journey as the integration checkpoint while expanding and hardening it.

Respect dependency and authority within this sequence. For each step, explain:

- what the step requires
- what must change
- why that change belongs there
- what later work depends on it
- how the step will be proven complete

Do not produce shallow bullet points that leave implementation to rediscover
the architecture. Make the plan explicit enough for implementation to follow
directly while remaining proportional to the work.
