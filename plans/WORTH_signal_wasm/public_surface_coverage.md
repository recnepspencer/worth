# Public Surface Coverage

Worth Signals has too much public API to protect with memory or a hand-written
sidebar. Documentation completeness therefore has two separate projections:

- public navigation is curated for humans;
- public surface coverage is exhaustive for maintainers and CI.

The two must never be collapsed. A complete sidebar would be unusable, while a
tasteful sidebar is not proof that every surface has an owner.

## Authority

The checked-in package declarations are the public API authority:

- `package/index.d.ts`
- `package/raw_surface.d.ts`
- `react/index.d.ts`

`pkg/package.json` is the authority for which npm entrypoints publish those
declarations. The census derives its entrypoint list from that manifest; there
is no second hand-maintained export list.

`package/product/documentation/public_surface_inventory.mjs` asks the
TypeScript compiler for every export reachable from those entrypoints. Each
inventory record includes its export name, declaration kind, source file,
direct member names, and normalized complete declaration signature.

The inventory is derived. Do not hand-edit or copy it into another source of
truth.

## Coverage Policy

`docs/metadata/public-surface-policy.json` assigns each declaration source to
exactly one documentation group. A group names:

- its support status;
- its guide status and exact-reference status;
- canonical human docs;
- implementation truth owners;
- executable evidence;
- the exact declaration count, direct-member count, and signature digest.

Source-prefix assignment makes the policy readable. The frozen counts and
digests prevent those prefixes from silently accepting new API. Adding a type,
method, property, or changed signature changes the group baseline and fails CI.

`referenceStatus: "incomplete"` is deliberately blunt. A useful guide may still
exist, but the current reference material does not yet make the complete
declaration family discoverable. Do not rename it to something friendlier. Fix
the docs.

## Semantic Contracts

Types cannot express every product promise. Durability, authority, deployment,
cancellation, replay, retention, and reconciliation posture live in
`docs/metadata/semantic-contracts.json`.

Every semantic contract has one guarantee, explicit non-guarantees, a canonical
document, an implementation truth owner, and executable evidence. This ledger
describes the contract; it does not replace the implementation or its tests as
truth.

## Working With A Surface Change

Run the complete census:

```bash
npm run docs:surface-report
```

Inspect one group:

```bash
npm run docs:surface-report -- resources
```

When the public API changes:

1. Read the declaration diff and the report for the affected group.
2. Decide whether the change is stable, mixed, compatibility-only, or an
   accidental export that should be removed.
3. Update the canonical guide or reference material.
4. Add or revise a semantic contract when the type signature cannot carry the
   important promise.
5. Add executable evidence for the real behavior.
6. Update the frozen baseline only after those decisions are complete.
7. Run `npm run test:documentation`.

Updating a digest without reviewing the documentation is equivalent to
clicking through a migration warning without reading it. The machine cannot
prevent bad judgment, but it can make that judgment visible in the diff.

## Using The Census For Documentation Phases

Before rewriting a domain, inspect its group report and classify every
declaration into one of three reader needs:

- task guidance: code a normal application developer writes;
- conceptual guidance: authority, lifecycle, and failure semantics;
- reference: exact types and less common operational surfaces.

The human pages may consolidate many declarations. Nothing needs one page per
type. The requirement is that every declaration remains inventoried, owned,
searchable, and deliberately covered.
