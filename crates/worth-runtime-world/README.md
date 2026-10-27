# `worth-runtime-world`

Runtime World owns memory-resident product branches and coordinated publication
across real Relational and Signal owners. Bridge owns their installed semantic
correspondence; component owners retain their own mutation, settlement and basis
authority. The public import surface is `worth_runtime_world::facade`.

Construct a World with all five required inputs: the Bridge correspondence port,
Relational services, generic Signal services, installed `RuntimeWorldBudgets`, and
an explicit `RuntimeWorldClock`. Bind Bridge correspondence to the actual Signal
graph before sealing that graph's owner services. Bootstrap explicitly from the
three owners' admitted bases; construction does not create an ambient product head.

The [executable example](examples/runtime_world_publication.rs) contains the full
public construction and lifecycle, including its real schema and correspondence
declarations. Run it with:

```sh
cargo run -p worth-runtime-world --example runtime_world_publication
cargo test -p worth-runtime-world --example runtime_world_publication
```

The example gives its worker an explicit 4 MiB stack for unoptimized component
construction on Windows. It executes bootstrap, a successful Relational-only
publication, a stale prepared attempt, and a retained partial followed by cleanup.
Its Cargo declaration uses `test = true` and `harness = false`, so package tests
execute the real workflow.

`RuntimeWorldOwner<D, I, E, Ctx, T>` is non-cloneable. Its cloneable ports are weak
access to the live owner, not independent owners. Observation, branch, lifecycle,
inspection and recovery ports do not expose the Signal generic bundle;
`RuntimeWorldPublicationPort<D, I, E, Ctx, T>` preserves it. Event and context
types need not implement `Clone` or `Default`.

Publication starts with a complete `ProductBranchObservation`. Use
`prepare_without_signal` for a Relational-only intent, or `prepare_with_signal`
for Signal-only/combined work. Their prepared types cannot be exchanged.
`execute_with_signal` accepts the real Signal transaction callback. No
application callback runs after the final product comparison cutoff.

Handle every `RuntimeWorldPublicationOutcome`:

- `Performed`: the product CAS installed the exact successor. Borrow the
  canonical component results and consume the linear delivery once.
- `NoEffect`: neither component nor product moved. Correct the denied intent or
  acquire an appropriate fresh observation before preparing a new attempt.
- `ProductUnpublished`: named component effects exist but the product head did
  not move. Preserve the returned evidence and use explicit recovery/cleanup;
  it does not authorize a product commit or sibling continuation.

Keep the World owner and component roots alive while using their ports. Explicit
close fences new work, reports outstanding observations and retained records, and
returns any owner-created component retirement work. Drop caller observations and
results, then the closed World state before checking final component lease release.
World close does not close component owners.

See [history](COMPOSITE_HISTORY.md), [publication](COORDINATED_PUBLICATION.md),
and [retention/recovery](RETENTION_AND_RECOVERY.md) for exact contracts.

## Exact retained product history

`observation_port().open_exact_product_history(execution, &observation)` retains
that genuine product occurrence. `prepare_exact_product_history_page` follows
one authoritative parent step and retains its exact component bases, including
inherited fork ancestry. It reads no current head or reconstructed historical
branch generation. Missing/retired raw parents return a typed history gap;
retirement splices remain confined to the older inspection traversal.

Accept the prepared page through its originating cursor to receive `(page,
successor)` under the stored original request. The original cursor stays intact
until the composition owner installs the complete successor state. Drop a staged
page or uninstalled successor to decline advancement. Opening performs one catalog
admission; each page inspects one product-parent step with constant additional
position storage. Opening and new retained positions reserve memory through the
original request's effective serial or ancestor context before allocation;
the actual retained owners keep those reservations until final drop.
Pages, cursors and their pinned source observations release
their genuine history/component custody on drop. Component publication evidence
distinguishes new native changes from `RetainExact` reuse; this owner does not
invent a native event for every product occurrence. See the
[exact history courts](tests/runtime_world_certification/court/exact_history.rs).
The declared application history query and fresh disclosure remain Phase 10 work.

## Owned async source binding

`RuntimeWorldOwner::admit_owned_async_request` first checks the World owner and
the exact retained Bridge source observation. It derives the source commit from
that admitted Relational observation's immutable reference target. A populated
fork may have branch truth version zero while still selecting a committed source;
branch versions and global issuance counters never substitute for that commit.
Sibling publication and discarded prepared occurrences do not move this binding.

An empty target returns `RuntimeWorldOwnedAsyncRequestAdmissionDenial::NoCommittedSource`
before Signal request admission. Revalidation has the corresponding
`RuntimeWorldOwnedAsyncRevalidationDenial::NoCommittedSource`; refusal keeps the
original request available for completion or retirement. Bridge and Signal own
the remaining lifecycle, including exact-request completion, timeout/retry and
cleanup. The grouped
[owned async court](tests/runtime_world_certification/owned_async.rs) exercises
the native reference equality, empty/committed twins and active-request release.
The application resource entry is a separate Query integration contract.

`admit_unqueued_owned_async_request` is the custody-preserving internal engine
route. It applies the same exact World source validation and returns the genuine
Bridge custodian even when its later descriptive handoff refuses. Queue mode
comes from the installed declaration; queued declarations cannot enter this door.
Initial supersession retains and retires the displaced timeout, while the created
request keeps its own timeout. This lower route grants no application permission.

For a read-only source, `admit_unqueued_owned_async_request_at_branch_head`
selects the genuine observed head snapshot through the same World/Relational
affinity checks. Its retained `BranchHead` basis carries no commit identity or
claim of fresh currentness. The UI presentation backend uses this posture because
it owns no product writes. It cannot substitute for an authoritative committed
source; the committed entrance still refuses an empty target.

Queued Bridge declarations may opt into `with_process_local_retention` before
installation. The genuine installed Signal descriptor selects this posture;
legacy producers retain their existing policy. Exact release requires a terminal
occurrence, an empty queue, no pending retry/wake cleanup, and the genuine commit
owner for fulfillment. A refusal returns both custodians; success returns released
evidence without a live request capability. Exact predecessor, retry, denial and
retired-wake disposal uses the existing retention owners and omission summaries,
leaving other producers untouched. Fulfilled process-local requests release their
completion first and enter again through fresh Start rather than revalidation.
A pending retry can be explicitly aborted before due through its genuine
custodian, retiring that exact backoff without advancing the clock; its actual
schedule and retirement evidence remain in a fixed released slot.
The stored occurrence enforces this custody protocol: raw Native timeout, retry
and revalidation entrances refuse ProcessLocal occurrences before effects.
Legacy queued occurrences keep their original behavior. Releasing a predecessor
preserves the generation charge selected by its genuine current custodian,
including pending retries hidden by a newer same-node index entry.
These operations do not supply Query provider storage, authorization or restart.

This crate has no Query, Store, persistence, replay, codec or physical-runtime
dependency. It does not provide durable restart, multi-parent history, rollback,
or Query public completion. Query already consumes this crate through
`worth_runtime_world::facade` (from `worth-query-execution` and
`worth-query-publication`) to publish application commits and report their
product outcomes.
