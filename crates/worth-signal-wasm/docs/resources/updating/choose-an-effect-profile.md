# Choose An Effect Profile

## What This Feature Is

Effect profiles decide how local writes behave.

They answer questions like:

- is the write optimistic
- what kind of confirmation is expected
- what rollback posture exists

## Why You Use It

- choose write behavior explicitly
- keep optimistic and cautious flows intentional
- make those choices inspectable later

## Stable Entry Points

- `signals.resource.effects.branchNative()`
- `signals.resource.effects.serverCanonical()`
- `signals.resource.effects.pessimistic()`
- `signals.resource.effects.deliveryAuthoritative()`
- `signals.resource.effects.nonReversible()`
- `signals.resource.effects.sensitive()`
- `signals.resource.effects.closeoutMatrix(profile)`

## Core Mental Model

An effect profile is not a label. It is a bundle of runtime behavior.

If your main question is "how should this write feel and recover?", you are
choosing an effect profile.

## How It Executes

The chosen profile becomes part of the admitted request posture and later shows
up through diagnostics, lifecycle and rollback behavior.

## Small Example

```ts
const branchNativeApi = signals.api({
  effects: signals.resource.effects.branchNative(),
});
```

## Real Example

```ts
const branchNative = signals.resource.effects.branchNative();
const matrix = signals.resource.effects.closeoutMatrix(branchNative);

console.log(branchNative.name);
console.log(matrix.profileName);
console.log(matrix.rows.map((row) => [row.effectFamily, row.capability]));
```

## How It Relates To Other Features

- Use [Write A Resource](./write-a-resource.md) when the main problem is the
  route-first write itself.
- Use [What Happens After A Write](./what-happens-after-a-write.md) once you
  want to inspect the runtime result.
- Use [Branch-Native Resource Effects](../branch-native-effects.md) for the
  deeper lifecycle and proof lane behind these profiles.

## Inspection And Debugging

Start with:

- `line.diagnostics().lastEffect`
- `signals.resource.effects.closeoutMatrix(profile)`

The closeout matrix is the fastest way to answer "what does this profile claim
to support?"

## Anti-Patterns

- choosing a profile by name alone without checking what it admits
- using effect profiles to solve fetch freshness problems

## Current Limits

- this page chooses the profile
- the deeper effect proof and rollback lanes still live in the older
  effect docs for now

## Related Docs

- [What Happens After A Write](./what-happens-after-a-write.md)
- [Older Branch-Native Resource Effects](../branch-native-effects.md)
- [Effect Closeout Matrix](../effects/README.md)
