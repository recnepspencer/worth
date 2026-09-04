# Authored Composition

## What This Feature Is

Authored composition lets you define the same Worth UI application from `.wui`
files or typed Rust values. Both routes are compiled by `worth-ui-dsl` into one
sealed semantic package before runtime prepares an application.

## Why You Use It

- Keep product composition editable in files.
- Generate composition in Rust without serializing through text or JSON.
- Preserve one set of identities, diagnostics, and runtime behavior across
  both authoring styles.
- Keep malformed edits outside the active generation.

## Stable Entry Points

- `worth_ui_dsl::WorthUiDslCompiler`
- `worth_ui_dsl::WorthUiRustAuthoredArtifactInput`
- `worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule`
- `worth_ui::facade::source::WorthUiFilesystemSourceProvider`
- `worth_ui::facade::source::WorthUiFilesystemSourceWatcher`
- `WorthUiSettledSourceSnapshot::attempt_source_rebind(...)`
- `WorthUiApplicationBuilder::with_rust_authored_input(...)`
- `WorthUiApplicationBuilder::with_candidate_submission(...)`
- `WorthUiPreparedApplicationAuthority::authored_overlay_material()`
- `WorthUiNativeApplicationShell::begin_source_rebind(...)`

The source facade owns transport, settlement, revision affinity, and candidate
ingress. Authored nodes, spans, compile reports, and semantic packages belong to
`worth-ui-dsl`.

## Core Mental Model

Files and Rust are two ways to author meaning, not two runtime formats.
Compilation freezes imports, declarations, ordering, provenance, and semantic
identity into one package. Runtime ingress attaches the exact source revision
and settlement evidence, then hands the complete candidate to application
preparation or hot rebind.

Malformed syntax stops in the DSL and returns a typed compile report bound to
the held snapshot. A syntactically valid package may still be denied when the
active capability snapshot cannot admit its runtime requirements. Neither
denial changes the current application.

## How It Executes

```text
filesystem event
-> debounce and settle one immutable snapshot
-> one DSL compile against the active capability basis
-> sealed candidate submission | typed compile report
-> initial application preparation or begin_source_rebind
-> semantic comparison
-> preservation, bounded successor, or typed denial
```

Typed Rust input enters the same DSL semantic compiler before application
preparation. After `freeze`, frames use the active sealed package and runtime
indexes. They never reread or reparse authoring source.

## Small Example

```rust
use worth_ui::facade::app::WorthUi;
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;

let app = WorthUi::app()
    .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::default())
    .freeze()?;
```

This is the smallest honest Rust-authored application. Real applications add
typed modules and body atoms before passing the input to the builder.

## Real Example

### Initial Filesystem Launch

```rust
use worth_ui::facade::app::WorthUi;
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;

let snapshot = WorthUiFilesystemSourceProvider::new(workspace_root).read()?;
let capability_app = WorthUi::app().freeze()?;
let submission = snapshot
    .attempt_source_rebind(capability_app.capabilities())
    .into_candidate_submission()?;
let app = WorthUi::app()
    .with_candidate_submission(submission)
    .freeze()?;
let session = app.launch()?;
```

The capability-only preparation supplies the exact admission snapshot for
lowering; it does not launch or become a second active application. The
candidate retains source revision, ordering, semantic handoff, and provenance
as one unit. That handoff also carries the compiler-issued overlay declaration
bindings, admitted backdrop/relation material, portal-anchor rows, and their
source provenance; prepared authority exposes those facts only as immutable
borrowed reads.

An authored Portal may explicitly name its semantic surface, and an interaction
route may explicitly name the Portal it opens. The compiler resolves those
names to issued declaration identities; the prepared runtime binds the declared
surface and Portal owner under the current application generation, staging and
settling the binding with the existing Portal transition. Unqualified legacy
surface creation remains valid but does not acquire authored-Portal eligibility.
This Gate 4 boundary is staged and unpublished: it emits no live appearance.

### Watched Edit

After launch, do not lower loose source parts or prepare another application.
Pass the next settled snapshot to the running shell:

```rust
let request = UiSourceRebindRequest::new(snapshot)
    .with_deadline(shell.rebind_deadline_at(deadline_tick))
    .observed_at_tick(now_tick);
let outcome = shell.begin_source_rebind(request)?;
```

The ordinary bridge performs the one DSL compile, source observation
admission, semantic comparison, affected-scope planning, and governed
publication. See [Hot rebind](./hot-rebind.md) for outcome ownership.

## How It Relates To Other Features

- Register capabilities and Query views on the same application builder before
  `freeze`.
- Use the filesystem watcher for exact settled revisions, not as a publication
  owner.
- Use application inspection for provenance and admitted meaning after
  preparation.
- Use visual comparison only after semantic rebind has produced exact
  predecessor/successor evidence.

## Inspection And Debugging

DSL failures return `WorthUiDslCompileReport` with typed diagnostic identities,
spans, stop classes, and the attempted source revision. Runtime preparation and
rebind failures are separate typed denials. Keep those layers distinct when
presenting errors.

If an edit appears stale, compare provider, package, event, and sequence
affinity. Never reread the latest file and attach the old revision.

## Anti-Patterns

- Parsing `.wui` files inside frame or host-adapter code.
- Converting typed Rust authoring through JSON or source text.
- Extracting declarations from a candidate and rebuilding runtime input.
- Rereading the file after settlement instead of compiling the held snapshot.
- Treating a digest as permission to skip semantic comparison.
- Publishing directly from the watcher or compiling twice for one attempt.

## Current Limits

`WorthUiDslSupportPosture` remains conservative. A type name in the DSL crate
does not promise that every future language family is admitted today. Follow
the current compiler diagnostics and capability support rows.

## Related Docs

- [Worth UI architecture](./architecture.md)
- [Application lifecycle](./application-lifecycle.md)
- [Hot rebind](./hot-rebind.md)
- [Application inspection](./inspection.md)
