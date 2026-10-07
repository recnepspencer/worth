# D.3 Page-Granular Authoritative Trees

This note governs [D.3](roadmap.md#d3-page-granular-authoritative-trees).
Store owns physical keys, values, pages, and publication; callers own their
meaning and order-preserving encodings. D.3 supplies the format and batch engine
that D.4, D.5, D.6, D.9, and D.11 consume. It introduces no alternate backend.

Code citations use these directory prefixes, followed by `path:line`:

- `S/`: `workspaces/worth-store/crates/worth-store/src/physical_runtime/`.
- `F/`: `workspaces/worth-store/crates/worth-store-physical-format/src/`.
- `I/`: `workspaces/worth-store/crates/worth-store-physical-integrity/src/`.
- `R/`: `workspaces/worth-store/crates/worth-store-recovery-runtime/src/`.
- `P/`: `workspaces/worth-store/crates/worth-store-recovery-physics/src/`.
- `O/`: `workspaces/worth-store/crates/worth-store-offline-integrity-observer/src/`.
- `T/`: `workspaces/worth-store/crates/worth-store/tests/`.
- `Proof/`: `crates/worth-proof/src/`.

Statements about the destination below are decisions, not claims that the code
already implements them. Test names prefixed `d3_` are planned. No Cargo command
was run while writing this note; their execution times are unverified. Each
focused command has a five-minute budget that D.3.1's runner must enforce.

## 1. Decisions

### 1.1 One node owns one independently releasable page

**Use immutable, page-sized frames in the existing packed extent arenas.** A
node allocation is exactly `P` bytes, aligned to `P`, with one node and no other
record packed into it. `P` is the admitted 16, 32, or 64 KiB page size
(`F/binary_format/page_size.rs:4`). Open rejects qualified media whose required
alignment exceeds `P`. The existing alignment decision already incorporates
allocation and direct-I/O geometry (`S/record_serving/arena/geometry.rs:6`).

Keep `PersistedRecordIdentity` as the stable reference, including its allocation
epoch; do not introduce a second node-ID spelling. Its selected placement routes
to the page's arena range. Interior references remain 24 bytes, as in
`F/btree_node/codec.rs:48`. The format distinguishes tree pages from opaque
records. Allocation, routing, WAL, scheduler admission, pool loading, checkpoint,
and retirement remain owned by the existing physical platform.

The new page has the 80-byte envelope in section 1.10 and a 64-byte node header.
The node header carries its record identity, family code, kind, level, cell count,
used-byte count, and first child. Leaf first-child bytes are zero. Interior cells
hold a separator and right child; the header holds the leftmost child. Slots use
32-bit offsets and 16-bit lengths, totaling 12 bytes per cell. No offset wraps at
64 KiB. Keys and cells are strictly ordered, cell regions do not overlap, unused
bytes are zero, and the entire page is integrity covered. An empty tree has no
root page; an interior root has at least two children. Non-root occupancy is
checked with the family's admitted geometry, rather than accepted from a header.

This replaces inline-only node placement in
`S/record_serving/publication/director/durable_preparation/protected_index.rs:65`
and the nested record format in `F/btree_node/codec.rs:13`. Today's inline page
has a 24-byte prefix and a 40-byte directory entry per record
(`F/page_record/durable_page.rs:14`); giving a node such a slot does not give it
independent physical retirement. Conversely, exact arena ranges already retire
without deleting the shared arena (`S/durability/retention/retired_artifact.rs:14`,
`:48`). Add a `TreePage` retirement case for one exact range and allocation epoch;
do not implement a new filesystem deleter or one file per node.

All ordinary node faults use the existing `BoundedFrameLoader` and integrity
admission. It already checks that an exact read stays inside its arena range
(`S/record_serving/residency/frame_loading/bounded_loader.rs:37`). Routing metadata
faults remain separately charged until D.5 removes the per-read routing re-walk.

### 1.2 Remove sibling links

**Remove both sibling fields and their codec slots.** Current constructors carry
them (`F/btree_node/node.rs:58`), but the writer supplies `None`
(`S/layout/maintenance/tree.rs:181`) and scans already use a parent stack
(`S/layout/btree/scan.rs:29`, `:206`). Retaining unused hints buys no capability
and invites a cross-root traversal later.

Point reads descend from the selected root. Range reads descend to the lower
bound, advance within the leaf, then ascend to the next parent child and descend
again. Separators are exact lower bounds of right subtrees; a delete updates a
changed bound along the copied path. No scan discovers neighbors through media
addresses or links. D.5 resumes with an exclusive last-returned key and a root
lease, never a saved physical slot.

### 1.3 Declare bounded, typed families at open

**Replace the two-row derived-only registry with admitted declarations.** The
total count is at most 64, including Store's BlobCatalog and DedupeIndex. Preserve
their stable codes 1 and 2; admit other stable nonzero codes only through the
open declaration. Duplicate codes, unknown persisted families, changed class or
key cap, and layout mismatches refuse open before serving or effects. Open may
admit a superset containing new empty families; it cannot reclassify existing
bytes. There is no lazy registration or filesystem-based family discovery.

The declaration contains family identity, `FamilyClass::{Authoritative, Derived}`,
`LayoutClass::{BTree, LsmReserved}`, key cap, admitted access shapes, integrity
version, and retention/recovery participation. Only `Derived` has a declared
rebuild basis. `LsmReserved` has a stable wire tag but no writer, reader, or facade
port; its capability is `Absent` with the roadmap's LSM owner and return point.
This meets all three deferral conditions without promising an LSM API.

Runtime handles are class typed: an authoritative handle cannot be passed to
rebuild, and a derived handle cannot be passed to authoritative submission. The
same private page algorithm serves both; it does not unify their authority or
lifecycle. An authoritative write consumes the concrete platform admission in
section 3. Reading bytes or constructing a declaration never grants it.

Change `S/artifact_family/registry.rs:61`, `:154`, `:191` and open/instance
construction. Today's directory permits 64 entries but decodes only the two
built-ins (`F/derived_family_root_directory.rs:9`, `:222`). Replace that codec and
the manifest's derived-only binding (`F/manifest/derived_family_directory.rs:35`)
with one `FamilyRootDirectory` spelling. Keep the existing root-manifest slot
(`F/manifest/durable_root.rs:48`); do not add a competing authoritative directory.

Each descriptor persists family code, class, layout tag, key cap, optional root,
root height, and the Store generation at which that family last changed. Keep
that stamp even when delete removes the final root, so empty-to-populated-to-empty
cannot make an old expected basis current again. The directory also preserves
the existing blob publication and
quarantine watermarks (`F/derived_family_root_directory.rs:38`). It fits one page
at the maximum count: reserve a 128-byte directory header and at most 128 bytes
per descriptor, so `80 + 128 + 64 * 128 = 8,400 < 16,384`. D.3 has no separate
per-family generation counter; its change stamp uses the one assigned Store
generation. Its expected basis is the selected stamped family descriptor and
Store incarnation. D.4 adds per-root generations without changing node bytes.

### 1.4 One bounded TreeKey constructor

**A key is any byte string of length `0..=K`, compared lexicographically as
unsigned bytes.** Empty keys and empty inline values are legal; an empty tree
and a missing value remain distinct from either. The declaration admits
`1 <= K <= 1,024`; recommend 256 for ordinary families. No locale, typed scalar,
or comparator callback enters Store. The encoder is the caller's responsibility.

`TreeKey::admit(bytes, admitted_family)` is the only bounded constructor. Its
private representation carries the admitted family binding. Batch admission
checks that binding once; decoders invoke the same bounded admission at the
integrity boundary. There is no unchecked conversion through a public `Vec`,
slice, or alternate point-key constructor. Ordinary consumers receive keys from
the tree facade; format parsing does not promote them to write authority.

Replace the engine's fixed `cell_shape` checks
(`S/layout/maintenance/tree/insertion.rs:75`, `S/layout/btree/lookup.rs:135`) and
the general point/range surface's fixed arrays (`S/layout/btree/scan.rs:17`, `:47`).
Blob-specific encoders retain their own 24- and 64-byte contracts; those widths
do not remain engine policy (`S/artifact_family/registry.rs:161`, `:177`).

### 1.5 Variable values and large column chunks

**Inline at most 1,024 bytes; larger values use a tree-owned immutable chunk
tree.** A leaf value is an explicit tagged sum: inline bytes, including length
zero, or `{ root record, total length: u64, SHA-256 }`. The external descriptor
is 64 bytes before the cell's tag/length prefix. A large value has no public blob
object identity, ingest expiry, dedupe policy, or independently published head.
The key's tree root owns its entire value closure.

The alternatives are real, but have different costs:

| Option | Evidence and reason for the decision |
| --- | --- |
| Overflow page chain | It would be new; current leaf values are owned byte vectors (`F/btree_node/node.rs:14`). A chain streams cheaply but a cold byte-range read must visit every preceding link. Bulk geometry requires bounded access to distant column ranges, so do not choose it. |
| Existing blob path | Its builder already keeps one frontier per level (`S/blob/tree/builder.rs:23`), but emitted records require completed C.5 publication before joining that frontier (`:23`, `:147`). Its 4,096-entry nodes are about 256 KiB (`:6`), and C.11 explicitly gives ingest declarations, chunks, frontiers, and generation publication separate durable lifecycles (`plans/worth-store/physical-reconstruction-c11-layout-index-and-native-blob-adoption.md:606`). Calling it from a put would reintroduce root advances per sub-operation and unrelated blob custody. Its facade ports are also deferred by this roadmap. Do not use that lifecycle as tree-value authority. |
| Tree-owned chunk tree | Use the existing arena, SHA-256, integrity, WAL, and bounded-read mechanisms, with value-specific frames owned under the general tree. This supports sequential reads and logarithmic range selection, and includes every value page in the batch's one publication. Choose it. |

A value chunk is one `P`-byte page with an 80-byte envelope and a 64-byte value
header; its maximum payload is `C = P - 144`. An index page has the same header
budget and fixed 72-byte entries: child record, digest, covered length, and byte
offset. Its maximum fanout is `b = floor((P - 144) / 72)`. Entries describe
contiguous, ordered byte ranges, and child digests bind the selected bytes. The
final page may be short. Each external value has at least one index root even
when it contains one chunk; this keeps its descriptor interpretation uniform.

For `L > 1,024`, `n = ceil(L / C)` chunk pages and
`J(n) = ceil(n/b) + ceil(n/b^2) + ... + 1` index pages are written. Stop at the
first term equal to one; compute with successive integer ceiling divisions.
Whole-value verification streams the same closure. Range reads descend by byte
offset and visit only overlapping chunks. D.9 can put a multi-megabyte column
chunk under one key without making one key per vertex or buffering that value.

`put` receives an owned, fallible stream with declared length, not a mandatory
`Vec<u8>`. A page window, one chunk-tree frontier per level, incremental hashing,
and bounded WAL frames suffice. Admit declared byte length, WAL retention,
allocation claims, queue space, and deadline before effects; refuse exhaustion,
never silently turn one put into multiple commits. A short, long, or failed
source before any WAL attempt is denied without effect; after a WAL attempt it
retains the attempt's exact indeterminate fate. No tree-value page is selected
until the whole batch is durable. There is no public unfinished-value backend.

### 1.6 Byte occupancy, split, delete, and underflow

**Use a 25% minimum occupancy for non-root nodes.** Let `U = P - 144` and
`T = ceil(U / 4)`. Occupancy counts slot bytes, key bytes, and value/reference
bytes; it excludes the fixed envelope and header. Interior maximum cell cost is
`K + 12 + 24 = K + 36`. Leaf maximum cell cost is
`12 + K + 9 + max(1,024, 64)`. Admit geometry only when each maximum is at most
`floor(U/4)`. The proposed caps satisfy that at every supported page size; at
16 KiB and `K = 1,024`, the maximum leaf cell is 2,069 bytes and `T = 4,060`.

When a node exceeds `U`, choose a legal boundary minimizing byte imbalance,
breaking ties toward the lower key. Both outputs must occupy `[T, U]`. Interior
splitting includes the promoted separator in the calculation, then removes it
from the children and installs it in the parent. The maximum-cell condition
leaves a legal quarter-full split even at the worst key width. Test that claim
with exhaustive boundary occupancies; do not substitute `cells.len()/2`, which
is today's split rule (`S/layout/maintenance/tree.rs:192`, `:272`).

Delete a missing key is unchanged. Deleting a present key removes its leaf cell
and updates affected separators. Repair an underfull node with its adjacent
sibling under the same parent, preferring the left sibling. Merge if their
combined representation fits `U`; for interiors include the parent separator.
Otherwise redistribute at the legal byte-balanced boundary so both have at
least `T`. Remove the merged-away child and separator, and repair the parent
bottom-up. If a batch empties several adjacent children, normalize that run
before emitting it; two underfull pages whose sum is below `T` are not a legal
finished merge. Continue with the next adjacent child until the result is legal,
or remove the empty run and propagate the underflow. Collapse a one-child root;
deleting the last key removes the root. A nonempty root is exempt from `T`.

The sorted batch visits shared ancestors once and emits only final normalized
pages. No transient split followed by a merge becomes a media write. A value
replacement follows the same delete/insert occupancy rules. These replace the
insert-only recursion in `S/layout/maintenance/tree.rs:101` and the scalar
continuation in `S/layout/maintenance/tree/insertion.rs:21`.

### 1.7 One derived height bound

**Derive the bound once, in admitted tree geometry; readers, writers, validators,
retirement, and cursor allocation consume it.** No consumer spells a height.
An occupancy-only rule cannot derive an absolute height without an addressable
capacity bound. Therefore also admit a Store addressed-byte ceiling
`B <= u64::MAX` and reject allocation beyond it before effects.

The minimum non-root interior fanout is
`f = 1 + ceil(T / (K + 36))`. The minimum leaf entry count is
`ell = ceil(T / (12 + K + 9 + 1,024))`. With `N = floor(B/P)` maximum allocated
pages, a tree of height `h >= 2` has at least `2 * f^(h-2)` leaf pages. Thus the
safe bound is `H = 2 + floor_log_f(floor(N/2))`; handle capacities below two
pages separately (empty or height one). Derive it with checked integer
multiplication/division, not floating-point logarithms. It is conservative:
internal pages and value pages also consume `N`.

For `B = u64::MAX`:

| P | K | f | H |
| --- | --- | --- | --- |
| 16 KiB | 256 | 15 | 14 |
| 16 KiB | 1,024 | 5 | 23 |
| 32 KiB | 1,024 | 9 | 17 |
| 64 KiB | 1,024 | 17 | 13 |

Replace all three constants: `S/layout/btree/lookup.rs:15`,
`S/layout/btree/scan.rs:13`, `S/layout/maintenance/tree.rs:41`. Also replace the
height-sized replacement array and charge arithmetic
(`S/layout/maintenance/tree/replacements.rs:105`, `:56`). Validate level descent,
cycles, and admitted height at ingress; a height bound is not permission to
follow malformed topology. Maximum-height algorithm tests use small admitted
capacity ceilings and independently calculated geometry, not a physically
impossible `u64::MAX` fixture. Maximum-cap keys use every supported page size.

### 1.8 One sorted batch, one mutation, one new root

**A nonempty effective batch is one durable mutation member in one WAL group.**
It contains sorted, unique keys with put/delete operations on one selected
family root. Admission rejects duplicates and descending order before page
allocation. Do not sort caller input or choose last-write-wins. Use the existing
`CanonicalUniqueVec` proof (`Proof/collections/canonical_unique_vec.rs:12`) over
keys, with an equally bounded private operation sequence; operation ordering
must not let two entries with the same key appear distinct. The runtime owns
batch entry, key-byte, scratch-byte, page, value-byte, and WAL-byte budgets.

Read only the union of affected root-to-leaf paths and siblings required for
rebalance. Memoize changed nodes by stable identity in batch-budgeted scratch,
so each shared old node is copied at most once. Stream large values and their
page descriptors; do not retain every value page or every family key. An empty
batch, or only missing deletes and byte-identical inline puts, produces zero
writes and no generation advance. Large streamed puts may write a new immutable
value even if its content matches; D.3 does not promise content deduplication.

Remove per-node `append_layout_record` calls
(`S/layout/maintenance/tree.rs:298`, `S/layout/maintenance/append.rs:52`). Build
one final node/value closure and one successor `FamilyRootDirectory`. Submit
them together through the existing mutation, WAL barrier, data settlement, root
candidate, namespace synchronization, and current-root advance. Today's
managed sequence is already ordered that way
(`S/record_serving/publication/director/managed_mutation.rs:79`), and current-root
advance already requires exactly one successor generation
(`S/durability/publication/current_root_owner/advance_validation.rs:28`).

Large values require a bounded streaming payload form, not the current
projection's whole `Vec` encoding (`F/recovery_projection/codec.rs:36`). Reserve
the mutation's WAL range and allocation runs up front. Emit bounded page-image
frames followed by one terminal batch descriptor containing expected family
basis, successor root/directory, image count, and digest. Only that complete
descriptor admits the member to group sealing. After the WAL barrier, apply the
images from the retained WAL with a page window. Recovery reads the same bounded
sequence; it does not collect the member into memory. Persisted identities and
allocation runs are replay data, never authority. Extend the existing WAL owner
and projection grammar; do not add a spool backend or publish chunk sub-batches.
Unsealed/uncertain allocation claims remain reserved until fate reconciliation,
as existing arena reservations do (`S/record_serving/arena/reservation.rs:11`,
`:97`, `:177`). The group limit must admit an individual large-value batch or
refuse it before effects; it cannot split its atomicity at a segment boundary.

D.3 assigns one ordered publication slot to this one-member batch. While image
frames stream, no later slot may bind to an incomplete predecessor descriptor.
At the terminal-descriptor turn, merge against the last completely admitted
ordered family directory and bind the actual physical predecessor/successor
generation. Include that one final directory page in the member's image digest.
Namespace selection follows that same WAL order after data settlement; it does
not assemble a different directory from an unlocked later snapshot. A shared
barrier may cover several groups, but each D.3 batch keeps its own one successor
slot. D.4's explicit multi-root composition is a different declared operation.

Publish through the existing family-directory slot. One changed batch produces
one tree root and one Store generation advance, regardless of node or value
page count. The ordinary path does not re-walk the entire successor closure to
prove a directory: consume the carried batch construction proof. External
integrity ingress and recovery revalidate at their trust boundaries. Today's
closure collector (`S/layout/maintenance/tree/retirement.rs:273`, `:304`) is not
the ordinary admission algorithm for authoritative data.

### 1.9 Remove whole-Store submission serialization

**Batching removes repeated root advances; scope admission removes the global
pending-submission gate. Both are D.3 work.**
`S/durability/publication/current_root_owner/pending_publication.rs:19` calls
`register_exclusive_pending`; `S/durability/retention/admission.rs:105` rejects
any other pending identity, without comparing physical scope. Reducing a batch
to one call leaves that regression intact.

Replace exclusive pending registration with bounded admitted family scopes.
Same-family mutations serialize through terminal publication or an unsettled
fate; disjoint families may prepare, stream, and settle data concurrently.
Reclaim retains its real intersecting fence. Opaque record operations declare
their physical allocation/routing scope too; they cannot reserve a universal
lock merely because their facade is older. Allocation/frontier and registry
locks remain short; the existing frontier lock is already released before
payload planning (`S/record_serving/publication/director/wal_data_planning.rs:185`,
`:195`). Do not hold them through reads, WAL I/O, or data I/O.

At the existing shared publication turn, compare the admitted family's expected
descriptor against the ordered directory basis, merge the disjoint family delta,
and build the single successor directory/manifest. A changed physical Store
generation alone does not stale a disjoint family. Carry this distinction into
WAL redo: the batch binds its family predecessor; the final physical publication
binds its actual Store predecessor and successor. Remove whole-source equality
as a prerequisite for disjoint delta rebasing
(`S/record_serving/planning/rebased_root.rs:48`), while preserving exact final
root transition validation (`S/durability/publication/current_root_owner/advance_validation.rs:9`).
Never retry by redoing the already prepared tree or losing another family update.

The short shared owners remain WAL ordering/barriers, root publication, and
checkpoint cutover. They may coordinate final durable order; they do not own an
entire submission. A deterministic test pauses A before its WAL handoff after
tree preparation, lets B on another family reach Terminal, then finishes A.
Both family roots survive, and A performs zero second tree preparations. A
same-family B cannot pass A. D.4 changes the scope identity to named roots; D.7
adds its stronger facade concurrency matrix. D.3 does not claim that an
incomplete earlier contiguous WAL range can be bypassed by a durability barrier
(`S/durability/wal/port/group.rs:153`, `:228`).

### 1.10 Reserve S.11 key identity and change the format

**Use an 80-byte durable envelope with a 32-byte reserved key-identity slot.**
Keep current fields through byte 44, including the page LSN at 36..44. Replace
the old checksum slot with key identity at 44..76 and CRC32C at 76..80. Cover
key identity and payload in integrity. D.3 writers zero the key slot and readers
reject nonzero bytes; only S.11 may define its nonzero interpretation. It is
opaque identity space, not a key, permission, or encryption implementation.

Bump `PhysicalRecordFormatVersion::V2` to the sole current `V3`
(`F/binary_format/record_declaration.rs:3`, `:88`), node version 1 to 2
(`F/btree_node/slotted.rs:3`), directory wire version 2 to 3
(`F/derived_family_root_directory.rs:6`), and recovery projection domain v16 to
v17 (`F/recovery_projection.rs:33`). The envelope identity/schema must change
with its header geometry (`F/record_framing/durable_frame.rs:4`, `:200`). Assign
one new current schema per affected kind; root variants remain explicit current
operations, not historical schema readers. Preserve Frames, SourceCopy, release,
quarantine, and tier semantics through that coordinated cutover.

Old stores are refused before recovery promotion, serving, or writes. There is
no migration, dual reader, auto-reset, or compatibility alias. Update bootstrap,
all producers, integrity declarations, checkpoint/root codecs, recovery ingress,
Store rejoin, independent observer, and fixtures together. In particular the
observer independently hardcodes a 48-byte envelope and validates its own version
(`O/integrity_observation/families/durable_frame.rs:10`, `:135`); changing only
the runtime codec is insufficient. Current-format corruption and torn writes
still exercise recovery; unsupported-version refusal is not their replacement.

### 1.11 Move BlobCatalog and DedupeIndex without changing behavior

**Use the general engine as two `Derived` families, with their existing
encoders, source watermarks, verification, and rebuild ownership.** BlobCatalog
remains `(object, generation) -> publication record`, with point/range/prefix;
DedupeIndex remains `(scope, digest) -> publication, ordinal, chunk`, point only
(`S/artifact_family/registry.rs:148`, `:154`, `:171`; its 56-byte value codec is
`S/blob/dedupe/key.rs:60`). Neither becomes truth or a release authority.

Accumulate the catalog and dedupe edits from a publication into sorted family
batches, using the shared multi-page mutation machinery. Preserve exact stale
value/conflict rules currently carried by `superseded`
(`S/layout/maintenance/tree/insertion.rs:50`), scope separation, byte comparison,
quarantine, localized corruption, and rebuilding from selected publications and
claims (`S/layout/rebuild/execution.rs:116`, `:135`). These checks belong in blob
maintenance, not in the general put implementation. Preserve the watermark's
freshness validation while removing the engine's fixed widths. Delete the
derived scalar writer, continuation chain, and private node-append path when
their consumers switch; no derived-only tree engine survives.

### 1.12 Retirement and reclamation before D.4

**Authoritative selected pages are retained until D.11.** D.3 does not infer
authority to free them from one currently visible root. This follows the
roadmap's standing retention rule, including value chunks and value-index pages.
Resource pressure is a typed refusal before effects, not an excuse to free them.

Today's Store-global derived families can retire replaced pages on their one
root line. The batch carries the exact removed-page set; directory replacement
unroutes it atomically. Actual range reuse waits for reader/root protection,
checkpoint and WAL obligations, durable retirement Intent and Completion, and
published free-space release. Those existing blockers are explicit
(`S/durability/retention/retirement.rs:40`, `:44`;
`S/record_serving/publication/director/retirement.rs:93`, `:109`). Merely writing a
new root does not free a page. Retirement is separately counted maintenance,
not additional foreground batch node writes.

Use bounded retirement manifests/quanta instead of retaining a whole old-tree
closure in a `Vec` (`S/layout/maintenance/tree/retirement.rs:27`, `:98`). Remove
the need for all free ranges resident on reopen: the current allocator restores
every arena free range into address/size indexes
(`S/record_serving/arena/reconstruction.rs:55`, `S/record_serving/arena/free_ranges.rs:10`).
Replace that heap requirement with bounded page-backed address/size membership
access under the existing free-space owner, plus bounded in-flight allocation
claims. Reuse the existing membership-block platform; do not make it a second
user family or recursively allocate a free-list tree through itself. Reserve
metadata allocation headroom explicitly. This prerequisite belongs in D.3,
since independently freeing node pages otherwise ties maximum database size to
free-list residency. No foreground batch scans the full free list.

The same residency rule applies to publication metadata. Current free-space
repacking explicitly budgets backing for every output block
(`S/record_serving/planning/free_space_routing/successor/repack.rs:63`), and settled
projection merging accumulates records, placements and manifests
(`S/record_serving/planning/settled_root_projection.rs:70`). Replace whole-output
frame retention with streamed manifests and bounded frontiers. Ordinary tree
batches keep fixed admitted metadata block geometry and copy only affected
membership/routing paths. A full metadata repack is an explicitly budgeted
maintenance operation with streamed output and separately reported whole-tree
work, never an ordinary mutation fallback. This extends the existing metadata
owners; it does not install a second allocator or tree backend.

Invalidate the released arena range in the pool before making it reusable
(`S/record_serving/residency/frame_ports.rs:51`); reject stale allocation epochs
on every page read. Coordinate invalidation and allocation so the pool's
coordinate-based key cannot return an earlier page at a reused address
(`S/record_serving/residency/frame_loading/bounded_loader.rs:66`). A canceled
pre-WAL claim can return directly; a possibly WAL-exposed claim cannot. Recovery
reconciles its exact attempt first. Failed unselected batch allocations have
attempt custody, not semantic authoritative liveness; release them only after
reconciliation proves they were never selected and no obligation protects them.

### 1.13 Crash behavior at every batch edge

Recovery admits the complete mutation and publishes its root atomically; it
never promotes a prefix of page-image frames. A WAL group is a grouping/barrier
mechanism, not by itself a proof that a series of separately rooted appends is
one transaction (`S/durability/wal/port/group.rs:19`, `:273`). The one-member
terminal descriptor, closure digest, expected basis, and one directory advance
are necessary. Extend the existing typed operation sum
(`F/recovery_projection/operation.rs:73`) and independent projection validation
(`P/redo_replay/plan/projection_validation/blob_semantic.rs:140`) to tree batches.

| Crash edge | Recovered selection and required fate |
| --- | --- |
| Before admission, after scratch admission, after allocation reservation, before first WAL attempt | Old root; zero selected new pages. Proven no-effect permits releasing claims. |
| Before/after each image frame, before/after the terminal descriptor, before WAL durability | Old or new only if the complete member actually reached durability; an incomplete member cannot select any page. The caller has an indeterminate attempt once WAL was attempted. |
| After complete WAL durability, before any data image | New root after exact redo of all images and the directory. |
| Before/after each data image and data settlement | New root after redo; partial data is never serving truth. |
| Before/after candidate metadata writes and synchronization | New root after redo; unselected candidate files are not a second head. |
| Before/after root selector replacement, namespace sync, and current-root installation | New root, exactly once; replay completes the same durable transition. |
| After installation, before/after acknowledgment | New root; retry/fate answers the same terminal attempt rather than applying the batch twice. |
| Before/after checkpoint cutover and WAL pruning | Checkpoint plus bounded tail recovers the same directory, roots, values, and allocation custody. |
| Before/after retirement Intent, range-release publication, and Completion | No protected page is reused; recovery resumes the exact retirement, never repeats a free as a new allocation. |

Run each numbered frame/image edge, not just the first and last. Hash-valid
missing/duplicated/reordered images, changed family basis, and directory/closure
substitution refuse recovery before serving. Current checkpoint selection binds
root generation and tree identity (`S/durability/checkpoint/capture/publication_cutover.rs:75`);
preserve that binding when the directory becomes general.

## 2. What the roadmap gets wrong or leaves missing

- **“Keys are fixed width” describes the registered engine, not its codec.**
  `F/btree_node/node.rs:15` stores byte vectors; `F/btree_node/slotted.rs:46`
  accepts varying nonempty lengths. Registry `cell_shape` and callers impose
  fixed widths. D.3 must change those boundaries, not merely add a new codec.
- **Variable values partially exist.** The node codec already stores varying
  values (`F/btree_node/codec.rs:47`), but rejects empty values and caps lengths
  at `u16` (`F/btree_node/slotted.rs:49`). It has no large-value closure or
  streaming put. Removing the shape tuple alone does not supply bulk columns.
- **Sibling links do not currently route scans.** The parent stack already
  supplies traversal (`S/layout/btree/scan.rs:206`). Their removal is a format
  simplification and a guarantee for shared roots, not a repair to a demonstrated
  sibling-following bug.
- **Per-node root advances are real, but batching alone does not kill the
  serialization regression.** `S/layout/maintenance/append.rs:59` explicitly
  describes each append advancing the root; the global exclusive pending gate
  remains independently blocking (`S/durability/retention/admission.rs:105`).
  D.3 includes scoped pending admission and disjoint delta rebasing.
- **“Computed from page size, key cap and minimum fanout” is incomplete.** An
  absolute height also needs an addressable-page/byte ceiling. Section 1.7
  declares it and accounts for the root's smaller fanout.
- **The existing directory cannot already publish arbitrary families.** It
  accepts only BlobCatalog and DedupeIndex tags and record roots
  (`F/derived_family_root_directory.rs:222`). Its codec, binding, typed WAL
  operation, selected-content validation, checkpoint and rejoin must cut over.
  Scoped rebase also needs a last-changed Store-generation stamp to detect ABA
  when a family returns to an empty root; root identity alone cannot do that.
- **Page-granular free requires more than a new node header.** Current nodes
  are inline-only; exact arena range retirement exists, but requires a new
  page case and pool invalidation. Whole-tree retirement collection and whole
  free-list reconstruction also conflict with larger-than-memory operation.
  Retaining every output metadata frame during repack is another such limit
  (`S/record_serving/planning/free_space_routing/successor/repack.rs:63`).
- **“One WAL group” does not mean arbitrary members are indivisible truth.**
  Partial group append has a continuation today (`S/durability/wal/port/group.rs:40`).
  The batch needs one complete, validated mutation member and terminal descriptor.
- **D.3 and D.11 retention are different promises.** Page allocations can be
  freed individually; authoritative selected pages still cannot be reclaimed
  before D.11. D.4's multiplicity fence is not available in D.3.

These are corrections requested by this note. The roadmap file is not edited
here. C.11's single-line record adoption remains design reference, not the
implementation sequence: it deliberately appended chunk and node records through
one root, then retired displaced generations
(`plans/worth-store/physical-reconstruction-c11-layout-index-and-native-blob-adoption.md:687`).
Finishing its remaining custody work before multi-root storage would bake that
assumption in again, as the replacement roadmap's “Why The Path Changed” states.

## 3. Bug classes and contracts

Use existing proof machinery where it exists. `AuthorityMarker` is an open trait;
`AuthorityWitness::from_authority_marker` is only as sealed as the marker's
constructor (`Proof/proof/witnesses.rs:3`, `:26`). A generic marker bound on a
governed Store surface is therefore not authorization. The Store manifest already
depends on worth-proof and worth-foundational
(`workspaces/worth-store/crates/worth-store/Cargo.toml:44`).

There is no existing general tree-mutation authority in worth-proof's exports
(`Proof/lib.rs:77`, `:92`). **Add one reusable concrete admitted authority there,
not a Store-local marker per method.** `AdmittedTreeMutationAuthority` proves
legal submission for an installed owner occurrence, admitted family scope,
incarnation, expected root basis, and batch digest. Its fields are private and it
is not Clone. An occurrence seal follows the existing exact-owner contract
(`Proof/source_observation.rs:7`, `:75`): creating a foreign issuer with identical
descriptive bytes cannot satisfy the installed owner. The runtime retains the
issuer privately and issues admission only after its family/lifecycle checks;
no public facade exposes the installed issuer. Counters, live reservation tables,
revocation, and Drop remain in Store. D.4 and D.6 reuse this contract for root
submission; neither invents a weaker token. Reconstruction must re-admit through
its own current authority rather than deserialize this proof.

| Recurring bug class | Contract and home |
| --- | --- |
| Oversized/foreign key, comparator drift | One `TreeKey` admission bound to the family; bytewise order and persisted key cap, under `F/tree/key/` and `S/tree/admission/`. Decoder and all batch entry points share it. |
| Unsorted or duplicate batch | `CanonicalUniqueVec<TreeKey>` plus bounded operation arity, under `S/tree/mutation/admission/`; existing constructor rejects non-increasing keys (`Proof/collections/canonical_unique_vec.rs:12`). No derived `Ord` over `(key, operation)` can weaken key uniqueness. |
| Derived bytes promoted to authority; counterfeit writer | Class-typed handles and concrete `AdmittedTreeMutationAuthority`; exact installed-owner occurrence, scope, incarnation, and digest checks. Sealed marker authoring is available at `Proof/proof/marker_authoring.rs:44`; generic `AuthorityMarker` is never the public requirement. Rebuild admits only Derived. |
| Height drift, illegal split, underflow hidden by fixed widths | One admitted geometry and one normalization algorithm in `F/tree/geometry/` and `S/tree/mutation/rebalance/`. All phases consume it; integrity rejects malformed levels/occupancy. Independent ordered-map and occupancy evidence falsify it. |
| A shared path copied per key; hidden scalar submission | One sorted traversal plan and one final page emitter in `S/tree/mutation/`; exact `C+S-M-E+G` output counts. Remove scalar continuation/append entry points rather than document batching as advice. |
| Reservation spent twice or dropped after possible effect | `LinearResource` carried inside Store's allocation/batch lifecycle, not copied proof (`Proof/linear.rs:45`, `:84`, `:99`). Owner Drop releases pre-effect resources or retains an unresolved obligation. Rust does not require termination; the substrate explicitly leaves leak detection to the runtime (`:18`). |
| Permission mistaken for performed durability/publication | `Performed<Action, ConcreteAuthority, Outcome>` recorded after the successful effect and consumed by the next phase; it is not Clone (`Proof/effect/performed.rs:39`, `:99`). Extend existing WAL/data/root phase artifacts, not a second progression tower. A constructed descriptor or admission cannot stand in for barrier completion. |
| False no-effect after a WAL attempt; failure loses recovery posture | Existing `TransitionOutcome` categories (`Proof/transition/outcomes.rs:24`) and mutation terminal fates, with linear continuation/attempt custody. Pre-effect denial and post-attempt indeterminate stay separate in `S/tree/mutation/` and existing durability owners. |
| Lost update, a global submission lock, stale basis or empty-root ABA accepted | Admitted scope plus stamped expected-family basis; only final publication consumes its actual Store transition. Rebase disjoint deltas, reject same-family conflicts, retain unsettled scope. Home: `S/durability/retention/` and current-root owner, not tree codec or branch labels. |
| Half batch, mismatched directory, omitted large-value page | One member's terminal count/digest and complete root/value closure; one namespace publication. Home: `F/recovery_projection/`, WAL owner, integrity ingress, and `P/redo_replay/`. Independently parsed media must agree. |
| Live page reclaimed; reused-address cache hit | Exact range/epoch retirement with existing root, reader, checkpoint and WAL guards, pool invalidation before reuse. Authoritative retirement is structurally unavailable until D.11; D.4 adds its multiplicity fence. |
| Whole tree/family/value materialized | Budgeted changed-path scratch, streaming WAL/value frames, paged free-space access and retirement quanta. Measured resident bytes and retained descriptors are bounded separately; no full-family `Vec` enters an ordinary port. |
| Unsupported bytes served or silently reset | Sole current version admission before effects, coordinated independent parsers; no fallback init. Home: physical format, bootstrap, recovery, integrity and observer. |

New types must protect these recurring classes across admission, ordinary
mutation, recovery, and successor consumers. Do not add a witness to guard one
conditional or certify a single call site. QA focuses on authority substitution,
partial WAL/data effects, concurrent directory changes, and resident/retained
memory; it does not count tests as proof of these properties.

## 4. Destination and successor handoffs

The stable axes are physical meaning in Format, operational tree lifecycle in
Store, integrity admission, and cert-only reconstruction. The facade exports
capabilities and delegates; it implements no algorithm. Proposed paths below
are destinations, not empty placeholders to create now.

```text
worth-store-physical-format/src/
  tree/                             [new: replaces btree_node/]
    key/{mod.rs, admission.rs}      [bounded physical key meaning]
    geometry/{mod.rs, occupancy.rs, height.rs}
    node/{mod.rs, header.rs, cells.rs, codec.rs}
    value/{mod.rs, chunk.rs, index.rs, descriptor.rs}
  family_root_directory/            [replaces derived_family_root_directory.rs]
    mod.rs, descriptor.rs, codec.rs
  record_framing/                    [existing: one envelope owner]
  recovery_projection/               [existing: batch operation and streaming codec]
worth-store/src/physical_runtime/
  artifact_family/                   [existing: open admission and class inventory]
  tree/                             [new: replaces generic behavior in layout/]
    facade.rs, mod.rs
    admission/{family.rs, root_basis.rs}
    read/{point.rs, range.rs, page_port.rs}
    mutation/
      admission/{batch.rs, authority.rs}
      traversal.rs, page_emission.rs, publication.rs
      rebalance/{split.rs, underflow.rs, separators.rs}
    value/{write.rs, read.rs, frontier.rs}
    retirement/{removed_pages.rs, manifest.rs}
  layout/                           [existing: derived access/rebuild orchestration only]
  blob/                             [existing: catalog/dedupe meaning, policy, encoders]
  record_serving/arena/              [existing: allocation, paged free ranges, claims]
  durability/{wal/, publication/, retention/, checkpoint/} [existing effect owners]
worth-proof/src/
  tree_mutation_authority.rs         [new: reusable legality/occurrence contract]
worth-store/tests/
  physical_tree_journeys.rs          [new: one integration target]
  physical_tree_journeys/            [batch, occupancy, value, concurrency, retirement]
worth-store-recovery-runtime/tests/
  tree_batch_recovery.rs             [new: one fresh-process target]
```

Integrity validation and the independent observer gain tree-node/value/directory
children under their existing family boundaries, replacing their old BTreeNode
parsers. Recovery physics and runtime extend their existing operation/ingress
owners. They do not import ordinary Store internals. Every code/test file stays
within 400 lines; no exemption is requested. Visibility, facade snapshots and
boundary-check preserve the direction. Forbidden destinations include a second
`general_btree`, a blob-value backend, public raw node append, and catch-all
utility modules.

### D.4: root table, zero-copy fork, atomic multi-root publication

The format gives D.4 immutable node/value closures, no sibling links, class and
layout tags, nullable empty roots, and an expected-basis batch artifact. D.4 can
share a family's root without copying any node or value pages, place family sets
in its root table, and compose several batch deltas in one WAL group and one
namespace publication. It must remove the remaining store-global directory
selection as the ordinary branch root source. No D.3 node contains branch names
or a per-root generation, so D.4 adds descriptors without rewriting nodes.

“Fork writes zero pages” must mean zero **data-tree/value pages**. Persisting a
new root name still writes WAL/root-table metadata. Otherwise D.4's default
page-backed root table contradicts its own zero-write acceptance. D.3 supplies
zero data copying, and section 7 requests that counting clarification.

### D.5: bounded cursors and one pin per level

The format gives ordered separators and direct selected child references, one
page per node, independently bounded value pages, and a carried height. Parent
stack traversal needs at most one pin per data-tree level; value range descent
has its separately charged stack and chunk window. No sibling walk or whole
value materialization is required. D.5 owns the scoped/non-Send cursor contract,
root lease across pauses, key-based resume, work charging, and per-owner routing
handle table. D.3 does not claim exactly one total cold fault per level while
C.5 routing metadata still faults; it reports those faults separately.

### D.11: reclamation across branches

The format gives stable allocation identities, independently releasable ranges,
and an enumerable directed closure: interior child pages, leaf external values,
value-index pages, and chunks. Its mark traversal starts from family descriptors
in every root, retained checkpoints, and reader leases. Sharing is represented
only by references; there is no mutable refcount in a page to repair after a
crash. D.11's generation-set mark proof authorizes free, and a later lease blocks
it. Until then, D.4 must fence any multiply rooted family from retirement, even
if its class is Derived. D.3 never supplies a single-root inference as that proof.

## 5. Slices

Every slice lands green and deletes the replaced path when its replacement
becomes operational. The proof and streaming work are prerequisites, not
temporary backends. Use one implementer and one fresh independent reviewer per
slice as the roadmap requires; this note does not execute that implementation.

**D.3.1 — fast iteration: split `production_entry` and Phase 8.** Already in progress; keep focused groups under five minutes.

**D.3.2 — key, family, geometry and authority admission.** Crates:
worth-proof, worth-store-physical-format, worth-store. Establish the concrete
reusable authority, sole bounded key constructor, open declarations, class-typed
handles and height derivation. Replace `RegisteredDerivedFamily` and its static
two-row-only installation in `S/artifact_family/registry.rs`; remove engine
fixed-width admission and the three height constants in the same slice. Existing
built-in encoders remain consumers of the admitted general vocabulary. No new
authoritative execution port is exposed before D.3.4. Focused commands:

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-physical-format --lib d3_tree_admission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --lib d3_family_admission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_runtime_authority_ui tree_mutation
```

worth-proof belongs to the root workspace; run its focused contract proof there:
`cargo test --manifest-path Cargo.toml -p worth-proof --lib tree_mutation_authority`.
Do not pretend it is a Store workspace member. Consolidate positive and negative
UI cases in the existing target (`workspaces/worth-store/crates/worth-store/Cargo.toml:93`).

**D.3.3 — sorted mutation and byte rebalance.** Crates: worth-store and
worth-store-physical-format. Replace `S/layout/maintenance/tree/insertion.rs`
and its recursive scalar topology editing with one batch traversal/normalizer;
remove cell-count midpoint splits. Keep the existing physical effect owner until
the coordinated page cutover; do not install a model as a backend. Local algorithm
tests protect shared-path normalization, missing deletes, separator changes,
root contraction and occupancy. The authoritative facade remains unavailable.

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --lib d3_batch_rebalance
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-physical-format --lib d3_height_geometry
```

**D.3.4 — atomic page and streaming-value cutover.** Crates: worth-store,
worth-store-physical-format, worth-store-physical-integrity, worth-store-wal,
worth-store-recovery-physics, worth-store-recovery-runtime,
worth-store-offline-integrity-observer, worth-store-offline-verifier. Change the
envelope, dedicated arena page placement, general directory and manifests, one
streaming batch member, data settlement, root publication, checkpoint/rejoin and
all independent decoders together. Land large values here: a whole-value Vec is
not an intermediate implementation. Wire both derived consumers and the complete
put/delete engine through the one general batch entry. Delete `F/btree_node/`,
the derived-only directory spelling, `S/layout/maintenance/append.rs`, scalar
continuations, and protected single-node/directory append methods. Replace their
consumers; remove historical golden/version readers, preserving current-operation
semantic and corruption cases. Expose authoritative tree execution only now.

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-physical-format --lib d3_tree_page_format
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-physical-integrity --test page_validation d3_tree_page
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_atomic_pages
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_large_value
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-physics --lib d3_tree_batch_redo
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_current_format
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-offline-integrity-observer --lib d3_tree_pages
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-wal --lib d3_streamed_member
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-offline-verifier --lib d3_tree_batch
```

**D.3.5 — scoped submission and disjoint directory rebase.** Crates:
worth-store, worth-store-physical-format, worth-store-recovery-physics,
worth-store-recovery-runtime. Replace `register_exclusive_pending` in
`S/durability/retention/admission.rs` and whole-source-only rebase in
`S/record_serving/planning/rebased_root.rs`; delete their all-submission conflict
assumptions and tests. Admission carries a bounded physical scope. Keep final
namespace progression exclusive. Prove no second preparation after another
family advances, preserve same-family conflict and unresolved scope, and replay
the actual durable publication order. Retain an expected-empty token, insert and
delete the last key, then submit with that old token: it must be Stale with zero
page allocations and WAL writes, including after reopen.

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_disjoint_submission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --lib d3_scope_admission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_disjoint_rebase
```

**D.3.6 — bounded free-space and single-line retirement.** Crates:
worth-store, worth-store-buffer-pool, worth-store-physical-format,
worth-store-recovery-physics, worth-store-recovery-runtime. Replace the
whole-free-list restore/index requirement in
`S/record_serving/arena/{reconstruction.rs,free_ranges.rs}` with bounded
membership access. Replace whole selected-tree retirement vectors and five-level
replacement arrays with bounded manifests and the carried delta. Add exact page
range/epoch retirement, pool invalidation and recovery. Delete the old whole-tree
ordinary retirement collector, whole-output metadata frame retention, and the
ordinary full-repack fallback in `S/record_serving/planning/free_space_routing/successor/repack.rs`.
Authoritative selected-page retirement stays
unavailable; preserve all current blob release/retirement semantics.

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_page_retirement
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --lib d3_paged_free_space
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-buffer-pool --lib d3_reused_page
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_retirement
```

**D.3.7 — acceptance, derived parity and documentation closure.** Crates:
worth-store, worth-store-recovery-runtime, integrity and observer owners. Add the
seeded oracle and exact counters below to the two intentional journey targets;
remove obsolete fixed-height/layout fixtures and duplicate scalar-path evidence.
Preserve built-in behavioral journeys, including rebuild and quarantine. Revise
`workspaces/worth-store/crates/worth-store/README.md` and `docs/api.md`'s internal
Store surface for tree declarations, batch outcomes, streaming values, budgets,
and current-only refusal. Revise existing Store format/observer documentation for
page geometry, retirement and coverage. Retire C.11 text that claims the replaced
derived-only operational path; the database roadmap is the sequencing authority.
Compile public examples through the real facade and keep boundary snapshots and
generated AGENT_CONTEXT consistent through tooling, never hand edits.

```text
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_differential_seed_0
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_write_counts
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_tree_journeys d3_resident_bound
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_crash_before_checkpoint_inline_admission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_crash_after_checkpoint_inline_admission
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_crash_before_checkpoint_large_value_image_0000_before
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-recovery-runtime --test tree_batch_recovery d3_crash_after_checkpoint_large_value_image_0000_before
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_blob_journeys layout_catalog
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_blob_journeys layout_dedupe
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_blob_journeys layout_range
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --features certification-test-authority --test physical_blob_journeys layout_rebuild
cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store --doc tree
```

Every planned filter must list at least its intended positive test before its
execution is accepted. A zero-test green is no evidence. Differential cases are
separate seed filters (0, 1 and 2), each with 256 batches of at most 16 edits.
Crash cases are single-edge filters: use the exact two recovery command forms
above, changing only the named shape, numbered edge, and before/after suffix.
For example image 0001 after is
`d3_crash_before_checkpoint_large_value_image_0001_after`. Run the complete
generated edge list at slice closeout; do not put thousands of fresh-process
crashes behind one iteration filter. If even one edge exceeds five minutes,
fix its fixture/harness before accepting the slice; never drop the edge or claim
an unmeasured runtime. No iteration runs unfiltered production_entry or Phase 8.

At each implementation slice, format the touched packages with
`cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p <crate> -- --check`
(substitute each named package), run `bash scripts/ci/check_workspace_rust_line_caps.sh dirty`,
then the required boundary commands:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

D.3.4 and D.3.6 each run the affected C.7/C.8 process, checkpoint, integrity and
retirement matrices once as their heavy closeout lane, split by D.3.1's targets.
D.3.7 runs the built-in derived-family suites once and all D.3 acceptance shards.
Exact revised heavy target names are unverified while D.3.1 is in progress;
its green target/filter inventory is a prerequisite to those implementation
gates. The superseded C.11 full heavy campaign remains deferred to S.12 under
this roadmap; it is not a substitute for the affected recovery checks here.

## 6. Acceptance mapped to D.3

The production subject is Store opened on qualified file media, using the real
tree entry, mutation owner, pool and fresh-process recovery. Expected contents
come from an external ordered map of byte keys to bytes. An independent media
walk checks occupancy, child ranges, family identity and large-value digests.
Neither the model nor the observer grants mutation authority.

### Seeded differential oracle

Use reproducible seeds 0, 1 and 2, each with 256 random batches of at most 16 edits,
including empty keys/values, absent deletes, replacements of changing widths,
maximum keys, adjacent delete runs, left/right redistribution, cascading merges,
root splits/contraction, and large values. Check the map after every batch through
point and bounded range reads. Add deterministic occupancy-boundary cases so a
seed cannot miss a split/merge class. Retained old root leases continue to read
their earlier map while a successor is published. This is single-line retained
reading in D.3, not a test-only branch implementation.

### Exact page-write counts

Derive expected counts from the **pre-batch independently decoded topology and
operations**, before observing emitted pages. For one effective family batch:

```text
W_nodes = C + S - M - E + G
W_values = sum over external puts v of [ceil(L_v / (P-144)) + J(ceil(L_v / (P-144)))]
W_directory = 1
W_tree_data = W_nodes + W_values + W_directory
```

`C` is the number of distinct changed old nodes, including rebalance neighbors;
an ancestor shared by many keys counts once. `S` counts additional outputs from
splits/repartition (one for a two-way split), `M` counts outputs removed by merges
(one for two inputs becoming one), `E` counts empty-node removal or elided old
roots not already counted in `M`, and `G` counts new nodes with no old counterpart,
such as an initial leaf or a new higher root. Keep categories disjoint. The
all-unchanged case has `W_tree_data = 0`, not one directory page. Old value pages
are retained/reclaimed by their lifecycle and never rewritten on delete.

Assert these consequences explicitly:

- Updating any number of admitted keys in one leaf with no structural change in
  a height-`h` tree: `C=h`, so exactly `h+1` tree-data pages.
- Updating two leaves whose paths share `a` ancestors, without rebalance:
  `C=2h-a`, so exactly `2h-a+1` tree-data pages.
- A root-leaf split: `C=1,S=1,G=1`, so three node pages plus one directory page.
- A merge with root contraction: subtract one output for the merge and one for
  the elided root; never write the discarded root or merged-away page.
- Deleting the last key: `C=1,E=1`, so zero node pages and one empty-root
  directory page. A second delete writes zero pages.
- A 64 MiB external value at 16 KiB pages: `C_payload=16,240`, `n=4,133`,
  `b=225`, `J=19+1=20`, so exactly 4,153 value pages, plus the separately derived
  changed-node and directory counts. No whole 64 MiB buffer is admitted.

Count physical metadata too. Define `W_meta` as the exact number of rewritten
routing, membership and free-space **page frames** in the independently decoded
publication footprint; then `W_pages_total = W_tree_data + W_meta`. Report
non-page root/selector/manifest bytes and calls separately, and assert their
exact planned counts, one batch member, one family root selection, one Store
generation advance, and one namespace publication. Do not call `W_nodes` the
whole-Store I/O count. The current routing platform has a distinct merge and
publication footprint (`S/record_serving/planning/settled_root_projection.rs:12`);
its metadata cannot disappear from measurement. For no-split/path-sharing
fixtures, calculate `W_meta` from fixed block geometry and changed membership
paths, not from the writer's own count. Failed/retried and maintenance writes
have separate counters and their own exact edge expectations.

### Crash at every batch edge

Use the matrix in section 1.13 for no-split update, split, merge/root contraction,
large-value replacement, and derived-directory update. Crash in distinct writer
processes at each numbered frame/image edge, then reopen in another process.
Before complete durable admission the selected map is exactly old or exactly
new; after durable admission it is exactly new. No mixture is an allowed model
state. Compare directory, family root, values, generation, fate and allocation
custody; perform a same-token retry. Repeat before the first checkpoint, above a
checkpoint, and after lawful tail pruning. Independently remove one image or
alter the terminal descriptor with valid outer integrity: recovery must deny,
not expose a prefix. The existing selected-operation payload validation is a
predecessor guarantee (`P/redo_replay/plan/projection_validation/blob_semantic.rs:140`).

### Height bound at maximum key

At every supported page size with `K=1,024`, prove all emitted non-root nodes
occupy `[T,U]`, interior fanout is at least derived `f`, and every observed height
is at most carried `H`. Independently verify integer-bound arithmetic at ceiling,
overflow and tiny-capacity cases; allocation beyond `B` is a zero-effect denial.
Keys of `K+1` are denied at construction with zero allocations and zero WAL
writes. Cycle/level corruption is rejected independently of height exhaustion.

### Derived-family suites remain green; larger than memory

Keep `T/physical_blob_journeys/layout_catalog.rs:18`,
`layout_dedupe.rs:42`, `layout_range.rs:14`, and production split/rebuild/corruption,
reuse-claim, quarantine and release coverage through the new engine. The
fixed-height fixtures cannot serve as evidence for the new bound. Deleting all
derived pages and rebuilding from selected blob authority restores identical
results; deleting authoritative pages gives corruption, never rebuild success.

At least ten times as many data pages as resident frames, and a streamed value
at least four times the entire admitted operation memory budget, must pass exact
reads and crash/reopen. Count node faults, value faults and routing faults
separately. At every progression edge, resident frames are at most the configured
pool limit; transient and retained operation memory stay within their separately
admitted envelopes. No proof requires every free range or family key resident.
Use a 64-frame, 16-KiB pool (1 MiB) and a separately declared 4-MiB operation
memory budget, at least 640 data pages, and the 64-MiB value fixture above. Admit
a 128-MiB WAL segment/group byte ceiling and retained-tail headroom sufficient
for its page images and metadata; count that disk retention separately from
resident memory. Exhausting a smaller declared group is a pre-effect refusal,
never permission to materialize or split the value into independent commits.
For a cold point fixture with warmed routing metadata, assert exactly `h` node
faults. D.5 owns the later cold-reopen claim of one total data page per level and
its routing-cost elimination. Timing and a timeout do not prove these bounds.

### QA considerations

Review authority sealing and owner occurrence through the actual open and
submission surfaces. Review WAL ordering, checkpoint replacement, uncertain
allocation claims, and reused-address invalidation together. The external model
and independent media parser must expose missing images, a stale directory,
underfull nodes, a full-value buffer, and repeated shared-path writes. Run focused
proofs on qualified file media with speculation disabled for exact cold-fault
fixtures; test coalescing/speculation separately without transferring those
counts. Treat local format refusals separately from filesystem qualification
denials. Required format and recovery cutover checks remain gates, even when
unsupported historical versions correctly refuse.

The serialization acceptance in section 1.9 is additional evidence required to
make the roadmap's claimed regression removal true. B reaches Terminal while A
is held at the named preparation edge; A and B each advance once, neither loses
the other's directory entry, and A's preparation count stays one.
The empty-root ABA negative case must reject the earlier stamped basis, while a
disjoint family's unchanged stamp still permits rebasing.

## 7. Owner decisions

- **Public key-cap envelope and inline size:** recommend the locked 1,024-byte
  maximum key, ordinary declared cap 256, and 1,024-byte inline value limit.
  These are product capacity choices, not architecture left to an implementer.
  A larger supported key cap needs a newly proven cell/fanout contract.
- **D.4's zero-page fork wording:** recommend “zero node/value page writes,” with
  root-table and WAL metadata counted separately. Literal zero total durable
  writes cannot persist a new root name in the proposed table.

Neither decision authorizes a compatibility backend or weakens crash behavior.
All other architecture here is decided by the governing laws and inspected code.
