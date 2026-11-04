# P-C installation and retained-authority inventory

Authority: coordinator rulings following aee56f91a1. This is a boundary inventory,
not a qualification or completion claim. The inventory baseline is aee56f91a1,
before implementation edits. D16 remains deferred. The seven BC9002
Native courts belong to P-A grant12 and are not edited here.

## Installation inventory

PreparedApplicationRuntimeContributions has exactly handlers, producers,
conditionals, resources and managed_indexes (application_contribution/composition.rs).
Home source/assembly.rs transfers handlers/resources/indexes into bootstrap and
program activation separately. Home world.rs seals handlers, compiles authorization,
validates indexes and transfers program/roster/activation/build identity. It drops
pending producers and conditionals. Resources reach bootstrap, but home_attachment.rs
only validates them: it never calls PendingResourceRegistry::install and never
transfers installed resources into conditional_cleanup.resources.

Compared with construction.rs and application_runtime/installation.rs, Home also
omits producer edition sealing/support/package validation; conditional declaration
binding installation and inventory validation; producer Signal routes; readiness
routes and validation; output-family lineage binding; resource installation;
conditional reconstruction/reconciliation; conditional cleanup ownership; accepted
output readmission and checkpoint-lineage restoration. It defaults conditional
operations, producer/readiness routes, installed producer/conditional registries,
recovered outputs and conditional cleanup owner. Managed indexes and program
support are filled afterwards, not dropped. New-run demand/dispatch/recovery counters,
empty transient custody registries and observed-source meanings are ordinary
per-runtime defaults on both paths, not pending contributions.

There is also a missing trusted-service input outside the five pending fields.
Ordinary construction accepts `authorization_time_source` (construction.rs:26)
and passes it through both publication routes (:167/:187). Home request and
HomeApplicationState have no corresponding field, and Home world attachment
hardcodes `WorthQueryRuntimeClock::system()`. This violates the Phase 3 trusted
time-service input contract. World clock resources are transferred separately by
home_preparation/binding.rs; that does not supply Query authorization time.
The first source reviewer identified this omission from the original inventory.
It is accepted as an installation design gap; its production transfer remains
part of the unfinished common installation cutover, not a completed fix.

The ordinary contribution owner must be factored and consumed by both installation
paths, retaining real Home Bridge/World/Native owners. Home already receives a
sealed Bridge; ordinary installation uses BridgeConditionalRuntimeBuilder before
seal. Existing owned conditional extension/reconstitution APIs must be evaluated;
creating a replacement Bridge or memory World is not an acceptable shortcut.

Accepted-output recovery takes DecodedApplicationCheckpoint.accepted_outputs and
funded metadata through WorthQueryRecoveredOutputs::from_checkpoint, then
checkpoint_lineage::restore. Home currently carries only Native CheckpointRestoreWork
and no Query decoded checkpoint. Store registry.rs requires query.obligations absent.
A persistence/readmission seam must be established before claiming that sharing
producer installation alone proves restart reuse or changed-build staleness.

Retirement must preserve the actual registry until its runtime owner is consumed.
Registry Clone shares Arc<InstalledProducerProvider> entries; normal temporary
installation clones must be distinguished from externally retained live shares.
Do not clear entries to pass close or treat external aliases as retired.

## Retained capability and session inventory

Admitted access carries runtime/Home identity, binding, capability/operation names,
principal/input/projection/resolved request, deadline/scope, canonical work,
retained authorization and graph_work. The authorization records are decision,
grant and request facts; generic application input remains caller-owned data.
The platform resource-bearing edge is graph_work, not those descriptive identities.

WorthQueryManagedGraphWorkSession owns plan capacity reservations plus its basis:
ProductReadBasis owns both Product observation leases and a GraphReadOwnerPort;
the port owns RelationalSourceOwner and layout Arc. Mutation basis owns an optional
ApplicationSnapshotLease (snapshot, graph integration handle and product lease).
Read intent carries installed Inspection/Resource contract Arcs. Query source context
retains another product lease; prepared-query pin owns RelationalPinnedRead and
layout; managed-index session borrows parent/product and retains completed source
and a reserved plan. Read proofs and completion receipts carry identities/facts.

Provider protocol also owns resources: SessionLease.provider is
Arc<GraphProviderAnchor>, with token/request and a Drop abort; ProductAffinity's
Application variant owns ProductBranchLease. SessionAffinity retains run borrow,
contract, that product and lease. These must not become portable authority values.
The bind-at-issuance/pass-at-use split must keep required abort/cleanup custody
with the receiver owner, not drop it at issuance or in a court.

Integration's protected application commit progression session_admission.rs stores
staged SessionBoundReadsAndEffects and binds product by value before readmit/prepare.
This is a later execution owner, not proof that portable capability issuance must
change it. Independent source review did not establish any required protected
edit. The earlier stop claim is withdrawn; the issuing-owner split proceeds
without changing that progression frame. No protected file is edited.

## Planned implementation and evidence

1. Complete boundary/API inventory, including real durable fork reach and actual
   overflow traces with compiler-emitted frame allocations.
2. Factor contribution sealing, installation, transfer and accepted-output recovery
   into their existing ordinary owners; consume them from Home with actual owners.
3. Make owner-consumed producer retirement preserve the original registry and refuse
   external live shares. Split portable grants from receiver-owned execution state.
4. Stop/report any required protected file/field or missing artifact admission seam;
   do not create a second installation path, a test door or a parallel file format.
5. Compile the affected owners once per coherent batch before focused named lanes.
   Use existing runners; one build at a time, stop below25GB. Never execute ignored,
   scale, exhaustive Home oracle or Store scenario suites. Run phase checks at the
   implementation phase boundary; obtain fresh independent review with these rulings
   before committing. Keep complete exact lane logs and the external progress note.


## Current capability implementation batch

Admitted access no longer stores ManagedGraphWorkSession. The issuer retains only
its actual runtime/principal/binding/session identities, branch, capability context,
decision count, an owner-issued nonretaining Native basis-admission identity and a
pure Product selector. Original read/plan resources end with issuance. Native's
basis_admission_identity.rs explicitly guarantees its token has no basis, runtime
or retention lease. Receiver checks Home incarnation before admitting that selector
and acquiring live snapshot resources. Selected-index preparation also carries
only that opaque identity, rather than retaining a Native basis. Issuance is split
into its own module to respect the file cap; no new public authority type is added.
The protected commit progression is unchanged. Focused current, foreign and
changed-state controls pass (28/0/0). The authentic directory N+1 capability
court passes through nextest and directly with RUST_MIN_STACK absent, including
its current-incarnation governed read after the stale refusal.

A draft recorder was rejected during API inventory: ProviderSessionToken is
move-only and its view is protocol-private. Do not derive Clone, copy a token into
an observer, fabricate authority from an observation or count a metadata comparison
as a real N+1 provider-session journey. The authentic session journey remains open.

The Query court's own two open outcomes now live in caller-owned Box storage;
all original close, receipt, supplier and balance assertions are preserved. This
addresses its measured243816-byte baseline frame without changing Host opening,
Native publication or integration's protected commit progression.

## Outstanding owner seams

The proposed live-session split requires a receiver parameter at physical use:
`protocol/readmission/prepared.rs:36` currently exposes `prepare(self)` and
obtains the provider exclusively from `SessionLease.provider`. Removing that
port without supplying a receiver would remove the only ordinary call target.
Moving live provider/product and Drop-abort custody to a receiver-owned session
table therefore requires changing the actual call at
`primary_graph/application_attempt/provider_execution/phase/prepared/running/progression/session_admission.rs:38`
from `session.prepare()` to receiver-supplied preparation. That protected file
also stores `WorthQueryAdmittedProviderSession.staged`, the current resource-owning
phase. No edit is made. This is a concrete dependency of this proposed protocol
cutover, not a claim that capability issuance requires it or a proof against all
possible session designs. Authentic N+1 live-session qualification remains open.

Home accepted-output restoration still needs an admitted durable artifact:
ordinary `DecodedApplicationCheckpoint.accepted_outputs` and funded metadata
are not in Home's Native restore work. Native locator-only priors explicitly
cannot establish accepted Ready lineage (`application_checkpoint/capture/native_priors.rs`).
The existing Store registry requires `query.obligations` absent. A new format or
a P-A checkpoint transport is a boundary decision; no second Home installer or
speculative artifact format is introduced while that decision is pending.
Installation and installed-registry retirement are not implemented or qualified.

For the sibling court, the capability index task rows555-559 and the linked
branches guide expose the ordinary `create` route. Searches for `fork_expected`,
`branches().fork`, `DurableBranchPublicationRequired` and durable branch creation
in the certification examples and Bank reference application find no admitted
durable sibling-publication example. World `service_ports/branch.rs` exposes only
ordinary create/retire; the create owner deliberately refuses durable Homes.
The public durable publication port exposes bootstrap-root and advancement,
not sibling creation. Bootstrap is already performed by Home and cannot create
a second root. The needed API is source-bound durable sibling creation/publication
preserving the exact selected Native and Signal components. The court and every
affinity/supplier/balance assertion remain unchanged; no new door is added.

## Frame ownership and bounded qualification

Actual first-chance stack traces plus compiler PE unwind allocations are retained
in `target/lane-evidence/pc-d7-rulings-stack-{query,world,query-selector}-frames.json`.
Executable hashes, exact names, source locations and unset-stack environment are
recorded there. Query's baseline failure occurs at its first balance mutation,
not Native `decode_payload`:201 frames, with Native `register_candidate`245048B
and protected integration commit progression131288B beneath the own court243816B.
Boxing the owning Home outcomes fixes that court; its direct run passes.
The selector court's later direct abort occurs during uncheckpointed journal
reopen:144 frames, including Host `prepare_directory`296648B, Native candidate
245048B, Host `readmit_native`181416B and the own court149256B. Boxing its original
owning Home outcomes also fixes the direct run. Original assertions remain.

World's external-read close court overflows during the positive Native reopen
at `world_home.rs:153`. Its66-frame trace enters Store manifest decode97400B
under integration-owned recovery handoff529608B and construction234568B; the
court itself is271944B. Integration owns the dominant physical-recovery frames.
No protected recovery or refusal-close frame is changed; this control remains
failed pending integration's repair and must be rerun after that fix merges.

The independent baseline-inventory review approved only its limited snapshot.
Fresh source review of tree1e1514039a8e5ca2dfff516460d5b68ab5e93b99 rejects
whole-work acceptance: incomplete installation/retirement, omitted trusted-time
input, incomplete live-session cutover, seven assigned Native boundary findings,
integration's World stack abort and the missing durable sibling API. The reviewer
found no supported material correctness issue in the capability custody removal
or the two Query court storage repairs, and independently verified47 raw log
hashes. This inventory correction records the accepted trusted-time finding;
it does not claim any of those remaining production obligations are complete.


## Typed exact-observation receiver correction

The second fresh review found a design flaw within the portable-capability batch:
exact observation returned Option, so elevation/delegation could classify an
old runtime as absent projection/support before the typed Home check. The
correction makes this owner return Result<Option<Output>, AuthorizationDenial>;
its six consumers must propagate the outer refusal. The original optional
current/foreign classifications remain inside that result. A shared stale-only
guard also runs at elevation, review, close, delegation and replay entry before
phase/role/projection/time work; it grants no admission to foreign authority.
The existing real N+1 disclosure court exercises that same guard at read
validation. Current/foreign/lifecycle/replay controls and that filesystem court
are required after compilation. No synthetic stale access or new test door is
introduced, and no protected commit progression is edited. The affected compiler
check passes. The51 named current/foreign/lifecycle/replay controls pass, as do
the three selected Host courts and each direct run with RUST_MIN_STACK absent.
The real filesystem stale-disclosure court exercises the same shared stale
guard; it is not a separate filesystem elevation/delegation journey. The typed
outer observation result makes its six consumers explicitly preserve or handle
the refusal. Evidence is indexed in pc-d7-rulings-lane-index.json. Fresh source
review is still required. This does not close the live-session obligation.
