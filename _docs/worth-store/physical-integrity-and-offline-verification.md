# Physical integrity and offline verification

C.9 separates three questions: are these physical bytes intact, what may their
owner do with them, and what did an independent observer find? A checksum is
neither authenticity nor semantic health. An intact frame does not authorize
recovery selection, repair, readmission, or a mutation.

## Runtime admission and failure meaning

Ordinary clean loads pass through the family integrity validator before the
owner projects bytes into resident meaning. The owner binds that result to the
exact source, Store, frame generation, and lifecycle. Resident reuse carries
that binding; queries do not rehash an already-admitted frame. Recovery uses
the same integrity front door but retains its separate C.8 precedence, redo,
publication, and recovered-runtime handoff rules.

The current families include bootstrap catalogs, current/previous selectors,
root manifests, routing and membership nodes, free-space header/nodes, pages,
extent manifests/chunks, WAL records, physical-work obligations, and checkpoint
header/dirty-basis/compaction/binding/footer records. Unsupported format versions
are not damaged bytes. Version changes require a declaration and explicit
compatibility handling, not silently accepting a new layout.

`Intact` covers the exact inspected scope. `Damaged` includes the artifact,
range, failed relation/field, and defensible blast radius. `Unknown` means an
expected artifact or required parent context was unavailable. `Indeterminate`
means bounds, source change, or incomplete acquisition prevented a conclusion.
These outcomes remain distinct from the owner's authoritative-versus-derived
disposition: a derived artifact may be rebuildable, but integrity cannot decide
to rebuild it. A checksum mismatch cannot identify which covered byte was
originally wrong, so its localization may cover the whole canonical frame.

## Bounded online scrub

Call `ServingPhysicalRuntime::start_physical_integrity_scrub` with explicit
typed targets and their expected scopes. The caller supplies addressing and
bounds, never bytes, a validator callback, a scheduler receipt, or repair
authority. This is a pull-based diagnostic session, not a maintenance policy
or automatic whole-Store discovery service.

```rust
use std::time::Duration;
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIntegrityScrubRequestDenial, PhysicalIntegrityScrubTarget,
    ServingPhysicalRuntime,
};

fn begin_inspection(
    store: &ServingPhysicalRuntime,
    targets: impl IntoIterator<Item = PhysicalIntegrityScrubTarget>,
) -> Result<ManagedPhysicalIntegrityScrubHandle, PhysicalIntegrityScrubRequestDenial> {
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        store.store_identity(), targets,
        64 * 1024,          // largest single acquisition
        16 * 1024 * 1024,   // total declared acquisition bytes
        Duration::from_secs(30),
    )?;
    store.start_physical_integrity_scrub(request)
}
```

Requests reject empty, foreign, overlapping, or overflowing scopes. There are
at most 4,096 targets, 16 MiB per window, 16 active handles per runtime, and a
positive deadline of at most 24 hours. These are hard ceilings, not resource
reservations: C.6 or C.5 may still defer an otherwise valid request.

Each `next_window()` attempts at most one source acquisition. It holds the
actual C.6 `Scrub` allocation and C.5 background capacity through exact effect
settlement and inspection, then releases them before returning. Only one scrub
window runs at once per runtime. Background admission atomically preserves at
least half of each configured scheduler resource for ordinary work; it does
not borrow a foreground receipt and then pretend the capacity is idle.

`WindowInspected` contains the actual outcome, validation counts, acquisition
counts, bounded allocation high water, and optional quarantine observation.
`Deferred` retains the cursor and names allocation, concurrent-window, or typed
scheduler/dependency denial. There is no hidden result queue or retry loop.
`Completed` means all declared targets were visited, not that the whole Store
is intact; examine damaged counts and each outcome. Unavailable/unsupported/
indeterminate windows produce the terminal `Indeterminate` summary.

`pause()` returns an opaque continuation. Only the same handle, runtime
generation, and cursor may resume it. A cancellation token stops new windows;
cancellation after an actual effect preserves that window and its counters.
Terminal pulls are idempotent. Close/abort stops admissions and waits for the
single in-flight scrub window before media teardown. Retained handles then
report `Closed` and cannot reopen the old generation. A deadline cannot interrupt
an OS call: late reads are counted and reported indeterminate, not intact.

Extent chunks require the matching previously inspected manifest. Checkpoint
aggregate validation requires an ordered, contiguous header-to-footer sequence.
Omitted or unrelated parent context is `Unknown`, not corruption. Checkpoint
windows retain one opaque OS source-version observation; publication between
windows invalidates their aggregate and yields source-change indeterminacy.
A stale expected checkpoint identity is unknown. No byte buffer or collection
of file handles is retained between windows.

Quarantine here is a descriptive `DamageObserved` observation. It cannot remove
reachability, revoke unrelated serving authority, alter bytes, release a
quarantine, or authorize repair. Repair/recovery selection/semantic readmission
require their own owner-controlled workflows.

For comparison input, call `handle.write_observation_report(context, max_bytes,
&mut sink)`, with
`worth_store::integrity_observation::PhysicalIntegrityRuntimeReportContext::new(run, scenario)`.
The JSON adapter belongs to Store's diagnostic observation module, outside the
physical runtime; it consumes only bounded scrub observations and descriptive
scope/counter facts, never raw media or admission authority.
This acquires the remaining targets and streams version-1 JSON to a caller-owned
`Write` sink; Store does not allocate a report-sized buffer. Process/executable
identity is sampled by the runtime. The sink must be outside the Store if it is
a file. Source allocations are released before writing each observation.
Sink failure or report-size exhaustion can leave a partial document: discard
it; the handle's actual completed cursor/counters remain available. Paused or
deferred scans emit an indeterminate partial-scope report without retries.
`report_bytes` is exact; source-window high water excludes caller-owned output.
Checksum-call counts are explicitly unavailable (`null`) rather than inferred
from frame counts. The comparator preserves an indeterminate `observed_range`
when the runtime observed only a prefix.

## Independent offline observation

Close or isolate the Store first. The independent executable does not import
runtime parsers, validators, Store, recovery, or repair; only immutable format
declarations and portable report vocabulary cross that boundary. Its own
readers/checksums therefore provide a separate implementation to disagree with.

Build `worth-store-offline-integrity-observer` using the Store workspace manifest.
For an existing closed Store and an already-existing external report directory:

```text
physical_store_integrity_observer observe --store-root <closed-store> --report <external-report.json> --max-entries 10000 --max-bytes 67108864 --max-open-files 16 --max-depth 12 --max-symlinks 0 --max-elapsed-ms 30000 --max-report-bytes 16777216
physical_store_integrity_observer compare --runtime-observation <runtime-v1.json> --offline-observation <external-report.json> --report <external-comparison.json>
```

`observe` never writes into the Store. It rejects output beneath the canonical
Store, uses create-new external output, and can emit to stdout with `--report -`.
It bounds entries, bytes, descriptors, depth, symlinks, elapsed time, and report
bytes. Hostile paths, changing files, aliases, missing expected children, and
exhausted bounds remain explicit evidence. Traversal completeness is not a
universal integrity claim. See the
[observer contract](../../workspaces/worth-store/crates/worth-store-offline-integrity-observer/README.md)
for platform acquisition details and every wire field.

Observation protocol `store.physical.integrity-observation` uses version `1`,
compatibility `[1,1]`; the offline role remains `offline-root-observer` for wire
compatibility despite all-family coverage. Comparison checks schema, bounds,
Store/scenario identity, and distinct runtime/offline process/executable roles.
It retains both inputs and exact disagreements, including same-posture identity,
range, or localization differences. It never chooses a winner or reconciles
observations into authority. Its output must also stay outside the Store; the
comparator does not open the Store to enforce that operational fact.

The Rust example above is included in the scrub module's documentation tests.
Real-media lifecycle tests exercise that facade; observer integration tests
exercise the command grammar, output exclusion, report version, and comparison.
