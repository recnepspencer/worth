# Your First Form

This tutorial builds one editable form. It uses no validation, action, route,
or resource machinery because none is needed to explain the core loop.

## 1. Create A Source

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();
const task = signals.input({
  title: "Ship docs",
  done: false,
});
```

The input owns source truth. A form reads that truth; it does not take ownership
of the input or mutate it when a field changes.

## 2. Declare Fields

```ts
const form = signals.form({
  id: "task-editor",
  source: task,
  fields: ({ field }) => ({
    title: field<string>("title"),
    done: field<boolean>("done"),
  }),
});
```

The object keys (`title`, `done`) are field IDs used by the controller. The
strings passed to `field(...)` are paths into the source value. Nested paths
such as `"owner.name"` are supported.

For a live application source, the direct signal form above is fine. Use
`signals.form.source.signal(task, { id: "task" })` when you want the source
authority to be explicit and inspectable in the declaration.

## 3. Edit Through Field Handles

```ts
form.fields.title.set("Publish docs");

console.log(form.fields.title.sourceValue()); // "Ship docs"
console.log(form.fields.title.draftValue());  // "Publish docs"
console.log(form.fields.title.value());       // "Publish docs"
```

`set(...)` changes controller-local draft truth. It does not write through to
the source. `clearDraft()` removes the draft for that field.

For coordinated edits, `form.setFields({ title: "Publish docs", done: true })`
prepares all declared field paths and write postures before changing the draft,
then publishes one summary revision. Field IDs, rather than source property
paths, are the keys. `form.setFields({}, { clearDraft: ["title", "done"] })`
removes those overlays together. Unknown IDs, duplicate IDs, and overlapping
parent/child paths are rejected before the draft changes. The source remains
authoritative for fields outside the batch.
Pass `touch: ["title"]` in the same options object to record field interaction
in that summary revision. Touch IDs are validated before any draft change.

## 4. Read The Form

```ts
const sourceValue = form.source();
const draft = form.draft();
const effective = form.effective();
const dirty = form.dirty();
const patch = form.patchPlan();
const readiness = form.readiness();
```

- `source()` reads the current declared source.
- `draft()` contains edits, not a second complete source object.
- `effective()` overlays draft values on the source.
- `dirty()` compares semantic values, not merely whether `set(...)` ran.
- `patchPlan()` describes the changes the field declarations can prove.
- `readiness()` explains whether the current form may submit and why not.

An unchanged form is normally not submit-ready. If you set the title back to
`"Ship docs"`, dirty state becomes false and the patch plan becomes empty.

## 5. Let The UI Render

Worth does not render an `<input>`. A UI adapter reads the handle and reports
user work back through a small binding like this:

```ts
const titleBinding = {
  value: form.fields.title.value(),
  onValue(value: string) {
    form.fields.title.set(value);
  },
};
```

Framework adapters can add raw-input, composition, focus, message, layout, and
accessibility semantics. The form controller remains the source of draft and
readiness truth; the DOM remains the renderer.

## Inspect Before Adding Machinery

```ts
console.log(form.sourceAuthority());
console.log(form.declaration());
console.log(form.fieldContract());
console.log(form.diagnosticsSummary());
```

If all you need is a normal editable value, stop here. Add validation when you
have a validation rule, and add an action when something must actually execute.

## Next

- [Choosing A Form Source](./choosing-a-form-source.md)
- [State, Fields, And Changes](../state/README.md)
- [Validation And Messages](../validation/README.md)
- [Actions And Submission](../actions/README.md)
