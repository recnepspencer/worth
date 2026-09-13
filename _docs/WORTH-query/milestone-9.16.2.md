# Milestone 9.16.2: Portable Query Packages And Fresh Readmission

> **Status:** In progress — Phases 1 and 2 are committed; Phases 3 and 4 close
> this milestone before Milestone 9.17.1 begins.
>
> **Product posture:** This milestone carries exact Query package meaning
> across process, build, and storage boundaries. It does not persist application
> state, provide runtime durability, or select a physical database.

## Goal And Roadmap Placement

Let a host export one validated Query package as complete typed records, encode
those records as deterministic bounded bytes, attach host-owned release
metadata and signature bytes, and later reconstruct the package as untrusted
meaning that Query validates freshly against an independently expected semantic
identity.

Milestone 9.16.2 follows Milestone 9.16.1.1 because portable carriage must
include the repaired installed graph contracts: declaration-owned application
aspect identity and revision, native schema catalog, typed operation read and
touch scopes, external-effect correlation, and complete aftermath meaning.
Milestone 9.17 consumes the resulting stable identity vocabulary and package
readmission surface while remaining an in-memory runtime milestone.

Persistence has an independent telos and is deliberately absent here:

- PostgreSQL runtime durability is not a Query milestone;
- application state remains memory-resident through Milestones 9.17–9.21;
- hosts may retain or publish package archive bytes, but doing so persists
  descriptive release meaning, not live application state or authority; and
- continuous durability, recovery, demand materialization, physical fetch,
  restart-safe dispatch, and greater-than-memory operation enter with Worth
  Store integration.

## Central Claim

For every validated package accepted by this milestone, canonical export,
archive encoding, archive decoding, bounded reconstruction, and fresh Query
validation reproduce the same semantic identity and complete installed
meaning. Corruption, omission, duplication, reordering, cross-package
splicing, unsupported versions, claimed identity, signatures, and stored bytes
cannot mint Query authority or cause partial acceptance.

The claim is false if:

- a Rust module path or `type_name` participates in portable identity;
- copied operation, effect, query, capability, schema, or marker text can
  counterfeit declaration membership;
- any semantic input to validation is missing from the typed record inventory;
- reconstruction trusts the manifest identity or skips ordinary Query
  validation;
- archive decoding accepts trailing, duplicate, reordered, oversized, or
  unsupported required meaning;
- a signature, checksum, filename, Git tag, release name, archive repository,
  or decoder result becomes installation authority;
- callbacks, providers, credentials, proofs, handles, runtime ids, receipts,
  live subscriptions, or application state enter the archive; or
- ordinary installed execution performs archive, reconstruction, signature,
  or repository work.

## Current Boundary

Query declaration and installation already own package meaning, validation,
canonical identity, installed contracts, and private proof-bearing validated
packages. Phases 1 and 2 established stable portable identities,
declaration-minted provenance, and complete typed package records.

The remaining boundary is intentionally cold:

1. consume untrusted records under explicit count, byte, nesting, and work
   limits;
2. reconstruct only a non-authoritative candidate;
3. run fresh ordinary Query validation and independently compare identity;
4. encode and decode one deterministic versioned archive and release envelope;
5. make host signing and retention useful without allowing either to install
   or activate a package.

## Ownership Lock

| Responsibility | Owner |
| --- | --- |
| Schema, operation, query, capability, effect, workflow, policy, artifact, and contribution meaning | Query declaration and installation |
| Stable semantic identities and declaration membership provenance | Owning Query declarations |
| Complete typed record families, canonical order, export budgets, reconstruction, and fresh validation | Query installation |
| Deterministic archive and release-envelope protocol, bounded decode, checksums, and compatibility profile | `worth-query-package-archive` |
| Signature algorithm, keys, signer trust, Git provenance, release approval, and expected release selection | Host release system |
| Immutable storage or publication of exact archive bytes | Host-selected repository; descriptive storage only |
| Live providers, secrets, installed handles, runtime authority, and application state | Current host/runtime owners; never the package archive |
| Future application-state durability, recovery, residency, physical reads, replication, and retention | Worth Store integration |

No layer substitutes for another. Archive structure is not Query validity;
Query validity is not host trust; host trust is not installation authority;
storage success is not activation; and none of them is application-state
durability.

## Public Contract

Query meaning is exported and reconstructed through the Query host audience:

```rust
let records = validated_package.export_typed_records()?;

let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
    records.manifest().clone(),
    WorthQueryPortablePackageReconstructionLimits::DEFAULT,
)?;

for (index, record) in records.records().iter().cloned().enumerate() {
    reconstruction = reconstruction.push_record(index as u32, record)?;
}

let reconstructed = reconstruction.close()?.materialize()?;
let validated = reconstructed.validate_freshly(
    WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(expected_identity),
)?;
```

Archive carriage is a separate descriptive boundary:

```rust
let archive = encode_package_archive(
    &records,
    WorthQueryPackageArchiveLimits::DEFAULT,
)?;

let decoded = decode_package_archive(
    &archive,
    WorthQueryPackageArchiveLimits::DEFAULT,
)?;
```

Exact API spelling may follow the implemented public facade, but authority and
phase meaning may not drift. Decoder output remains untrusted. The host must
independently select the expected package identity and verify signature bytes
under its current trust policy before installation.

The optional archive repository contract stores and retrieves one immutable
exact archive record. It may report stored, already stored, conflict, denial,
or indeterminate physical outcome. It cannot decode, validate, select,
activate, or execute a package and is not a general database abstraction.

## Adversarial Constraint And Decisive Proof

The decisive court begins with two valid packages that share names and several
member identities but differ semantically. It exports each through the real
Query host facade, then attempts every relevant hostile transformation:

- move Rust modules without changing declared semantic identity;
- mutate declared meaning without changing a human release name;
- omit, duplicate, reorder, truncate, or append a typed record;
- splice a schema, operation, query, capability, effect, artifact, or
  aftermath record from the neighboring package;
- replace a declaration-minted reference with equal caller-authored text;
- alter the claimed manifest identity, checksum, requirements, provenance,
  signer descriptor, or signature bytes;
- exceed each declared byte, record, nesting, text, and reconstruction-work
  limit at the narrowest boundary;
- supply every unsupported protocol-layer version; and
- attempt to install directly from records, decoded bytes, repository output,
  checksum, or signature presence.

The valid package round-trips to the exact semantic identity and complete
installed contract. Every hostile form fails at reconstruction, compatibility,
host trust, expected-identity comparison, or fresh Query validation before
installation or runtime effects. Deleting fresh validation, trusting claimed
identity, accepting noncanonical order, widening compatibility without a real
reader, or exposing a proof-bearing constructor must turn this evidence red.

This is a package-boundary court. It requires no database, runtime restart,
dispatch worker, application-state recovery, or persistence reenactment.

## Product Decision Lock

1. Every package-relevant Rust type axis has an explicit stable semantic
   identity. Module paths and `type_name` are diagnostics only.
2. Operation and effect references are minted by their owning declaration and
   carry private membership provenance. Equal spelling cannot substitute.
3. Export consumes a freshly validated package and produces descriptive typed
   records only.
4. The record inventory contains every input capable of changing package
   validation or semantic identity, including the Milestone 9.16.1.1 repaired
   installed contracts.
5. Reconstruction accepts only untrusted records, enforces closure and work
   budgets, and produces no validated package or installed authority.
6. Fresh ordinary Query validation is mandatory after reconstruction.
7. Expected package identity comes from an independent host choice and is
   compared with the freshly derived identity.
8. Archive, manifest, record-frame, and release-envelope versions evolve
   independently under explicit compatibility windows. Unsupported required
   meaning fails closed.
9. Canonical ordering and encoding are deterministic. Unknown optional data is
   ignorable only when its versioned contract proves semantic irrelevance.
10. The host owns signing keys and trust. Query and the archive crate handle no
    private key and infer no trust from signature presence.
11. A repository retains exact bytes under exact identity. Mutable `latest`
    selection and overwrite are not part of the repository contract.
12. Portable packages contain definition meaning, not runtime state. They do
    not serialize callbacks, providers, credentials, proofs, receipts,
    subscriptions, caches, workflow instances, answers, or database topology.
13. Export, encoding, signing, decode, and reconstruction are cold
    release/startup work. Warm execution performs none of them.
14. PostgreSQL, runtime durability, state snapshots, recovery, residency,
    persistent dispatch, and application composition are excluded rather than
    represented as deferred provider hooks.

## Destination Topology

```text
workspaces/worth-query/crates/worth-query-declaration/src/
    portable_identity/                    # existing; stable declared identity
    application_schema/                   # existing; minted member provenance

workspaces/worth-query/crates/worth-query-installation/src/package/
    portable_records/                     # existing; complete typed export
        manifest.rs
        record.rs
        record_view.rs
        record_set.rs
        limits.rs
    reconstruction/                       # created; untrusted fresh readmission
        candidate.rs
        progression.rs
        limits.rs
        denial.rs
        expected_identity.rs

workspaces/worth-query/crates/worth-query-package-archive/src/
    facade.rs                              # stable archive audience
    protocol.rs
    compatibility/
    encoding.rs
    decoding/
    manifest.rs
    record/
    envelope/
    limits.rs
    denial.rs
    repository/                           # descriptive exact-byte contract

tools/worth-query-release/                # host release preflight/finalization
.github/workflows/publish-worth-query-release.yml

workspaces/worth-query/crates/worth-query/docs/
    portable-packages.md                  # developer feature authority
```

Stable meaning remains above volatile transport. The archive crate may depend
on Query declaration/installation facades and Foundational protocol values;
Query owners must not depend on a physical repository. Forbidden destinations
include a generic database backend, Query runtime persistence module,
application-composition workspace, PostgreSQL adapter, callback serializer,
proof serializer, `latest` release registry, or archive-owned package
validator.

## Ordered Phase Plan

### Phase 1: Stable Identity And Declared Provenance — complete

Replace package-canonical representation identity with explicit declared
semantic identity for every relevant Rust axis. Make operation and effect
references declaration-minted. Collision, module-move, semantic-mutation,
forgery, and compile-boundary evidence lets Phase 2 trust identity without
trusting representation accidents.

### Phase 2: Complete Typed Package Export — complete

Export one freshly validated package as a versioned manifest plus complete
typed records in canonical order under explicit work limits. Closure inventory
and omission/duplication/cross-family mutations let Phase 3 trust that portable
records carry every semantic input exactly once.

### Phase 3: Bounded Reconstruction And Fresh Validation

Consume untrusted typed records through a compiler-visible progression,
enforce manifest closure and budgets, materialize a non-authoritative
candidate, validate it through ordinary Query installation, and compare its
fresh identity with an independently expected identity. Round-trip,
cross-splice, noncanonical order, forged identity, and budget evidence lets
Phase 4 trust semantic readmission rather than decoding success.

### Phase 4: Neutral Archive And Host Release Boundary

Encode the complete record set as one deterministic versioned archive. Define
the host-owned release envelope, signing payload, external signature carriage,
compatibility profile, exact archive repository contract, and release
preflight/finalization tooling. Golden bytes, tamper, unsupported-version,
coexistence, no-overwrite, wrong-expected-release, docs, facade, dependency,
and residue evidence close the milestone.

No fifth phase exists. Runtime durability is not an unfinished phase of this
milestone.

## Documentation Deliverables

- `workspaces/worth-query/crates/worth-query/docs/portable-packages.md` is the
  continuing developer authority for export, archive creation, signing,
  reconstruction, trust separation, compatibility, limits, anti-patterns, and
  current non-goals.
- `workspaces/worth-query/crates/worth-query/docs/AI_README.md` routes callers
  through the public Query host and archive facades and explicitly distinguishes
  package carriage from application-state persistence.
- The release tool and reusable GitHub workflow document their protected-tag,
  expected-identity, signer, and no-overwrite requirements next to the actual
  executable entry points.

Examples and workflows are executable compatibility surfaces and must be
checked against the real facade.

## Performance And Resource Contract

- Export cost is proportional to complete declared package meaning and occurs
  only on the cold release path.
- Reconstruction and archive decode are bounded by explicit bytes, records,
  nesting, text, and declared-work limits before allocation or semantic work
  can escape those envelopes.
- Canonical ordering is established once; no record family may induce hidden
  quadratic global comparison.
- Exact identity lookup is keyed by canonical semantic identity, never a scan
  for a mutable release name.
- Warm package installation/execution performs zero archive encoding,
  signature, repository, reconstruction, or global release-catalog work.

## QA Considerations

Architecture review must confirm that descriptive records, archive bytes,
checksums, signatures, repositories, and expected identities cannot mint Query
authority. Compatibility review must cover corruption, unsupported versions,
unknown-field posture, canonical ordering, and deterministic bytes. Resource
review must verify all decode and reconstruction budgets before broad work.
DX review must confirm that callers can export and reconstruct through public
facades without learning internal module topology. Tests must use independent
expected identity and mutation-sensitive hostile inputs rather than asking the
decoder to certify itself.

## Must Ship

- stable semantic identities and declaration-minted provenance;
- complete typed manifest and record families;
- bounded untrusted reconstruction and fresh Query validation;
- independent expected-identity comparison;
- deterministic versioned archive and release envelope;
- explicit compatibility profile and typed denials;
- host release preflight/finalization and protected publication workflow;
- optional immutable exact-archive repository contract;
- public facade, documentation, golden vectors, hostile reconstruction and
  archive tests, dependency enforcement, and residue proof.

## Must Preserve

- all Milestone 9.16.1.1 installed graph and aftermath contracts;
- Query ownership of declaration, validation, semantic identity, and
  installation;
- private proofs, installed handles, runtime authority, providers, callbacks,
  and secrets;
- Foundational ownership of protocol and canonical boundary vocabulary;
- host ownership of signing keys, trust, provenance, expected release, and
  publication policy;
- ordinary/reconstructive cost separation; and
- future Worth Store freedom to store or index package bytes without inheriting
  a PostgreSQL schema or Query-owned physical model.

## Explicit Non-Goals

- application-state, workflow-instance, or answer persistence;
- PostgreSQL, SQLite, filesystem snapshot, or Worth Store runtime integration;
- durable-before-publication ordering or runtime commit acknowledgement;
- checkpoints, replay recovery, residency, eviction, paging, physical fetch,
  dispatch leasing, fencing, retry persistence, backup, or restore;
- serializing callbacks, providers, credentials, proofs, receipts, handles,
  subscriptions, caches, or live authority;
- package activation, mutable `latest` selection, or fleet rollout policy; and
- Milestone 9.17 owner/composite branch semantics.

## Acceptance Evidence

Milestone 9.16.2 closes when focused owner tests, archive protocol tests,
release-tool tests, executable docs, facade/dependency checks, formatting,
line-cap checks, boundary enforcement, and agent-context checks prove:

- exact export/reconstruction equality for every retained installed contract;
- module moves preserve declared identity while semantic changes alter it;
- copied or same-spelled member references cannot enter validated meaning;
- record omission, duplication, reorder, trailing data, corruption,
  cross-splicing, unsupported versions, and exceeded budgets fail closed;
- archive and release-envelope bytes are deterministic under the current
  profile;
- signature presence, claimed identity, checksum, repository output, and
  release name grant no Query authority;
- independently expected identity is compared after fresh validation;
- multiple same-named releases coexist by exact semantic identity;
- protected release tooling signs only admitted canonical payloads and never
  overwrites an existing release; and
- warm execution performs no archive or reconstruction work.

## Successor Handoff

Milestone 9.17 receives stable package identity, declaration provenance,
complete typed package records, and fresh readmission. It adds in-memory owner
bases, branch-local MVCC, installed Runtime Bridge correspondence, dedicated
Runtime World composite history/currentness, and Query carriage without adding
persistence or moving package authority.

Worth Store integration later receives the same descriptive package archive
as one input to physical application composition. Store defines continuous
state durability, recovery, residency, physical fetch, retention, replication,
and restart-safe operational lifecycle under the semantic owners' contracts.
It does not reinterpret package bytes, accept archive identity as runtime
authority, or require Query to learn Store topology.
