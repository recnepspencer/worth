# Local Truth And Platform Authority

## The Decision

Default to the full Worth platform for production domain truth. Choose
TypeScript Local Truth only when process-local browser authority is an
intentional product boundary and losing it at process termination is
acceptable.

The two lanes are complementary, not interchangeable:

```text
Browser-local:  TypeScript Local Truth -> Signal

Full platform:  Query -> Relational -> Bridge -> Signal
```

The happy browser-local path is small and can grow into branches, snapshots,
and bounded inspection within the running process.

## What Each Layer Owns

### TypeScript Local Truth

Within one running JavaScript process, Local Truth owns:

- declared plain-object entity values;
- aspect validation and equivalence;
- runtime-issued bases, branch heads, snapshots, and commits;
- bounded retained history;
- rebuilding its disposable Signal projections.

Its public kind is `"typescriptInMemoryLocalTruth"`; inspection reports
`supportPosture: "inMemoryProcessLocal"`.

### Query

Query is the ordinary domain-facing declaration and execution boundary. It
owns the public request context, planning, and outcome contract presented to
application code. Application callers should not reach around it to mutate
Relational storage directly.

### Relational

Relational owns authoritative durable commit, MVCC, invariants, history, and
platform merge semantics. This is the primary truth store when state must
survive processes, coordinate actors, or support real recovery and audit.

### Runtime Bridge

The bridge carries committed causal change from the platform authority into
Signal. It transports truth; it does not invent, reinterpret, or re-authorize
the commit.

### Signal

Signal owns derived scheduling, invalidation, evaluation, and inspection.
Signal proofs can explain derived behavior. They cannot authorize an upstream
application-value commit. Signal state is rebuildable from the authority that
fed it.

## Choose The Browser-Local Lane When

- one tab or worker process is the whole collaboration boundary;
- restart loss is acceptable or the host independently recreates the initial
  state;
- local branch editing improves the interaction;
- inspection is for the running workflow, not a durability promise.

Examples include a local configurator, an ephemeral modeling workspace, an
offline experiment whose persistence is explicitly handled elsewhere, or a
standalone demonstration.

## Choose The Full Platform When

- a second user, process, machine, or service must observe the same truth;
- state must survive restart or deployment;
- concurrency needs MVCC or transactional invariants;
- actor identity, authorization, retention, recovery, or audit is required;
- a regulator or customer could reasonably read “history” as durable evidence.

For those applications, keep domain truth in Query and Relational. A browser
may still use Forms, Resources, Router, and Signal projections without becoming
a competing authority.

## The Exact Non-Promise

TypeScript Local Truth does not provide MVCC, persistence, replication,
authenticated actor identity, multi-user coordination, cross-process locks,
or a supported checkpoint export/restore protocol.

Do not call process-local history durable or restart-stable. A digest proves
internal identity within this implementation; it does not prove durable
retention, external notarization, or authenticated authorship.

Commit `metadata` is supplied by the host and is not verified identity. Branch
names are labels and are not authorization. A checkpoint is bounded in-process
retention and is not a database backup.

## Avoid Split-Brain Ownership

Do not write the same domain fact independently into Local Truth and
Relational and then attempt to reconcile them as peers. Pick one authority.

In a full-platform application, the healthy direction is:

```text
domain command
  -> Query public contract
  -> Relational authoritative commit
  -> Bridge committed causal change
  -> Signal derivation
  -> UI projection
```

Local UI drafts can remain local, but the product must name them as drafts and
define the explicit command that proposes them to the platform authority.

## Related Docs

- [Your First Local Truth Store](./getting-started.md)
- [Branches And Snapshots](./branches-and-snapshots.md)
- [History, Compaction, And Rebuild](./history-and-rebuild.md)
- [Local Truth API Reference](./api-reference.md)
