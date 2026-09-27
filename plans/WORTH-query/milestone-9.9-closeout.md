# Milestone 9.9 Closeout: Graph Touch Obligation Authority

> **Status:** Closed
>
> **Closure authority:** `Milestone 9.9 Graph Touch Obligation Authority Hostile Certification Matrix`
>
> **Required suite:** `Milestone 9.9 Graph Touch Obligation Authority Hostile Certification Matrix`

Milestone 9.9 closes graph touch obligation authority as a Query-owned
boundary. Closure is not claimed from API presence. It is derived from the
Phase 20 hostile certification matrix, docs agreement, Consumer Kit adoption
proof, and reference-consumer adoption proof in `worth-topo` and
`worth-kernel`.

The closed surface makes touch descriptor, operating world, and obligation
index determine obligation selection and dispatch on covered graph-bearing
surfaces. Downstream consumers adopt that authority through Query-shipped
registration, selector coverage, support pinning, in-memory execution proof,
bypass audit, adoption manifest, and residue manifest surfaces instead of
building local legality ceremony.

## Closed Authority Families

- graph touch descriptors and selectors
- obligation registration and index-backed selection
- canonical dispatch, materialized dispatch, execution rows, and reduction
- support posture matrix with supported, unsupported, not-applicable,
  diagnostic-only, and deferred-to-backstop rows
- execution budget, state-load counters, and `BudgetExceeded` policy evidence
- envelope projection onto receipts, decision traces, and mutation evidence
- Consumer Kit graph-obligation adoption proof
- docs agreement and AI guidance
- `worth-topo` operator catalog adoption
- `worth-kernel` construction adoption

## Explicit Residue

Milestone 9.9 does not claim zero residue across every downstream construction
surface. It claims zero silent covered-lane bypass. Remaining residue is
certified, owner-tagged, introduced in a named phase, capped by
`must_not_exceed_count`, and tied to a removal trigger.

`worth-kernel` construction adoption carries these explicit residue classes:

- `kernel-handoff-only-result-helper`
- `kernel-motion-preflight-sequencing`
- `kernel-primitive-family-cardinality-gap`
- `kernel-birth-selector-conjunction-gap`

These rows are accepted only as manifest residue. They are not alternate Query
authority, not local support-pinning ceremony, and not permission to bypass
covered graph obligation dispatch.

## Defended Exclusions

Automatic graph-read access planning and background index provisioning are
Milestone 9.10 scope.

Durable persisted obligation envelopes and persisted Consumer Kit proof archives
remain later store-backed scope.

## Verification

Milestone 9.9 closeout is verified by:

```powershell
cargo test -p worth-query --test graph_obligation_hostile_certification
cargo test -p worth-query --test graph_obligation_docs_agreement
cargo test -p worth-query --test graph_obligation_consumer_kit_facade
cargo test -p worth-query milestone_9_9_authority_matrix
cargo test -p worth-topo milestone_9_9_graph_obligation_operator_closeout
cargo test -p worth-kernel milestone_9_9_graph_obligation_kernel_closeout
cargo fmt --check
```

The closeout also requires the targeted document/status audit to show that the
spec links this closeout, no 9.9 closeout surface still claims `Draft`, AI
guidance teaches Consumer Kit adoption as the ordinary downstream path, and
residue is named instead of hidden.
