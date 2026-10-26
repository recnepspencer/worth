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
lease carriage, resident pin/publication witnesses, envelope catalog retirement
and the remaining history
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


## Retention mirror custody

Retention plan inspection and reclamation now accept the caller request and
return the native typed execution/storage failure. They demand the whole mirror
before taking its writer, then seal the current edition as resident without
performing I/O under the mutation lock. Publication only inserts resident entries;
exclusive recovery is the sole cold-directory installer. A displaced edition is
redemanded before retrying acceptance. Traversal receives mutable resident
partition borrows, rather than looking the same partition up again for every slot.
Reclaimed identities are admitted after releasing the storage writer. Inspection
retains its existing lifecycle refresh behavior. Query's snapshot-count controls
read the owned handle count directly; its retention control preserves native
failure through the existing application-attempt denial.

This closes the explicit retention authority's cold entry path. General
inspection, replay pin admission and publication's carried resident witness are
still required; this paragraph does not certify those surfaces.


## Inspection custody

Current graph and connectivity summaries read the exact retained Full mirror
edition demanded by their operation. Neighborhood and structural inspection
also carry that resident edition into private record, adjacency and sidecar
reads. Pure-looking storage accessors perform no loads. Retention summaries and
per-record retention inspection now take the caller request and return the
native execution/storage failure; record absence remains a successful `None`,
and existing budget-degradation summaries retain their meaning. The cold fault
matrix exercises all six of these inspection doors, sticky failure, unchanged
journal position and no offered publication. Existing inspection budget and
substrate-acquisition courts remain green.


## Canonical patch batches

Patch stream reads require the caller request, select a finite batch from
index references and demand only those envelope cells. The default batch is
256 commits. An envelope refusal is `PatchStreamReadErrorClass::ReadRequest`
with the exact native execution/storage cause; it is never a retained-history
gap or an empty batch. Receipt lookup at an indexed position uses its index
entry directly and does not require the envelope. A genuine 17-commit
checkpoint fixture proves selected-batch journal reads, repeated equality,
sticky failure and unchanged publication/journal state. Subscriber continuity,
other envelope surfaces and envelope retirement remain required cutovers.

## Indexed receipts and demanded body surfaces

Immutable version receipts and ancestry traverse catalog index entries. They
neither require nor reconstruct envelopes. Version cardinality summaries,
individual/recent commit inspection and compiled artifacts now take the caller
request and demand only selected bodies. Recent inspection selects from the
complete position index, including cold entries; branch filtering uses indexed
parentage before body admission. Compiled artifact identity issuance follows
successful source admission. Merge planning owns its selected delta envelopes,
and policy base contexts carry the admitted body beside their record view;
source and target identity-only contexts need no envelope.

Query's mutation/index refresh reads a finite complete patch batch and preserves
native storage refusal together with any already observed committed receipt.
An incomplete batch is an explicit capacity refusal, never successful partial
index maintenance. Cold-source proofs cover version summaries, commit and
recent inspection, branch-filtered recent inspection and artifact compilation,
with readable index facts, sticky refusal and no journal/publication advance.
These cuts do not yet close aspect history, subscriber continuity, retained
export, envelope retirement or candidate-wide retry and symbol staging.

## Candidate round boundary review

The present proposal path simulates mutation with a detached identity allocator
and cloned interner, then interprets the same plan again during authoritative
preparation. A custom invariant can therefore observe identities that were never
reserved by the live owner. A copied symbol frontier can likewise issue an id
whose meaning differs from another concurrent candidate. Neither representation
is adequate custody for a retryable candidate.

The live identity allocator continues to own record reservations and monotonically
spent generations. The runtime symbol owner must similarly own reservations for
pending text, deduplicated across concurrent candidates. A pending symbol's bucket
must already be resident before issuance. Its meaning remains unpublished until
the exact branch cutover; dropping the last pending reservation removes its text
entry but never rewinds its issued frontier. The standalone interner remains a
meaning map; the runtime owns reservation lifetime and synchronization.

A round borrows the selected root through an explicit probe view. It owns one
mutation result, record reservations, pending symbol meaning, diagnostics and
provisional instrumentation. Validation judges that exact result. A miss drops
all of it before the operation orchestrator demands the recorded regions or
symbol buckets. Unique transaction mutation authority stays outside the loop.
Only an accepted round may move its evidence and leases into publication.

Implementation order:

1. Add runtime-owned pending symbol reservations and a candidate symbol binding;
   prove concurrent deduplication, abandoned-identity spending, and refusal to
   mint from an unadmitted bucket. Move existing symbol-table tests into their
   owned test module to retain the file cap.
2. Replace mutation's implicit field/aspect interning with explicit delta symbol
   admission. Keep text reads pure and keep cold loads in the round orchestrator.
3. Carry one applied draft and its live reservations from proposal validation
   through authoritative preparation. Remove detached allocation simulation and
   the second mutation interpretation. Publish pending meaning at the same
   successful cutover as its root.
4. Install candidate-wide bounded probe rounds for transactions, lowered
   strategies and merge preparation. Isolate diagnostic and instrumentation
   effects per round; preserve actual spent work. Accept only miss-free custody.
5. Complete Head-only draft/history layering and resident mirror publication
   witnesses. Use the existing authenticated history append frontiers; never
   reinterpret an empty resident suffix as complete history.

The proof matrix includes real cold failures before publication, custom invariant
observations across discarded rounds, concurrent candidates sharing text, aborts
between validation and publication, and retained observations across checkpoint
and reopen. Focused Native proofs precede the complete requested lanes. No new
Cargo features, cloned mutation authority or second publication lane is allowed.

## Applied draft and pending dictionary custody

The first three candidate-boundary steps are implemented. Proposal validation
and publication now share one applied mutation and the live record reservations
that produced it. Revision stamping explicitly admits its aspect and field
references into the candidate dictionary. The canonical grammar records those
references and requires the exact union with body references; recovery compares
the same evidence. Required region symbol discovery includes both field-stamp
key components.

Pending text reservations belong to the runtime owner, deduplicate equal text
across concurrent candidates and retain monotonically spent ids after abandonment.
Accepted previews keep their own dictionary custody. Only the exact canonical
dictionary publishes at successful branch cutover. Independently admitted field
text retains its existing explicit publication semantics and shares an already
pending id. Checkpoints preserve the symbol issuance frontier, including gaps.
Strategy lowering takes the actual caller request and returns preparation failures
by value; Query batch composition carries authored intents into its final owner
instead of dropping the pending dictionary behind normalized symbol ids.

The cold-symbol proof uses a genuine 601-record producer checkpoint, an actual
hash-bucket collision and the production cold source. Field-text admission,
plan preview, validation, candidate preparation and revision stamping preserve
the typed sticky storage refusal without journal movement or published meaning.
A healthy cold commit spends a fresh record identity after its discarded draft
and warms a bounded minority of dictionary members. Concurrent reservation,
abandonment, exact publication and checkpoint-frontier courts cover the owner.

Two prior test claims are deliberately retired by the authoritative milestone-2
decisions D7 and D9. Preview normalization no longer publishes global text:
tests now check its owned meaning, absence before cutover and exact meaning after
real publication. A refused already-applied draft no longer preserves issuance
frontiers byte-for-byte: the retirement court compares every other checkpoint
manifest fact exactly and asserts spent record/symbol frontiers independently.
No test is removed, ignored or weakened into accepting a storage failure.

Function scrutiny reports no scan errors. The added orchestration functions
retain their single named phase responsibility; the applied-draft function's
arguments are its explicit authority and input boundaries. Existing broad
publication/checkpoint orchestration remains within the file cap. This slice
does not claim candidate-wide probes, provisional counter/diagnostic settlement,
Head-only mutation, resident publication witnesses or envelope/lineage retirement.

The request court also names `PlanPreview` admission and each
`PreparationRound` consult. These are distinct from the later authoritative
`Draft` stage: a warm round adds one exact visit, while a cold retry starts a
fresh round visit. The existing stage-count court retains its exact complete
order and one visit per warm stage, extended with the newly named round.

## Transaction proposal rounds

Transaction validation now runs locality, plan preparation, one live applied
draft and all three invariant judgments inside the same bounded region round.
A borrowed selected-branch read carries the explicit probing source; owning
selection facts remain separate from record access. Invariant observations
borrow that source, including their speculative before-image. A cold region's
provisional conflict or invariant failure is discarded before its demand.
The accepted proposal carries its applied draft and the retained root's counted
leases into the existing candidate phases; those phases reuse its judgments.

Each region and symbol round owns provisional complexity counters and a separate
diagnostic store. Only a miss-free outcome settles those observations. Runtime
identity sequences, record reservations and actual request work retain their
live owners throughout. Schema reframing remains an explicit whole-root demand;
the mirror write demands the accepted structural touched set before handoff.
Plan preview uses the same explicit region view and overlay rollback, and
strategy lowering consumes that preview before constructing bulk metadata.

A genuine split checkpoint with two partitions warms only the mutation's
partition. The installed global uniqueness rule discovers the other partition
after application. Validation, candidate preparation and commit return the exact
sticky region failure without publishing meaning, diagnostics, provisional
scan counts or a journal position. The healthy cold path spends another applied
record identity than the warmed reference, stays within headroom and settles
one round of invariant evidence. These are production recovery/preparation
proofs, not a fabricated partition-access double.

This closes the transaction/validated-strategy proposal boundary at Full.
Independent merge preparation, Head-only mutation/history layering, complete
publication/pin witnesses and the remaining history/lineage cuts are still open.

The bulk naming audit removes one more copied-frontier issuance lane:
`plan_bulk_mutation_batch` is a pure description of already authored keys.
It never interns into a cloned dictionary. A caller needing naming-stable symbol
ids first prepares the transaction's fallible plan, which retains their meaning.
Strategy lowering follows that order. D9 retires the former test claim that a
pure bulk description may synthesize normalized ids. Its replacement proves no
issuance during description, unchanged locality/provenance, and exact owned
preview meaning without global publication after real normalization.
The hostile naming-admission fixture now prepares through that owner and proves
its initial naming is admitted before replacing the symbol with raw text;
otherwise its old replacement would be a no-op rather than a hostile input.

## Replay pin boundary review

Replay retention currently materializes the historical selected root, then edits
the mirror by bare record id. Those regions can have different cells after
reopen, so the historical read is not a mirror residency witness. Release also
rediscovers its pin set through an evictable cache. The runtime pin owner will
instead demand the selected mirror partitions before any pin effect and retain
that exact resident edition together with the derived pin state. Storage pin
edits will require a counted Full lease and a slot in that resident partition;
they will have no bare partition-id entry point. The move-only replay retention
entry owns this custody through release, so unpinning performs no loads and
does not depend on cache survival. These are runtime lifecycle types, not
cross-runtime meaning or a new platform authority lane.


The pin proofs also exposed a concurrency defect in the prior reference
protocol: eight simultaneous first retains each applied a physical pin set,
then overwrote one index entry. Removing an entry to decrement it similarly
opened an ownership gap. Resident registration now rechecks and installs under
the replay index lock; release decrements or unpins and removes under that same
lock. Cold demand, diagnostic emission and cache work remain outside it.
Retention fences are computed before this transition and before mirror writer
acquisition, preserving the index-before-mirror lock order. No fallible loading
occurs during pin effects.


Replay pin custody is implemented. Production recovery proofs retain an exact
fork source independently of the serving mirror, inject unread mirror damage,
and verify the typed sticky failure without retention installation or journal
advance. A successful retain survives visibility-cache eviction; final release
uses its owned pin set without reads. Eight concurrent retainers and releasers
share exactly one physical pin set. The failing concurrency proof observed
eight sets before the index transition fix; the product protocol was wrong.
Relational lib (1,879 passing, 31 existing ignored), Bridge (1,102), World (263)
and Server (12) pass. Scoped function review found only cohesive operation,
fixture and typed pin-coordinate advisories; no extra responsibility is hidden
in the resident custody or index transitions.
