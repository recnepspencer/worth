# Typed Results, Denials, And Unavailability

Worth uses discriminated results when "the operation could not honestly run"
is ordinary application state. Exceptions are reserved for invalid calls,
construction failure, failed capability assertions, and host/runtime faults.

## Result Vocabulary

| Word | Meaning | Typical handling |
| --- | --- | --- |
| fulfilled / applied / committed / admitted | The requested transition ran. | Consume the value or evidence. |
| partial | Some declared result is usable, but completion is incomplete. | Render the usable state and preserve the missing-detail reason. |
| rejected / denied / blocked | A policy, basis, dependency, or admission rule refused the transition. | Show the reason and available recovery. |
| unavailable | The runtime lacks authority, retained evidence, host support, or declared shape for an exact result. | Choose an explicit fallback or stop. |
| stale / superseded / duplicate | A newer or already-settled operation owns the lifecycle. | Ignore as instructed by the typed result; do not replay blindly. |
| incompatible | The supplied artifact or descriptor belongs to a different contract. | Recreate it from the current declaration. |

The exact discriminator differs by domain. Check the relevant union rather than
assuming every result has a property named `kind`.

## When The Package Throws

The package throws when the call itself is invalid or cannot establish a
runtime:

- invalid construction options;
- worker-first construction without a usable `Worker` constructor;
- a failed `assertCompatibility(...)` requirement;
- a handle supplied to the wrong runtime;
- an invalid declaration or operation shape;
- a host or runtime exception that the domain contract does not model as an
  ordinary result.

Worker-unavailable construction errors carry structured fields:

```ts
import {
  createSignals,
  type SignalsConstructionArtifact,
} from "worth-signals-wasm";

function isConstructionArtifact(
  value: unknown,
): value is Error & SignalsConstructionArtifact {
  return value instanceof Error && "artifactFamily" in value;
}

try {
  await createSignals();
} catch (error) {
  if (!isConstructionArtifact(error)) throw error;

  if (error.artifactFamily === "workerUnavailableConstruction") {
    console.error(error.reason, error.compatibilityRecovery);
  }
}
```

The recovery object is advice, not an automatic fallback.

## Resource Results

Resource lines use typed results for:

- settlement: fulfilled, partial, timed out, rejected, or otherwise failed;
- optimistic patch admission and duplicate effect IDs;
- effect confirmation, rejection, and cancellation;
- server delivery applied, duplicate-ignored, basis-rejected, or
  basis-refreshed;
- exact replay, restore, and rollback unavailable when history or proof is
  insufficient;
- download and transfer descriptors that are ready, unavailable, or
  incompatible.

```ts
const settlement = await projectLine.awaitSettlement({ timeoutMs: 5_000 });

switch (settlement.resultKind) {
  case "fulfilled":
  case "partial":
    renderProject(settlement.value);
    break;
  default:
    renderProjectUnavailable(settlement);
}
```

Use the exact domain reference before enumerating cases; lifecycle unions can
gain detail without changing the high-level posture.

## Form Results

Forms keep readiness, admission, execution, and presentation separate:

- `form.readiness()` and `form.actionReadiness(id)` expose blockers before an
  action runs;
- action plans and executions preserve `resultKind`, recovery actions, and the
  plan digest used for execution;
- reset and replay/restore report no-op or unavailable instead of pretending a
  rollback happened;
- resource drift exposes stale posture;
- route authority can freeze, discard, defer, or clear draft continuity.

Do not collapse these reads into one `isValid` boolean. A form can be valid but
blocked by admission, offline posture, source drift, or an unchanged patch.

## Router Results

Router outcomes distinguish raw browser location from admitted route state.
Projection, admission, recovery, speculative navigation, history replay, and
restore return their own typed artifacts. A raw URL is not admitted merely
because it parsed.

Handle non-admitted outcomes at the router boundary. Do not let components
guess route authority from `window.location`.

## Local Truth Results

Local Truth mutation outcomes distinguish committed state from denied or
unavailable operations.

## Retain Evidence, Not A Second Truth Store

Result artifacts can explain what happened and support UI decisions. They do
not become the application value being explained. Retain only the evidence your
diagnostics or audit UI needs, and continue to read the value from its owning
resource, form, router, or Local Truth surface.

## Anti-Patterns

- parsing error or reason strings instead of narrowing a discriminator;
- converting unavailable into `null` and losing why exactness was impossible;
- retrying stale, duplicate, or superseded operations without inspecting their
  lifecycle;
- treating an optimistic visible value as confirmed server truth;
- manufacturing a successful-looking artifact when proof is unavailable;
- storing result artifacts as a parallel application state engine.

## Related Reference

- [Support Status](./support-status.md)
- [Resource Line](../api-reference/resource-line.md)
- [Resource API](../api-reference/resources.md)
- [Form API](../api-reference/forms.md)
- [Construction API](../api-reference/construction.md)
