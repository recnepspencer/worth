# Residency phase 2 implementation

Governing specification: [Milestone 2](milestone-2.md), D9 and Residency.

## Boundary review

Store's journal and authenticated checkpoint members own durable truth.
Relational owns readmission, sticky damage, counted residency, identity issuance,
and operation outcomes. An operation may load only through an explicit demand;
record access must neither do I/O nor translate cold storage into absence.

Phase 1 already supplies transaction Head demands, typed transaction/preparation
denials, envelope cells, paged symbol dictionaries, and a cold index image.
Preparation still warms a declared set at Full; root and mirror readers retain
infallible access paths. Historical catalog readers can reach undemanded envelope
bodies. Lineage member capture exists, but cold directory restore does not install
a serving lineage page directory. These are integration boundaries, not new
durable authorities.

The adversarial case is an invariant or graph traversal discovering a different
cold region after provisional work has issued identities. No provisional answer,
diagnostic, or publication may escape; identities from an abandoned attempt must
remain spent. Publication must consume carried resident custody and cannot load.
History reads must distinguish an absent commit from a failed journal read.

## Implementation order

1. Add the native execution storage failure and preserve it through World,
   Query, and bank-world exhaustive translations. Establish explicit resident
   read witnesses and demands for known read footprints.
2. Introduce miss-recording read views and bounded retry orchestration for
   preparation, visibility, and invariant execution. Retain accepted leases in
   the candidate; abandon diagnostics and speculative state between rounds.
   Cut publication and retention over to resident witnesses.
3. Demand indexed envelopes in bounded history orchestrators; replace durable
   catalog body retention with in-flight/head custody and readmission through
   the journal. Keep memory-only history resident.
4. Install and demand lineage pages by node range; audit symbol lookup and
   interning so cold buckets cannot admit duplicate text identities. Preserve
   load-free capture and untouched page reuse.
5. Exercise cold-source refusals and sticky damage for all five subjects,
   retry identity continuity, Head/Full type separation, retained observations,
   and cold/warm/reference equivalence through the home oracle.

Production responsibilities remain under `storage/residency`, `branch` read
views, `history/commit`, operation-specific access/orchestration modules, and
the owning runtime subsystems. No second storage lane or catch-all module is
introduced. Public failure vocabulary is exported through existing facades.
Eviction and Milestone 4 scan summaries remain outside this phase.

## Verification

Use `cargo test` for Relational lib/contracts, World, Bridge, Server, Query lib
and integration, bank world, and the Store home oracle. Use the CI clippy targets
and each touched workspace's formatting check. Run the dirty Rust line-cap
guard, boundary-check, and agent-context; regenerate context only with its tool.
Query and bank builds carry the fixed implementation build id and test debug=0.
Every executed lane must finish green; record root causes and corrections in
the reviewer report. No claim of phase completion precedes this evidence.

## Current cutover

The implemented slice adds Full-history probe rounds to legacy visibility
point, adjacency, and frontier reads, and explicit demands to their scans and
historical materialization. It carries storage failures through Query and bank
outcomes and counts slots from directory facts without warming payloads.

Phase 2 remains open: commit/merge and candidate-wide invariant probe rounds, accepted-candidate
lease carriage, resident pin/publication witnesses, fallible retention and
inspection, envelope catalog retirement and the remaining history
readers, lineage page serving, and the complete symbol/unique-field audit still
require implementation and their specified fault evidence. The visibility
slice's checks do not certify those unfinished boundaries.

The next history cutover makes bounded canonical patch reads take the caller's
request and demand the selected indexed envelope. Journal refusal travels as
the native execution storage failure through inbound completion reconstruction
and the public index-repair outcome. This does not yet retire admitted envelopes
or replace the remaining infallible history surfaces.

Storage statistics, partition statistics, chunk summaries and chunk diagnostics
now require the caller request and return typed native read failures. Their
computations accept a `ResidentPartitionEdition` issued only after the entire
mirror's Full demand succeeds; they retain that exact edition for all counts.

## Next audited boundary: invariant rounds

Invariant request construction already reads relation scopes, so the invariant
access probe includes construction and execution together. Committed,
enforcement and before-image sources share one miss ledger; an overlay hit does
not hide a cold before-image. The probe preserves touched-slot declarations and
overlay source selection. Provisional complexity telemetry uses a borrowed,
stack-local sink and settles only with a miss-free round. Work already spent
remains spent. This read boundary
does not issue candidate identities; candidate-wide abandonment and symbol
staging remain separate required cuts. The current Full-view substrate still
requires the specified Head-only preparation cutover before phase completion.
