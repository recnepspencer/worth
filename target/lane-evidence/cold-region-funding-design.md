# Cold Full materialization funding design

Status: boundary review and implementation plan; no storage source edited yet.
Governing decision: coordinator Ruling 1, request-funded cold DATA, same path
for every Full load. No borrowed prefix-preserving emitter, format change,
canonical replay, new feature, or maintenance exemption.

## Boundary facts

`cell/demand.rs::raw_full` routes both backed and owned-Head Full demands to
`region_load::load_full` or `ResidentHistoryReadmission::load_full`.
Both return ResidentRegion with `_memory: None`. HistoryFolder caches decoded
RegionHistoryImage, builds chunk stacks and clones earlier metadata. Its
`charge_output` only limits serialized size. The ordinary region encoder is
not a funding substitute. Original Full uses its existing admission unchanged.

The materialized partition is DATA; root serving authority remains in the
existing root/kind admission. The emitter must use the exact counted Full
lease on the same cell. No lookup of another root, fresh basis or currentness.

## Owner changes and charging points

| Granted file | Change |
|---|---|
| region_load.rs | Before decode/clone/rebuild, admit typed transient and retained storage; charge byte/row/edge reconstruction and verification. Keep retained reservation in ResidentRegion._memory through last Arc. |
| resident_history.rs | Same funding for owned Head plus retained prefixes; quote current state before cloning, and folded prior rows before appending. No separate capture-only loader. |
| history_fold.rs | Fund member read buffers, cache entries, decoded images, directory shares, row/chunk/result vectors before growth. Charge authentication, decoding, visits and copies. Keep input and expanded-output read limits as independent safety bounds. Transfer only final backing charge; scratch releases with folder. |
| cell/resident_leases.rs | Admit backed Full after successful funded Full materialization. Retain original-Full rule; Head serving guard unchanged. Count leases exactly as today. |
| checkpoint_capture/complete.rs | Apply reviewed same-cell DATA-emitter extraction. Demand ordinary funded Full if not resident, using existing region read ceiling; map request denials without damage. Invoke emitter on actual counted lease. |
| region_capture/complete.rs | Preserve one prefix-free funded emitter and existing request-funded immutable outputs. Document Full materialization prerequisite; no alternate wire/output path. |

Funding must precede each allocation, including malformed input with large
claimed lengths. A serialized-byte multiplier is not an allocation quote.
Typed preflight accounts concrete Vec element capacities and relocation,
Arc/control layouts, B-tree nodes, shared columns/maps, metadata/extra values,
portable aspect strings/collections, adjacency, commitment and verification
scratch. Reuse existing canonical aspect allocation vocabulary where its
visibility and grammar permit. Quote compact REG6 sequences, not the named-map
canonical-envelope grammar. Keep checked arithmetic and classified request
stops. No live state publication on a failed quote/load; retries remain lawful.

## Required placement extension (not yet granted)

The six files are integration owners, not suitable containers for the whole
concrete allocation preflight. region_load.rs is 300 physical lines and
history_fold.rs 285 physical lines (`len(read_text().splitlines())`).
Existing nearest analogue is durability/derived_index_artifacts/
generation_decode_quote.rs plus entries.rs; canonical decoded_quote similarly
separates concrete owner families. There is no existing region quote found by
`rg -n 'allocation_bound|clone.*request|reserve_retained_memory'` in the owning
residency and checkpoint-image paths that covers these reconstruction owners.

Request these exact new private files, registered only from already-granted
region_load.rs using an explicit path (no residency/mod.rs edit):
- region_load/materialization_funding.rs: reservation lifecycle and checked
  retained/transient/work plans shared by both Full loaders and HistoryFolder.
- region_load/materialization_funding/region_image.rs: concrete compact REG6
  Head/History and arena/metadata/extra/adjacency allocation preflight.
- region_load/materialization_funding/aspect_state.rs: portable aspect nested
  ownership and native readmission storage preflight.
- region_load/materialization_funding/tests.rs: denial-before-allocation,
  retained-until-last-owner, refund/retry and work-charge controls.

All remain owning-runtime private; no public facade or boundary expansion.
If concrete quote families exceed 400 lines, request the exact additional leaf
before writing it. Do not put unchecked post-allocation inventory accounting
in a loader to fit the six-file list.

## Cost contract

S is captured selected region DATA, including intrinsic retained record
metadata history, columns, aspect values, relation memberships, and required
commitment storage. T is the bounded postselection prefix/layer suffix needed
by the region. Each physical history member is decoded/authenticated once per
folder; every emitted row occurrence belongs to S. Charge actual copying and
verification, with ordered-container navigation separately accounted.
No covered canonical bodies or historical canonical replay occur.
Materialization work and peak/retained memory are O(S+T), plus the existing
ordered-index navigation term; the bound does not claim arbitrary B-tree
construction is linear. Existing consolidation bounds restart layer depth.

## Implementation order and proof plan

1. Resolve exact private-file placement extension and independently review this
   design before source changes. Implement concrete typed preflight first.
2. Cut both Full loaders onto it; retain reservation on ResidentRegion. Charge
   HistoryFolder cache/fold work before each allocating step.
3. Admit funded backed Full and apply same-cell DATA emission. Keep selected
   Embedded refusal and actual-membership checks unchanged.
4. Compile affected owner/courts once, then selected tests only: Native three
   previously failing second-Complete/retry/index cases, Host killed image+
   tail and unchanged <=40 bound, cold zero-covered-read court, Store observer
   recovery. Add direct memory/work denial/refund controls through the owner.
5. Existing CDC, ACK, refusal and recovery selections; Clippy, formatting, dirty
   line caps, boundary and generated context at planned phase completion.
   Independent adversarial review before funding commit.

Any fixture crossing a budget reports actual measured charge and ceiling;
no silent fixture-budget changes. Explicit callers and Host upkeep use the
same loader; their existing request origins differ, never their accounting.

## Boundary-review corrections before implementation

A Full demand from Cold first calls load_head. Its retained raw_head cache is
part of the materialization prerequisite, so region_load.rs must fund Head
read/decode/reconstruction and keep that reservation on the Head ResidentRegion
as well; charging only load_full would leave actual retained memory unfunded.

HistoryFolder::read returns an Arc still used after drop(folder). Replace its
private returned/cache image owner with a private funded image backing (image
and its reservation together, dereferenced as DATA). No reservation held only
by the folder may fund an escaping Arc. Chunk rows and final result-vector
reservations remain live through copying into the admitted Full result; a
request-owned aggregate ticket may resize only before growth, never after.
The retained Full quote is separate from simultaneous Head/image/fold scratch.

## Settled funding owner signatures (revision after independent rejection)

```rust
// Owner-private runtime custody, defined in history_fold.rs, no facade.
struct FundedHistoryImage {
    image: RegionHistoryImage,
    _memory: Option<ExecutionMemoryReservation>,
}
struct FundedRows<M> {
    rows: Vec<Vec<M>>,
    _memory: Option<ExecutionMemoryReservation>,
}
// Owner-private region_load/materialization_funding.rs.
struct MaterializationFunding {
    retained: Option<ExecutionMemoryReservation>,
    transient: Option<ExecutionMemoryReservation>,
}
```

`read` and its cache return/hold Arc<FundedHistoryImage>. Neither image nor
its ticket is separated when the folder drops. Each fold result returns its
actual backing with its ticket. The loader destructures a result into rows
and ticket in the same scope, retaining the latter until after the admitted
native reconstruction consumes/drops every wire row. The final Full ticket
is admitted before that reconstruction and moves into ResidentRegion._memory;
the old wire ticket drops only after wire-to-native conversion returns.
Head gets an independently admitted ticket before decode/rebuild and retains
it in its own ResidentRegion._memory, including its raw cache. Temporary
read-buffer/decode/commitment scratch scopes end only after the corresponding
actual owners drop. There is no ticket swap that temporarily leaves both old
and new DATA unfunded. Request-native no-accounting behavior remains exactly
ExecutionRequest::reserve_retained_memory's existing contract.

## Settled traversal/accounting correction

Replace HistoryFolder's uncharged BTreeMap with ExecutionOrderedMap for its
member cache. Replace CachedHistory.directories' linear Vec with a separately
funded ordered verification set keyed by (actual member locator, actual
directory allocation identity). Pointer identity only deduplicates this DATA
verification; it grants no authority and never sets canonical output order.
Each member is decoded once; each (member,directory) pair is verified once.
Verification collects actual direct bases into a request-funded ordered set,
compares them to the directory's borrowed bases, and does not call the
allocating uncharged direct_bases shortcut. Every row-prefix hop, cache lookup,
directory check, copied metadata value and byte hash/verification is charged
before its work/growth, including cache hits. Request stops leave the same
retryable cell. No missing-prefix default or depth-based new refusal.

For production Home materialization, S includes the complete retained record
state across changed selected regions and T includes their actual bounded
postselection region layers. REG6 capture emits each physical row's own
prefix (history_capture::split_rows), so a given physical row is followed at
most once per output row per layer. Production checkpoint capture consolidates
at HISTORY_CAPTURE_MAX_LAYERS=8 and HISTORY_CAPTURE_INPUT_TARGET, while Complete
emission itself is flat. Therefore row-prefix visits are bounded by the fixed
layer count times represented rows in S+T; repeated metadata copies are bounded
by the represented Full output S. The ordered cache and verification-set term
is accounted separately, not called linear. This relies on actual Home
producer provenance, not a codec rule inferred from acyclicity.

Arbitrary independently admitted deeper history graphs retain their existing
legality. Their general Full materialization charges actual prefix hops H and
verified directory pairs D, with work O(input+output+H+D) plus ordered
navigation. They are not claimed to satisfy the production Home's eight-layer
bound. No claim about all acyclic graphs follows from a single member decode.
The focused cold Home proofs must observe zero covered canonical bodies; the
existing discarded-history growth proof proves that independent canonical
history axis, not the region layer/alias axis.

## Final bound correction: member basis cache and verification volume

Store the once-derived actual direct-base set beside each FundedHistoryImage,
with its own included request reservation. Derive it once while visiting the
decoded member rows (charged); per-directory validation compares that cached
set to the directory's borrowed direct bases. No per-pair whole-image rescan.
Let V be total actual compared base entries across verified distinct directory
pairs. General Full work is O(input+output+H+V) plus ordered navigation, not
O(input+output+H+D) for D pairs treated as constant. Every compared entry is
charged; malformed/large graphs still stop by actual request limits.

The production selected Home proof is specifically Complete's prefix-free
REG6 emission (region_capture/complete.rs) and flat-only covered reuse
(checkpoint_complete/regions/covered.rs). Its postselection journal tail can
retain those actual flat base rows; it never supplies another canonical replay
chain. Flat members have no direct bases: their once-derived basis is empty,
so directory checks do not rescan their unrelated rows. Their row visits and
actual current suffix work belong to captured region state S and bounded tails
T. This is the primary Home argument. Ordinary standalone layered export's
8-layer consolidation policy is a separate policy, not loader validation or a
substitute for this Complete producer invariant.

## Test placement constraint

The new private tests.rs is only for pure allocation-quote/value logic; it
constructs no runtime, session, SerialRequest root or local provider adapter.
Memory retention/refund, work denial and retry observations belong to the
existing externally constructed relational certification courts under the
already owned source_initial_inputs/checkpoint_admission family (cost where
measuring growth). Do not add another BC9002 baseline entry for funding tests.

Existing funded Full commitment owner is `PartitionContentCommitment::capture_full_for_request`; use it rather than the current unbudgeted `capture(..., None)` calls. Its internal owned backing and request charges must not be charged a second time by the outer materialization ticket. Head-only commitment still needs its own explicit preflight/charge as part of Head admission.
