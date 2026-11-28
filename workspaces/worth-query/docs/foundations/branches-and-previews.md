# Branches and Previews

This page covers application product branches: `application.branches()` and
`application.on_branch(...)` from `worth-query-host`.

A product branch is a Runtime World identity with its own history and program
revision.

## Application product branches (host)

A **product branch** is a separate public concept. Runtime World owns its
identity, current composite commit, bounded single-parent history, and exact
Relational/Signal component composition. In a host advancement callback, use
the supplied `phase` for creation, selection and history traversal:

```rust,ignore
let source = application.current_world();
let branch = application
    .branches()
    .fork(source)
    .components(|components| {
        components.fork_relational().reuse_exact_signal_basis()
    })
    .create(&phase)?;
let selected = application.on_branch(branch).select(&phase)?;

// After one or more successful transactions advance this branch:
let page = application
    .branches()
    .history(&phase, branch, NonZeroUsize::new(1))?;
let prior = page.continue_ancestry(&phase, NonZeroUsize::new(1))?;
let entry = prior.entries().next().expect("the prior generation is retained");
let historical = prior.select(&phase, &entry)?;
```

The component selector has four complete postures: reuse both exact bases,
fork Relational and reuse Signal, reuse Relational and fork Signal, or fork both.
The product identity never collapses into either component identity. Branches
that share Signal may advance their Relational histories independently.

### Fork from an admitted selection

Use `fork_expected` when creation must use a selection already admitted by the
application. It consumes that selection and refuses if its source has moved.
The ordinary `fork` operation resolves its source during creation.

```rust,ignore
let selected = application.on_branch(source).select(&phase)?;
let candidate = application
    .branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
let branch = candidate.branch();
```

Run ordinary requests with `.on_branch(branch)`. Keep `candidate` while working:
it retains the exact source admission alongside the new branch.
`candidate.expected_source()` returns descriptive information about that source.

To select the candidate as a model's head, pass the model owner's actual mutable
branch slot to `adopt`. Run the authored requests before this call:

```rust,ignore
let previous = candidate.adopt(&mut model.head, &phase)?;
// Subsequent requests use model.head. Retire the previous branch separately.
application.on_branch(previous).close(&phase)?;
```

Adoption compares the slot with the original source and checks the source's exact
Native/World basis while replacing the slot. It refuses if the source moved during
candidate evaluation, even if the source's values later returned to their original
values. A retired candidate or a phase from another runtime also refuses. The
operation synchronizes with ordinary commits and protects both heads against
retirement during replacement. Cancellation or deadline expiry refuses before
the slot changes, including interruption while waiting for the guards.

A refused adoption leaves the supplied head unchanged and returns the candidate
and typed denial through `failure.into_parts()`. Its ordinary request effects
still exist on the candidate; retain it for inspection or discard it with `close`.
A refused close of the previous branch after successful adoption leaves the new
head selected. Handle that close's retry or pending cleanup separately.

Discard the candidate with `candidate.close(&phase)`. A refused close returns
the candidate and original denial through `failure.into_parts()`; inspect
`failure.denial()` to select the ordinary retry or pending-cleanup path. A
`ProductUnpublished` creation outcome retains its existing creation recovery.
Complete that recovery before issuing another creation attempt.

If the expected source is stale, obtain a fresh selection before creating a
replacement candidate. A selection from another application is refused.
The [compiled adoption cases](../../crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/adoption.rs)
show ordinary candidate requests, head replacement, stale-source refusal and
cleanup.

### Own an application branch head

Consume a newly created expected-source candidate with `into_owned_head()` when
the application needs a move-only owner for its branch slot. Keep that owner in
the application state and use its descriptive `branch()` for ordinary requests.
A copied selector cannot construct another head owner.

```rust,ignore
let mut head = initial_candidate.into_owned_head();
let selected = application.on_branch(head.branch()).select(&phase)?;
let candidate = application.branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
// Run the ordinary candidate requests before adoption.
let previous = head.adopt(candidate, &phase)?;
previous.close(&phase)?;
```

`head.adopt` uses the same exact-source and currentness guards as candidate
adoption. Success returns the previous head as a separate owner; refusal leaves
the current head unchanged and preserves the candidate in the failure.
Converting the initial candidate releases its expected-source admission. The
head owner manages branch custody; ordinary request authorization still applies.

Close each head explicitly. A `Retained { head, denial }` failure returns a live
head after a refusal before retirement. A `Retired { branch, failure }` failure
returns the actual cleanup obligation after retirement. Retain
`failure.into_cleanup()` and call `retry_in_advancement(&phase)` inside an
admitted advancement. Standalone callers can use `retry()` outside an active
advancement. A further failure retains that same
cleanup through `into_cleanup()`. For example, a reader holding the retired
head can keep cleanup pending until that reader is released.

The [compiled head-owner cases](../../crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/owned_head.rs)
show adoption, independent heads, wrong-source refusal, foreign-phase close
refusal and cleanup after releasing a held reader.

### Personal workspace history

Personal tree undo, redo, checkout and grouped gestures are held for
branch-native workspace history.

### Store an expected basis and select it again

Keep records on a separately owned storage branch when saving them must leave
the authored model's basis current. Fork that branch's Relational component
with `parts.fork_relational()` and bind recording requests with
`.on_branch(storage_branch)`. Keep the storage branch's lifecycle owner and
close it explicitly when the records are no longer needed.

Use a declared canonical field binding for structured records so ordinary
queries restore their typed values. The compiled
[record schema](../../crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/schema/data.rs)
and [storage example](../../crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/storage.rs)
store model and target occurrences, ordered typed parameters and comparison
data through ordinary mutations and queries. Its collection query returns a
complete result or a typed limit refusal; exact-key selection follows the
successful read.

Retain one observation privately and perform the model's governed read at that
observation before describing it. Use `observation.select_on(&phase, runtime)`
to describe the same selection after admission. The
[compiled reader](../../crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/application/model.rs)
uses current READ authority with retained data; storage and model reads have
independent policies over the actual grant facts on their respective branches.

Call `selected.describe_basis()` before releasing an admitted selection. Store
the returned `WorthQueryExpectedProductBasisDescription` with ordinary versioned
application data. It implements Serde serialization and contains comparison
coordinates only; keeping it does not retain a snapshot or keep a branch alive.

```rust,ignore
let selected = application.on_branch(model.head).select(&phase)?;
let bytes = serde_json::to_vec(&selected.describe_basis())?;
drop(selected);

// Load the application record under a fresh ordinary READ request first.
let description: WorthQueryExpectedProductBasisDescription =
    serde_json::from_slice(&bytes)?;
let selected = application.branches().select_expected(&description, &phase)?;
let candidate = application.branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
```

`select_expected` asks the installed World and Native owners to admit the exact
described basis. It refuses foreign owners, retired or replaced occurrences,
movement and unsupported description versions. Returning the values to their
previous contents does not restore the original basis. A retained historical
Native basis also cannot substitute for the current described basis. These
coordinates address their original live installation. Use them only within that
installation's lifetime; they are not portable branch restoration data.

Loading or editing the description grants no domain READ or mutation permission.
Use ordinary requests for those operations. Creation and adoption still check
the expected source, so movement after readmission can refuse either operation.
Keep and close candidate custody as described above. The
[compiled stored-basis cases](../../crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/proposal_acceptance/stored_basis.rs)
show serialization, fresh owner admission, ordinary candidate requests, adoption
and stale or mixed-basis refusal.

### Branch history and retirement

To correct a retained operation on a branch, bind an ordinary request with
`.on_branch(target)` and use its
[recorded correction entry](../execution/recorded-source-correction.md).
Selection proves the source belongs to the target's retained ancestry; preparation
checks current disclosure, program meaning and inverse authority. Publication
appends one child of that target and preserves its disjoint intervening edits.

Correction alternatives also belong to the selected branch's history. Use
`request.corrections().alternatives(...)`, follow its continuation to completion,
and select the historical reversal the user chose. The completed traversal
resolves the active reversal on that exact target. For example, after a source
is reversed, reapplied and reversed again, choosing the first reversal reapplies
the retained source input and consumes the currently active reversal. The
receipt records both identities. A sibling branch keeps its own active reversal
and continuation.

Each history continuation binds its target observation. If the branch advances
between pages or before selection, obtain a new traversal with a fresh request.
Keep the command's idempotency key when retrying a submitted correction: an
equivalent retry resolves its original performed occurrence. The
[history chooser example](../../crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/tree_reapplication.rs)
shows explicit older-alternative selection, pagination and ordinary close.

A selected occurrence pins exact history for repeatable reads while the branch
head advances. Closing a product retires its World history first and then any
private component branches. History a live branch still reaches is not the
closed branch's to release: a live fork keeps its whole ancestry, including
history it inherited from the closed branch, and the close completes. A
publication that installed its successor without moving the head keeps that
history until its recovery is released, and the release then frees the closed
branch's history the same way. Retained
reads can defer cleanup; the returned pending cleanup retains the only retry
authority. A retired occurrence cannot be selected again, even if an
application later reuses the same display name.

Run isolated transaction steps on a fork of the selected branch. Keep the
application's selected branch in its owning model state, and have that owner
choose the branch used by subsequent requests after the steps succeed. Use
`close` to retire a discarded branch and release its resources. A refused step
on the fork leaves the parent's head and history unchanged.

The fork reads its parent's history as its own, so corrections can select a
source made on either branch. Retained history grows with the actual authored
operations across branch selection changes. The
[fork transaction example](../../crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/fork_head_adoption.rs)
shows request execution, explicit branch selection and retirement.
History pages are branch-occurrence scoped and bounded by the caller's nonzero
limit. Their entries are descriptive until `select` asks World to issue an
exact historical observation; copied commit identities cannot perform that
admission. Drop history pages and historical reads before closing the branch
whose ancestry they protect.
See the executable
[advanced product branching example](../../crates/worth-query-certification/examples/advanced_product_branching.rs).

### Branch-local application programs

Each product occurrence carries one owner-admitted application-program
revision. Sibling branches may therefore remain on P0 and P1 simultaneously.
`application.request(...).on_branch(branch).programs().inspect()` reports the
program carried by that exact selection; the report is not adoption authority.
Ordinary queries, mutations, retained reads, security bases, pending producers,
and required-output work resolve through the same selected occurrence.

Adoption compares installed canonical meaning, derives migration and custody
requirements, validates existing state under the target rules, and reserves
one branch-bound publication. A component-local success does not activate the
target. Only the final World `Performed` outcome records the target program;
`ProductUnpublished` retains the original source, target, migration, owner
effects, and exact recovery route.

For multiple branches, call
`application.branches().program_adoption_coverage(&branches, maximum_targets)`
and then `request.on_branches(coverage, ordered_targets)`. The returned handle
is bounded and explicitly non-atomic: inspect `progress()`, resolve a
`NoEffect` or `ProductUnpublished` stop, and then resume, cancel, or close it.
Later-created branches never enter previously issued coverage. Handle each
branch's program adoption outcome before resuming the remaining set.

### Workflows on a forked branch

A fork copies the source branch's workflow definitions and instances as
history, not execution. A copied instance cannot act on the fork: its
approvals and receipts open nothing there. To keep that work going on the
fork, issue `prepare_workflow_fork_continuation(&workflow, instance, target,
resume_at)` on the fork. It ends the fork's copy and starts a successor on the
fork's current `target` definition, under the same law as migration. The
instance on its own branch is untouched. The instance-start capability
authorizes it. It is refused while an owner operation is unsettled and after
the instance's deadline. See
[Migration and fork continuation](workflows.md#migration-and-fork-continuation).

For the request lanes that run against a selected branch, see
[Branch-resolved mutation lanes](ordinary-application-front-door.md#branch-resolved-mutation-lanes).

## Current Limits

Product-branch history and coordinated Relational-plus-Signal publication are
owned by [Runtime World](../../../../crates/worth-runtime-world/README.md).

## Related Docs

- [Graph Touch Obligation Authority](../authoring/graph-touch-obligation-authority.md)
