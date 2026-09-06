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

This crate has no Query, Store, persistence, replay, codec or physical-runtime
dependency. It does not provide durable restart, multi-parent history, rollback,
or Query public completion. Query application integration is milestone 9.17.3.
