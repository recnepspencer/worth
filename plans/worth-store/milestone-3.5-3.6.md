# Milestone 3.5 And 3.6 Engineering Spec: Durable Media Semantics, Write-Path Certification, And Adversarial Crash Recovery

> **Status:** Closed
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-1.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-1.md)
> - [milestone-1-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-1-closeout.md)
> - [milestone-2.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-2.md)
> - [milestone-2-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-2-closeout.md)
> - [milestone-3.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3.md)
> - [milestone-3-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3-closeout.md)
>
> **Impacted later milestone:** [milestone-4.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-4.md)
>
> **Primary architectural driver:** make acknowledged durable truth depend on
> declared media barriers and typed recovery source precedence instead of
> platform folklore, optimistic driver behavior, or backend-local guesswork

## Goal

Make `worth-store` durable in the strongest sense that matters:

- acknowledgment means the bytes that justify retained truth crossed declared
  media barriers
- crash recovery chooses the right recovery source by explicit precedence
- interrupted publication and interrupted maintenance fail or recover by
  declared rules rather than by luck
- repeated restart converges to one quiescent truth conclusion instead of
  reopening already-settled work

## Why These Milestones Exist

Milestone 3 made the durable commit boundary real.

That was necessary, but it is not the endgame foundation.

A database that wants to be truly hard to corrupt cannot stop at:

- "the WAL exists"
- "restart usually works"
- "SQLite or the filesystem probably flushed things"
- "rename is probably atomic enough"
- "we can reconstruct intent from whatever files are around"

Milestone 3.5 and Milestone 3.6 exist because the next failure layer is below
ordinary business logic:

- torn writes
- truncated tails
- reordered persistence
- partially durable publication families
- interrupted maintenance
- recovery trusting the wrong source because it is convenient
- recovery repeatedly reprocessing closed work because no exact terminal state
  exists

If these two milestones are weak, every later milestone is standing on
optimism:

- snapshots may be published on media semantics that are softer than their
  restore contract claims
- delta layering may inherit hidden partial-publication assumptions
- compaction and reclaim may be impossible to recover honestly once they are
  interrupted
- replication capsules may reuse write-path folklore and then certify the wrong
  bytes

Milestone 3.5 and 3.6 therefore harden the layer that most databases talk
around instead of specifying.

## Hard Part

The hard part is not "be more careful about fsync."

The hard part is holding five different things apart even though naive designs
constantly collapse them:

- semantic authoritative truth
- media durability evidence
- publication eligibility
- recovery source precedence
- operator-visible degraded-state truth

The design fails if:

- record framing is too vague to distinguish a clean tail from a torn write
- backend-local "success" is treated as a durability barrier without a declared
  crash contract
- acknowledgment is allowed before all durability classes needed for retained
  truth have crossed their declared barriers
- recovery treats the richest-looking source as authoritative instead of the
  highest-precedence admitted source
- interrupted maintenance artifacts are trusted because they are newer than the
  last known-good artifacts
- salvage, quarantine, rebuild, and restart are allowed to blur together into
  "try something and see if the store opens"

## Explicit Assumptions

- Milestone 1 authoritative artifact families remain the only semantic durable
  truth authority.
- Milestone 2 operating-mode boundaries remain unchanged; this spec hardens
  durable mode and recovery, not embedded-mode artifact intake.
- Milestone 3 already established WAL-coordinated durable publication and a
  rebuild control lane, but did not yet freeze the full media semantics or
  broader recovery-source hierarchy required for endgame-grade durability.
- `worth-relational` still owns mutation semantics, replay meaning, branch
  semantics, lineage semantics, and canonical commit-envelope production.
- later milestones may add snapshots, compaction, reclaim, replication,
  capsules, and extension-defined derived artifact families, but this milestone
  must already define how those future families rank during recovery even
  before they ship fully.
- authenticity and integrity are separate: bytes may be well-formed and
  uncorrupted yet still not be trusted if they fail declared authenticity or
  source-admission rules.
- backup/restore and disaster recovery posture begin here as durable-artifact
  compatibility and admitted recovery-source contracts, even though higher
  operator workflows land later.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the failure that will destroy
  trust later, not the one that is easiest to demo now. This spec therefore
  starts from torn writes, partial publication, and source-precedence lies
  rather than from "better restart speed."
- `arch_laws.md`
  The most important thing it protects here is proof-bearing state progression
  and explicit authority boundaries. Law 41 is central: raw bytes, framed WAL
  records, barrier-crossed media records, publication-admitted artifacts,
  recovery-admitted source sets, quarantined families, and recovered trusted
  truth must all be distinct types with sealed transitions. Law 33 and Law 36
  matter too: authority stays authoritative, and recovery acceleration must be
  reconstructible from declared bases rather than ambient residue.
- `perf_laws.md`
  The most important thing it protects is truthful cost and truthful broadness.
  Media scans, tail validation, recovery source evaluation, interrupted-work
  salvage, and quiescent restart checks must expose named complexity bases and
  exact counters instead of hiding broad scans or repeated rediscovery behind a
  cheap-looking API.
- `domain_laws.md`
  The most important thing it protects is decomposition by changing reason.
  Record framing, barrier semantics, publication families, source precedence,
  restart planning, salvage, quarantine, audit evidence, and operator-visible
  degraded-state reporting must be separate subdomains rather than one "durable
  storage helper."
- `worth_store_vision.md`
  The most important thing it protects is that store owns survival while the
  runtime owns semantics. This spec therefore makes media durability and
  recovery exact without allowing backend layout or recovery shortcuts to
  redefine truth.
- `worth_store_roadmap.md`
  The most important thing it protects is order. These milestones belong before
  snapshots, delta layering, and maintenance programs because later physical
  work is dishonest if the media and recovery substrate beneath it is still
  folklore.
- `test-requirements.md`
  The most important thing it protects is certifiable hostility. This spec must
  therefore define named write-path and recovery suites that prove exact crash
  barriers, recovery-source precedence, interrupted-maintenance handling,
  quiescent restart, and typed damaged-media failure.
- `milestone-3.md`
  The most important thing it protects is the first honest durable commit
  boundary. This spec must harden that boundary without replacing it; Milestone
  3 remains the semantic durable-commit program, while this spec deepens the
  media and broader recovery layer underneath and around it.
- `milestone-3-closeout.md`
  The most important thing it protects is the already-shipped publication and
  restart shape. This spec must preserve those proofs while making them more
  exact about media durability classes, restart quiescence, and source
  precedence.
- `milestone-4.md`
  The most important thing it protects is snapshot non-authority and
  snapshot-plus-tail restore honesty. This spec therefore has to define the
  write-path and recovery rules snapshots will later inherit, especially for
  partial publication and source precedence.
- `milestone-4-closeout.md`
  The most important thing it protects is that snapshot capture and restore are
  already structurally honest at the semantic level. This spec may force
  conformance hardening underneath those surfaces, but it must not reopen the
  snapshot authority model itself.
- `worth_relational_vision.md`
  The most important thing it protects is that semantic replay and MVCC meaning
  belong to the runtime. This spec therefore hardens the bytes and recovery
  program around canonical artifacts rather than inventing a storage-native
  semantic story.
- `worth_relational_roadmap.md`
  The most important thing it protects is deterministic truth publication and
  replay. This spec must preserve deterministic replay by making media and
  recovery choose between admitted already-declared sources, not by
  rediscovering semantics from partial residue.
- `worth_runtime_bridge_vision.md`
  The most important thing it protects is stable basis and explainable
  downstream truth. Recovery, quarantine, and degraded states emitted by store
  must still be describable in canonical artifact terms for later bridge and
  signal consumers.
- `worth_runtime_bridge_roadmap.md`
  The most important thing it protects is typed downstream consumption of
  committed truth. This spec therefore requires operator-visible and
  machine-checkable recovery classification instead of ambiguous "store opened"
  behavior.

## Combined Adversarial Constraint

Milestone 3.5 and Milestone 3.6 together must survive this hostile condition:

> A crash, power loss, kernel flush lie, torn WAL tail, partially durable
> snapshot family, interrupted compaction rewrite, interrupted reclaim,
> interrupted replication capsule publication, mixed backend family fleet, and
> operator restart under uncertainty must all converge to one typed conclusion
> about what truth is still trusted, what work remains unpublished, what must
> be rebuilt, what must be quarantined, and what may be acknowledged, without
> allowing backend-local artifacts, partial writes, or newer-looking derived
> residue to outrank canonical authoritative truth.

## Product Decision Lock

- media durability is a first-class store contract, not an ambient property of
  the host OS, filesystem, embedded database, or cloud volume
- every admitted backend family must declare exact barrier classes for:
  - append durability
  - metadata durability
  - rename or publication durability
  - directory-entry durability when relevant
  - transactional-commit durability when relevant
- acknowledgment is never keyed to "write returned success"; it is keyed to the
  declared barrier classes required for retained truth in that backend family
- record framing must make truncated tails, torn records, partial checksums,
  and unsupported versions mechanically distinguishable
- recovery must consume sources in declared precedence order, not by recency,
  convenience, richness, or performance
- incomplete or interrupted maintenance artifacts are never implicitly promoted
  because they are newer than the last good authoritative or derived family
- salvage and quarantine are admitted first-class outcomes; the store is not
  allowed to bluff a clean restart when the right answer is degraded
- authenticity and integrity must remain separate in both write-path and
  recovery logic
- restart must become quiescent for already-terminal work; reprocessing closed
  work forever is out of spec

Normative consequence:

- any implementation that acknowledges based on optimistic buffer residency,
  unspecified driver behavior, or undeclared transactional semantics is out of
  spec
- any implementation that cannot tell a clean tail from a torn or truncated
  tail for admitted durable families is out of spec
- any implementation that lets recovery choose among source families without a
  declared precedence table is out of spec
- any implementation that silently downgrades into salvage, quarantine, or
  rebuild while presenting the state as clean durable success is out of spec

## Scope

### In Scope

- durable media record framing and media-family versioning for admitted
  write-ahead and publication-critical durable families
- exact backend-family durability barrier declarations
- typed torn-write, truncation, partial-write, and publication-gap families
- startup scan and tail validation rules
- restart-time recovery source precedence
- broader crash-class taxonomy beyond Milestone 3's core durable-commit classes
- interrupted publication and interrupted-maintenance recovery rules for:
  - WAL families
  - snapshot families
  - compaction products
  - reclaim markers
  - replication or capsule publication families
- explicit recovery-mode matrix including:
  - ordinary crash restart
  - authoritative rebuild
  - integrity-audit rebuild
  - salvage
  - quarantine
  - snapshot-plus-tail recovery when later admitted
  - replication/bootstrap recovery when later admitted
- operator-visible trusted-truth and degraded-state reporting
- write-path and crash-recovery certification bundles
- backup/restore compatibility posture at the artifact-family and recovery
  source level

### Explicitly Out Of Scope

- new semantic authoritative artifact families
- consensus replication or quorum commit
- multi-region leadership or lease protocols
- encryption-at-rest key management beyond the requirement that authenticity
  and integrity stay distinct
- higher-level operator UX beyond typed surfaces and machine-checkable bundles
- tenant policy design beyond ensuring recovery and repair stay tenant-visible
  when relevant
- advanced scheduling policy for maintenance workers beyond the recovery rules
  needed when maintenance is interrupted

## Part I: Durable Media Semantics And Write-Path Certification

### Why Media Semantics Need Their Own Program

Milestone 3 assumed a backend durability barrier strongly enough to make the
crash boundary meaningful.

Milestone 3.5 now has to make that assumption mechanical.

Without this part, the store still has dangerous ambiguity about:

- what exact bytes count as durable
- whether directory entries matter for recovery truth
- whether rename is a publication barrier or just a metadata hint
- how torn writes are detected
- how tail truncation is localized
- whether transactional commit in an embedded database maps cleanly to the
  barrier the store thinks it is using

This is where naive databases become "probably durable."

### Backend Durability Barrier Rule

Every admitted backend family must declare one exact `DurabilityBarrierClass`
for each write operation that matters to publication or recovery.

Minimum barrier classes:

- `ProcessBufferOnly`
- `KernelBufferResident`
- `FileContentDurable`
- `FileAndRequiredMetadataDurable`
- `RenameOrPublicationMarkerDurable`
- `DirectoryEntryDurable`
- `TransactionalCommitDurable`

Rules:

- a backend family may refine these classes, but may not skip declaring them
- the store may acknowledge only after all barrier classes required by the
  family-specific publication contract have been crossed
- if a backend cannot honestly expose which class a write has crossed, it is
  not an admitted backend for durable mode
- `KernelBufferResident` is never sufficient for durable acknowledgment
- `TransactionalCommitDurable` is not automatically equivalent to
  `DirectoryEntryDurable`; the backend adapter must declare what durable truths
  that transaction actually covers

### Record Framing Rule

Every admitted append-only durable family that may be scanned after crash must
use explicit framing.

Minimum framing fields:

- family identifier
- family version
- record length
- durable mutation or artifact family identity
- payload digest
- header digest or equivalent framing-integrity check
- terminal completeness marker or equivalent suffix-validity proof

Required guarantees:

- startup scan can distinguish:
  - complete valid record
  - valid prefix with truncated tail
  - complete-length record with payload corruption
  - header corruption
  - unsupported family version
  - family mismatch
- framing must be mechanically parseable without needing ambient file length
  folklore
- framing must define how unsupported future fields are skipped or rejected

### Record Framing Ambiguity Checklist

The implementation must explicitly resolve all of these:

- byte-order basis for length and digest fields
- whether lengths cover header only, payload only, or full record
- whether padding exists and whether padding bytes are covered by the digest
- whether checksums cover pre-normalized or post-normalized payload bytes
- alignment and sector-boundary behavior where relevant
- whether multi-record batching changes individual record framing or only the
  carrier format
- how duplicate terminal markers or duplicate trailing bytes are treated
- how partially zero-filled tails are classified
- how future-added fields avoid retroactively changing old digest meaning

Milestone 3.5 code is not ready until each ambiguity class is explicitly
resolved.

### Write-Path Publication Family Rule

Every publication-critical artifact family must declare:

- whether it is append-only, replace-in-place, rename-published, or
  transaction-published
- the exact barrier classes required before it may be considered published
- whether directory durability is required for visibility after crash
- the exact partial-publication states it admits
- the exact recovery classification for each partial-publication state

Minimum publication-critical families in this milestone:

- WAL intent and follow-on WAL families
- authoritative publication families from Milestone 1
- acknowledgment eligibility markers if stored separately
- any snapshot, compaction, reclaim, or capsule publication markers once those
  later milestones are present in the implementation

### Write-Path Atomicity Rule

The store must declare one `DurablePublicationUnit` per admitted publication
family group.

For durable-mode commit publication, the unit still spans:

- WAL intent family
- required follow-on WAL families
- canonical authoritative append unit
- branch-head publication
- acknowledgment eligibility state

Milestone 3.5 extends this by requiring:

- exact media-family barrier mapping for every member of the unit
- exact partial-durability classifications for any member missing its required
  barrier
- explicit rollback-free semantics: the store may reject, quarantine, retain,
  or rebuild, but may not pretend partial-durable state never existed if it
  was already physically emitted

### Acknowledgment Barrier Contract

Acknowledgment eligibility must be defined as a function of declared barrier
classes, not helper success.

Required mechanical statement:

`AckEligible = all publication-critical members of the admitted DurablePublicationUnit have crossed the backend-family barrier classes declared for retained truth`

Implicit requirements made explicit:

- no later async flush may be required to make an already acknowledged commit
  durable
- no background thread may complete an omitted barrier after acknowledgment and
  still count as a correct path
- no caller-visible acknowledgment may race ahead of directory durability if
  directory durability is part of the family contract
- if one backend family treats a barrier as stronger than another, the public
  meaning of acknowledgment must still remain identical

### Barrier Ordering Rule

Barrier classes are not enough on their own. The store must also define the
required ordering in which those barriers are crossed.

Required rule:

- for every publication-critical family, the spec must define:
  - bytes written
  - barrier crossed for those bytes
  - metadata or rename/publication marker write
  - metadata or rename/publication barrier
  - directory-entry barrier where relevant
  - acknowledgment eligibility point

Minimum ordering constraints:

- append-only families may not acknowledge before the record bytes and all
  framing bytes needed for restart classification cross their declared barrier
- rename-published families may not treat rename visibility as durable
  publication unless the family's required metadata and directory barriers have
  also crossed where relevant
- transaction-published families may not treat transactional success as a
  complete publication proof unless the backend contract explicitly states which
  file-content, metadata, and visibility effects that transaction durably
  covers
- multi-family publication units must define whether their barriers are:
  - fully ordered
  - partially ordered with an explicit completion marker
  - transactionally collapsed by one stronger backend primitive

Normative consequence:

- "all the right writes happened eventually" is not sufficient
- a backend implementation is out of spec if it cannot explain the ordering by
  which a post-crash observer concludes the same acknowledgment boundary the
  pre-crash writer claimed

### Startup Tail Validation Rule

On restart, every admitted append-only family must pass through one of:

- `TailValidatedClean`
- `TailValidatedTruncated`
- `TailValidatedCorrupt`
- `TailValidationVersionUnsupported`

Rules:

- `TailValidatedTruncated` may allow suffix discard only if the framing and
  family contract explicitly admit truncation as a non-authoritative tail loss
- `TailValidatedCorrupt` must fail typed unless a later explicit salvage rule
  admits constrained recovery
- startup tail validation must not silently rewrite or trim bytes before
  producing a typed classification
- any automated tail repair must itself emit typed repair evidence

### Media Authenticity Rule

Integrity and authenticity must stay distinct in the write path.

Required distinction:

- `IntegrityValid` means the bytes are well-formed and match their declared
  framing-integrity contract
- `AuthenticityValid` means the bytes also came from an admitted source and
  have not crossed an authenticity-trust boundary illegally

Rules:

- a byte sequence may be integrity-valid but authenticity-invalid
- admitted backends must expose whether authenticity is guaranteed by local
  trust boundary alone or requires explicit higher-level validation
- recovery may not trust integrity-valid but authenticity-invalid records as
  authoritative or publication-completing evidence

## Part II: Adversarial Crash Recovery And Recovery Source Precedence

### Why Recovery Needs Its Own Hardening Program

Milestone 3 already made restart and rebuild first-class.

Milestone 3.6 hardens the broader system answer to:

- which source outranks which when they disagree
- what happens when derived families are partially published
- what happens when maintenance work is interrupted
- when the system must quarantine instead of guessing
- when restart is allowed to become a no-op because everything is already
  terminal

Without this part, every future milestone would be able to smuggle in new
recovery folklore.

### Recovery Source Precedence Rule

Every recovery decision must name which admitted source family justified it.

Minimum precedence order:

1. canonical authoritative artifacts
2. barrier-valid WAL families admitted by Milestone 3 and 3.5
3. typed recovery-decision artifacts already emitted by a prior admitted
   recovery pass
4. published and integrity-valid derived families whose milestone explicitly
   admits them as acceleration substrates:
   - snapshots
   - compaction products
   - replication capsules
   - later derived artifact families
5. quarantined or salvage-only artifacts for diagnostic localization only

Rules:

- lower-precedence sources may accelerate or localize work
- lower-precedence sources may not overrule a higher-precedence admitted source
- if two sources of the same precedence disagree, recovery must fail typed or
  enter an explicit salvage/quarantine mode; it may not pick one silently
- recency does not outrank precedence
- lower operational cost does not outrank precedence

### Recovery-Decision Artifact Non-Authority Rule

Recovery-decision artifacts from a prior admitted recovery pass are not a new
truth source.

They are allowed to record:

- that a prior recovery pass already resolved a specific observed state
- that a durable mutation or maintenance family already reached a terminal
  outcome
- that restart may become quiescent without repeating the same work

They are not allowed to establish:

- retained authoritative truth not already justified by higher-precedence
  authoritative or WAL sources
- branch-head meaning
- snapshot, compaction, reclaim, or capsule publication truth unsupported by
  the original higher-precedence basis

Required rule:

- every recovery-decision artifact must carry explicit lineage to the
  higher-precedence source set that justified it
- if that basis source set is missing, damaged, or contradicted, the
  recovery-decision artifact loses standalone authority and may only be used
  for localization or operator diagnostics until reverified

This is the anti-"yesterday's recovery note became today's authority" rule.

### Recovery Source Precedence Checklist

The implementation must explicitly answer all of these:

- what happens if authoritative artifacts are intact but WAL says publication
  was incomplete
- what happens if WAL is intact but authoritative append is missing
- what happens if a snapshot is newer than the last known-good authoritative
  publication
- what happens if a compaction product is complete but its source retention
  markers are not
- what happens if a capsule import family is intact but authenticity-invalid
- what happens if a previous recovery decision artifact exists but its basis
  source family is now damaged
- what happens if a lower-precedence source would allow cheaper recovery than a
  higher-precedence source

No implementation is ready until each case has an explicit answer.

### Crash-Class Taxonomy

Milestone 3's durable-commit crash classes remain in force.

Milestone 3.6 extends the taxonomy to include at minimum:

- `CrashDuringWalAppend`
- `CrashAfterWalBarrierBeforeAuthoritativeBarrier`
- `CrashAfterAuthoritativeBarrierBeforeAck`
- `CrashAfterAck`
- `CrashDuringSnapshotPublication`
- `CrashDuringCompactionRewrite`
- `CrashDuringReclaimPublication`
- `CrashDuringCapsulePublication`
- `CrashDuringRecoveryDecisionEmission`
- `CrashDuringSalvageOrQuarantineTransition`
- `CrashDuringDirectoryMetadataPersistence`
- `CrashWithTruncatedTail`
- `CrashWithTornRecord`

Every admitted class must map to one exact outcome family.

### Recovery Outcome Matrix

Every crash or damaged-media state must end in one of:

- `RetainAndResume`
- `RetainAndSuppressDuplicateReplay`
- `DiscardIncompleteUnpublishedWork`
- `FinishPublicationFromHigherPrecedenceSource`
- `RebuildFromHigherPrecedenceSource`
- `QuarantineAffectedFamily`
- `EnterSalvageMode`
- `RequireOperatorDecision`
- `FailTyped`

Rules:

- these are semantic recovery classes, not UI labels
- one observed state may admit only one of these outcomes absent an explicit
  higher-level operator policy surface
- if the implementation cannot reduce an observed state to one exact outcome
  class, it must fail typed rather than improvisedly recovering

### Interrupted-Maintenance Recovery Rule

The store must define exact recovery behavior for interrupted maintenance.

Minimum maintenance families:

- snapshot publication
- compaction rewrite publication
- reclaim marker publication
- replication or capsule publication
- future extension-defined maintenance families once admitted

Each maintenance family must declare:

- publication unit
- last known-good source basis
- incomplete-publication classification
- whether incomplete output is:
  - ignored
  - rebuilt
  - quarantined
  - salvageable
- whether old input remains authoritative until new output is fully published

The default rule should be conservative:

- old known-good inputs remain trusted until replacement publication is fully
  barrier-complete
- incomplete replacement output is not trusted merely because it exists

### Restart Quiescence Rule

Repeated restart must become quiescent for already-terminal work.

Required meaning:

- once a durable mutation or maintenance publication has reached a terminal
  recovery class, later restart scans may rediscover its existence but must not
  emit new semantic recovery work for it
- quiescence must be keyed to explicit terminal recovery evidence, not merely
  absence of pending scans

Forbidden drift:

- every restart appending fresh recovery records for already-settled work
- every restart re-running salvage analysis for already-quarantined artifacts
- every restart reopening acknowledged retained truth as though it were still
  pending

### Salvage And Quarantine Rule

Salvage and quarantine are first-class recovery outcomes, not shame states.

Required distinction:

- `Quarantine` isolates a family or scope because it is not trusted for normal
  operation but is still valuable for diagnostics, operator decision, or later
  repair
- `Salvage` performs constrained, declared recovery work over damaged inputs
  without claiming a clean ordinary restart

Rules:

- quarantine must preserve enough artifact identity to localize what was
  isolated
- salvage must declare its allowed source set and its non-authority boundary
- quarantine and salvage outcomes must be operator-visible and
  machine-checkable
- neither outcome may silently degrade into ordinary clean restart

### Quarantine Scope And Blast-Radius Rule

Quarantine must declare the narrowest safe affected scope.

Minimum admitted scope classes:

- `ArtifactInstanceScope`
- `ArtifactFamilyScope`
- `BranchScope`
- `TenantScope`
- `StoreWideScope`

Rules:

- the implementation must quarantine the narrowest scope that still preserves
  truth and trust boundaries
- if recovery escalates to a wider scope than the immediately damaged artifact
  because lineage, publication coupling, or authenticity uncertainty require
  it, that escalation must be explicit and machine-checkable
- tenant-visible systems must report when quarantine crossed a tenant boundary
  or was contained within one
- quarantine scope may not be inferred from convenience or current code layout;
  it must be derived from declared publication and source-coupling rules

### Backup, Restore, And Disaster-Recovery Compatibility Rule

Milestone 3.6 must make explicit what recovery sources are admitted for backup,
restore, and disaster recovery.

Required declarations:

- which authoritative families are sufficient to reconstitute trusted truth
- which derived families may accelerate restore but may not be required for
  truth
- what version compatibility window is admitted for backup import
- when a backup restore must downgrade into rebuild, quarantine, or typed
  incompatibility failure

Rules:

- backup presence does not outrank live higher-precedence authoritative truth
  during ordinary restart
- disaster-recovery restore must still publish the same canonical truth model
  as ordinary authoritative rebuild
- media-valid but compatibility-invalid backup artifacts must fail typed rather
  than partially restore

### Ordinary Restart Versus Restore-Mode Separation Rule

Local crash restart and imported backup or disaster-recovery restore are
distinct recovery modes.

Required rule:

- ordinary local crash restart may consult only the locally admitted live
  authoritative, WAL, and declared local derived families for that store
  instance
- imported backup or disaster-recovery artifacts may participate only when the
  store is explicitly opened in a restore, bootstrap, or disaster-recovery mode
- imported artifacts may not silently appear in the precedence set for an
  ordinary local restart just because they are present on disk

Normative consequence:

- backup media is not a side-channel authority source for local crash restart
- a clean local restart and an explicit restore-mode import remain distinct,
  operator-visible operations with distinct certification surfaces

## Proof-Carrying Media And Recovery Pipeline

Law 41 is load-bearing across this combined milestone.

Minimum intended phase chain:

- `RawDurableBytes`
- `FramedDurableRecord`
- `IntegrityValidatedDurableRecord`
- `BarrierClassifiedDurableRecord`
- `PublicationAdmittedFamily`
- `ObservedCrashOrDamageState`
- `RecoverySourceSet`
- `PrecedenceResolvedRecoveryDecision`
- `QuarantinedOrSalvagedFamily`
- `RecoveredTrustedTruth`

Rules:

- each later phase consumes explicit proof from the prior phase
- raw bytes may not jump directly to trusted recovery evidence
- constructors for proof-bearing media and recovery types must be crate-sealed
- fields on proof-bearing types must be private
- accessor methods on proof-bearing types must expose only read-only views that
  preserve the established invariant
- tests may only bypass phases through dedicated fixture-only modules
- recovery planning may not accept weaker types than
  `ObservedCrashOrDamageState` plus `RecoverySourceSet`
- operator-visible trusted-truth output must consume
  `PrecedenceResolvedRecoveryDecision`, not raw backend inspection

### Law 41 Enforcement Rules

This milestone must follow Law 41 mechanically, not narratively.

Required enforcement:

- every proof-bearing phase type in this milestone must be constructible only by
  the proving function or proving subsystem that establishes its invariant
- no `pub` fields, deserialization path, debug hook, feature flag, or
  convenience constructor may mint a proof-bearing type directly
- any serialization or persistence decode path must terminate in a single
  verification gateway before it can produce the corresponding proof-bearing
  runtime type
- if an operator action requires a proven precondition such as explicit
  quarantine scope, restore mode, or salvage admission, that precondition must
  be represented as a witness or typestate, not as an ambient boolean or
  comment-level contract
- if a later phase accepts an earlier weaker phase plus an extra runtime check,
  the proof chain is incomplete and the spec is not satisfied

Forbidden drift:

- `BarrierClassifiedDurableRecord` created outside the barrier classifier
- `RecoverySourceSet` assembled ad hoc by callers instead of the source-set
  constructor
- `PrecedenceResolvedRecoveryDecision` reconstructed from raw rows in multiple
  places
- quarantine or salvage entry admitted by mode flags rather than a typed witness

Normative consequence:

- implementations must prefer extra wrapper types and witness types over
  convenience surfaces when the choice is between compile-time invalid-state
  prevention and runtime checks

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| record framing distinguishes clean tail, truncated tail, and torn record | framing validation | `media/framing/` | `DurableRecordFramingInvalid`, `DurableTailTruncated`, or `DurableTornWriteDetected` | `write_path_digest` and `failure_digest` |
| acknowledgment occurs only after declared backend barrier classes are crossed | barrier proof | `media/barriers/` and `publication/` | `DurableBarrierContractViolation` | `ack_boundary_report` |
| integrity-valid but authenticity-invalid records are not trusted for publication or recovery | authenticity validation | `media/authenticity/` and `recovery/` | `DurableRecordAuthenticityInvalid` | `recovery_source_report` |
| lower-precedence sources never outrank higher-precedence authoritative truth | precedence resolution | `recovery/precedence/` | `RecoverySourcePrecedenceViolation` | `recovery_source_report` and `truth_digest` |
| interrupted maintenance output never displaces the last known-good input before full publication | maintenance recovery | `recovery/maintenance/` | `InterruptedMaintenancePublicationGap` | `maintenance_recovery_report` |
| restart becomes quiescent after terminal recovery classification | restart planning | `recovery/restart/` | `RecoveryQuiescenceViolation` | `quiescence_report` and `counter_snapshot` |
| quarantine and salvage remain explicit degraded outcomes | degraded-state classification | `recovery/degraded/` | `RecoveryQuarantineViolation` or `RecoverySalvageViolation` | `degraded_state_report` |
| backup or disaster-recovery artifacts respect compatibility windows and source precedence | backup/restore validation | `recovery/backup/` | `BackupRestoreCompatibilityViolation` | `compatibility_digest` and `restore_digest` |

## Failure Taxonomy

Milestone 3.5 and 3.6 must ship explicit typed failures at minimum covering:

- `DurableRecordFramingInvalid`
- `DurableTailTruncated`
- `DurableTornWriteDetected`
- `DurableBarrierContractViolation`
- `DurableDirectoryDurabilityGap`
- `DurablePublicationMarkerGap`
- `DurableRecordAuthenticityInvalid`
- `DurableFamilyVersionUnsupported`
- `RecoverySourcePrecedenceViolation`
- `RecoverySourceConflict`
- `InterruptedMaintenancePublicationGap`
- `RecoveryQuiescenceViolation`
- `RecoveryQuarantineRequired`
- `RecoverySalvageRequired`
- `BackupRestoreCompatibilityViolation`
- `DisasterRecoverySourceInsufficient`
- `RecoveryTrustedTruthAmbiguous`
- `RecoveryOperatorDecisionRequired`

Rules:

- public failures must be store-owned semantic failures, not backend-driver
  jargon
- typed failures must localize the affected family, scope, branch, tenant, or
  durable mutation identity where possible
- degraded outcomes are not generic success; they must be represented either as
  typed failures or typed degraded-result classes

## Required Internal Subsystems

- `media/framing/`
  record framing, checksums, length validation, version handling
- `media/barriers/`
  backend-family durability barrier declarations and proof surfaces
- `media/authenticity/`
  admitted-source and authenticity validation
- `recovery/precedence/`
  source-set construction and precedence resolution
- `recovery/restart/`
  restart scanning, terminal-state detection, quiescence enforcement
- `recovery/maintenance/`
  interrupted snapshot, compaction, reclaim, and capsule recovery
- `recovery/degraded/`
  salvage, quarantine, and trusted-truth classification
- `recovery/backup/`
  backup, restore, and disaster-recovery compatibility boundaries
- `diagnostics/`
  counter surfaces, operator-visible degraded-state reports, certification
  bundles
- `harness/`
  write-path corruption, torn-write, interrupted-publication, and precedence
  certification fixtures

## Complexity Contracts

Minimum write-path contracts:

- startup frame scan cost is proportional to:
  - records scanned in the admitted families
  - tail bytes examined
  - framing validations performed
- durability-barrier proof cost is proportional to:
  - publication-family members in the durable publication unit
  - backend-family barrier checks required
- truncation and torn-write localization cost is proportional to:
  - the damaged suffix width, not the entire store

Minimum recovery contracts:

- recovery source-set construction cost is proportional to:
  - candidate source families admitted for the observed crash class
  - damaged or incomplete families requiring classification
- precedence resolution cost is proportional to:
  - conflicting same-scope sources for one recovery decision
- restart quiescence verification cost is proportional to:
  - restart-relevant work that lacks terminal evidence
  - not total historical durable work
- interrupted-maintenance recovery cost is proportional to:
  - affected publication families and scopes
  - not all historical maintenance artifacts

Minimum counters:

- `durable_frame_scan_count`
- `durable_frame_reject_count`
- `durable_truncated_tail_count`
- `durable_torn_write_count`
- `durable_barrier_verified_count`
- `durable_ack_barrier_violation_count`
- `recovery_source_precedence_resolution_count`
- `recovery_source_precedence_fallback_count`
- `recovery_quiescent_restart_count`
- `recovery_non_quiescent_restart_count`
- `recovery_quarantine_count`
- `recovery_salvage_count`
- `interrupted_maintenance_recovery_count`
- `backup_restore_compatibility_reject_count`

## Phases

### Phase 1: Lock Media Barrier Vocabulary And Recovery Outcome Vocabulary

Required work:

- define durability barrier classes
- define record framing families and version vocabulary
- define crash-class and damaged-media taxonomy
- define recovery outcome classes
- define the combined proof-bearing media and recovery pipeline

Exit condition:

- "durable" and "recoverable" are no longer ambient words
- every later subsystem has one locked vocabulary to build on

### Phase 2: Implement Record Framing, Tail Validation, And Barrier Proofs

Required work:

- implement framing for admitted append-only families
- implement startup tail validation
- implement backend-family barrier declarations and proof surfaces
- expose typed framing, truncation, torn-write, and barrier failures
- emit exact framing and barrier counters

Exit condition:

- the store can classify clean, truncated, torn, and unsupported records
- acknowledgment barriers are mechanically provable per backend family

### Phase 3: Harden Publication Units Against Partial Media Truth

Required work:

- map all publication-critical families to explicit publication units
- declare partial-publication states and exact recovery classification
- forbid optimistic helper success from becoming acknowledgment proof
- make authenticity distinct from integrity in publication-critical paths

Exit condition:

- retained truth is tied to declared barriers and admitted family states
- partial publication is classified, not hand-waved

### Phase 4: Implement Recovery Source-Set Construction And Precedence Resolution

Required work:

- construct explicit recovery source sets for each crash and damage class
- implement the precedence resolver
- expose typed same-precedence conflict handling
- emit source-precedence diagnostics and counters

Exit condition:

- recovery can say why it trusted one source and not another
- source choice is no longer implementation folklore

### Phase 5: Implement Interrupted-Maintenance Recovery

Required work:

- implement interrupted snapshot publication recovery
- implement interrupted compaction publication recovery
- implement interrupted reclaim publication recovery
- implement interrupted capsule or replication publication recovery
- preserve old known-good inputs until new outputs are fully barrier-complete

Exit condition:

- partial maintenance outputs no longer create ambiguous restart state

### Phase 6: Implement Quiescent Restart, Salvage, And Quarantine

Required work:

- implement terminal recovery evidence
- enforce restart quiescence for closed work
- implement salvage and quarantine surfaces
- emit trusted-truth and degraded-state reports

Exit condition:

- repeated restart converges
- degraded recovery remains explicit and operator-visible

### Phase 7: Implement Backup, Restore, And Disaster-Recovery Source Contracts

Required work:

- define admitted backup and DR source families
- define compatibility windows and typed reject surfaces
- implement backup/restore source-precedence interaction
- emit compatibility and disaster-recovery reports

Exit condition:

- backup and DR are subordinate to the same authoritative and precedence model
- version incompatibility cannot bluff a partial clean restore

### Phase 8: Certify Write-Path Exactness And Adversarial Recovery

Required work:

- run the Milestone 3.5 named suite:
  `Durable Media And Write-Path Certification Test`
- run the Milestone 3.6 named suite:
  `Adversarial Crash Recovery And Recovery Source Precedence Test`
- compare backend-family write-path lanes
- compare restart, rebuild, salvage, and quarantine lanes where admitted
- emit machine-checkable write-path, source-precedence, degraded-state, and
  counter bundles

Exit condition:

- durable truth depends on declared media barriers, not folklore
- recovery-source choice is deterministic, explainable, and typed
- interrupted maintenance and repeated restart are certifiably honest

## Must Ship

- explicit record framing and startup tail validation for admitted durable
  families
- declared backend-family durability barrier contracts
- typed torn-write, truncation, partial-publication, and authenticity failures
- explicit recovery source precedence
- interrupted-maintenance recovery contracts
- quiescent restart semantics
- salvage and quarantine as typed degraded outcomes
- backup, restore, and DR source-admission and compatibility contracts
- machine-checkable Milestone 3.5 and 3.6 certification bundles

## Must Preserve

- semantic authority remains in canonical authoritative artifacts
- runtime semantics remain owned by `worth-relational`
- derived or maintenance families never outrank higher-precedence authority
- authenticity and integrity remain distinct
- later milestones may accelerate recovery, but may not renegotiate this source
  precedence model silently
- backend variation may change mechanics, never acknowledgment meaning or
  recovery truth conclusions

## Acceptance Evidence

This combined milestone is complete only when the store satisfies both named
suites:

- `Durable Media And Write-Path Certification Test`
- `Adversarial Crash Recovery And Recovery Source Precedence Test`

Required machine-checkable outputs across the combined program:

- `truth_digest`
- `artifact_digest`
- `write_path_digest`
- `recovery_source_report`
- `maintenance_recovery_report`
- `degraded_state_report`
- `compatibility_digest`
- `failure_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- torn writes and truncated tails are localized and typed
- acknowledgment never outruns declared barrier classes
- source precedence is deterministic and mechanically explainable
- interrupted maintenance never displaces last known-good inputs prematurely
- restart becomes quiescent for closed work
- salvage and quarantine remain explicit degraded outcomes
- backup and DR restores remain subordinate to compatibility and source
  precedence rules

Milestone 3.5 and 3.6 are not closed by "the store restarted" or "SQLite
committed" tests.

## Architectural Notes

- The smart abstraction is not "durability helper." The smart abstraction is
  one proof-bearing path from raw durable bytes to trusted recovered truth.
- Record framing and barrier semantics should remain backend-mechanical, while
  precedence resolution and degraded-state truth remain store-semantic.
- Do not let later milestones add ad hoc publication markers or recovery
  source families without extending the precedence table and failure taxonomy.
- If a future optimization cannot explain itself in these source-precedence and
  barrier terms, it probably belongs below an implementation detail boundary
  and should not change public recovery meaning.
- Milestone 4 and later may need conformance hardening after this lands, but
  they should inherit these rules rather than renegotiate them.

## Sequencing Notes

This combined spec belongs immediately after Milestone 3 and before any further
physical-storage or derived-recovery acceleration work.

- `Milestone 4` and `Milestone 5` should be treated as semantically correct but
  possibly needing conformance hardening underneath once this program lands.
- `Milestone 11`, `Milestone 12`, `Milestone 14`, and `Milestone 22`
  directly depend on this milestone's barrier, precedence, degraded-state, and
  compatibility vocabulary.
- future extension-defined artifact families are not allowed to bypass this
  milestone; they must declare how they participate in publication barriers,
  source precedence, and degraded recovery.
