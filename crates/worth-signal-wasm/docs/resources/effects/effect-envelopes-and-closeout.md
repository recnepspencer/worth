# Effect Envelopes And Closeout

## What This Feature Is

This page is about the proof record for one effect and the support matrix for a
chosen effect profile.

## Why You Use It

Use it when you need to inspect:

- what one effect actually did
- what proof the runtime retained
- what a profile claims to support across local patch, confirmation, rollback,
  and diagnostics history

## Stable Entry Points

- `line.diagnostics().lastEffect`
- `line.effects().get(effectId)?.envelope`
- `line.effects().open()`
- `line.history().verificationPackage().lifecycle.lastEffect`
- `signals.resource.effects.closeoutMatrix(profile)`

## Core Mental Model

The envelope is the per-effect record.
The closeout matrix is the per-profile support summary.

Use the envelope to debug one write. Use the matrix to choose or explain a
profile.

## How It Executes

The envelope carries:

- profile digest
- provenance
- patch and delivery digest
- branch posture
- optimistic lifecycle
- rollback posture
- confirmation posture

The closeout matrix carries profile-wide capability rows with proof lanes.

## Small Example

```ts
const profile = signals.resource.effects.branchNative();
const matrix = signals.resource.effects.closeoutMatrix(profile);

console.log(matrix.profileName);
console.log(matrix.rows[0].effectFamily);
```

## Real Example

```ts
const effect = line.effects().get(effectId)?.envelope;
if (!effect) throw new Error("effect is unknown");
const profile = signals.resource.effects.branchNative();
const matrix = signals.resource.effects.closeoutMatrix(profile);

console.log(effect.provenance);
console.log(effect.patch.kind);
console.log(effect.optimistic.rollback.kind);
console.log(matrix.rows.map((row) => row.capability));
```

The important boundary here is that `lastEffect.profile` is a digest for
inspection. `closeoutMatrix(...)` takes the declared profile object, not the
inspection digest.

## How It Relates To Other Features

- Use [Branch-Native Effects](./branch-native-effects.md) for the common
  optimistic lane.
- Use [What Happens After A Write](../updating/what-happens-after-a-write.md)
  when you need the broader post-write read path.

## Inspection And Debugging

Start with `line.effects().get(effectId)?.envelope` when investigating a
request. Use `lastEffect` only for recent diagnostics. If the question becomes
"should this profile ever support this?", check the closeout matrix.

## Anti-Patterns

- Do not use the closeout matrix to explain one concrete effect.
- Do not treat an effect envelope as a general family declaration.
- Do not use `lastEffect` to choose among concurrent requests.

## Current Limits

The matrix explains shipped support. It does not bypass runtime-specific denials
such as unavailable branch proof or restore retention.

## Related Docs

- [Effect Closeout Matrix](./README.md)
- [Effect Envelope Contract](./effect-envelopes-and-closeout.md)
