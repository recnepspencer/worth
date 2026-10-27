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


## Symbol page shape and routing review

Initial symbol capture bounds id pages but emits an entire hash bucket as one
member. Its later captures carry prior pages and append new pages, so bounding
the initial writer also bounds every new bucket member without reading any old
member. Runtime admission authenticates the frame and canonical dictionary but
does not compare decoded ids or hash membership with the selected directory.
A misplaced authenticated page could therefore corrupt negative lookup proof.

Bound every new symbol member to 256 entries. On explicit demand, validate
strict id order, nonempty bounded shape and the selected range or bucket before
importing any text. Cache compact decoded shape facts with the existing sticky
page outcome so a page shared by range and bucket directories is validated in
each context without another source read. Exercise a real producer checkpoint
with more than two pages of colliding texts, including late-page deduplication,
late-page faults and wrong authenticated routing descriptors. This changes
artifact admission and capture, not interning authority or issuance ownership.


The bounded symbol writer and routing validation are implemented. Seven focused
residency proofs pass, including 601 colliding spellings across bounded pages,
late-page reuse of the original id, and a sticky late-page refusal without
issuance. An authenticated wrong id range is rejected before import; a cached
body is revalidated under another directory selection without rereading bytes.
The complete Relational lib passes (1,883 tests; 31 existing ignored), as do
Relational lib clippy, formatting, dirty line caps and agent-context. The
initial routing test tried to read another owner's private `cold` field; it was
corrected to observe admitted vocabulary and construct the existing production
page owner directly for its cache-shape unit proof. No visibility was widened.
Scoped review keeps capture's metadata orchestration and its three existing
function advisories together; the new shape facts and cache transitions carry
one page-admission responsibility.


## Independent merge draft boundary review

Validated transactions carry one applied draft and three judgments, but the
independent merge context reaches draft preparation with no such scope. That
fallback clones directly from the selected owner and then judges invariants in
later phases, so a cold dependency can become a false absence. The fallback
must instead run scope derivation, one live applied draft and all three
judgments inside the same region/symbol probe round, with provisional counters
and diagnostics. An accepted scope must always contain an applied draft; remove
the optional unapplied scope state. Only after a miss-free round may its touched
mirror set be demanded and its evidence settled. The existing later phases
consume the carried judgments and applied draft. This extends the current Full
round boundary to independent merge drafts; Head-only mutation layering and
complete artifact/publication custody remain separate open requirements.


Independent merge drafts now use the shared region/symbol discovery protocol,
retain their accepted selected root and demand the touched mirror before any
accepted evidence settles. `PreparedAuthorityScope` always owns its applied
mutation and proposal identity; there is no unapplied fallback. Existing
invariant phases consume the accepted judgments. Real reopened branch/journal
proofs show a healthy merge adopts the feature record while preserving main's
records, and a cold region failure during public merge preparation remains
typed and sticky with no journal advance. Full-branch planning already warms
its selected regions, so those public proofs do not claim to force a later
region miss in the candidate draft. Transaction discovery proofs cover that
shared miss ledger; a complete candidate/publication boundary remains open.
Relational lib passes (1,885; 31 existing ignored), as do Relational lib clippy,
formatting, dirty line caps and agent-context. Scoped function review keeps the
merge round's one draft and its three coupled judgments together; the other
reported functions retain their existing authority/fixture responsibilities.


## Cached negative symbol lookup review

Following the routing change in `7f738629f5`, the no-I/O interning probe still
accepted any cached page outcome as proof that its hash bucket was resident.
A page first admitted by id range must also satisfy the bucket descriptor
before that probe can authorize a fresh spelling. Validate the cached shape in
`value_bucket_is_resident`, store a sticky typed damage outcome on mismatch,
and prove that a genuine authenticated page routed to the wrong bucket cannot
advance the live symbol issuance frontier. This closes a second context of the
same page-admission boundary; it adds no source reads or control door.


The no-I/O negative lookup now validates and, on mismatch, poisons its cached
bucket outcome. A genuine captured page is first admitted by id, then routed
under an adversarial bucket descriptor. The live pending-symbol operation
returns the typed damage without issuing an identity or importing a new
meaning, without a read, and with the same sticky failure on explicit demand.
Relational lib passes (1,886; 31 existing ignored), all three CI owner/service
clippy commands pass, and formatting, dirty caps, agent-context and scoped
function review pass. The sole function advisory is this complete adversarial
page-to-negative-probe proof, kept together so its authority and oracle remain
visible.


## D7 lineage continuity and external fixture members

D7 is explicit: "Commit, version and lineage frontiers resume one past the
largest durable value." It also says an identity never durable or acknowledged
may be lost on process death. An uncommitted prototype incorrectly required
abandoned lineage clocks to survive checkpoint/reopen. Its new manifest fields,
restore changes, validator and cancellation-oracle rewrite are withdrawn. The
original cancellation byte-equality court is restored exactly. This removes
only this slice's own uncommitted prototype; all prior task commits remain.

The uncommitted claims `abandoned_lineage_and_event_ids_are_not_reused_after_any_checkpoint_restore`,
`checkpoint_lineage_frontiers_must_cover_published_identities`,
`replay_of_an_inflight_candidate_preserves_a_later_abandoned_lineage_floor`, and
`cold_manifest_frontiers_are_checked_against_the_admitted_head_envelope` are
retired because they require metadata or cross-process guarantees D7 does not
specify. Their replacements exercise real discard, image/embedded/cold restore
and tail replay, require every published node/event identity to remain protected,
and require the specified one-past-durable floor. Live discard still never
rewinds a running allocator. No product identity mechanism is changed.

Those real cold tests exposed a valid fixture defect: the in-crate medium filed
external checkpoint headers but its `read_member` always returned
SourceUnavailable. It now persists actual New member bytes under the same
journal lock as the checkpoint and retains earlier content-addressed members.
Its read port enforces the requested limit. Verify this through external reopen,
ordinary publication, inflight tail replay and reads of older retained members.
This corrects the test medium, not the Store home or Relational's typed failures.


The corrected scope passes the full Relational lib (1,889; 31 existing ignored),
all three CI owner/service clippy commands, formatting, dirty caps and
agent-context. Scoped function review reports only the two cohesive end-to-end
D7 restore/replay proofs. The production checkpoint format, recovery counters
and original cancellation court are unchanged. The only implementation change
is the test medium's actual member storage and bounded read port.


## Lineage node page boundary review

Native capture already writes node-range members, but cold restore installs no
node read port. `for_record` reads an undemanded mirror partition and then asks
the empty live node map, conflating cold data with absence. Add a runtime-owned
node-page residency owner, installed from the accepted directory without reads.
Explicit point demand authenticates the frame, bounded ordered node range and
exact entity binding, shares concurrent loads and preserves sticky failures.
The operation retains a counted page lease while reading and returns owned
immutable node meaning. In-memory publication remains in the existing node
owner; no new allocation authority or issuance rule is introduced.

`for_record` will take the original execution request, demand its mirror region
at Head, then demand its selected node page. Update direct callers and propagate
storage denials through read orchestrators. Page import validates all identities
before updating the derived node indexes. Checkpoint capture continues to carry
unloaded pages and emit only new resident nodes. Prove real cold point reads,
source failures, sticky damage, bounded page warming and one concurrent load.
Lineage event paging, complete historical traversal and envelope-query bounds
remain required follow-on work in this same phase; this node boundary alone
will not be reported as phase completion.


The node cut also removes `RuntimeLineageNodeSource`: its only reader treated an
empty undemanded node map as authority. Point association now takes the original
request and retains a Head mirror witness plus a counted node-page lease.
Historical association demands Full selected-root regions. Graph comparison,
lineage aspect-history readers and retained-observation node projections return
the typed execution/storage refusal. Capture's deliberately resident-only node
snapshot is named for that contract and still carries unloaded members.

Retained entity/relation reads and record-history selection now demand their
exact retained-root partition before reading it. Their denial distinguishes a
foreign observation from a request/storage refusal. Origin-only commit
selection keeps a separate foreign-origin admission type; it acquires no new
storage failure. Bridge snapshot, continuity and replay adapters preserve the
native refusal. Continuity source/planning and Query granular projections carry
the existing request through the newly fallible boundary. Query uses its existing
native execution-denial translation, including StorageUnavailable.

The producer-written cold-node fixture is shared by node and retained-region
courts. Seven node doors check sticky page damage, and retained entity, relation,
history and lineage projections check sticky selected-region damage. A separate
point court fails the mirror Head. Failure advances neither the journal nor its
publication offers. A graph court may admit healthy pages before its damaged
page; the oracle requires no node from that damaged page and no further work on
sticky retry, rather than incorrectly requiring all healthy caches to stay
empty. Cold and concurrent point reads compare with the producer's node meaning,
keep minority-population headroom, and require one actual selected-page load.


Node/retained-read verification: Relational lib 1,894 passed (31 unchanged
ignored), Bridge 1,102, World 263 and Server 12 passed. Query certification's
application_graph target passed 365 (five unchanged ignored), and
granular_invalidation passed 15. All owner test targets compile; Query lib and
both affected certification targets check successfully. All three CI
owner/service clippy commands, formatting, dirty line caps, agent-context and
plain boundary-check passed. Function review found no additional responsibility
cut required: the new parser, cell owner, producer-written fixture and fault
courts each have one responsibility; other candidates carry existing coherent
orchestrations or only gain the original request parameter. The final whole
phase still requires its complete verification matrix after the remaining
implementation.


The final cap run found that request forwarding plus formatting grew Query's
bridge-backed backend from 394 to 403 lines. Its result was consumed after the
initial node commit; that commit is amended before continuing. Split validated
backend construction into the owning `bridge_backed/construction.rs` child,
with identical methods and visibility, rather than compressing formatting or
adding an exemption. Repeat Query compilation, formatting, caps and context
checks for the corrected cut. No test or product behavior is weakened.


## Head append boundary review

Phase 1's Head/Full leases correctly restrict read APIs, but its head member
removes every metadata row. Current payload is available through `extra`, while
an append cannot retire the previous current metadata version without loading
history. Phase 2 therefore needs a clean region-format cut: Head carries the
last metadata row per slot; History carries earlier rows. Full admission joins
them and verifies the unchanged complete content commitment.

The carried Merkle right spine already opens its last metadata value. Head
readmission must validate that exact last row, exact occupied-slot frontier
coverage and representable history indices before accepting it. Seeded head
capture retains those checked history commitments rather than treating the
current row as history index zero. Incremental append then retires that resident
last row at its carried offset and extends the same commitment. Prove this
against a separately recomputed Full mutation, with no history read before the
comparison. Keep historical access absent from the Head lease API.

Use region framing version 3 for the changed split; no compatibility variant is
retained. Strengthen the real producer fixture with relation endpoint history so
its historical-membership tamper court still removes actual earlier versions.
This is a required phase-2 change to phase 1's representation. Typed Head working
states, layered history capture and candidate/publication custody remain the
next integration work in this same phase.


The first complete lib run passed 1,890 cases and exposed eight honest failures:
the shared cold-source fixture corrupted only the old literal REG2 magic and
therefore stopped injecting region damage after the clean REG3 cut. Keep every
failure/no-effect/stickiness assertion and select the region framing family in
the injector. The product's current-format reader still accepts REG3 only.
Frontier readmission checks exhaustion of its u128 carried index, rather than
requiring an opaque cold prefix to fit the reader's address space.


The last-key binding court still proves that a finite relabelled position changes
the commitment. An exhausted u128::MAX position is now covered separately by
stronger structural rejection before next-position arithmetic; no claim is
retired. All 18 focused residency cases and the complete 1,898-case lib pass
(with 31 unchanged ignored) after restoring real fault injection.

The next current-read cut must not reinterpret a partial PartitionState as Full.
Preparation/invariant readers need a distinct current-partition interface that
only lends Head views. Owned mutation buffers can retain their resident append
suffix, while historical algorithms continue to require an actual Full witness.
Construction of a Head working state must retain the admitted leases, and it
must not implement a full-history access trait. Full replay/maintenance buffers
need their own proven construction path. These are storage residency/custody
states in the owning runtime, not new business authority markers.

A changed Head also needs append history capture without loading its cold base.
An immutable history layer can name the previous history member and serialize
only newly historical resident rows. Its base-member closure must be carried in
compact checkpoint metadata: Store's checkpoint_record.rs builds the new member
set from the capture's explicit members, and StoreColdSource resolves only
registered locators. Do not assume omitted old members remain readable. This
closure contains locators, not historical payloads. Full admission must bound
cumulative reads, reject cycles/unknown bases, join layers with the current head
and verify the complete original content commitment.

Candidate-wide rounds must cover artifacts as well as draft/invariant work.
Every discarded candidate spends fresh real identities and drops provisional
effects; accepted read leases travel with the candidate. Publication must consume
resident target entries and owned write custody, rather than retain a generic
cold-capable entry and merely assume its expects cannot run.


The corrected head cut passes the full Relational lib (1,898; 31 unchanged
ignored), all three CI owner/service clippy commands, formatting, dirty line
caps, agent-context and a fresh plain boundary-check. Function review keeps the
paired region codec, full readmission, canonical record commitment and real
producer fixture together: their flagged orchestrations each implement one
complete contract. No additional responsibility split is needed for this cut.


## Current-source interface boundary review

The current-source interface will lend RelationalRegionHeadView and current
arena views only; it will not return PartitionState or a historical column.
Full PartitionAccess will extend that current-source interface, keeping its
historical methods available only to Full readers. Source ids, touched-slot
facts and miss reporting belong to the common current-source contract rather
than parallel metadata authorities. Add a Head-only probe round implementing
only the current interface. Root/edition implementations must distinguish
Absent from Cold and explicitly demand/cache their root-admitted Head leases.
A shared raw load cannot bypass another root's serving admission.

Invariant state views should consume the current contract and latest metadata,
not index a historical column. Their selected-root/current-candidate owner
continues to determine the semantic basis. Historical visibility/reconstruction
keeps Full access. Current overlay reads must use the probing base, including
before-image reads; do not add a dummy Full implementation to a Head round.
After this read cut, Head working buffers and layered history capture can be
integrated into draft and candidate-wide preparation without presenting a
partial buffer as Full.


The current-source cut is implemented. Invariant observation, preparation,
sidecar, cardinality, uniqueness, aspect and adjacency readers consume only
CurrentPartitionAccess and typed current arenas. Before-images still select the
independent baseline source. Full readers retain PartitionAccess, which extends
the current contract; sparse row selection has one shared decision rule.
Head and Full probes share one inline miss ledger, discard/load loop and bounded
escalation policy. HeadReadRound cannot implement the Full source contract.

The real region court proves current metadata, aspects and adjacency work with
an unavailable historical member, while a Full demand returns its failure.
Related sources and a cold before-image converge independently; a provisional
Head absence cannot escape or mask sticky damage. The real recovery court now
has earlier committed versions and runs the public invariant orchestrator with
its mirror history unavailable: cold and warm results agree without reading
history; statistics demands Full and returns the same sticky typed damage with
no journal advance or publication. The full Relational lib passes 1,903 cases
with 31 unchanged ignored. The earlier focused Head family passed 40 cases.

The source-lattice compiler cut exposed one remaining Full getter in custom
applicability; the caller was migrated to the current interface. The source
split also made the state-view test file exceed 400 lines; enumeration-refusing
access fixtures now have their own semantic file. No assertion was weakened.
Function review retains the typed row selector's five explicit facts, existing
complete invariant evaluations and real producer/test orchestrations together.
The Head/Full discovery policy was consolidated instead of duplicating its
ledger and retry authority. All three CI owner/service clippy commands, a fresh
plain boundary-check, dirty line caps and agent-context pass for this cut.

The specification explicitly says Milestone 2 never evicts regions and a
retained root keeps its counted leases. Do not expand this phase into Milestone
4 release/eviction work. Head mutation buffers, history-prefix carry and typed
candidate/publication custody are the next required cut.


## Head write and history carry boundary review

WorkingState is a mutation buffer, not proof of complete history. Remove its
Full source implementation and make preparation's selected read and overlay
sources current-only. Keep historical algorithms on the actual Full interface;
do not add a conversion or dummy Full implementation to recover compilation.
Current mutation cloning belongs inside storage and must not expose a raw
partition through Head's read API. Keep the outer preparation probe Full until
cold-prefix custody and capture are implemented, so no partial buffer can be
published without its history basis during this cutover.

A mirror publication can combine untouched slots from its own edition with
changed slots from the selected branch. Therefore one partition-wide cold
history base is insufficient: cold history prefix custody must follow each
record's history through sparse cloning, adoption, suspension and generation
reuse. The owning arena's history columns are the semantic location; clearing
or replacing a history must clear or replace that prefix too. No identity may
move to another slot. History members can encode per-slot prefix references and
resident earlier rows; Head retains only current rows. Full readmission folds
only registered prefix members, bounds cumulative reads, rejects cycles and
verifies the complete content commitment. Capture carries the referenced
member closure explicitly so Store never relies on omitted old placement.

Region eviction and cache release remain Milestone 4 work, as specified. This
cut must carry counted read custody and genuine cold prefixes, rather than add
an eviction policy or a new authority lane.


The concrete prefix representation is an optional column on each RecordArena,
following metadata_history through persistent column cloning/adoption. A prefix
owns the original read-only cold-source port and the selected history member,
partition, schema carrier and physical source row. The checked append frontier
remains in the region's content commitment rather than importing that owner
into the arena substrate. At Head admission, install the prefix only after
current rows and frontiers authenticate against the original leaf.

Prefix custody follows the actual metadata rows: reset and generation reuse
preserve those rows and therefore preserve their prefix; materialization
suspension clears both. Slot adoption adopts both together, preserving the
source member's physical row even if the destination row differs. The frontier
includes the baseline current row, while the cold member contains only its
earlier rows. Rehashing a changed resident suffix retires/rewrites the baseline
row at its carried offset, then appends subsequent rows, including rows from a
reused generation.

Use per-slot wire prefix references plus resident earlier rows in the changed
history grammar, with a clean format cut. Different slots in the mirror may
name different selected-branch bases. Carry a compact registered dependency
closure in checkpoint metadata and include every reachable base as Carried;
Store's selected-members map is not an implicit archive of omitted members.
Full readmission can cache decoded members within one bounded demand, reject
cycles and unknown references, fold selected slot prefixes, and independently
recompute the complete content commitment. The Head lease/read view must still
have no historical accessor. Encoding custody may inspect resident suffixes
and prefix references without claiming Full serving custody.

Root construction should use the accepted current view's carried commitment
for its incremental update and derive a Head cell whenever prefixes remain.
Do not require a Full previous region merely to hash or account for a Head
change. Capture and later Full admission must both understand that state before
switching outer preparation and mirror touched-set demands from Full to Head.
Candidate-wide artifact discovery and effect-only publication custody remain
required in this same phase; this type cut is not their completion.


The mutation-source type cut now passes. WorkingState, preparation overlays and
SelectedRelationalBranchRead implement CurrentPartitionAccess only, and the
working partition map is storage-private. The negative compile-time court
protects all three plus HeadReadRound from gaining a Full implementation.
Intent admission, stale-target checks, touched-scope inference, lineage
finalization, descriptive touches and candidate output associations use current
views. Current arena methods lend current version/revision columns and creation
positions without exposing historical columns. Storage-owned mutation cloning
retains the previous clone/selection semantics. The empty StorageRead marker
and now-unused Full sparse selectors were removed rather than kept as aliases.
Touched relation scope uses the slot directory through extra_at, including
sparse slot layouts, rather than indexing a physical column by logical slot.

The relation-birth court selects the exact canonical creation version and its
unique native allocation/Created operation. Current birth metadata corroborates
it when creation is still current; historical birth metadata is not demanded
by preparation. Canonical structural birth and exposed endpoint operations must
still agree, and roster assertions never supply their own native proof. The
existing real-retention hostile-envelope court passes unchanged apart from its
source parameter type. The eventual fallible history cut must preserve indexed
envelope digest checks and typed read failures at these exact-version lookups.
The Full composite probe court retains its historical assertion through an
explicit test source built from actual Full-owned staged contents and the real
cold edition; current-only preparation overlays are not given a test-only Full
implementation. No assertion was weakened or claim retired.

The full Relational lib passes 1,903 tests (31 unchanged ignored), and all lib/
integration test targets compile. All three CI owner/service clippy commands,
a fresh plain boundary-check, dirty line caps and agent-context pass. Function
review keeps existing complete admission/member/touch courts together: their
current-source type narrowing does not add responsibilities or accounting.
One compile round exposed misplaced imports before inner module documentation;
the imports were moved below the documentation. The CI clippy rerun also hit
OS error 112 because the shared disk filled. Cargo's package-scoped cleaner,
explicitly pinned to this worktree's target, removed 21 GiB of regenerable
Relational artifacts; compilation, the full lib and all three clippy commands
were then rebuilt green. This was an environment failure, not a product or test
failure. An unused test import was removed and all test targets checked again.
Outer preparation remains Full pending the next cold-prefix/capture integration;
this prerequisite does not claim complete Head write or candidate custody.


## Reviewed phase-1 integration and known input admission

The reviewed milestone head 5ff19564e7 is merged in 6273ec42c1. The merge
preserves cell-local proven damage, retryable source/resource refusals,
per-runtime dictionary admission, root-authenticated kind projections,
destination-scoped member carry, cold fork pin clearing and resident pin
custody, together with both review and phase-2 tests.

The first integration lib run exposed a stale test capture policy: the Head
invariant fixture described an empty destination after cold recovery. Commit
5346365b52 supplies its actual member directory and read source; every Head,
Full, sticky-failure and no-I/O assertion remains unchanged.

The complete home oracle then exposed a product error in preparation. A known
cold input partition was omitted from the provisional working state; a draft
could issue symbols and records against that empty state before the recorded
miss discarded it. Retrying correctly spent fresh identities, but ordinary
cold fork writes consequently differed from the memory reference, making
later scripted identities stale. The process-kill fork court and the fork
commit/transition media sweeps caught the resulting artifact and endpoint
mismatches. No oracle assertion or canonical artifact was normalized away.

Known intent/footprint partitions are now demanded before draft identity
issuance, through one shared preparation guard used by preview, proposal and
independent merge preparation. Data-dependent reads retain probe/retry and
never rewind an issued identity. The new fault-free home regression reproduces
the original failure and passes all three oracle seeds with the guard, including
script continuation. The full Relational lib passes 1,920 tests (31 unchanged
ignored), including the post-apply miss court's spent-identity and single-round
telemetry assertions. The original fork process-kill court passes. Fresh
contracts (203 passed, three unchanged ignored), World (265 unit and 41
integration passed, three unchanged ignored, plus its example and doc court)
and the complete home oracle (35 passed, no ignored, 2,251.57 seconds) now pass
this correction. The required reviewed-phase-1 integration gate is green. Preparation still demands Full pending the
cold-prefix cut; this correction does not claim that remaining work complete.


### Cold-prefix construction boundary

The arena owns row selection, metadata suffixes, generation reuse and sparse
adoption. It should carry a fixed-size history-prefix reference alongside each
metadata history column. That reference identifies the source member and its
physical history row, partition and schema carrier, and retains the existing
read-only cold-source port. It contains no residency cell, runtime symbol owner
or schema execution authority; lower storage therefore does not import those
runtime mechanisms. Full readmission owns schema decoding, request limits and
symbol admission. Checkpoint encoding owns the persisted reference and explicit
member closure.

First install references only after Head's current rows and append frontiers
have passed the original leaf verification. Carry references through sparse
cloning and slot adoption, retain them through reset/reuse, clear them together
with suspended history, and initialize new/full-decoded histories without a
prefix. Include the fixed-size column in physical allocation accounting. A
partial history must refuse ordinary flat region capture until layered encoding
can preserve its base, rather than silently emit its resident suffix as Full.
Producer-written region tests must prove the reference survives row lifecycle
and that Full readmission removes it only after reconstructing complete history.
The outer preparation probes remain Full until layered capture, root construction
and publication all consume this representation. This construction step alone
cannot claim Head writes are complete.


The construction cut installs the fixed-size prefix column after Head leaf
verification. Its three producer-written courts pass: reset/reuse followed by
suspension; sparse adoption into a different physical row; and flat capture
refusal followed by complete Full readmission. The full lib passes 1,923 tests
with 31 unchanged ignored. Head writes are still gated on layered encoding and
materialization; no partial partition is published as Full by this cut.

The integration boundary check found BC7014 in the earlier known-input guard:
a direct check_request bypassed the named commit consult. A first correction
incorrectly added another PreparationRound consult and failed all four exact
stage-count courts. The final correction in ad604a965c removes the redundant
helper check: preview, validation and draft entry already consult their stages,
and every actual residency load still consults the original request. All four
courts pass unchanged in the full lib rerun. The product-side fix preserves the
existing phase contract rather than weakening its tests.

### Retained prefix placement boundary

StoreColdSource.select_members replaces its shared map after publication, and
its physical reader accepts only currently selected anchor members. Keeping an
Arc to that source does not preserve an older member's selected membership.
Current Full preparation happens to warm the prior root before replacing it;
Head writes must carry the older history basis explicitly instead. A retained
Head observation must remain promotable after a newer root has warmed to Full
and checkpointed a flat history image.

Before switching preparation to Head, establish owned placement custody for
live cold regions/prefixes (including forked readers) and preserve their needed
members at checkpoint publication. This is read-side placement retention, not
record pin authority or a second Store truth lane. Capture and publication must
retain destination-scoped verification and avoid payload loads. Review the
recovery-carrier-to-cell handoff and concurrent checkpoint selection so custody
has no acquisition gap. Do not assume a frozen source map or retain every past
member without a live custody basis. Layer dependency metadata should be shared
rather than copying a transitive list into every historical leaf; Full demand
must validate the actual reference edges and bound its cumulative work.


Construction verification also passes all three CI owner/service clippy commands,
owner formatting, dirty Rust line caps, whitespace, agent-context and a fresh
plain boundary-check. Function review retains each existing complete codec,
adoption/accounting orchestration and producer fixture in its responsibility;
the prefix adds no separate authority or allocation counter.


### Placement custody implementation plan

The next cut adds an explicit metadata-only retain_members operation to the
existing cold-source port and an owned, clonable RelationalDurableMemberCustody
carrier through the durability facade. It carries a provider-owned lifetime,
not byte admission or record pin authority. Every provider implements the new
contract explicitly. Immutable inline/fixture sources already retain their
addressed bytes for the source lifetime; forwarding fault ports delegate custody
to their source without injecting a payload read.

Store owns a locator-count registry under cold_source/member_custody.rs.
Acquisition verifies current selected membership without reading member payloads
and may return the existing storage cause. A publication read guard freezes
acquisition/release while checkpoint_record publishes and selects its anchor;
the member union carries each live locator from that destination's verified
current member table. Taking this guard before reading the prior anchor closes
the selection race. It grants no ordinary journal or branch publication door.
Recovery constructs all cold cells before the home is exposed to concurrent
checkpoint calls. Cell construction acquires backing-member custody and returns
a typed region failure on refusal; Head suffix references share independent
history-member custody so sparse adoption can outlive the source cell.

Tests must use actual Store anchored members, prove metadata retention performs
no payload read, preserve a carried member through replacement while custody
lives, release it when the final owner drops, and verify the selection guard
excludes custody mutation. Keep the existing cached-locator/no-membership court.
Relational tests must prove custody drops only after all prefix copies and cell
owners release it, and propagate acquisition refusal without journal effects.
Run Relational lib/contracts, Store lib and the focused home regression, World,
CI clippy, formatting, caps, boundary and context. The complete home oracle is
required again after Head writes and layered capture integrate this cut. Layer
closure custody will extend the same member set, never introduce a second port.


Placement custody is implemented through the existing read port, its owning
provider registry, cold cell construction and prefix lifetime. The member union
uses the destination's verified current locators and freezes custody mutation
until the anchor is published and selected. Owned custody also retains the
provider's physical session. The producer-backed Store court proves actual
checkpoint replacement, clone lifetime, no payload read during acquisition,
release after the last owner and physical-session lifetime; the reviewed foreign
carry and cached-locator courts remain unchanged and pass.

The first new native lifetime court incorrectly requested GraphSparseEntities
with no entity-slot set, which deliberately produces only entity shape. The
fixture therefore owned no entity history to retain. Supply its actual selected
slot; the lifetime assertions remain unchanged. Both new native courts pass,
including typed acquisition refusal and retry with no payload reads.

This cut passes Relational lib (1,925; 31 unchanged ignored), contracts (203;
three unchanged ignored), World (265 unit, 41 integration with three unchanged
ignored, plus example and doc), Store lib (five), and the complete home oracle
(35, no ignored, 1,993.95 seconds). All three CI owner/service clippy commands,
owner formatting, dirty caps, whitespace, plain boundary-check and agent-context
pass. Function review keeps the complete checkpoint publication boundary and
its end-to-end lifetime court together; existing codec and scope courts retain
their responsibilities. The outer preparation level remains Full until history
layering, mixed mirror commitments and typed publication custody integrate it.


## Per-record history opening boundary

A mirror can adopt slots from different selected roots while keeping its own
retention lifecycle. Its next commitment cannot use one previous partition's
history cache as the basis for every adopted slot. Each prefix must therefore
carry the original authenticated per-record append opening alongside its source
row. The existing construction's region-only opening is insufficient for that
next step; preparation remains Full until this is integrated.

Move only the serialized opening facts to the record-arena substrate facade;
keep Patricia reconstruction, hashing and verification in the content owner.
There is no equivalent frontier vocabulary in worth-foundational. Install the
opening in a prefix only after Head validates the current row, exact slot
coverage and original complete leaf. Keep it behind shared storage ownership so
sparse adoption preserves the same opening without cloning its sibling vector.
Account for the actual owner and sibling bytes in ordinary inventories and
unique/delta walks. Preserve the current wire fields and format in this data
placement cut; no compatibility re-export remains at the old owner path.

Producer-written tests must preserve the opening through reset/reuse and
physical-row adoption and prove it still names the baseline while a resident
suffix appends. Run the existing independent Full/Head digest and frontier
courts, full Relational lib, formatting, caps, boundary, context and CI clippy.
Mixed-prefix hashing and the layered wire/reader remain the immediately
following integration work, rather than treating the carried hash as Full data.

## Mixed-prefix commitment boundary and plan

The record commitment currently offsets a resident suffix using the destination
cache and the previous resident suffix length. This is valid only while the
prefix basis is unchanged. Sparse mirror adoption can replace that basis, and
materialization suspension deliberately removes it. Continuing the old tree in
either case retains unrelated history or attempts to edit opaque omitted rows.

Keep ordinary Full reconstruction independent: a complete capture refuses an
arena carrying omitted history. Add an explicit layered capture for already
admitted prefixes. In incremental capture, compare immutable prefix facts;
unchanged bases use the existing column delta, changed bases reconstruct the
selected record opening and hash its resident suffix, and removed bases start
from complete resident rows. Keep this decision and suffix hashing in a named
record-history module under the commitment owner. Invalid coverage is a typed
capture denial rather than an arithmetic panic. This adds no storage load.

Producer-written tests will combine independently captured Head histories and
compare the result with a separately materialized Full reference, including
replacement in both directions, physical-row changes, removal and continued
appends. Preserve the prior leaf-authentication and independent Full courts.
Run focused residency/content courts and full Relational lib, CI clippy,
formatting, caps, boundary-check and context. Layered wire, Full folding and
publication custody follow this hashing boundary; preparation remains Full.

The opening-carriage cut is committed as a4cb5d4fc4. The mixed-prefix cut now
passes the full Relational library (1,930; 31 unchanged ignored), all three CI
clippy commands and the four producer-backed mixed-history courts. The final
focused rerun also crosses from a cold prefix to complete rows and back before
appending again. Formatting, dirty caps and whitespace pass; boundary-check and
agent-context completed successfully during this batch. Function review keeps
the canonical arena update and complete producer-written proofs together; the
prefix-basis decision is now separate from record and adjacency hashing.

## Historical-kind carriage boundary and plan

Mixed-history hashing is committed as 3d952e5593. The next Head-write boundary
is historical adjacency truth. REG4 puts the root-authenticated historical
projection only in the history member. A current-only Head consequently cannot
preserve it while preparing a changed region. Rebuilding locally owned claims
from its one current row would discard earlier endpoints and generations.
Sparse mirror adoption also requires those owned claims to follow the adopted
record, rather than trusting the destination's historical cache.

Move immutable relation-projection facts to the storage partition facade while
leaving authentication and reconstruction in residency. REG5 will put the same
historical projection in Head and History, bound to the existing historical
kind commitment. Head verifies it, checks that every known current owned claim
is included, and rebuilds the historical buckets without reading old versions.
Full still discards supplied local claims and independently rederives them from
all decoded history. Prefixes carry shared owned projection facts for their
record; canonical capture combines those facts with resident suffix rows and
keeps the current peer-projection rule. No new read authority or pure accessor
load is introduced.

Tests must compare unchanged and mutated Head historical kinds with separately
loaded Full data, including adoption from a separately captured history and generation
reuse. Corrupting the Head historical projection must be a sticky Region failure
without a History read. Preserve the reviewed owned-history forgery court by
making its supplied Head and History agree on the forgery, admitting Head first,
then proving Full independently refuses the false owned claim. Run residency
and full lib courts, contracts, the home oracle, CI clippy and enforcement after
the wire cut. Layered capture and typed publication custody remain required.

## Layered region capture boundary

The native checkpoint member list is the complete publication membership;
Store's selection replaces that directory. Placement custody preserves live
references, but cannot substitute for declaring a layer's dependencies in the
native artifact (embedded transport has to carry them too). Carry eligibility
must include every required base in the destination's current directory.

The next format will carry a history row as an optional base reference
`{member, partition, schema_digest, physical_row}` plus its resident earlier
suffix. Head admission continues to reference the selected history member and
its current append opening, including when that member is itself layered.
Capture removes only the current suffix row into Head; Full demand folds the
base's earlier rows and all resident earlier suffixes, then appends Head's
current row and independently checks content and historical-kind commitments.
Full materialization must not accept a carried digest as reconstructed data.

Use one eager, sorted direct-dependency directory for history members, with
entries only for layered members. Reject duplicate/unordered edges, cycles and
unreachable entries before constructing cells. Cold cells and prefixes share
that immutable directory by Arc; there is no recursive chain of owning runtime
nodes and no transitive vector copied into every record. Each closure traversal
uses an iterative work stack. The cold cell and adopted prefix retain the
metadata custody of the required closure through the existing port. A decoded
history member's actual direct references must exactly match its admitted
directory entry, including the flat/no-dependencies case.

Region capture returns its pruned dependency closure and explicitly carried
base members. A checkpoint binding owns the leaf and that dependency directory
together, so acknowledging a layered encoding cannot lose its closure on the
next unchanged capture. Native capture merges these facts canonically and
rejects conflicting declarations for the same content address. A foreign
destination may encode complete resident data; an omitted foreign basis cannot
be erased or loaded by capture.

Full folding caches decoded members by locator and checks each edge's partition,
schema and row facts even on a cache hit. It walks row bases iteratively, clones
each emitted row once, and bounds unique input bytes and expanded output under
the original request. A member cache must not retain a fully folded vector for
every intermediate layer: that would make a linear history quadratic. Frame,
digest, directory and final-content failures retain the current region subject;
symbol admission failures retain their symbol subject. This stage must also
cover resident Head cells with an isolated readmission dictionary; the live
runtime's pending symbol meanings must never become published by reconstruction.

Required proof includes multiple capture/reopen layers, mixed row bases,
destination membership, malformed edges and cyclic or expanding graphs,
original-request cancellation, sticky versus retryable refusal and retained
Head observations promoted after later checkpoints. Head preparation stays
disabled until the layered writer, bounded Full reader and publication custody
are integrated together.

The dependency directory lives as immutable member facts under durability data,
exported through that facade. One constructor checks canonical order, reachability
and cycles. Manifest admission carries that checked directory with its selected
heads, eliminating the prior repeated manifest validation in cold recovery.
Cells and prefixes share the directory and retain their closure through the
existing custody port. CHK10 adds the directory; the layered row format will
require REG6. Checkpoint selection and each recorded region leaf must carry the
same directory facts used by publication.

D9's requirement that writes append at Head deliberately retires the former
flat-capture refusal claim once layered encoding is installed. Its existing
court will become a no-load layered-capture/base-custody court and keep its
independent Full-promotion and flat-recapture checks; no test is deleted or
ignored. The Full verifier will continue to require complete reconstructed rows.

The new cached-guard court exposed a product defect: after a valid Head was
cached, a Full demand could prove its named History member missing, but a fresh
Head guard returned success from the cached lease. The full lib run confirmed
this single failure (1,933 passed; one failed; 31 unchanged ignored). Already
borrowed immutable views must remain usable, while fresh guards must propagate
the sticky Region refusal and probes must record a miss instead of accepting a
new damaged view. Store the terminal failure once on the cell and retain the
state's Damaged marker; this exposes the same failure to cached guards without
adding a lock to every healthy warm view or creating a second failure authority.
Keep foreign failures, cancellation and allocation failures retryable as before.

REG5 and the cached-guard correction now pass all 42 residency courts and the
full library (1,934; 31 unchanged ignored), including the retained immutable view,
the fresh guard refusal, prefix-owned kind reconstruction and independent Full
forgery refusal. All three CI clippy commands, formatting, dirty caps and
whitespace pass after the correction. The earlier contracts binary passes 203
tests (three unchanged ignored); its compile-time courts also compiled the REG5
sources. Fresh contracts, World and the home oracle remain required for this
format cut. Boundary and context passed before the cached-guard correction; the
post-correction run is in progress. No outer preparation demand has changed.

The post-correction boundary and context runs both passed. Fresh contracts pass
203 tests (three unchanged ignored), and Store lib passes five. World completed
its ordinary cases but its UI lane saw an overly wide private re-export from
the in-progress directory edit (`RecordedRegionCheckpoint`, E0365); the private
import fixes that product visibility error. Its fixtures and expected errors
remain unchanged, and World must be rerun after native verification stabilizes.
The sequential runner stopped there, so that run did not start the home oracle.

CHK10/REG6 now encode per-row bases and resident earlier suffixes. Base references
also carry their original authenticated historical-kind digest, so each ancestor
projection is checked without loading its old Head. Supplied historical caches
are checked against that projection before their rows enter the fold. Four-layer
capture/reopen, Head append versus independent Full rows, acknowledgement/carry,
ancestor corruption and aggregate-input refusal courts pass in the 45-test
residency filter. The full lib run also includes iterative directory admission,
undeclared-reference refusal and expanded-output bounds. Resident Head mutation
and candidate publication custody are still not integrated; preparation remains
Full and this format work is not phase-2 completion.

The completed CHK10/REG6 owner run passes 1,942 library tests (31 existing
ignored), including the directory and expansion courts; the focused residency
run passes 47. All three CI clippy commands, formatting, dirty line caps and
whitespace pass. The first expansion fixture was too small to amplify beyond
its reference descriptors; producer-authored history now establishes the
amplification precondition independently and the refusal assertion is unchanged.
The test file exceeded 400 lines after extending its producer fixture; moving
that fixture into its named region-fixture module restores the cap without
removing tests. Fresh contracts, World, Store and home lanes are running serially;
no completion claim is made for their pending results.

### Resident Head admission and explicit Full reconstruction

Boundary: a changed Head owns current rows and resident suffixes; its prefixes
own original member addresses, source ports and authenticated openings. These
facts cannot grant a Full view. Schema plans come from the retained root schema
authority. Reconstruction must use an isolated dictionary that retains the
source page directory, so page admission cannot publish pending runtime meanings.
It must fold each row's own prefix (including mixed sources), preserve slot and
current adjacency facts, reconstruct local historical kinds from complete rows,
and independently compare the resulting content commitment. The existing cell
load boundary owns request checks, retry classification, serialization and sticky
Region damage; symbol failures stay under Symbols.

Plan: add a resident-Head constructor and owned schema/dictionary readmission
context under residency. Share the bounded history folder with cold readmission;
never encode a temporary member or invent a cold locator to promote owned Head.
Full demand consumes the Head image and its context, validates complete rows and
publishes a separate immutable Full region. Keep accepted Head views intact.
Prove no-load construction/capture, independently appended Full equivalence,
retryable limit refusal and sticky damaged-base refusal. Wire root/mirror
construction and candidate custody in the following integrated cutover; until
then outer preparation remains Full.

Incremental root assembly currently borrows `PartitionState` through Full
retention in both merge and commitment capture. Merely switching the outer
guard would panic there and could mislabel a partial arena as Full. Move
assembly, history commitment and level selection into the storage residency
owner. It consumes an already accepted immutable region, the sealed overlay
and journal, schema authority and the candidate's existing RuntimeSymbolTable
binding (which already preserves cold pages and unpublished meanings). Root
construction receives prepared retention and metadata only. Initial complete
readmission remains an independently complete capture; a partial initial input
must still be refused. Slot-count aggregation uses eager region facts. Keep
existing allocation deltas, lifecycle deltas and identity ordering honest.

The per-runtime symbol wrapper's CurrentPartitionAccess implementation still
used its Full getter and inherited a default Head probe that classified a cold
entry as Absent. The existing outer Full round masked that defect. Cut it over
to explicit Head probe/demand using the same runtime dictionary admission
identity as Full. A fork court must read a shared Head in both runtime orders
while its named history member is missing, and require each runtime to admit
its own region dictionary closure. The court checks cold-to-resident Head
probing independently in both fork orders and materializes the expected record;
text page faults remain a separate matrix obligation. No second admission
registry is introduced.

### Mirror publication boundary findings

The mirror's current owner is `StorageSubsystem` and its read surface is one
immutable `PartitionEdition` behind `RuntimeOwnedState`. Edition acquisition
clones an Arc, and mutation copies only observed spines and regions. Existing
publication plans retain ids and raw overlays, not resident mirror targets.
`MirrorResidencyDemandBinding::demand` currently discards its admitted edition.
Publication runs after durable append and reacquires the writer for each delta;
the existing strategy then assumes Full retention or removes an entry and
extracts its full partition. These are cold-capable representations at a point
where refusal is too late.

Pin adjustments and retention change current mirror metadata under that same
writer between preparation and publication. Preparing replacement partitions
from an earlier edition and overwriting the mirror would lose those changes.
Holding a mirror write guard over journal I/O would also break the current
reader/settler lock boundary. The replacement needs stable, resident publication
targets whose current contents are edited under the existing mirror writer,
while old editions retain their own immutable views. New-partition targets
must remain candidate-private until publication; admission must not insert a
visible empty partition merely to manufacture a stable target.

A candidate holds a revocable payload/control and obtains a live owner operation
at publication, rather than holding that operation throughout its lifetime.
Before selecting target invalidation rules, verify recovery/configuration
revocation and the exact branch-cutover handoff. Do not infer that a retained
candidate prevents recovery. A stable target must be per-runtime across fork,
with shared immutable cell payloads and separately owned pin metadata. The
physical residency cell remains the sole payload admission/cache owner.

Resident Head/root assembly now passes the full owner compile check, all three
CI clippy commands, formatting, dirty caps, boundary and agent-context. The
clippy run caught a nested-if style violation in the new retained capture
accessor; the corrected let-chain passes. Removing the unused production Full
convenience accessor also exposed two old unit-test calls; they now borrow the
same accepted Full lease directly, preserving every assertion. Native execution
for the new courts is queued after the already running serial workspace lanes.
The format contracts run passes 203 tests with three existing ignores; World
is progressing through its UI court. These runs do not close phase 2.

Root completeness inspection still skipped Head payloads because it asked only
for an optional Full lease. A newly owned Head is resident content too: inspect
its current/suffix rows under its authenticated prefix openings without loading
a base, while cold cells continue to contribute their admitted leaf facts.
Unify content-capture denials under the storage owner and propagate known cell
damage rather than panicking or skipping it. Reclaimable resident allocation
estimates likewise must include Head arenas. Test against separately completed
history after a Head append, and preserve the damaged-cell refusal.

Content inspection uses the cell's existing isolated readmission dictionary
when it has one; that checks storage meaning without requiring an unrelated
runtime's dictionary to have admitted the shared cell. It grants no serving
authority and publishes no symbols. The owner compile check and three CI clippy
commands pass; native execution remains in the serial queue.

### Typed root-preparation refusal handoff

The root now returns a typed StorageUnavailable refusal, but both publication
preparation runtime implementations format every capture denial into String.
The caller consequently turns storage damage into a bundle-assembly error, and
metadata-only tail replay calls it ReplayFailure. This is a lossy pre-effect
boundary even though ordinary preparation still has an outer Full guard.

Introduce one publication-preparation denial vocabulary for semantic invalidity
and storage unavailability. Preserve existing semantic details, carry the exact
storage failure from root capture, map it to CommitPreparationReason with the
original branch basis, and preserve ColdStorageUnavailable during tail replay.
No conversion from the denial to String may silently erase a storage failure.
Test the source-to-consumer refusal mapping with a producer-created branch basis
and retain the existing invalid-root/publication courts. Position reservation
remains downstream and no new side effect is introduced.

### Full custody ownership transfer

Resident publication targets will need to consume accepted data, not rediscover
a Full state through an optional retention. The existing lease implements Drop
itself, preventing safe field moves from an accepted Full lease. Move the same
counting lifetime into a private cell-custody field; the public-in-crate typed
lease remains the counted view. A Full-only consuming method can then release
cell custody and move an unobserved region, or copy its structurally shared
partition when another observation remains. No unsafe code, placeholder cell,
load, authority promotion, new feature or alternate residency mechanism is
needed.

Use the consuming handoff in embedded checkpoint expansion, after the same Full
demand and root-qualified kind admission. Kind authority checks only the
demanded cell's bound identity, so consuming completed cells does not weaken
another admission. Build independent kind images from the admitted partitions
then move those partitions into their checkpoint images, removing duplicate
partition clones. Preserve all existing concurrent lease and old-observation
courts and add a consuming-custody observation/refcount court. Publication's
current raw-retention extraction remains a separate required cutover.

### Symbol admission follows the requested view level

A Head lease may reference a physically Full immutable region. The current
collector walks all metadata history even for that Head operation, while the
cache key identifies only the region allocation. Narrowing the collector alone
would let a Head mark falsely satisfy later Full symbol admission.

Keep the one runtime admission registry and add Head/Full closure level to its
immutable-view identity. Head admission collects current columns and extras,
Full also collects historical aspect states. A successful Full admission records
its covered Head identity; Head never records Full. Probe predicates use the
requested identity. Keep fork dictionaries independently admitted and retain
Weak identities to prevent allocation-address reuse. Prove the subset and cache
behavior through public field-text admission, real committed symbolic fields,
checkpoint reopen and a fork taken before the shared region is fully warmed.
An unavailable history-only symbol page must refuse Full with its own sticky
Symbols failure while Head continues to return current truth. Use headroom for
the admitted symbol population, not exact page counts.

The identity names the immutable logical cell version and requested closure,
not a transient decoded-region allocation. Head-to-Full promotion replaces
the physical region but preserves every current column; invalidating the Head
key there would create repeated misses for an unchanged current view. A Weak
cell identity prevents address reuse and forces copy-on-write rather than
in-place modification while a dictionary admission can name that version.
Full coverage records only the safe downward Head identity.

Remove the cell-birth shortcut from dictionary admission: copied resident data
can still reference cold dictionary pages in the demanding runtime. That
runtime's existing page authority decides whether admission is needed. A
memory-only dictionary remains the complete no-cold-source fast path, while a
paged dictionary records each requested cell/closure even for newly resident
cells. This also prevents a copied serving cell from bypassing the per-runtime
admission established by the phase-1 review.

### Mirror dictionary admission boundary

Visibility point and adjacency reads probe a bare PartitionEdition; scans,
statistics, retention and inspection demand its Full cells without admitting
text into the demanding runtime. The isolated cell dictionary authenticates
storage bytes but cannot grant that runtime symbol identity. A fork sharing a
warmed physical region can therefore return tokens whose meanings its own
paged dictionary has never admitted. Root reads already use the runtime's one
region/closure registry; the mirror must consume that same authority.

Add a borrowed symbol-admitted edition carrier under storage/partition_edition.
Its probes report a miss until both the requested physical view and the runtime
symbol closure are admitted. Explicit demand performs the existing region load
then the existing dictionary admission, preserving the original request and
subject. No new cache or I/O-bearing accessor is introduced. Full demand_all,
touched-set mirror preparation and retention writer admission require the
runtime dictionary explicitly. Cut visibility's probe orchestration and declared
partition/scans, statistics and inspection over together. Test public historical
visibility, declared scans, statistics, inspection and retention against a shared
Full region and a damaged history-only symbol page, retaining sticky refusal,
Head availability and unchanged journal checks. Memory-only execution remains
the existing no-cold-source fast path. Candidate publication and lineage/event
paging remain subsequent work, not claims of this slice.

The mirror dictionary cutover passes the four cold-symbol courts (including
all 27 healthy/damaged public read doors), the full native library (1,951
passed, 31 existing ignores), all three CI clippy commands, formatting and
dirty line caps. Boundary-check and agent-context also return zero. Function
QA found no scan errors: the read-door adapter is a closed typed operation
table, existing read/retention algorithms keep their named responsibilities,
and the fixture follows real admission, publication, capture and reopen.

Two new-test failures were corrected honestly. The first compile used the
fixture filename rather than its declared `view_levels` module name; fixing
the explicit import removed the cascading wrong-fixture field errors. The
first executed matrix expected the mirror to retain a history-only value after
the producer had released its earlier snapshot. That fixture was wrong: retain
the real produced snapshot across the update and assert that mirror metadata
history actually contains multiple versions before checkpointing. Every
original failure assertion is retained and the corrected matrix passes.

### Resident mirror publication targets

The main mirror currently classifies and rediscovers its publication targets
only during settlement, after the canonical cutover. Full demand makes its raw
retention extraction succeed by convention, but the candidate carries no typed
target. Pin and retention writes between preparation and settlement must remain
visible: copying an earlier partition into the candidate and later replacing
the mirror would overwrite their metadata. A mirror lock cannot span journal
I/O or consumer callbacks.

Use stable per-runtime resident targets, separate from immutable reader
editions. Storage owns a current map and its immutable edition; its existing
writer is the only target editor. Each target contains a counted accepted
resident lease, never a cold retention. A candidate retains the target, and
settlement edits its current contents under the existing mirror writer. A new
partition target stays private in the prepared publication until installation.
Recovery and runtime fork create their own target spines; immutable region
cells remain shared. Publication's owner admission and configuration/branch
checks remain before effect, so a revoked candidate cannot edit an old spine.

The cached edition must preserve O(1) reader acquisition and existing COW
semantics. Before editing a target, detach only that cached entry, preserving
external editions. Consume its accepted lease into a writer-owned edit and
restore a new immutable resident cell on writer release. The target's temporary
editing state is guarded by that same exclusive writer, never a cold state or
a loader. An unobserved payload moves; an observed payload copies. Preserve
fork pin detachment and the existing copy-lane accounting.

First cut Full publication over to this target protocol, with preparation
producing the entire effect plan before position reservation. Remove effect-
time target classification, missing/resident expectations and raw retention
extraction from publication. Keep a single mirror writer for pinning,
retention and publication. Prove preservation of intervening pin changes,
immutable old editions, private new targets, disposal and stale/recovered
candidate rejection through the existing owner courts plus focused target
courts. Run native library/contracts and owner CI clippy before extending the
same protocol to owned Head edits. Head pin/current-metadata access and
post-publication retention must then be cut over before enabling Head
preparation; a Head payload must never be labeled Full for convenience.

The Full target cutover now carries a PreparedMirrorPublication through the
candidate and completion. Existing targets retain the mirror's latest accepted
Full lease; new targets borrow the prepared root's Full witness and stay private.
The storage effect interpreter performs no target classification or demand.
CurrentMirror explicitly owns awaiting directory entries and resident targets,
with a derived immutable reader edition. Full edits consume a lease and freeze
back into a new immutable cell. Retention entry reads remain lazy and do not
copy until an edit; accepted Full entry edits are now infallible. The former raw
retention extraction, raw cell mutation and cold-backing edit special cases are
removed. Full custody still owns the member-retention guard through Drop.

The first target check caught an attempted private cell-field read; obtain the
partition id from the accepted Full view instead of widening cell visibility.
The first native run then exposed a real new product loop: the awaiting target
directory and reader edition had separate retention admission caches. The
declared demand filled the edition's root-qualified cache, while target minting
asked the copied directory cache for a witness and kept refusing. Stop that
scoped run after diagnosis and transfer the guarded edition's accepted witness.
Do not promote a raw physically Full cold cell: phase 1 deliberately requires
root-qualified serving admission. The existing 27-door symbol matrix is the
regression. The corrected four symbol courts, two new public preparation/
publication courts and full native library pass (1,953 passed, 31 existing
ignores). Cleaning up the now-infallible inner retention edit also exposed two
stale `?` calls in its sparse-slot court; remove those operators, preserving all
assertions. The corrected native library passes again. Contracts, CI clippy and
the repository enforcers are running against the stable source before the next
cutover.

### Head write cutover boundary

The remaining Full dependencies are concrete. Preparation's outer proposal,
preview and merge rounds still probe Full even though their selected read is
current-only. Mirror targets and editors currently accept only Full. Settlement
then reconciles current lifecycle through a Full mirror accessor and invokes
full-history trimming without an execution request. Public snapshot inspection
also resolves a materialized snapshot behind a pure Option-returning API.
Enabling Head only at the draft loop would leave these later readers dishonest.

Extend the target protocol with separately typed owned Head edits, carrying
schema and isolated dictionary readmission context. A Head edit preserves its
authenticated cold row prefixes and resident suffix, never fabricates Full, and
freezes a new Head version. Full promotion remains an explicit fallible demand;
its accepted edition witness upgrades the matching current target. Current pin
and lifecycle operations use current-only edit views. Historical retention and
replay pin custody retain their Full witnesses. New partition/adoption targets
must accept the level actually proven by root assembly.

D9 requires Head writes and Full retention. Retire settlement's implicit
full-history trim uniformly, including the memory reference, and perform that
work in the explicit Full retention pass. Choosing whether to trim from the
current residency temperature would make history/statistics depend on warming.
The existing touched-record automatic-trimming claim is retired by this D9
cutover; keep its retention assertions and exercise the guarded pass explicitly.
Extend that pass to trim live record history under the same retained-version
fence before reclamation. Keep current lifecycle reconciliation in settlement,
using the published resident targets, without history reads or loads.

Snapshot inspection must take the original request, declare Full root demand
for the selected observation, and return the existing typed execution failure.
Its identity-only consumers should use root/binding facts rather than building
snapshot record materialization merely to validate a handle. Audit every caller
and propagate the failure through World, Query and the bank world. Only after
these handoffs are closed should proposal/preview/merge discovery change to
Head. Add a type-level negative history boundary with a valid Full counterpart,
actual cold-history failure/locality courts, and retained-observation agreement.
Candidate-wide artifact/root/custody probing remains a required audit beyond
changing those three outer round calls.

The stable target slice passes all 1,953 native tests (31 existing ignores),
203 contracts (three existing ignores), all three CI clippy lanes, formatting,
dirty caps, boundary-check and agent-context. The final small custody cleanup
hoists the guarded edition outside the touched-set loop and checks the original
request even for an empty publication. Native library and all three clippy lanes
pass again after that cleanup. These results establish the Full target protocol;
they do not claim Head preparation or the remaining history/lineage work.

The Full retention pass now trims live history under its accepted whole-mirror
Full entries and the existing visibility/branch fence, then performs record
reclamation. Commit settlement reconciles only current lifecycle. Split those
mutation responsibilities under storage/authority/retention. The publication
timing field is renamed from history trimming to retention reconciliation so
it describes the work still performed; no old alias remains.

D9 deliberately retires the automatic touched-record history-trimming claim.
The existing complexity court keeps its final history and trimming assertions,
asserts publication leaves older versions present, then invokes the explicit
Full pass. The two real producer-continuity courts that previously called the
raw trim method now invoke that same public pass. Their assertions that native
birth metadata is actually gone are unchanged and pass, preserving their
canonical creation-evidence claim. No test is deleted or ignored.

The initial check found two missing imports after responsibility splitting and
two stale direct trim callers. Correct the imports and move those callers to
the real Full pass rather than restoring a raw compatibility method. The full
native suite passes 1,953 tests (31 existing ignores), all three CI clippy lanes
pass, and formatting/dirty caps pass. Boundary/context verification is running.

### Snapshot inspection demand boundary

The public inspect_snapshot operation calls resolve_snapshot_inspection, which
builds snapshot state and traverses exact Full partitions behind Option. Its
inspection facade wrapper inspect_snapshot_pinning has the same hidden work.
The binding and read-path diagnostic functions inspect only registry/cache
facts; they do not materialize record truth and remain metadata-only.

Make snapshot content inspection take the original execution request, resolve
its owner binding, demand the exact root at Full with the demanding dictionary,
and return Result<Option<_>, RelationalExecutionDenialCause>. Preserve None for
an unknown/released/foreign handle and preserve authoritative binding selection
when the caller's descriptive version is stale. Update the inspection wrapper
and all real callers. Searches find only Relational callers; World, Bridge,
Server, Query and the bank world do not call either inspection operation.
Extend the existing public read-door matrix with snapshot reads, snapshot
inspection and snapshot-pinning inspection using actual owner-issued handles;
check both region failures and per-runtime historical-symbol page failures,
sticky refusal, no journal effect and Head availability where appropriate.

Snapshot inspection now declares Full demand on its exact owner binding and
preserves typed failures through inspect_snapshot_pinning. The existing public
read-door family now covers 30 operations, including ordinary snapshot reads
and both inspections, with region damage and history-only dictionary damage.
Native library passes 1,953 tests (31 existing ignores); nine snapshot contracts,
all three CI clippy lanes, formatting, dirty caps, boundary-check and
agent-context pass. The first complete caller check found five external
certification callers omitted from the initial src-only search; update them to
pass their real request and distinguish request failure from an absent handle.
Their original branch/root/cancellation assertions remain. The corrected full
caller check passes.

A subsequent member-lifetime audit found no placement custody in the symbol,
lineage-node or deferred-index directory owners. StoreColdSource intentionally
reads only currently selected members; retain_members is the existing mechanism
that extends membership across replacement. In particular, symbol detached
snapshots copy their page cache, so warming the live dictionary does not warm a
region's isolated historical dictionary. Trace checkpoint page reuse/replacement
and prove whether an older unadmitted page can disappear before adding custody;
then use the existing port, not a second registry or a whole-manifest retainer.

### Dictionary directory placement custody

Boundary review: capture_metadata rebuilds both ID-range pages and hash-bucket
pages from the checkpoint dictionary. Adding a meaning changes the last range
and its bucket member. Store's cold source serves selected placement only;
detached dictionaries copy their admission cache, so another owner's warming
cannot discharge their cold obligations. Region custody retains region members,
not these dictionary members. An older directory therefore needs its own
placement owner. This is metadata retention, not payload residency or admission.

Plan: retain the union of each dictionary directory's ID and bucket locators
through the existing cold-source port before installing it. Detached owners
clone that opaque custody. Keep constructor storage failures typed through
recovery and retain invalid-frontier refusals separately. Add a replacement
court using real checkpoint-produced pages and a pruning cold-source port: the
old owner stays cold across replacement, reads its old ID and bucket pages,
and releases placement after its final detached owner drops. Also exercise a
retryable custody refusal without reading payloads. Run focused courts, native
lib, formatting, caps and enforcement before committing. Then verify the two
reported regressions before building the full subject-by-path matrix.

Dictionary custody is now acquired before directory installation/frontier
mutation and cloned by detached owners. The replacement court uses two genuine
checkpoint captures: both the tail ID member and the selected bucket change.
A pruning source retains the old members while a detached dictionary still
owns them, performs no payload reads on acquisition/replacement, and releases
the old placement after the final owner drops. A retryable placement refusal
preserves its Symbols subject and ColdStorageUnavailable recovery class without
changing the dictionary frontier. Focused courts pass 2/2; the full native
library passes 1,955 tests with 31 existing ignores. Initial verification caught
a missing path attribute on the new child test module; the corrected module
uses the established sibling-test layout. Production behavior was not weakened.

The same complete native run re-verifies both coordinator regression tests:
complexity_budget_unobserved_write_copies_no_partition and
publication_port_performs_one_exact_candidate_and_reports_the_loser_stale.
Both pass unchanged. Commit 0715b8f785 already transfers mirror target custody
rather than retaining a prepared Full observer. Commit 3a727cdad0 already
resolves branch-local recent IDs through immutable_commit_receipt, whose
performed canonical-envelope fallback covers the cutover-before-settlement
window and whose branch filter prevents unrelated performed routes leaking.
No second mechanism or cleanup-branch merge is needed. The exact run is recorded
in target/lane-evidence/dictionary-native.log (HEAD aef7ca6237, dirty tree);
final closeout lanes must be recorded again after all implementation commits.

### Subject-by-path failure matrix: guard families

Boundary review: thirty public visibility, scan, statistics, retention and
inspection doors share Full region guards, but their declared root versus
mirror scopes differ. The existing damaged-region loop does not distinguish
Head-member from history-member failures and exercises proven damage only.
Historical dictionary admission has a separate physically warmed-fork court.
The cold port can select only one damaged member today; it cannot inject a
retryable failure into all placements of one consulted subject. Envelope,
lineage and index guards have different production doors and must remain
separate adapters, rather than being mislabeled as dependencies of all reads.

Plan: keep one closed inventory of read doors, split its adapter out of the
already-near-cap visibility court, and add a table crossing every door with
region Head, region history and historical symbol pages. Inject all storage
failure classes through a locator-selective source; assert the exact typed
subject/cause, no journal or offered publication advance, sticky proven damage,
and successful retry after transient/allocation/limit refusal. Preserve the
existing cross-fork dictionary and retained-observation courts. Then extend
the same matrix vocabulary to actual envelope, lineage and index adapters,
transaction/commit/merge guards and retained reads, with explicit applicability
so unconsulted subjects cannot masquerade as tested cells. Matrix coverage is
not complete until those families and the compile-time Head fence and home
cold/warm/fully-warmed agreement have real evidence.

The cold lineage matrix exposes a production panic in aspect history:
branch_commit_envelopes traverses commit_envelopes_snapshot, which dereferences
all catalog bodies without accepted demands. The prior node court prewarmed
those bodies and therefore could not expose this independent dependency.

Boundary and fix plan: select branch ancestry from immutable receipts first,
with a finite 4,096-commit policy ceiling checked before any envelope demand;
refuse excess work with the existing WorkExhausted request cause, never partial
history. Demand only the selected authoring branch's bodies with the original
request and existing authenticated envelope cells. Keep the selected Arc bodies
through result construction. Make the four direct aspect-history operations
execution-taking Results, propagate through inspection, and preserve lineage's
existing typed Result. Remove the production all-catalog body traversal;
retain only the owner-test snapshot door under cfg(test). Add a cold envelope
failure cell for each history surface and a small-bound refusal court using
actual producer history. No fixture-wide body warming may conceal this gap.

The index matrix reaches its intended red boundary: SnapshotPin with an injected
MissingMember returns Ok(false), silently dropping the real restored generation.
Pinning currently reads exact_generation before admitting the deferred image.

Boundary and fix plan: pinning is the operation orchestrator. Validate owner,
exact basis and every selected definition first, demand the exact root at Full
with its symbol authority, then demand the selected index image before retaining
native generation handles. Empty index selections do not demand the image.
Carry the existing pin Request variant's native execution cause directly rather
than restricting it to MapKernelStop; this preserves storage subjects and all
request-control causes through field, join, ordering and Query callers. Refresh
the pinned symbol snapshot after Full admission, including view pins whose
metadata snapshot predates warming. Preserve query preparation's existing
kernel-stop reports while routing storage refusal as its typed ReadRequest.
The index matrix covers two pin doors, field preparation and validation,
candidate preparation and commit; each failure must precede journal effects.

The first six-subject matrix wave now passes 696 injected-failure cells:
32 Full read/pin doors x Head/history/symbols x six causes (576), seven
envelope doors x six (42), seven node-consuming doors x six (42), and six
index doors x six (36). This is scoped coverage, not completion of every
transaction, merge, retained export, event and index-build path. All cells
refuse with the exact typed subject/cause before journal effects; proven
MissingMember/Damaged/IntegrityMismatch stay sticky, while SourceUnavailable,
LimitExceeded and MemoryExhausted succeed after the injected refusal clears.

The matrix found and corrected two product gaps: aspect history dereferenced
unadmitted catalog bodies; pinning treated an unadmitted index image as absent.
Aspect history now demands the selected records at Full and bounded selected
branch bodies through authenticated cells. Four direct history APIs and their
inspection callers carry the original request and Result. Native whole-history
snapshotting is test-only. Pinning validates definitions before payload demand,
retains Full root custody, loads the selected index image and captures meanings
after admission. Its Request denial carries the native execution cause; Query
uses its existing native storage-denial mapper. The native cause owns the
single projection of kernel-only stops so both query preparation and managed
Query reads preserve interruption reports without turning storage failure into
a synthetic kernel stop.

Fixture corrections remain distinct from product fixes. The new dictionary
child initially lacked its sibling path attribute. A Windows ANSI write made
one matrix source non-UTF-8; it is now explicit UTF-8. The symbol matrix initially
returned a memory-only fork while asserting a durable medium; warming the fork
and testing the still-cold durable original preserves both claims. The index
matrix initially used a materialized view where pinning requires an exact
projection, and accessed a denial field instead of its getter. It then exposed
that merely registering an index does not publish a generation: setup now uses
the declared field schema, builds through the actual index authority, verifies
the exact producer head generation, and captures that state. Its checkpoint
fixture is separated from the fault courts. No test claim is weakened.

Query caller checking initially rejected the widened refusal at two kernel-only
mappers. The performed-approval caller now uses relational_read_stopped; managed
read denial projects genuine kernel stops and preserves other native causes.
The corrected Query lib/test caller check passes. The complete native library
passes 1,960 tests (31 existing ignores), including all four matrix families,
the bounded-history ceiling court, and the existing Head compile-time fence.
All three owner-service CI clippy lanes, formatting, composition/dirty caps,
boundary-check and agent-context pass. A boundary run started during edits
observed a module declaration before its file existed and refused; its corrected
stable-tree run passes. Composition reports 59 advisories: the new long routines
are closed operation adapters or explicit producer/fault courts, not mixed
production responsibilities; API forwarding in existing larger courts retains
the original assertions. No touched file exceeds its applicable cap.

These intermediate logs are under target/lane-evidence/matrix-*.log with
commands, HEAD c83ea18eca, dirty-tree status, UTC start/end and exit codes.
They do not substitute for the final post-implementation lane list. The three
owner-service clippy lanes are not the entire CI lint list: query_gate also
names lint-lower, lint-query, lint-fixture, lint-bank and lint-ui-binding; final
verification must include those actual command suites.

### Head-only current-operation matrix

Boundary review: current transaction reads already declare Head demand, but
preparation's declared input demand and the stable mirror target still require
Full. Root Head capture and authenticated layered folding are available; the
publication target's owned edit currently accepts only Full leases. The next
matrix must expose this dependency before changing the mechanisms.

Plan: produce real entity/relation rows and old versions, capture and reopen a
writer on the exact same canonical medium prefix, then fault every selected
history member under each failure class. Current entity/relation transaction
reads, validation, candidate preparation and commit must succeed without any
history-member read. A committed update must advance only its normal journal
position and remain readable through a Head transaction. Use the failing cells
to cut preparation and publication to Head without adding an effect-time load;
retain Full witnesses for explicit history, retention and replay-pin operations.
Do not weaken the new matrix or the existing Full failure matrix.

Candidate preparation now reaches the remaining Full-only mirror target. The
implementation plan is to keep a single stable target core with a sealed
Head-or-Full lease (no cold variant), consume that lease into the corresponding
owned edit, and restore its level without effect-time I/O. Full maintenance
continues to mint its wrapper only after the current edition has accepted Full
custody and the demanding runtime's Full meanings. The current mirror's cached
edition remains an owner projection, removed before COW; planned targets retain
the stable core, not an extra payload observer.

An owned Head edit is trusted newly produced state, like the existing owned
Full edit, whose commitment cache is also optional. It retains the prepared
root's schema and isolated dictionary for later Full readmission. If its derived
commitment cache is absent, a Full demand first derives the expected digest
from the resident Head/suffix and authenticated prefix openings, then folds cold
bases independently and compares the complete result. This keeps publication
load-free without adding a digest calculation that can fail after effects.
Checkpoint admission still supplies the externally checked commitment; the
owned-current constructor cannot turn unadmitted checkpoint bytes into a Head.

The current-operation table now has six doors (entity/relation transaction
reads, validation, candidate preparation, update commit and relation deletion)
under all six source failure classes. Its positive companion retains the old
observation, writes through Head, captures without I/O, observes a retryable
Full history refusal without a journal effect, recovers, and compares current
and retained entities/relations against an independently executed memory runtime.
The repeated warm Full read performs no further member read.

The first native run exposed four integration regressions in addition to a
wrong new test assertion. Transaction reads retain retired record metadata;
the deletion court now compares the old observation's absent retirement with
the new observation's exact deletion version. The phase-1 replay pin court
implicitly assumed a current write loaded Full: it now additionally proves
that Head cannot supply its pin witness, explicitly demands Full, and retains
all original pin-count assertions. The cold-fork court enumerated current
relation identities through a Full-only test accessor; its enumeration now
uses the demanded Head arena, preserving every record comparison.

Removing no-op lifecycle edits exposed a genuine initialization coupling:
new mirror targets shared the immutable branch root's cell, and the old
post-publication no-op edit had detached that sharing. Preparation now gives
a new target its own resident owned payload before effects. Existing targets
still consume their exact stable Head/Full custody after dropping the mirror's
owner projection; the unobserved-write and whole-mirror copy budgets remain
unchanged. This phase-2 level-aware edit extends phase 1's Full-only mirror
mechanism because the Residency section explicitly permits writes at either
level. Full historical mutation and replay pin witnesses remain Full-only.

The corrected native library is green: 1,962 passed, 31 existing ignores,
including both coordinator regression tests, every preceding fault table and
the new current/Full courts. Owner library clippy (-D warnings -A dead-code),
fmt, dirty line caps, agent-context and diff whitespace checks pass. Composition
scans 26 files with zero hard violations and 21 advisories. The new long test
routines are explicit producer/fault/reference courts; the longer production
writer functions consume or restore one target, and the existing preparation
orchestrators retain their declared stages. No responsibility is split merely
to meet the cap. Logs matrix-head-* record this intermediate dirty-tree epoch.

### Remaining preparation failure classes

Boundary review: the original symbol bucket court covers five real doors but
only damaged bytes; MemberSource now carries native per-locator failure causes.
The existing global invariant court discovers a remote Head after an applied
draft, and currently proves only malformed content. Both use genuine recovery
and canonical medium prefixes, so the new tables can preserve production
provenance rather than introducing another injection hook. The same doors must
retain native typed failures for all six classes; only proven damage may stick.

Plan: add a nested symbol-bucket table using the original fixture/door adapter,
assert diagnostics, dictionary and journal invariance immediately after refusal,
then distinguish sticky refusal from successful retry. Add the corresponding
remote invariant Head table selected from actual manifest locators, preserving
the existing identity-spending and accepted-counter courts. Keep these changes
separate from envelope lifetime and lineage-event paging, which need their own
producer/fault cases. Run the focused tables before broad native verification.

The bucket table passes all 30 cases (five native doors by six causes). The
remote invariant tables pass 18 Head refusal cases and 18 history-independent
success cases; the original malformed-Head and identity-spending courts pass
unchanged. Boundary-check for the preceding Head slice also completes green.
The current-operation producer and adapter are now separate named modules so
the same six door implementations cover both successful unavailable-history
runs and typed Head/symbol-page failures. The declared-demand table adds 72
cells using actual root/mirror Head locators and actual symbol id pages. Its
adapter checks the original native transaction-read or preparation variant
before projecting the storage payload; a post-effect or generic failure fails
the court rather than being normalized into the expected answer.

The declared-demand table's first run failed an overly broad I/O assertion:
a sticky symbol-page refusal may be reached after admission of another region
on the next preparation path. The failure remains the same native subject and
cause. The corrected assertion checks the actual failed member set, refusing
any reread of those members while permitting unrelated region admission. This
matches damage isolation instead of treating one subject's damage as a global
no-I/O latch. Journal and diagnostic invariance assertions are retained.

The corrected declared-demand family passes all 72 refusal cases, the 36
unavailable-history success cases, and the retained-reference Full promotion
court. The successful retry of a publishing door advances only its ordinary
commit; both its failed first call and every nonpublishing retry leave the
journal unchanged. These tables reveal no additional product defect. Their
fault adapters and producer extraction preserve the previous assertions.

The all-test owner clippy lane exposed a real module-wiring defect from the
phase-2 reservation split: #[path = "symbol_table/reservations.rs"] made its
implicit mod tests resolve to symbol_table/tests.rs again. Clippy's duplicate_mod
error was correct; six reservation tests in reservations/tests.rs had not been
compiled at all. An explicit path now loads that intended file. The parent
symbol-table tests remain unchanged and run once, not twice. No test or claim
is deleted; the corrected tree must run the previously disconnected reservation
courts as well as the new matrices, then repeat the failed clippy lane.

That clippy run also correctly rejected two now-identity .into() conversions
in the pin court after its denial type widened to RelationalExecutionDenialCause.
The assertions now compare the same native values directly. No lint is allowed
away and the failed all-test lane will be rerun on the corrected sources.

With the reservation path and pin comparisons corrected, all six newly wired
reservation courts pass and the complete native library passes 1,970 tests
with 31 existing ignores. The new fault tables have 120 denial cells (30 symbol
buckets, 18 remote Heads, 72 declared Head/id pages), plus 54 history-independent
current success cells across this and the preceding slice. Every table checks
its typed native boundary before extracting the payload. Formatting, dirty
caps, context and whitespace pass; the corrected all-test clippy run is active.

### Envelope flight custody boundary

Boundary review: indexed recovery admits bodies through envelope cells, but
those cells use OnceLock and retain a queried body permanently. Recovered
nonheads have no other required owner. The catalog artifact also retains
canonical bytes for admitted heads. Separately, live publication still retains
whole envelopes in the ledger, performed stream and durability log, and a
new medium does not currently expose its cold source. These are distinct
retention owners that must ultimately cut over together; fixing only a cache
would not prove the full history-on-demand obligation.

Plan: first expose the minimal lifetime violation with a real recovered
nonhead query: hold two simultaneous returned envelope Arcs, prove they share
one admitted body, drop both, then inject a transient journal failure and demand
again. No branch head, unsettled route or external observation names that old
commit. The next query must read through the cold source and return its native
Envelope failure, then recover after the source returns. Keep memory-only
resident cells and positioned head readmission proof intact. From this failing
court, replace permanent cold-body ownership with flight custody and carry
owned answers through body consumers. Then add live-publication growth and
settlement courts before changing the ledger/route/log retention owners and
the medium-to-cold-source handoff. The existing selective history and all six
failure-class courts remain mandatory evidence throughout the cutover.

The flight court fails on the original OnceLock: its Weak observer remains
upgradeable after both genuine query readers are dropped. Cold cells now keep
only a weak body reference plus per-cell proven damage; demand returns an owned
Arc to its caller. The memory-only backing owns a strong Arc. Head readmission
uses the same journal/digest/readmission routine while returning its positioned
commit directly, so a borrowed body never substitutes for a position proof.

The owned handoff exposes four mechanical borrow callers, now corrected. It
also reveals that recovery's retained_cold_entries classified catalog identity
by current body residency. Rebuilding unrelated heads could therefore drop an
index just because a reader held its body. Recovery now names the exact IDs it
will replace and retains every other entry independently of cache state; a
producer-backed unit court covers an in-flight body across that projection.
Metadata-alias visibility compares the artifact's sealed root descriptor with
the selected root directly instead of opening a body through a pure accessor.
This uses the indexed root fact already validated at admission and preserves
metadata-only version aliases without a cold-source operation behind an accessor.

The first owned-envelope caller check identified three production borrows and
one pointer-equality test that still expected borrowed Arcs; the corrected
caller check passes. All 45 envelope-focused courts pass, including the new
flight-lifetime court and catalog projection court. The complete library is
green with 1,972 passed and 31 existing ignores. Formatting, dirty caps,
agent-context and whitespace pass. Composition scans 14 files, zero hard
violations and seven advisories: existing catalog/recovery orchestration remains
one recovery responsibility, and demand retains its single serialized admission
boundary. The corrected all-test clippy from the preceding preparation slice
also completed green. Envelope clippy and boundary are running for this slice.

This does not claim live history release: publication's in-memory ledger,
performed nodes and durability log still retain bodies, and new durable medium
attachments do not yet supply their cold read source. Those owners remain an
explicit next matrix target, alongside lineage-event payloads and final oracle
coverage. The weak cold cell fixes recovered query lifetime rather than masking
those separate live retention owners.

Envelope all-test clippy completes green (-D warnings -A dead-code).

### Aspect-history guarded subject table

Boundary review: direct entity/relation history (with and without traces) and
lineage aspect history share branch-envelope demand, but the first envelope
table covers patch/inspection/summary surfaces only. Their region and symbol
Full demands also need explicit coverage. The current fixture already produces
real entity/relation changes and a cold recovered journal; capture the producer's
actual lineage ID before encoding, then keep recovery cold.

Plan: table those six history doors against Head, history, symbol id-page and
old envelope faults under all six classes. Require the same native execution
storage failure, no journal/diagnostic publication, own-subject sticky damage,
and genuine nonempty contents after retryable failure. Query source attempts
are observed at the named failed member/position, independently of admission
of other subjects. Retain the existing positive current/Full comparison courts.

The first aspect-history invocation used the source filename as its test filter;
the Rust module is named aspect_history_matrix, so Cargo selected zero tests.
That log is not evidence for the table. The corrected invocation selects the
exact residency_failure_matrix_aspect_history_doors test name.

The corrected aspect-history table passes all 144 cells. The shared fixture
family passes all four courts: current no-history, declared Head/symbol failure,
retained-reference Full promotion and the new historical subject table. All-test
owner clippy, formatting, dirty caps, context and whitespace checks pass. No
product change was necessary for these added history doors.

### Live durable envelope release boundary

Boundary review: the recovered cell now honors flight custody, while live
bodies have four permanent owners: the catalog artifact, ledger sidecar,
performed stream/route and durability log. The prepared route's handoff already
releases its root at performance, so a normal settled commit with no observation
can reclaim its old root. The durable medium currently publishes commits and
reads transitions, but only recovery attaches a cold source. Releasing bodies
without first making the journal read port available would create false absence.

Plan: expose live retention with a paired memory/durable court using ordinary
commit helpers that release snapshots, no retained observation, and explicit
bounded root reclamation. The durable nonhead's Weak body must expire, while
the memory nonhead remains resident. Both must still answer the same bounded
patch query. Then trace all four owners into one indexed-cell authority and
carry the medium's real cold source from attachment, preserving performed
receipts before settlement and positioned head admission. Add transient and
proven live-journal failure cases and rerun recovery, publication races, capture,
all history query matrices and downstream owner lanes after the cutover.

The live lifetime court is red exactly at the durable nonhead Weak release;
its memory-only half passes. The first implementation step makes cold_source a
required medium port, retains that read-only handle in the medium binding even
after publication access moves, and binds it at attachment. Store supplies its
existing shared cold-source provider, not another placement registry. Native
medium wrappers delegate the same provider. World's lost-answer fixture now
retains its actual committed/lost-answer bytes instead of claiming commits
without a recoverable journal. The existing fork fixture already has real
bytes and supplies its existing read adapter. The three ownership compile
courts share a construction-only journal fixture and must retain their intended
ownership errors when their diagnostics are regenerated by the harness.
A direct attachment court reads and readmits the first actual durable artifact
through the handle obtained before any publication; memory-only has no port.

The medium-to-source attachment court passes. Relational and World lib/test
caller checks pass. Store lib/test checking and the ownership compile court
are running; no product edits occur while trybuild compiles the current sources.

Live owner cutover design, from the traced preparation handoff:

- PreparedRelationalPublication already owns its catalog artifact before route
  reservation. Expose that artifact's existing envelope cell to the route so
  route, catalog, ledger and durability references share one cell, rather than
  introducing a second live body cache.
- Keep PositionedCanonicalCommit as the sealed, fully resident positioned
  value. Place a weak position-retention witness in the canonical publication
  owner, minted only from that sealed value. It remembers the exact weak body
  allocation and admitted position; upgrading it recreates a positioned value
  without I/O or rebinding an arbitrary body to a bare number. It belongs in the
  runtime because it holds a live weak lifetime. Thread it through the runtime
  facade chain. A journal readmission can mint the same witness from its sealed
  positioned result.
- Extend the envelope cell's lifecycle with explicit memory, unsettled journal
  and settled journal states. Memory owns its body permanently. An unsettled
  journal state owns its body until settlement; a settled state keeps the index,
  source, weak flight and position witness. Bind journal digest/placement from
  the actual successful durable step before publication. Publication performs
  no loads. Branch roots and accepted readers keep their existing strong body
  Arcs, so a head or in-flight body remains available after settlement.
- Replace body-owning ledger, performed-node and durability-log sidecars with
  references to that same cell and immutable receipt/position facts. Pure
  resident accessors name residency; historical operations explicitly demand
  cells and retain their owned answers. Capture selects resident heads and
  carries the complete index without loading nonhead bodies. Do not reconstruct
  the index by scanning whatever bodies happen to be resident.
- Drop prepared encoded payload buffers on durable catalog installation while
  retaining the small derived digest/inventory facts needed by inspection.
  Memory-only fixtures retain their bytes and bodies. Recovered index entries
  keep the journal digest as the readmission verification source.
- The preparation history binding's commit_record_set currently filter_maps
  missing resident envelopes. Once live bodies release, that would turn cold
  changes into an empty conflict set. Give the operation an execution-taking,
  fallible demand over its metadata-selected commit IDs and preserve the typed
  pre-effect denial. Add a recovered intermediate-commit conflict/fault court;
  a current head alone must not counterfeit complete merge history.

Acceptance is the live paired memory/durable release court, typed transient and
sticky live journal faults, selective history reads, source-only intermediate
merge conflicts, publication race/position courts, load-free checkpoint and
reopen, then the full owner and downstream lanes. These changes are required
by the already-red lifetime court; no compatibility body-retention lane remains.

The first Store --lib --tests check omitted its existing
certification-test-authority feature; the integration fixture therefore could
not import certification types or call certification_serving (29 cascading
errors). This was a lane configuration error, not a product fix or a new feature.
The corrected owner check enables that already-retained feature. The ownership
UI run preserved E0599 for cloning and E0382 for moving the medium twice; only
fixture locations and qualified display changed. Its pass court succeeds. The
harness's TRYBUILD=overwrite command regenerates the two expected diagnostics,
followed by a normal rerun. The evidence runner now also records TRYBUILD and
other explicit build overrides in its environment header.


Live custody implementation and matrix follow-up:

- Ledger and canonical-log sidecars now retain the shared envelope cell and
  immutable position/identity facts. Performed stream nodes retain no body.
  The runtime's actual durable binding supplies journal placement before
  cutover; settlement releases the flight. Prepared catalog installation drops
  durable encoded payload buffers and keeps an exact streamed payload digest.
- The paired live lifetime court is green: old durable bodies disappear after
  reader/root release and read back; memory-only bodies remain. A complete
  external capture/reopen and retained recovery court preserves every indexed
  receipt while resident bodies fit headroom. A runtime position-retention court
  proves that weak custody cannot keep a body alive or rebind an arbitrary one.
- The first native owner run reported 1807 passed, 171 failed, 31 ignored. The
  common product failure was legacy restore installing routes before catalog
  admission. Admit its catalog once before installing shared-cell routes. That
  correction yielded 1975 passed, three failed, 31 ignored. The remaining
  failures exposed source locality for post-reopen embedded envelopes, route
  installation preceding recovery cell synchronization, and one accidentally
  retained test-local Arc. Each was corrected; the next native run passed 1978
  with 31 ignored. These are intermediate dirty-tree logs, not final evidence.
- Retained export now selects complete indexed cells, including released live
  commits; it exports through their shared damage state. Embedded checkpoint
  capture carries each cell's actual journal placement rather than reading all
  commits through the prior directory source. New live commits and adopted old
  directory commits can therefore coexist without source substitution.
- The envelope matrix now includes retained artifact export, embedded native
  checkpoint export and portable memory checkpoint export (18 added cells).
  Its first run exposed an unguarded reconstructive memory image. The operation
  now demands the exact captured roots/mirror at Full, every selected envelope,
  and the selected symbol/node pages before constructing the image. Native
  external capture still selects index facts without loading bodies.

Still to prove or close before declaring phase 2 complete: recovered tail body
lifetime, the cold intermediate-commit merge court, index reconstruction and
remaining checkpoint subject cells, lineage event paging/association, the real
home cold/warm/fully-warmed oracle, and every final downstream/CI lane. These
remain required scope, not deferred repository debt.


The expanded envelope matrix passed all 60 cells after the explicit memory-image
operation demand. The owner lib rerun passed 1978, failed zero, ignored 31,
including both priority regression courts unchanged. Owner clippy first refused
two collapsible-if shapes; both were corrected and the same clippy lane passed.
Formatting exposed checkpoint_restore.rs at 403 lines; its history admission
step moved into checkpoint_restore/history_readmission.rs (one named recovery
responsibility), and the dirty line-cap rerun passed. A logged bash invocation
first resolved Windows' WSL shim, whose /bin/bash was absent; the corrected lane
uses the exact Git Bash executable and passed. Boundary-check and agent-context
passed. Contract/ownership UI verification is still running at this entry.


Contracts completed green: 203 passed, zero failed, three ignored, including
all changed medium move/clone compile courts. The independent failure table now
states expected sticky behavior explicitly instead of calling the product's
classifier. All ten selected matrix families passed; the expanded envelope
family also proves body reads share export/capture damage. Owner clippy passed
after that test-only strengthening. Composition reports no hard errors; its
long-function/argument advisories were reviewed as atomic publication,
checkpoint/recovery orchestrators and producer-backed matrix courts. World
lib/integration tests have reported no failures and are still running their
compile-fail contract test at this commit boundary. The coverage ledger is
residency-phase-2-matrix.md; pending cells are explicitly named there.


## Next matrix slice: nonhead merge history

Boundary review: independent merge planning already explicitly demands every
metadata-selected delta commit in merge/planning.rs::branch_commit_delta. The
ordinary transaction merge-parent path is different: preparation history's
inspect_merge_from_bindings and commit_record_set read resident routes and
filter missing bodies away. After the live custody cutover a committed cold
intermediate source change can therefore disappear from conflict detection.
This is a product gap; current branch-head bodies are insufficient evidence.
The preparation binding owns route/index custody and candidate identity
reservation, while the commit phase owns failure attachment and diagnostics.
An unknown structural head still maps to MissingMergeBase, but an indexed cold
body must either readmit successfully or return the exact storage refusal.

Plan: first produce two entities, a shared fork base, a source update to the
first entity followed by a source-head update to the second, and a conflicting
target update to the first. Reopen the real checkpoint/journal through the
fault-selectable cold source. Compare against producer-derived conflict facts,
then inject all six independently classified failure cases into the intermediate
source envelope through both doors. Preserve the position-reservation/journal
frontiers, retry only unproven damage, and prove accepted retries retain the
real overlap. Add an explicit execution-taking demand to the route/index owner;
move preparation merge inspection into its own named child module and thread
that request through authoritative history resolution. Keep one route/cell
mechanism. No resident-only compatibility path remains. Run the focused matrix,
transaction/merge history courts, owner clippy, fmt, caps, boundary and context;
the final full lanes still run after the remaining matrix slices close.


World completed green after d2bec84327: 265 unit plus 41 integration tests
passed; three integration tests were ignored. The compile-fail contract court
passed. The nonhead merge reproducer then failed for the intended product
reason: transaction merge-parent validation published commit 6 with parents 5
and 4 and no changed records, although intermediate source commit 3 overlapped
target entity A. It had silently filtered the cold intermediate body away.
The corrected preparation operation explicitly demands its indexed delta cells
and carries the request through the commit history phase. Metadata-only head
receipts remain available independently of body warmth. Source/storage failure
uses the existing pre-effect preparation denial; structural conflicts retain
their existing diagnostics. The named child module needs an explicit #[path]
because this owner itself is loaded by a path attribute; the first corrected
build exposed and repaired that module-location mistake. The focused reproducer
and all 18 nonhead envelope refusal cells passed. The matrix now also checks
that the patch-position reservation counters remain unchanged on every refusal
and semantic overlap retry.


Nonhead merge closure verification: full Relational lib passed 1980, failed
zero, ignored 31; the focused two courts passed, including all 18 refusal cells.
The reservation-counter assertions passed in the full owner run. Owner clippy,
formatting, dirty line caps and agent-context passed. No tests were removed or
weakened; the original conflict diagnostics and missing-parent semantics remain.
The matrix ledger now inventories 996 refusal cells plus 54 explicit
unavailable-history success cells. Boundary-check will run again as part of the
remaining milestone gates; no new public type or alternative route was added.


## Next matrix slice: recovered tail envelope custody

Boundary review: native indexed restore creates journal-backed cells, but tail
replay builds ordinary memory artifacts. Its final inventory synchronization
therefore has no fact saying which verified journal payload backs each body.
ReadmittedCanonicalCommit currently discards the verified artifact digest; the
recovery plan has a source, but using that global source or an unverified new
body digest as journal placement would counterfeit custody. Retained recovery
also must preserve per-cell sources when an older directory and a newer live
journal coexist. Memory/current readmission without a durable source must keep
its only copy.

Plan: add a real no-checkpoint tail lifetime reproducer first. Preserve the
verified canonical frame digest through readmission, attach the actual recovery
source only at the source-owning recovery-plan boundary, and carry that small
placement fact through replay. After canonical inventory/root reconciliation,
bind matching cells to their indexed journal copy and settle them without an
extra read. A rewritten body cannot borrow an unrelated source digest. Keep
memory-only inputs resident. Extend the envelope refusal matrix to the tail
recovery shape so every existing history/export door must return its typed
cold failure after replay. Use the existing cell and position custody rather
than a second history cache. Verify selective reads, complete capture, tail and
codec recovery courts, owner lib/clippy, fmt/caps/context, then continue the
remaining lineage/index/oracle cells.


Tail reproducer failed for its intended reason: a recovered nonhead body stayed
alive after all real readers and retired roots were released. The correction
preserves the verified frame digest as private readmission state, binds the
actual source at the source-owning recovery boundary, and preserves per-cell
placement in retained recovery plans. Final replay inventory binds journal
cells only after checking the recovered positioned payload against that exact
digest. No extra journal read is needed. The encoded payload cache moved from
immutable catalog shells into the shared cell so the same journal transition
releases it for every observer. Memory cells retain their sole copy. Existing
preencoded-sharing assertions now observe the same cache allocation through the
cell, preserving their original claim. The focused lifetime reproducer passed;
the envelope matrix now runs both checkpoint and no-checkpoint tail shapes.


The full tail-custody run exposed eight failures: four cert replay paths reached
an unguarded body accessor; derived-index preparation and two head/lifetime
courts exposed position custody pointing at an equal, unowned replay allocation;
the tail export matrix also refused a capture through that cold head. Recovery
must retain the root-owned allocation after authenticating exact positioned
canonical payload equality, excluding only the diagnostics/cache lanes that
the journal grammar already excludes. The position seal now validates that
same payload before admitting an equivalent resident allocation; arbitrary
body/position rebinding remains refused.

This revises the slice boundary: replay's public envelope read is a history
surface too and must take execution, return a typed refusal, and replace its
duplicate owned alias. Replay orchestration must declare its metadata-selected
ancestor envelope demands and hold their cells through planning/reconstruction.
The internal pure capability names resident observation explicitly and performs
no I/O. Update cert/test callers without discarding their claims and propagate
the Query bootstrap consumer's exact read refusal. Add direct replay-envelope
and replay-execution doors to the envelope matrix. These are exposed product
gaps, not test relaxations or an additional history authority.


### Tail readmission and replay boundary correction (in progress)

The 13-door envelope matrix is now exercised in both checkpoint-backed and
journal-tail recovery shapes. The tail producer exposed a real lifetime gap:
replay rebuilt memory-owned envelopes and never transferred them to their
verified journal placement. Readmission now carries its exact frame digest and
actual cold-source port into recovery; memory-only recovery keeps its sole copy.
The canonical payload cache lives in the shared residency cell, so transferring
custody releases encoded duplicates for all catalog observers together.

Recovery must bind a position seal to the allocation actually owned by its
branch head. Equivalent allocation admission compares the canonical journal
payload at the original position; it cannot seal changed canonical facts.
Source envelopes replace reconstructed diagnostic captures when source authority
is restored. Their visibility axis is readmitted from admitted source facts only
after schema and current storage binding checks. Branch replacement uses the
existing retirement reservation, head transfer and retirement operation once per
branch. An initial direct cell replacement was wrong: it bypassed the live-head
obligation and failed 84 native courts. That implementation was removed.
The corrected native run passed 1,981 tests with one matrix failure remaining.

That remaining replay failure exposed an authority mismatch: RCCA explicitly
omits diagnostic entries, while replay always promised to compare a capture.
Journal-backed replay compares the diagnostic scope, kind and determinism
that RCCA authenticates, without claiming its omitted capture entries. Memory-
owned replay still compares the full retained diagnostic capture, including an
empty capture. A first surface-removal approach failed three existing live/
recovery equivalence courts and was replaced with this projection comparison.
The existing diagnostic-tampering court remains unchanged and must pass. This
is an authority/lifecycle correction, not permission to suppress canonical drift.

The matrix also exposed unguarded historical snapshot projection, public replay
envelope access, and ancestor receipt minting. These doors now take the request,
demand the actual selected bodies/Full view, hold their witnesses, and preserve
typed storage failure. Exact-observation receipt minting uses its already carried
head body. Bridge and Query callers follow the same fallible owner door.


### Next matrix slice: lineage event pages (boundary review and plan)

Authority review: LineageSubsystem owns publication event rows and per-runtime
resolution indexes. Its node residency owns page cells, but currently lacks the
placement custody already required by dictionary residency. The checkpoint
manifest carries only node-page locators. Cold restore installs no event rows;
request-free historical traversal then accepts a resident-only empty index.
This is a missing subject/path family, not a reason to warm every envelope.

First add a producer-written Replace chain across an external checkpoint and
compare genuine before/after lineage resolution, graphs and selected-root
history. Preserve independent unrelated lineages so page locality is visible.
Add a pruning cold source court for an older node/event directory surviving
checkpoint replacement. These tests must fail before changing the owner.

Implement immutable event members routed by fixed node ranges. A member holds
bounded ordered event rows with their publication commit identity. An event
participating in several ranges may appear in several members; readmission
accepts an already admitted row only after exact event and publication equality.
It never relaxes ordinary recovered-batch duplicate rejection. New events append
members rather than rewriting old range payloads. The directory carries prior
placement without loading old events. Node and event residency own the exact
member custody they need, independently of selection-directory lifetime.

Historical orchestration demands the selected node range and discovers target
ranges before accepting traversal. Full graph/inspection/export doors declare
all relevant pages. All these operations take the request and return the same
typed storage refusal; pure index accessors never initiate loads. Readmission
checks the selected manifest/member digest, kind, routing, ordered unique IDs,
publication/index binding and byte/request limits before any row admission.
Capture encodes only newly published rows and carries the prior member closure.
Reopen loads neither nodes nor events, and unrelated ranges stay cold.

Expand the table for six independent failure classes across every event consumer,
then exercise cold/warm memory-reference agreement and retain observations through
subsequent publication and capture. Update the coverage ledger only from passing
executed tables. The selected-manifest checksum/member SHA verification anchor
remains the specification's anchor; it is not a second canonical-journal root.


Tail slice validation (intermediate dirty-tree evidence at d38dfc2b88):
`matrix-tail-native-projection.log`: 1,982 passed, 31 ignored; no failures.
`matrix-tail-contracts-green.log`: 203 passed, 3 ignored; no failures.
The normal contracts rerun follows tool-regenerated goldens; only the two caret
locations shifted, and the original authority-type errors remain. The first
contracts run had 202 passing tests and that one stale-golden court failed.
The envelope table now executes 156 refusal cells (13 doors, six classes, two
recovery shapes), bringing the refusal inventory to 1,092 cells plus 54 positive
unavailable-history cells. Both priority regression tests pass in the native run.
Owner Relational/Bridge lib+tests clippy passed with the CI dead-code allowance;
Query execution lib check and both affected workspace formatting checks passed.
Dirty line caps, composition scan (zero scan errors), boundary-check and
agent-context check passed. All are slice evidence; final clean-commit lanes,
lineage event paging, index-build/checkpoint cells and the real home agreement
court remain to be completed. No final phase-2 completion is claimed.


Lineage event matrix first red: `matrix-lineage-events-first.log` runs actual
producer publications (601 creates, then two replacements), captures external
members and reopens through the public recovery source. Reopen read zero
members and admitted zero nodes/events as required. Historical resolution then
returned lineage 601 instead of the producer's lineage 603. The product is
wrong: the cold directory contains no event authority and traversal accepted
its empty resident event index. The test has not been weakened.

Refinement before production editing: store each bounded event payload page
once, with directory routing by the distinct fixed node ranges participating
in its events. A wide split/merge must not duplicate its entire body for every
range. One cell per member is shared by all its range routes. Readmission
recomputes the complete route set from the admitted rows and checks it against
the member's directory declaration, so negative route lookup is justified by
the selected directory. A point query can overread at most one bounded payload
page per consulted route entry. New capture appends only newly published event
rows and carries prior pages; placement checks remain destination-scoped.
The native format will advance cleanly when the event directory is introduced.


The older-directory custody court is independently red:
`matrix-lineage-custody-first.log` loses the authentic node member for range
513..603 after a modeled home checkpoint replacement. Its cold source retains
only members named by actual live custody handles; a bare directory/source Arc
confers none. Node residency had none, so the point demand returned its typed
MissingMember error. This is a product placement-lifetime failure, the same
category as the dictionary audit fixed earlier. The fix makes node-directory
installation fallible, acquires the exact node-member custody without reading
payloads, and carries that handle in the shared node-residency owner. Forks share
it; the last owner drops it. Recovery preserves a typed cold-storage refusal if
placement cannot be acquired. Event residency must obey the same contract.


### Paged lineage events and their complete capture

CHK11 adds bounded event payload members (MBR1 kind 3), each stored once, with
node-range/branch routes and exact event-id membership in the directory. Capture
appends new publication rows, carries verified prior pages and writes O(1) new
members for a small change. The directory validates coverage against the whole
published lineage basis without reading bodies. A demand authenticates the
member, recomputes all routing/id facts, checks event structure and exact indexed
commit receipts, then admits immutable rows into that runtime's resolution
index. Duplicate admission is allowed only for the exact same row and publication;
ordinary recovered-batch duplicate rejection is unchanged. Admission membership
and the copied index are sampled together for a runtime fork.

Historical lineage and selected-root traversal explicitly discover and demand
the event closure before accepting results. Selected observations load a cold
retired-record seed rather than accepting an empty node index as absence. Graph
and divergence declare their branch's event pages. Memory image export demands
all selected event pages. Native embedded export uses the same event cells for
raw member export, sharing typed failure and sticky damage. Native capture never
loads cold event payloads. Its existing checkpoint gate still refuses unsettled
performed publications; no phase-1 settlement protocol was replaced.

The table exposed an observation history returning None, and embedded export
wrapping a storage error as generic corruption. Both were product errors; their
guards and exact error propagation are fixed. A range-locality court exposed
zero-based bucket arithmetic for one-based node ids (node 512 shared node 601's
route). Correct range arithmetic fixes the locality; the headroom assertion
was retained. A separate byte-changing source proves actual member integrity
checking and damage isolation, independently of the six injected error classes.

A native foreign-destination capture from a fork exposed another product error:
all shared page cells were resident, but the fork's query index was empty, so
capture emitted no event pages. Capture now acquires only already resident page
witnesses, merges their immutable rows with live publication rows and holds the
leases through encoding. It does not mutate the fork's query index. Final native
event-directory validation refuses any incomplete producer output. Event export
providers advertise only the members they actually own; resident live suffix
rows can materialize exact inline bytes after the member digest matches, so an
older directory is never asked to serve a newly published member.

Validation so far: native `matrix-lineage-native-snapshot.log` 1,990 passed,
31 ignored; contracts `matrix-lineage-contracts.log` 203 passed, 3 ignored;
owner Relational/Bridge lib+tests clippy passed, as did format, line caps,
composition scan (zero scan errors), boundary-check and agent-context.
These are dirty-tree slice logs at 41b5748788, not final phase-2 lane evidence.
The event table contributes 66 refusal cells (11 doors times six independent
classes), bringing the refusal inventory to 1,158 plus 54 positive unavailable-
history cells. Real home agreement, index-build/checkpoint subject expansion,
outstanding reservation/checkpoint crossing and the final complete lanes remain.
