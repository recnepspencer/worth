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

## Next slice after integration 82e2661e68

The coordinator released the live session-preparation seam and deferred accepted
outputs to D12's Query application section. Same-build accepted-output reuse and
D16 therefore remain outside this slice. Durable sibling implementation also
waits for D12; its reviewed design lives at
`target/lane-evidence/durable-sibling-publication-design.md`, with original Native
association/ACK evidence, dependency-ordered image/tail readmission, typed custody
and an exact file inventory. No owner format changes are implemented here.

The session cutover separates a portable `WorthQueryProviderSessionGrant` from a
scoped `WorthQueryProviderSessionReceiver`. Issuance checks the real run/product
and advancement phase, reserves the existing session generation, and carries
only the sealed contract and original token-admission grant. The contract's
SnapshotHandle is scalar identity, with no source or retention owner. Preparation
receives the actual run/product/provider owner, checks the original Home fence
before contract comparison or physical readmission, then opens and prepares the
provider session. The existing SessionLease keeps its original-request Drop-abort
custody throughout physical prepared/staged/terminal phases. No replacement
request, token cloning, metadata-made authority or new session table is added.

This deliberately retires the former implementation claim that issuing the
retained grant itself opens a physical session (6b70aaeac4's
`dropping_a_readmitted_session_aborts_once`). The retained court is renamed
`dropping_a_portable_session_grant_has_no_physical_session_to_abort` and requires
zero readmission/abort effects. All physical prepared/staged/panic/terminal abort
counts remain unchanged. Receiver-owned compiled controls capture authentic grants
at that boundary; their handles have no provider or product port. The original
grant and capture handle both survive Home abandonment in the new filesystem
court. A distinct foreign-Home court uses the same actual preparation receiver.

The first Host qualification found a design flaw in late grant injection: a new
mutation evaluates its handler before it reaches session admission. Late
injection could not establish refusal before supplier evaluation. It is removed.
The real runtime now exposes `prepare_provider_session_grant` with its actual
installed provider and Home witness and a lazy live-receiver constructor. It and
the direct protocol path share the same preparation owner. The Home fence runs
before invoking that constructor; a current run must still match the exact
original contract and provider pointer. The Host courts move their authentic
grants directly into this real N+1 API, and forbid constructing any new live work.
The original issuing request legitimately evaluated its handler once; refusal
on the receiving Home must add zero supplier calls. No late replacement control
or synthetic grant remains. Existing actual-session affinity controls exercise
the same runtime entry positively.

The governing coordinator decision is: "retained session values carry identity
and grant only, and the receiver supplies the live provider at use." Physical
session readmission therefore begins at receiver preparation. The existing
readmission-rejection/panic and prepared/staged Drop-abort courts still require
their original callback counts and recovery postures; none is deleted or ignored.

Home now carries its optional trusted authorization clock through the existing
Home state, without sampling at open. MemoryOnly delegates to the existing
source-aware ordinary construction owner. Directory attachment uses the same
RuntimeClock owner as ordinary installation, including on reopen. Every direct
prepare_home_program caller must choose a source or explicit system default.

The remaining shared installation cutover is not implemented. Producer routes
allocate Signal nodes through Bridge's unsealed builder; the Home Bridge is
already sealed. Its extension owner only replaces existing definitions. D15's
InitialEmpty contract forbids allocated nodes/definitions. A coordinator question
is pending on the new owner installation/seed contract. Producer/conditional/
resource installation and owner-consumed registry retirement stay together; no
unsupported refusal is recorded as the required produce/reopen/regenerate proof.

### Requirement-to-court map for this slice

These are qualification targets, not pass claims. Full log results are indexed
under `target/lane-evidence/pc-d7-next-*` after execution.

| Requirement | Court / compiler enforcement |
| --- | --- |
| D7: retained session has no Home ports | Grant has no lifetime, provider, request, product lease or run borrow; the separate receiver owns those fields. `reopened_home_refuses_the_authentic_prior_provider_session_before_any_provider_effect` keeps the real grant and capture handle through abandonment and actual N+1 open. |
| D7/D17: stale before effects | Same court requires typed StaleIncarnation, zero provider readmissions/preparations/registrations and no additional supplier contacts on the receiving Home, a forbidden live constructor, unchanged balance, and a successful fresh request. |
| D7: foreign remains foreign | `foreign_home_session_grant_is_refused_before_provider_preparation` requires ForeignGraphAuthority with the same zero-effect counters and successful current control. |
| D17: exact session binding | `token_minted_for_an_earlier_plan_cannot_open_a_later_plan`, existing direct/workflow protocol and independent-affinity courts; physical token generation must equal the original reserved grant generation. |
| D7: honest physical cleanup | Existing prepared/staged/panic/rejection/commit/abort courts retain their exact original backend abort counts; port-free issuance court requires no physical effect. |
| Phase 3: trusted time input | `directory_reopen_uses_the_new_trusted_clock_without_sampling_during_open` and `memory_home_uses_its_supplied_trusted_authorization_clock` use real public Home requests and capability admission. |
| D7: retained capabilities/selectors | Existing real filesystem stale-capability and stale-selector courts, with current governed-read and exact durable-retry controls. |
| D7: process lease | `competing_host_processes_admit_exactly_one_writer_until_process_death`, including direct unset-stack execution. |
| Changed-build native/authored admission | `changed_host_build_keeps_initializer_closed_and_reaches_native_readmission`; this is not producer/accepted-output lineage qualification. |
| Default stack after merge | World `closed_world_releases_its_bridge_and_retains_the_same_native_home_until_external_reads_release` and selected Host controls run directly with RUST_MIN_STACK absent. |

### Qualification of the first reviewed candidate (a6a627a951)

Evidence index: `target/lane-evidence/pc-d7-next-lane-index.json` and `.md`.
The logs record their actual parent HEAD and dirty state, not a claimed clean
committed-HEAD execution. Broad suites, ignored/scale cases, exhaustive Home
oracle, Store scenarios and tool self-tests remain unrun.

Root/Query/Store initial workspace checks and the affected Query/Store/public-
audience test checks pass. The first 87 unit controls pass. The corrected
receiver batch passes 91 affected/projection controls. The first eight-case Host
court lane records 6 passed and 2 failed; both failures and the affected current
paths pass the four-case correction lane and each direct unset-stack run. The
unaffected capability/time/foreign-selector controls retain their earlier green
evidence. The three Store Host controls pass; contention and changed-build
history also pass directly without RUST_MIN_STACK. Public audience is 1/0/0;
the four changed rustdoc phase contracts are 4/0/0. The World external-read close
control now passes directly, resolving its former integration-stack blocker.

Formatting passes; dirty line caps pass for 83 Rust files; generated context
passes without regeneration. Boundary-check fails with 10 BC9002 findings.
Seven are the coordinator-assigned Native courts. Integration 82e2661e68 adds:

- `crates/worth-relational/src/authority/commit/pipeline/phase_backing_tests.rs`;
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/publication_tests.rs`;
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared/running/completion_storage.rs::tests`.

These three files were unchanged by the first P-C candidate relative to the
original merge tree. Independent review rejected that integrated candidate for
these three new scoped violations; the seven assigned P-A paths were separate.

### Resolution of the source-court review finding

The three new source courts move into the existing court binaries. No test claim,
case, assertion or ignored status is removed. The production owners supply narrow
compiled controls, sealed to the exact court files in control-callers.txt:

| Existing cases | New court owner | Compiled owner control and facade |
| --- | --- | --- |
| phase_backing_tests.rs, two candidate backing cases | worth-court-relational-certification/process/commit_phase_backing.rs | RelationalCommitPhaseBackingForTest, facade::transactions |
| symbol_table/publication_tests.rs, three canonical symbol custody cases | worth-court-relational-certification/process/symbol_publication.rs | RelationalSymbolTableForTest, facade::symbols |
| completion_storage.rs::tests, three original-grant completion cases | worth-query-court-query-decl-host/process/certification/managed_phase_storage.rs | WorthQueryManagedPhaseStorageForTest, facade::primary_graph |

Each control delegates to the actual original owner; backing/phase storage owns
its actual production value and Drop custody. The symbol control owns the actual
RuntimeSymbolTable and observes its stored configuration key. Its text constructor
delegates the existing managed-text constructor with the original request. The
first relocation check exposed four private-constructor calls; moving those calls
through this same owner control fixes court reach without changing production
symbol allocation. No BC9002 exemption or new feature is added. Internal facade
forwarding and the three exact caller entries are the only boundary additions
for this correction. The seven coordinator-assigned P-A courts remain untouched.

Constitution completion is not claimed while their findings remain.
Installation/registry retirement and directory producer changed-build legs remain
unimplemented and unrun pending the InitialEmpty/installed-definition contract
ruling documented above. Accepted-output restoration and same-build reuse remain
D12-deferred. The lane index records the corrected named cases and phase checks
separately from the retained first-candidate failures.

Final topology correction qualification: Root and Query selected check--tests pass;
the relocated Native court is 5/0/0 and Query court is 3/0/0. Formatting passes.
Dirty Rust caps pass for 100 files. Generated context passes without regeneration.
Boundary-check now reports exactly the seven unchanged P-A-assigned BC9002 paths;
the three first-review findings are absent. The prior ten-finding log and failed
private-constructor check remain in the evidence index. No broad or ignored lane
is run for this correction. The previously qualified D7 paths are unchanged.
