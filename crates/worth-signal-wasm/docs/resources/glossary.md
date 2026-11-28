# Resource Glossary

## Family

The declaration for one resource shape.

Examples:

- a detail family for `/users/:userId`
- a collection family for `/tasks`
- a paged family for `/tasks/search`

Families are created by `signals.api(...).url(...).detail(...)` and friends, or
by the raw `signals.resource.*(...)` lane.

## Line

One live member of a family.

You create a line with `family.line(params)`.

That line owns reads like:

- `line.value()`
- `line.summary()`
- `line.request()`
- `line.diagnostics()`

## Canonical Identity

The stable identity for one logical resource member.

In the raw lane, `normalizeParams(...)` must return
`resourceParamIdentity(params, canonicalKey)`. The canonical key is what keeps
refresh, replay, history, and rematerialization talking about the same line.

## Detail

One resource member.

Use `.detail(...)` when the value is "one thing", even if that thing is a large
object.

## Collection

One list-shaped resource value with stable item identity.

Use `.list(...)` or `signals.resource.collection(...)` when you want one loaded
snapshot of a list and item-level helpers can make sense.

## Paged

One list-shaped resource value where later pages can accumulate into the same
line.

Use `.paged(...)` or `signals.resource.paged(...)` when page accumulation is a
real part of the feature.

## Request Posture

The admitted request configuration for a line:

- auth
- headers and request context
- retry / stale policy
- continuation
- processing posture
- upload posture

Read it back with `line.request()`.

## Effect Profile

The runtime policy for how local writes behave.

Examples:

- `signals.resource.effects.branchNative()`
- `signals.resource.effects.serverCanonical()`
- `signals.resource.effects.pessimistic()`

An effect profile sets optimism and rollback posture.

## Patch

A local runtime change applied to a loaded line.

You normally create patches from family helpers such as:

- `family.patch.replace(...)`
- `family.patch.itemAspect(...)`
- `family.patch.field(...)`

Then apply them with `line.patch(...)`.

## Delivery

A pushed or externally admitted update packet for a loaded line.

You normally create delivery packets from family helpers such as:

- `family.delivery.replace(...)`
- `family.delivery.patch(...)`
- `family.delivery.invalidate(...)`

Then apply them with `line.deliver(...)`.
