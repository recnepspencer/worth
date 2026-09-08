# C.9: Physical Integrity, Corruption Localization, And Offline Truth

## Goal

Install one canonical physical-integrity law for every persisted artifact that
C.5 through C.8 made real:

> Bytes obtained from media are untrusted until the integrity owner validates
> their exact artifact identity, format envelope, physical generation, covered
> range, and checksum before any ordinary or recovery interpretation can use
> them.

C.9 also installs two observational lanes over that law:

- a bounded online scrub that observes the live physical Store without
  acquiring repair or reachability authority; and
- an independent offline verifier that walks an inert Store root without
  importing the runtime validator, recovery planner, Store, or any repair
  surface.

C.9 is complete only when every current authoritative and derived C.5-C.8
artifact is either admitted before interpretation or rejected with the
smallest honest localization and blast radius; runtime and offline conclusions
can disagree explicitly; and neither integrity evidence, scrub output,
quarantine posture, nor an offline report can choose recovery sources, mutate
reachability, rebuild derived state, or reopen semantic service.

## Current Verification Policy

C.9 behavior is protected by direct Cargo tests, one production-valid
multi-process corruption scenario, consolidated compile-fail coverage, and
repository boundary gates. Generated proof ledgers, source bindings, mutation
catalogs, evidence bundles, and report pipelines are retired and must not be
recreated. Git preserves implementation history.

## Why This Milestone Exists

C.5 established canonical checksum-protected physical frames and artifacts.
C.6 established bounded residency and frame generations. C.7 joined WAL,
checkpoint, data, root publication, and acknowledgment. C.8 established
fresh-process recovery and a recovered-runtime handoff.

Those milestones make corruption consequential, but they do not yet create a
single compiler-visible admission boundary between persisted bytes and their
interpretation. Today, integrity code still points upward into `worth-store`,
the offline verifier consumes runtime integrity classification, and recovery
code can call physical-format checksum mechanisms directly. The controlled
defect registry below makes the plausible fakes concrete: post-decode checking,
scattered checksum authority, one-join-only admission, stale record reuse,
shared verification, optimistic rebuildability, quarantine mutation, dishonest
amplification, and surviving legacy routes.

C.9 is therefore an admission, authority, and independent-observation
milestone. It is not a new checksum algorithm, an offline repair tool, a
recovery-policy rewrite, or a semantic health service.

## Roadmap Placement And Inherited Truth

C.9 consumes:

- C.4's sole filesystem media owner, exact media operation identity, stable
  namespace, bounded I/O, and explicit indeterminate outcomes;
- C.5's canonical declarations actually used by current ordinary or recovery
  routes: page, extent, WAL, checkpoint, root, segment-membership, and
  free-space, including their checksum coverage and version rules; dormant
  segment-manifest, index, and blob codecs do not enter C.9 merely because
  reserved format or test residue exists;
- C.5.1's Store-owned physical operation identity, derived Signal readiness,
  bounded scheduler admission, executor-only media access, cancellation law,
  runtime generation, and exact effect settlement;
- C.6's admitted resident allocation, frame identity, frame generation,
  bounded residency, dirty typestate, and writeback settlement;
- C.7's canonical durability progression, current-root publication, WAL and
  checkpoint identities, and physical acknowledgment; and
- C.8's recovery source-precedence authority, bounded discovery, selected
  recovery plan, redo, publication, and `RecoveredPhysicalRuntimeHandoff`.

C.9 may consume those truths. It may not reconstruct them, replace them with
an integrity-shaped approximation, or make a checksum result authoritative
for a fact owned by an earlier milestone.

C.9 precedes C.10 because maintenance may schedule scrub only after scrub has
a bounded, non-authoritative contract. It precedes C.11 because each new
index, blob, and compaction artifact family must have an additive integrity
insertion point before the layout family expands.

## Governing Boundary

C.9 owns:

- pre-interpretation validation of the physical envelope and checksum coverage
  declared by the artifact's canonical format;
- binding successful validation to the exact Store, family, artifact identity,
  physical generation, byte range, format version, checksum coverage, and
  media or resident-frame generation that was inspected;
- typed rejection before owner-specific or logical decoding;
- the smallest honest physical damage localization and declared blast radius;
- integrity observation counters;
- bounded online scrub mechanics below Store-owned lifecycle and scheduling;
- non-authoritative quarantine observations; and
- a versioned, read-only offline integrity-observation protocol whose
  implementation is independent of runtime integrity mechanics.

C.9 does not own:

- authenticity, malicious-tamper resistance, encryption, or key custody;
- semantic MVCC visibility, record meaning, branch health, Query readmission,
  transaction retry, or conflict resolution;
- recovery source precedence, redo eligibility, or C.8 publication decisions;
- root or manifest reachability mutation;
- repair, replacement, deletion, salvage, release from quarantine, or
  compaction;
- the decision that a derived artifact is rebuildable; that decision belongs
  to the artifact owner and requires an intact authoritative rebuild basis;
- online maintenance policy, cadence, priority, or resource scheduling;
- format invention outside the canonical artifact owner; or
- semantic service availability.

A checksum match means only that the declared covered bytes match the declared
checksum mechanism for the exact inspected scope. It is not proof of origin,
freshness, semantic correctness, publication authority, recovery precedence,
or harmlessness.

## Adversarial Constraint

The canonical integrity path must survive this world:

> A real Store at least 32 times larger than its admitted resident-memory
> budget contains multiple data segments, inline and extent-backed records, a
> multi-level manifest, free-space metadata, a current and retainable previous
> root, a checkpoint, at least two WAL segments, and every derived index or
> blob artifact that has an ordinary production path at C.9 entry. The writer
> exits cleanly. An isolated artifact-editor process then applies exactly one
> declared corruption to a copied Store root: checksum bytes, framing length,
> generation, pointer, payload, truncation, removal, duplication, stale format
> version, or identity substitution. A fresh C.8 recovery process and a
> separate offline verifier process receive only the copied root, stable
> physical declarations, admitted budgets, and expected scenario identity.
> Runtime integrity must reject poisoned bytes before C.8 or ordinary decoding;
> the offline verifier must reach its conclusion through separately implemented
> bounded parsing and traversal; both must localize the smallest honest target
> and blast radius or explicitly report why the result is unknown or
> indeterminate. A live clean Store is separately scrubbed under memory,
> cancellation, and queue pressure without whole-Store residency or repair
> effects.

The milestone fails if a clean run passes while a checksum bypass, post-decode
check, stale proof reuse, runtime-parser import by the verifier, silent
agreement coercion, quarantine mutation, derived-state authority promotion,
unbounded walk, or direct raw-artifact route can still pass.

## Decisive Integrity Scenario

### Production subjects and composition roots

The runtime production subjects are both real entry seams:

```text
fresh-process recovery
  -> well-known namespace entry supplies expected root family/identity
  -> C.4 media owner returns untrusted bounded root bytes
  -> C.9 recovery-ingress integrity admission
  -> C.8 decodes admitted root and issues child read expectations
  -> each child repeats C.4 acquisition -> C.9 admission -> C.8 decode
  -> C.8 source precedence, redo, publication, and runtime handoff

ordinary resident load
  -> Store physical runtime requests bounded page or extent bytes
  -> C.4 media owner and C.6 resident frame generation
  -> C.9 resident integrity admission
  -> owner-specific page, extent, index, or blob interpretation
  -> ordinary record-serving work

online scrub
  -> Store-owned scrub lifecycle and scheduling
  -> bounded C.6/C.5.1 allocation and I/O windows
  -> C.9 integrity inspection
  -> descriptive observation only

offline verification
  -> physical-store offline observer executable
  -> independent bounded directory and artifact walk
  -> independent framing and checksum implementation
  -> versioned offline integrity report
```

The `worth-store-physical-format` crate remains the stable byte-grammar and
mechanism owner. The runtime integrity owner decides whether untrusted bytes
satisfy that grammar for an exact scope. Store and recovery privately bind an
Intact validation to the actual C.4/C.6 source incarnation. The Store composes
that admission with resident lifecycle. The recovery runtime composes admission before C.8
interpretation. The offline verifier implements a separate reader of the same
declared grammar.

The harness may create a deterministic Store through public production
facades, copy a closed Store root, edit copied artifacts after the producer is
dead, terminate a child process, wrap the admitted media-observation boundary,
and compare versioned child reports. It may not mint an admitted artifact,
inject private runtime state, call a private validator as its oracle, patch
bytes after recovery begins, or label an arbitrary error as corruption.

### Process roles

The decisive suite uses distinct executable roles and distinct process
identities:

1. **Producer** creates the production-valid baseline through ordinary Store
   writes, C.7 publication, and a clean close.
2. **Pending-obligation producer** is used only for the physical-work family.
   It ordinarily reopens the producer's same physical root after clean close,
   submits one ordinary production
   mutation, lets recovery-journal preparation complete, and blocks the
   executor's first target-media effect after that preparation at the admitted
   backend boundary. Arrival at that boundary proves the v6 `.pending`
   obligation write and directory synchronization completed; the parent also
   observes its durable visibility, then terminates the child without close or
   settlement. The boundary may delay the target effect but may not delay or
   create, encode, or edit the obligation.
3. **Artifact editor** opens only an isolated case subject after producer death
   (a copy or the identity-preserving restoration described below) and
   applies one typed corruption operator to a target selected from the clean
   artifact manifest.
4. **Fresh recovery reopener** invokes the real C.8 recovery composition root.
   It reports integrity observations and recovery outcomes without receiving
   producer heap state or editor expectations.
5. **Offline integrity verifier** opens no Store runtime and imports no Store,
   runtime-integrity, recovery, repair, or maintenance crate.
6. **Parent oracle** derives expected artifact identity, range, corruption
   kind, and minimum honest blast radius from the clean manifest and the
   editor's independently declared operation.
7. **Online scrub subject** runs against a separately produced live clean
   Store through the Store-owned scrub facade under constrained budgets.

Ordinary reopen is bound to the admitted physical root identity: byte-identical
files copied into a different root directory do not preserve the persisted
durability-policy binding. A runtime-inspection or pending-obligation subject
therefore retains its producer's actual root directory. The parent may retain
an immutable external snapshot and restore its exact files into that same
directory between cases, only after every prior child has exited. Restoration
must never replace the root directory itself, must verify baseline bytes and
namespace before the next process starts, and must leave independent before/
after observations for every case. This is fixture preparation, not runtime
repair. Recovery-only cases continue to use isolated copies. A clean same-root
ordinary reopen must succeed; copying the root must not be made legal by
weakening persisted policy-identity checks.

The runner rejects binary reuse between recovery and verifier roles and
rejects protocol, Store, scenario, or run identity substitution.

### Initial world

The producer fixes before any hostile action:

- one real named filesystem and admitted backend profile;
- one Store root whose occupied physical bytes are at least 32 times the
  resident-byte budget;
- at least three data segments and enough records to cross page, extent,
  segment, and manifest boundaries;
- inline and extent-backed payloads;
- a current root and one retainable previous root generation;
- one complete checkpoint and at least two WAL segments, including a bounded
  valid tail after the checkpoint;
- multi-level root, segment, and extent manifest structures where the current
  format provides them;
- nontrivial free-space state;
- every index and blob family that has a canonical ordinary production path at
  C.9 entry, with families that do not yet have that path declared unsupported
  rather than simulated;
- deterministic workload, schedule, corruption, and traversal seeds that are
  independently recorded;
- exact resident, I/O, queue, traversal-entry, traversal-byte, symlink,
  elapsed-time, and report-size budgets;
- a clean artifact manifest containing artifact identities, families,
  generations, exact lengths, declared covered ranges, and expected
  reachability, produced without runtime integrity classification; and
- an empty external report directory outside the Store root.

The clean artifact manifest is test oracle input, not a production artifact and
not authority offered to the recovery or verifier process.

Page-size coverage uses three separately production-issued, clean-closed Store
worlds because page size is Store-wide and one root cannot carry multiple
declarations:

1. the **16 KiB primary world** is the full-scale world above and runs the
   complete cross-family matrix, recovery-precedence coexistence, hostile
   traversal, online scrub, residence/cost counters, container removal and
   duplication, and the 16 KiB page cases;
2. the **32 KiB page world** is a bounded ordinary Store with two distinct
   inline page frames in different concrete scopes, enough publication to
   reopen through production recovery, and one clean manifest; it runs clean
   runtime/offline/recovery agreement plus page `B K L S T U`, decoder-entry,
   and checksum-pass assertions; and
3. the **64 KiB page world** has the same bounded topology and assertions as
   the 32 KiB world under a 64 KiB declaration.

The secondary worlds do not rerun scrub, scale, checkpoint/WAL precedence, or
unrelated family operators. They have distinct Store/scenario identities,
format declarations, manifests, output paths, and immutable baselines; bytes
are never copied across page-size worlds. Each page-world `U` case separately
targets durable-frame schema and embedded physical-record-format version.

Physical-work rows do not use the clean-close world because clean settlement
removes every `.pending` obligation and C.5.1 forbids clean close with an
unmatched live obligation. Their immutable baseline is the separately produced
post-termination world from role 2. The parent independently records the exact
pending pathname, fixed 160-byte length, and digest after child death. The
uncorrupted row inspects that production-issued file; each damaged row restores
that same terminated world's exact contents into the retained physical root
before applying one typed edit. No test constructs
obligation bytes, calls `PhysicalEffectJournal` privately, or treats the
inspection-required owner posture caused by a live obligation as checksum
damage.

### Corruption operator matrix

Except for the separately produced physical-work baseline frozen below, each
row starts from a fresh isolated copy of the applicable clean-close world
frozen above and changes
only the named condition:

| Operator | Required target | Required runtime/offline distinction |
|---|---|---|
| covered-byte flip | payload or structural field inside checksum coverage | checksum mismatch localizes to the covered frame or block before decoding |
| checksum flip | checksum field only | checksum-field damage is distinct from payload damage where the format permits that distinction |
| framing-length lie | page, extent, WAL, checkpoint, or manifest frame | framing is rejected before any length-directed allocation or owner decode |
| generation substitution | valid bytes from another generation | checksum success cannot widen scope; expected/observed generation mismatch is typed |
| identity substitution | valid artifact copied under another identity | checksum success cannot substitute Store, family, segment, extent, page, or root identity |
| pointer corruption | root or manifest pointer | localization names the pointer field and expands blast radius only when descendants cannot be bounded honestly |
| truncation | strict nonzero prefix removed from an artifact | exact incomplete range is reported without reading beyond admitted bytes |
| artifact removal | reachable authority or derived artifact absent | absence is not checksum damage and becomes damaged, unknown, or indeterminate according to owner truth |
| artifact duplication | artifact copied into a conflicting lawful-looking location | duplicate identity is reported; traversal order cannot select one silently |
| supported-version coexistence | a previous version inside a real declared coexistence window, when one exists | accepted through its exact version adapter and reported as that version; current single-version families mark this row not applicable rather than inventing bytes |
| unsupported version | version outside the declared window | `Unsupported` is preserved and never collapsed into corruption |
| derived damage | one current derived artifact with intact rebuild basis | integrity reports damage; only the artifact owner may add `RebuildableDerived` |
| missing rebuild basis | same derived damage plus missing or damaged authority | result is `Unknown` or `Indeterminate`, never optimistically rebuildable |

The editor contract freezes intended-cause mutation semantics:

- `B` changes exactly one covered non-checksum byte and refreshes no checksum
  or digest layer;
- `K` changes no protected payload/structure byte and corrupts the selected
  checksum field; if a distinct enclosing checksum/reference covers that
  field, the editor refreshes only those enclosing checksum fields so the
  selected checksum remains the sole false invariant;
- `L` changes only the encoded framing/length relation and recomputes every
  otherwise-valid enclosing checksum/digest;
- `S` uses independently valid bytes from a different concrete scope, or
  performs the minimum re-encoding needed for that substitution, so all self-
  integrity and enclosing checksum layers pass and only the expected scope
  relation is false;
- `P` changes only the pointer/child reference, leaves the referenced child
  world unchanged, and recomputes the containing and enclosing checksums so
  pointer/child localization—not checksum mismatch—is the intended failure;
- `T` retains the strict nonempty prefix `[0,n)` where
  `0 < n < original_length` and refreshes nothing;
- `R` and `D` change namespace/container presence only and never edit bytes;
  and
- `U` writes one unsupported value on the selected version axis and recomputes
  every checksum/digest covering it, leaving version support as the only false
  invariant.

For checkpoint selective aggregates, the targeted dirty-basis or binding
record is re-encoded with a valid record CRC while the corresponding stored
selective aggregate is deliberately left inconsistent; footer framing and all
unrelated records remain valid. Every editor result is checked independently
against this mutation contract before either runtime is launched.

The mandatory `U` axes are: namespace identity encoding version and namespace
version; durable-frame schema 2 and embedded physical-record-format version 1
for every applicable common-frame granule; physical-work obligation version 6;
WAL frame version 1; and checkpoint record schema 1 for each of the five record
kinds. One `U` row changes exactly one axis. Unsupported byte-order, protocol,
or field-tag cases are separate format-denial tests and cannot stand in for a
version row.

The following compact applicability lock is exhaustive for current granules.
Codes are `B` covered-byte flip, `K` checksum-field flip, `L` encoded
framing/length lie, `S` checksum-valid substitution of the granule's concrete
Store/role/family/identity/generation/ordinal/LSN/binding scope, `P` typed
pointer or child-reference corruption, `T` truncation, `R` whole-artifact or
container removal, `D` conflicting lawful-looking duplication, and `U`
unsupported version. Every listed code is mandatory; every unlisted code is
not applicable under the locks following the table.

| Current granule or exact grouped shape | Mandatory operators |
|---|---|
| namespace identity v1 | `B K T R U` |
| physical-work obligation v6 | `B K S T D U` |
| bootstrap catalog | `B K L S T R D U` |
| current selector; previous selector | `B K L S P T R D U` for each role |
| root manifest | `B K L S P T R D U` |
| root-routing block | `B K L S P T R D U` |
| segment-membership block | `B K L S P T R D U` |
| inline page frame at 16, 32, and 64 KiB | `B K L S T U` at each declared size |
| extent manifest | `B K L S P T R D U` |
| extent chunk frame | `B K L S T U` |
| segment and extent containers that hold embedded page/chunk ranges | `R D` as traversal/reachability cases, not fabricated page/chunk artifacts |
| free-space header; free-space-membership block | `B K L S P T R D U` for each granule |
| WAL v1 frame | `B K L S T U` |
| WAL segment container | `R D` as traversal/reachability cases |
| checkpoint schema-1 stream header, dirty-basis, binding-compaction, binding, and footer | `B K L S T U` for each record kind; selective aggregate corruption is an additional `S` case for dirty-basis, binding, and footer |
| checkpoint stream container | `R D` as traversal/reachability cases |

`L` is not applicable to fixed-size namespace identity or physical-work
obligation because neither has an encoded length field. `P` is not applicable
outside the rows that carry a pointer or child reference. Inline pages, extent
chunks, WAL frames, and checkpoint records are ranges or records inside
containers, so their own `R` and `D` are N/A; the separately listed segment,
extent, WAL-segment, and checkpoint-stream container rows localize reachability
honestly without inventing standalone embedded artifacts. Namespace identity has one lawful
singleton path, so duplication is N/A. It is also C.4's sole Store-identity
trust anchor, so checksum-valid self-substitution `S` is N/A: C.4 lawfully
adopts the substituted identity without an external expected identity. The
mandatory cross-Store substitution instead targets the first dependent
bootstrap catalog and both selector roles; runtime and observer localize those
dependent Store-binding mismatches without relabeling the trust anchor as
corrupt. Removing the sole physical-work pending record erases the only
persisted expected-obligation basis and is not detectable by runtime or offline
integrity; `R` is therefore N/A rather than a parent-only oracle claim. Its `D`
case is detectable because the copied record's internal operation identity
cannot match a second canonical pending filename.
Supported-version coexistence is not applicable to every current row because
no real stale version is supported; `U` remains mandatory. No current C.9
granule is owner-classified as derived, so the derived-damage and
missing-rebuild-basis rows are not applicable at C.9 entry and must not be
simulated with dormant index/blob formats. The first real derived family must
add both owner-twin rows in its insertion packet.

Arbitrary random scribbling and assertions that merely call `is_err()` are
forbidden as decisive evidence. Randomized property tests may supplement the
typed operator matrix but cannot replace its independent localization oracle.

### Recovery-precedence coexistence siege

One matrix row presents a damaged current root alongside an intact retainable
previous root and lawful WAL/checkpoint evidence. Integrity admission reports
the validity of each candidate independently. C.8 alone chooses whether the
previous root, checkpoint, WAL tail, or no source governs recovery.

The proof fails if C.9 selects the previous root, if C.8 interprets the damaged
current root before admission, or if a valid alternate causes the damaged
candidate's observation to disappear.

### Semantically plausible poison siege

A format-aware editor changes a covered structural field to another value that
would be accepted by the owner-specific decoder if checksum/scope validation
and owner-private source-bound admission
were skipped. A decoder-entry counter located after integrity admission must
remain zero for the poisoned artifact in both recovery and ordinary resident
load. Offline verification must independently reject the same artifact.

This siege convicts post-decode validation and parser error masquerading as
integrity validation.

### Runtime/offline disagreement siege

The scenario includes one always-real disagreement case and one conditional
compatibility case:

1. a file that changes between two offline bounded reads, causing the verifier
   to report `Indeterminate`, while the runtime holds a stable media snapshot
   and reports an intact or damaged result; and
2. only when a real persisted-format coexistence window contains more than one
   supported version, a version supported by the runtime but reported
   `Unsupported` by an independently versioned observer build.

At the current Phase 1 baseline every production family exposes a single
supported version. The second case is therefore recorded as not applicable,
not simulated with invented v0/v2 bytes or a test-only narrowed observer. The
comparison codec still has a direct role-bound DTO test proving it preserves a
version-window disagreement without choosing a verdict; that test is not
claimed as end-to-end format-compatibility evidence.

The parent must receive both role-bound protocol observations and an
explicit `PhysicalIntegrityDisagreement`. No component may choose one result,
intersect them into a fabricated consensus, or silently downgrade either.
C.9 does not claim cryptographic authenticity for either observation.

### Hostile offline-walk siege

The offline verifier is run against a bounded fixture containing, outside the
canonical artifact set:

- a high-cardinality directory fan-out;
- a symlink escaping the Store root;
- a hard link that aliases a visited file where the host supports it;
- an unknown artifact version;
- a file whose metadata or bytes change during inspection; and
- a report-output path outside the Store root.

The verifier must remain beneath declared entry, byte, open-file, depth,
elapsed-time, and report-size budgets. It must not follow the escaping symlink,
must not inspect the same physical file twice through a hard-link alias, must
preserve `Unsupported`, `Unknown`, and `Indeterminate`, and must never create or
modify anything inside the Store root.

### Bounded online-scrub siege

The live Store is scrubbed while ordinary resident work continues and while:

- the Store remains at least 32 times larger than resident memory;
- scrub receives no more than one declared window of resident and I/O budget;
- the scheduler temporarily denies admission;
- cancellation arrives once before dispatch and once after an admitted read;
- one resident frame is evicted and reloaded between windows; and
- the Store closes while a final window is outstanding.

Scrub may pause, defer, cancel, or become indeterminate according to the exact
effect boundary. It may not pin the whole Store, bypass C.5.1 scheduling,
starve ordinary work, revalidate each record separately, mutate reachability,
repair bytes, or survive its Store/runtime generation.

### Proof-lifetime and substitution siege

Compile-fail and runtime tests attempt to:

- apply a page admission to an extent or WAL frame;
- pair valid bytes with another Store, artifact identity, physical generation,
  or byte range;
- retain a resident admission after the guard, frame generation, or runtime
  generation ends;
- reuse an admission after mutation, reload, or eviction;
- construct admission proof fields outside their owning module; and
- feed an untrusted or merely checksum-matching view into C.8 or an ordinary
  decoder.

Each attempt must fail at compilation where Rust can express the lifetime or
type distinction, and otherwise fail through an exact typed scope/generation
denial before interpretation.

### Independent observations and assertions

The parent compares, without using either implementation as the oracle:

- the editor's typed operation and clean artifact manifest;
- the runtime integrity observation emitted before C.8 or ordinary decode;
- the C.8 recovery outcome, which remains a separate result;
- the offline verifier's versioned report;
- decoder-entry counters;
- C.4 media and C.5.1 admission counters;
- resident high-water, scrub-window, traversal-entry, traversal-byte,
  duplicate-file, unsupported-version, and indeterminate-read counters;
- a before/after filesystem manifest proving offline and quarantine paths were
  read-only; and
- typed disagreement output where the two observers differ.

Repeating one isolated input with the same traversal and observer version must
produce the same validator outcomes, localization, blast radius, and counters
apart from declared process/run identity and timing fields. Determinism is
asserted independently for runtime and offline output; agreement between them
is not used as the determinism oracle.

For an unambiguous local corruption, both observers must name the exact
artifact family and identity, physical generation, covered range, corrupt field
when independently knowable, and the same minimum honest blast radius. When a
pointer or missing authority prevents narrower proof, a broader localization is
correct and a guessed narrow result is wrong.

### Mutation sensitivity

The decisive suite must contain controlled defects that each turn a specific
assertion red:

1. ignore checksum mismatch;
2. call an owner decoder before admission;
3. infer missing expected scope from payload or accept a matching checksum
   under the wrong scope/generation;
4. collapse damaged, missing, unsupported, unknown, and indeterminate;
5. label derived damage rebuildable without intact authoritative basis;
6. let the verifier import or call the runtime validator/parser;
7. coerce disagreement into agreement;
8. let quarantine observation delete, rename, replace, or release an artifact;
9. let an offline report mint runtime or recovery authority;
10. retain a clean-validation record after frame reuse or rehash the same page
    for every record decode;
11. require whole-Store residency for scrub; and
12. preserve a direct raw-artifact route around admission.

The structural and runtime evidence must identify which defect it convicts. A
single snapshot or broad `is_err()` assertion cannot satisfy several defects by
accident.

## Product Decision Lock

### Decision 1: integrity admission precedes interpretation at two runtime joins

C.9 closes with both joins:

1. **Recovery ingress** validates untrusted discovery bytes before any C.8
   source-specific interpretation, candidate construction, or precedence
   decision.
2. **Resident ingress** validates a newly loaded or reloaded frame before any
   ordinary page, extent, index, or blob decoder receives it.

Recovery admission is recursive, not one untyped prepass. C.4 first admits the
fixed `namespace/identity` trust anchor and supplies stable Store identity. The
well-known current/previous selector locators then supply family and role; C.9
admits each selector before exposing its root generation, selector identity,
format version, or child locator. The admitted selector issues the addressed
root-manifest expectation. Later admitted parents issue concrete expectations
for referenced children; every child requires a new bounded acquisition and
admission. A checksum match never fills a missing expected identity, and an
unknown directory entry is `Unknown`, not an invitation to probe formats until
one parses.

The first scope is therefore intentionally staged rather than fictitiously
complete: externally known locator, Store identity, family, role, byte bound,
and declaration window constrain selector envelope admission; checksum-safe
selector fields then narrow the child scope. Checkpoint streams and the first
WAL frame follow the same law where sequence, LSN, or record count exists only
inside checksummed framing: the validator may parse only the bounded framing
needed to verify that envelope, then expose sealed typed fields for owner
binding. Owner interpretation never runs on the unadmitted bytes.

The C.8 `RecoveredPhysicalRuntimeHandoff` is consumed by post-recovery
residency and scrub composition, but it is too late to protect recovery
discovery itself and cannot mint integrity admission. A milestone
implementation with only one join is incomplete. For each artifact family, its
old route is removed in the same commit that redirects all recovery and clean
resident consumers of that family; no consumer may retain both routes in one
build.

### Decision 2: canonical formats own checksum declarations; C.9 owns admission

C.9 reuses C.5's canonical checksum-protected format. It does not add a
competing checksum field, algorithm registry, envelope, or legacy header mode.

Before implementation changes any format, the C.5-C.8 artifact inventory must
record for each family:

- canonical owner and version;
- identity and generation fields;
- framing and length law;
- checksum algorithm and exact covered range;
- pointer and child-range law;
- whether the artifact is authoritative or derived;
- ordinary and recovery read callsites; and
- current write/encode callsite.

If an artifact on a real current path lacks checksum coverage, its owner must
introduce an explicit versioned format change with compatibility evidence.
Reserved zero fields in retired or legacy headers must not be silently
activated or reinterpreted.

### Decision 3: validation is per immutable physical granule, not per record

The admission granule is the smallest canonical independently checksummed frame
or block that the owner decoder consumes. One successful admission may cover
multiple record decodes while the bytes, identity, scope, and frame generation
remain unchanged.

The C.6 frame entry owns one private clean-validation record keyed by an explicit
frame generation, artifact scope, and covered range. The buffer pool owns a
monotonic process-local frame generation for each newly installed byte image;
the Store adapter separately binds the current Store lifecycle generation.
Neither `PhysicalFrameLoadingIdentity` nor slot identity substitutes for these
facts. Store integrity composition obtains the pure validation result and commits
its owned descriptive record only through the live C.6 clean guard. Dirty transition,
identity promotion, eviction, slot reuse, reload, or runtime-generation change
clears it before the frame can be observed under its next state. Bootstrap
frames loaded before the transition to record serving are explicitly rebound
or re-admitted at that transition. A matching record plus live lease may create
a fresh Store-private admitted view without rehashing; absence or mismatch
forces fresh validation and owner binding. Eviction,
identity-promotion, and frame-reuse tests must prove the next load increments
the validation counter.

Admission is invalidated by:

- byte mutation or dirty transition;
- media reload or resident-frame reuse;
- eviction;
- artifact, Store, or family substitution;
- physical generation change;
- runtime generation change;
- covered-range change; or
- expiry of the immutable media/resident guard.

The existing C.6 `DirtyPhysicalFrame` typestate remains the only non-admitted
decode basis for unpublished mutation bytes of its exact dirty generation. It
is not integrity proof and cannot wrap clean media bytes. Clean-path decoders
accept only admitted views; dirty-path decoders accept only that concrete dirty
typestate. Writeback/reload consumes the dirty view, and later clean use requires
admission.

### Decision 4: public proof is family-specific and concrete

No governed decoder accepts a generic `AuthorityMarker`, a boolean, a checksum
receipt, an offline report, or an untyped “validated bytes” wrapper.

The runtime integrity crate privately shares mechanisms and publicly returns
family-specific sealed validation results. Those values prove byte/format/scope
facts only and cannot open governed owner decoders. Store and recovery
composition roots bind them to their actual C.4/C.6 source handles in private
lifecycle-bearing admitted types; no other crate can construct those wrappers
or reconstruct their binding from exposed fields.

### Decision 5: validity, owner disposition, and quarantine are separate joins

Integrity validation answers what the inspected bytes prove. Artifact-owner
disposition answers whether damaged state is authoritative or derived and, for
derived state, whether an intact current rebuild basis exists. Quarantine
records an operational observation.

These are ordered joins:

```text
untrusted bytes
  -> integrity validation or typed rejection
  -> artifact-owner disposition
  -> optional quarantine observation
  -> later S.10 repair/rebuild policy (outside C.9)

Intact validation + matching live C.4/C.6 source
  -> owner-private source-bound admission
  -> governed owner decoder
```

Neither an integrity validator nor a quarantine recorder may infer
`RebuildableDerived`. Neither may mutate reachability or perform repair.

### Decision 6: offline verification is implementation-independent

The offline verifier may share only:

- stable physical-format declarations that describe the persisted grammar;
- foundational boundary-protocol identity/version vocabulary and descriptive
  physical-integrity observation DTOs with no decision helpers; and
- golden byte fixtures with literal expected checksum values and literal
  covered ranges derived from the format specification, never calculated from
  the shared declarations under test.

It must not depend on or call:

- `worth-store`;
- `worth-store-physical-integrity`;
- `worth-store-recovery-runtime`;
- runtime owner decoders;
- repair, maintenance, or operational mutation crates; or
- a runtime-produced classification helper.

It implements its own bounded byte acquisition, framing checks, checksum
calculation, traversal, duplicate detection, and report projection. Shared
coverage declarations are a deliberate common-mode dependency; the literal
golden vectors must turn red when that declaration drifts from persisted format
truth.

### Decision 7: disagreement is evidence, not policy

Runtime and offline observations retain their original protocol role, version,
scope, and outcome. A comparison operation may produce
`PhysicalIntegrityDisagreement`, but it cannot merge the inputs, choose a
winner, mint admission, or affect recovery or service state.

The operator-side comparison module in `worth-store-offline-integrity-observer`
owns this operation. It consumes serialized foundational observation DTOs, not
runtime crates or admission proofs, and emits a distinct comparison outcome and
comparison-run counters. Agreement raises confidence for an operator; it does
not create a stronger runtime authority type. Runtime and offline observation
counters never include agreement because neither observer can see the other.

### Decision 8: scrub is bounded diagnostic work

The integrity crate owns pure scrub-window inspection and counters. Store owns
the managed scrub handle, runtime-generation binding, C.5.1 scheduling,
allocation, cancellation, close behavior, and public facade. C.10 may later
schedule scrub by maintenance policy through that facade.

Scrub is ordinary-priority-aware diagnostic work in the fixed background-scrub
class. C.5.1 admission preserves foreground resource headroom; callers cannot
promote a scrub window into foreground work. It must be incremental over declared
windows and must release all acquired resources at every terminal or paused
boundary.

### Decision 9: offline output never enters the Store root

The offline verifier is read-only over the Store root. Its report is written by
the invoking process to an explicitly separate destination after observation,
or emitted to stdout. The verifier refuses a report target inside the Store
root. Report emission failure does not change the observation and never causes
a Store-root mutation.

### Decision 10: C.9 removes legacy integrity authority islands

The existing broad `worth-store-physical-integrity` evidence and operational
repair surfaces are not preserved through aliases or compatibility re-exports.
Each current callsite is classified during Phase 1 as:

- retained under the narrow integrity mechanism;
- moved to Store-owned composition;
- moved to its actual artifact owner;
- deferred to the S.10 operational repair owner; or
- removed because it is an unadmitted parallel authority path.

No `evidence::*` bundle, generic proof-role system, operational repair facade,
or certification-only authority may remain as an alternate way to open a C.9
governed decoder.

## Semantic Vocabulary Lock

The following terms are normative:

- **untrusted physical artifact** — bounded bytes plus descriptive location
  and expected scope, with no validity authority;
- **artifact scope** — exact Store, artifact family, artifact identity,
  physical generation, covered byte range, and format version against which
  validation is performed;
- **integrity validation** — a sealed family-specific result proving only that
  the supplied immutable bytes satisfied the canonical physical envelope and
  checksum for the supplied exact descriptive scope; it carries no media,
  resident-frame, recovery, or decoder-opening authority;
- **integrity admission** — an owner-private Store or recovery wrapper that
  binds one integrity validation to the actual C.4 bounded-media or C.6
  resident-frame incarnation and is the only value that opens a governed clean
  owner decoder;
- **integrity observation** — descriptive result and counters; never admission
  authority;
- **damage localization** — the smallest boundary that available physical
  evidence can honestly identify, including cause, range, field when known,
  and blast radius;
- **blast radius** — the physical set whose integrity cannot be proven because
  of the localized defect; it is not semantic impact;
- **authoritative artifact** — persisted physical truth whose replacement or
  source use is governed by its owner and C.8/S.10 policy;
- **derived artifact** — state that an artifact owner can prove is reproducible
  from an intact, current authoritative basis;
- **quarantine observation** — a durable or emitted record that an operator or
  later policy may consume, with no mutation, repair, or release authority;
- **offline integrity report** — a versioned read-only observation produced by
  the independent offline executable; and
- **integrity disagreement** — preservation of two non-equivalent observations
  for the same declared scope, with no resolution policy.

Classification has three non-substitutable axes:

| Axis | Vocabulary | Constructor and limit |
|---|---|---|
| validator outcome | `Intact`, `Damaged`, `Unsupported`, `Unknown`, `Indeterminate` | runtime and offline observers construct this independently; even `Intact` says nothing about source provenance, admission authority, rebuildability, or quarantine |
| owner disposition | `IntactAuthority`, `IntactDerived`, `DamagedAuthority`, `DamagedDerived`, `RebuildableDerived` | the named artifact owner joins validator outcome with role truth; rebuildability requires a concrete intact current basis |
| quarantine posture | absent or `PhysicalQuarantineObservation` | orthogonal observation only; it cannot erase either preceding axis or imply reachability change |

The offline report carries only validator outcome and localization. Runtime and
offline agreement compares family, identity, generation, range, cause, and
blast radius; it never demands matching owner disposition or quarantine.

## Normative Runtime Contracts

The exact private representation is an implementation decision. The public
responsibilities, type separation, and constructor authority below are
normative.

### Untrusted input and exact scope

```rust
pub struct UntrustedPhysicalArtifact<'media> {
    // Bounded immutable bytes or bounded chunk source; no source authority.
}

pub struct PhysicalArtifactScope {
    // Private exact identity, generation, range, and version fields.
}
```

`UntrustedPhysicalArtifact` and `PhysicalArtifactScope` are descriptive pure-
mechanism inputs. Their fields stay private, but their family-specific
constructors are public because constructing either opens no decoder and proves
no source provenance. The byte input accepts only an immutable bounded slice,
sealed snapshot, or budgeted chunk source; the scope is assembled from
canonical owner/physical-format identity values, never lookalike primitives.
Their read-only accessors use canonical
identity vocabulary from the artifact owner or
`worth-foundational` when that meaning is stable across runtimes. C.9 must not
invent Store, page, extent, WAL, checkpoint, or root identity lookalikes.
The pure input and result do not bind a C.4 media capability or C.6 frame. Store
and recovery composition privately perform that binding after validation. Those
lifecycle facts stay in their runtime owners rather than becoming lower-crate
lookalikes.

### Family-specific validation

```rust
pub enum PageIntegrityValidation<'media> {
    Intact(IntegrityValidatedPageFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum WalIntegrityValidation<'media> {
    Intact(IntegrityValidatedWalFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum CheckpointStreamHeaderIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointStreamHeader<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum CheckpointDirtyBasisIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointDirtyBasis<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum CheckpointBindingCompactionIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointBindingCompaction<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum CheckpointBindingIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointBinding<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub enum CheckpointFooterIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointFooter<'media>),
    Rejected(PhysicalIntegrityRejection),
}
```

Equivalent concrete validated views exist for every current C.5-C.8 granule:
physical-work obligation, bootstrap catalog, current selector, previous
selector, root manifest, root-routing block, segment-membership block, inline
page frame, extent manifest, extent chunk frame, WAL frame, each checkpoint
record kind, free-space header, and free-space-membership block. C.4 continues
to own runtime admission of `namespace/identity`; its declaration is shared so
the independent observer can report it. Mutation-lock contents are explicitly
non-authoritative metadata and are observed only as a namespace/type role.
Dormant `DurableSegmentManifest`, index, and blob formats have no current
ordinary Store path and are reported `Unsupported`; fake adapters and
test-only production enums are forbidden.

Each intact validated view:

- borrows the immutable bytes or owns a sealed immutable snapshot;
- carries the exact scope internally;
- exposes only validated format projections needed by an owner-private
  admission wrapper, never a governed owner decoder entry;
- cannot be publicly constructed, cloned into a wider lifetime, or retargeted;
  and
- is consumed only by a lifecycle-bearing Store/recovery wrapper.

An internal sealed enum may reduce validator duplication, but no pure validation
result and no public generic admission may open a governed family decoder.

### Rejection and localization

```rust
pub enum PhysicalIntegrityRejection {
    Damaged(PhysicalDamageLocalization),
    Unsupported(UnsupportedPhysicalIntegrityVersion),
    Unknown(UnknownPhysicalIntegrityPosture),
    Indeterminate(IndeterminatePhysicalIntegrityPosture),
}

pub struct PhysicalDamageLocalization {
    pub scope: PhysicalArtifactScope,
    pub cause: PhysicalDamageCause,
    pub damaged_range: PhysicalByteRange,
    pub field: Option<PhysicalFormatField>,
    pub blast_radius: PhysicalBlastRadius,
}
```

The rejection types preserve framing, length, checksum, version, identity,
generation, pointer, truncation, missing-artifact, duplicate-artifact, changing
source, and exhausted-bound distinctions where the observer can prove them.
They do not expose a stringly error as the only programmatic result.

Localization follows this rule:

> Report the narrowest range supported by independent physical evidence, and
> widen to the nearest enclosing frame, block, artifact, or reachable subtree
> when the corrupt field prevents a narrower honest conclusion.

### Artifact-owner disposition

```rust
pub struct PhysicalArtifactDisposition {
    // Private validator outcome, optional compatible owner disposition,
    // and optional quarantine observation.
}

pub enum PhysicalArtifactRoleDisposition {
    IntactAuthority(IntactPhysicalAuthorityObservation),
    IntactDerived(IntactPhysicalDerivedObservation),
    DamagedAuthority(DamagedPhysicalAuthorityObservation),
    DamagedDerived(DamagedPhysicalDerivedDisposition),
}

pub enum DamagedPhysicalDerivedDisposition {
    RebuildableDerived(RebuildablePhysicalDerivedObservation),
    Unknown(UnknownDerivedRebuildability),
    Indeterminate(IndeterminateDerivedRebuildability),
}
```

Private constructors permit owner disposition only when compatible with the
retained validator outcome; unsupported, unknown, and indeterminate outcomes
have no invented owner disposition. The artifact owner constructs every
role-bearing variant. Constructing
`RebuildablePhysicalDerivedObservation` requires the owner's concrete derived
identity plus concrete intact authoritative-basis observation for the current
physical generation. A generic integrity proof or marker bound cannot satisfy
that constructor. An intact derived observation requires the admitted derived
artifact and its current owner-declared derivation identity; it never becomes
authority. Damage remains explicit when rebuildability is unknown or
indeterminate.

Quarantine remains orthogonal, so adding it cannot erase validator or owner
facts. Disposition is descriptive. Neither outcome axis nor quarantine
authorizes a read decoder, recovery choice,
repair, deletion, rebuild, reachability mutation, or semantic service
transition.

### Future adapter evidence

```rust
pub struct PhysicalIntegrityObservationEvidence {
    // Private identity, generation, range, validator posture, owner class,
    // and optional quarantine posture.
}

pub struct RecoveryPhysicalIntegrityAdapterEvidence {
    // Private observation evidence plus bounded C.8-provenance options.
}
```

The offline reporter and Store/recovery composition separately project into
the first `worth-foundational` description; the offline projection leaves
owner and quarantine fields absent. It contains no admission proof or decision
helper. Only C.8 recovery composition constructs the second and adds
provenance-bearing physical options such as previous-root or checkpoint/WAL
candidacy. The offline observer and comparator never produce or compare
recovery options. Neither type selects or authorizes an option; a later adapter
that executes one requires C.8/S.10 concrete authority.

### Resident admission and proof lifetime

```rust
pub struct IntegrityAdmittedResidentPage<'frame> {
    // Private Store/runtime/frame identity plus matching validation.
}

impl ServingPhysicalRuntime {
    pub fn load_integrity_admitted_page(
        &self,
        request: ResidentPageLoadRequest,
    ) -> Result<IntegrityAdmittedResidentPage<'_>, ResidentIntegrityLoadDenial>;
}
```

The Store composition root constructs this wrapper only while holding the C.6
resident guard and matching frame-owned validation record for the exact frame
generation. Clean-path owner decoders accept this wrapper or its family-specific
borrowed projection, never raw resident bytes. Dirty-path decoding requires the
distinct C.6 `DirtyPhysicalFrame`; dirty transition consumes the admitted view
before mutable access becomes available.

Equivalent wrappers are required for extents and any current resident index or
blob granule. They share private lifecycle mechanics, not a public
cross-family authority.

### Recovery ingress

```rust
pub(in crate::orchestration) enum IntegrityAdmittedRecoveryArtifact<'media> {
    BootstrapCatalog(IntegrityValidatedBootstrapCatalog<'media>),
    CurrentSelector(IntegrityValidatedCurrentRootSelector<'media>),
    PreviousSelector(IntegrityValidatedPreviousRootSelector<'media>),
    RootManifest(IntegrityValidatedRootManifest<'media>),
    RootRoutingBlock(IntegrityValidatedRootRoutingBlock<'media>),
    SegmentMembershipBlock(IntegrityValidatedSegmentMembershipBlock<'media>),
    PageFrame(IntegrityValidatedPageFrame<'media>),
    ExtentManifest(IntegrityValidatedExtentManifest<'media>),
    ExtentChunk(IntegrityValidatedExtentChunkFrame<'media>),
    WalFrame(IntegrityValidatedWalFrame<'media>),
    CheckpointStreamHeader(IntegrityValidatedCheckpointStreamHeader<'media>),
    CheckpointDirtyBasis(IntegrityValidatedCheckpointDirtyBasis<'media>),
    CheckpointBindingCompaction(
        IntegrityValidatedCheckpointBindingCompaction<'media>,
    ),
    CheckpointBinding(IntegrityValidatedCheckpointBinding<'media>),
    CheckpointFooter(IntegrityValidatedCheckpointFooter<'media>),
    FreeSpaceHeader(IntegrityValidatedFreeSpaceHeader<'media>),
    FreeSpaceMembershipBlock(IntegrityValidatedFreeSpaceMembershipBlock<'media>),
}
```

This recovery-runtime-private enum is the only raw-artifact input accepted by
C.8 owner-specific discovery after cutover. Every variant also privately carries
the exact C.4 bounded-media-read identity/incarnation that recovery ingress
matched to the validated scope; the abbreviated signatures above omit that
private binding. It preserves family specificity
while allowing recovery orchestration to route admitted inputs. C.8 candidate
construction and source precedence remain C.8 responsibilities.

### Online scrub

```rust
impl ServingPhysicalRuntime {
    pub fn start_physical_integrity_scrub(
        &self,
        request: PhysicalIntegrityScrubRequest,
    ) -> Result<PhysicalIntegrityScrubHandle, PhysicalIntegrityScrubDenial>;
}

pub enum PhysicalIntegrityScrubProgress {
    WindowCompleted(PhysicalIntegrityScrubWindowObservation),
    Paused(PhysicalIntegrityScrubResumePoint),
    Deferred(PhysicalIntegrityScrubDeferral),
    Cancelled(PhysicalIntegrityScrubCancellation),
    Completed(PhysicalIntegrityScrubObservation),
    Indeterminate(IndeterminatePhysicalIntegrityPosture),
}
```

The request declares Store scope, exact family targets, traversal and byte
bounds, and a bounded deadline. Scheduling uses the background-scrub class
described in Decision 8, not a caller-supplied priority. Observations are pulled
from the handle; the optional bounded report writer takes an explicit
caller-owned output sink at the outer diagnostic boundary, not inside the
request or integrity mechanism. Output failure cannot mutate Store state or
silently restart inspection. The handle is Store-owned, bound to one runtime
generation, and cannot perform work after runtime close. Resume points are
descriptive continuation positions scoped to the unchanged Store/runtime
generation; they are not integrity proofs and are rejected after generation
change.

The integrity mechanism receives only one immutable admitted window at a time.
It does not import Signal or own scheduler/executor effects.

### Quarantine observation

```rust
pub struct PhysicalQuarantineObservation {
    pub scope: PhysicalArtifactScope,
    pub localization: PhysicalDamageLocalization,
    pub posture: PhysicalQuarantinePosture,
    pub observed_at: PhysicalObservationSequence,
}
```

Construction records what was observed and why an artifact is considered
quarantined by an external owner or operator. The type contains no media
capability, path mutation handle, reachability authority, repair token, or
release method. Any future quarantine mutation belongs to S.10 and requires a
different concrete authority.

### Counters and cost observations

Runtime-integrity counters cover inspected/validated/rejected bytes and frames
by family and failure cause. Store/recovery owner counters separately cover
source-bound admissions, skipped decoder entries, clean-frame validation reuse and fresh
rehash, scrub lifecycle/high water, quarantine observations, owner-proved
rebuildability, unknown, and indeterminate outcomes. Offline-run counters cover
entries, bytes, depth, open files, duplicate identities, symlink refusals,
unsupported versions, and exhausted bounds. A comparison run separately counts
agreements and disagreements; those counts never appear as if observed by one
lane.

Counters are observations only. They cannot open decoders or substitute for
typed outcomes.

## Normative Observation And Comparison Protocol

The independent `worth-store-offline-integrity-observer` crate owns a dedicated
executable with two read-only operations:

```text
physical_store_integrity_observer observe \
  --store-root <closed-or-isolated-store-root> \
  --report <path-outside-store-root> \
  --max-entries <n> \
  --max-bytes <n> \
  --max-open-files <n> \
  --max-depth <n> \
  --max-symlinks <n> \
  --max-elapsed-ms <n> \
  --max-report-bytes <n>

physical_store_integrity_observer compare \
  --runtime-observation <path> \
  --offline-observation <path> \
  --report <external-report-path>
```

Neither operation offers repair, delete, quarantine, recovery-choice, or
"accept anyway" behavior. `observe` canonicalizes and rejects output inside
its Store root. `compare` consumes only serialized foundational observation
DTOs and cannot access or infer a Store root; the parent scenario or operator
that knows the root owns output exclusion, while `compare` itself rejects an
output path equal to either input report.

The protocol identity is
`store.physical.integrity-observation` with version `1`. Version identity and
the admitted compatibility range use `worth-foundational` boundary-protocol
types. Store diagnostic projection and offline `observe` both emit this schema
with distinct role identities; neither report is admission. The protocol
includes:

- protocol, role, executable, process, run, scenario, and Store identities;
- declared observer compatibility window;
- declared and consumed traversal/resource bounds;
- artifact observations with family, identity, generation, range, validator
  outcome, localization, and blast radius;
- unsupported, unknown, and indeterminate facts without lossy projection;
- traversal and amplification counters;
- report completeness posture; and
- no runtime admission, repair, or recovery authority field.

`compare` emits `store.physical.integrity-comparison` version `1`, retaining
both input identities/outcomes plus exact agreement or disagreement fields. It
contains no reconciled verdict.

The report schema has its own wire DTOs. It must not serialize a runtime proof
type. Adding protocol version `2` later means adding a version adapter under the
offline protocol boundary, not rewriting version `1` in place.

## Failure And Compatibility Law

### Detection versus compatibility

Format compatibility and observer compatibility are distinct:

- the artifact owner declares which physical format versions can be parsed;
- runtime integrity declares which of those versions it admits in this build;
- the offline verifier independently declares its observation window; and
- the comparison layer may report different windows without changing either.

Within a supported window, every version adapter must validate that version's
actual framing and checksum coverage before projecting stable observations.
Outside the window, the result is `Unsupported`, not `Damaged`.

### Missing and changing evidence

Absence is classified using owner-declared reachability and artifact role. A
missing reachable authority may become `DamagedAuthority`; an optional or
unknown file remains `Unknown`; a file that changes during observation becomes
`Indeterminate` unless one stable bounded snapshot was admitted.

No observer may retry until a convenient stable answer appears without
recording the intervening indeterminate observation and consuming retry budget.

### Indeterminate effects

If a scrub read or offline observation has begun and the system cannot prove
whether the complete intended range was stably observed, it returns
`Indeterminate` with the known prefix/range and consumed budget. Cancellation
does not rewrite an admitted or partially completed effect into no-effect.

## Performance And Amplification Contract

Ordinary reads pay integrity cost once per newly loaded immutable canonical
granule, not once per record decode, query, or logical access. A resident hit on
the same admitted frame generation performs no media reread and no checksum
rehash. Mutation, eviction, reload, or generation change requires new
admission.

Recovery inspection remains bounded by C.8's discovery and replay budgets. C.9
may add one linear framing/checksum pass over each artifact range C.8 already
must obtain; it may not add an unbounded whole-Store prepass or duplicate WAL
replay.

Online scrub obeys all of the following:

- resident bytes are bounded by the declared scrub window plus already
  admitted runtime overhead;
- outstanding I/O and queue occupancy are bounded through C.5.1 admission;
- each canonical granule is inspected at most once per completed scrub scope
  unless an explicit retry consumes retry budget;
- progress and cancellation are observable at window boundaries; and
- ordinary record-serving work retains its admitted priority and resources.

Offline verification is linear in the bytes and entries it actually admits,
bounded by explicit entry, byte, depth, open-file, time, and report-size limits.
Duplicate aliases do not multiply byte inspection. Unknown files and
unsupported versions consume declared traversal budget and remain visible.

The decisive tests assert high-water and amplification counters. Wall-clock
latency alone is not acceptable performance evidence.

## Architectural Destination

### Dependency direction

The production dependency direction after C.9 is:

```text
worth-foundational
      ^
      |
worth-store-physical-format
      ^
      |
worth-store-physical-integrity
      ^                 ^                         ^
      |                 |                         |
worth-store-buffer-pool worth-store-recovery-runtime worth-store

worth-store-physical-format
      ^
      |
worth-store-offline-integrity-observer
```

Additional existing lower artifact-owner dependencies are omitted from the
diagram. The governing constraints are:

- `worth-store-physical-integrity` has no dependency on `worth-store`;
- the final normal dependency set of `worth-store-physical-integrity` is
  exactly `worth-foundational` and `worth-store-physical-format`; its current
  direct `sha2`, `worth-proof`, Store, authority, contracts, aspect-native, and
  security edges are removed;
- `worth-store` and `worth-store-recovery-runtime` may depend downward on the
  narrow integrity mechanism;
- `worth-store-buffer-pool` adds one downward integrity edge only to retain the
  owned descriptive validation record whose invalidation follows the C.6 frame
  lifecycle; physical backend adds no integrity edge;
- `worth-store-offline-integrity-observer` has no dependency on runtime integrity,
  recovery runtime, Store, maintenance, operations, or repair;
- physical format exposes a narrow stable declaration facade plus writer/runtime
  byte mechanisms, but no admission or recovery policy;
- the offline C.9 crate may import only the declaration facade and implements
  checksum calculation and parsing independently; and
- no reverse or optional-feature edge recreates the forbidden direction.

The current upward Store dependency in `worth-store-physical-integrity` is a
blocking architectural defect, not an inconvenience to wrap. Store-specific
`PhysicalRecordChunkBasis`, `PhysicalRecordChunkView`, allocation types,
`LifecycleGeneration`, Signal/scheduling adapters, and managed scrub lifecycle
move to Store-owned composition or their actual owner before Store or recovery
may consume the integrity crate.

### Stable composition rules

The topology is organized along four distinct axes:

1. **artifact-family validation** in the pure runtime integrity mechanism;
2. **runtime lifecycle and effect composition** in Store;
3. **recovery interpretation order** in recovery runtime; and
4. **independent media observation and comparison** in its dedicated crate.

These axes may exchange narrow types through declared facades. They do not
share files, catch-all helpers, generic evidence bags, or mutual dependencies.

## Required Destination Directory And Module Tree

Status markers mean:

- `[existing]` remains in place with its responsibility narrowed as stated;
- `[create]` is created by C.9;
- `[move]` relocates an existing responsibility and deletes the old location;
- `[replace]` supersedes a current module or surface and removes it in the same
  phase;
- `[remove]` is deleted after its classified consumers move or disappear; and
- `[successor]` is a committed destination, not an empty placeholder created
  by C.9.

### Cross-runtime descriptive vocabulary

```text
crates/worth-foundational/src/
└── physical_integrity_observation/                 [create]
    ├── mod.rs                                      [stable descriptive facade]
    ├── artifact_identity.rs
    ├── artifact_family.rs
    ├── physical_generation.rs
    ├── byte_range.rs
    ├── integrity_posture.rs
    ├── authority_class.rs
    ├── quarantine_posture.rs
    ├── recovery_option.rs
    ├── disagreement.rs
    └── adapter_evidence.rs
```

The dominant axis is physical observation meaning that remains identical when
crossing process or future adapter boundaries. These types are bounded,
serializable descriptions only. They contain no checksum calculation, parser,
classifier, runtime constructor, media capability, admission proof, recovery
selection, repair token, or semantic conclusion.

Runtime Store/recovery composition and the offline report layer separately
project their outcomes into the common observation vocabulary. Only C.8
projects provenance-bearing recovery options. The stable facade lets comparison
retain exact physical facts without either observer importing the other's
decision code. No foundational value is accepted by a governed runtime
decoder.

Future observation fields add named files or a versioned adapter projection;
they do not turn `adapter_evidence.rs` into a generic evidence bag.

### Canonical physical-format declarations

```text
workspaces/worth-store/crates/worth-store-physical-format/src/
├── integrity_declarations/                         [create from current truth]
│   ├── mod.rs                                      [stable declaration facade]
│   ├── algorithm.rs
│   ├── coverage.rs
│   ├── version.rs
│   ├── family.rs                                   [stable family identity]
│   └── families/
│       ├── mod.rs
│       ├── namespace_identity.rs                   [C.4 trust-anchor declaration]
│       ├── physical_work_obligation.rs
│       ├── page_frame.rs                           [range within segment artifact]
│       ├── extent_chunk.rs
│       ├── wal.rs
│       ├── checkpoint/
│       │   ├── mod.rs
│       │   ├── stream_header.rs
│       │   ├── dirty_basis.rs
│       │   ├── binding_compaction.rs
│       │   ├── binding.rs
│       │   └── footer.rs
│       ├── root/
│       │   ├── mod.rs
│       │   ├── bootstrap_catalog.rs
│       │   ├── current_selector.rs
│       │   ├── previous_selector.rs
│       │   ├── manifest.rs
│       │   └── routing_block.rs
│       ├── segment_membership.rs
│       ├── extent_manifest.rs
│       └── free_space/
│           ├── mod.rs
│           ├── header.rs
│           └── membership_block.rs
├── wal_frame/                                       [move byte mechanism]
│   ├── mod.rs                                       [grammar/checksum facade]
│   ├── header.rs
│   ├── checksum.rs
│   └── encode.rs
├── physical_work_obligation/                        [move byte mechanism]
│   ├── mod.rs                                       [v6 grammar/checksum facade]
│   ├── field_code.rs
│   ├── checksum.rs
│   └── encode.rs
└── <other existing family codecs/checksum execution> [existing/narrow]

committed successors:
src/integrity_declarations/families/index/           [successor C.11]
src/integrity_declarations/families/blob/            [successor C.11]

removed or relocated after consumer classification:
src/offline_walk/                                   [move C.9 observation to the
                                                    offline verifier; remove
                                                    format-owned I/O/decisions]
```

The dominant axis under `integrity_declarations/` is persisted meaning shared
across runtimes: algorithm identity, coverage ranges, version identity, and
artifact-family declaration. It contains no byte-reading effect, parser,
checksum execution, admission decision, classification, recovery policy, or
lifecycle.

The `families/` axis prevents new persisted formats from accumulating in one
declaration bag. Root, checkpoint, and free-space are directories because their
independently framed granules have different identities, coverage, retention,
or stream roles. `page_frame` is a range within a segment container; it is not
a fabricated page-file identity. C.11 adds named index/blob declaration
siblings only when the canonical formats become real; C.9 creates no empty
successor directories. The dormant `DurableSegmentManifest` codec and literal
golden are preserved `pub(crate)` under physical-format as unsupported format-
owner residue. Phase 4 removes its public export, `RecordArtifactFile` locator,
recovery-media reader, scheduler/recovery-locator vocabulary, and external
certification constructors. It is not activated or exported as a current C.9
family.

Existing physical-format family encoders and runtime codecs retain the canonical
checksum execution used to write and validate format bytes. Two current
exceptions move in Phase 2 without changing bytes: WAL v1 magic/version/header/
footer grammar, SHA-256 coverage, and byte encode/decode move from
`worth-store-wal/src/artifact_store/frame_codec.rs` to
`worth-store-physical-format/src/wal_frame/`; physical-work v6 magic/version,
fixed 160-byte grammar, field/tag encoding, SHA-256 `[0,128)` coverage, and byte
encode/decode move from Store
`physical_runtime/work/recovery/{locator,effect_obligation}.rs` to
`worth-store-physical-format/src/physical_work_obligation/`. `worth-store-wal`
retains segment-file I/O, prefix continuity, append planning, LSN/publication
semantics, paths, and lifecycle denials. Store retains physical-work journal I/O,
operation/target lifecycle types, pending-name policy, inventory/disposition,
and maps those owner types to and from the format-owned field-code DTOs. The old
WAL and Store byte/checksum implementations are deleted in the same Phase 2
move; neither owner keeps a private duplicate. Runtime integrity may call
the format mechanisms behind validation. The offline C.9 observer imports
only `integrity_declarations` and separately implements checksum calculation and
parsing. A source/dependency guard rejects imports from family runtime codecs or
checksum execution into `worth-store-offline-integrity-observer`.

Rust dependency direction means the lower physical-format crate cannot accept
higher-layer admitted or dirty types. Its cross-crate raw codecs therefore
remain narrow byte mechanisms, not owner-facing decoder authority. After
cutover, a mechanical source-route guard permits persisted-input calls only
from named `worth-store-physical-integrity/src/artifact/**` family validators;
format-owner implementation/tests may exercise the mechanisms directly.
Store's only additional raw decode allowance is
`worth-store/src/physical_runtime/integrity/dirty_decode/**`, whose functions
require a C.6 `DirtyPhysicalFrame` and cannot accept a clean lease. The
independent observer calls neither lane.

Clean owner-facing decoding is exposed only by Store/recovery-private admission
wrappers that bind a concrete `IntegrityValidated*` view to the matching live
C.4/C.6 source incarnation. Ordinary Store,
recovery, maintenance, certification, and other consumers have an expected
zero for direct persisted-input raw codec, inspector, or checksum-validity
calls. Writer-side encode/checksum construction over newly created candidate
bytes remains allowed and is distinguished by path and typed input; it cannot
be used to admit bytes read from media. The guard and paired compile tests are
the cycle-free enforcement seam—public visibility of a lower mechanism is
never presented as proof that arbitrary callers are lawful.

This split does not change persisted format. If the current declaration cannot
be separated without a format version change, the owner earns that explicit
change through the compatibility law rather than reinterpreting bytes.
The current format-owned `offline_walk` I/O, checksum, decode, and
classification path cannot serve as the C.9 independent observer and cannot
remain an alternate C.9 facade. Phase 1 classifies its other consumers; lawful
non-C.9 observation responsibilities move to their owning offline modules, and
stable persisted declarations remain here.

### Runtime integrity mechanism

```text
workspaces/worth-store/crates/worth-store-physical-integrity/
├── Cargo.toml                                      [replace dependencies]
├── README.md                                       [replace public contract]
└── src/
    ├── lib.rs                                      [replace broad facade]
    ├── validation/                                 [replace/narrow]
    │   ├── mod.rs
    │   ├── untrusted_artifact.rs                   [create]
    │   ├── artifact_scope.rs                       [create]
    │   ├── rejection.rs                            [create]
    │   └── validated/                              [create]
    │       ├── mod.rs
    │       ├── <one file/directory per runtime-admitted C.5-C.8 granule above>
    │       ├── index/                              [successor C.11; do not create yet]
    │       └── blob/                               [successor C.11; do not create yet]
    ├── artifact/                                   [replace family islands]
    │   ├── mod.rs
    │   ├── <one file/directory per runtime-admitted C.5-C.8 granule above>
    │   ├── index/                                  [successor C.11; do not create yet]
    │   └── blob/                                   [successor C.11; do not create yet]
    ├── localization/                               [replace flat classification]
    │   ├── mod.rs
    │   ├── cause.rs
    │   ├── damaged_range.rs
    │   ├── format_field.rs
    │   └── blast_radius.rs
    ├── observation/                                [create]
    │   ├── mod.rs
    │   ├── outcome.rs
    │   └── counters.rs
    ├── scrub/                                      [replace/narrow]
    │   ├── mod.rs
    │   ├── window.rs
    │   ├── inspection.rs
    │   ├── outcome.rs
    │   └── counters.rs
    └── quarantine/                                 [replace/narrow]
    │   ├── mod.rs
    │   ├── observation.rs
    │   └── posture.rs

removed after callsite classification:
src/authority/                                      [remove generic authority]
src/evidence/                                       [remove proof/report bundles]
src/offline_classification/                         [remove shared offline truth]
src/operational_repair/                             [remove/defer to S.10 owner]
src/recovery_handoff/                               [replace in recovery runtime]
src/compaction_source_clearance.rs                  [move to compaction owner]
src/damage_handoff.rs                               [replace with observation]
src/damage_classification.rs                        [replace with localization]
src/checksums/                                      [move canonical pieces to
                                                    physical-format; remove rest]
src/*_integrity_compile_fail_proofs.md              [remove generated-style proof]
```

The dominant axis under `artifact/` and `validation/validated/` is canonical
artifact family. Each family validator owns only physical envelope, scope, and
checksum validation for its persisted format. It excludes owner semantics,
recovery precedence, repair, and Store lifecycle. Family files may call the
canonical physical-format codec mechanism privately, but the narrow `lib.rs`
facade exports only untrusted input, exact scope, concrete family validations, rejection,
localization, scrub-window inspection, quarantine observation, and counters.

The committed C.11 `artifact/{index,blob}` and
`validation/validated/{index,blob}` destinations are shown because those
families are roadmap commitments. C.9 creates no empty directories. When an
index or blob format becomes a real ordinary path, it adds its validator and
validated-view siblings under those existing family axes without splitting
`artifact/mod.rs`, moving the facade, or generalizing existing proofs.

The current `wal_frames`, `manifests`, `containers`, `index_pages`, and
`blob_chunks` islands are not bulk-renamed blindly. The disposition table below
assigns each current responsibility one exact destination and action; no
implementation-time preserve/move/remove choice remains.
Types that decide recovery precedence, compaction clearance, repair, or
semantic container meaning cannot remain merely because they currently live
near integrity code.

### C.6 frame-owned validation record

```text
workspaces/worth-store/crates/worth-store-buffer-pool/src/physical_residency/
└── integrity_validation/                           [create]
    ├── mod.rs
    ├── clean_record.rs
    └── invalidation.rs
```

The dominant axis is clean-frame lifecycle, not validation. Each C.6 `FrameEntry`
holds at most one owned `PhysicalIntegrityValidationRecord` from runtime
integrity. That descriptive record contains family, exact-scope digest, byte-
range digest, and validation mechanism/version, but no byte borrow and no
decoder-opening authority. Buffer pool assigns an explicit monotonically
changing frame generation on load/reuse and permits record commit only while a
live clean loading/resident guard identifies that entry and generation. Its
state transitions clear the record on dirtying, eviction, reuse, reload, and
runtime-generation invalidation; Store composition may construct its private
resident admission wrapper only while holding the same live lease and matching
record. The module contains no checksum, decoder, artifact policy, admission
wrapper, or side table. The existing `DirtyPhysicalFrame` remains the separate
unpublished-mutation typestate.

### Store-owned runtime composition

```text
workspaces/worth-store/crates/worth-store/src/physical_runtime/
├── integrity/                                      [create]
│   ├── mod.rs                                      [stable Store facade]
│   ├── resident_admission/                         [create]
│   │   ├── mod.rs
│   │   ├── load.rs
│   │   ├── admitted_page.rs
│   │   ├── admitted_extent.rs
│   │   ├── record_binding.rs
│   │   └── denial.rs
│   ├── recovery_join/                              [create]
│   │   ├── mod.rs
│   │   ├── handoff_binding.rs
│   │   └── runtime_generation.rs
│   ├── disposition/                                [create]
│   │   ├── mod.rs
│   │   ├── authority.rs
│   │   ├── derived.rs
│   │   └── classification.rs
│   ├── scrub/                                      [move Store lifecycle]
│   │   ├── mod.rs
│   │   ├── request.rs
│   │   ├── handle.rs
│   │   ├── scheduling.rs
│   │   ├── progress.rs
│   │   ├── cancellation.rs
│   │   └── close.rs
│   ├── quarantine/                                 [create]
│   │   ├── mod.rs
│   │   └── observation.rs
│   └── diagnostics/                                [create]
│       ├── mod.rs
│       └── counters.rs
└── record_serving/
    └── work_semantics/
        └── integrity_admission.rs                  [create consumer rule]
```

The dominant axis under `physical_runtime/integrity/` is live Store lifecycle
composition. This location owns C.6 resident guards, C.5.1 scheduling and
cancellation, Store/runtime generation, C.8 handoff binding, and the public
managed scrub facade. It excludes checksum implementation, offline traversal,
recovery source choice, repair, and JSON serialization.

The Store-owned runtime observation protocol is a diagnostic consumer outside
that physical-runtime boundary:

```text
workspaces/worth-store/crates/worth-store/src/integrity_observation/ [create]
├── mod.rs                         [stable diagnostic report facade]
├── runtime_report.rs              [bounded caller-sink export]
└── runtime_report/
    ├── artifact.rs                [descriptive address projection]
    ├── outcome.rs                 [lossless outcome projection]
    └── vocabulary.rs              [version-one wire names]
```

It receives remaining declared scopes, settled window observations, and
counters from the managed handle. It cannot access media, lifecycle owners,
allocation grants, or validated authority. Serialization never enters the
physical runtime, and no guard exemption or alternate parser is permitted.

`record_serving/work_semantics/integrity_admission.rs` states the narrow
ordinary-work prerequisite and projects a family-specific admitted resident
view into the established record-serving progression. It does not implement a
validator or create another integrity facade.

C.10 adds maintenance policy as an external caller of
`start_physical_integrity_scrub`; it does not move scrub lifecycle into a
maintenance crate. C.11 adds resident admission siblings for adopted index and
blob frame types only when those types participate in ordinary residency.

### Recovery-ingress composition

```text
workspaces/worth-store/crates/worth-store-recovery-runtime/src/
├── integrity_ingress/                              [create]
│   ├── mod.rs                                      [private orchestration facade]
│   ├── untrusted_source.rs
│   ├── admission.rs
│   ├── admitted_artifact.rs
│   ├── rejection.rs
│   └── counters.rs
└── orchestration/
    ├── manifest_facts.rs                           [replace direct checksum use]
    └── planning/
        └── selected_source_inventory.rs            [replace direct checksum use]

workspaces/worth-store/crates/worth-store-recovery-physics/src/
└── source_precedence/                              [preserve existing C.8 authority]
```

The dominant axis is ordering between media acquisition, integrity admission,
and C.8 interpretation. `integrity_ingress` consumes untrusted bounded media
views and returns the recovery-private family enum. It does not select a
source, classify semantic recoverability, or own public runtime integrity
types.

After cutover, direct calls from recovery orchestration to checksum helpers for
validity decisions are forbidden. Format helpers may still be used behind the
integrity validator for byte grammar. The existing
`worth-store-recovery-physics/src/source_precedence/` owner continues to own
C.8 policy; recovery-runtime orchestration only sequences it and supplies facts
decoded from admitted artifacts. C.9 creates no recovery-runtime
`source_precedence.rs` and moves no C.8 authority.

### Independent offline observer and comparator

```text
workspaces/worth-store/crates/worth-store-offline-integrity-observer/ [create]
├── Cargo.toml                                      [independent manifest]
├── README.md                                       [operator contract]
└── src/
    ├── lib.rs                                      [narrow observation facade]
    ├── integrity_observation/
    │   ├── mod.rs
    │   ├── request.rs
    │   ├── limits.rs
    │   ├── artifact_walk.rs
    │   ├── untrusted_media.rs
    │   ├── duplicate_identity.rs
    │   ├── outcome.rs
    │   ├── localization.rs
    │   ├── counters.rs
    │   ├── report.rs
    │   ├── report_protocol.rs
    │   ├── report_wire.rs
    │   └── families/                              [one file per current family]
    ├── comparison/
    │   ├── mod.rs
    │   ├── request.rs
    │   ├── disagreement.rs
    │   ├── outcome.rs
    │   └── counters.rs
    └── bin/
        ├── physical_store_integrity_observer.rs
        └── physical_store_integrity_observer/
            ├── arguments.rs
            ├── observation.rs
            ├── comparison.rs
            └── report_output.rs

committed successors:
src/integrity_observation/families/index/            [successor C.11]
src/integrity_observation/families/blob/             [successor C.11]
src/integrity_observation/report_v2/                 [successor only on v2]
```

The dedicated crate boundary makes independence Cargo-visible.
`integrity_observation/families/` owns separately implemented grammar and
checksum decisions; `artifact_walk.rs` owns bounded traversal; `comparison/`
consumes two serialized foundational observations and never a Store root.

The existing `worth-store-offline-verifier` keeps its C.8, backup, forensic,
disaster-recovery, and other responsibilities and is not C.9 evidence. Phase 1
classifies its imports of integrity surfaces being removed; those consumers
move to their actual owner without broadening C.9's new crate or applying its
manifest ban to unrelated legacy responsibilities.

### Decisive process proof

```text
workspaces/worth-store/crates/worth-store-physical-certification/src/
└── c9_integrity_localization/                       [create]
    ├── artifact_editor.rs
    ├── clean_artifact_manifest.rs
    ├── corruption_operator.rs
    ├── counters.rs
    ├── editor_result_audit.rs
    ├── external_report_paths.rs
    ├── external_report_paths_tests.rs
    ├── frame_checksum.rs
    ├── mod.rs
    ├── oracle_expectation_assertions.rs
    ├── parent_oracle.rs
    ├── process_courtroom.rs
    ├── process_courtroom_assertions.rs
    ├── process_identity_substitution.rs
    ├── process_manifest.rs
    ├── process_poison.rs
    ├── process_protocol.rs
    ├── process_recovery_observation.rs
    ├── process_subject.rs
    ├── producer_fixture.rs
    ├── production_store.rs
    ├── recovery_adapter.rs
    ├── recovery_request.rs
    ├── root_artifact_role.rs
    ├── scenario.rs
    ├── test_world.rs
    ├── tests.rs
    ├── wire.rs
    └── wire_tests.rs

scripts/ci/
└── run_worth_store_c9_root_process_courtroom.py
```

This is test orchestration, not a new certification framework. It reuses
established child-process and scenario primitives where those primitives do not
predetermine integrity conclusions. It must not add transcript replay bundles,
evidence-authority types, production proof catalogs, or nested Cargo runs.

The production executable entrypoints remain in their owner crates. The
certification module coordinates them and owns only scenario protocol, typed
corruption operations, independent expectations, and assertions.

### Compile-time enforcement topology

```text
workspaces/worth-store/crates/worth-store-physical-integrity/tests/
└── ui/                                              [create/consolidate]
    ├── family_substitution.rs
    ├── scope_substitution.rs
    ├── proof_escape.rs
    ├── proof_construction.rs
    └── owner_valid_admission.rs                    [compile-pass counterparts]

scripts/ci/
└── check_worth_store_integrity_dependencies.*       [create if boundary-check
                                                      cannot express the rule]
```

Use one consolidated UI test target, not one compile crate per case. Prefer
existing `boundary-check` and agent-context dependency rules. A dedicated CI
guard is justified only for a forbidden import or dependency that those tools
cannot express, and it must check manifests and source routes without becoming
a generated proof ledger.

Every governed decoder signature introduced in a phase has an owner-valid
compile-pass counterpart in the same consolidated target. Phase 2 covers each
contract shape it installs; Phase 3 and every Phase 4/5 family packet add the
corresponding concrete owner path before its negative substitution case can
count. A compile-fail result without its compiling owner-valid counterpart is
not evidence because an unusable API or broken harness could produce the same
failure.

## Placement And Export Enforcement

The following rules are mechanical acceptance conditions:

- runtime-integrity `lib.rs` contains explicit exports; glob export of
  `evidence`, `authority`, `operational_repair`, or family internals is
  forbidden;
- family validated-view fields are sealed; only the validator constructs them,
  and they expose no governed decoder entry;
- Store/recovery admitted-wrapper fields and constructors are `pub(crate)` or
  more restrictive in their respective owner crates;
- owner-facing decoders do not expose a raw-byte entry that bypasses their
  family admission; lower physical-format raw mechanisms are callable only
  from the guarded validator, dirty-frame, and format-owner locations above;
- recovery-runtime source interpretation accepts only its private
  `IntegrityAdmittedRecoveryArtifact`, which binds one validated view to the
  actual C.4 bounded read;
- Store ordinary-load composition starts from a live C.6 lease/guard, creates
  descriptive validation input, matches/commits the frame-owned record, and
  returns a lifecycle-bound admitted resident view;
- the dedicated offline-integrity-observer manifest and all of its source
  imports contain no runtime-integrity, Store, recovery-runtime, repair,
  maintenance, operations, or legacy offline-verifier edge;
- quarantine modules expose observations only and contain no filesystem
  mutation verbs or media capabilities;
- Signal, scheduler, allocation, lifecycle, and `Drop`-dependent types remain
  in Store/runtime owners, never in the pure integrity crate; and
- each Rust source and test file stays at or below the workspace 400-line cap
  unless an explicit repository exemption exists.

Flat files such as `helpers.rs`, `common.rs`, `util.rs`, `shared.rs`, a generic
`integrity_types.rs`, or a single all-family validator are forbidden. Likewise,
the topology forbids placing offline parsing under runtime `artifact/`, putting
recovery precedence under integrity, or putting scrub scheduling into the pure
mechanism.

## Current-Surface Migration And Deletion Inventory

Phase 1 must produce a reviewed in-spec implementation inventory in the plan or
PR description; it is not a generated repository artifact. At minimum it maps:

| Current responsibility | C.9 destination | Required action |
|---|---|---|
| checksum algorithm/declaration/execution | physical-format declaration facade and canonical codec mechanism, including `wal_frame/` and `physical_work_obligation/` | retain one writer/runtime mechanism; move WAL/PW byte mechanisms and delete their old copies in Phase 2; remove integrity duplicates; offline remains independent |
| C.4 namespace identity | C.4 runtime owner plus physical-format declaration and offline observation | retain one C.4 admission authority; create no C.9 runtime duplicate |
| C.5.1 physical-work obligation | `artifact/physical_work_obligation.rs` plus Store lifecycle adapter | add canonical family validation, Store-private source-bound admission, and offline observation; Store retains obligation lifecycle |
| physical-format `offline_walk` I/O/decode/classification | dedicated offline-integrity observer for C.9; legacy verifier/WAL owner for their existing non-C.9 duties | replace C.9 behavior independently; move lawful legacy duties to those owners; retain declarations only in format; remove `offline_walk` |
| pre-decode byte view and validation | `validation/{untrusted_artifact.rs,artifact_scope.rs,validated/**}` plus Store/recovery-private source-bound admission wrappers | remove Store imports from pure validation; no validation result opens a decoder |
| Store allocation and lifecycle coupling | Store `physical_runtime/integrity/` | move; delete old constructors |
| WAL integrity family | `artifact/wal.rs` and `validation/validated/wal.rs` | preserve physical validation only; owner-private admission remains above integrity |
| manifest integrity family | named root, segment-membership, extent-manifest, and free-space granules | remove source-precedence and owner policy from integrity; dormant segment-manifest codec is not current closure |
| container/page integrity | `validation/validated/{page_frame,extent_chunk}`; Store/recovery retain segment/extent container traversal and private admission | replace generic container authority; do not preserve a competing lane |
| index/blob integrity | committed C.11 `artifact/{index,blob}` and `validation/validated/{index,blob}` insertion points | remove current generic C.9 index/blob surfaces and sentinel validators; create no C.9 adapter |
| runtime-integrity offline classification | legacy `worth-store-offline-verifier` for C.8/backup/DR/forensic classification | move lawful classifier/posture meaning into that owner with an independent implementation; operations maps results into operations-owned repair contracts; remove physical-integrity `offline_classification` and all shared imports |
| recovery handoff | recovery-runtime `integrity_ingress/` | replace; C.8 handoff remains C.8-owned |
| generic integrity authority/evidence bundles | none | remove after consumers adopt concrete owner types |
| quarantine authority/receipt/request | observation only in C.9; mutation deferred to S.10 | remove mutation-shaped authority |
| operational repair | S.10 operations owner | move continuing contracts directly to operations and remove them from C.9 without an alias |
| compaction source clearance | `worth-store-physical-isolation` C.11 owner | replace `CompactionSourceIntegrityClearance` with owner-local compaction admission derived from C.9 observation plus C.11 truth; remove the C.9 clearance and move its fixture to the owner |
| certification-only proof reports | direct tests and process scenario | delete rather than adapt |

Every external consumer of `worth-store-physical-integrity` is classified as a
runtime mechanism consumer, Store composition consumer, artifact-owner
consumer, test-only observer, successor owner, or obsolete island. No consumer
is grandfathered by adding an alias. Unrelated pre-existing debt outside the
complete C.9 dirty set remains reported debt and does not authorize a broader
rewrite.

### Phase 1 frozen implementation inventory

This is the reviewed Phase 1 decision lock. The detailed source-cited audit may
live in the implementation plan or PR description; implementation must not
generate or check in another inventory authority. In the tables below, `O`
means ordinary Store, `R` means recovery, and `F` means independent offline
observation.

#### Canonical persisted granules

| Persisted granule | Current version and checksum coverage | Current owner and routes | Authority posture | C.9 destination and action |
|---|---|---|---|---|
| `namespace/identity` | identity encoding v1; SHA-256 over bytes `[0,40)`, digest `[40,72)` excluded | C.4 backend writes and admits; O/R consume C.4 identity; no independent F | C.4 trust anchor | keep runtime admission in C.4; add declaration and independent F observation; no second C.9 runtime authority |
| `namespace/mutation.lock` contents | text v1; no checksum | backend/OS lock owner; O/R inspect role/type, not contents | contents are non-authoritative metadata | exclude as an integrity family; F observes namespace/type only; any future protected contents require a new format |
| `families/physical-work/*.pending` | obligation v6; SHA-256 over `[0,128)`, digest `[128,160)` excluded | C.5.1 Store writes and reopens; no current R/F | authoritative while present; malformed file is residue | add `physical_work_obligation`; retain Store lifecycle, add admission and F observation |
| bootstrap catalog | durable frame schema 2 / record declaration 1; CRC32C over `[0,44) + [48,N)` | Store writes; O/R decode; no F | published authority; staging candidate/residue | root/bootstrap declaration, sealed validated view, and owner-private source admission; cut all raw routes |
| current-root selector | same durable-frame envelope; selector kind | Store writes; O/R, reopen, and cleanup decode; no F | current root locator authority | distinct root/current-selector view; must be admitted before it issues root generation/locator |
| previous-root selector | same durable-frame envelope; selector kind | Store writes; R decodes; no F | retained rollback authority | distinct root/previous-selector view; validity cannot select fallback |
| root manifest | same durable-frame envelope; root kind | Store writes; O/R decode; no F | selected or retained authority; staging candidate/residue | root/manifest view; Phase 3 selector-plus-root cut |
| root-routing block | same envelope; parent CRC32C covers complete child `[0,N)` | Store writes; O/R recursively decode and recheck | authoritative reachability | root/routing-block view; remove distributed child CRC calls |
| segment-membership block | same envelope; parent CRC32C covers complete child `[0,N)` | Store writes; O/R recursively decode and recheck | authoritative record/page membership | `segment_membership`; do not call it a current segment-manifest file |
| inline page frame | same envelope; exactly declared page size; frame is a range in a segment artifact | Store writes; O/R decode; no F | authoritative record bytes | `page_frame`; admit the exact range, not a fabricated page file |
| extent manifest | same envelope; fixed manifest kind | Store writes; O/R decode; no F | authoritative geometry | `extent_manifest` |
| extent chunk frame | same envelope; chunk ordinal identity plus extent/record binding | Store writes; O/R exact-range decode; no F | authoritative record bytes | `extent_chunk` |
| free-space header | same envelope; root stores complete-child CRC32C | Store writes; O/R decode and recheck | allocation authority | `free_space/header`; remove Store/recovery direct CRC decisions |
| free-space-membership block | same envelope; parent CRC32C covers complete child `[0,N)` | Store writes; O/R recursively decode and recheck | allocation authority | `free_space/membership_block` |
| WAL frame | WAL v1; payload SHA-256 and header-plus-payload SHA-256 footer | WAL encoder/Store append; O/R owner scanners; existing owner-side offline verifier | authoritative log; empty/interrupted tail is residue | `wal`; independent F implementation; external whole-segment digest is observation, not self-persisted truth |
| checkpoint stream header | checkpoint record schema 1; record CRC32C | Store writer; O/R stream reader | checkpoint authority/candidate | distinct checkpoint stream-header view |
| checkpoint dirty-basis record | schema 1; record CRC32C plus ordered dirty-record SHA-256 aggregate | Store writer; O/R | recovery basis | distinct checkpoint dirty-basis view |
| checkpoint binding-compaction header | schema 1; record CRC32C | Store writer; O/R | compaction/binding context | distinct checkpoint binding-compaction view |
| checkpoint binding record | schema 1; record CRC32C plus ordered binding-record SHA-256 aggregate | Store writer; O/R | recovery binding | distinct checkpoint binding view |
| checkpoint footer | schema 1; record CRC32C; stores selective aggregates | Store writer; O/R | stream closure/count evidence | distinct checkpoint footer view; do not claim one whole-file checksum |

`DurableSegmentManifest` has a codec, reserved locator, tests, and recovery-media
method but no production Store writer or ordinary reader. It is dormant
format-owner residue, not a current family. Index and blob likewise have no
canonical ordinary Store artifact route. All three remain `Unsupported` for
C.9 and may become real only through a successor owner with a production
writer, ordinary reader, declaration, and cutover proof.

The common frame's `identity` word is family-specific, not uniformly a physical
generation: selectors use selector identity, routing/free-space blocks use
block identity, pages and extent manifests use generation, and extent chunks
use ordinal. Family declarations and scopes must use the concrete meaning.

#### External consumer disposition

| Current direct or semantic consumer | Frozen classification and destination |
|---|---|
| `worth-store-aspect-native` canonical registry | semantic registry consumer without a Cargo edge; remap checksum coverage to physical-format, descriptive observation DTOs to foundational, and scrub/quarantine lifecycle to Store; remove generic integrity evidence and closeout entries |
| `worth-store-blob-chunks` | C.11 successor owner; remove generic checked-frame, denial, handoff, and repair imports; no C.9 blob adapter |
| `worth-store-certification` | test observer; retain behavioral assertions through real family validation and owner-private admissions; delete generic proof construction |
| `worth-store-formal-models` | test/model observer; model descriptive observation only, never quarantine or handoff authority |
| `worth-store-layout-indexes` | C.11 successor owner; move any continuing readmission/disposition contract to layout ownership and remove C.9-minted authority |
| `worth-store-maintenance` | future C.10 policy caller; Store owns managed scrub lifecycle, integrity owns pure window inspection |
| `worth-store-offline-verifier` | preserve C.8/backup/DR/forensic roles; move its lawful vocabulary local and remove runtime-integrity classification imports; never feed the new observer |
| `worth-store-operations` | S.10 repair owner; move continuing repair region/plan/receipt contracts here immediately, with no C.9 alias |
| `worth-store-physical-certification` | test observer/orchestrator; use direct production-root scenarios and typed observations, not proof bundles |
| `worth-store-physical-isolation` | C.11 compaction owner; derive any clearance under C.11 from concrete observations and C.11 truth, not C.9 authority |
| `worth-store-test-support` | test observer only; fixtures supply untrusted canonical bytes and observe real ingress; remove denial/proof/quarantine minting |

The transitive wrapper chains through readiness, layout, analysis, isolation,
replication, retention, tiering, and subscriptions inherit the direct owner's
decision; they do not create extra C.9 facades. The canonical aspect registry
is remapped in the same migration: format owns checksum coverage; Store owns
scrub/quarantine lifecycle; foundational owns descriptive DTOs; generic
integrity evidence and closeout entries are removed.

#### Raw-route and deletion matrix

| Route family | Current bypass | Owning cutover packet and required zero |
|---|---|---|
| C.4 trust anchor | backend decodes namespace identity before C.9 | C.4 remains sole runtime owner; zero alternate namespace-identity runtime admission |
| physical-work obligation | `PhysicalEffectJournal::inspect` reads each bounded pending record and calls Store-local `decode_locator` directly | Phase 5 physical-work packet admits each exact obligation before locator interpretation, keeps lifecycle in Store, and adds independent offline observation; zero production raw persisted-input `decode_locator` entry |
| bootstrap catalog | ordinary Store open and C.8 conditional fallback call `BootstrapCatalog::decode` on raw bounded bytes | Phase 5 bootstrap packet redirects ordinary open and conditional recovery fallback together; zero raw production `BootstrapCatalog::decode` outside format-owner tests, while C.8 alone decides whether fallback is needed |
| root protocol | selector predecode addresses root; selector/root decode repeats in C.8, reopen, cleanup, and Store open | Phase 3 redirects current, previous, successor, reopen, cleanup, and clean-serving selector/root callers together; zero raw production selector/root decode |
| recursive routing | raw root/segment/free-space child decode plus direct complete-child CRC in four ordinary Store readers and nine recovery readers | Phase 5 family packets redirect both O and R; zero persisted-input `durable_artifact_checksum` validity decisions in ordinary Store or recovery, while writer/candidate construction remains |
| checkpoint | ordinary Store reopen directly decodes five raw record kinds and recovery enters the raw stream inspector | Phase 5 checkpoint packet redirects ordinary reopen and recovery together; zero raw production checkpoint-record decode or inspector entry outside format-owner validator tests |
| WAL | raw owner scanner interprets header/length before digest | Phase 5 WAL packet; zero recovery raw WAL inspector entry while C.8 torn-tail policy remains |
| inline page | raw frame is inspected/decoded repeatedly in ordinary and recovery routes | Phase 5 page packet; admitted page projection only; zero clean raw page decoder entry |
| extent | raw manifest/chunk decoders and repeated page-LSN decode | Phase 5 extent packet; zero clean raw extent decoder entry |
| resident clean bytes | `PhysicalFrameLease` and `LoadedPhysicalFrame` expose raw slices; published-tail copies to untyped `Vec` | Phase 5 Store lifecycle packet; governed clean decoders require admitted views, record invalidates on every named transition |
| legacy generic authority | broad integrity facade mints evidence, handoff, repair, quarantine, and clearance | owner moves begin Phase 2 and complete Phase 8; zero compatibility aliases or generic authority constructors |
| offline observation | format `offline_walk`, legacy verifier, and WAL owner verifier execute shared runtime decisions | Phase 7 dedicated crate imports declarations only; zero runtime parser/validator/classifier edges |

Writer-side checksum construction for newly encoded candidate bytes remains in
the canonical writer/format mechanism. It is not a persisted-input admission
route and must not be deleted by the recovery expected-zero searches.

The four ordinary persisted-input checksum callers are
`record_serving/admission/open.rs`,
`record_serving/access/segment_membership.rs`,
`record_serving/access/manifest_routing/reader.rs`, and
`record_serving/planning/free_space_routing/reader.rs`. The nine recovery
callers are `orchestration/manifest_facts.rs`, the three decisions in
`orchestration/planning/selected_source_inventory.rs`, the two decisions in
`orchestration/planning/successor_candidate_observation/free_space.rs`, its
`root_routing.rs` and `segment_membership.rs` siblings, and
`worth-store-recovery-physics/source_precedence/page_facts.rs`. Construction
calls under Store planning/residency or recovery `progression/planned` are not
persisted-input validity decisions.

The ordinary checkpoint bypasses are all in
`physical_runtime/durability/checkpoint/reopen/binding_compaction.rs`:
`decode_checkpoint_binding_record`, `CheckpointStreamDecoder::begin`,
`CheckpointStreamFooter::decode_record`, and
`CheckpointBindingCompactionHeader::decode_record`. Recovery enters
`inspect_checkpoint_stream` from
`worth-store-recovery-runtime/src/orchestration/discovery/observation.rs`.
Their Phase 5 packet moves together; none is left as a later cleanup route.

#### Decisive feature and version coverage

The decisive C.9 scenario builds distinct producer, recoverer, observer, and
parent/editor roles. It covers the current physical record declaration v1 with
frame schema 2 and 16/32/64 KiB page declarations, WAL v1, checkpoint record
schema 1, namespace identity v1, and physical-work obligation v6. Each current
granule above has a clean row and every applicable format-aware corruption
operator. Mutation-lock contents, dormant segment manifest, index, blob, and a
stale supported format version are explicit not-applicable rows, not simulated
closure. Unsupported future versions remain mandatory. A supported stale row
becomes mandatory only when a real coexistence window is introduced.

The reusable C.7/C.8 process launch, child lifecycle, canonical-path inequality,
fresh recovery, and bounded protocol primitives are harness inputs only. The
new observer cannot reuse their parser, checksum, classification, report
conclusion, broad process-failure oracle, or recursive-copy behavior. Exact
localization, nonzero selected-test counts, independent parent expectations,
decoder-entry counters, checksum-pass counters, hostile traversal bounds, and
role-bound reports supply the C.9 proof.

#### Owner decisions that implementation must not reopen

- C.4 remains sole runtime owner of namespace identity admission; C.9 shares
  only its declaration and offline observation.
- C.5.1 physical-work obligations are in C.9 coverage and remain Store-owned
  lifecycle state.
- S.10 operations receives continuing repair contracts; C.9 exposes no repair
  compatibility surface while that move occurs.
- `worth-store-layout-indexes` receives owner-local C.11 disposition,
  observation-binding, and readmission contracts; its generic C.9 proof route
  is removed. `worth-store-physical-isolation` replaces C.9 clearance with its
  owner-local compaction admission and owns the corresponding fixture.
- Maintenance remains a caller of the Store-owned scrub lifecycle.
- Recovery owns source precedence and emits descriptive recovery facts;
  operations owns repairability policy.
- The legacy verifier owns its remaining non-C.9 DTOs locally. Shared
  foundational values are descriptive only.
- Phase 3 is one selector-plus-addressed-root vertical cut across every
  production root decoder surface, not a current-root-manifest-only adapter.
- Checkpoint and first-WAL-frame admission use staged sealed envelope fields;
  they do not infer absent scope from unchecked payload or move C.8 policy into
  C.9.
- The resident record lives in `FrameEntry`; the buffer pool adds explicit
  frame generation and Store binds lifecycle generation. No side table or
  loading-identity surrogate is permitted.

## Parallel Construction And Merge Law

The phase numbers below are acceptance order, not a demand that one person
finish every edit in a phase before anyone prepares later work. C.9 is built as
a wavefront: independent owners may develop against the latest admitted
contract commit, but a later truth may merge or become a dependency only after
its predecessor gate is green. Private preparation may run ahead; public
exports, production routing, authority consumption, and deletion follow the
ordered gates.

One integration owner maintains the authority spine. Parallel lanes own
artifact families or structurally separate runtimes, not fragments of the same
semantic decision. The integration owner resolves contract changes, owns the
shared composition choke points, and admits lane commits one at a time so a
defect can be reverted at family or route granularity.

### Sequential authority spine

```text
inventory truth
    -> contract and dependency freeze
    -> one root vertical proof
    -> all-family validation
    -> recovery and clean-residency cutover
    -> bounded scrub and quarantine observation
    -> independent offline completion and comparison
    -> courtroom closeout
```

This spine is not parallelized. In particular, all-family fanout does not
precede the root vertical proof, the all-family recovery/residency cutover does
not precede complete all-family validation, and scrub or offline comparison
cannot define facts that their predecessor owners have not established. The
Phase 3 root-protocol packet is the sole earlier vertical cut and includes
complete validation for every selector/root family and production route it
redirects.

### Contract commit before fanout

Before Phase 2 implementation fans out, the integration owner lands one
reviewed contract commit that freezes:

- the canonical artifact-family/version/checksum-coverage matrix;
- the concrete artifact identities and cross-runtime descriptive vocabulary;
- exact untrusted-scope validation, owner-private source-bound admission,
  rejection, and localization shapes;
- the separation of validator outcome, owner disposition, and quarantine;
- format-declaration and literal-golden-vector surfaces;
- dependency direction, destination modules, visibility, and public exports;
- the shared composition choke points and their single owners; and
- the focused verification commands and expected-zero route searches.

The commit contains real contracts used by the root slice, not empty future
modules or generic placeholders. Every worker branch starts from this commit
or rebases onto a later admitted replacement before merge.

### Shared choke-point ownership

At the start of each wave, the implementation plan names exactly one owner for
workspace and crate manifests, public `lib.rs` and facade `mod.rs` files,
shared protocol identities, foundational enums, boundary rules, and the Store
or recovery composition root. Other lanes do not opportunistically edit those
files. They deliver a narrow export request and the tests that justify it; the
integration owner performs the connector edit.

Ownership may be reassigned explicitly between waves, but never overlap. This
rule does not justify a `common`, `shared`, helper bag, generic artifact enum,
or umbrella adapter. Disjoint work means disjoint semantic owners and file
subtrees.

### Parallel wave map

| Calendar wave | Admitted prerequisite | Concurrent construction lanes | Work permitted to merge |
|---|---|---|---|
| 1 | this specification | format/version/checksum coverage; runtime-integrity callers and removals; recovery/raw routes; residency/frame lifecycle; offline/process/tests/docs | Phase 1 only, after one reconciled matrix and deletion inventory has no unresolved owner or route |
| 2 | Phase 1, then the contract commit above | foundational observation protocol; format declarations and golden-vector grammar; runtime validation core; C.6/Store lifecycle adapters; offline-crate dependency skeleton | Phase 2, after manifests, sealed types, compile denials, and the real root contract agree |
| 3 | Phase 2 | complete current-selector, previous-selector, and addressed-root-manifest validation plus owner-private source admission at every existing root route; independent offline root reader; external editor/oracle; process protocol and route counters | Phase 3, after the complete selector/root operator matrix and every named positive route pass, the real root courtroom passes, and the old raw routes are zero |
| 4 | Phase 3 | separate runtime lanes for page, extent, WAL, checkpoint, root-routing/segment-membership, and free-space; owner-disposition/vector and dependency/UI lanes; separate Phase 7 offline-family preparation may begin from declarations under different owners | Phase 4 family packets only; advance offline work remains private and unexported |
| 5 | Phase 4 | recursive root-routing/segment-membership/free-space recovery; WAL/checkpoint recovery; page/extent clean residency; C.6 frame record and dirty lifecycle; route/dependency deletion checks; private scrub-window and hostile-traversal preparation | Phase 5 packets, one family at a time after all consumers redirect and the old route is deleted |
| 6 | Phase 5 | Phase 6 pure scrub, scheduling, close, quarantine, and boundedness lanes run concurrently with Phase 7 offline traversal, observation, comparison, protocol, CLI, process-proof, and documentation lanes | Phase 6 is accepted first; Phase 7 then rebases on that integrated head and may merge without reimplementing its private preparation |
| 7 | Phase 7 | implementation review; test-evidence review; topology/composition review; documentation and deletion audit | Phase 8 fixes, final independent gate, and complete acceptance evidence |

Phase 4 is the widest useful construction wave and may sustain six to eight
coding lanes because its family/runtime ownership is already frozen. Phases 1
and 2 usually sustain three to five lanes; Phases 5-7 usually sustain four to
six. Review lanes are additional concurrency, not substitutes for construction
ownership. More workers are not admitted when they would share a choke file or
make the same authority decision.

Runtime and offline implementations of the same family must have different
owners and may share only the canonical format declaration and literal bytes.
This makes apparent agreement less likely to be shared-defect agreement.
Advance Phase 7 work in Waves 4 and 5 cannot depend on unmerged runtime family
branches; it is rebased from approved contracts and remains disposable until
its own merge gate.

### Family cutover packets

Phase 5 merges continuously; it does not accumulate branches for one final
connector. A cutover packet contains the family validator and sealed validated
view, the owner-private admission wrapper/binding at each genuine source join,
all affected ordinary and recovery consumers, focused evidence, expected-zero
route searches, and deletion of the superseded path. A packet is incomplete if
it needs a compatibility alias or leaves one caller for a later cleanup wave.

Shared recovery source precedence remains under one recovery-composition owner.
Family lanes supply sealed validated inputs and typed rejections; the named
Store/recovery composition owner alone binds Intact results to actual source
incarnations. Neither may change
candidate ordering, fallback, or selection. Shared resident lifecycle remains
under one Store-composition owner; family lanes cannot invent generation or
dirty-state policy.

### Connector and adapter law

A connector applied by the integration owner is lawful only when it is
mechanical and exhaustive. It may translate representation after consuming the
exact sealed proof, route a concrete family to its named owner, or attach a
lifecycle record whose authority already exists. It may not:

- infer identity, generation, expected scope, rebuildability, or source
  precedence;
- collapse intact, damaged, unsupported, unknown, or indeterminate validation
  outcomes;
- default, retry, fall back, or select a recovery candidate;
- accept raw bytes, stringly family tags, generic authority markers, or
  unsealed evidence;
- add a compatibility alias, parallel decoder, or second truth source; or
- change validation count, I/O bounds, cancellation, or effect settlement.

If a connector needs any such decision, the lanes discovered a missing
contract. Work touching that contract pauses; the specification and contract
commit are corrected, affected lanes rebase, and the root proof is rerun when
the change reaches its authority or observation boundary. An adapter must not
hide disagreement merely to preserve parallel velocity.

Private mechanical changes remain lane-local. Additive changes that preserve
meaning and authority are integrated centrally and rebase only affected lanes.
Changes to public type shape, authority, protocol meaning, family identity,
source precedence, lifecycle, failure classification, or contractual cost are
semantic changes and invalidate every dependent lane until reconciled.

### Worktree, review, and advancement protocol

Each coding lane works in an isolated worktree or equivalent isolated branch
from the admitted contract commit, owns a declared subtree, and runs its
focused tests before handoff. Its handoff names changed contracts, requested
exports, routes deleted, routes still expected, and verification performed.
The integration owner rebases or applies lanes onto the current integration
head and runs the affected cross-boundary evidence after every admitted packet.

Implementation correctness, test honesty, and composition/topology are
reviewed as independent parallel lanes throughout a wave. They may report
against rolling diffs, but only the merged final diff can clear a phase. One
independent phase gate runs after all accepted fixes and repository checks; an
earlier review of a superseded branch is not closure.

A phase advances only when its work is present together on the integration
branch, its focused and inherited gates pass, its deletion conditions hold,
and no old route or competing authority remains. This keeps bugs local to the
family packet or composition seam that introduced them instead of deferring a
multi-phase integration blast radius.

## Ordered Implementation Phases

The phases are accepted in the authority and proof order below under the
parallel construction law above. Implementation may split a phase into
reviewable commits and prepare independent later work, but a governed consumer
may never retain old and new routes in the same admitted commit. Closeout-only
gates are enforced after all families cut over; family route/deletion gates
become mandatory in the commit that redirects that family.

### Phase 1: freeze the artifact, caller, and removal truth

**What becomes true**

Every persisted C.5-C.8 artifact and every integrity-related public consumer
has one reviewed owner, format version, checksum coverage, ordinary/recovery
read route, authority posture, and C.9 destination. The implementation plan can
name every path that must move or die.

**Consumes**

C.5-C.8 specifications, current manifests, current physical-format sources,
current runtime/recovery/offline callsites, and the destination tree in this
specification.

**Establishes**

- the canonical artifact matrix;
- the external-consumer classification;
- the direct raw/checksum/decode bypass inventory;
- a preserve/move/replace/remove decision for each current integrity surface;
  and
- exact feature/version coverage for the decisive C.9 scenario.

**Mechanically forbids**

Uninventoried compatibility aliases, activating reserved checksum fields,
claiming index/blob coverage for a family without an ordinary path, and
deleting a consumer before its real owner is identified.

**Evidence enabled**

A reviewer can trace each future edit to a canonical format and destination;
route searches have a finite expected-zero list.

**Next phase may trust**

The dependency cut changes placement without changing artifact meaning or
losing a lawful consumer.

**Cleanup in this phase**

None beyond obviously dead unreferenced residue whose removal is independently
verified. This phase is observation and decision lock, not a speculative
rewrite.

### Phase 2: invert the dependency and install narrow vocabulary

**What becomes true**

`worth-store-physical-integrity` is lower than Store, owns no live Store
lifecycle, and exposes only the exact descriptive untrusted-input, scope,
family-validation,
rejection, localization, scrub-window, quarantine-observation, and counter
vocabulary required by C.9.

**Consumes**

The Phase 1 inventory and existing concrete foundational/artifact identities.

**Establishes**

- the foundational cross-runtime descriptive observation vocabulary;
- the physical-format declaration facade plus the moved WAL v1 and physical-
  work v6 byte/checksum mechanisms, with their old owner copies deleted;
- bit-exact Phase 2 relocation goldens in
  `worth-store-physical-format/tests/{wal_frame_v1_golden,
  physical_work_obligation_v6_golden}.rs`, installed and passing against the
  current and replacement mechanisms before either old copy is deleted;
- the dependency direction in this specification;
- sealed family-specific `IntegrityValidated*` types and the owned descriptive
  `PhysicalIntegrityValidationRecord`, none of which opens a governed decoder;
- Store-owned adapters for record views, allocation, runtime generation, and
  managed scrub lifecycle;
- Store/recovery-private admitted wrappers that bind validation to an actual
  live C.4/C.6 source incarnation;
- exact rejection/localization vocabulary; and
- consolidated compile-fail tests for counterfeit construction, scope
  substitution, family substitution, and lifetime escape, paired with
  owner-valid compile-pass use of every governed decoder signature introduced
  in this phase.

**Mechanically forbids**

Integrity importing Store, backend, or buffer-pool types; forged marker
authority; public construction of Store/recovery admission wrappers or decoder
entry from a validation result/record;
Signal or scheduler ownership below Store; format declarations containing I/O
or decisions; and offline classification exported from runtime integrity.

**Evidence enabled**

Manifest/boundary checks prove the integrity crate's exact normal dependency set
is `worth-foundational` plus `worth-store-physical-format`, with its old direct
`sha2`, `worth-proof`, Store, authority, contracts, aspect-native, and security
edges absent. Paired UI tests prove public descriptive validation inputs and
sealed validation results cannot open family decoders while owner-private
C.4/C.6 binding can reach each intended decoder shape.

The relocation goldens are an additional Phase 2 deletion gate, not deferred
Phase 4 family evidence:

- WAL v1 freezes one empty-prefix frame with segment `1`, generation `2`, LSN
  `[3,4)`, declared identity string `c9-wal-v1-golden`, and payload bytes
  `10 20 30`. The test contains literal expected 116-byte header, payload SHA-
  256, complete 151-byte frame, and header-plus-payload SHA-256 footer. It checks
  the covered ranges and recomputes both digests with an independent test-only
  SHA implementation, never the moved checksum helper.
- Physical-work v6 freezes Store identity bytes `01 02 03 04 05 06 07 08 09 0a
  0b 0c 0d 0e 0f 10`, runtime `1`, generation `2`, and two operations: operation
  `3`, family `DurabilityBarrier`, target `RecordNamespaceSynchronization`, no
  payload digest; and operation `4`, family `WalAppend`, WAL interval segment
  `7`, generation `8`, offset `9`, byte count `10`, payload digest `ab` repeated
  32 times. The test contains both literal 160-byte records, literal SHA-256
  digests for `[0,128)`, and literal filenames
  `effect-0000000000000001-0000000000000002-0000000000000003.pending` and
  `effect-0000000000000001-0000000000000002-0000000000000004.pending`.

`worth-store-wal` owner tests must map its publication/prefix/LSN contracts to
the frozen WAL format DTO and reproduce the literal frame. Store owner tests
must map its operation/target/identity types to both physical-work format DTOs,
round-trip the exact filenames/identities, and reproduce the literal records.
Before deletion, a temporary migration assertion compares current owner output,
replacement output, and the same literal; the final tests retain the literal
versus replacement and owner-mapping comparisons. Expected bytes and digests
must not be emitted by either encoder or derived from declarations under test.

**Next phase may trust**

One runtime and one offline vertical slice can be built without committing to
an upward dependency or a generic proof that later families must refactor.

**Cleanup in this phase**

Delete superseded generic authority constructors and Store-coupled constructors
as their consumers move. In the same phase, delete the WAL-owner `WORTHWAL`
grammar/SHA implementation and Store-owner `WPEFFECT` v6 grammar/SHA
implementation only after the Phase 2 bit-exact goldens and owner-mapping parity
tests above pass; retain the
owner I/O and lifecycle mappings named above. Do not leave compatibility
re-exports or duplicate private codecs.

### Phase 3: prove and cut one end-to-end root-protocol vertical slice

**What becomes true**

The current-selector, previous-selector, and addressed-root-manifest formats
have complete runtime validators, sealed validated views, and independent
offline readers. Every existing production route that decodes one of those
granules—C.8 discovery/admission,
successor-candidate observation, reopen, cleanup, and ordinary Store open—binds
validation to its exact C.4 source before owner interpretation. A checksum-valid
clean current selector plus addressed root is independently admitted by runtime
and observed by the offline process; a format-aware poisoned selector or root is
rejected by both before root interpretation; the parent localizes it without
using either classifier as oracle.

**Consumes**

The Phase 2 scope/validation and owner-private admission vocabulary, canonical
root format, C.4 bounded media input, and existing child-process protocol
primitives.

**Establishes**

- complete current-selector, previous-selector, and root-manifest validators
  with their sealed family-specific validated views;
- every mandatory `B K L S P T R D U` operator/localization row in both the
  runtime and independent offline lanes for each selector role and for the root
  manifest, including the route-level removal/duplication cases;
- owner-private source binding for the fixed current/previous selector slots,
  staged selector candidates, and the addressed current, previous, and
  successor root manifests those selectors or production plans name;
- the recovery-ingress ordering seam before selector or root decode;
- route-specific positive integration evidence and post-admission entry
  counters for C.8 discovery/admission, successor-candidate observation,
  reopen, cleanup, and ordinary Store open;
- the independent offline C.9 walk/parser/report skeleton with complete
  selector/root-manifest readers;
- the external artifact editor and clean-manifest oracle; and
- decoder-entry and resource counters.

**Mechanically forbids**

Post-decode validation, shared runtime/offline decision code, in-memory
reenactment, arbitrary scribble, report output inside the Store root, deleting a
raw route without its owner-positive counterpart, and leaving any named
selector/root consumer for a Phase 4 or Phase 5 cleanup.

**Evidence enabled**

Focused runtime-owner and independent-offline tests separately prove every
mandatory selector/root-manifest operator and localization row. Runtime-owner
tests then drive clean admitted selectors and addressed roots through every
named production route and assert the expected post-admission route counter is
nonzero. The private vertical test and child-process scenario then convict the
central pre-interpretation fake with a clean current-selector/root control and
independently localized poisoned-selector and poisoned-root cases. The
expected-zero raw decode search counts only when those validator, offline, and
positive route tests pass on the same integrated change.

**Next phase may trust**

The architecture can carry the complete selector-plus-addressed-root protocol
through both independent lanes, and every existing production root route is
known to survive the cutover with the required order, identity, localization,
and cost posture.

**Cleanup in this phase**

Redirect current, previous, successor-candidate, reopen, cleanup, and ordinary
Store selector/root consumers through their owner-private admissions in the
same commit that removes every old production selector/root route. Deletion is
gated by the route-specific positive evidence above, not by an expected-zero
search alone. The public facade waits for complete family coverage in Phase 5.

### Phase 4: complete canonical family validation and owner disposition

**What becomes true**

Every current C.5-C.8 artifact family has a runtime validator and concrete
`IntegrityValidated*` view. Store/recovery owner-private wrappers bind those
views to actual sources at their cutover joins. Artifact owners can project intact/damaged authority and
rebuildable-derived observations without moving recovery or repair policy into
integrity.

**Consumes**

The Phase 1 artifact matrix, Phase 3 vertical pattern, canonical format
declarations, and concrete owner identities.

**Establishes**

- physical-work-obligation and bootstrap-catalog validators;
- the remaining root-tree validators: root-routing-block and
  segment-membership-block, while Phase 3's current-selector,
  previous-selector, and root-manifest validators remain the inherited
  production route;
- inline-page, extent-manifest, and extent-chunk validators;
- WAL-frame and all five checkpoint-record-kind validators;
- free-space-header and free-space-membership-block validators;
- no index/blob validators at C.9 entry; their first real C.11 family packet
  uses the committed insertion points;
- all remaining family operator/localization rows and
  field/range/blast-radius localization, while the already-complete Phase 3
  selector and root-manifest evidence remains inherited unchanged;
- version-window adapters;
- authority-versus-derived owner joins; and
- literal checksum/range golden vectors independently consumed by runtime and
  offline tests.

**Mechanically forbids**

A generic all-family decoder, format invention in integrity, optimistic derived
rebuildability, unsupported-as-corruption collapse, and a new artifact family
without a named insertion point.

**Evidence enabled**

Owner-local focused tests cover clean, framing, checksum, identity, generation,
version, truncation, and substitution laws. When a real current derived family
exists, its owner packet adds derived/authority twins proving that identical
byte damage has different disposition only when concrete owner truth differs;
C.9 entry has no such family and cannot simulate this with index/blob residue.

**Next phase may trust**

Recovery and clean residency can cut over family by family without retaining a
second route for any redirected consumer.

**Cleanup in this phase**

Delete replaced family islands and duplicate checksum declarations after their
callers move. Preserve only one canonical mechanism and one format declaration
per family/version.

### Phase 5: cut recovery and clean residency over together

**What becomes true**

At every applicable current-family join frozen by the Phase 1 route matrix,
owner interpretation receives only admitted artifacts or typed rejection. The
C.8 rule applies to its recovery routes; the C.6 clean-resident rule applies
only to resident families. Physical-work obligations enter through the
Store-owned inventory route, and namespace identity remains delegated to C.4's
sole runtime admission. The C.6 frame owns the clean-validation record;
unpublished dirty decoding remains available only through
`DirtyPhysicalFrame`. C.8 retains source precedence.

**Consumes**

Complete Phase 4 family validation, C.8 source identities/plans, C.4 bounded
media reads, C.6 frame lifecycle, and Store record-serving progression.

**Establishes**

- `integrity_ingress/` in recovery runtime;
- recursive bootstrap and child read expectations;
- admitted bootstrap-catalog fallback and physical-work-obligation inventory
  ingress before owner interpretation;
- admitted recovery-artifact routing;
- integrity observations alongside, not inside, recovery outcomes;
- coexistence evidence for damaged current and intact previous roots; and
- explicit handling of unsupported, unknown, and indeterminate discovery;
- frame-owned clean-validation records and Store resident admission wrappers;
- exact invalidation on dirtying, eviction, reload, reuse, and generation; and
- clean-versus-dirty decoder signatures plus one-hash-per-load counters.

**Mechanically forbids**

Direct recovery validity calls to checksum helpers, clean decode from raw bytes,
C.9 source selection, stale record reuse, per-record rehash, and integrity
outcome disappearance when another candidate succeeds.

**Evidence enabled**

Fresh-process recovery proves poisoned decoder-entry count zero and C.8-only
source choice. Residency tests cover hit, eviction, reuse, reload, mutation,
close, scope substitution, dirty serving, and rehash after invalidation.

**Next phase may trust**

The recovered runtime and every clean resident frame were constructed without
an integrity bypass; scrub can reuse pure inspection without becoming read
authority.

**Cleanup in this phase**

Remove direct `durable_artifact_checksum` decisions from recovery orchestration,
including manifest-fact, selected-source inventory, and successor-candidate
observation paths, and from the four named ordinary Store readers. Remove
obsolete integrity recovery-handoff types rather than aliasing them. Delete
clean raw decoder routes and Store-local duplicate checksum decisions in the
same family commits. Remove raw production `BootstrapCatalog::decode`, all raw
ordinary/recovery checkpoint record entry, and persisted-input physical-work
`decode_locator` entry in their family packets. Test-only raw parsers remain
private to format-owner unit tests.

### Phase 6: install bounded online scrub and non-authoritative quarantine

**What becomes true**

The live Store can inspect declared artifact windows under C.5.1/C.6 budgets,
pause/defer/cancel honestly, close without leaked work, and emit quarantine
observations without mutation or repair authority.

**Consumes**

The admitted recovered runtime, Phase 5 resident composition, pure scrub-window
inspection, Store scheduling/allocation/lifecycle, and exact effect settlement.

**Establishes**

- Store-owned scrub request, handle, progress, resume, cancellation, and close
  behavior;
- bounded resource and amplification counters;
- generation-scoped resume points;
- quarantine observation and posture; and
- the public scrub facade C.10 may later schedule.

**Mechanically forbids**

Whole-Store residency, scheduler bypass, ordinary-work starvation, scrub-owned
repair, reachability mutation, quarantine release, and handle survival across
runtime generation or close.

**Evidence enabled**

The online-scrub siege proves bounded high water, exact cancellation/close
outcomes, ordinary progress, and a byte-for-byte unchanged Store root.

**Next phase may trust**

Offline completion and disagreement can use the same stable descriptive
vocabulary without sharing runtime authority or lifecycle.

**Cleanup in this phase**

Remove legacy scrub scheduling/allocation types from the pure integrity crate
and remove mutation-shaped quarantine request/receipt/authority surfaces.

### Phase 7: complete independent offline truth and disagreement

**What becomes true**

The offline executable independently walks every current family within bounds,
preserves intact/damaged/unsupported/unknown/indeterminate validator outcomes,
emits the version-1 protocol, and can disagree explicitly with runtime
observation.

**Consumes**

Phase 3 offline skeleton, Phase 4 format-family matrix and literal golden bytes,
foundational protocol identity/version types, and the dedicated offline crate.

**Establishes**

- all-family independent offline readers;
- hostile traversal behavior for symlinks, hard links, duplicate identities,
  changing files, high cardinality, and exhausted bounds;
- output-path exclusion from Store root;
- protocol version `1` and compatibility declaration;
- explicit comparison/disagreement output; and
- dependency/source guards proving implementation independence.

**Mechanically forbids**

Runtime parser/validator imports, repair flags, silent consensus, lossy outcome
projection, unbounded traversal, duplicate amplification, and report
serialization of runtime proof types.

**Evidence enabled**

Clean agreement, artifact-localization matrix, hostile-walk, compatibility,
changing-file, and declared-disagreement cases run through the real offline
binary.

**Next phase may trust**

The full decisive courtroom can distinguish runtime admission, C.8 recovery,
offline observation, owner disposition, and quarantine without an authority
cycle.

**Cleanup in this phase**

Delete runtime-integrity `offline_classification` after current consumers move
to their actual non-C.9 owner. Remove any shared classifier/parser experiment.
Move lawful consumers out of physical-format `offline_walk`, delete its
format-owned I/O/decision surface, and retain declaration meaning only.

### Phase 8: close the courtroom, delete islands, and publish the contract

**What becomes true**

All decisive cases pass against production composition roots; every expected
bypass and legacy authority route is absent; public caller and operator docs
describe the real implementation; and C.10/C.11 can enter additively.

**Consumes**

Phases 1-7, the complete hostile matrix, current repository laws, and the
successor handoffs below.

**Establishes**

- the multi-process corruption scenario at required scale;
- controlled-defect sensitivity;
- focused owner tests and consolidated UI evidence;
- final dependency, boundary, line-cap, route, formatting, and agent-context
  closure;
- the narrow public facades and durable documentation; and
- zero remaining C.9-scoped legacy evidence/repair/recovery authority islands.

**Mechanically forbids**

Happy-path-only closure, broad `is_err()` evidence, generated proof systems,
compatibility aliases, dead alternate decoders, and successor work that must
move C.9's established facade or reverse its dependency direction.

**Evidence enabled**

The full acceptance suite below and a reviewed deletion inventory demonstrate
that bypassing or weakening the central admission mechanism turns evidence red.

**Next phase may trust**

C.10 may schedule bounded scrub without acquiring integrity or repair
authority. C.11 may add artifact-family siblings without restructuring C.9.

**Cleanup in this phase**

Delete obsolete proof reports, evidence bundles, operational-repair exports,
test-only authority islands, dead feature flags, stale docs, and retired
callsite adapters. Run the dirty Rust line-cap guard over the complete scoped
change.

## Test Architecture

### Owner-local tests

Tests live with the responsibility they convict:

- physical-format tests prove encoding, byte grammar, version declarations,
  checksum coverage, and golden bytes;
- runtime-integrity tests prove exact-scope validation, family distinction,
  rejection/localization, validated-result sealing, and pure scrub windows;
- Store tests prove private source-bound admission, resident lifetime,
  one-validation-per-load, scheduling,
  bounded scrub, cancellation, close, and quarantine non-authority;
- recovery-runtime tests prove admission-before-interpretation and preservation
  of C.8 source precedence;
- offline-integrity-observer tests prove separate parsing, bounded hostile
  traversal, protocol projection/comparison, compatibility, and read-only
  behavior; and
- physical-certification owns only the cross-process production scenario and
  independent oracle.

Fixture support may generate valid Store worlds and typed corruptions. It may
not call the production validator to construct expected outcomes, prebuild a
damaged production artifact that the runtime never emits, or expose private
production mutation hooks.

### Required case families

The completed milestone includes evidence for:

1. clean runtime/offline agreement on validator facts over every current family;
2. semantically plausible poisoned bytes rejected before decoder entry;
3. checksum, length, generation, pointer, payload, removal, duplication,
   truncation, identity substitution, and stale/unsupported version cases;
4. owner-disposition twins for every real current derived family; this is an
   explicit not-applicable row at C.9 entry because the frozen current matrix
   contains no derived family, and dormant index/blob formats cannot simulate
   it;
5. current/previous-root and WAL/checkpoint recovery-precedence coexistence;
6. explicit runtime/offline disagreement;
7. bootstrap/child expected-scope progression plus family, identity,
   generation, and lifetime substitution;
8. bounded online scrub with denial, pause, cancellation, eviction/reload, and
   close;
9. hostile offline traversal and report-output exclusion;
10. quarantine observation with byte-for-byte and namespace immutability;
11. compatibility/detection distinction;
12. one-validation-per-load, eviction/reuse record clearing, forced rehash, and
    bounded-amplification counters;
13. compile-time dependency, dedicated offline-crate, and facade restrictions;
14. deletion and route gates for every old authority path.

Do not expand this into a ceremonial Cartesian matrix. Each case must name the
defect it convicts and use the smallest fixture that still exercises the real
boundary. The multi-process case carries scale and role separation; local tests
carry exhaustive family mechanics.

### Compile-cost discipline

Use one consolidated UI-test target for negative compilation. Do not create a
crate per case, run nested Cargo from Rust tests, duplicate the entire workspace
for dependency checks, or rebuild the multi-process world once per assertion.
The decisive runner creates one immutable clean baseline per workload seed and
uses filesystem-supported safe copies for isolated corruption rows without
sharing mutable artifacts between rows.

## Documentation Deliverables

C.9 changes a caller workflow, operator workflow, architectural boundary,
failure vocabulary, and compatibility contract. Completion requires these
durable documents:

1. **Create**
   `_docs/worth-store/physical-integrity-and-offline-verification.md` for Store
   callers and operators. It explains:
   - what integrity admission proves and does not prove;
   - ordinary-load and recovery behavior;
   - artifact families and compatibility windows;
   - damage localization and blast-radius interpretation;
   - authoritative versus derived disposition;
   - scrub request, progress, cancellation, resume, resource, and close
     lifecycle;
   - quarantine non-authority;
   - offline command usage, budgets, output location, and read-only guarantee;
   - runtime/offline disagreement; and
   - why repair, recovery selection, semantic health, and readmission are
     separate operations.
2. **Replace** the public-surface sections of
   `workspaces/worth-store/crates/worth-store-physical-integrity/README.md` so
   they match the narrow facade and dependency direction and no longer present
   evidence bundles, repair, generic authority, or offline classification as
   integrity responsibilities.
3. **Revise**
   `crates/worth-foundational/README.md`, its documentation entrypoint, and
   crate/facade rustdocs with the cross-runtime descriptive integrity
   vocabulary, bounded construction, projection examples, and explicit rule
   that the DTOs cannot open runtime, recovery, or repair paths.
4. **Revise**
   `workspaces/worth-store/crates/worth-store-physical-format/README.md` with the
   stable integrity-declaration facade, canonical writer/runtime checksum
   mechanism, offline import restriction, versioning rule, and declaration
   example for a new artifact family.
5. **Create**
   `workspaces/worth-store/crates/worth-store-offline-integrity-observer/README.md`
   with the `observe` and `compare` contracts, compatibility window, traversal
   bounds, report schemas, hostile-filesystem behavior, output-path rule, and
   manifest-level independence restriction.
6. **Revise** `_docs/worth-store/physical-recovery-and-reopen.md` to state that
   C.8 discovery consumes C.9-admitted artifacts while C.8 retains source
   precedence and recovery authority.
7. **Revise** C.10/C.11 roadmap or specifications only where their implemented
   handoffs differ from the successor contract here; do not duplicate C.9's
   full explanation.

The durable guide includes compiling Rust caller examples and executable CLI
examples exercised by documentation tests or focused integration tests. CLI
`--help`, public Rust signatures, protocol fixtures, and documentation examples
are compared in CI or their focused owner tests. Documents made false by C.9
are corrected or removed in the same phase as the associated public surface.

## Must Ship

C.9 ships only as the complete Goal and Closeout Bar, proven by the required
case families. The non-negotiable joins are recovery ingress, frame-owned clean
resident admission, bounded scrub, the dedicated independent observer and
comparator, orthogonal validator/owner/quarantine outcomes, deletion of scoped
legacy routes, and synchronized caller/operator documentation. Partial family
coverage or one missing join is not a smaller shippable C.9.

## Must Preserve

C.9 must preserve:

- C.4's sole media owner, bounded I/O, and indeterminate-effect honesty;
- C.5's canonical formats and checksum coverage unless an explicit versioned
  format migration is separately earned;
- C.5.1's Store-owned Signal and scheduler/executor topology;
- C.6's bounded residency, frame generations, dirty typestate, and exact
  writeback settlement;
- C.7's durability order, publication authority, and acknowledgment meaning;
- C.8's fresh-process boundary, source precedence, redo, and recovered-runtime
  handoff;
- one-way tier and crate dependencies;
- concrete platform authority rather than generic marker bounds;
- pure-meaning Query independence and cert-only replay boundaries;
- ordinary-path cost proportional to newly loaded physical granules, not
  records or queries; and
- the default 400-line cap for every touched Rust source and test file.

## Non-Goals

The Governing Boundary exclusions are normative. In particular, C.9 adds no
authenticity/encryption, repair or recovery choice, semantic health/readmission,
C.10 maintenance policy, C.11 formats/compaction, backup/DR/forensic claim,
whole-Store residency requirement, compatibility alias, or new certification
framework.

## Acceptance Evidence

### Focused owner lanes

Implementation closeout runs the exact package targets selected from the final
dependency inventory. At minimum the evidence includes:

```text
cargo test -p worth-foundational
cargo test -p worth-store-physical-format
cargo test -p worth-store-physical-integrity
cargo test -p worth-store-buffer-pool
cargo test -p worth-store-recovery-runtime
cargo test -p worth-store-offline-integrity-observer
cargo test -p worth-store
python scripts/ci/run_worth_store_c9_root_process_courtroom.py
```

If workspace configuration requires an explicit manifest path, the commands
use `workspaces/worth-store/Cargo.toml`. Owner packages added by the actual
artifact matrix are included; unrelated full-workspace debt is reported
separately.

### Required structural evidence

Closeout proves:

- no Store dependency or source import from runtime integrity;
- no runtime-integrity, Store, recovery-runtime, repair, maintenance,
  operations, or legacy offline-verifier dependency/import from the dedicated
  observer crate;
- no direct recovery validity decision through checksum helpers;
- no clean-path owner decoder accepting raw bytes or a generic marker, and no
  dirty-path decoder accepting anything except `DirtyPhysicalFrame`;
- no clean-validation side table outside the C.6 frame entry;
- no quarantine mutation surface;
- no legacy `evidence`, generic authority, operational repair,
  offline-classification, or integrity recovery-handoff facade in scoped
  production exports;
- no current family omitted from the Phase 1 matrix; and
- all successor directories enter additively under the committed tree.

Exact route searches are captured in ordinary test or review output, not a
generated ledger checked into the repository.

### Repository gates

Before completion, run:

```text
cargo fmt --all --check
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
```

On Windows, invoke the line-cap shell script through the repository's admitted
shell environment. Also run any existing worth-store format, dependency,
feature-matrix, documentation, and public-API gates affected by the final
inventory.

### Decisive evidence posture

The final scenario output must make these facts independently inspectable:

- which production entrypoint was exercised;
- which process held each role;
- which clean artifact and typed corruption were selected;
- whether owner decode was entered;
- what runtime integrity observed;
- what C.8 recovery decided;
- what the offline verifier independently observed;
- whether observers agreed or disagreed;
- which range and blast radius the parent expected;
- what resource and amplification bounds were consumed; and
- whether Store-root bytes or namespace changed during offline/quarantine
  observation.

The output is transient test evidence or an operator-requested offline report,
not a committed proof artifact.

## Closeout Bar

C.9 is closed only when:

- every persisted authority on a current C.5-C.8 path is rejected or admitted
  before semantic, owner-specific, ordinary, or recovery interpretation;
- every current derived artifact is validated before use and is called
  rebuildable only by its owner with an intact current authority basis;
- a successful checksum cannot be reused across Store, family, identity,
  generation, range, reload, eviction, mutation, or runtime generation;
- the C.8 recovery runtime has no direct raw/checksum validity lane beside
  integrity ingress;
- the clean resident path has no raw decoder lane beside frame-owned admission,
  and unpublished dirty decoding requires `DirtyPhysicalFrame`;
- the offline verifier can reach and preserve a conclusion that differs from
  runtime without importing runtime decision code;
- scrub and offline verification remain bounded and read-only;
- quarantine observation opens no repair, reachability, recovery, or semantic
  authority;
- controlled defects prove that bypass, wrong ordering, scope widening,
  stale-proof reuse, shared verification, classification collapse, silent
  disagreement, unbounded work, and legacy routes are detected;
- the scoped destination tree and deletion inventory are real; and
- caller/operator documentation matches the actual facade, protocol, failure,
  lifecycle, compatibility, and cost contract.

Clean bytes plus two agreeing implementations are insufficient if either
implementation can pass through the same disputed decision function. A broad
error is insufficient if it does not prove pre-interpretation refusal and
honest localization. A runtime proof is insufficient if it survives the bytes
or generation it inspected.

## Successor Handoff

### C.10 maintenance and operator workflows may trust

C.10 receives:

- one Store-owned managed scrub facade;
- bounded request, progress, resume, denial, cancellation, close, and
  indeterminate outcomes;
- exact resource and amplification counters;
- descriptive damage, disposition, quarantine, and disagreement observations;
  and
- no implicit permission to repair, mutate reachability, release quarantine,
  choose recovery sources, or change semantic service.

C.10 adds scheduling policy, cadence, prioritization, operator workflow, and
later authorized repair composition as siblings above this facade. It must not
move scrub lifecycle into the integrity mechanism or turn C.9 observation into
authority. Prioritization chooses which bounded scrub handle to advance and
when; changes to resource policy belong to the existing C.5.1 scheduler, never
to a caller label that bypasses its admission or C.6 allocation bounds.

### C.11 indexes, blobs, and compaction may trust

C.11 receives:

- a named additive runtime validator/admission family axis;
- a named additive independent offline family axis;
- Store resident-admission siblings for real resident index/blob granules;
- owner disposition requiring concrete current rebuild basis; and
- format/version/checksum/localization/golden-vector rows that every new
  persisted family must supply.

For each new C.11 artifact, adoption requires one canonical format declaration,
one separately implemented runtime family validator, one separately implemented
offline observer family, one Store or recovery registration at the genuine read
join, one focused owner test family, and one row in the decisive localization
matrix. These additions do not rename or split C.9's facade, reverse a
  dependency, generalize existing validated views or owner-private admission
  wrappers, or place compaction policy
inside integrity.

### Later semantic and operational work may not infer

No successor may infer from C.9:

- that physical integrity proves logical correctness;
- that an offline report can be consumed as runtime admission;
- that `RebuildableDerived` authorizes reconstruction;
- that quarantine changed the reachable Store;
- that damaged authority implies a particular C.8 recovery choice;
- that agreement authorizes semantic readmission; or
- that physical checksum coverage provides cryptographic authenticity.

C.9 hands forward physical evidence, exact scope, honest uncertainty, and
bounded observation—nothing more.
