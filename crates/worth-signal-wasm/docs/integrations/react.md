# React

The React adapter subscribes components to Worth Signals state. It does not
copy that state into a second React-owned store.

The published adapter supports React 18 and React 19. React and its type
package remain application-owned dependencies; installing Worth Signals alone
does not choose either version for the application.

## Stable Entry Points

- `createReactSignalsStore(signals)`
- `ReactSignalsStoreProvider`
- `useReactSignalsStore()`
- `useSignalValue(handle, store?)`
- `useOutputValue(handle, store?)`
- `useOptionalSignalValue(handle, store?)`
- `useSignalsDiagnostics(store?)`
- `useSignalsHistory(store?)`

Resource, form, and router hooks live in the same `worth-signals-wasm/react`
subpath.

## Create One Store

```tsx
import { createSignals } from "worth-signals-wasm";
import {
  createReactSignalsStore,
  ReactSignalsStoreProvider,
} from "worth-signals-wasm/react";

const signals = await createSignals();
const store = createReactSignalsStore(signals);
// Worker-first empty roots are supported: React attach does not require
// importGraph(...) first. Snapshot diagnostics start empty and refresh after
// standalone mutations or importGraph(...). why(id) works for known authored
// or imported signal ids.

root.render(
  <ReactSignalsStoreProvider store={store}>
    <App />
  </ReactSignalsStoreProvider>,
);
```

The store adapts subscriptions and transactions. `signals` remains the state
runtime.

The first signal subscription revalidates the snapshot read during render, so
a write before subscription attachment is visible without waiting for another
write. An unchanged value retains its snapshot reference. Worker readback also
preserves a newer host projection while its input writes await acknowledgement;
older worker replies cannot replace that newer visible value.

Measure action-to-DOM delivery, worker acknowledgement, and startup separately.
Worker messaging can cost more than a small synchronous computation, even when
the host projection makes its visible update sooner. Placement is not a promise
that every operation completes faster.

## Read And Write A Signal

```tsx
import { useSignalValue } from "worth-signals-wasm/react";

function QuantityEditor({ quantity }) {
  const value = useSignalValue(quantity);

  return (
    <button onClick={() => void quantity.set(value + 1)}>
      Quantity: {value}
    </button>
  );
}
```

Do not mirror `value` with `useState` and an effect. That creates two owners and
adds a race you now have to explain.

React state is still appropriate for genuinely UI-owned values such as whether
a popover is open or which diagnostic row a person selected.

## Read A Published Output

```tsx
import { useOutputValue } from "worth-signals-wasm/react";

function Total({ checkout }) {
  const total = useOutputValue(checkout.output("total"));
  return <strong>{total}</strong>;
}
```

Graph output names remain part of the published feature boundary. React does
not need to know the graph's internal handles.

## Use Your Existing Form Components

`useSignalsForm(...)` returns headless field and action bindings. Pass them to
the input components your application already owns:

```tsx
import { SubmitButton, TextInput } from "../ui/forms";

function ProfileEditor() {
  const editor = useProfileEditor();

  return (
    <form onSubmit={editor.save}>
      <TextInput label="Email" field={editor.email} />
      <SubmitButton action={editor.submit}>Save profile</SubmitButton>
    </form>
  );
}
```

Worth owns values, messages, readiness, and interaction lifecycle. Your
components own HTML, accessibility presentation, and styling. Do not copy a
binding's value into component state or perform resource I/O in the component.
See [Form API](../api-reference/forms.md) for the complete form, validation, and
hook recipe.

## Lifecycle

Create the runtime and store at an application boundary that outlives the
components using them. Free feature subscriptions when their feature unmounts.
Free the runtime only when the application is permanently done with every
handle it created.

## Anti-Patterns

- Do not mirror signal values with `useState` plus `useEffect`.
- Do not create one Signals runtime per component.
- Do not subscribe manually when a package hook already owns that lifecycle.
- Do not put application truth in React context merely to make it reachable.

## Related Docs

- [Your First Signal](../getting-started/first-signal.md)
- [Form API](../api-reference/forms.md)
- [Graphs And Controllers](../core/graphs-and-controllers.md)
- [Diagnostics And Explanation](../core/diagnostics.md)
