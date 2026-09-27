# Milestone 9.11 Closeout: Declarative Downstream Basis Authority And Consumer DX

## Status

Closed on 2026-07-12 on branch `ui` in commit `c4be42a7`.

Milestone 9.11 now has one Query-owned downstream authority product. A consumer
declares the guarantees and facts it needs, invokes the result-attached facade
method, and receives either one sealed authority or one typed non-admitted
outcome. Receipts, facts, source labels, and digests cannot be recombined as a
second authority path through the curated facade.

## Canonical Product Boundary

The closed public boundary consists of:

- `ProjectionAuthorityContract`
- result-attached `consume_projection_authority(...)`
- `ProjectionAuthorityOutcome`
- `WorthQueryConsumedProjectionAuthority`
- `ConsumedProjectionAuthorityDenial`
- `consumed_projection_authority_support_matrix()`
- `ProjectionAuthorityContractDocument` and the fail-closed terminal JSON
  loader

The authority is sealed and move-only. It binds the admitted basis, source
lineage, projection contract, settlement, consumption receipt, typed facts,
and structural counters from one canonical transition. Evidence and getters
remain borrowed or derived observation surfaces and cannot construct a
successor authority.

## Phase Closure

### Phase 1: Closure Contract

`downstream_authority_closure_contract()` freezes the required relationships,
deletion obligations, and support expectations. Consumer residue classification
includes independently pairable completion parts, local compatibility scans,
digest promotion, raw source re-entry, and direct internal Query imports.

### Phase 2: Canonical Authority

`WorthQueryConsumedProjectionAuthority` is produced only by the canonical
authority transition. Hostile tests cover cross-basis, cross-generation,
cross-contract, cross-source, cross-receipt, collision, replay, partial
settlement, and typed denial behavior.

### Phase 3: Declarative DX

All admitted ordinary source artifacts expose
`consume_projection_authority(...)`. Fluent and explicit paths share the same
transition, counters, evidence, and denial taxonomy. Terminal JSON persists the
declaration, not operational authority, and rejects unknown schemas or malformed
facts.

### Phase 4: Facade Convergence

The curated facade exposes the contract, outcome, authority, support,
inspection, counters, and certification surfaces. It no longer exposes
`CompletedProjectionFactConsumption` or `ProjectionFactConsumptionAttempt`;
compile-fail fixtures make that deletion mechanical.

### Phase 5: Consumer Cutover

Worth UI certification fixtures consume `ProjectionAuthorityOutcome` from real
Query-backed worlds. The measurement basis retains a typed evidence index, and
planning, allocation, scroll, portal, activation, and publication tests execute
without local Query-basis reconstruction.

### Phase 6: Certification And Deletion

`ConsumedProjectionAuthorityCertificationBundle` closes canonical admission,
deterministic replay, typed denial, authority-product support, and exact
complexity. Consumer Kit seals a
`WorthQueryDownstreamAuthorityDeletionReceipt` only after the required source
inventory has zero authority-capable residue. Its four rows match the closure
contract exactly:

- independently pairable completed parts
- consumer basis compatibility scans
- digest-to-authority promotion
- raw source identity re-entry

## Complexity Evidence

The authority transition is bounded by declared requirement width and consumed
fact width. Certification records exact counters across five axes:

- requirement width
- fact width
- unrelated Query workspace growth
- historical basis growth
- downstream consumer graph growth

Authority construction remains exactly one on admitted paths and zero on typed
denial paths. Unrelated growth does not change authority-admission counters.

## Documentation And DX Evidence

The ordinary product story is documented in this order:

1. ordinary fluent path
2. contract reference and terminal replay
3. typed denial and inspection
4. advanced lifecycle
5. migration history

The feature guide, downstream-runtime recipe, AI orientation, facade exports,
and compile golden are bound by `downstream_authority_docs_agreement`.

## Verification

The final tree passed:

- `cargo test -p worth-query`
  - 2,618 Query library tests
  - all integration, compile-pass, compile-fail, cross-crate certification, and
    doc-test targets
- `cargo test --manifest-path workspaces/worth-ui/Cargo.toml --workspace --all-features`
  - all Worth UI crates, runtime/integration suites, facade boundaries, and doc
    tests
- direct rustfmt checks for every changed non-UI-fixture Rust file
- `git diff --check`
- the 400-line cap for every changed Rust file; the largest touched file is 397
  lines

The repository-wide line-cap script was also run. It reports pre-existing
violations in untouched crates and untouched Query files. Milestone 9.11 adds
no line-cap violation; the global backlog remains a separate repository-wide
cleanup concern.

## Deferred Ownership

Store-backed source admission remains Milestone 10 scope. Durable authority
artifact reload remains Milestone 11 scope. Both must extend this
canonical authority product through typed support postures; neither may
reintroduce consumer-side pairing or digest promotion.
