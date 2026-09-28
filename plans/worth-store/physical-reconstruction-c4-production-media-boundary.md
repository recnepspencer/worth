# Worth Store Physical Reconstruction C.4: Production Media Boundary And Stable Store Namespace

Status: implemented and verified

Roadmap position:

```text
C.3 sealed physical runtime authority and lifecycle
  -> C.4 production media boundary and stable store namespace
  -> C.5 durable page, segment, extent, and manifest path
```

## Goal

Turn the sealed but deliberately non-physical C.3 runtime into the first
genuinely machine-backed runtime phase. C.4 installs one concrete filesystem
owner, one stable and confined store namespace, one process-level mutation
owner, and one honest vocabulary for file effects, synchronization,
publication, partial completion, and indeterminate outcomes.

The milestone closes when only successful admission of a real supported
filesystem namespace can construct `MediaOwnedPhysicalRuntime`, every ordinary
effect crosses one fault-interposable production media boundary, and fresh OS
processes can reconcile the resulting namespace without receiving writer heap
state.

## Why This Milestone Exists

C.3 established who may progress toward physical authority, but it correctly
installed no media owner and exposed no physical operation. C.5 cannot assign
page, segment, extent, or manifest meaning to bytes until the runtime has one
real owner for paths, handles, positioned transfers, synchronization,
publication, and namespace confinement.

C.4 is therefore not a filesystem utility project. It is the trust boundary
between typed runtime authority and operating-system effects. If that boundary
lies about containment, exclusive ownership, short transfers, barriers,
rename durability, or post-failure uncertainty, every later durability and
recovery claim inherits the lie.

## Governing Summaries

- `MENTALITY.md` protects foundation-first engineering under adversarial
  pressure. Its strongest constraint here is that the namespace, durability,
  and failure model must survive crash seams and hostile paths before page or
  WAL features are allowed to depend on it.
- `arch_laws.md` protects compiler-visible authority progression and explicit
  boundary crossings. Its strongest constraint here is that C.3 authority is
  consumed into a distinct phase only after the concrete media owner exists,
  with partial and indeterminate effects carried in typed outcomes rather than
  repaired by convention.
- `composition_laws.md` protects one named responsibility per file and per
  function. Its strongest constraint here is that admission, confinement,
  transfer mechanics, durability, ownership, publication, accounting, and
  shutdown remain separately readable responsibilities instead of becoming a
  private filesystem runtime in one module.
- `domain_structure_laws.md` protects truth-source and external-boundary
  topology. Its strongest constraint here is that namespace grammar, OS media
  effects, runtime composition, and certification observation occupy distinct
  owners, and that every filesystem crossing is locatable without grep.
- `perf_laws.md` protects cost honesty and bounded mechanical work. Its
  strongest constraint here is that ordinary operations are range-oriented,
  allocation-scoped, and counter-bearing; capability discovery and namespace
  reconstruction remain explicit admission cost rather than hidden hot-path
  work.
- `physical-foundation-reconstruction-roadmap.md` protects the sequence from
  sealed authority to real byte survival. Its strongest C.4 constraint is that
  no memory/mock backend, supplied heap representation, isolated file writer,
  or test-created artifact can earn construction of the media-owned runtime
  phase.
- `worth-proof` protects proof-bearing progression law. Its strongest C.4
  constraint is that root/profile capability qualification and the consuming
  C.3-to-C.4 transition preserve success, denial, deferment, staleness,
  rebind-required, and terminal failure as distinct typed outcomes without
  replacing the Store-owned runtime or media owner.
- `worth-foundational` protects shared boundary meaning. Its strongest C.4
  constraint is that namespace identity, canonical comparison, support
  evidence, and performance evidence lower from stronger Store-owned facts
  only at explicit boundaries; Foundational artifacts never establish an OS
  effect or promote runtime authority.

## Adversarial Constraint

> Across every supported Windows and POSIX profile, with multiple processes
> contending for one root and deterministic failures injected before, during,
> and after every namespace and durability edge, at most one process may hold
> mutation authority; every effect must remain confined beneath one persistent
> store identity; every result must distinguish denied, partial, completed,
> and indeterminate effects accurately; and a fresh process must reconcile the
> resulting paths, lengths, bytes, ownership, and publication state without
> receiving writer heap state. No mock, memory backend, raw path, replay
> artifact, test authority, copied identity, digest, profile report,
> Foundational boundary artifact, or declaration-only capability may construct
> the media-owned runtime phase. Only current root/profile-bound proof produced
> from real operating-system behavior may mint the concrete capabilities that
> the consuming runtime transition requires.

## Product Decision Lock

1. C.4 supports qualified local filesystems first. Network, removable,
   userspace, virtualized, or otherwise unqualified filesystems receive a typed
   unsupported-profile denial until separately qualified.
2. C.4 admits exactly one process-level mutation owner for a store namespace.
   Stable cross-process runtime reads wait for C.10 physical read leases.
3. A persistent lock file may expose diagnostic owner metadata, but the live
   operating-system lock held by an open handle is the mutation authority. A
   PID, timestamp, or stale lock-file payload is never authority.
4. The persistent store identity is generated once during namespace creation,
   durably published, and thereafter discovered from bytes. It is not a path
   digest, runtime id, process id, or caller-supplied id.
5. Namespace admission detects whether the declared root is absent, staged,
   durably initialized, incompatible, contended, or ambiguous. It does not ask
   callers to assert which state exists.
6. Namespace meaning and stable byte grammar belong to
   `worth-store-physical-format`; OS file effects and capability qualification
   belong to `worth-store-physical-backend`; `worth-store` owns runtime
   composition and phase progression.
7. The public runtime facade exposes admission, immutable observation, and
   consuming lifecycle transitions. It does not expose arbitrary raw file
   creation, paths, handles, reads, writes, renames, or deletion to product
   consumers.
8. Artifact owners in later milestones consume sealed namespace-relative
   capabilities. They never provide arbitrary `Path`, `PathBuf`, or string
   paths to the media owner.
9. The production phase contains one concrete filesystem owner. There is no
   public generic backend parameter, backend selector, memory implementation,
   or trait-object constructor for `MediaOwnedPhysicalRuntime`.
10. Capability admission is root- and backend-profile-specific. Operating-
    system name alone is not capability evidence.
11. Optional mmap, sparse allocation, preallocation, and direct-I/O behavior is
    represented by concrete admitted capability handles. A boolean flag or a
    method that merely returns `Unsupported` after callers enter the operation
    is not sufficient.
12. The ordinary media path is synchronous at C.4. C.10 may schedule and pace
    these explicit operations, but C.4 does not hide a scheduler, thread pool,
    asynchronous runtime, or background retry loop inside the port.
13. A completed write, file synchronization, atomic replacement, and durable
    directory publication are distinct facts. No single `flush` or `sync`
    vocabulary collapses them.
14. Retrying an operation is permitted only when its typed outcome says retry
    is safe. Indeterminate publication is preserved for later inspection.
15. Namespace cleanup deletes only residue whose C.4 ownership and
    non-publication are proven. Ambiguous residue is classified and preserved.
16. C.4 performs no page, WAL, checkpoint, index, blob, semantic MVCC,
    recovery-source, stable-read, or scheduled-I/O work.
17. The certification interposer may decorate the admitted real filesystem
    owner with deterministic faults and observation. It cannot replace the
    backend, choose persisted truth, create the runtime phase, or write the
    expected result itself.
18. Every existing direct writer found in C.4's causal call-path review receives a migrate,
    quarantine, or delete disposition before C.4 closes. No ordinary physical
    path may bypass the admitted media owner.
19. C.4 admits one configured service-account access posture. Store performs
    no permission-widening step and creates entries through ordinary POSIX
    mode/umask or Windows ACL inheritance. Auditing arbitrary deployment ACL
    intent and multi-tenant ACL semantics remain outside this milestone.
19a. The admitted service account is the namespace mutation trust boundary.
    Every cooperating process using that account must mutate through the media
    owner and contend for the canonical OS lease. Direct mutation by an
    administrator, kernel component, storage appliance, or unmanaged process
    running with the same credentials is out-of-contract tampering: admission
    and reconciliation must detect visible drift and fail closed, but C.4 does
    not claim a portable atomic compare-and-rename/unlink primitive against an
    actor the operating system itself authorizes to bypass the lease. Phase 8
    must qualify this access posture before production authority can exist.
20. Namespace, contention, and fault scenarios compile into one
    `physical_media_journeys` integration-test executable. Separate tests and
    child-process roles do not justify separate linked copies of the same
    runtime graph.
21. One tiny `physical_media_os_observer` binary remains separate because
    process identity and dependency independence are part of the proof. Its
    source imports no runtime, backend, certification, or production decoder
    surface.
22. One `physical_media_authority_ui` runner owns a small representative set of
    authority-class fixtures. Per-method, per-type, per-platform, and per-error
    compile fixtures are forbidden.
23. Compiler UI is used only when the required guarantee is that invalid code
    cannot compile. Runtime lifecycle, path, fault, counter, concurrency,
    capability, and fault behavior belongs in ordinary tests.
24. Add a compiler fixture only for a materially distinct compiler-enforced
    authority class; consolidate or delete redundant cases during review.
25. All UI fixtures share one Cargo target root and runner compilation. No
    fixture may invoke Cargo, create a target directory, or become its own
    test target. The consolidated runner declares the certification feature
    directly and compiles the complete authority boundary once in its maximal
    profile; ordinary default-profile compilation remains owner-lane work.
26. Warm owner feedback targets under ten seconds and the C.4 developer-smoke
    product targets under one minute on the declared reference machine.
    Maximal-feature UI and multi-process fault campaigns
    remain CI/release products and do not inflate ordinary owner feedback.
27. Store-owned types remain the strongest forms while meaning is local to
    physical format, backend execution, or runtime composition. Neither shared
    substrate replaces `StableStoreIdentity`, `FilesystemMediaOwner`, media
    operation outcomes, mutation leases, or phase-bearing runtime types.
28. `worth-proof` is mandatory for checked capability qualification and the
    consuming media-admission progression. Its witnesses authorize named
    transitions; they are never substitutes for the concrete root handle,
    live OS lease, backend owner, or established media effect.
29. `worth-foundational` is used only where C.4 meaning becomes portable or
    cross-owner: canonical namespace basis, identity-boundary lowering,
    independent comparison, support evidence, and counter-backed performance
    evidence. It is not on the primitive I/O hot path.
30. C.4 extends or replaces the existing proof-backed backend capability lane
    in one visible migration. It may not leave the existing
    `BackendCapabilityClaimOutcome` beside a second unrelated media
    qualification authority system.
31. C.4 accepts no caller-provided `platform_authority`. No real upstream
    platform owner exists at this roadmap point. Private or public zero-input
    witness minting that merely decorates C.3 admission is forbidden.
32. Physical framing checksums and durability barriers remain Store-owned
    mechanics. A Foundational canonical digest is comparison evidence, not the
    namespace record checksum, store identity, mutation authority, or proof
    that bytes reached stable media.

## Authority And Truth Model

### Store-owned current facts and authority-bearing objects

- `AdmittedPhysicalRuntime` is the sole C.3 authority eligible to attempt C.4
  progression.
- `StableStoreIdentity` is persistent namespace identity decoded from the
  durably published identity record. It is an authoritative identity fact, not
  permission to open, mutate, or promote a runtime.
- `FilesystemMediaOwner` is the concrete internal owner of admitted root,
  handles, backend capabilities, mutation lease, and media effects.
- `MutationOwnershipLease` is the live process-local representation of the
  operating-system mutation lock. It is move-only and dies with the media
  owner.
- `MediaOwnedPhysicalRuntime` is the sole runtime phase carrying the installed
  filesystem owner.
- concrete qualified media capability handles are Store-owned authority minted
  only after the proof-bearing qualification transition succeeds for the live
  root/profile basis.

### Derived or observational objects

- normalized display paths, owner metadata records, capability reports,
  operation receipts, counter snapshots, diagnostic traces, and artifact
  manifests are observations. None can reopen, mutate, or promote a runtime.
- a persistent lock-file payload describes the last known owner attempt; it is
  not the live lock.
- a capability qualification report explains why capability handles were
  admitted; it cannot construct those handles.
- certification evidence and OS-observer reports compare effects; they cannot
  become namespace or media authority.
- `worth-proof` witnesses, checked outcomes, and basis/freshness forms describe
  legal progression; none owns a file handle, mutation lease, or runtime.
- Foundational canonical, identity, diagnostic, receipt, and performance
  artifacts are portable boundary meaning derived from Store-owned facts. They
  do not become stronger than their Store source and cannot be readmitted
  without the owning Store boundary revalidating current source truth.

### Mechanism ownership

| Responsibility | Owner | Explicitly does not own |
| --- | --- | --- |
| Namespace identity and version grammar | `worth-store-physical-format` | OS handles, locks, sync, runtime lifecycle |
| Relative namespace roles and publication-name grammar | `worth-store-physical-format` | Artifact-family semantics introduced after C.4 |
| Filesystem handles, transfers, metadata, barriers, rename, and delete | `worth-store-physical-backend` | Store identity meaning and runtime promotion |
| Root capability qualification and OS mutation lock | `worth-store-physical-backend` | C.3 lifecycle and product facade |
| C.3-to-C.4 consuming transition and lifecycle propagation | `worth-store` | Byte grammar and backend syscall mechanics |
| Checked qualification and phase-progression topology | `worth-proof` | OS execution, Store identity meaning, runtime ownership |
| Canonical comparison and portable boundary evidence | `worth-foundational` | Runtime execution, OS effects, mutation authority |
| Fault scheduling and independent comparison | certification surfaces | Backend replacement or production authority |

Rust has no visibility that grants a backend item to exactly one sibling crate.
Accordingly, the Store runtime progression and public facade are compiler-sealed,
while activation of the unpublished backend's `store-runtime-owner` feature is a
workspace-policy boundary enforced by the Store test runner. The code and tests
must not describe that feature gate as compiler sealing: publishing the backend
or permitting another manifest consumer requires a deliberate boundary redesign.

## Critical DX Target

Ordinary application code sees one obvious progression and no raw media
surface:

```rust
let admitted = PhysicalStore::admit(PhysicalRuntimeAdmission::new(
    declared_store_root,
)?)?;

let media_owned = match admitted.try_admit_filesystem_media(
    FilesystemMediaAdmission::local_filesystem(media_configuration),
).into_raw() {
    TransitionOutcome::Success(media_owned) => media_owned,
    TransitionOutcome::Denied(denial) => {
        let (admitted, report) = denial.into_parts();
        emit_media_admission_report(report);
        return admitted.close().map(|_| ());
    }
    TransitionOutcome::Deferred(deferred) => return defer_media_admission(deferred),
    TransitionOutcome::Stale(stale) => return refresh_media_admission_basis(stale),
    TransitionOutcome::RebindRequired(rebind) => return rebind_media_admission(rebind),
    TransitionOutcome::Failed(indeterminate) => {
        return require_media_admission_inspection(indeterminate);
    }
};

let observation = media_owned.observer();
assert_eq!(observation.store_identity(), media_owned.store_identity());

let closed = media_owned.close()?;
```

The failed consuming transition returns the still-admitted C.3 authority when
no media effect made safe retry impossible. A failure after an indeterminate
namespace publication returns an inert terminal outcome plus an inspection
report; it never hands back authority that might race an effect whose status is
unknown.

Later artifact owners receive internal, responsibility-specific capabilities,
not a public filesystem escape hatch:

```rust
// C.5-internal composition target, not a public product API.
let (runtime_core, filesystem_media_owner) = media_owned.into_record_path_parts();
let serving = install_record_path(runtime_core, filesystem_media_owner)?;
```

## Target Directory Skeleton

This is a strongly opinionated target, not permission to create empty modules.
Files land only when their responsibility is implemented.

```text
workspaces/worth-store/crates/
  worth-store-physical-format/src/store_namespace/
    mod.rs
    identity_record.rs
    identity_canonical_basis.rs
    identity_boundary.rs
    namespace_version.rs
    namespace_roles.rs
    staged_name.rs

  worth-store-physical-backend/src/filesystem_media/
    mod.rs
    admission.rs
    backend_profile.rs
    capability_qualification.rs
    namespace_confinement.rs
    directory_handle.rs
    file_handle.rs
    positioned_transfer.rs
    append_transfer.rs
    allocation.rs
    metadata.rs
    synchronization.rs
    namespace_publication.rs
    mutation_ownership.rs
    operation_context.rs
    operation_outcome.rs
    operation_counters.rs
    shutdown.rs

  worth-store/src/physical_runtime/media_ownership/
    mod.rs
    admission.rs
    admission_outcome.rs
    runtime.rs
    observation.rs
    resource_lifecycle.rs
    shutdown.rs

  worth-store/src/bin/
    physical_media_os_observer.rs

  worth-store-certification/src/media_boundary/
    fault_schedule.rs
    fault_interposer.rs
    os_observer.rs
    evidence.rs

  worth-store/tests/
    physical_media_authority_ui.rs
    physical_media_authority/
      supported_media_admission.rs
      media_runtime_authority_is_sealed.rs
      non_authority_values_cannot_promote.rs
      raw_media_surface_is_private.rs
      optional_capabilities_require_handles.rs
      maximal_features_cannot_mint_authority.rs
    physical_media_journeys.rs
    physical_media_journeys/
      namespace_discovery.rs
      mutation_contention.rs
      partial_effects.rs
      child_dispatch.rs
```

The backend directory is allowed to decompose further by stable responsibility
when Windows and POSIX mechanics genuinely have different failure or durability
topologies. It must not split into platform folders merely to duplicate the
same lifecycle, and it must not introduce `helpers`, `common`, `utils`,
`manager`, or C.4/phase provenance names.

## Compilation And Test Cost Contract

- C.4 uses direct owner tests, the `physical_media_journeys` suite, a
  dependency-minimal OS observer, and focused compiler authority checks.
- `physical_media_journeys` spawns itself
  into writer, reopener, contender, successor, and faulted-writer roles. Those
  roles are new processes, not new Cargo targets.
- `physical_media_os_observer` is the only separate scenario helper. It is a
  normal tiny binary target with `test = false`, built once and located through
  Cargo's integration-test binary environment, then spawned by the journey
  suite. It does not pay for an empty libtest harness.
- `physical_media_authority_ui` keeps one representative compiler denial per
  materially distinct authority class. Fixtures are ordinary test inputs, not
  an inventory or completeness authority.
- The complete authority UI runner executes once in the maximal certification
  profile. Default-profile production compilation is covered by ordinary
  owner checks and strict lint; it does not duplicate the trybuild campaign.
- No C.4 behavioral test invokes Cargo, rustc, or trybuild. Only the one UI
  runner may invoke the standardized cache-sharing UI harness.
- No test writes mandatory evidence for another test to consume. Scenario
  evidence is disposable output from the owning executable.
- Adding a broad or expensive lane to developer smoke requires an explicit
  cost review against C.1 warm-time and dependency-breadth expectations.
- A slower correct test moves to the proper CI/release lane; it is not split
  into more binaries or weakened to satisfy the smoke budget.

## Phase Plan

### Phase 1: Freeze The Media Boundary And Its Negative Space

Freeze the owner map, public facade, operation vocabulary, and the capabilities
that remain impossible before and after C.4. This phase produces design and
compiler contracts only; it does not introduce `MediaOwnedPhysicalRuntime`
before a real owner exists.

**Relevant subsystems**

- C.3 physical runtime facade and lifecycle
- physical format namespace contract
- physical backend media contract
- existing proof-backed backend capability admission
- Foundational canonicalization and boundary-identity facades
- certification feature and dependency topology
- current physical-writer call paths

**Relevant APIs**

- existing `PhysicalStore::admit(PhysicalRuntimeAdmission)`
- existing `AdmittedPhysicalRuntime`
- planned `FilesystemMediaAdmission`
- planned `MediaOwnedPhysicalRuntime`
- planned `MediaAdmissionDenial`
- existing `BackendCapabilityClaimOutcome`
- `worth_proof::ProofOutcome` / `worth_proof::TransitionOutcome`

**Required boundary inventory**

- Name every operation the backend must support: open/create, positioned read,
  positioned write, append, truncate, allocation, metadata, directory listing,
  file synchronization, directory synchronization, same-namespace atomic
  replacement, and deletion.
- Classify each operation by transfer cardinality, possible partial effect,
  synchronization meaning, retry posture, handle authority, and required
  capability.
- State which calls are owner-internal, artifact-owner-internal, observable,
  certification-only, and public.
- Trace every in-scope production or certification callsite that writes files
  and migrate, quarantine, or delete each bypass in source.
- Freeze the negative-space list: C.4 exposes no record append, page lookup,
  WAL append, checkpoint, recovery, stable semantic read, maintenance, layout,
  index, blob, Query, or MVCC operation.
- Classify every proposed shared-substrate use as Store-owned authority,
  proof-bearing progression, or Foundational boundary lowering. Any type that
  cannot be assigned exactly one of those roles is rejected before code lands.

**Warnings**

- Do not create a universal `StorageBackend`, `FileSystem`, `IoManager`, or
  generic async abstraction. The contract exists for Worth Store physical
  media semantics.
- Do not add `MediaOwnedPhysicalRuntime` as an empty shell, feature-gated alias,
  optional field, or enum variant before Phase 9 installs the concrete owner.
- Do not expose backend operations publicly just because external integration
  tests are easier to write that way.
- Do not classify a writer as migrated because it calls a similarly named
  method. Trace it to the terminal OS effect and the runtime caller.
- Do not introduce a local generic witness, proof set, receipt envelope, or
  canonicalization vocabulary where `worth-proof` or `worth-foundational`
  already owns the exact shared meaning.
- Do not weaken a Store-owned owner or outcome into a shared substrate merely
  because a similarly named Foundational type exists.

**Test requirements**

- A consolidated compiler specimen must prove that C.3 callers cannot name,
  construct, pattern-match, or obtain the future media owner or media-owned
  phase before the real transition exists.
- A dependency/source boundary test must reject ordinary `worth-store` product
  paths that import certification fault constructors, memory backends, or raw
  replay/persisted-layout authority.
- An operation-contract table test must fail if a new backend operation lacks
  transfer, failure, retry, counter, and capability classifications.
- The consolidated authority specimen must reject a copied identity,
  capability report, canonical digest, Foundational boundary artifact, and
  unrelated `worth-proof` witness as inputs to media-owner or runtime-phase
  construction.

**Engineering decisions**

- `worth-store` is the only crate that can progress C.3 runtime authority.
- The physical backend may expose a narrow contractual facade to its owning
  runtime crate; its internal platform topology remains private.
- `worth-proof` supplies checked progression topology; Store supplies every
  concrete authority marker, proving transition, basis value, and payload.
- `worth-foundational` receives only explicit lowerings from already-strong
  Store facts and cannot appear in primitive filesystem operation signatures.
- One consolidated UI target owns C.4 compile denials. Per-case Cargo projects
  and per-fixture target directories are forbidden.
- C.4 proof distinguishes backend owner tests from joined runtime journeys.
  Neither is allowed to claim the other ran.

**Open questions**

- None.

### Phase 2: Define Stable Namespace Grammar And Persistent Identity

Define the minimal on-disk namespace C.4 itself owns and the deterministic
classification of absent, initializing, initialized, incompatible, contended,
and ambiguous roots. This is format meaning only; no filesystem call belongs
in the format owner.

**Relevant subsystems**

- `worth-store-physical-format::store_namespace`
- `worth-foundational` canonicalization and authority-identity boundaries
- C.3 root declaration and runtime identity
- later C.5 artifact-family ownership
- offline C.4 namespace observer

**Relevant APIs**

- `StoreNamespaceVersion`
- `StableStoreIdentity`
- `StoreNamespaceIdentityRecord`
- `StoreNamespaceClassification`
- `StoreNamespaceRelativeRole`
- `StagedNamespaceName`
- `prepare_store_namespace_identity_canonical_basis`
- `StoreNamespaceIdentityBoundary`

**Canonical C.4 namespace**

```text
<declared-root>/
  namespace/
    identity
    mutation.lock
  families/
  staging/
```

- `namespace/identity` is a checksummed, length-framed record containing magic,
  namespace format version, persistent store identity, and encoding version.
- `namespace/mutation.lock` is a persistent lock target. Its optional owner
  payload is diagnostic and may be stale; the live OS lock is authoritative.
- `families/` is initially empty and reserved for responsibility-specific
  durable family roots installed by later milestones. C.4 exposes no generic
  public file allocation beneath it.
- `staging/` contains only names minted by the namespace owner. Each name
  carries a non-authoritative attempt identity sufficient to classify known
  uncommitted residue without being mistaken for published truth.

**Namespace classification**

- an absent root and an existing truly empty root are eligible for creation
  under the same admission request; the caller does not choose based on a
  preflight race
- a valid published identity record defines the stable store identity
- an exact C.4-only incomplete scaffold with no published identity is
  retryable after deterministic cleanup of proven staging residue
- any unknown file, directory, reparse point, malformed published identity, or
  conflicting identity candidate makes the root ambiguous or damaged and
  blocks media-owned admission
- an unsupported namespace version is distinct from corruption
- a valid namespace currently locked by another process is compatible but
  contended, not damaged

**Shared boundary lowering**

- `StableStoreIdentity` remains the strongest Store-owned identity while the
  meaning is inside physical format, backend admission, or runtime
  composition. It is not replaced by a generic Foundational id.
- Physical format exposes one explicit canonical-basis lowering for the
  decoded namespace version, encoding version, persistent identity, and
  publication posture using `worth-foundational` canonicalization.
- A Store-owned identity-boundary adapter may admit that canonical Store fact
  into Foundational identity vocabulary when it must cross an offline,
  integration, export, or support boundary. Raw record bytes enter that adapter
  only after physical-format validation.
- Crossing a process, transport, or persisted-observation boundary weakens the
  portable identity form. Current Store use requires fresh decoding and
  revalidation from the namespace record; a projection label or digest cannot
  readmit itself.
- The Foundational form is derived boundary meaning. It cannot open a root,
  acquire a lease, mint a backend capability, or construct a runtime phase.

**Warnings**

- The pathname is locator input, never store identity.
- Runtime id, generation, process id, lock-attempt id, and stable store identity
  are different types even if all use the same integer or UUID representation.
- Do not create page, WAL, checkpoint, index, blob, quarantine, or recovery
  directories in C.4. `families/` reserves ownership space without assigning
  those meanings early.
- Do not treat a parseable staged identity as published identity.
- Do not delete an existing nonempty root merely because no identity record is
  present.
- Do not use a Foundational canonical digest as the identity-record checksum
  or compare digest bytes in place of canonical semantic comparison.
- Do not expose the Foundational boundary form as a more convenient substitute
  for `StableStoreIdentity` inside ordinary Store code.

**Test requirements**

- A table-driven format test must encode and decode the identity record
  bit-for-bit across minimum/maximum supported values and reject bad magic,
  length, checksum, version, duplicate fields, and trailing bytes before any
  identity becomes available.
- A namespace-classification test must distinguish absent, empty, exact
  incomplete scaffold, valid initialized, incompatible, malformed,
  contended-compatible, and ambiguous non-store roots without consulting a
  runtime cache.
- A format-level metamorphic identity test must prove namespace
  classification accepts no locator, runtime, or process identity input and
  that identical published record bytes always yield the same stable store
  identity. Phase 13 owns the real root relocation plus fresh-process proof;
  the format owner must not grow a filesystem harness to imitate it early.
- A canonical-boundary parity test must independently lower two equivalent
  decoded records into equal Foundational canonical bases, localize version,
  identity, and publication-posture differences to distinct canonical loci,
  and refuse digest-only equivalence.
- A boundary-denial specimen must prove raw bytes, a proposed identity,
  runtime identity, path projection, canonical digest, and bridged identity
  cannot satisfy an API requiring the current Store identity boundary.

**Engineering decisions**

- Stable store identity candidate creation uses cryptographically strong operating-system
  randomness or the workspace's existing stable identity authority if that
  authority is already stronger. Pure format golden tests may construct
  nonzero candidate bytes directly because they prove byte grammar and grant
  no runtime authority; any deterministic identity entering namespace
  admission or publication requires a certification-owned concrete source
  that cannot enter ordinary admission.
- Namespace records use physical-format framing and integrity rules, not an
  ad hoc JSON/TOML/text file.
- The framing checksum remains a physical-format corruption detector.
  Foundational canonicalization begins only after successful decode and exists
  for reproducible boundary meaning and comparison.
- Store owns the concrete identity-kind and admission authority used by any
  Foundational identity lowering. Witness construction remains private to the
  validated format boundary and is never a public zero-input mint.
- C.4 initialization residue is recoverable only because C.4 knows every byte
  it could have created. Unknown residue is preserved for manual/offline
  inspection.
- The stable identity record is immutable after first durable publication.

**Open questions**

- None.

### Phase 3: Define Typed Media Outcomes And Backend Capability Vocabulary

Define the exact facts returned by a media operation before platform mechanics
are implemented. Later phases must not smuggle uncertainty through `io::Error`,
`bool`, byte count conventions, logs, or generic receipts.

**Relevant subsystems**

- physical backend contract facade
- existing proof-backed backend capability admission
- `worth-proof` checked progression topology
- runtime media admission denial
- C.4 diagnostics and counters
- later C.5/C.7 physical artifact owners

**Relevant APIs**

- `MediaOperationIdentity`
- `CompletedMediaTransfer`
- `PartialMediaTransfer`
- `MediaEffectStatus`
- `MediaOperationFailure`
- `MediaRetryPosture`
- `FilesystemBackendProfile`
- `QualifiedMediaCapabilities`
- concrete optional capability handles
- `BackendCapabilityClaimOutcome`
- planned `MediaCapabilityQualificationOutcome`

**Outcome topology**

- `DeniedBeforeEffect` proves no requested media effect occurred
- `PartialTransfer` carries requested bytes, completed bytes, exact offset or
  append position when knowable, and whether continuation is valid
- `CompletedEffect` carries the weakest sufficient fact actually established
- `IndeterminateEffect` carries the operation identity, attempted effect,
  last established boundary, and required inspection posture
- EOF on positioned read is a normal typed observation and is not conflated
  with an interrupted or partial read
- operating-system error code, operation role, path role, handle identity, and
  causal boundary remain machine-readable context; a display string is
  derived presentation
- primitive media operation outcomes remain Store-owned domain types. They do
  not become `worth-proof` phase transitions or Foundational receipts merely
  because they carry established facts.

**Capability topology**

- base qualified capabilities cover ordinary files, directories, positioned
  transfers, metadata, listing, deletion, required file synchronization, and
  the admitted namespace-publication protocol
- optional capabilities are concrete handles for data-only synchronization,
  sparse allocation, eager allocation, mmap, and direct I/O
- each optional handle carries alignment, granularity, scope, and semantic
  restrictions required to call its operation
- unsupported and indeterminate qualification are distinct; an unmeasured
  capability is never reported as supported
- qualification uses `worth-proof` checked outcomes so success, denial,
  deferment, staleness, rebind-required, and hard failure cannot collapse into
  `Result<bool, _>` or a report field
- the successful proof payload is still not an operational capability; the
  backend owner consumes it together with the live root/profile basis to mint
  the corresponding concrete Store capability handle

**Warnings**

- `std::io::Error` may be retained as a sealed source detail but is not the
  public or cross-owner error topology.
- Do not automatically retry interrupted or short operations below the
  boundary and then report only final success. Explicit whole-transfer
  operations may retry, but their receipts and counters must retain every
  primitive attempt.
- Do not use one generic `CapabilityProof<T>` or marker trait. Each admitted
  optional behavior has concrete domain meaning and a concrete type.
- Do not expose richer success claims than later callers require. Weakest-
  sufficient facts reduce false coupling.
- Do not wrap each successful syscall in Foundational evidence on the ordinary
  path. Runtime-local outcomes and counters are lowered only when an explicit
  support or certification boundary requests them.

**Test requirements**

- For every operation family, an outcome-law test must cover zero effect,
  partial effect, complete effect, and indeterminate effect where mechanically
  possible, and must prove retry posture matches the actual prefix.
- A controlled primitive-transfer classification case must make code that
  converts `completed_bytes > 0` into complete success fail at the causal
  assertion. Phase 11 repeats the same law through the real backend
  interposer; Phase 3 must not create a pretend backend merely to name the
  outcome vocabulary.
- A compile test must prove callers cannot invoke an optional operation from a
  boolean, profile report, or unsupported capability observation.
- The same authority specimen must prove a successful outcome for capability
  A, a witness from another authority family, a stale claim, a canonical
  digest, and a Foundational report cannot mint or satisfy capability B.
- A checked-progression test must preserve denied, deferred, stale,
  rebind-required, and failed qualification categories without manufacturing a
  real backend or collapsing them into display strings.

**Engineering decisions**

- Operation identities are generated by the media owner and are unique within
  one runtime incarnation. They are observation correlation, not persisted
  artifact identity.
- Admission failures that definitely precede media authority installation
  return the consumed C.3 runtime for close or retry. Indeterminate namespace
  publication consumes the authority into an inert inspection-required
  outcome.
- The existing `BackendCapabilityClaimOutcome` is the starting substrate, not
  a parallel legacy lane. Phase 8 must either extend it coherently or replace
  it atomically while preserving all existing consumers and compile-time
  denials.
- Store-specific capability types carry root, backend-profile, qualification,
  scope, and restriction meaning; `worth-proof` carries progression posture,
  not those domain semantics.
- Exact OS codes are retained without making platform-specific integer values
  part of cross-platform semantic matching.
- Retry policy is decided above primitive effects from the typed posture; the
  backend does not run an invisible unbounded retry loop.

**Open questions**

- None.

### Phase 4: Establish Namespace Confinement And Relative Path Authority

Make path escape structurally unavailable after root admission. Artifact
owners describe namespace roles and owner-minted relative components; only the
filesystem owner resolves them against its held root and directory handles.

**Relevant subsystems**

- physical-format namespace roles and staged-name grammar
- physical-backend root and directory handles
- Windows reparse/junction and POSIX symlink handling
- atomic replacement and deletion

**Relevant APIs**

- `AdmittedStoreNamespace`
- `NamespaceDirectoryHandle`
- `NamespaceRelativePath`
- `StagedNamespacePath`
- `NamespacePublicationTarget`
- `ArtifactFamilyDirectory`
- `StagingDirectory`
- `NamespaceConfinementDenial`

**Confinement rules**

- absolute paths, parent traversal, empty/interpretable-special components,
  device paths, alternate data streams, reserved device names, and embedded
  separators are rejected before OS resolution
- relative components are minted from stable format roles or later owner-
  admitted names, not accepted as arbitrary caller strings
- directory traversal proceeds from an admitted root/directory handle with
  no-follow or equivalent reparse protection at each boundary
- the backend validates the final opened object's canonical identity and
  namespace ancestry where platform mechanics can race between lookup and open
- source and destination of replacement must share the admitted namespace and
  the capability-required directory/volume scope
- deletion accepts an owner-issued file/directory capability, not a raw path
- no operation may follow a link-like object created after initial admission
  into a location outside the root
- Store must not perform an explicit permission-widening operation; created
  entries follow the platform's ordinary mode/umask or ACL inheritance, while
  the deployer remains responsible for configuring the service-account root

**Warnings**

- String normalization and `starts_with(root)` are not confinement.
- Canonicalizing once at admission does not prevent a later symlink, junction,
  mount, or reparse swap.
- Case folding, UNC forms, verbatim paths, trailing separators/dots/spaces, and
  Windows device namespaces need explicit classification.
- Do not solve escape risk by banning every nested directory. Later artifact
  families require principled namespace delegation.
- Display paths and logged paths are projections and may not be passed back as
  authority.

**Test requirements**

- Negative compile cases must prove that arbitrary caller strings, absolute
  paths, and unbound format roles cannot mint namespace-relative capabilities.
  Where a later artifact-name API genuinely accepts caller text, its production
  admission path must reject parent, device, UNC escape, alternate-stream,
  reserved-name, separator-smuggling, case/normalization, and invalid-component
  inputs with typed localization and zero media effects. A test-only raw-path
  parser is not acceptable evidence.
- A deterministic race test must swap a symlink/reparse/junction candidate
  between classification and open/rename/delete. The operation must deny or
  remain within the root; an outside sentinel must remain byte-identical.
- A same-name relocation test must create two roots with identical relative
  layouts and prove a capability from root A cannot operate on root B.
- A platform-access test must deny a link-like, wrong-type, or inaccessible
  root before owner installation and prove that C.4 creation performs no
  permission-widening step. Phase 8 must then exercise real create/open/lock
  behavior under the current service account. A permission change during an
  admitted operation must produce the exact before-effect or indeterminate
  outcome; Store need not infer arbitrary deployment intent from ACL or mode
  metadata.

**Engineering decisions**

- Root identity binds every namespace-relative capability. Representation-
  equal relative paths from different roots are not interchangeable.
- `NamespaceRelativePath` and admitted directory handles remain Store-owned
  capabilities. A Foundational locator, canonical identity, display path, or
  boundary-bridged identity may describe the same semantic location but may
  never satisfy a confinement API or be readmitted directly into a handle.
- Platform implementations may use different syscall sequences where their
  failure or race topology genuinely differs, but both implement the same
  confinement facts rather than pretending their mechanics are identical.
- Path parsing and namespace role validation occur before handles or buffers
  are allocated where practical.
- Phase 4 establishes confinement mechanics and a no-widening creation law.
  Phase 8 establishes the narrower observable claim that the current service
  account can perform the required real root operations and hold the OS lease;
  neither phase claims to audit arbitrary deployment ACL/mode intent. The
  sealed production admission authority remains unavailable until then.
- Phase 4 names the confinement-denial emission point and counter class. Phase
  11 installs and proves exact structural counters at that production edge;
  Phase 4 must not fake the later counter owner with local bookkeeping. Rich
  path diagnostics remain absent unless resolved diagnostic policy requests
  them.

**Open questions**

- None.

### Phase 5: Implement Concrete Filesystem Handles And Range Operations

Implement the real local-filesystem effect owner behind the contractual media
facade. Operations are range-oriented and handle-bound so later pages and WAL
do not inherit whole-file materialization or repeated pathname resolution.

**Relevant subsystems**

- `worth-store-physical-backend::filesystem_media`
- admitted root and directory handles
- operation context and counters
- platform-specific file mechanics

**Relevant APIs**

- `FilesystemMediaOwner`
- `NamespaceFileHandle`
- `NamespaceDirectoryHandle`
- `PositionedReadRequest`
- `PositionedWriteRequest`
- `AppendRequest`
- `TruncateRequest`
- `AllocationRequest`
- `MediaMetadata`
- `NamespaceEntryBatch`

**Required mechanics**

- open existing and create-new have distinct requests and outcomes; neither
  silently changes into the other
- positioned reads and writes preserve caller offset and return exact transfer
  width without mutating an ambient cursor
- append obtains and reports the exact append range assigned by the backend
  under the admitted process ownership model
- explicit whole-transfer helpers may compose primitive attempts but must
  remain bounded, cancellable at attempt boundaries, and fully counted
- truncate and allocation distinguish logical length, allocated range, sparse
  posture, and platform support
- listing is paged/bounded and returns only immediate admitted-directory
  entries; recursive whole-namespace enumeration is reconstructive work
- metadata exposes only stable required fields plus explicitly platform-scoped
  observations
- handles are owned by the filesystem owner and borrowed or leased internally;
  they are not publicly cloneable or detachable from runtime lifecycle

**Warnings**

- Do not implement append as metadata-length lookup followed by an unrelated
  positioned write unless the mutation ownership and platform sequence make
  the allocation atomic and the receipt preserves the actual range.
- Do not read an entire file to satisfy a range request or list an entire tree
  to answer one directory operation.
- Do not let high-level artifact code call `std::fs`, platform APIs, or raw
  handle traits around this owner.
- Do not make platform handles serializable, replayable, or reconstructable
  from integers.
- Do not hide allocation inside convenience methods; allocation bytes and
  lifecycle scope are part of the request and counters.

**Test requirements**

- Real-filesystem owner tests must cover unaligned small ranges, page-sized
  ranges, large bounded ranges, EOF crossings, empty files, sparse candidates,
  append contention inside one owner, truncate growth/shrink, bounded listing,
  and handle invalidation after close.
- A scale-slope test must operate on a file far larger than the test buffer and
  prove memory and copied bytes scale with requested ranges rather than file
  length.
- Primitive classification tests must prove partial transfer width and the
  subsequent continuation offset exactly, while owner-bound handle lifetime
  tests prove stale and cross-owner handles cannot reach a new OS effect. Phase
  11 repeats the transfer proof through the real fault interposer.

**Engineering decisions**

- Ordinary operation buffers are caller- or owner-lifecycle-scoped. The media
  owner does not allocate an unbounded `Vec` proportional to file or namespace
  size.
- Primitive transfer outcomes remain Store-owned execution facts. They are not
  wrapped as `worth-proof` phase progressions and do not materialize
  Foundational receipts, diagnostics, canonical artifacts, or performance
  reports on the ordinary I/O path.
- Handle identity includes runtime incarnation and owner-local generation so a
  recycled OS handle value cannot revive stale authority.
- Read-only handle borrowing and mutation operations remain distinguishable;
  the shared mutation lease is not duplicated with every file handle.
- Synchronous operations expose the coordination boundary explicitly. C.10 may
  schedule calls around them without changing their semantics.

**Open questions**

- None.

### Phase 6: Implement Synchronization And Durable Publication Protocols

Implement distinct synchronization facts and the C.4-owned durable namespace
creation/replacement/delete protocols. No method named merely `flush` may hide
which persistence boundary was established.

**Relevant subsystems**

- filesystem synchronization mechanics
- namespace identity creation
- staged file publication
- directory and parent-directory durability
- typed indeterminate effects

**Relevant APIs**

- `FileDataSynchronization`
- `FileStateSynchronization`
- `DirectoryPublicationSynchronization`
- `StagedNamespaceFile`
- `NamespacePublicationTarget`
- `CompletedAtomicReplacement`
- `DurablyPublishedNamespaceFile`
- `DurableDeletion`
- `IndeterminateNamespacePublication`

**Required protocols**

Namespace identity creation progresses through named facts:

```text
validated absent/empty root
  -> created root and fixed directories
  -> created-new staged identity file
  -> completed framed identity write
  -> synchronized identity file state
  -> atomically renamed staged identity to namespace/identity
  -> synchronized namespace directory publication
  -> synchronized store-root scaffold publication
  -> synchronized parent publication when root creation requires it
  -> initialized namespace
```

- Each transition consumes the prior fact and produces the next; skipped or
  reordered transitions are uncallable in the owner implementation.
- Same-directory replacement uses the backend's admitted atomic replacement
  mechanism and then the required directory barrier.
- The OS lock handle, admitted mutation fact, and owner-local namespace
  sequence remain held from immediate source-identity revalidation through
  rename/delete and the required containing-directory barrier. Under the
  admitted coordinated-writer posture, no second supported writer can enter
  between those steps.
- Durable deletion distinguishes entry removal from durably published removal.
- A barrier failure after a visible rename/delete produces an indeterminate
  outcome, never rollback fiction.
- Directory synchronization is a first-class backend operation. If a profile
  cannot establish the required namespace guarantee, the profile is denied or
  admitted only to a weaker explicitly named non-production tier that cannot
  construct `MediaOwnedPhysicalRuntime`.

**Warnings**

- `File::flush`, buffered-writer flush, write completion, file sync, and
  directory sync are not synonyms.
- Atomic visibility and power-loss durability are different facts.
- Do not report durable rename after file synchronization alone.
- Do not attempt to reverse a possibly visible rename and then claim the first
  operation never happened.
- Do not make a staged filename the canonical identity of the published file.
- Do not describe immediate name revalidation as protection against an
  unmanaged same-credential OS actor. That actor is outside the admitted trust
  model and its interference is tampering, not a supported concurrent writer.

**Test requirements**

- Production transition facts and real-filesystem observation must assert the
  exact create/write/file-sync/rename/namespace-directory-sync/store-root-sync/
  root-parent-sync sequence for namespace initialization, replacement, and
  deletion. Phase 11's recording interposer adds exact zero/nonzero structural
  counters without replacing these real effects.
- Phase-local failure classification tests cover every deterministic,
  naturally inducible transition failure in the following table. Phase 11
  injects only failures that the host filesystem cannot induce reliably on
  demand, and Phase 15 requires the fresh-observer path/byte match for every
  injected outcome.

| Evidence owner | Required failure pressure |
| --- | --- |
| Phase 6 real filesystem | create-new collision, absent or substituted staged source, foreign-owner publication target, wrong-type replacement target, absent or substituted deletion target, and synchronization denied after ownership invalidation |
| Phase 11 real-backend interposer | file-sync failure, namespace-directory-sync failure, store-root-sync failure, root-parent-sync failure, post-call completion ambiguity, and a pause on each named causal edge |
| Phase 15 fresh observer | allowed path/byte states and typed retry posture for every Phase 11 directive, including omitted-barrier fault cases |

  A Phase 6 unit fixture must not fake a successful or failed synchronization
  syscall. The production durability claim closes only after the Phase 11 and
  Phase 15 rows pass against the real backend.
- Reordering rename before complete write or before required file sync must
  fail typestate/compiler proof or the sequence assertion at the causal edge.

**Engineering decisions**

- The production claim is limited to the admitted backend profile and recorded
  assumptions. C.4 process-death tests do not overclaim sudden-power-loss
  behavior; hardware qualification owns that stronger evidence.
- Publication typestate is a private Store protocol because each state names a
  concrete media effect already owned by the backend. It does not become a
  generic `worth-proof` recipe, and no proof witness may substitute for an
  omitted file or directory barrier.
- Publication protocols emit one immutable operation summary derived from the
  actual transition chain. Diagnostics and evidence derive from it; any later
  Foundational lowering is an explicit support boundary and cannot feed back
  into publication, retry, cleanup, or authority decisions.
- No barrier is retried invisibly after an indeterminate outcome.
- Parent-directory synchronization is required when creating a previously
  absent root only where the backend profile and deployment contract make that
  boundary meaningful and qualifiable; the admitted profile records the exact
  law.

**Open questions**

- None.

### Phase 7: Establish Process-Level Mutation Ownership

Make one live OS-enforced lease the only process-level mutation authority for a
store root. Lock-file bytes remain diagnostic; acquiring the actual operating-
system lock is the proving action.

**Relevant subsystems**

- namespace mutation lock target
- filesystem handles and platform locking
- process and runtime identity
- admission contention and shutdown

**Relevant APIs**

- `MutationOwnershipAttempt`
- `MutationOwnershipLease`
- `MutationOwnerObservation`
- `MutationOwnershipDenial`
- `OwnershipReleaseOutcome`

**Ownership protocol**

- absent/empty-root contenders may idempotently establish only the fixed C.4
  directory and lock-target scaffold before ownership is decided
- each contender opens the canonical lock target without following links and
  attempts the platform's exclusive non-inherited process lock
- exactly one successful lock attempt receives the move-only lease
- only the lease holder may create/publish identity, run qualification,
  reconcile owned staging residue, or perform mutation operations
- owner metadata written to the lock target includes process/runtime/attempt
  observation but is never consulted as permission
- handles are non-inheritable or close-on-exec by default; a spawned child does
  not accidentally extend the parent's ownership lifetime
- explicit close releases resources in declared order; abort/unexpected drop
  release the OS lease without reporting a successful normal close
- abrupt process termination relies on OS handle closure, allowing a fresh
  process to contend and obtain a new lease while preserving stable store
  identity

**Warnings**

- `create_new(mutation.lock)` alone is not a live lock and leaves stale-file
  ambiguity after death.
- PID liveness checks are diagnostic heuristics and must not grant ownership.
- Do not delete the persistent lock target on close; deletion races and open-
  handle differences would make file existence a false authority signal.
- Do not allow a second process into a “read-only runtime” before C.10 defines
  stable physical read leases. An offline OS observer may inspect but cannot
  claim a stable concurrent view.
- Fork/exec and Windows handle-inheritance behavior must be explicit in the
  backend profile and tests.

**Test requirements**

- A multi-process start barrier must release at least eight contenders against
  one absent root. Exactly one process obtains ownership and all others receive
  typed contention with zero post-lease identity publication, qualification,
  staging, family, or mutation effects. Any contender may perform only the
  explicitly classified idempotent fixed-scaffold creation needed to reach the
  canonical OS lock, and those attempts must reconcile exactly.
- Killing the winning process without cleanup must allow exactly one fresh
  contender to acquire a new lease and discover the same stable store identity
  with a new process, runtime, and ownership-attempt identity.
- A stale owner-metadata case must prove that plausible live PID/timestamp
  bytes neither block a free OS lock nor grant a contended one.
- A handle-inheritance test must spawn an unrelated child, terminate the owner,
  and prove the child does not keep the mutation lease alive.

**Engineering decisions**

- Single-process mutation ownership is the C.4 production policy even if an
  underlying filesystem could support more. Branch-level multi-writer
  semantics belong to Part II above physical transaction law.
- The successful OS lock and its still-live non-inherited handle are the lease
  authority. A `worth-proof` witness may describe legal admission progression,
  and Foundational support vocabulary may describe an observation after the
  fact, but neither can mint, restore, serialize, or prolong the lease.
- Contention is a normal typed denial, not corruption and not automatic retry.
- Phase 7 exposes exact acquisition, contention, invalidation, release, and
  drop lifecycle observations. Phase 11 installs and proves their structural
  counters at these production edges; Phase 7 must not create a competing
  local counter authority.
- Losing or invalidating the live lock places the runtime into an inert
  ownership-lost outcome; it cannot continue writes on optimistic assumptions.
- Lease lifetime is established by the qualified platform contract: the
  private lock handle retains the lock until explicit unlock, handle close, or
  process death. There is no polling query whose answer can replace that fact.
  Every mutation atomically admits against the live lease state. Backend-
  observed invalidation closes all later admission immediately; an operation
  that already received authority is ordered before that invalidation. Fault
  injection may invalidate only at named operation boundaries, not pretend an
  OS call can be interrupted safely. A filesystem whose lock may be silently
  revoked while the handle remains live is not a supported production profile.

**Open questions**

- None.

### Phase 8: Admit Root-Specific Backend Capabilities

Admit a concrete backend profile from observed mechanics on the declared root,
not from configuration optimism or operating-system labels. Qualification is
separate from ordinary startup: production admission classifies the current
root/profile and consumes capability law already established for the concrete
backend, while certification and hardware qualification execute destructive
probes. Later operations consume carried capability handles without repeating
either work.

**Relevant subsystems**

- filesystem backend profile detection
- existing `worth-store-physical-backend::io_capability` proof-backed lane
- `worth-proof` assumption-basis, freshness, witness, and checked-outcome
  progression
- fixed namespace/staging roles
- platform capability probes
- hardware qualification evidence
- media admission denial

**Relevant APIs**

- `FilesystemQualificationRequest`
- `FilesystemBackendProfile`
- `RootProfileQualificationBasis`
- `MediaCapabilityQualificationOutcome`
- existing `BackendCapabilityClaimOutcome`
- existing `BackendCapabilityClaimWitness`
- `QualifiedBaseMediaCapabilities`
- `QualifiedDataSyncCapability`
- `QualifiedSparseAllocationCapability`
- `QualifiedPreallocationCapability`
- `QualifiedMmapCapability`
- `QualifiedDirectIoCapability`
- `MediaQualificationDenial`

**Qualification progression**

- derive stable platform observations from the opened root and filesystem,
  including volume/device identity, local/remote classification, relevant
  filesystem type, alignment/granularity, and available synchronization forms
- deny unqualified remote, removable, userspace, or unknown profiles before
  production-phase construction
- bind the concrete backend implementation and current root/profile identity to
  the separately versioned support/qualification contract
- represent that binding as one Store-owned `RootProfileQualificationBasis`
  carried through the `worth-proof` assumption-basis/freshness lane; a copied
  profile description or report is not the basis
- require the live `MutationOwnershipLease`; on new namespaces, the actual
  identity-publication protocol establishes the base create/write/file-sync/
  replacement/directory-sync facts needed by that admission
- bind the caller-declared coordinated-service-account deployment contract to
  the observed root/profile and live lease; qualify that the current account
  can use the root without a permission-widening step, that Store-created
  entries use the platform's normal inheritance path, and that the platform
  retains an exclusive lock until explicit unlock, handle close, or process
  death
- on existing namespaces, validate current profile identity and namespace law
  without performing a destructive self-test or rewriting stable files
- in certification and deployment qualification only, run a uniquely named
  bounded transaction under `staging/` that exercises create-new, positioned
  transfer, append, metadata, bounded listing, truncate/allocation posture,
  file synchronization, same-directory replacement, directory synchronization,
  and durable cleanup through the production backend
- compare certification transaction paths and bytes through an independent OS
  reader before accepting the corresponding qualification evidence
- emit a checked proof outcome for each required capability claim; denial,
  deferment, stale evidence, rebind-required basis drift, and hard probe failure
  remain distinct
- issue optional capability handles only by consuming the corresponding
  successful claim with the live root/profile basis and mutation lease; record
  their exact constraints
- remove and durably publish removal of certification qualification residue
  before that qualification lane succeeds

**Warnings**

- A successful syscall on one temporary path does not prove every hardware
  durability claim. Capability scope and assumptions must remain explicit.
- Do not cache a capability report across root/device/profile identity changes.
- Do not trust a checked-in table saying that all Windows or POSIX filesystems
  behave alike.
- Do not claim arbitrary-hostile-process TOCTOU resistance from an advisory OS
  lock or from a separate file-identity check. The qualified deployment access
  posture is part of the production contract.
- Do not make a previous qualification report authority. Reports can inform
  diagnostics and satisfy a bound deployment policy; current admission must
  still match its concrete backend and current root/profile identity.
- Do not run optional direct-I/O or mmap mechanics on ordinary operations
  merely to keep their capability status fresh.
- Do not turn every ordinary open into a destructive backend self-test. That
  adds startup latency, flash wear, and new failure surface without improving
  authority.
- Do not preserve current proof when root, volume/device, backend build,
  qualification-contract version, relevant mount posture, or declared
  capability assumptions change. Those changes must produce stale or
  rebind-required posture as specified by the owning basis.
- Do not let a `worth-proof::AuthorityWitness`, successful claim payload, or
  Foundational report become an operational capability by itself. Only the
  backend owner can combine a current claim with its live OS-owned inputs.
- Do not build a second media qualification registry beside
  `BackendCapabilityClaimOutcome`; migrate the existing lane in the same slice
  when its current vocabulary is insufficient.

**Test requirements**

- A real-root qualification test must prove the base capability sequence
  executes through the production backend in the qualification lane while the
  live mutation lease is held, leaves no scratch residue, and prevents
  production-profile admission when any required behavior is unqualified.
- An existing-root admission test must prove ordinary reopen performs no
  destructive qualification transaction and does not rewrite stable namespace
  files while still rejecting a mismatched current profile.
- Access-posture qualification must exercise actual create/open/lock behavior
  and deny when the current service account cannot use the root without Store
  widening permissions. The report records the declared cooperation contract
  separately from observed platform facts and does not pretend an ACL can
  prove that every same-credential process will honor an advisory lock.
- A profile-substitution test must replay a report from a different root,
  volume/device identity, or backend profile and prove it opens no capability
  or runtime door.
- A basis-drift test must change root identity, volume/device identity,
  qualification-contract version, and backend profile independently and prove
  each previously successful claim becomes the specified stale or
  rebind-required outcome before any optional operation is callable.
- An optional-capability matrix must prove absence is honest and cheap, while
  each present handle carries the actual alignment/granularity constraints and
  rejects a violating request before media effects.
- Capability tests must prove that a supported profile is emitted only after
  the real qualification transaction and its operation/scratch observations.
- A compile-time authority specimen must reject an unrelated proof witness, a
  successful claim for another capability, a bridged/stale claim, a
  Foundational canonical artifact, and a serialized qualification report as
  inputs to concrete capability-handle construction.
- A migration gate must fail while both the old proof-backed claim lane and a
  second production media-qualification authority can independently admit the
  same capability.

**Engineering decisions**

- Base capability admission is mandatory for every production admission;
  destructive capability qualification is not. Expensive destructive/power-
  loss hardware qualification remains a separately bound deployment artifact
  and cannot be inferred from a process-local transaction.
- Qualification uses the real root so mount/filesystem behavior cannot be
  substituted by the system temp directory.
- The bounded qualification transaction has a fixed byte and operation budget
  recorded in counters.
- Optional capability absence does not make the base media owner optional.
- `worth-proof` owns the checked progression topology. Store owns the concrete
  qualification basis, authority markers, denial/defer/stale/rebind/failure
  payloads, and capability handles.
- Successful claim proof is carried inside the same trust boundary and is not
  rediscovered on ordinary operations. A genuine basis change forces Store-
  owned revalidation; it is not hidden behind a cache refresh.
- Qualification reports lower into Foundational support or performance
  evidence only after the operational decision. That lowering is optional
  policy-governed work and cannot affect the admitted capability set.

**Open questions**

- None.

### Phase 9: Install The Consuming Media-Owned Runtime Transition

Only after the namespace contract, real backend, durability protocols,
capability qualification, and OS ownership lease exist may C.4 add the consuming
runtime transition and media-owned phase.

**Relevant subsystems**

- C.3 runtime composition root
- filesystem namespace admission
- physical backend owner installation
- runtime capability observation
- admission failure recovery
- `worth-proof` checked phase progression

**Relevant APIs**

- `AdmittedPhysicalRuntime::try_admit_filesystem_media`
- `FilesystemMediaAdmission`
- `MediaAdmissionOutcome`
- `MediaAdmissionDenial`
- `MediaAdmissionDeferred`
- `MediaAdmissionStale`
- `MediaAdmissionRebindRequired`
- `MediaAdmissionInspectionRequired`
- `MediaOwnedPhysicalRuntime`
- `PhysicalMediaObserver`

**Transition progression**

```text
AdmittedPhysicalRuntime
  -> validated C.4 configuration and retained C.3 root authority
  -> admitted/confined root handles
  -> acquired mutation ownership
  -> classified or initialized stable namespace
  -> qualified required backend capabilities
  -> installed concrete FilesystemMediaOwner
  -> MediaOwnedPhysicalRuntime
```

- the transition consumes `AdmittedPhysicalRuntime`; no second call can race
  the same authority
- `MediaAdmissionOutcome` is the Store-specific `worth-proof::ProofOutcome`
  alias whose success payload is `MediaOwnedPhysicalRuntime`; the Store-owned
  non-success payloads preserve the fate of the consumed C.3 authority
- pre-effect and fully reconciled denials return the original C.3 runtime plus
  typed report for explicit retry or close
- compatible contention or temporarily unavailable environmental readiness is
  `Deferred`, not corruption or generic denial, and returns prior authority
  only when zero effect is established
- basis invalidation is `Stale` or `RebindRequired` according to whether fresh
  observation or explicit Store revalidation is required; neither posture can
  promote from a report, digest, or bridged identity
- a denial after ambiguous or indeterminate publication returns an inert
  `Failed(MediaAdmissionInspectionRequired)` terminal object, never reusable
  admission authority
- `MediaOwnedPhysicalRuntime` contains the original exhaustive runtime core and
  exactly one concrete `FilesystemMediaOwner`
- construction remains private to the transition; fields and internal owner
  accessors are not public
- `AdmittedPhysicalRuntime` gains no physical methods and cannot be upgraded by
  a capability report, lock record, store identity, backend owner, or observer
- the transition consumes concrete qualified capability handles and the live
  filesystem owner; it does not accept a generic proof set or caller-supplied
  authority witness
- existing C.3 observers become phase-stale after progression; a newly acquired
  media observer can observe stable store identity, admitted backend profile,
  capability status, ownership identity, lifecycle, and counters without
  mutating them

**Warnings**

- Do not represent owner installation as `Option<FilesystemMediaOwner>` on the
  C.3 type or as an enum flag checked by every operation.
- Do not let failed admission drop the C.3 runtime silently; its authority fate
  must be explicit.
- Do not create a public constructor for testing, deserialization, replay, or
  recovery.
- Do not let feature unification substitute a memory/certification owner for
  the concrete filesystem owner.
- Do not expose raw file operations on the product facade to prove C.4.
- Do not wrap `AdmittedPhysicalRuntime` or `MediaOwnedPhysicalRuntime` in a
  generic recipe/artifact carrier. The Store runtime types remain the owning
  phase forms; `worth-proof` governs their checked transition outcome.
- Do not collapse `Deferred`, `Stale`, `RebindRequired`, and terminal
  indeterminate failure into one `MediaAdmissionDenial` for API convenience.

**Test requirements**

- Compiler UI specimens must prove the media phase cannot be constructed,
  cloned, reconstructed, field-extracted, forged from capability/identity/
  lock observations, or obtained from a non-filesystem backend.
- Runtime tests must prove a definite pre-effect denial returns the same C.3
  runtime identity, while an indeterminate publication consumes it into an
  inert outcome and leaves no callable admission or mutation surface.
- One table-driven transition test must force success, denial, deferment,
  staleness, rebind-required, and inspection-required failure from real or
  boundary-faithful inputs and prove the exact prior-authority fate and effect
  counter posture for every category.
- A transition audit must assert every construction, close, abort, panic,
  observation, and unexpected-drop site handles the new owner exhaustively.
- An all-features compiler specimen must prove certification feature activation
  alone cannot construct or inject `MediaOwnedPhysicalRuntime`.
- The authority specimen must prove a raw `TransitionOutcome`, unrelated
  `AuthorityWitness`, qualification report, Foundational identity boundary,
  canonical digest, or copied successful claim cannot call private runtime-
  phase construction.

**Engineering decisions**

- Runtime incarnation identity survives successful C.3-to-C.4 phase
  progression; stable store identity is newly discovered and remains a
  separate identity category rather than operational authority.
- `worth-proof` carries transition stage and non-success topology but never
  owns, clones, serializes, or reconstructs the runtime payload.
- C.4 defines concrete private marker/authority types only where a proof
  transition genuinely spends authority. It exposes no generic
  `AuthorityMarker` bound and no public witness constructor.
- A C.3 observer does not silently widen into media observation. Phase-specific
  observation must be reacquired from the new runtime.
- The media-owned phase remains move-only and has no public subsystem bag.
- The later C.5 transition will consume this exact concrete phase rather than
  add page owners beside it optionally.

**Open questions**

- None.

### Phase 10: Propagate Observation, Close, Abort, And Crash Lifecycle

Extend C.3 lifecycle law across real handles and mutation ownership without
turning resource release into a false durability acknowledgment. Every terminal
path has one explicit owner fate and one observable result.

**Relevant subsystems**

- C.3 lifecycle and process-local root registry
- filesystem handle registry
- mutation ownership lease
- media observation handles
- panic/unexpected-drop accounting

**Relevant APIs**

- `MediaOwnedPhysicalRuntime::observer`
- `MediaOwnedPhysicalRuntime::close`
- `MediaOwnedPhysicalRuntime::abort`
- `ClosedRuntime`
- `AbortedRuntime`
- `PhysicalMediaObservation`
- `MediaShutdownOutcome`

**Lifecycle rules**

- normal close consumes the media runtime, prevents new internal operations,
  completes only already-required C.4 owner shutdown, closes owner handles,
  releases the OS mutation lease, releases the C.3 process-local root
  admission, and publishes terminal observation
- close does not invent durability for an operation whose protocol never
  reached its required barrier
- abort consumes the runtime, stops new work, abandons only owner-proven
  uncommitted staging state according to explicit policy, releases resources,
  and reports abort distinctly from close
- unexpected drop and panic unwind close OS handles and release the live lock
  through RAII, increment their own terminal counters, preserve ambiguous
  residue, and never report normal close
- no backend mutation may begin after ownership release starts
- stale observers remain immutable and generation-aware; an observer from a
  prior runtime incarnation cannot become current when the same store is
  re-admitted
- shutdown order is exhaustive over every installed handle and resource class
  so adding a new C.4 owner field breaks incomplete lifecycle propagation

**Warnings**

- Do not call broad file or directory synchronization from `Drop` and then
  imply orderly durability.
- Do not use `Arc<FilesystemMediaOwner>` or cloneable lock guards to make
  shutdown easier; that makes the authority release time unknowable.
- Do not erase shutdown failures into logs. Resource release and publication
  failures retain typed inspection posture.
- Do not delete staged residue during panic/drop unless non-publication is
  mechanically proven.
- Do not release the live OS lease and then invoke media operations during
  bookkeeping.

**Test requirements**

- An exact lifecycle test must cover close, abort, panic unwind, unexpected
  drop, and process death, reconciling handle opens/closes, ownership acquire/
  release, process-root registry state, terminal observations, and residue.
- A deterministic shutdown race must pause immediately before lease release,
  contend from a second process, and prove no new mutation begins after release
  starts and no old-runtime operation occurs after the contender acquires.
- Unexpected drop must remain distinct from close, and the shutdown-race test
  must directly prove that no backend call occurs after lease release.

**Engineering decisions**

- Explicit close and abort may return typed shutdown outcomes. Terminal handles
  are inert even when cleanup reports a failure.
- OS-handle cleanup remains best-effort at language drop boundaries, but its
  uncertainty is observable and cannot recreate authority.
- Re-admission always creates new runtime, media-owner, handle, operation, and
  lease identities while preserving the decoded stable store identity.
- The C.3 root-registry terminal linearization remains canonical for
  process-local admission; the OS lease is released as a named step within the
  expanded terminal sequence.

**Open questions**

- None.

### Phase 11: Install Fault Interposition And Exact Structural Accounting

Make every production media effect observable and deterministically faultable
at its true boundary while keeping diagnostics, certification, and counters
outside authority. The interposer decorates the real backend; it never replaces
it with a memory implementation.

**Relevant subsystems**

- media operation context
- production filesystem backend facade
- certification fault schedules
- exact structural counters
- diagnostic materialization policy
- `worth-foundational` boundary-evidence and performance lowerings

**Relevant APIs**

- `MediaOperationContext`
- `MediaOperationRole`
- `MediaFaultSchedule`
- `MediaFaultDirective`
- `MediaFaultInterposer`
- `MediaOperationCounters`
- `MediaCounterSnapshot`
- `MediaOperationSummary`
- `FoundationalCounterBackedPerformanceReceipt` as an explicit support-boundary
  output, never an ordinary operation result

**Operation context**

Every boundary call carries or derives:

- runtime incarnation identity
- stable store identity after it becomes available
- media-owner and operation identity
- namespace role and handle identity
- operation family and primitive-attempt ordinal
- requested offset/range/length where applicable
- publication stage and required capability identity
- diagnostic policy and certification yieldpoint identity without granting
  either mutation authority

**Fault directives**

- fail before effect with an exact typed OS/backend cause
- allow an exact prefix and then return a partial transfer
- execute the underlying effect and then return an indeterminate observation
- fail a file or directory barrier
- pause before or after a named production boundary for process contention or
  termination
- interrupt atomic-replacement observation without editing private state or
  manufacturing destination bytes

**Required counters**

- admission attempts, qualification transactions, ownership attempts,
  acquisitions, contentions, and releases
- file/directory opens, creates, closes, and live-handle peaks
- primitive and whole positioned reads/writes, append attempts, requested and
  completed bytes, short transfers, EOF observations, and retries
- truncate, allocation, metadata, listing entry/batch, rename, deletion, file-
  sync, directory-sync, and parent-publication operations
- confinement denials, stale-handle denials, unsupported capabilities,
  before-effect failures, partial effects, indeterminate effects, cleanup
  actions, and preserved residue
- hot-path allocations and peak operation-scoped bytes at the named media
  boundary

**Warnings**

- A counter increment is not permission and a receipt is not authority.
- Do not use elapsed time, sleeps, logs, or random fault selection as
  correctness proof.
- Do not let fault schedules match mutable private addresses or source-line
  numbers; match stable semantic operation identities and ordinals.
- Do not materialize rich traces on the ordinary path when the diagnostic
  policy requests structural counters only.
- Do not count requested bytes as completed bytes.
- Do not let a Foundational claim, policy-admission receipt, or materialized
  report say execution happened. Only a Store operation outcome plus exact
  causal counters may lower into counter-backed execution evidence.

**Test requirements**

- A parity test must run one identical qualification/publication program with
  no interposer and with a pass-through interposer, proving bitwise-equal files,
  identical semantic outcomes, and identical structural counters except for
  explicitly interposer-owned observations.
- A fault-schedule determinism test must repeat the same source/binary/profile/
  seed schedule and produce the same operation match, prefix effect, typed
  outcome, path set, and counter snapshot.
- Counter law tests must reconcile every attempted primitive to exactly one
  terminal classification and prove requested/completed byte conservation.
- A rich-diagnostics-off test must prove ordinary operation results and files
  remain identical while rich artifact allocations and trace entries stay
  exactly zero.
- A support-lowering parity test must lower one completed Store operation and
  its exact counter rows into Foundational counter-backed evidence, then prove
  that missing, duplicated, unexpected, or mismatched rows fail closed. A
  claim-only or policy-admission artifact must remain unable to satisfy this
  executed-evidence boundary.
- A hot-path isolation test must prove disabling support materialization removes
  every Foundational report/evidence allocation and canonicalization step while
  leaving OS effects, Store outcomes, and causal counters identical.

**Engineering decisions**

- The certification fault schedule is a concrete sealed value minted by the
  certification owner. It can decorate only an admission already committed to
  the real filesystem implementation.
- Counters live at the causal boundary and are aggregated outward; separate
  layers do not independently guess the same effect count.
- Each operation summary is immutable and self-describing enough for evidence
  without querying mutable backend internals.
- Store operation summaries and counter snapshots remain the stronger local
  sources. Foundational boundary evidence is materialized only by an explicit
  support/certification adapter after execution and cannot feed back into
  operation, retry, qualification, or runtime-admission decisions.
- Counter widths saturate or fail explicitly according to a declared policy;
  silent wrap is forbidden.

**Open questions**

- None.

### Phase 12: Eliminate Physical Writer Bypasses And Seal The Public Surface

Migrate or quarantine existing writer islands, prove the canonical runtime is
the only ordinary composition path, and keep raw filesystem authority out of
the product facade before acceptance journeys grant C.4 closure.

**Relevant subsystems**

- every current physical writer reachable from the scoped call paths
- Worth Store crate dependency graph
- product and certification facades
- consolidated compiler UI suite
- source/dependency boundary checker

**Relevant APIs**

- `PhysicalStore`
- `AdmittedPhysicalRuntime`
- `MediaOwnedPhysicalRuntime`
- internal physical-backend facade
- certification-only media decoration surface

**Required cutover**

- migrate reusable direct file mechanisms behind the canonical media owner
  when their semantics match the C.4 contract
- delete duplicate implementations whose only purpose is superseded
- quarantine retained isolated mechanisms so no ordinary runtime, readiness,
  operation, or product facade can call them or promote their result
- remove public replay, heap-layout, raw-handle, raw-path, memory-backend, and
  test-construction routes that could impersonate media admission
- keep offline OS observation and physical-format decoding read-only and
  structurally separate from runtime construction
- resolve every affected writer in source and verify the resulting route with
  direct owner or boundary tests; no inventory artifact is required
- install mechanical dependency/source gates for direct filesystem effects
  outside the backend boundary, with narrow named allowances only for
  configuration/root declaration and independent certification observers

**Warnings**

- Do not “migrate” a writer by wrapping it while leaving the old public call
  path intact.
- Do not ban harmless filesystem reads across the entire repository with a
  fragile string rule. Gate authority-bearing effects at admitted crate and
  module boundaries, and manually resolve exceptional observations.
- Do not let the offline observer import runtime recovery or media-owner
  internals.
- Do not expose `FilesystemMediaOwner` through `Deref`, generic subsystem
  lookup, debugging accessors, or test support.
- Do not issue physical-platform readiness; C.13 remains the only S.10 handoff.

**Test requirements**

- A dependency/source gate must reject a deliberately added ordinary
  `std::fs::write`, raw platform writer, memory backend, certification writer,
  or direct backend constructor outside the admitted boundary and localize the
  owning rule.
- Compiler UI must prove an external consumer cannot access media internals,
  construct the phase, inject a backend, invoke raw operations, promote an OS
  observer, or reuse C.4 authority after close/abort.
- A reverse reachability review must trace every surviving OS writer to either
  the canonical media owner or a named non-production/offline role and must
  fail closure on any unclassified writer.
- A maximal-feature build must preserve the same public authority denials.
- The Store runner dispatches the named C.4 owner, journey, observer, and UI
  tests directly and fails when a selected product executes no tests.

**Engineering decisions**

- Mechanical gates protect stable crate/module boundaries; human review of the
  current causal paths protects semantic interpretation. Neither is inflated
  into a generated proof ledger.
- Independent observer allowances are read-only, named, and incapable of
  constructing runtime or mutation types.
- Existing strong backend mechanisms are reused only when their failure and
  durability semantics can implement this spec without adapters that lie.
- C.4 closeout requires no ordinary physical writer bypass, even if that
  expands implementation across neighboring Store crates.

**Open questions**

- None.

### Phase 13: Prove Namespace Creation And Fresh-Process Discovery

Build the first joined journey through the public production facade. It proves
that namespace admission creates real artifacts through the admitted backend,
that successful close leaves no live authority, and that a fresh executable
discovers the same persistent identity from bytes alone.

**Relevant subsystems**

- public physical runtime facade
- namespace initialization and capability qualification
- real filesystem backend
- mutation ownership lifecycle
- independent OS/format observer
- Foundational canonical comparison of independently produced namespace meaning

**Named executable and roles**

- Cargo integration-test executable: `physical_media_journeys`
- named test: `namespace_creation_and_fresh_process_discovery`
- child role `namespace-writer`: uses only the public production facade and
  receives root plus production configuration
- child role `namespace-reopener`: uses only root plus production configuration
  and performs a new C.3/C.4 admission
- separate binary executable `physical_media_os_observer`: uses ordinary OS
  reads plus duplicated stable namespace constants/grammar and an
  independently implemented minimal decoder; it does not call the production
  decoder or construct a runtime

**Initial world**

- root A is absent
- root B exists and is truly empty
- the backend profile, namespace version, source/binary identity, page-neutral
  qualification byte pattern, and deterministic schedule are recorded
- no identity, lock target, staging file, family directory, cached report,
  replay artifact, or supplied store identity exists

**Execution**

- run the writer against root A and root B independently
- allow media admission to acquire ownership, create/initialize the namespace,
  execute the real bounded certification qualification transaction once,
  clean it durably, produce media observation, and close normally
- discard writer process state completely
- after root A closes, rename its entire root within the same qualified
  filesystem and use only the relocated path for subsequent observation and
  reopening
- run the OS observer and then the ordinary non-destructive reopener in distinct
  fresh processes
- treat the observer's decoded identity as an external boundary value and
  compare its independently produced namespace basis with the runtime-produced
  Store basis through Foundational canonical comparison; do not promote the
  observer value into current Store identity
- close the reopened runtime and inspect again

**Assertions**

- both initial worlds converge to the exact canonical C.4 namespace path set
- identity bytes pass independent framing/integrity checks and yield the same
  stable identity to writer-time observation, relocated-root observer, and
  reopener
- runtime, observer, and reopener namespace bases compare canonically equivalent
  under the declared basis while retaining distinct producer shapes; changing
  version, encoding, identity, or publication posture produces the matching
  structured canonical mismatch
- writer and reopener have different process, runtime, media-owner, handle,
  operation, and mutation-lease identities
- `families/` and `staging/` are empty; no qualification or temporary residue
  remains; `mutation.lock` persists but is not locked after close
- no path exists outside either declared root and no writer-returned byte buffer
  is used as expected truth
- exact initialization, qualification, transfer, barrier, rename, cleanup,
  ownership, handle, allocation, and terminal counters reconcile with the
  recorded operation trace; all page/WAL/checkpoint/recovery counters remain
  structurally absent

**Warnings**

- Do not let the writer serialize its `StableStoreIdentity` into the observer's
  expected input. Expected identity is “the one valid identity independently
  decoded from this root,” not a writer-supplied value.
- Do not use graceful same-process reopen or a temp-directory object's retained
  handle as fresh-process evidence.
- Do not make the test create expected namespace files.
- Do not let canonical equivalence replace direct byte, path, barrier, or
  process-independence assertions. It proves shared boundary meaning only.

**Test requirements**

- The full journey above must run on the declared Windows development profile
  and supported POSIX CI profile.
- Re-running the observer and reopener without mutation must yield identical
  namespace bytes and stable identity with new ephemeral identities and zero
  initialization writes.
- The omitted identity-file-barrier fault case must fail the sequence assertion
  even when observer bytes happen to look correct.
- Altering one observer semantic field must fail at that named canonical locus
  without granting the observer value Store identity or runtime authority.

**Engineering decisions**

- The integration-test binary is process orchestration, not production
  authority. Writer and reopener call only the public runtime facade.
- The independent observer may share namespace byte declarations from physical
  format but not admission, capability qualification, locking, cleanup, or
  runtime classification code.
- The observer does not import runtime authority or mint Foundational current-
  authority identity. Canonical comparison occurs after its independent parse
  and remains descriptive unless Store separately revalidates persisted source
  truth through ordinary admission.
- Both absent and existing-empty roots are covered so callers never need an
  unsafe check-then-create API.

**Open questions**

- None.

### Phase 14: Prove Contention, Confinement, Death, And Re-Admission

Build the hostile ownership journey. It combines concurrent first creation,
outside-root escape pressure, abrupt owner death, inherited-handle pressure,
stale observations, and immediate re-admission around one persistent identity.

**Relevant subsystems**

- process mutation ownership
- namespace confinement
- C.3 process-local root admission
- C.4 OS-level root admission
- unexpected death and stale observation

**Named executable and roles**

- Cargo integration-test executable: `physical_media_journeys`
- named test: `mutation_contention_confinement_and_readmission`
- eight `mutation-contender` child processes released by one parent barrier
- one `unrelated-inheritance-probe` child spawned by the winner
- one `post-death-successor` fresh process
- the separate `physical_media_os_observer` process hashes sentinel files
  before and after the schedule using OS APIs only

**Initial world**

- one absent target root
- one separate outside-sentinel root containing same-named directories, files,
  and immutable random bytes
- hostile relative-path corpus and platform-appropriate symlink/junction/
  reparse candidates are prepared before contention
- every contender receives identical production configuration and no backend,
  store identity, lock result, or expected winner id

**Execution**

- release all eight contenders simultaneously
- hold the one admitted runtime at named production yieldpoints while the seven
  denials finish and while path-escape/open/rename/delete attempts are made
  through owner-level qualification requests
- spawn the unrelated inheritance probe and confirm it receives no mutation
  handle
- kill the winning writer without normal close while it owns no partially
  published higher-level artifact
- immediately start the successor and have it admit, observe, and close
- run the outside observer and independent namespace observer

**Assertions**

- exactly one initial contender receives `MediaOwnedPhysicalRuntime`; seven
  receive typed contention and perform zero post-lease identity publication,
  qualification, cleanup, staging, family, or mutation operations; any
  pre-lease fixed-scaffold attempts are the only allowed loser effects and are
  counted exactly
- all attempted escapes are rejected before an outside effect; the sentinel
  path set, metadata required by the oracle, lengths, and bytes remain exactly
  unchanged
- the unrelated child does not prolong the OS lock after winner death
- the successor acquires ownership without deleting or trusting stale owner
  metadata, discovers the same stable store identity, and receives entirely new
  ephemeral identities
- the winner's last observation is bound to the terminated process and cannot
  be used by the successor or promoted into authority
- exact ownership, denial, confinement, operation, and handle counters match
  through the named in-flight release boundary; the OS termination status and
  successor acquisition prove death/release without inventing a post-mortem
  in-memory counter claim

**Warnings**

- Do not use threads as a substitute for the process contention claim.
- Do not mutate backend private state to “release” the dead owner.
- Do not accept “one process succeeded” without proving the seven losers made
  no physical progress beyond the minimum idempotent scaffold operations
  explicitly required to contend for the canonical OS lock.
- Do not rely on sleep duration to establish interleaving; use named production
  yieldpoints and process barriers.
- Do not claim that counters stored only in the killed process can report an
  event after that process dies. Retain the exact pre-death boundary snapshot,
  then use OS exit status and fresh-process reacquisition as the death oracle.

**Test requirements**

- The entire hostile schedule must execute twice from independent absent roots
  and produce an identical semantic and post-lease counter outcome modulo
  explicitly projected ephemeral ids. Each run must also retain and validate
  its exact pre-lease scaffold counters against the bounded idempotent-prefix
  law; OS scheduling may choose which contender observes or creates each
  scaffold entry, so that distribution is not falsely claimed deterministic.
- A cross-root capability substitution must prove a same-relative-name handle
  from root A cannot affect root B.
- Controlled defects that grant ownership from lock-file metadata or allow an
  inherited child handle to retain the lease must fail separate ownership
  predicates.

**Engineering decisions**

- The path corpus reaches the real confinement boundary through the sealed
  qualification/test decorator but cannot mint arbitrary product file
  authority.
- Process death is performed by the parent with an OS termination primitive
  after the winner emits a production-boundary reached observation.
- Counter comparison projects away only declared ephemeral identity values and
  the per-contender distribution of the explicitly named absent-root scaffold
  race. Exact raw counters are still retained and checked in each run; every
  post-lease operation cardinality and outcome remains in the deterministic
  comparison.

**Open questions**

- None.

### Phase 15: Prove Partial Effects And Barrier Honesty

Build deterministic fault scenarios over real filesystem effects. Each
fault case starts from its own root, kills or exits the actor as required, and
uses fresh observation to prove that typed outcome, actual bytes, operation
sequence, and counters describe the same reality.

**Relevant subsystems**

- certification fault interposer
- media operation outcome topology
- namespace publication protocol
- independent OS observer

**Named executable and roles**

- Cargo integration-test executable: `physical_media_journeys`
- named test: `partial_effects_and_barrier_honesty`
- `faulted-media-writer` child using the production runtime facade plus one
  sealed certification fault schedule
- separate `physical_media_os_observer` child using OS APIs and duplicated
  stable namespace grammar only
- `fault-reopener` child attempting fresh admission where the observed state
  permits it

**Fault schedule**

Use one fresh root for every named seam:

- before root creation
- after fixed directory creation
- before and after staged identity create
- after an exact short identity-write prefix
- after complete write but before file synchronization
- after file synchronization but before replacement
- after replacement but before directory synchronization
- after directory synchronization but before caller observation
- during qualification append, positioned write, truncate/allocation,
  metadata/list, cleanup delete, and cleanup directory synchronization
- before and after mutation-lock release

Cases whose purpose is abrupt-death observation terminate the writer at the
yieldpoint rather than translating death into `Err`.

**Assertions**

- every case matches exactly one typed outcome and retry/inspection posture
- requested bytes equal completed bytes plus the exact unperformed suffix;
  neither counters nor receipts claim bytes the OS observer cannot find
- before-effect denials leave no unauthorized residue
- known staged residue is classified and cleaned only by a later lease holder
  that proves its ownership and non-publication
- visible replacement followed by failed directory barrier is reported
  indeterminate, never rolled back in diagnostics
- no fault case constructs `MediaOwnedPhysicalRuntime` unless all required
  ownership, identity, capability, cleanup, and publication facts completed
- identical source/binary/profile/schedule reruns localize to the same operation
  and yield the same observable result

**Warnings**

- Do not edit persisted files after the writer exits to manufacture the fault
  effect; the interposer must deliver it at the production boundary.
- Do not allow the independent observer to call runtime namespace
  classification or cleanup code.
- Do not accept `is_err()`, non-empty logs, or nonzero counters as localization.

**Test requirements**

- Every fault case asserts its typed outcome, exact causal-boundary role and
  ordinal, operation/handle identity when issued, and observed artifact delta.
- A pass-through control, a definite-before-effect fault, a partial-transfer
  fault, an indeterminate-publication fault, and abrupt process death must run
  in developer/CI lanes according to cost; the full seam campaign runs in
  release certification.

**Engineering decisions**

- Each root is single-use per destructive fault case, preventing prior residue
  from becoming hidden input.
- The oracle predicts the allowed path/byte outcomes before the writer runs.
- Any Foundational diagnostic, receipt, canonical, or performance artifact is
  materialized only after direct OS, Store-outcome, and counter assertions have
  decided the result; diagnostic output never substitutes for those checks.
- Runtime and observer disagreement is emitted explicitly and fails the
  relevant predicate; neither side silently reconciles the other.

**Open questions**

- None.

## Non-Fake Acceptance Setup

### Production subject

The exact production facade under joined test is:

- `PhysicalStore::admit(PhysicalRuntimeAdmission)`
- `AdmittedPhysicalRuntime::try_admit_filesystem_media(FilesystemMediaAdmission)`
- `MediaOwnedPhysicalRuntime::{observer, close, abort}`

The exact internal owners required on the joined call path are:

- C.3 runtime core in `worth-store`
- store-namespace grammar in `worth-store-physical-format`
- concrete local-filesystem media owner in `worth-store-physical-backend`
- C.4 runtime composition in `worth-store`

The exact dedicated proof executable targets are:

- `physical_media_journeys`
- `physical_media_os_observer` (dependency-minimal binary target)
- `physical_media_authority_ui`

The only test layers allowed around the production boundary are:

- a parent process that allocates an enclosing temporary test directory,
  starts/stops child executables, and records process exits
- a sealed certification fault interposer decorating the real filesystem owner
- an independent OS observer using ordinary read-only filesystem APIs and
  stable physical-format namespace declarations
- allocation instrumentation that observes process/media allocation without
  supplying buffers or expected state

The writer/reopener roles may not call `std::fs::write`, raw platform file
APIs, a memory backend, certification-only file creation, replay reconstruction,
or a test fixture writer.

### Initial world

- The primary store root is absent. A second control root exists and is truly
  empty. The parent directory is on a named qualified local filesystem, not a
  RAM disk, mock filesystem, or in-memory mount.
- Diagnostic output identifies the OS, filesystem, backend profile, namespace
  format, root, feature posture, and fault schedule needed to reproduce a
  failure; it does not hash source trees or test binaries.
- The certification qualification payload is deterministic and at least 1 MiB
  + 257 bytes, generated incrementally with a maximum 64 KiB operation buffer
  so no whole-payload allocation is required. Ordinary existing-root admission
  does not execute this destructive qualification transaction.
- The ordinary media-operation transient allocation ceiling is 4 MiB per
  process for the C.4 qualification/journey workload, excluding executable,
  Rust test harness, and OS cache memory. Exact scoped allocator observations
  accompany the claim.
- No store identity, namespace directory, lock target, family directory,
  staging path, capability report, backend instance, open handle, cached page,
  replay artifact, persisted heap layout, or expected writer output exists
  before execution.
- The outside-sentinel root is created independently with deterministic random
  bytes and same-relative-name collision candidates. Its expected digest and
  path manifest are fixed before any writer starts.
- Fault profiles and expected outcomes are fixed before the corresponding
  writer executable runs.
- Page size, WAL format, checkpoint interval, and semantic record model are
  explicitly not applicable because C.4 installs no such owner.

### Execution topology

1. Run the backend owner tests directly against a real qualified local root.
2. Run the namespace journey against absent and existing-empty roots through
   the public runtime facade.
3. Terminate each writer process and start distinct observer/reopener processes
   with only root and production configuration.
4. Run the eight-process contention/death/confinement journey with named
   production yieldpoints and no sleep-based ordering.
5. Run the fault journey with one new root per seam. Where death is the fault,
   terminate the writer process rather than returning an error from a live
   process.
6. Compile the ordinary product under its default profile, then run the
   consolidated compiler authority suite once under its directly declared
   maximal certification feature.
7. Run canonical namespace comparison and explicit support/performance lowering
   inside the existing owner/journey products; these are assertions, not new
   Cargo targets or evidence-producing prerequisite tests.
8. Run source/dependency gates and constitutional checks before closeout.

Process identity, runtime identity, stable store identity, media-owner identity,
mutation-lease identity, operation identities, root identity, and every
authority transition are recorded as separate fields.

### Independent observation

- The OS observer receives only the root path, read-only observation output
  path, and declared namespace format version range.
- It does not receive runtime objects, store identity, capability reports,
  writer buffers, operation receipts, expected path manifests, decoded writer
  values, replay artifacts, or persisted heap layouts.
- It parses `namespace/identity` with an independent minimal parser derived
  from stable format constants and grammar. It shares neither the production
  encoder/decoder nor runtime admission, locking, cleanup, capability
  qualification, or publication-decision code.
- The parent comparison boundary treats its decoded semantic output as a
  Foundational external/boundary value; the dependency-minimal observer need
  not import Foundational. Canonical comparison may consume that output, but
  only ordinary Store admission may revalidate persisted bytes into current
  `StableStoreIdentity` use.
- Expected allowed path/byte outcomes for every fault seam are generated before
  the writer runs and are selected from the observed terminal seam, not from a
  writer claim about success.
- Outside-root observation occurs before and after every confinement campaign.
- Hardware/deployment qualification remains distinct evidence; C.4 process-
  death observation does not impersonate sudden-power-loss certification.

### Assertions

- Only successful real namespace admission constructs
  `MediaOwnedPhysicalRuntime`.
- Exactly one live mutation owner exists per store root; denied contenders
  produce zero forbidden effects.
- The stable store identity is byte-derived, persists across relocation/reopen,
  and remains distinct from every ephemeral identity.
- Exact canonical paths, lengths, bytes, staging cleanup, family-directory
  emptiness, and lock-target posture match the predeclared oracle.
- All effects stay inside the admitted root and the outside sentinel is exactly
  unchanged.
- File write, file synchronization, replacement, directory synchronization,
  and caller observation remain distinct and exactly ordered.
- Typed outcomes agree with OS-observed zero, partial, complete, or
  indeterminate effects.
- Capability and media-admission proof outcomes preserve success, denial,
  deferment, stale, rebind-required, and terminal failure exactly, and their
  basis identities match the live root/profile observations.
- Canonical namespace comparison agrees across runtime and independent observer
  producers without substituting for direct byte and barrier assertions.
- Any materialized Foundational executed/performance evidence is derivable from
  the Store outcome and exact counter rows; disabling it changes neither
  execution nor counters and performs zero boundary-materialization work.
- Requested/completed byte conservation, handle conservation, ownership
  conservation, and one-terminal-outcome-per-attempt hold exactly.
- Ordinary operation memory scales with declared range/buffer width, not file,
  payload, or namespace size.
- Rich diagnostics disabled produces zero rich-artifact allocations while
  preserving operational results.
- No page, record, WAL, checkpoint, recovery, integrity, scheduler, index,
  blob, Query, Relational, Signal, or Runtime Bridge authority is installed.
- No direct ordinary writer bypass or public media escape hatch remains.

### Mechanical anti-substitution gates

Closeout must mechanically reject:

- public construction, cloning, reconstruction, serialization, or field
  extraction of `MediaOwnedPhysicalRuntime`, `FilesystemMediaOwner`, admitted
  capability handles, or the mutation lease
- a memory/mock/certification backend constructing the production phase
- raw path, handle, store identity, capability report, lock metadata, replay
  artifact, or persisted heap layout promoting into media authority
- a generic or unrelated `worth-proof` witness, stale/bridged proof-bearing
  form, raw `TransitionOutcome`, Foundational identity/boundary artifact,
  canonical digest, support report, or performance claim promoting into a
  capability handle or runtime phase
- a second capability qualification authority remaining independently usable
  beside the migrated `BackendCapabilityClaimOutcome` lane
- direct ordinary file effects outside the physical-backend boundary
- same-process writer/reopen evidence
- writer-provided expected bytes or identity reaching the independent observer
- private-state corruption or test-created output files
- automatic whole-file/whole-namespace materialization
- per-fixture nested Cargo builds or target directories
- behavioral guarantees moved out of direct owner/journey tests and into
  compile-fail or source-shape tests
- success inferred only from file visibility, `is_ok`, `is_err`, nonzero
  counters, logs, or elapsed time

### Evidence and rerun

The three hostile scenario products assert Store outcomes, independent
filesystem observations, barrier order, exact artifact changes, and structural
counters directly. On failure they report the case, process role, fault seam,
backend/profile, and relevant operation identity needed for diagnosis. Test
reports are disposable diagnostics: they are not hashed, reconciled into a
certificate, accepted as runtime authority, or used as prerequisites for later
tests.

Canonical commands are expected to include:

```text
cargo test -p worth-store-physical-backend
cargo test -p worth-store --all-features --test physical_media_journeys namespace_creation_and_fresh_process_discovery
cargo test -p worth-store --all-features --test physical_media_journeys mutation_contention_confinement_and_readmission
cargo test -p worth-store --all-features --test physical_media_journeys partial_effects_and_barrier_honesty
cargo test -p worth-store --all-features --test physical_media_authority_ui
cargo clippy -p worth-store-physical-backend --all-targets --all-features -- -D warnings
cargo clippy -p worth-store --all-targets --all-features -- -D warnings
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

Exact package/target names may change only if the responsibility name improves;
the spec and roadmap must be updated before implementation silently diverges.

## Must Ship

- one consuming C.3-to-C.4 transition installing one concrete real filesystem
  owner
- one non-cloneable `MediaOwnedPhysicalRuntime` with private exhaustive
  construction and phase-scoped observation
- one persistent, versioned, checksummed store namespace identity independent
  of path and runtime identity
- one Store-owned canonical namespace lowering and explicit Foundational
  identity-boundary posture for portable comparison without weakening
  `StableStoreIdentity`
- deterministic classification of absent, empty, initializing, initialized,
  incompatible, contended, damaged, and ambiguous roots
- one confined namespace-relative path authority with link/reparse/race defense
- real positioned read/write, append, truncate, allocation, metadata, bounded
  listing, synchronization, replacement, and deletion mechanics
- typed zero, partial, completed, unsupported, stale, and indeterminate outcomes
  with explicit retry posture
- root-specific base capability qualification and concrete optional capability
  handles
- one proof-backed root/profile qualification progression preserving denied,
  deferred, stale, rebind-required, and failed outcomes, with no parallel
  capability authority lane
- one proof-backed C.3-to-C.4 admission outcome whose payloads preserve the
  exact fate of consumed runtime authority
- one OS-enforced process mutation lease with non-inherited handle posture
- exact namespace initialization, replacement, deletion, and cleanup protocols
- deterministic fault interposition around the real backend
- exact operation, byte, barrier, handle, ownership, denial, residue, and
  allocation counters
- optional Foundational canonical/support/performance lowerings derived from
  Store-owned facts outside primitive I/O and unable to influence execution
- exhaustive close, abort, panic, unexpected-drop, and process-death behavior
- migration, quarantine, or deletion of every in-scope writer bypass
- direct fresh-process hostile scenarios, an independent read-only observer,
  and focused compiler authority denials
- default/maximal-feature compile denials and constitutional boundary gates

## Must Preserve

- C.3 remains the sole runtime admission and lifecycle root.
- `AdmittedPhysicalRuntime` remains non-physical and exposes no media method
  except its one consuming progression.
- physical format owns namespace byte meaning but performs no OS effects.
- physical backend owns OS effects but cannot promote runtime authority.
- `worth-proof` owns checked progression topology but never the runtime, OS
  lease, root handle, Store identity, or media effect.
- `worth-foundational` owns portable boundary vocabulary but never primitive
  I/O, framing checksums, capability admission, or runtime promotion.
- certification observes and injects faults but cannot replace the backend or
  mint production authority.
- authoritative, staged, derived, diagnostic, and observer state remain
  structurally distinct.
- ordinary media operations remain range/buffer bounded and do not materialize
  entire files, roots, or qualification payloads.
- unsupported platform behavior is denied honestly rather than emulated by a
  weaker memory or temporary-file claim.
- C.5 retains ownership of pages, segments, extents, manifests, records, and
  fresh physical reopen.
- C.7 retains WAL and physical acknowledgment law; C.8 retains recovery source
  precedence; C.10 retains stable physical reads and I/O scheduling.
- Query, Relational, Signal, Runtime Bridge, semantic MVCC, and branch policy
  remain absent from Part I physical media.

## Acceptance Evidence

C.4 closes with:

- physical-format namespace encoding/classification tests
- Foundational canonical namespace parity and structured mismatch tests
- real-filesystem backend owner tests and capability qualification evidence
- proof-outcome freshness, rebind, cross-capability substitution, and
  single-qualification-authority tests
- default-profile production compilation and one consolidated maximal-feature
  compiler authority specimen
- namespace creation/fresh-process discovery journey
- multi-process contention/confinement/death/re-admission journey
- partial-effect/barrier/fault determinism journey
- exact structural counter and scoped allocation snapshots
- explicit support/performance lowering parity plus hot-path zero-
  materialization evidence
- independent OS namespace and outside-sentinel manifests
- zero unresolved ordinary writers in the scoped source and call-path review
- strict lint, workspace check, boundary-check, and agent-context results
- reproducible fault schedules and direct failure diagnostics

Test-lane posture:

- backend owner checks and namespace format tests belong in the owner lane and
  target seconds after warm compilation
- public lifecycle/observer checks belong in developer smoke and target under
  one minute on the declared reference machine
- namespace qualification/discovery, multi-process contention, representative
  fault seams, and maximal UI belong in CI
  jobs partitioned by proof family
- the full seam campaign and supported-platform matrix belong in release
  certification
- sudden-power-loss and device-specific durability claims belong only in
  hardware qualification

## Sequencing Notes

- Phases 1 through 3 freeze semantic ownership and truthful outcomes before
  platform implementation begins.
- Phases 4 through 8 build confinement, real effects, durability, capability
  truth, and OS ownership without creating a pretend runtime phase.
- Phase 9 is the first phase allowed to introduce
  `MediaOwnedPhysicalRuntime`, because it installs the already-real owner in the
  same coherent slice.
- Phases 10 through 12 complete lifecycle, observability, faultability, and
  cutover before system acceptance begins.
- Phases 13 through 15 are ordered control, hostile authority, then direct fault
  scenarios. Each exercises the joined production path.
- C.5 may begin only after C.4 closeout proves the media-owned phase and the
  internal artifact-family handoff. C.5 must consume the phase rather than
  recreate or select a backend.
- C.4 does not issue an S.10 readiness object and does not restore any reopened
  S.1 through S.9 claim beyond its precise media/namespace boundary.

## Completion Standard

### C.6 boundary hardening amendment

- Root/profile qualification is handle-bound. The ambient path may select the
  initial open and must be revalidated against that handle before admission,
  but root identity, volume identity, mount selection, and capability facts
  are derived from the opened directory handle. Matching volume candidates
  that disagree materially fail closed.
- Every artifact mutation uses one owner-local coordinator in this order:
  live mutation ownership, namespace sequence when names can change, sorted
  owner-relative path reservation, no-follow open, deterministic stable-file
  sequence, then the effect. A chunked new file retains its reservation until
  the construction handle is dropped.
- Exact range writes return backend-minted receipts bound to owner, Store,
  artifact coordinate, offset/length, payload digest, operation identity,
  completed bytes, and durability posture. Partial or indeterminate effects
  never project as completion.

C.4 is complete only when Worth Store can truthfully say:

- the sealed runtime owns one real qualified local-filesystem namespace
- store identity comes from durably published bytes, not caller or path
- only one process holds mutation authority and death releases it without
  trusting stale metadata
- paths and handles cannot escape, cross roots, or outlive their authority
- short, partial, failed, completed, synchronized, published, and indeterminate
  effects remain distinguishable
- file and directory durability operations actually reach the production
  backend in the required order
- every effect is faultable and structurally counted at its causal boundary
- fresh processes independently observe the same namespace truth without live
  heap state
- C.4 direct owner, scenario, observer, and UI tests meet C.1 warm owner/smoke
  budgets without nested compilation
- no memory/mock backend, raw path, replay artifact, test authority, or direct
  writer bypass can construct or impersonate the media-owned phase
- C.5 receives one concrete, bounded, honest media owner on which real pages,
  segments, extents, and manifests can finally be built

Anything less is still a filesystem-shaped model, not the physical foundation
required by the reconstruction roadmap.
