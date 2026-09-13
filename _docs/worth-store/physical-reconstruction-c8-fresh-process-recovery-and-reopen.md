# C.8: Fresh-Process Recovery And Reopen

## Goal

C.8 replaces a dead writer process with a genuinely fresh process that derives
one canonical physical truth from persisted C.7 authority, reconstructs that
truth inside declared recovery bounds, publishes it durably, and returns a
quiescent physical handoff with a new runtime identity and exact physical
operation fates.

C.8 is complete only when recovery needs no live writer object, heap-derived
layout, copied closeout value, replay artifact, ambient singleton, semantic
model, or operator guess. Identical admitted bytes and configuration must
produce the same selected basis, redo decisions, physical result, operation
fates, cleanup posture, and counters.

## Why This Milestone Exists

C.7 makes ordinary physical effects durable and leaves bounded persisted facts
for reconstruction. It does not prove that those facts can replace a dead
process. A runtime that can close cleanly, produce a rich in-memory handoff, or
reopen while retaining its own heap has not proved crash recovery.

C.8 closes that gap. It establishes the first point at which all volatile
physical state may be destroyed and current physical truth can still be
reconstructed from the store root and qualified platform inputs alone.

C.8 precedes C.9 because corruption localization needs a real recovery
consumer whose admission points and source decisions it can protect. It
precedes C.10 because stable reads, reclaim, and maintenance need one recovered
root generation and explicit retained-generation posture.

## Roadmap Placement And Inherited Truth

C.8 consumes the closed guarantees of:

- C.3 sealed runtime lifecycle and fresh construction discipline;
- C.4 qualified physical media, real filesystem effects, barrier semantics,
  and production yieldpoints;
- C.5 durable physical record identity and format;
- C.5.1 one bounded physical work scheduler and executor path;
- C.6 bounded Recovery-scoped allocation and physical residency law; and
- C.7 WAL-before-data ordering, pageLSN binding, namespace-durable checkpoint
  and root publication, bounded idempotency compaction, retained WAL tail, and
  persisted physical operation identity.

The [C.7 successor handoff](physical-reconstruction-c7-durable-publication-join.md#c8-successor-handoff)
is a crash-recovery contract, not a Rust value that survives process death.
Its physical truth must be reopened from persisted artifacts; its backend
profile, static configuration, and resource requirements must be freshly
admitted in the new process.
`PhysicalDurabilityRecoveryHandoff` may describe orderly C.7 closeout for
observation and certification, but C.8 entry must not accept it, serialize it,
or reconstruct it from a copied field set. C.8 independently reopens the
artifacts from which its facts were derived.

C.8 must preserve the following inherited truths:

- stable Store identity survives restart while runtime identity never does;
- only C.4 executes physical media effects;
- ordinary and reconstructive execution remain separate lanes;
- WAL is physical redo authority, never semantic transaction authority;
- pageLSN is physical replay currency, never branch or commit meaning;
- checkpoints bound reconstruction but do not become unrebuildable truth;
- C.7 acknowledgment authority is not reconstructed or reissued by C.8;
- unresolved and unexpired idempotency bindings remain bounded and cannot be
  reclaimed merely because a process died; and
- no Query, Relational, Signal-graph, Bridge, MVCC, branch, or semantic-writer
  authority crosses the C.8 boundary.

## Governing Boundary

C.8 owns:

- fresh-process recovery admission;
- bounded discovery of C.7 physical recovery artifacts;
- deterministic current/previous root, checkpoint, WAL-tail, page, manifest,
  compaction-product, and residue precedence;
- valid WAL-prefix admission and torn-tail classification;
- immutable checkpoint-plus-tail redo planning;
- idempotent page redo into a non-current staging generation;
- physical operation-fate reconciliation;
- quiescence and new-runtime construction;
- namespace-durable recovered-root publication;
- post-publication cleanup eligibility and execution; and
- the narrow physical handoff to later readmission.

C.8 does not own:

- semantic state reconstruction or validation;
- branch selection, transaction visibility, MVCC, Query plans, or writer
  admission;
- ordinary mutation, acknowledgment delivery, checkpoint creation policy, WAL
  append policy, compaction strategy, repair, PITR, backup, restore, replica
  promotion, or disaster-recovery source selection;
- comprehensive corruption localization, quarantine policy, or repair
  authorization reserved for C.9;
- stable-reader, reclaim, epoch, latch, or maintenance-interference law
  reserved for C.10; or
- historical explanation, diagnostics, or evidence materialization as a
  prerequisite for ordinary recovery.

Existing implementations of backup, restore, PITR, rollback, replica
bootstrap, corruption readmission, and forensic observation do not expand C.8.
They may consume the final C.8 handoff later, but they cannot force a
compatibility facade, alternate recovery entry, or wider C.8 authority.

## Adversarial Constraint

Assume the writer is killed at the worst named production yieldpoint after one
or more physical effects may have escaped but before the next in-memory state,
receipt, acknowledgment, or cleanup action exists. The durable namespace may
contain a valid previous root, a newly durable current root, an incomplete root
replacement, a fuzzy checkpoint, a contiguous WAL prefix with a torn final
frame, page images at mixed pageLSNs, a partially visible compaction product,
unresolved operation bindings, and unrelated backend residue.

The writer's heap, scheduler, Signal graph, buffer pool, handles, counters,
decoded artifacts, and runtime identity are destroyed. A different executable
in a different process receives only the store root, static configuration,
backend profile request, recovery limits, and an output-evidence path. Its
composition root must acquire exclusive root ownership, qualify the backend,
and mint the concrete platform authority inside that fresh process. It must
derive exactly one lawful outcome without scanning from genesis or treating
directory contents as current authority.

A recovery design is false if it can pass by:

- preserving any writer-owned object or global state;
- accepting `PhysicalDurabilityRecoveryHandoff` at the recovery entry;
- choosing the highest generation found by directory enumeration;
- falling back heuristically until some artifact decodes;
- replaying every WAL segment or walking every page in the Store;
- applying redo without pageLSN and generation comparison;
- resolving absent evidence as proven no effect;
- publishing a recovered root before the staging generation is closed;
- constructing a serving runtime before namespace durability;
- deleting the previous root or required WAL before the recovered root has
  been durably published and independently reopened; or
- letting the verifier call recovery selection, redo, or runtime decoding.

## Decisive Fresh-Process Suite

### Production subjects

The suite uses four real process roles:

1. The dedicated `physical_store_c8_writer` executable uses the canonical
   ordinary Store facade and only production C.4 effects. It is a shipped
   production binary with no certification authority or replay import.
2. `physical_store_recover` uses the C.8 recovery composition facade.
3. The existing `physical_store_offline_observer` executable runs in its C.8
   observation mode, opens the dead store read-only, and interprets stable
   physical formats through an independent decision path.
4. The parent harness owns the deterministic history model, process lifecycle,
   schedule, fault program, and comparison. It owns no Store authority.

The writer and recoverer must have different operating-system process
identities. The recovered physical runtime must also have a runtime identity
different from every identity emitted before the crash.

### Initial world

The Store is larger than the admitted recovery memory budget. The writer
performs deterministic multi-page mutations across at least two checkpoint
intervals and two WAL rotations. The history includes:

- acknowledged durable operations;
- operations whose durable completion precedes acknowledgment construction;
- proven-no-effect operations;
- operations killed at an effect-ambiguous seam;
- a fuzzy checkpoint with pages both below and at or above tail redo LSNs;
- current and immediately previous root generations;
- one fully durable compaction cutover and one incomplete product;
- an unexpired terminal idempotency binding and an unresolved binding; and
- unrelated discoverable residue that is structurally plausible but not
  selected by persisted authority.

The parent records submitted identities and expected semantic record values in
its own history model. Neither the recoverer nor observer receives that model.

The Phase 8 checkpoint-crash fixture uses an independently modeled C8 v1 dirty
record whose payload is `2 * (16 KiB - 112 bytes) + 1 = 32,545` bytes. That
record produces one new-artifact extent frame followed by existing-artifact
writebacks, so the parent must observe the real pre-effect writeback boundary.
The C8 writer requires an explicit sealed `--writer-durability-profile` before
record-store initialization. Checkpoint and ordinary writeback worlds use
`c8-phase8-checkpoint-writeback-v1`, whose local WAL segment limit is 128 KiB;
the workspace runtime admission and WAL rotation rules remain unchanged, and
the checkpoint fixture still crosses multiple physical WAL segments. Cleanup
seam worlds use the distinct `c8-phase8-cleanup-rotation-v1` profile, whose
24 KiB segment limit fits each cleanup durability group while forcing the
checkpoint group and its durable-unacknowledged tail into separate physical
segments. The parent cleanup-world owner independently parses those raw WAL
frames and refuses to enter a cleanup seam unless it proves one complete
checkpoint-covered segment, one retained tail segment, contiguous LSN/segment
topology, and no torn suffix. No arbitrary numeric WAL limit is accepted from
the process boundary.

The separate durable-before-ack process world intentionally uses the ordinary
8 KiB mutation payload and fate `2`; its parent expectation and writer profile
are selected by that semantic launcher profile rather than by overriding the
checkpoint fixture's 32,545-byte payload or fate `4`.

### Recovery profiles and memory budgets

Ordinary Phase 8 recovery uses the named `c8-phase2-admission-v1` profile and
its admitted 512 KiB recovery-memory budget. The checkpoint-fate and recovery-
yieldpoint campaigns use the explicitly named
`c8-phase8-fate-coverage-v1` profile with a separate 4 MiB admission because
their deliberately oversized multi-checkpoint fixture must retain enough
physical evidence to cross every named boundary. Those campaigns assert their
own 4 MiB bound; they are additional crash-composition evidence and do not
substitute for the ordinary 512 KiB budget proof.

### Hostile sequence

For every named C.7 yieldpoint, the parent:

1. creates a clean store through the production writer;
2. drives the declared deterministic operation history;
3. pauses at the exact production yieldpoint;
4. terminates the writer without close, drain, destructor cleanup, or a final
   in-process report;
5. records only process-external facts already visible to the parent;
6. launches the offline observer against the dead bytes;
7. launches the recovery executable with only the declared public inputs;
8. launches a second fresh recovery observation over the published result; and
9. compares writer history, observer report, recovery report, final artifacts,
   exact counters, and runtime identities.

The required crash seams include:

- before and after WAL frame creation;
- after a partial WAL frame write;
- after complete WAL write but before its durability barrier;
- after WAL durability but before data publication;
- during a multi-page data publication;
- after data durability but before root replacement;
- after root replacement but before namespace synchronization;
- during checkpoint candidate write, checkpoint synchronization, selector
  replacement, and selector namespace synchronization;
- after physical completion but before acknowledgment construction;
- after acknowledgment construction but before caller observation;
- during compaction product publication and cutover; and
- during C.8 staging write, recovered-root replacement, namespace
  synchronization, independent reopen, and cleanup.

### Required outcomes and observations

For every case:

- the selected root/checkpoint/WAL basis is deterministic for identical bytes;
- current namespace authority outranks directory discovery and residue;
- a torn final WAL suffix is rejected without discarding its valid prefix;
- a gap, overlap, stale generation, or invalid frame before the required
  durable frontier blocks recovery rather than becoming a tail truncation;
- redo applies only when target generation agrees and pageLSN is below the
  frame LSN;
- repeated recovery over the same admitted basis converges to identical page
  bytes, roots, fates, and counters;
- no acknowledged durable physical operation is absent from recovered state;
- no operation is classified as proven no effect from mere non-observation;
- every submitted stable physical identity is classified by the weakest exact
  persisted fate or remains `Indeterminate`;
- the recovered runtime and all handles are fresh and quiescent;
- no semantic authority or semantic conclusion appears in the handoff;
- recovery work remains within the exact source, WAL, redo, staging, memory,
  and cleanup bounds; and
- cleanup cannot remove any artifact used by either the selected basis or the
  independently reopened recovered root.

The observer reports artifact identities, generation links, durable selectors,
checkpoint coverage, valid WAL prefix, pageLSNs, manifest membership, and
residue. It must not return a Store recovery type or mint recovery authority.
Runtime/observer disagreement is explicit evidence and blocks closeout; the
runtime may not silently normalize the observer to its own answer.

### Schedule perturbation

Every staged process scenario has a deterministic schedule seed and a distinct
perturbation seed derived from its stable scenario identity. CI runs the
explicit scenario matrix. The release lane runs the canonical schedule plus
the complete named crash matrix.

A failure identifies the scenario, seeds, backend profile, and yieldpoint.
Replaying those values must reproduce the same failure on unchanged source.

### Hostile sensitivity

Direct C.8 tests must reject at least these defect classes:

- accept the C.7 in-memory handoff or reuse writer runtime identity;
- choose the highest enumerated root or checkpoint generation;
- let residue outrank a durable selector;
- accept a noncontiguous WAL tail;
- treat middle corruption as a torn final suffix;
- ignore Store incarnation in source or operation identity;
- ignore target generation during redo;
- invert or delete the pageLSN skip/apply comparison;
- apply the same non-idempotent redo twice;
- promote missing operation evidence to proven no effect;
- promote durable-unacknowledged work to acknowledged;
- construct the handoff before recovered-root namespace durability;
- scan from WAL genesis or scale discovery with total Store size;
- delete the previous root, selected checkpoint, required WAL, or unresolved
  binding before safe publication; and
- let the offline observer call recovery-runtime or source-precedence code;
- let a sealed type-level recovery witness substitute for the exact root,
  session, backend-profile, media-generation, configuration, or budget binding;
- omit one recovery binding axis while every remaining axis still agrees;
- accept a caller-supplied checkpoint-generation sample or freshness policy for
  idempotency retention or cleanup revalidation;
- promote admitted, scheduled, or attempted media work to a performed staging,
  publication, reopen, or cleanup effect;
- let one recovery session terminate twice or disappear without owner-visible
  non-terminal-drop evidence; and
- decode a recovery or observer report under the wrong protocol family or an
  unsupported protocol version, or let that descriptive protocol authorize a
  persisted Store artifact.

Every real C.8 defect fixed later adds the smallest direct regression test that
would have exposed it at the responsible boundary.

### Forbidden substitutes

The following cannot close C.8:

- same-process reopen;
- graceful close or abort represented by `Err` while the writer survives;
- a supplied `PersistedPhysicalLayout` or decoded artifact graph;
- a test-only recovery constructor or media path;
- replay from an expected-history file;
- copied production parsing in the observer;
- a source-text substring gate standing in for runtime or type evidence;
- elapsed time without structural cost counters; or
- a recovery report produced without publishing and independently reopening
  the recovered root.

## Product Decision Lock

1. **Persisted artifacts are the only crash-surviving authority.** The C.7
   handoff defines required facts but its in-memory value is not a C.8 input.
2. **Recovery has a separate reconstruction-band composition root.** A new
   `worth-store-recovery-runtime` crate owns fresh-process orchestration.
   `worth-store` remains the ordinary runtime owner and exposes only two narrow
   reconstruction-band ports: recovery-freshness sampling and final
   recovery-construction. `worth-store-recovery-physics` owns pure selection,
   redo, fate, and cost law.
3. **Recovery physics is not an executable catch-all.** By C.8 closeout it does
   not depend on `worth-store`, execute media effects, host the independent
   verifier, or export unrelated backup/PITR/rollback/replica workflows through
   the C.8 facade.
4. **Discovery and authority are separate.** Directory enumeration may produce
   bounded candidates. Only stable Store identity, durable selectors,
   generation linkage, checkpoint coverage, WAL continuity, and admitted
   format facts can select current truth.
5. **Selection and planning are effect-free.** All source precedence, fate
   reconciliation inputs, pageLSN decisions, expected counters, and branches
   are fixed in an immutable plan before staging writes begin.
6. **Redo never mutates the selected source generation.** It writes a distinct
   staging generation. Publication consumes a closed staging proof.
7. **A recovered root is not current until namespace durable.** A visible
   rename or replace without the required namespace barrier opens no handoff.
8. **Recovery produces a quiescent physical runtime, not a serving runtime.**
   It carries a new runtime identity and fresh bounded handles but has no
   semantic writer, Query, Signal-graph, or ordinary mutation authority.
9. **Operation fate is evidence-limited.** Absence is never no-effect proof.
   Indeterminate is a successful classification when persisted evidence cannot
   lawfully decide more.
10. **Cleanup is post-commit maintenance.** Recovery success does not depend on
    deleting residue. Cleanup failure is reported as deferred work and cannot
    invalidate an already durable recovered root.
11. **No compatibility lane survives cutover.** Existing unreleased APIs are
    preserved only when they already express the destination contract. Wrong,
    redundant, speculative, milestone-coded, or duplicated surfaces are
    narrowed, moved, replaced, or deleted.
12. **Behavioral tests are the verdict.** Warnings-denied compilation, focused
    tests, boundary checks, and direct process scenarios run after each coherent
    implementation batch. No ledger, report reconciliation, source fingerprint,
    or test-of-tests may stand between a defect and its failing test.
13. **A sealed witness proves the lane, not the recovery instance.** C.8 uses
    the Worth Proof sealed marker-authoring pattern for private minting, but the
    concrete Store authority also retains a private exact binding. A zero-sized
    witness, marker type, or successful equality check is never root-, Store-,
    session-, profile-, generation-, configuration-, or budget-specific
    authority by itself.
14. **Recovery bindings declare every axis once.** Store-owned concrete entry
    and admitted-world bindings use Worth Proof `Binding`/`BindingAxes`
    machinery privately beneath their sealed types. Across the progression the
    declared axes cover root-ownership identity, stable Store identity once
    admitted, recovery-session identity, backend-profile identity, qualified
    media generation, static-configuration identity, and recovery-limit
    identity. A successful comparison returns no reusable authority token; the
    same owner continuation constructs the exact next concrete state.
15. **Freshness is sampled by the owner.** Idempotency lease expiry is evaluated
    against the selected namespace-durable checkpoint generation, never wall
    clock time. Cleanup-plan revalidation is evaluated against the current
    published-root generation. The Store owner supplies the concrete
    `FreshnessSource`, sealed establishment basis, and policy; callers, reports,
    adapters, and fixtures cannot supply a sample, source, or policy.
16. **Admission is not performance.** Worth Proof `Performed` may exist only
    privately beneath concrete Store effect evidence recorded in the same owner
    function after the exact C.4 effect completes. Staging writes, recovered-
    root replacement, namespace synchronization, independent reopen, and each
    cleanup removal have distinct action kinds and exact dynamic bindings.
    Scheduler completion, permission, intent, attempted I/O, counters, and
    ambiguous effects cannot mint performed evidence.
17. **One recovery session ends once.** The concrete recovery session uses the
    Worth Proof `LinearResource` law privately as
    `LinearResource<RecoverySessionIdentity, PhysicalRecoveryTerminal,
    PhysicalRecoverySessionAuthorityMarker>`, with terminal variants matching
    `Recovered`, `Refused`, `Blocked`, and `PublicationIndeterminate`. The
    recovery runtime owns identity issuance, enumeration, non-terminal-drop
    detection, and quiescence; the generic linear value or terminal receipt is
    not public recovery authority.
18. **Boundary protocols describe reports, not Store truth.** The recovery
    executable emits protocol family `store.physical.recovery-report` version
    1, and the offline observer emits
    `store.physical.recovery-observer-report` version 1. C.8 consumers initially
    admit exactly version 1 through a Foundational compatibility window.
    Identity mismatch or a version that predates, exceeds, or has been retired
    from a declared window is typed before payload interpretation. These
    protocol values never admit roots, checkpoints, WAL, pages, effects, or a
    handoff, and the initial one-version windows create no compatibility facade.

## Semantic Vocabulary Lock

The following terms have one C.8 meaning:

- **recovery session**: one fresh-process attempt bound to one stable Store
  identity, one newly minted recovery-session identity, one qualified backend
  profile, and one finite recovery budget;
- **Store incarnation**: the stable `StableStoreIdentity` minted when the
  physical namespace is created; it survives process restart and changes on
  destructive Store reinitialization, so C.8 introduces no competing
  incarnation identifier;
- **candidate**: a bounded discovered artifact that has no current-truth
  authority;
- **admitted source**: a candidate whose stable identity, schema, generation,
  integrity minimum, and linkage are valid for one role;
- **selected basis**: the unique root, checkpoint, WAL-tail, page, and binding
  cut chosen by persisted precedence law;
- **valid WAL prefix**: the maximal contiguous admitted prefix beginning at the
  selected checkpoint redo frontier;
- **torn tail**: an incomplete or invalid suffix at the physical end of the
  newest otherwise contiguous WAL segment family;
- **middle corruption**: an invalid frame before a later required or valid
  frame, or before the durable frontier; it is never a torn tail;
- **redo plan**: the immutable ordered apply/skip/deny decisions derived before
  effects;
- **staging generation**: a non-current physical generation receiving recovery
  effects and incapable of serving reads or mutations;
- **published recovered generation**: a closed staging generation selected by
  a recovered root whose replacement and namespace durability are proved;
- **physical operation fate**: the weakest exact crash-surviving conclusion for
  one Store-incarnation-scoped physical operation identity;
- **quiescent physical handoff**: the C.8 output containing fresh physical
  handles and current-root authority but no semantic or serving authority;
- **residue**: discoverable material that is not selected authority;
- **cleanup candidate**: residue whose deletion is separately proved safe
  after recovered-root publication and independent reopen; and
- **observer report**: read-only independent evidence that can disagree with
  recovery and can never authorize it;
- **recovery authority binding**: the complete Store-owned set of dynamic axes
  against which one concrete recovery authority or phase state was issued;
- **performed physical effect evidence**: a concrete Store wrapper recorded
  only after one exact C.4 action completed for one bound recovery session and
  subject; and
- **report protocol envelope**: a Foundational family identity and positive
  version preceding a descriptive cross-process payload and carrying no Store
  admission authority.

Do not use `valid`, `current`, `complete`, `durable`, `recovered`, `safe`, or
`clean` without the exact subject and proof boundary. A decoded artifact is not
an admitted source. A selected source is not a published generation. A
published generation is not a serving runtime. A diagnostic report is never a
recovery witness.

## Authoritative And Derived Truth

### Authoritative persisted inputs

C.8 may derive current physical truth only from:

- stable Store namespace identity and incarnation-bearing artifact identities;
- the durable current/previous selector cells and their generation linkage;
- root and checkpoint manifests admitted under their exact schemas;
- checkpoint-covered LSN frontier and bounded idempotency compaction;
- contiguous retained WAL segment identities, frame ranges, attempt bindings,
  redo payloads, and durable frontier evidence;
- admitted page and extent identities, generations, pageLSNs, and manifest
  membership;
- admitted compaction cutover records and old-generation recoverability facts;
  and
- backend profile facts required to interpret the C.7 durability barriers.

Phase 1 distinguishes existing persisted producers from required producer
gaps. A gap row is not persisted authority and cannot be substituted with a
derived recovery posture. Phase 2 delivers the durable root-selector protocol:
genesis persists one unlinked current selector because it has no predecessor;
each successor publication stages reciprocally linked previous/current
selectors and publishes both with the bootstrap catalog under one observed
compound root-protocol effect. Phase 3 persists the compaction-cutover record
inside the checkpoint binding stream and admits it only after whole-stream
checkpoint verification. Phase 4 adds the concrete checkpoint security binding
to that whole-stream format and reconstructs its policy and retention axes only
after verification. WAL security does not gain a parallel decorative frame
header: Phase 4 derives it from a verified WAL frame whose payload contains the
exact persisted C.7 attempt binding and canonical redo digest, decoded by the
Store owner. The persisted-input contract binds both producer and decoder
chains so later work cannot silently substitute an in-memory receipt, identity
wrapper, or derived classification for crash-surviving truth.

The successor triplet is an ordered compound effect, not one filesystem
transaction. A crash after publishing only the previous selector, or after
publishing previous plus current while the bootstrap catalog remains old, is
an expected indeterminate input. Phase 3 must discover both fixed selector
slots, validate their reciprocal identity and generation linkage against the
referenced roots, and apply the selection table below; it must not infer truth
from triplet completion, the bootstrap catalog alone, filename order, or mtime.
When the current selector is undecodable, the previous selector may be used
only when its linked successor selector identity and linked root generation
both equal the current-root generation in the independently decoded bootstrap
catalog, and Store plus format identity also agree. This is a combined
selector-linkage proof: the catalog is corroborating publication evidence and
never selects a root by itself. A missing, damaged, stale, foreign-Store, or
wrong-format catalog blocks torn-current fallback. A valid current selector
continues to outrank an older catalog in the legal two-of-three publication
prefix.

The recovery request's configuration and qualified platform authority decide
what may be opened and what budgets apply. They do not decide which persisted
generation is current.

A C.7 death after synchronizing a root candidate but before root-protocol
replacement may leave complete artifacts at the exact successor generation.
Those bytes are never current authority. When recovery must publish that same
successor, effect-free planning may read only that addressed candidate and its
referenced manifest blocks under the existing observation and manifest-entry
limits. Recovery independently derives the exact incremental copy-on-write
successor required by the admitted physical-format contract from the selected
current topology and persisted recovery projection. Adoption requires exact
root and free-space headers, exact reused selected-source references in their
expected positions, and exact identity and canonical bytes for every
successor-owned block. Recovery's candidate-absent full reconstruction is a
different lawful strategy and is not the adoption oracle. A malformed,
semantically conflicting, differently packed, or differently sourced candidate
blocks before effects. Recovery never skips a generation, overwrites mismatched
immutable bytes, or deletes a candidate to make publication proceed. If no
publication is required, non-current successor material cannot displace or
invalidate the selected root.

### Derived state

These are derived and may be destroyed and recomputed:

- candidate inventories and discovery traces;
- source-precedence graphs and decision reports;
- redo indexes, page worklists, skip sets, and execution ordering;
- operation-fate joins and diagnostic explanations;
- recovery counters and performance evidence projections;
- offline observer reports; and
- cleanup plans.

If deleting any derived state prevents recovery from the authoritative inputs,
the implementation has promoted a cache into authority and fails C.8.

## Deterministic Source-Precedence Contract

Precedence is role-specific. No generic score, confidence value, timestamp,
filename order, or highest-generation heuristic may select authority.

| Conflict | Required decision | Forbidden decision |
| --- | --- | --- |
| visible current root and durable previous selector | use the root bound by the valid durable current selector; retain previous as fallback evidence | choose by filename, mtime, or generation maximum |
| torn or unsupported current selector with valid retained previous selector | select the exactly linked previous root or block if linkage cannot be proved | scan for any decodable root |
| valid current selector names an invalid/missing root | block with exact current-root denial unless the persisted selector protocol itself authorizes the retained previous slot | silently demote current because previous opens |
| several checkpoint files are present | admit only the checkpoint selected by the chosen root/selector chain and exact Store/generation binding | choose the newest enumerated checkpoint |
| selected checkpoint plus older valid checkpoint | use the selected checkpoint; retain the older one only as declared fallback/cleanup evidence | merge their page or binding contents |
| checkpoint redo frontier plus retained WAL | require the first selected complete tail-frame LSN to be exactly contiguous with the checkpoint frontier, even when that frame is inside a retained physical segment | search later segments for a convenient start or treat the containing segment's physical first LSN as the logical tail start |
| incomplete final WAL frame/suffix after a valid prefix | admit the prefix, classify and exclude the torn suffix, preserve evidence until safe cleanup | reject the prefix or decode partial payload |
| invalid frame followed by required/later valid material | block as middle corruption or missing required range | truncate it as a torn tail |
| pageLSN at or above redo LSN with matching generation | skip the frame and record the skip | reapply because WAL is newer in enumeration order |
| pageLSN below redo LSN with matching generation | apply exactly once in plan order | skip because page bytes decode |
| page generation differs from redo target | deny or route to an explicitly planned rebuild branch | apply by page identifier alone |
| complete compaction product with admitted durable cutover | use it only in the role named by the selected root/manifest | let presence imply visibility |
| incomplete or unreferenced compaction product | classify as residue; never select it | treat higher generation as current |
| plausible backend residue | retain as observation or cleanup candidate | use it to fill missing authority |

Candidate discovery is bounded by the exact current/previous selector slots,
the selected checkpoint family, the declared retained WAL segment window, and
manifest-addressed pages or extents. C.8 may not recursively enumerate the
entire store tree to prove completeness.

Format or integrity failure required to prevent unsafe decode belongs in the
C.8 admission path. Comprehensive damage localization, quarantine policy, and
repair choice remain C.9 responsibilities. C.8 reports an exact blocking or
unsupported physical scope instead of inventing repair.

## WAL Prefix And Redo Contract

The selected checkpoint contributes the base physical image and exact covered
LSN frontier. The retained WAL contributes only the contiguous suffix after
that frontier. “First tail LSN” means the first selected complete frame, not
necessarily the first frame in the physical segment that contains it; frames
covered by the checkpoint remain physical evidence but are excluded from the
logical recovery basis.

Before effects, C.8 must prove:

- every segment belongs to the same Store incarnation and admitted WAL family;
- segment generation and LSN ranges are ordered, nonoverlapping, and gap-free;
- frame boundaries, schema, integrity minimum, member identity, redo digest,
  and target generation are admitted;
- the valid prefix contains every range required by acknowledged or unresolved
  bindings in the selected basis;
- a rejected torn suffix is physically terminal;
- each redo target is bounded and addressable through manifest truth; and
- the complete plan fits admitted scan, frame, redo, staging, and memory limits.

`RecoveryRedoPlan` is immutable and fixes each frame as exactly one of:

- `Apply(PageRedoStep)`;
- `Skip(PageAlreadyAtOrBeyondLsn)`;
- `Skip(OperationAlreadyMaterialized)`; or
- `Block(RedoPlanningDenial)`.

Execution accepts only the all-admitted plan. It cannot rediscover candidates,
change source precedence, reinterpret a frame, widen a target, or add an
unplanned fallback. Page application consumes the prior staged-page proof and
produces a new page proof with the exact resulting pageLSN and digest.

Redo is idempotent in recovered state, not necessarily by repeating arbitrary
mutation code. The recovery record grammar and pageLSN/generation contract must
make a second recovery over identical bytes converge by skipping or producing
the identical staged bytes.

## Physical Operation-Fate Contract

Every fate is indexed by stable Store identity, Store incarnation, physical
operation identity, idempotency identity and lease, canonical request
fingerprint, and persisted attempt binding when one exists. Local ordinals,
runtime identities, group positions, or caller memory are insufficient.

| Fate | Minimum persisted proof | Caller meaning |
| --- | --- | --- |
| `AcknowledgedDurable` | C.7 acknowledgment-sealed terminal fact plus matching durable operation/root coverage | the same physical request is complete; this does not prove a particular client observed delivery |
| `DurableUnacknowledged` | matching durable WAL/data/root effect and terminal or reconstructed completion, without acknowledgment-sealed proof | effect is complete; retry must resolve as the same completion, not execute again |
| `ProvenNoEffect` | persisted no-effect/cancellation terminal fact or a closed admitted interval whose exact operation binding is proved never to have crossed its first irreversible effect | the same request may follow the C.7 no-effect continuation law |
| `Indeterminate` | the identity is admitted but available persisted evidence cannot prove completion or no effect | no automatic retry, acknowledgment, or destructive cleanup is authorized |

Mere absence from a checkpoint, WAL scan, page image, report, or observer output
cannot construct `ProvenNoEffect`. An expired terminal binding may be absent
only when the selected namespace-durable checkpoint proves that C.7 lawfully
compacted it. Every unresolved or unexpired binding remains represented.

The parent process suite may separately know whether a client observed a response.
C.8 does not infer network or caller observation. `AcknowledgedDurable` names a
persisted C.7 physical acknowledgment fact, not delivery telemetry.

Fate reconciliation is effect-free and completes before root publication. The
final fate set is part of the closed staging basis and cannot be changed during
publication or cleanup.

## Normative Public API

Names below are normative semantic surfaces. Private representation and exact
method decomposition remain implementation-plan decisions.

### Fresh-process entry

```rust
let request = PhysicalRecoveryOpenRequest::declare(
    store_root,
    static_store_configuration,
    qualified_backend_profile,
    recovery_limits,
    recovery_platform_authority,
);

match WorthStoreRecovery::recover(request) {
    PhysicalRecoveryOutcome::Recovered(handoff) => {
        // A later readmission owner may consume `handoff`.
    }
    PhysicalRecoveryOutcome::Refused(refusal) => {
        // No recovery effect occurred; correct the exact admission cause.
    }
    PhysicalRecoveryOutcome::Blocked(blocked) => {
        // Persisted truth is insufficient, unsupported, or damaged.
    }
    PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => {
        // Do not retry publication in-process; reopen from persisted bytes.
    }
}
```

`PhysicalRecoveryOpenRequest::declare` accepts no live `Store`,
`ServingPhysicalRuntime`, `PhysicalDurabilityRecoveryHandoff`, buffer-pool
handle, Signal graph, scheduler, decoded artifact collection, expected record
model, or prior runtime identity.

`PhysicalRecoveryPlatformAuthority` is one concrete sealed platform authority
minted by the Store composition root after exclusive root ownership, backend
qualification, and recovery-mode lifecycle admission. It is bound to the exact
root and attempt and is consumed by entry admission. Its private Worth Proof
marker/witness proves only the recovery-platform lane. A private
`PhysicalRecoveryEntryBinding` additionally retains root-ownership identity,
recovery-session identity, backend-profile identity, qualified-media
generation, static-configuration identity, and recovery-limit identity. After
the persisted stable Store identity is admitted, the successor concrete world
binding retains that identity as an additional axis. Generic `AuthorityMarker`,
a bare `AuthorityWitness`, a bare `Binding`, copied digests, profile labels,
Foundational receipts, or public constructors cannot substitute for the
concrete authority.

`physical_store_recover` mints that authority inside the fresh process before
constructing the request. The authority is never accepted from command-line,
wire, report, or serialized input. Axis comparison occurs inside entry
admission and returns only `()` or an exact per-axis drift denial; a match does
not escape as a reusable admission token.

### Compiler-visible progression

The runtime progression is consuming and non-forgeable:

```text
PhysicalRecoveryOpenRequest
    -> AdmittedPhysicalRecovery
    -> DiscoveredPhysicalRecovery
    -> SelectedPhysicalRecovery
    -> PlannedPhysicalRecovery
    -> StagedPhysicalRecovery
    -> NamespaceDurablePhysicalRecovery
    -> ReopenedPhysicalRecovery
    -> RecoveredPhysicalRuntimeHandoff
```

- `AdmittedPhysicalRecovery` proves exclusive recovery lifecycle, exact Store
  identity, qualified backend profile, new recovery-session identity, concrete
  platform authority, finite budgets, and one exact admitted-world binding.
- `DiscoveredPhysicalRecovery` owns bounded candidates and counters but no
  selected truth.
- `SelectedPhysicalRecovery` owns one deterministic source basis and exact
  rejected/residue classifications.
- `PlannedPhysicalRecovery` owns the complete immutable redo, fate, staging,
  publication, quiescence, and expected-cost plan.
- `StagedPhysicalRecovery` proves all planned redo is settled in one closed
  non-current generation with no live recovery work.
- `NamespaceDurablePhysicalRecovery` proves root replacement and required
  namespace durability from distinct concrete performed-effect values recorded
  after those C.4 actions.
- `ReopenedPhysicalRecovery` proves the published generation was opened again
  through fresh handles and agrees with the closed plan, using concrete
  performed reopen evidence bound to that generation and session.
- `RecoveredPhysicalRuntimeHandoff` carries the new runtime identity, verified
  physical roots, bounded access capabilities, operation fates, unsupported or
  quarantined scope, cleanup posture, and exact counters.

Only the predecessor constructs its successor. No public constructor,
`Clone`, serialization round trip, report, digest, or evidence projection can
mint a progression state. Earlier states cannot execute later effects. Bare
Worth Proof `Performed`, `DerivedFrom`, `Inverts`, `TerminalReceipt`, or a
successful binding comparison cannot satisfy a governed C.8 surface; only the
Store owner can wrap the applicable substrate in the exact concrete successor.

The recovery session itself is a concrete Store type over the Worth Proof
linear-resource law. Exactly one top-level outcome consumes it into the
matching terminal receipt. The owner runtime separately records every live
session and treats non-terminal drop as a lifecycle defect; Rust's move-only
value alone is not accepted as leak detection or quiescence proof.

### Outcome topology

`PhysicalRecoveryOutcome` has exactly these top-level meanings:

- `Recovered(RecoveredPhysicalRuntimeHandoff)` — current root is namespace durable,
  independently reopened, and safe to hand off; cleanup may be complete or
  explicitly deferred;
- `Refused(PhysicalRecoveryRefusal)` — entry or planning rejected before any
  staging or publication effect;
- `Blocked(PhysicalRecoveryBlock)` — persisted authority is missing,
  conflicting, unsupported, damaged, or outside admitted reconstruction bounds;
  no recovered current root was published; and
- `PublicationIndeterminate(PhysicalRecoveryPublicationIndeterminate)` — a C.8
  publication effect may have escaped, no handoff is issued, and a new process
  must reopen the bytes to resolve it.

Failures preserve exact Store, session, source, artifact, generation, LSN,
effect, budget, and recovery-direction context relevant to their cause. A
poisoned lock, unsupported format, missing range, backend denial, capacity
deferral, foreign Store, and integrity block must not collapse into `Io`,
`Invalid`, or `RecoveryFailed`.

Phase 3 carries source denials as typed evidence rather than reconstructing
them from counters: each rejected root slot retains its expected role, decoded
Store, observed role and generation when available; checkpoint failures retain
the exact stream or root-binding denial; and WAL failures retain the canonical
artifact identity, integrity denial, or continuity denial. One blocked outcome
may retain several source denials when several canonical artifacts fail. A
root-slot observation denial does not replace the resulting root-selection
denial: a torn current selector plus an absent or unlinked previous selector
retains both causes. Manifest routing observation likewise retains the exact
addressed reference and distinguishes duplicate reference, missing artifact,
decode, format identity, tree identity, and reference-integrity failures.
Backend observation failures retain the typed fixed record, checkpoint, WAL
directory, or exact WAL member address together with the backend failure kind
and operating-system error kind when one exists; they never collapse to an
unattributed media error.

### Narrow Store freshness port

`worth-store` exposes a separate reconstruction-only
`PhysicalRecoveryFreshnessPort`. The entry authority derives its concrete
`PhysicalRecoveryFreshnessAuthority`; callers cannot construct or substitute
that authority. `sample_binding` returns a
`StoreRecoveryBindingFreshnessSample` containing the owner-sampled selected
checkpoint generation, sealed C.7 basis identity, and concrete policy
identity. Cleanup freshness is sampled only while the Store consumes one
opaque admitted cleanup plan; `StoreRecoveryCleanupAttempt::freshness`
exposes the resulting current published-root generation, cleanup-plan
artifact identity, sealed publication basis, and concrete policy identity as
descriptive evidence. No public freshness-port method issues cleanup or media
effect authority. The recovery runtime may pass sampled values into pure
physics classification, but no sample authorizes an effect or crosses into
ordinary Store serving.

This freshness port is distinct from construction: it cannot accept a
recovery phase value, create cleanup or media-effect authority, publish a root,
or construct runtime handles.

### Narrow Store construction port

`worth-store` exposes one reconstruction-only
`PhysicalRecoveryConstructionPort`. It accepts only
`ReopenedPhysicalRecovery` plus the consumed concrete construction authority
and constructs the quiescent `RecoveredPhysicalRuntimeHandoff` with a new
runtime identity and fresh handles.

It cannot:

- accept a recovery report or observer conclusion;
- create source-selection, redo, or cleanup authority;
- construct `ServingPhysicalRuntime`;
- carry forward writer runtime identity, frame leases, queues, or counters; or
- be called from an ordinary Store feature or facade.

The dependency direction is:

```text
stable format / WAL / backend / budgets
                  |
                  v
worth-store-recovery-physics       worth-store
                  \                 /
                   v               v
              worth-store-recovery-runtime
                            |
                            v
             RecoveredPhysicalRuntimeHandoff
```

`worth-store` does not import recovery physics or replay. The reconstruction
composition crate depends on both participants and remains outside the ordinary
lane. `worth-store-recovery-runtime` may import Worth Proof only as private
contract substrate beneath its concrete Store types. Its `observation` module
and the offline verifier may import the Foundational facade for the locked
report protocols; source selection, redo, fate, publication, construction, and
cleanup do not accept Foundational protocol values.

## Signal, Worth Proof, And Foundational Law

### Worth Signal

No writer Signal graph survives the crash and no persisted Signal identity is
reconstructed. The fresh process may construct a new Signal graph and use the
existing C.5.1 scheduler as mechanism for bounded Recovery-scoped work, but
Signal does not select sources, determine fate, prove redo, or authorize root
publication.

C.8 installs Store-owned aspect-native bases for these exact work families:

| Store contract key | Role | Exact partition | May schedule | Cannot prove |
| --- | --- | --- | --- | --- |
| `store.physical.recovery.discovery-basis` | projection | stable Store identity plus recovery session | bounded artifact reads | candidate authority or current root |
| `store.physical.recovery.redo-basis` | mutation | selected basis identity plus staging generation | planned page/extent recovery writes | source selection or publication |
| `store.physical.recovery.publication-basis` | mutation | staging generation plus recovered-root identity | planned root replacement and namespace barrier | semantic or serving admission |
| `store.physical.recovery.cleanup-basis` | mutation | published generation plus cleanup-plan identity | proved post-publication removals | selection, recovery success, or last-copy deletion |

Store owns those contracts and derives the exact scheduler routing. Lower
format, WAL, backend, and recovery-physics owners remain Signal-agnostic. Raw
Signal bits, dependency observations, deadlines, cancellation, or scheduler
completion cannot mint a C.8 proof state.

Cancellation is honored only at declared safe points. Before an escaping
effect it returns an exact no-effect refusal with the consumed authority or a
lawful continuation. After a staging or publication effect may have escaped it
returns an exact retained or indeterminate recovery posture. Dropping a handle
is never cancellation.

### Worth Proof

Worth Proof supplies the private up-front contract substrate, consuming proof
outcomes, transition readiness, nonempty/ordered structural proofs, and
compiler-visible progression. C.8 uses those mechanisms through Store-owned
concrete wrappers for:

- privately minted recovery-platform and construction authority markers whose
  witnesses prove only their lanes;
- `Binding`/`BindingAxes` declarations for exact entry and admitted recovery
  worlds, with one generated drift kind and one hostile twin per axis;
- a concrete recovery-session wrapper over `LinearResource` with exactly one
  terminal outcome while Store retains the live registry and drop enforcement;
- owner-sampled generation freshness for idempotency binding expiry and cleanup
  retry revalidation;
- concrete performed-effect evidence recorded after exact C.4 staging,
  publication, namespace, reopen, and cleanup actions;
- exact root/checkpoint/WAL continuity joins;
- all-admitted plan construction;
- staging quiescence;
- namespace-durable publication;
- independent reopen; and
- safe cleanup eligibility.

Worth Proof does not own Store identity or recovery-session identity issuance,
live-resource registries, the selected checkpoint-generation source, source
precedence, WAL validity, pageLSN meaning, exact effect occurrence identity,
operation fate, current-root authority, or cleanup policy. `Branded` may prevent
substitution inside one lexical recovery scope, but it cannot replace the
process/runtime/Store identity values that cross phases or the final handoff.
`DerivedFrom` and `Inverts` may express action-kind law but cannot prove which
root, generation, artifact, or effect occurrence participated. Generic
`AuthorityMarker` bounds and bare generic Worth Proof carriers are forbidden on
governed public and cross-crate C.8 surfaces. All generic machinery remains
private beneath sealed concrete Store types.

### Worth Foundational and Store aspect-native

Foundational supplies stable boundary-envelope roles, canonical identities,
typed outcome and diagnostic posture, policy admission, and counter-backed
performance evidence where those facilities already fit. Store aspect-native
contracts declare the four recovery work families above. Foundational also
supplies the descriptive protocol family, positive version, compatibility
window, and typed unsupported-version posture for the two cross-process report
families:

| Producer boundary | Protocol family | Produced version | Initial consumer window | Authority limit |
| --- | --- | --- | --- | --- |
| `physical_store_recover` observation export | `store.physical.recovery-report` | 1 | inclusive 1 through 1 | report interpretation only |
| `physical_store_offline_observer` export | `store.physical.recovery-observer-report` | 1 | inclusive 1 through 1 | independent comparison only |

The protocol family is stable and version-free; the produced version is a
separate positive value. A consumer validates family identity and admits the
version through its declared window before interpreting payload fields.
Predates-window, exceeds-window, and retired postures remain distinct whenever
the declared window makes them applicable. Supporting version 1 does not create
an alternate C.8 recovery facade, and a future report version changes no Store
authority without a separately revised Store contract.

Foundational artifacts are one-way projections from executed Store recovery
facts. They cannot be accepted back as source admission, redo, publication,
fate, or cleanup authority. C.8 must not construct rich evidence objects, JSON,
or explanations on its ordinary recovery execution path unless the caller
explicitly requests that observation profile and pays its separate bounded
cost.

Store-owned persisted format identity, schema version, compatibility, and
migration law remain the only admission authority for selectors, roots,
checkpoints, WAL, pages, extents, manifests, and bindings. A Foundational report
protocol may describe those values after Store admission; it cannot reinterpret
old bytes, convert an unsupported physical format into a supported one, or
authorize recovery because a report version is accepted.

No Foundational branch, commit, merge, selected-node, selected-aspect,
skipped-scope, or semantic checkpoint vocabulary is applicable to C.8 physical
reconstruction.

## Production Authority Matrix

| Type or responsibility | Constructed by | Proves | Authorizes | Cannot authorize | Consumed by |
| --- | --- | --- | --- | --- | --- |
| `PhysicalRecoveryPlatformAuthority` | Store composition root from exclusive root lifecycle and qualified backend authority | this fresh process may attempt recovery for one root | entry admission only | current truth, redo, or serving | `PhysicalRecoveryOpenRequest` |
| `PhysicalRecoveryEntryBinding` | Store composition root from owner-issued dynamic identities | root ownership, session, profile, media generation, configuration, and limit axes agree with authority issuance | same-continuation entry admission comparison | reusable authority or persisted currentness | entry admission |
| `AdmittedRecoveryWorldBinding` | entry admission after persisted Store identity admission | the entry binding plus exact stable Store identity define one recovery world | construction of the concrete admitted phase in the same continuation | cross-session reuse or later effect authority | `AdmittedPhysicalRecovery` |
| `PhysicalRecoverySession` | recovery runtime over owner-issued session identity and private Worth Proof linear substrate | one live recovery attempt has not yet terminated | exactly one terminal C.8 outcome | quiescence, Store truth, or a second terminal transition | top-level outcome owner |
| `AdmittedPhysicalRecovery` | recovery runtime through concrete authority and limit admission | exact Store/session/profile/budget recovery world | bounded discovery | candidate selection or effects | discovery owner |
| `RecoverySourceCandidate` | bounded discovery | one artifact was found within declared scope | role-specific admission attempt | currentness | source admission |
| `AdmittedRecoverySource` | recovery physics from schema, Store/incarnation, generation, role, and integrity minimum | source is lawful for one role | participation in precedence | selection for another role | precedence owner |
| `SelectedRecoveryBasis` | source-precedence owner | unique root/checkpoint/tail/page/binding cut | immutable planning | media effects | planning owner |
| `RecoveryRedoPlan` | recovery planner from selected basis and exact limits | complete ordered apply/skip decisions and expected work | staging execution | re-selection or publication | redo executor |
| `RecoveryOperationFateSet` | fate owner from selected compaction, tail, and recovered effect facts | weakest exact fate for every retained operation identity | duplicate/readmission handoff facts | acknowledgment delivery or semantic retry | closed staging and handoff |
| `RecoveryBindingFreshness` | Store owner from sealed C.7 binding basis, selected checkpoint-generation source, and concrete policy | exact retained binding was classified against the owner-sampled selected generation | the matching retention, compaction, or unresolved-fate branch only | source selection, wall-clock expiry, or cleanup | fate planner |
| `PerformedRecoveryPhysicalEffect<Action>` | exact C.4 effect owner after the bound action completes | one named action completed for one session, subject, generation, and outcome | its one concrete successor transition | another action, ambiguous effect, serving, or reuse | staging, publication, reopen, or cleanup owner |
| `ClosedRecoveryStagingGeneration` | recovery executor after all planned work settles | exact staged bytes, pageLSNs, roots, fates, counters, and quiescence | recovered-root publication | serving | publication owner |
| `NamespaceDurableRecoveredGeneration` | C.4-backed publication owner | recovered root replacement and namespace barrier completed | independent reopen and cleanup planning | serving by itself | reopen owner |
| `ReopenedPhysicalRecovery` | fresh-handle reopen owner | published bytes agree with the closed plan | quiescent Store handoff construction | semantic readmission | Store construction port |
| `RecoveredPhysicalRuntimeHandoff` | Store recovery-construction port | one fresh quiescent physical runtime and exact C.8 output | later Part II physical readmission | branch, Query, semantic writer, or acknowledgment | C.9/C.10 and later integration |
| `RecoveryCleanupEligibility` | cleanup owner from publication, reopen, fallback, binding, and last-copy proofs | one exact residue action is safe | that one cleanup effect | recovery success or broader deletion | C.4 cleanup execution |
| `RecoveryReportEnvelope` | recovery observation exporter after Store outcome construction | versioned descriptive recovery facts under `store.physical.recovery-report` | compatible report interpretation | any Store transition or physical-format admission | parent harness/operator tooling |
| `RecoveryObserverReport` | independent offline process under `store.physical.recovery-observer-report` | versioned read-only artifact observations | compatible comparison and diagnostics | any recovery transition or physical-format admission | parent harness/operator tooling |

## Required Destination Directory And Module Plan

The implementation uses a parallel cutover. New C.8 orchestration is built in
the created `worth-store-recovery-runtime` crate while the old recovery-physics
facade remains untouched except for narrow shared contracts. Callers move only
after the new path passes its focused proof. The old path is then deleted; no
compatibility adapter survives closeout.

The existing `workspaces/worth-store/Cargo.toml` adds the new workspace member
and dependency entry in Phase 2; it does not expose recovery runtime through an
ordinary default feature.

Status markers:

- `[E]` existing and retained or narrowed;
- `[C]` created by C.8;
- `[M]` moved into the named destination;
- `[R]` replaced and removed after cutover;
- `[D]` deleted; and
- `[S]` committed successor destination, not an empty placeholder requirement.

```text
workspaces/worth-store/crates/
├── worth-store-recovery-runtime/                         [C]
│   ├── Cargo.toml                                        [C]
│   ├── README.md                                         [C]
│   └── src/
│       ├── lib.rs                                        [C] narrow public facade
│       ├── bin/
│       │   └── physical_store_recover.rs                 [C] production recovery entry
│       ├── entry/                                        [C] fresh-process boundary
│       │   ├── mod.rs                                    [C]
│       │   ├── request.rs                                [C]
│       │   ├── admission.rs                              [C]
│       │   ├── authority.rs                              [C]
│       │   ├── authority_binding.rs                      [C]
│       │   ├── session.rs                                [C]
│       │   └── outcome.rs                                [C]
│       ├── progression/                                  [C] compiler-visible phases
│       │   ├── mod.rs                                    [C]
│       │   ├── admitted.rs                               [C]
│       │   ├── discovered.rs                             [C]
│       │   ├── selected.rs                               [C]
│       │   ├── planned.rs                                [C]
│       │   ├── staged.rs                                 [C]
│       │   ├── published.rs                              [C]
│       │   ├── reopened.rs                               [C]
│       │   └── performed_effect.rs                       [C] private substrate wrappers
│       ├── orchestration/                                [C] cross-owner sequencing
│       │   ├── mod.rs                                    [C]
│       │   ├── discovery.rs                              [C]
│       │   ├── planning.rs                               [C]
│       │   ├── staging.rs                                [C]
│       │   ├── publication.rs                            [C]
│       │   └── reopen.rs                                 [C]
│       ├── handoff/                                      [C] quiescent successor
│       │   ├── mod.rs                                    [C] facade re-exports Store handoff
│       │   ├── operation_fates.rs                        [C]
│       │   ├── blocked.rs                                [C] persisted-source blocked terminal
│       │   └── cleanup_posture.rs                        [C]
│       ├── cleanup/                                      [C] post-publication only
│       │   ├── mod.rs                                    [C]
│       │   ├── plan.rs                                   [C]
│       │   ├── eligibility.rs                            [C]
│       │   └── execution.rs                              [C]
│       └── observation/                                  [C] cheap typed facts
│           ├── mod.rs                                    [C]
│           ├── counters.rs                               [C]
│           ├── protocol.rs                               [C]
│           └── report.rs                                 [C]
├── worth-store-recovery-physics/                         [E] narrowed to pure law
│   └── src/
│       ├── lib.rs                                        [R] narrow facade
│       ├── source_precedence/                            [E] replace/narrow internals
│       │   ├── mod.rs                                    [E]
│       │   ├── candidate.rs                              [M]
│       │   ├── admission.rs                              [M]
│       │   ├── current_previous_root.rs                  [C]
│       │   ├── checkpoint_base.rs                        [M]
│       │   ├── wal_tail.rs                               [M]
│       │   ├── compaction_product.rs                     [M]
│       │   ├── residue.rs                                [M]
│       │   └── selection.rs                              [R]
│       ├── wal_prefix/                                   [C] WAL prefix authority axis
│       │   ├── mod.rs                                    [C]
│       │   ├── continuity.rs                             [M]
│       │   ├── valid_prefix.rs                           [M]
│       │   ├── torn_tail.rs                              [M]
│       │   └── denial.rs                                 [M]
│       ├── redo_replay/                                  [E] pure planning law
│       │   ├── mod.rs                                    [E]
│       │   ├── record.rs                                 [M]
│       │   ├── plan.rs                                   [M]
│       │   ├── cursor.rs                                 [M]
│       │   └── denial.rs                                 [M]
│       ├── page_redo/                                    [E] pure page decisions
│       │   ├── mod.rs                                    [E]
│       │   ├── page_lsn.rs                               [E]
│       │   ├── eligibility.rs                            [E]
│       │   ├── transition.rs                             [R]
│       │   └── denial.rs                                 [E]
│       ├── operation_reconciliation/                     [C]
│       │   ├── mod.rs                                    [C]
│       │   ├── identity.rs                               [C]
│       │   ├── evidence_join.rs                          [C]
│       │   ├── binding_freshness.rs                      [C]
│       │   ├── fate.rs                                   [C]
│       │   └── denial.rs                                 [C]
│       ├── recovery_budget/                              [E] narrowed
│       │   ├── mod.rs                                    [E]
│       │   ├── limits.rs                                 [M]
│       │   ├── plan_cost.rs                              [M]
│       │   ├── counters.rs                               [M]
│       │   └── denial.rs                                 [E]
│       └── legacy executable/evidence/verifier facade    [D]
├── worth-store/
│   └── src/
│       ├── physical_runtime/recovery_freshness/          [C]
│       │   ├── mod.rs                                    [C]
│       │   ├── port.rs                                   [C] reconstruction-only sampling facade
│       │   ├── authority.rs                              [C] concrete Store sampling authority
│       │   ├── binding.rs                                [C] selected-checkpoint source, sealed basis, policy
│       │   └── cleanup.rs                                [C] current-root source, sealed basis, policy
│       ├── physical_runtime/recovery_construction/       [C]
│       │   ├── mod.rs                                    [C]
│       │   ├── port.rs                                   [C]
│       │   ├── authority.rs                              [C]
│       │   ├── runtime_identity.rs                       [C]
│       │   └── handoff.rs                                [C]
│       ├── bin/physical_store_c8_writer.rs               [C]
├── worth-store-offline-verifier/
│   └── src/
│       ├── c8_recovery_observation/                      [C]
│       │   ├── mod.rs                                    [C]
│       │   ├── artifact_walk.rs                          [C]
│       │   ├── physical_format.rs                        [C]
│       │   ├── conclusion.rs                             [C]
│       │   ├── report_protocol.rs                        [C]
│       │   └── report.rs                                 [C]
│       └── truth_composition/candidate_evaluation/        [C]
│           ├── mod.rs                                    [C]
│           └── candidate_set.rs                          [C]
└── worth-store-physical-certification/
    └── src/c8_fresh_process_recovery/                    [C]
        ├── mod.rs                                        [C]
        ├── scenario.rs                                   [C]
        ├── writer_process.rs                             [C]
        ├── recovery_process.rs                           [C]
        ├── observer_process.rs                           [C]
        ├── crash_matrix.rs                               [C]
        ├── oracle.rs                                     [C]
        ├── schedules/                                    [C]
        │   ├── mod.rs                                    [C]
        │   └── perturbation.rs                           [C]
```

The schedule harness remains only where it controls a real production
yieldpoint and reports a replayable seed.

### Boundary ownership and exclusions

| Boundary | Dominant axis and owner | Belongs | Excluded | Enforcement |
| --- | --- | --- | --- | --- |
| `recovery-runtime/entry` | fresh-process admission | public request, concrete authority, exact dynamic bindings, linear session, exact outcome | source logic, media effects, reports | facade, private Worth Proof substrate, and per-axis drift gates |
| `recovery-runtime/progression` | proof lifecycle | sealed consuming phase values and concrete performed-effect wrappers | orchestration, parsing, bare generic proof carriers | private fields, compile-fail tests, and action/binding identity checks |
| `recovery-runtime/orchestration` | cross-domain sequencing | calls into physics, Store port, and C.4 work route | domain meaning | dependency and composition checks |
| `recovery-runtime/observation` | descriptive recovery export | cheap typed facts, report protocol identity/version, bounded rendering | Store admission, effects, rich mandatory diagnostics | separate observation profile, compatibility-window tests, and import gates |
| `recovery-physics/source_precedence` | persisted truth selection | role-specific admission and deterministic choice | filesystem execution, operator policy | no `worth-store`, Signal, or runtime dependency |
| `recovery-physics/wal_prefix` | WAL continuity | exact prefix/torn/middle decisions | redo execution | focused property and adversarial tests |
| `recovery-physics/redo_replay` | immutable redo plan | admitted record grammar and order | media writes | pure API and dependency check |
| `recovery-physics/page_redo` | page transition meaning | generation/pageLSN apply or skip | buffer-pool lifecycle | pure transition tests |
| `operation_reconciliation` | physical fate | identity/evidence join and owner-sampled generation freshness | semantic retry, acknowledgment delivery, or caller-supplied freshness | exhaustive fate types and source/policy substitution tests |
| `recovery_freshness` | Store physical freshness ownership | owner-sampled selected-checkpoint and published-root generations, sealed bases, and concrete policies | pure classification, caller samples, or replay | exact authority-trace-to-topology owner equality and substitution tests |
| `recovery_construction` | Store physical runtime construction | new identity, fresh handles, quiescent handoff | replay and semantic serving | reconstruction-only feature/dependency gate |
| `c8_recovery_observation` | independent read-only evidence | stable format interpretation and observer report protocol | recovery decisions, authority types, and Store format admission | import/dependency, protocol-window, and controlled-defect gates |
| `truth_composition/candidate_evaluation` | offline candidate projection | owner-verified frontier observations, canonical candidate ordering, confidence, and conflict denial | source precedence, Store admission, media effects, and execution selection | exact candidate facade, bounded allocation, and independent identity/digest assertions |

The destination forbids `recovery_manager.rs`, `helpers.rs`, `common.rs`,
`util.rs`, generic evidence bags, flat phase-state files, and any module that
combines candidate discovery, selection, redo execution, publication, cleanup,
and reporting.

Committed successor growth is additive:

- C.9 adds integrity classification beside C.8 admission and observer facts; it
  does not move source precedence;
- C.10 consumes the quiescent handoff and adds stable-read/reclaim admission
  beside recovery construction; it does not widen C.8 cleanup; and
- later backup/PITR/replica workflows consume selected public facts through
  their own owners rather than entering the C.8 facade.

## Performance And Resource Contract

C.8 recovery may scale only with:

```text
fixed current/previous selector slots
+ selected checkpoint manifest and checkpoint-addressed recovery metadata
+ retained WAL tail after the selected checkpoint frontier
+ exact pages/extents targeted by admitted redo
+ bounded unresolved and unexpired operation bindings
+ declared partial-publication or damaged scope
+ one staging generation and publication granule
+ proved cleanup candidates
```

It may not scale with total Store size, total historical WAL, all historical
operations, all expired idempotency keys, directory-tree breadth, semantic
record count, diagnostic richness, or offline-observer work.

`PhysicalRecoveryLimits` separately bounds:

- selector and checkpoint candidates;
- manifest bytes and entries;
- retained WAL segments, frames, and bytes;
- redo targets, bytes, and distinct pages/extents;
- unresolved and unexpired operation bindings;
- staging bytes and dirty frames;
- concurrent Recovery-scoped commands;
- publication effects;
- cleanup candidates and cleanup bytes; and
- observation/report materialization.

Admission rejects before expensive allocation or effects when a limit can
already be known. A limit reached after bounded discovery returns the exact
dimension, observed amount, admitted limit, progress posture, and whether any
effect escaped. There is no hidden unlimited recovery mode.
Manifest routing decoders receive the remaining leaf-entry and branch-child
budgets before collecting either vector. WAL segment inspection receives the
remaining cumulative frame budget and rejects the crossing frame before it is
retained. Eventual post-decode or post-allocation comparison is not acceptable
limit evidence.

The ordinary recovery counter snapshot contains exact:

- selector cells read;
- candidates discovered and admitted by role;
- manifest bytes and entries read;
- checkpoints admitted and rejected;
- canonical WAL segments and recognizable frame envelopes scanned, valid
  segments admitted, and bytes scanned;
- valid-prefix frames and bytes;
- torn suffix frames/bytes rejected;
- middle-corruption and missing-range denials;
- redo frames planned, applied, and skipped by reason;
- pages/extents read and written;
- staging bytes allocated and peak recovery bytes held;
- operation fates by exact variant;
- entry and admitted-world binding comparisons and denials by exact axis;
- owner-sampled binding-freshness evaluations by exact verdict;
- recovery sessions issued and terminated by exact top-level outcome, plus
  owner-detected non-terminal drops;
- concrete performed physical effects by action kind and exact outcome;
- root and namespace publication effects;
- independent-reopen reads;
- residue classifications;
- cleanup actions planned, completed, and deferred; and
- scheduler submissions, deferrals, cancellations, and settlements for each
  C.8 work family.

Report-protocol identity/version reads, accepted versions, typed incompatibility
postures, encoded bytes, and payload fields materialized belong to the bounded
observation profile. They are not charged to or silently performed by the
ordinary recovery lane.

Counters are monotonic, stage-honest, and cannot report future work as zeroed
fields. Each phase exposes only facts already executed. Exact structural
counters prove boundedness; elapsed time is secondary qualification evidence.

The ordinary CI process lane runs the complete deterministic process matrix
with the small bounded profile. Larger named profiles are invoked directly on
their target hardware when scale or timing qualification is needed; those
runs are performance observations, not a second source-validity system. Every
timing report names source revision, hardware, filesystem, backend profile,
cold/warm posture, Store and tail scale, concurrency, repetitions, and
percentiles.

## Cleanup, Cutover, And Deletion Contract

### Runtime artifact cleanup

Cleanup begins only after all of these proofs exist:

1. the staging generation is closed and quiescent;
2. its recovered root replacement completed;
3. the required namespace barrier completed;
4. the published generation was reopened through fresh handles;
5. reopened identities, manifests, page frontiers, fate set, and counters agree
   with the closed plan;
6. the retained fallback and last-copy rules are satisfied; and
7. no unresolved binding or unsupported/quarantined scope needs the artifact.

Immediately before each cleanup effect, the cleanup owner samples the current
published-root generation through its concrete Store-owned freshness source and
re-evaluates the sealed cleanup-plan basis under the concrete cleanup policy.
The caller cannot present an observation, choose an evaluation moment, or
substitute a source or policy. A stale, shifted, unavailable, or failed
evaluation returns the exact retained/deferred posture and no cleanup
`Performed` evidence.

Cleanup eligibility is per artifact and consuming. It can authorize only the
exact removal it names. A cleanup batch cannot be constructed from filenames,
age, generation comparison, a checkpoint-present boolean, or a generic
`recovered=true` receipt.

The unpublished `worth-store-physical-backend` package is part of the trusted
C.4 implementation boundary, not a separately supported product facade or an
untrusted downstream caller. Store is the sole workspace package permitted to
enable its owner feature and invoke its cleanup mechanism. The backend adapter
accepts only the already-admitted physical request and cannot mint Store
eligibility, freshness, policy, performed evidence, or recovery progression.
The public authority guarantee therefore applies at the delivered Store and
recovery-runtime boundaries; a package deliberately added as a direct
dependency of the unpublished backend joins the C.4 trusted computing base and
is governed by the workspace manifest and callsite gates rather than Rust
cross-crate visibility.

Never delete or recycle:

- the current recovered root or any referenced artifact;
- the retained previous root before its C.10-era reclaim law permits it;
- the selected checkpoint or WAL required to reconstruct the current root;
- WAL required by an unresolved or unexpired operation binding;
- the only admissible copy of an authority artifact;
- an artifact involved in `PublicationIndeterminate`;
- unsupported, damaged, quarantined, or unexplained material whose retention
  is required for later classification; or
- temporary observer output still owned by an active process.

Cleanup failure after successful recovered-root publication yields
`RecoveryCleanupPosture::Deferred` with exact candidates and causes. It does
not roll back or invalidate the current recovered root. A later maintenance
owner may retry the exact still-valid plan only after revalidating current root,
generation through the owner-sampled freshness boundary, unresolved bindings,
and last-copy facts. Each successful removal records concrete performed-effect
evidence only after C.4 reports completion; attempted or ambiguous removal
retains the artifact disposition and cannot be relabeled successful.

### Source cutover

Cutover is judged from the current Git diff, Cargo dependency graph, public
facades, and direct consumers. Replaced surfaces are deleted in the same change
that migrates their callers. No cutover inventory, disposition table, source
registry, or generated topology report is maintained.

The existing recovery-physics crate receives special scrutiny:

- preserve and narrow the real source-precedence, valid-prefix, pageLSN, redo,
  and recovery-budget law;
- move executable orchestration to `worth-store-recovery-runtime`;
- move independent observation to `worth-store-offline-verifier`;
- delete duplicated offline-verifier, runtime-driver, generic materialization,
  and source-shape proof machinery that has no continuing owner;
- remove the direct `worth-store` dependency after the narrow construction port
  is live; and
- do not pull backup, PITR, rollback, replica, repair, or corruption-readmission
  policy into C.8. If an existing surface has no current roadmap-backed owner,
  delete it; otherwise migrate it to that owner without a C.8 compatibility
  re-export.

Each phase removes dead tests, fixtures, exports, modules, dependencies,
documentation, and certification cases exposed by its cutover. Cleanup is not
deferred wholesale to closeout.

## Verification Policy

C.8 verification is intentionally small and direct:

- Git identifies the reviewed source revision.
- Cargo targets and features define what is built.
- Focused unit, integration, compile-fail, and fresh-process tests decide pass
  or fail.
- The workspace boundary checker enforces dependency and authority direction.
- Independent reviewers judge whether those tests adequately cover the changed
  behavior.

Completed phase evidence is immutable Git history, not a live status system.
A later defect is fixed and protected by the narrowest useful regression test;
it does not reopen or rewrite a historical phase. Current changes are blocked
only by current compilation, tests, boundary checks, or material review
findings.

C.8 must not add requirement ledgers, source-closure maps, source or executable
fingerprints, proof reports, mutation receipts, report-to-report reconciliation,
or tests whose subject is another test. Structural checks are retained only
when they enforce a real compiler-visible boundary that the compiler and
workspace boundary checker cannot already express.

The Phase 8 process lane builds the writer, observer, and recovery executables
through separate locked Cargo invocations that reuse Cargo's configured target
directory. Separate invocations preserve role-feature isolation while normal
incremental artifacts keep the direct suite practical during iteration.
The process scenarios receive only those three paths and produce no
certification bundle.

## Documentation Deliverables

### Fresh-process recovery guide

Create `_docs/worth-store/physical-recovery-and-reopen.md` for Store callers and
operators. It must explain:

- when the ordinary Store open path yields to C.8 recovery;
- the exact fresh-process inputs and authority requirements;
- the distinction between sealed authority lane, exact dynamic recovery
  binding, owner-sampled freshness, and performed physical effect;
- outcome handling for recovered, refused, blocked, publication-indeterminate,
  and cleanup-deferred cases;
- physical operation-fate meaning and retry restrictions;
- boundedness and exact counter interpretation;
- unsupported/damaged scope and the C.9 boundary;
- safe restart after interrupted C.8 publication; and
- the recovery and observer report protocol family identities, initial version
  windows, typed unsupported-version handling, and their non-authority; and
- what C.8 explicitly does not restore.

Every Rust example compiles against the public recovery facade. Operator
examples are executed in a bounded documentation lane against a real temporary
store and distinct processes.

### Owner documentation

Revise or create:

- `worth-store-recovery-runtime/README.md` for the composition boundary,
  authority binding axes, linear session lifecycle, progression, performed
  effects, observation protocol, and exclusions;
- `worth-store-recovery-physics/README.md` for its narrowed pure-law role and
  dependency direction;
- `worth-store/` owner documentation for the reconstruction-only construction
  port and quiescent handoff;
- `worth-store-offline-verifier/README.md` for C.8 observer independence and
  the exact observer-report protocol contract;
- `physical-durability-and-checkpoints.md` for the corrected C.7/C.8 persisted
  boundary; and
- the physical reconstruction roadmap for C.8 and C.9/C.10 handoffs.

Mark `storage-foundation-s4.md` as historical and superseded for current C.7/C.8
authority. Remove or correct claims that the old wide recovery-physics facade,
same-crate verifier, sharp-checkpoint assumptions, or pre-C.7 WAL mechanics are
the current architecture.

Documentation drift checks bind API names, variants, report family identities,
produced versions, consumer windows, examples, and outcome semantics to the
real public facade. They do not assert prose through fragile production-source
substrings.

## Phase Plan

Phases close in order. A later phase cannot begin implementation until the
earlier phase's code, focused tests, cleanup, documentation slice, independent
review, commit, and push are complete. A discovered defect blocks the current
change until its root cause and regression protection are complete; historical
phase records are not rewritten.

### Phase 1: Freeze Recovery Truth, API, And Cutover Boundaries

**Becomes true:** C.8 has one persisted-input contract, one public entry path,
one authority path, and explicit destination owners visible in code.

**Consumes:** C.7 closeout guarantees, actual persisted formats, current
recovery-physics exports and consumers, current certification/observer routes,
and governing laws.

**Establishes:** the new crate boundary, exact fresh-process inputs, concrete
authority provenance, exact entry and admitted-world binding axes, linear
session terminals, performed action kinds, owner-sampled freshness sources and
policies, report protocol identities/version windows, phase-state names,
outcome/fate vocabularies, dependency direction, and deletion targets. The C.7
in-memory handoff is explicitly non-authoritative for C.8.

**Evidence:** focused facade compilation, dependency and boundary checks,
hostile protocol cases, and executable examples.

**Cleanup:** delete false placeholders, duplicate planned APIs, obsolete
milestone-coded vocabulary, and tests or documents that assert the rejected
entry contract.

### Phase 2: Admit A Genuinely Fresh Recovery Session

**Becomes true:** a new process can enter recovery only with exclusive root
ownership, qualified backend profile, concrete platform authority, new session
identity, and finite limits.

**Consumes:** Phase 1 contracts and C.3/C.4/C.6 lifecycle, backend, and Recovery
allocation authority.

**Establishes:** `worth-store-recovery-runtime`, the public request/outcome
facade, privately sealed authority markers, concrete entry and admitted-world
bindings, the concrete linear recovery session plus owner lifecycle tracking,
`AdmittedPhysicalRecovery`, fresh scheduler/Signal mechanism, and bounded
read-only discovery ports.

**Evidence:** compile-fail authority attacks, two-process identity journeys,
one-axis-at-a-time binding drift twins, wrong-root/profile/media-generation/
configuration/limit/session denials, duplicate-terminal compiler attacks,
owner-visible non-terminal-drop tests, allocation-bound tests, and zero-effect
counters for refusal.

**Cleanup:** remove any alternate recovery constructor, test-only entry, or
runtime-driver API replaced by the new facade.

### Phase 3: Select One Bounded Persisted Source Basis

**Becomes true:** bounded discovery and deterministic precedence produce one
`SelectedPhysicalRecovery` without media mutation.

**Consumes:** admitted session, current/previous selector protocol, checkpoints,
retained WAL inventory, manifests, manifest-addressed page and extent placement
facts, compaction cutovers, and residue. Phase 4 admits the addressed page or
extent bytes and their pageLSNs; Phase 3 does not pre-decode replay targets.

**Establishes:** role-specific candidate admission, exact precedence table,
selected root/checkpoint/tail/binding basis, rejected-source trace, and discovery
counters. Entry or cancellation denials remain `Refused`; a missing, conflicting,
unsupported, damaged, corrupt, or over-limit persisted source consumes the
session into top-level `Blocked(PhysicalRecoveryBlock)` after quiescence. That
terminal preserves exact Store, session, source/artifact, generation/LSN,
counter, limit, and zero-effect evidence relevant to the cause.

**Evidence:** exhaustive precedence model/property tests, hostile current versus
previous and foreign-Store cases, absent-versus-rejected checkpoint twins,
terminal partial-first-frame and nonterminal-corruption WAL twins, residue and
compaction attacks, deterministic repeated selection, cumulative WAL and
multi-block manifest exact-limit twins, exact terminal counters, and direct
negative precedence cases.

**Cleanup:** replace and delete old candidate-confidence, generic selection,
checkpoint-selection, and duplicated source-role surfaces that do not match the
locked precedence contract.

### Phase 4: Plan Valid-Prefix Redo And Exact Operation Fates

**Becomes true:** C.8 has one immutable, all-admitted, effect-free recovery plan.

**Consumes:** selected basis, page generation/pageLSN facts, WAL record grammar,
checkpoint binding compaction, retained attempt bindings, the sealed C.7 lease
bases, owner-sampled selected-checkpoint generation, concrete freshness policy,
the checksum-bound free-space allocation header including its persisted
segment-page capacity, and exact limits.

**Establishes:** maximal valid WAL prefix, torn-tail versus middle-corruption
classification, ordered apply/skip decisions, fate set, staging layout,
publication plan, exact owner-sampled binding-freshness evaluations, expected
counters, and no-effect/indeterminate boundaries. Absent inline targets obey
the ordinary producer's exact allocation law: reusable tail pages are consumed
first, each non-final new segment contains exactly the persisted page capacity,
and only the final new segment may be partial. The immutable redo plan
independently binds each pending projection allocation capacity and used-page
count to that selected allocation truth before staging authority exists.

**Evidence:** independent prefix decoder tests, property tests for gaps/overlaps
and repeated planning, every-fate identity blender, exact counter oracles,
source/policy/sample substitution compiler attacks, checkpoint-generation
boundary twins, compile-time plan progression, and direct adversarial cases.

**Cleanup:** delete old replay bases, receipts, staged-WAL adapters, and generic
evidence materialization made redundant by the plan and fate types.

### Phase 5: Execute Redo Into A Closed Staging Generation

**Becomes true:** every planned redo effect settles into a non-current staging
generation and the complete recovery world reaches quiescence.

**Consumes:** `PlannedPhysicalRecovery`, exact Recovery allocation, matching
Store aspect-native work bases, admitted scheduler policy, and C.4 media ports.

**Establishes:** fresh bounded frames, exact page/extent transitions, final
pageLSNs and digests, staged manifests, settled fate set, stage-honest counters,
concrete performed staging-effect evidence, and
`ClosedRecoveryStagingGeneration`.

**Evidence:** real-media apply/skip journeys, repeated-recovery convergence,
partial-effect cancellation and failure cases, wrong-generation/pageLSN cases,
admitted-as-performed and wrong-action compiler attacks, evidence-before-effect
cases, exact allocation/counter reconciliation, and quiescence
compile/runtime proof.

**Cleanup:** remove execution from recovery-physics, duplicate page-transition
routes, temporary staging adapters, and fixtures that bypass C.4/C.5.1.

### Phase 6: Publish, Reopen, And Produce The Quiescent Handoff

**Becomes true:** the recovered root is namespace durable, independently
reopened, and exposed only as a quiescent physical handoff with new identity.

**Consumes:** closed staging generation, exact publication plan, concrete
performed C.4 root-replacement and namespace-barrier effects, concrete
performed fresh-handle reopen evidence, concrete Store construction authority,
and final fate set.

**Establishes:** `NamespaceDurablePhysicalRecovery`,
`ReopenedPhysicalRecovery`, the narrow Store construction port, fresh runtime
identity/handles, `RecoveredPhysicalRuntimeHandoff`, and publication-
indeterminate continuation law.

**Evidence:** real crash seams around every publication effect, distinct-process
and distinct-runtime identity proof, fresh-handle reopen comparison, compile-
fail serving/semantic/performed-action attacks, evidence-before-barrier and
wrong-generation binding cases, and root/barrier counter cases.

**Cleanup:** delete the old executable recovery facade, broad Store imports, and
any handoff constructor or compatibility re-export bypassing the new port.

### Phase 7: Prove Post-Publication Artifact Cleanup

**Becomes true:** every C.8-created or discovered artifact has a current,
retained, deferred, quarantined/unsupported, or safely removed disposition.

**Consumes:** published/reopened recovery, fallback and last-copy facts,
operation fates, unresolved bindings, sealed cleanup-plan basis, owner-sampled
current-root generation, concrete cleanup freshness policy, residue
classifications, and exact cleanup limits.

**Establishes:** per-artifact cleanup eligibility, scheduled cleanup effects,
concrete performed removal evidence, deferred cleanup posture, and crash-safe
owner-sampled retry revalidation.

**Evidence:** hostile last-copy and unresolved-binding attacks, cleanup crash
matrix, deferred-cleanup recovery journey, freshness source/policy substitution
attacks, stale-plan rejection, ambiguous-removal evidence attacks, and exact
cleanup counters.

**Cleanup:** delete temporary cleanup adapters and redundant residue
classifications replaced by exact eligibility.

### Phase 8: Cut Over The Public Surface And Delete The Old Path

**Becomes true:** every C.8 caller uses the destination facade, recovery physics
has its final narrow role, documentation names the real public contract, and no
old executable recovery or compatibility path remains.

**Consumes:** proved destination entry through cleanup, the current dependency
graph, affected direct consumers, public facades, and named documentation
deliverables.

**Establishes:** final narrow facades, complete caller migration, exact deletion
closure, versioned recovery and observer report protocols, owner READMEs, and
the executable caller/operator guide.

**Evidence:** facade compile tests, dependency and feature checks, report
identity/version/window tests, wrong-family and future-version twins, compiled
documentation, separate locked builds for the three process roles,
warnings-denied focused tests, and the fresh-process recovery suite.

**Cleanup:** delete obsolete modules, exports, fixtures, reports, source-shape
gates, README claims, dependencies, and temporary parallel-cutover code.

### Phase 9: Hostile Process Verification And Successor Closure

**Becomes true:** the final source survives the complete distinct-process crash
matrix, deterministic schedules, direct hostile inputs, independent
observation, and boundedness proof.

**Consumes:** final cutover source, public/operator documentation, observer,
process harness, exact counters, and named profiles.

**Establishes:** `RecoveredPhysicalRuntimeHandoff` as the sole physical
successor boundary. C.9 may consume it for integrity, corruption localization,
quarantine, and offline truth. C.10 may consume it for stable reads, epochs,
reclaim, scheduled I/O, and maintenance interference. Neither successor may
reconstruct a parallel C.8 authority lane from reports or test artifacts.

**Evidence:** every persisted-effect C.7 mutation seam and every C.8 recovery
seam under real process death; explicit deterministic scenario seeds; direct
malformed, torn, stale, foreign, and unsupported inputs; independent observer
comparison; authority compile denials; documentation execution; exact bounded
counters; and line-cap, constitution, dependency, and focused API checks.
Named larger profiles remain direct target-hardware qualification, separate
from source closure. The scheduled and manually dispatched release job runs
the complete `process-scenario` matrix directly after the release-scale backup
proof; it does not substitute a hand-maintained test filter or generated
certificate for those executable scenarios.

Successor-candidate observation is production accounting. Planning exposes the
number of addressed candidate artifacts read, candidate bytes read, and peak
retained candidate materialization bytes. Root-routing, segment-membership,
and free-space observation share one cumulative manifest-entry budget. Direct
raw-media evidence must admit the exact total and deny one entry below it; a
second per-family or candidate-local allowance is forbidden. A failed child
decode reports the exact retained prefix peak for its family, including all
earlier successfully retained families. A recovery with no pending
publication must report zero for all three counters even when malformed bytes
exist at the next-generation path. Candidate boundedness uses two isolated
copies of the same killed-writer world: one with the candidate topology and one
without it. The exact observation and lifecycle-memory values must equal an
independent raw-media and public-structure calculation, followed by exact-limit
admission and one-byte-below denial. Lifecycle memory is the larger of
candidate observation plus one expected-artifact comparison scratch buffer and
the final publication materialization; recovery must not retain a second
complete expected candidate. A test may not use the production plan cost as
its own expected candidate delta.

Candidate identity is stronger than semantic equivalence. Recovery reproduces
the concrete C.7 producer's incremental allocation and packing order from the
selected topology, including capacity transitions, and compares each emitted
canonical artifact directly with the addressed immutable candidate bytes.
Same-capacity and capacity-transition derivation remain independent recovery
implementations; calling the candidate-absent full-rebuild helper is not an
incremental adoption proof. Canonically encoded stale selected-generation
topology, foreign topology, frontier drift, malformed frames, and corrupted
children produce exact typed denials before effects. A killed-writer capacity
transition must preserve the writer's exact multi-level successor bytes through
recovery and a second fresh reopen.

The complete mutation-seam matrix explicitly uses an extent-backed writeback
workload. Successor child-family corruption explicitly uses an inline-record
workload so root-routing, segment-membership, and free-space candidate children
are present. Crash stage and workload are independent required inputs; no stage
implicitly selects its workload.

**Cleanup:** delete temporary process-only code, cross-test lock bookkeeping,
recursive tests of parent-oracle helpers, and writer-issued mutation identity,
fate, or barrier receipts. Reaching a production C.4 yieldpoint is observed
from that real gate; a second marker cannot certify it. Expected operations are
bound from the parent-submitted program and independently decoded persisted
authority. Retain production C.4 yieldpoints/interposers, the independent
raw-byte observer, direct malformed persisted inputs, and the smallest process
launcher needed to kill and inspect real executables. Git records the reviewed
change; no ledger, certificate, fingerprint map, or generated closeout report
is produced.

## Must Preserve

- C.4 remains the only physical effect executor.
- C.5.1 remains the one bounded scheduler/executor topology; recovery creates a
  fresh instance rather than another scheduler design.
- C.6 Recovery allocation remains bounded and scope-specific.
- C.7 remains the authority for ordinary durability, checkpoint/root
  publication, acknowledgment construction, and idempotency retention law.
- Stable Store identity survives; runtime and process identities do not.
- Recovery and replay remain reconstructive and absent from ordinary features.
- WAL, pageLSN, root, checkpoint, and fate meaning remain physical and
  branch-agnostic.
- Worth Proof remains private legality/proof substrate beneath concrete
  Store-owned authority, identity, effect, freshness, and lifecycle types.
- Foundational remains one-way description; its report protocols, envelopes,
  identities, versions, and compatibility outcomes never become Store recovery
  or physical-format authority.
- Signal coordinates new recovery work but is not persisted truth.
- C.9 remains the owner of comprehensive corruption localization and repair
  posture.
- C.10 remains the owner of stable reads, epochs, reclaim, and maintenance
  interference.

## Explicit Non-Goals

C.8 does not:

- recover semantic transactions, branch heads, snapshots, subscriptions,
  Query state, or semantic writers;
- prove client/network receipt of an acknowledgment;
- perform PITR, rollback, backup restore, replica bootstrap, disaster recovery,
  or operator-selected source override;
- repair damaged authority, rebuild arbitrary derived families, or decide
  quarantine release;
- introduce undo recovery;
- use lexical Proof branding as persistent process, runtime, Store, or handoff
  identity, or use generic causal action-kind evidence as exact occurrence
  lineage;
- optimize checkpoint cadence, compaction, WAL layout, or profitability;
- introduce a second scheduler, media executor, or artifact namespace;
- preserve unreleased API compatibility; or
- certify all future physical artifact families.

## Acceptance Evidence

C.8 closes only with all of the following green on final source:

1. warnings-denied focused and all-target compilation for affected Worth Store
   crates;
2. workspace line-cap and composition checks for every dirty Rust file;
3. boundary and agent-context enforcement;
4. direct public-facade compilation and caller reachability;
5. exact dependency direction, including no ordinary replay import and no
   `worth-store` dependency from narrowed recovery physics;
6. no stale callers, duplicate entry paths, or compatibility residue;
7. compile-fail authority, phase-order, serving, semantic, bare-witness,
   bare-binding, bare-performed, duplicate-terminal, and counterfeit evidence
   attacks;
8. exact entry and admitted-world binding-axis declarations, identical-world
   positive twins, and one-axis-at-a-time drift denials;
9. owner-sampled freshness evidence rejecting caller samples and substitute
   source/policy types at compile time and generation boundaries at runtime;
10. concrete performed-effect progression proving admission, scheduling,
    attempts, counters, and wrong action kinds cannot advance a phase;
11. linear-session evidence proving one terminal transition and owner-visible
    non-terminal drop;
12. pure source-precedence, WAL-prefix, pageLSN, redo, fate, and budget property
   tests;
13. real-media focused recovery integration tests with exact counters;
14. real process death at every named C.7 and C.8 crash seam;
15. independent observer/runtime comparison with explicit disagreement cases;
16. deterministic repeated recovery over identical admitted bytes;
17. Store-size independence at fixed checkpoint/tail/damage scope;
18. exact operation-fate identity blender and absence-as-no-effect attack;
19. safe-cleanup last-copy, unresolved-binding, and interrupted-publication
    attacks;
20. recovery-report and observer-report family/version/window tests covering
    wrong family, supported version, future version, and every declared retired
    posture without granting Store authority;
21. direct controlled-defect tests for each material recovery branch;
22. explicit replayable scenario seeds for the crash matrix;
23. executable public/operator documentation; and
24. the focused current test and boundary lanes green at the reviewed revision.

Evidence is rejected if it can pass through comments, string literals, dead
syntax, copied production decision code, same-process state, a test-only
constructor, a bare generic proof carrier, a caller-supplied freshness sample,
an unvalidated report payload, broad `is_err()` assertions, elapsed-time luck,
or an oracle that shares the defect under test.

## C.9 And C.10 Successor Handoff

C.8 exposes one sealed `RecoveredPhysicalRuntimeHandoff` containing:

- stable Store identity and new runtime identity;
- selected and namespace-durable recovered root identity and generation;
- retained previous-root posture;
- selected checkpoint identity and exact covered LSN frontier;
- admitted valid WAL-prefix frontier and torn/rejected suffix posture;
- verified page/extent generation and pageLSN frontier required for bounded
  physical access;
- exact `RecoveryOperationFateSet` for every retained unresolved or unexpired
  operation identity;
- bounded fresh physical access capabilities;
- unsupported, damaged, or quarantined physical scope without repair policy;
- cleanup completed/deferred posture and retained-artifact reasons;
- exact backend profile identity relevant to interpreting the recovered facts;
  and
- final stage-honest recovery counters and source identity.

The handoff does not contain:

- source-selection, redo, publication, or cleanup authority;
- recovery platform authority or a reusable recovery session;
- writer runtime identity, handles, frames, leases, queues, Signal graph, or
  scheduler state;
- semantic mutation, branch, MVCC, Query, transaction, or writer authority;
- acknowledgment construction or a claim of client delivery;
- repair, quarantine-release, reclaim, PITR, backup, restore, or replica
  promotion authority; or
- an assertion that every indeterminate operation or unsupported scope has been
  resolved.

C.9 may consume the physical identities, selected sources, unsupported/damaged
scope, and observer disagreement posture to localize integrity failures. It
cannot reinterpret the recovered root or mint repair authority from the
handoff.

C.10 may consume the current and retained root generations, bounded access
capabilities, quiescence, and cleanup posture to establish stable reads,
maintenance coordination, and safe reclaim. It cannot reuse C.8 cleanup proof
as C.10 reclaim authority.

## Closeout Gate

C.8 closes only when a killed production writer can be replaced by a distinct
process that selects, reconstructs, publishes, and independently reopens one
deterministic physical truth from persisted authority alone; resolves every
retained physical operation to the weakest exact fate; remains bounded by the
checkpoint, WAL tail, touched/damaged scope, and admitted resources rather than
Store size; deletes nothing without post-publication last-copy proof; exposes
only a fresh quiescent physical handoff; and leaves no old executable recovery
or compatibility authority beside the destination. Its concrete authority is
sealed and bound to every exact recovery axis; generation freshness is sampled
only by Store owners; only performed C.4 effects advance effectful phases; the
recovery session terminates exactly once under owner lifecycle observation; and
versioned recovery/observer reports remain compatible descriptive evidence
that cannot open a Store door.
