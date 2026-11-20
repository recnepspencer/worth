# Installed Operation Certification Kit

## What This Feature Is

`worth-query-certification` is the cold, certification-only contract for
proving that an installed operation keeps the same meaning across admitted
providers and rejects generic authority attacks at the earliest boundary.

It owns:

- the eight domain-supplied semantic scenario families
- the complete provider-independent installed-operation journey vocabulary
- the generic hostile-attack registry
- exact denial-boundary and structural-counter evidence
- the two-provider semantic parity oracle

It does not own runtime installation and cannot mint Query authority.

## Audience Boundary

Ordinary code imports `worth-query-host`. Certification packages explicitly
allowed by the machine constitution may additionally import:

```rust
use worth_query_certification::facade::{
    certify_provider_pair,
    WorthQueryCertificationProvider,
    WorthQueryCertificationScenario,
    WorthQueryCertificationScenarioKind,
    WorthQueryCertificationSuite,
};
```

The certification crate itself depends only on the host and replay audience
facades. It does not depend on Query implementation modules. The boundary
checker rejects unlisted consumers and keeps replay cert-only.

## Domain Contribution

A downstream domain contributes a small complete suite:

```rust
let suite = WorthQueryCertificationSuite::complete([
    WorthQueryCertificationScenario::with_oracle(
        "geometry-workflow",
        WorthQueryCertificationScenarioKind::Workflow,
        [("result".to_owned(), "settled".to_owned())],
        expected_counters,
    )?,
    // replay, conditional node, correspondence, reversal, lineage,
    // dependency impact, and counter contract
])?;
```

Query owns the bounded public-facade journey and targeted runtime denials. A
domain does not reproduce those fixtures or build a product-specific evidence
ledger. Its provider adapter executes scenarios and projects results into
provider-neutral semantic facts and exact counters. Query derives and attaches
the journey checkpoints from the scenario family.

## Provider Parity

`certify_provider_pair` requires distinct provider identities, runs the same
domain suite against both, and compares each provider independently with the
domain-authored oracle:

- exact semantic facts
- exact structural counters

Query derives the generic journey requirements from the eight scenario
families and records them as requirements in the certification report, so
providers and domains do not reconstruct that matrix. This parity report does
not claim that returning semantic facts executed those checkpoints. Query's
real public-facade integration suite proves the journey itself. The aggregate
requirements cover
operation resolution, installation, single-root entry, graph participation,
multi-domain and multi-graph binding, workflow, re-execution, replay, reversal,
publication, lineage, promotion, support, execution, consumption, native
access, impact, sharing, lease, compatibility, invalidation, collection
windows, and collection patches.

`certify_hostile_provider` is the separate contract for a Query-owned
hostile-world adapter. The runner's substitute test proves canonical dispatch
and comparison sensitivity only;
production denial behavior remains the responsibility of Query's focused
runtime boundary suites. Domain provider adapters do not reproduce generic
authority-attack matrices.

## Static Enforcement

Ordinary integration tests and doctests compile the positive audience-facade
journey through its real package owners. Boundary-check snapshots the complete
nested `worth-query-host::installed` namespace, not only the top-level
`installed` module name.

Runtime-affine foreign, stale, cross-provider, lifecycle, replay, collection,
and correspondence failures remain typed runtime certification. They are not
misrepresented as generated compile tests.

## Related Docs

