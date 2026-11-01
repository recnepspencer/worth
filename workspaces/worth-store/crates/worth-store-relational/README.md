# Relational home on Store

## Initial selected-member read custody

The private controlled initial selector retains the actual verified EOF
membership and its original protected reader/session. It builds the existing
member placement map with request-funded SDK storage, then tries the existing
custody-before-placement gates. It installs only into a genuinely unselected
source. Refusal leaves that source unchanged; an exact controlled retry needs
no new map storage. Ordinary same-version placement is insufficient, and the
ordinary selector refuses to replace controlled custody with a newer version.

Member reads use the selected table index and original reader after later Home
anchor replacement. Duplicate content keeps its last listed physical placement.
Map construction is O(n log distinct members); payload reads preserve their
existing byte-returning, unbudgeted allocation contract. This prerequisite is
not coupled Home readmission, later selection replacement, Complete Native
acknowledgement or bounded final physical-reader retirement.

## Controlled application readmission

Store Application retains one private, funded `SelectedHomeSource` before
Native admission. Its original physical reader, verified EOF membership and
immutable Home payload supply both owner sections and World's frozen journal
bounds. A missing or loaned reader is a refusal; only that actual reader's
anchor lookup can establish no selected checkpoint. Later anchor replacement
does not select another source during an owning readmission retry.

The closed `readmit_home_from_read` entry installs the actual selected-member
route before the shared Native recovery core. The core performs no legacy
reselection; ordinary standalone entries retain their explicit ordinary setup.
Selected canonical and transition coverage can leave that core only after
`recover_with_authority` succeeds. Zero scheduling baselines are not empty-owner
or no-effect evidence.

The qualified Native format currently lacks the required Complete source
admission. Selected coupled Home2 therefore returns `CompleteNativeRequired`
before Native recovery, preserving the owning unshared Home and its original
source. Ordinary checkpoint output or a format marker cannot substitute for
Complete admission. This containment remains unfinished integration work;
genuine no-checkpoint recovery continues through the existing Native owner.

After both actual Native and World admissions consume the initial image and
tails, successful attachment releases the context's local source share outside
admission locks. Any required Native member route retains its own actual read
share. Refusal preserves the initial source. The older bare-error generic API
retains its ordinary abandonment semantics; this slice does not qualify full
Home close or bounded physical-reader retirement.

## Cooperative release of one cold member registration

Store's cold source issues the existing Native member-custody token with its
weak source identity and original private guard. `into_retirement()` and
`advance(&request, &issuing_source)` release one local share or one actual member
at a time. Issuance identity is checked before callbacks; Store additionally
checks the original registry and Serving Arc identities. Validation takes no
locks or I/O; registry advancement uses `try_write`. Busy/stop/refusal retain the
same original guard, member roster and cursor, so retry cannot release a member
twice. Other clones and independent cold readers keep their own protection.

After the final member, including an originally empty registration, the guard's
extra Serving Arc is detached while the exact source is borrowed. The disarmed
original allocation remains opaque until explicit unbudgeted `reclaim()`.
Ordinary incomplete Drop retires only the remaining suffix and is unbudgeted
abandonment, not successful close. Legacy opaque and composite Native providers
remain unsupported by this protocol.

Acquisition prepares the actual roster, count cells and release slots before
registry effects. Each Store member step is
O(log distinct registered members). Unique SDK edits consume prepared cells and
retain detached cells for disposal outside the registry guard. Advance and
ordinary owner Drop allocate no new backing. Final inert metadata disposal and
small Arc control-allocation reclamation retain their ordinary contract. Advance
performs no new reservation, rehash, shrink or arbitrary payload/final
physical-owner Drop.
This prerequisite qualifies one registration/share only, not cold graph,
ApplicationHome or physical close. See the [Native facade](../../../../crates/worth-relational/README.md#retire-one-provider-member-custody-share).

## Existing home format

`RelationalStoreHome::open` owns physical recovery, descriptor admission and
Relational readmission. It initializes a record family only when the lease
holder proves it absent; existing partial or damaged stores are completed or
refused, never overwritten. A surviving namespace identity alone counts as
an existing Store; losing its record family cannot reset incarnation. Competing writers receive the typed
`RelationalStorePhysicalRefusalKind::MutationOwnerContended`.

A recovery-to-serving handoff retains the same OS mutation lease throughout.
The home declares registry version 2, containing `relational.canonical-commits`
and `relational.branch-transitions`. Each journal retains its own unresolved
publication state. The `home` and `relational.checkpoint` records remain
separate anchors.

Schema 12 roots carry 32 anchors in a 3,760-byte payload. Schema 11 refuses with
`UnsupportedRootSchema` before recovery effects. The offline observer reads
schema 12 independently of the serving reader.

Descriptor format 2 under `home` records the Store-issued stable namespace
identity as the home identity, and a checked `u64` incarnation. It uses magic
`WSRHOME1`, version `u16`, identity `[u8;16]`, incarnation `u64`, then runtime
and profile as length-prefixed UTF-8 (`u32` lengths), and a `u16` binding count.
Tagged binding entries are `(u16 tag, u32 length, payload)`; Phase 2 emits zero
entries and refuses unknown tags. Later phases add tags without altering this
format. Integers are little-endian. Descriptor integrity is protected by the
anchored-record frame and root selection.

Every open publishes the next incarnation before Relational readmission and
service. Exhaustion refuses. `home_identity()` is unchanged across reopens;
`incarnation()` is the durably selected serving incarnation. An interrupted
open may have selected its advance even if it never served. Recovery decides
that fate and the next open advances from it once. Incarnation is a lifecycle
counter, not a commit identity; durable data remains valid across opens.
Authority-value fencing belongs to the application lifecycle in Phase 3.

Older descriptor versions and the retired `relational.home` anchor refuse with
`UnsupportedDescriptorVersion`; they are never migrated. A missing descriptor
is completed only when no Relational checkpoint or journal history exists.

The `relational_home` oracle compares recovered owner artifacts and graph state
against a memory-only Relational reference. Incarnation media sweeps and real
process kills cover open-time publication. Independent processes probe the OS
lease at recovery yieldpoints and the serving handoff.


## Protected Native source reads

Native recovery captures one actual protected physical reader, the two journal
heads and the verified selected member table from that reader. Its provider
payload retains the same Serving owner until the reader retires. Later journal
or anchor publication cannot expand that cut or replace its member route.
Owned outputs use the original request's fixed array allocation before payload
growth and return only after exact length and verified EOF. Foreign original
sources refuse before provider I/O. The initial selected-map contract remains
unchanged; this ordinary read capability does not implement Complete capture
or selected-backing replacement.


## Replace selected canonical backing in the same Home attempt

Native CHK18 selection retains the actual verified selected read, its member
map and original source identity. The selected-read callback exposes that fixed
member cut with zero journal heads; later roots cannot redirect member reads.
Foreign source registries refuse even when they share the same Serving owner.

The existing Home checkpoint attempt passes its original request through Native
acknowledgement. Replacement uses the pending selection's fixed version and
verified EOF read. Stop or Busy after admission preserves the same candidate,
old read and retirement work; retry neither republishes nor obtains a latest
reader. Initial selection retains its existing same-read and different-selection
refusals. Explicit ordinary admission failure cleanup remains separate from
successful controlled retirement.

Custody counts remain one private, never-cloned SDK ordered map. Controlled
admission prepares its owned member roster, singleton cells and detached-cell
slots before registry effects. Unique edits retain actual node/entry owners;
controlled advance and ordinary owner Drop allocate no new backing. Detached
cells and independent grants survive until disposal outside the registry guard.
Fixed weak-retained control metadata keeps its existing ordinary contract.
