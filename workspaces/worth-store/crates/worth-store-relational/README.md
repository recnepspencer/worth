# Relational home on Store

## Cooperative release of one cold member registration

Store's cold source issues the existing Native member-custody token with its
weak source identity and original private guard. `into_retirement()` and
`advance(&request, &issuing_source)` release one local share or one actual member
at a time. Issuance identity is checked before callbacks; Store additionally
checks the original registry and Serving Arc identities. Validation takes no
locks or I/O; registry advancement uses `try_write`. Busy/stop/refusal retain the
same original guard, member Vec and cursor, so retry cannot release a member
twice. Other clones and independent cold readers keep their own protection.

After the final member, including an originally empty registration, the guard's
extra Serving Arc is detached while the exact source is borrowed. The disarmed
original allocation remains opaque until explicit unbudgeted `reclaim()`.
Ordinary incomplete Drop retires only the remaining suffix and is unbudgeted
abandonment, not successful close. Legacy opaque and composite Native providers
remain unsupported by this protocol.

Acquisition adds one allocation and indirection. Each Store member step is
O(log distinct registered members); node rebalancing/deallocation and small Arc
control-allocation reclamation may occur. Advance performs no new allocation,
reservation, rehash, shrink or arbitrary payload/final physical-owner Drop.
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
