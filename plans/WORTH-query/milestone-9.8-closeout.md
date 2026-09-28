# Milestone 9.8 Closeout: Consumer Kit Product Closure

> **Status:** Closed
>
> **Closure authority:** `support_report().consumer_kit_closure()`
>
> **Required suite:** `Milestone 9.8 Consumer Kit Hostile Certification Matrix`

Milestone 9.8 closes the downstream Consumer Kit as a Query product surface.
The closure is not claimed from API presence. It is derived from the support
profile, the hostile certification matrix, public documentation agreement, and
reference-consumer adoption residue.

The Consumer Kit is the ordinary downstream path for crates that need to prove
Query support posture without reimplementing WORTH Query internals.

## Closed Consumer Kit Families

- `evidence-report-kit`
- `hard-prohibition-registry`
- `boundary-audit`
- `support-snapshot`
- `support-pinning`
- `in-memory-test-backend`
- `consumer-residue-audit`
- `reference-consumer-adoption`

## Closure Rule

Milestone 9.8 reports `Closed` only when every required family publishes a
closed evidence row with a nonempty digest, documentation agrees with the
support/profile family set, the hostile certification matrix is closed, and
the generic consumer-residue audit plus reference-consumer adoption publish
zero Query-owned folklore residue. The `consumer-residue-audit` rows are backed
by typed certification evidence, not by source-text marker checks. The
reference-consumer residue digest includes the generic audit report identity and
source-inventory digest so closure changes when the audited downstream source
set changes.

## Reference Consumer Residue

`worth-kernel` construction adoption now publishes zero Query-owned residue for
covered report/digest folklore, Query-prohibition audit folklore, and
support-pinning folklore. Remaining construction digest helpers are defended as
worth-domain artifact identity rather than Query evidence identity.

The generic `consumer-residue-audit` family is the Query-owned authority for
proof-folklore cleanup. It covers local Query reports, local Query proofs, raw
support-row spelunking, support-matrix row searches, debug-derived proof
strings, delimiter-joined proof strings, and delimiter-formatted proof strings.
Consumers do not provide their own class registry, scanner, or replacement
matrix. The shipped report publishes typed findings, finding identities, report
identity, audited source paths, skipped non-Rust source count, and a
source-inventory digest.

The in-memory test backend is published as a consumer-kit family. For current
`worth-kernel` construction, backend adoption is classified by applicability:
the covered construction surfaces publish zero hand-implemented Query runtime
adapter traits and zero hand-fabricated mutation receipts.

## Defended Exclusion

Durable persisted kit archives remain Milestone 10/11 scope.

## Public DX

Downstream callers inspect closure through the ordinary support report:

```rust
let closure = WORTHQueryApplicationFacade::runtime_backed_default()
    .support_report()
    .consumer_kit_closure();
```

The closure artifact carries family rows, docs agreement, hostile certification,
reference-consumer residue, defended exclusions, and a canonical
`WORTHQueryEvidenceIdentity`.
