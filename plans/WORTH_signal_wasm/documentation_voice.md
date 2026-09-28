# Worth Signals Documentation Voice

This is the editorial contract for public Worth Signals documentation. It is
not product documentation and does not belong in the public navigation.

## The Voice

Worth documentation should sound like a senior engineer patiently explaining a
system they care about.

The voice is:

- confident without pretending every trade-off disappeared;
- practical about what application code must do;
- skeptical of hidden magic and duplicated authority;
- warm enough to acknowledge why the tempting shortcut is tempting;
- occasionally wry, but never cute;
- exact about stable, compatibility, mixed, unavailable, and unsupported
  behavior.

Personality comes from judgment. It does not come from jokes, exclamation
marks, mascots, or marketing adjectives.

## What We Say Directly

Use "you" when addressing a developer. Use "we" only for a real product
decision.

Prefer:

> Do not mirror this value in React state. The copy creates a second owner and a
> synchronization problem that did not exist before.

Over:

> Consumers should avoid redundant state synchronization patterns.

Prefer:

> The runtime returns unavailable here because it cannot perform an exact
> replay. It does not assemble a convincing-looking approximation.

Over:

> Replay functionality may be unavailable depending on runtime posture.

The first form teaches. The second form protects the writer from making a
recommendation.

## Product Opinions

The documentation should consistently defend these positions:

1. Every value has one authority.
2. Derived state should be disposable and rebuildable.
3. Intent is not confirmed truth.
4. Evidence explains execution but does not become authority.
5. Worker-first is the default architecture; compatibility is explicit.
6. UI frameworks render Worth state rather than owning a second state engine.
7. Browser-local exactness is not the same promise as durable shared truth.
8. A lower-level API is not automatically a more serious API.

If a page quietly contradicts one of these positions, the page is wrong even
when every method name is spelled correctly.

## Page Types

Do not force every page into one template.

### Tutorial

Lead with the result. Build one honest path from start to finish. Explain only
the concepts needed to complete it, then link outward.

### Guide

Lead with when to use the feature. Name the stable entry points, truth owner,
execution lifecycle, realistic example, failure posture, anti-patterns, and
current limits.

### Concept

Explain the mental model and the decisions it supports. Code is optional when
the concept is clearer without ceremony.

### Reference

Be dense and predictable. List exact signatures, accepted shapes, returned
artifacts, lifecycle states, denials, and deployment constraints. Reference
pages may assume the reader already understands why the feature exists.

### Troubleshooting

Start with the symptom. Move through the shortest discriminating checks and
name the exact inspection surfaces. Do not ask a reader to dump everything and
hope the answer is visible.

## Rhythm And Restraint

- Use short opening paragraphs.
- Vary sentence length enough that the prose sounds spoken, not generated.
- Put the exact API near the top.
- Allow one memorable line per section when it sharpens the model.
- Keep caveats next to the code they constrain.
- Prefer one small example and one realistic example over five toy snippets.
- Remove a section that merely repeats its heading.

Avoid "seamlessly," "leverages," "empowers," "robust," and "comprehensive"
unless the sentence proves the claim immediately. Avoid internal terms such as
"proof lane," "closeout," and milestone phase numbers in public prose.

## The Read-Aloud Test

Read the first three paragraphs aloud. If they sound like a committee report,
rewrite them. If they sound like an advertisement, remove the adjectives. If
they sound cleverer than the feature, calm them down.

The target is a human explanation that an AI can also implement from: precise
structure, exact APIs, explicit limits, and enough judgment to make the intended
path unmistakable.
