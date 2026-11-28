# Use an Application Home

An application Home is where a WORTH application keeps its durable state. Store
owns the files; the Query Host opens, initializes, closes and discards the Home
for you. An application never opens Store or lower owners itself: it uses
`worth-query-host` only.

This page is the short path through the lifecycle. Each section links to the
longer contract. `host::` means `worth_query_host::facade`; a name written with a
module (such as `host::primary_graph::..`) lives in that module.

| Location | Durable? | Use it for |
|---|---|---|
| `host::WorthQueryApplicationHomeLocation::Directory(path)` | Yes: survives graceful close and a fresh-process reopen | Applications that must keep their data |
| `host::WorthQueryApplicationHomeLocation::MemoryOnly` | No restart guarantee | Tests and process-local tools |

A Directory Home survives graceful close, quiescent process kills, and the
interrupted-operation crash-recovery cases completed in Milestone 3 Tracks C1–C3
(see [After a crash](#after-a-crash-or-unclean-close)); full adversarial-oracle
and Tier-3 milestone certification runs in Track C4 / Landing 3.

## Open a Home

Build one `host::WorthQueryApplicationHomeRequest` and open it inside a
bootstrap advancement:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. MySchema, MyProgram, configuration, limits and
// seed_graph belong to the application.
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement,
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationProgramRoster},
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

let request = WorthQueryApplicationHomeRequest::new(
    WorthQueryApplicationHomeLocation::Directory(path),
    validated_program,                         // ValidatedApplicationProgram<MySchema, MyProgram>
    WorthQueryApplicationProgramRoster::new(),
    declaration,                               // MySchema::declaration()?
    configuration,                             // one value per contribution
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))?,
    limits,                                    // WorthQueryApplicationLimits
    |_phase, graph, installed| seed_graph(graph, installed), // runs only for a new Home
);
let opened = with_bootstrap_advancement(policy, |phase| request.open_in_advancement(&phase))?;
let home = match opened {
    Ok(home) => home,
    Err(refused) => return Err(close_refused_opening(refused)), // see below
};
let runtime = home.runtime(); // borrow for requests; keep `home` as the owner
```

- `with_bootstrap_advancement(policy, ..)` takes a
  `host::runtime::ExecutionRequestPolicy`. If you already hold an admitted
  `ExecutionRequest`, use `with_bootstrap_advancement_from_request(request, ..)`
  instead. The outer `Result` is the advancement admission; the inner one is the
  Home opening.
- `limits` is `host::application_installation::WorthQueryApplicationLimits`. Give
  every Home in the process the same execution service with
  `limits.with_execution_authority(Arc::clone(&service))`; see
  [One process execution service](build-an-application.md#one-process-execution-service).
- Build identity: every complete build gets a fresh UUID
  (`python scripts/application_build.py cargo build --locked`). See
  [Application build identity](build-an-application.md#application-build-identity).
- Optional request settings: `with_authorization_time_source(source)` for trusted
  capability time, `with_checkpoint_limits(limits)` for checkpoint thresholds
  ([Checkpoints](#checkpoints)), and `with_program_transition(..)` to change the
  program or schema of an existing Directory
  ([Directory development workflow](build-an-application.md#development-workflow-for-directory-homes)).
- Opening runs on WORTH's execution stack, so the application's ordinary
  `main` thread with the default stack is enough.

The returned `host::WorthQueryApplicationHome` owns the Home and its Store
mutation lease. `runtime()` returns the
`host::application_installation::WorthQueryProgramApplicationRuntime` you serve
requests from; `is_directory()` reports the location kind. For
workflows, call `home.retain_workflow_spec(installed_spec, signing_owner)`
before retaining runtime handles. It returns the same Home with its workflow
registered. A `host::WorthQueryWorkflowHomeBindingRefused` gives back the
Home, installed spec and signing owner.


## Initialize once, reopen many times

Store decides whether the Directory is new. Only a new Home calls the
`initial_state` closure; it seeds the graph before the first World root is
published. An existing Directory, including one whose first initialization did
not finish, never calls it again.

To reopen, build a new request for the same `Directory(path)` with the same
supported program and schema, and open it the same way. You still pass an
initializer; it does not run. Reopen readmits the recorded program, data and
retained owners (workflows, dispatch and inbound custody, original operations,
producer outputs). A new build UUID is accepted: stale derived output regenerates
on demand, and authored data is kept.

Every open is a new incarnation. Selectors, capability access and provider-session
grants from an earlier open refuse with `StaleIncarnation`. Authenticate again
and reacquire them from the new `home.runtime()`. Recorded receipts and
descriptors are data, not authority. See
[Reacquire authority after reopen](build-an-application.md#reacquire-authority-after-reopen).

Only one process can own a Directory. A second opener is refused with
`WorthQueryApplicationHomeOpenRefused` (phase
`WorthQueryApplicationHomeOpenRefusedPhase::Physical`); its `physical_refusal()`
returns the typed Store refusal and `detail()` names `MutationOwnerContended`.
If the owner process dies, the OS releases the lease and another process can
open the Home.

## Write, with idempotency keys and retries

Every mutation carries a stable idempotency key chosen by the application:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let outcome = runtime
    .request(&principal, &scope)
    .advanced()
    .mutate(intent)
    .without_source()
    .idempotency(&stable_key)
    .execute_in_program(runtime, allocation_policy)?;
```

Match `host::application_entry::WorthQueryApplicationMutationOutcome`:

| Outcome | Meaning |
|---|---|
| `Committed { receipt, result }` | New commit, with the handler's result. |
| `AlreadyCommitted(receipt)` | Same key and intent, receipt still live in this process. No new effect. |
| `RecoveredCommitted(receipt)` | Same key and intent, committed earlier (for example before a reopen). The `host::primary_graph::WorthQueryRecoveredApplicationCommitReceipt` is recorded data; there is no handler result. Read current state with a fresh query. |
| `IdempotencyIntentDrift` | The key was used for a different intent. Use a new key for new intent. |
| `DomainDenied(denial)` | The handler refused. |
| `Cancelled`, `DeadlineExceeded` | The request stopped. |
| `Commit(uncommitted)` | Did not land cleanly. Keep any pending, unpublished or indeterminate custody it returns; do not treat it as "retry with a new key". |

After a lost response, a timeout or a restart, retry with **the same key and the
same intent**. Never pick a new key just because an answer was lost: a timeout
does not prove that nothing happened. `outcome.recovered_receipt()` returns the
recorded receipt when there is one. Pending and unpublished outcomes and their
recovery routes are described in
[Commit, retry and close a Directory Home](build-an-application.md#commit-retry-and-close-a-directory-home)
and the [Host README](../workspaces/worth-query/crates/worth-query-host/README.md#commitment-and-same-key-retry).

## Close

Drop request and read owners and any runtime aliases, then consume the Home:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. `request` is a host::primary_graph::ExecutionRequest.
match home.close(&request) {
    Ok(closed) => closed.reclaim(), // releases retired payloads outside the close budget
    Err(refused) => {
        eprintln!("close refused: {}", refused.detail());
        // Fix the cause (for example a live alias), then retry the same custody:
        // refused.try_close(&fresh_request)
    }
}
```

- `close` returns `host::WorthQueryApplicationHomeClosed` (`is_closed()`,
  `outcome()`, `reclaim()`) or `host::WorthQueryApplicationHomeCloseRefused`
  (`detail()`, `query_denial()`, `directory_denial()`, `try_close(..)`,
  `resume()`). A refusal still owns the Home and its lease. `resume()` gives the
  serving Home back only if close had not yet sealed the World.
- Dropping a Home is not a successful close.
- `close` is the Directory lifecycle. On a MemoryOnly Home it refuses, and
  `resume()` returns the Home; a MemoryOnly Home ends when it is dropped.
- `close_with_policy(policy)` (and `try_close_with_policy(policy)` on refusal)
  acquires an owned serial `ExecutionResourceLease` from an
  `ExecutionRequestPolicy`, removing manual lease setup while preserving the
  exact owner on `WorthQueryHomeClosePolicyRefused::Admission`.
- `home.primary_graph()` lends the installed primary graph directly without
  calling `home.runtime().runtime()`.
- The `ExecutionRequest` for direct `close(&request)` comes from a lease built with
  `host::runtime::{ExecutionResourceLease, LeaseRequest, CancellationToken}`; the
  [shared-execution example's `home.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/examples/shared_execution/home.rs)
  shows one.

## If opening is refused

`host::WorthQueryApplicationHomeOpenRefused` is a pointer-sized owning refusal:
it can hold the partly opened Home and its mutation lease (inspectable via
`phase()` / `into_phase()`). Do not treat it as permission to initialize or
delete the directory. Release it with `refused.close(&request)` or
`refused.close_with_policy(policy)`, which returns
`WorthQueryApplicationHomeOpeningClosed` or
`WorthQueryApplicationHomeOpeningCloseRefused` (retry with `try_close` /
`try_close_with_policy`). Its `detail()` (also its `Display` and `Debug`
output) describes the cause.

If the recorded program or schema differs from the request, opening refuses and
`refused.transition_required()` describes both bindings; reopen with
`with_program_transition(..)` or the declarative helper below.

## In-Place Program Upgrades (v2 -> v3) & Zero-Boilerplate Build Identity

Applications evolving an existing Directory Home across schema/program versions
(for example, adding a defaulted, equality-indexed field to an existing entity
while retaining all prior rows) can declare the transition once with
`worth_query_program_transition!` and open/upgrade in a single call through
`host::program_transition::open_directory_with_transition`:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_decl::facade::worth_query_program_transition;
use worth_query_host::facade::program_transition::{
    open_directory_with_transition, worth_query_default_build_id,
    WorthQueryProgramTransitionOpenConfig, WorthQueryTransitionOpenOutcome,
};

worth_query_program_transition! {
    pub struct DaymarkV2ToV3;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_priority_field()
            => default = "Normal".to_owned(), required = false, label = "Priority::Normal",
    ],
    rebuild_indexes = [task_priority_field()],
    description = "Add TaskPriority::Priority defaulting to Priority::Normal",
}

// Honors WORTH_IMPLEMENTATION_BUILD_ID when set; otherwise deterministically
// derives an RFC 4122 §4.3 UUIDv5 (SHA-1) from CARGO_PKG_NAME@CARGO_PKG_VERSION.
let build = worth_query_default_build_id!();
let opened = open_directory_with_transition::<DaymarkV2ToV3>(
    &phase,
    v2_request(path.clone(), build),
    v3_request(path, build),
    WorthQueryProgramTransitionOpenConfig::default_bounded(),
)?;

match opened.outcome() {
    WorthQueryTransitionOpenOutcome::CreatedFresh { program_version } => { /* new v3 Home */ }
    WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version } => { /* already v3 */ }
    WorthQueryTransitionOpenOutcome::UpgradedInPlace {
        from_version,
        to_version,
        backfilled_records,
        indexes_marked_for_rebuild,
    } => { /* atomically upgraded v2 -> v3 in-place */ }
}
```

- **Typed fields only:** `added_defaults` and `rebuild_indexes` accept WORTH's
  typed `RecordSelection` / `FieldSelection` handles, validate/encode default
  values through the field's declared scalar binding at declaration time, refuse
  defaults targeting fields already present in the predecessor schema, and
  verify every `rebuild_indexes` target against the target program's installed
  equality-index catalog before populating `indexes_marked_for_rebuild`.
- **Optional custom step:** use `open_directory_with_transition_and_step` to
  append custom `WorkerIntentBatch` work to the declarative target step.
- **Successor managed indexes:** the target program may declare
  `#[derive(WorthManagedIndex)]` indexes the predecessor lacks. The target step
  prepares their Sources under the successor's catalog, and the upgraded Home's
  first `ensure_index_ready` (or `session.query_indexed`) builds each index from
  the retained records.
- **Fail-closed guarantees:**
  - Opening a `v2` Home under `v3` without a transition refuses with typed
    `refused.transition_required()`.
  - Opening a `v1` Home with a `v2 -> v3` transition refuses with
    `WorthQueryProgramTransitionOpenDenial::PredecessorMismatch`.
  - Declaring duplicate/overlapping records, defaults, index rebuilds, spoofed
    endpoint specs, or a downgrade (`v3 -> v2`) refuses with typed
    `WorthQueryProgramTransitionOpenDenial::Declaration(...)` before touching
    disk (and direct `v2` open on a `v3` Home refuses with `transition_required()`).
  - Exceeding `max_entities_per_kind` refuses before durable intent publication
    with typed `WorthQueryProgramTransitionOpenDenial::EntityScanLimitExceeded`.
  - Budget or cancellation stops during metered backfill planning or pre-intent
    index-budget preflight fail closed with `BudgetOrCancellationStopped` before
    publishing `TransitionPending`, leaving the `v2` Home intact and reopenable.
  - Reopening an upgraded `v3` Home reports `OpenedCurrent` and never re-runs
    the transition or overwrites post-upgrade edits.
- **Formal verification:** specified in
  [`specs/tla/program_transition/ProgramTransition.tla`](../specs/tla/program_transition/ProgramTransition.tla)
  and validated by the 64-seed Directory Home DST in
  `worth-query-court-decl-host-integration` (`host::program_transition::dst`).

## After a crash or unclean close

Reopen the same Directory in a fresh process. Recovery finishes in Store's
fixed owner order before the Home serves anything.

- **Graceful close, or a kill while no operation was running:** reopen normally.
- **A kill after a Native effect became durable but before World recorded its
  occurrence:** reopen succeeds. The Home keeps the original as recovered
  `ProductUnpublished` custody. Send the same request with the same mutation key
  and it continues the original; the handler does not run again.
- **A kill after World durably opened an operation but before any Native
  effect:** reopen durably releases that original before service. The same key
  then executes fresh exactly once, and a later reopen answers it with
  `RecoveredCommitted` and the same receipt.
- **A kill after a Signal effect whose open World recorded:** the effect is
  readmitted on reopen.
- **A kill inside a commit that carries a conditional definition** (a Native
  write and a Signal definition in one original): World durably records the
  planned Signal leg before Native's effect. If Native's effect is durable and
  Signal's is not, reopen keeps the original with Native as its only acting
  owner; continuing performs the recorded Signal leg exactly once, at the
  original's Signal basis, then publishes. Release stays available until that
  leg begins. If the reopened program no longer prepares the recorded leg,
  continuation refuses with `SignalLeg(..)` before any Signal effect; a leg whose
  Signal fate is unknown is never begun twice and waits for the next reopen.
  A kill after the open, before any owner effect, releases the original as
  proven no effect. A Signal leg supplied as a closure cannot be recorded, so a
  Directory Home refuses such a commit with `UnrecordableSignalLeg` before any
  owner effect.
- **A kill during a conditional or temporal operation:** its retained decision
  keeps its original key and intent. A decision whose commit answer was lost
  (`Indeterminate`) is classified before service by World's state for that
  key: published, it finishes and a retry returns the original occurrence; no
  committed open, it is released as no-effect and the key executes fresh once;
  a recovered unpublished original joins it, and the retry continues that
  original without running the operation again. A temporal operation killed
  between its commit and its response re-executes to its original occurrence,
  once.
- **A kill during a workflow** (between steps, or mid-step while a step's
  managed computation runs): the instance, its approvals and its original keys
  are graph records and survive; the running computation is cold-released
  (`Abandoned`) before service and the instance is not cancelled. Continue the
  workflow under its original keys; each step's effect happens exactly once.
- **An unfinished branch creation:** opening the Home decides it before
  service, under its original identity. With no owner fork it is released;
  once any fork is durable it continues. A kill between Native's fork and
  Signal's completes Signal's planned fork once, at the recorded basis,
  before the creation is published.
- **Durable branch retirement and history disposal:** retiring a product branch
  and disposing retired or unprotected historical commits publish through
  World's journals (`Branches` and `Custody`). Retired branches remain retired
  with history readable to the retained horizon across graceful close/reopen and
  quiescent kill/reopen; disposed commits release their roots and envelopes once
  and are never resurrected on reopen.
- **An interrupted program transition:** reopen with the same transition request
  and it resumes. Every step happens exactly once and the result equals an
  uninterrupted run. If the recomputed plan differs from the recorded one,
  opening refuses in phase `host::WorthQueryHomeProgramTransitionOpenPhase::Transition`
  with `TransitionPlanDrift` and extends nothing; a different successor refuses
  with `TransitionPending`.
- **An interrupted branch adoption, stopped branch-set adoption, or support
  retirement:** a kill after support custody is retained and before any Native
  effect discharges the empty adoption on reopen; a kill after Native's effect
  leaves the original in World's recovery catalogue so `readmit_branch_adoption`
  and `recover` finish it once without re-preparing; a kill after World
  publication discharges the adoption custody on reopen. A killed stopped
  branch-set adoption retains its ordered prefix and remainder for
  `readmit_stopped_branch_set_adoptions` and `programs().resume(..)`, refusing
  widened coverage so the completed prefix is never re-prepared. A completed
  `retire_program_support` survives a process kill and keeps refusing
  interpretation and commits under the retired revision.
- **An interrupted external dispatch (attempt durable, observation not yet recorded):**
  opening the Home admits every unobserved attempt from the ended incarnation
  (including concurrent ones with later frames after them) and durably records
  each one's observation as `Unresolved(UnknownProviderOutcome)` before service,
  never re-contacting the external rail during readmission. The recovered custody
  then enters the ordinary recovery handles (`reconcile_recovery_handle`,
  `safe_retry_recovery_handle`, `compensate_recovery_handle`,
  `resolve_recovery_handle`, or authentic inbound completion callback).
  `reconcile_recovery_handle` consumes the recovery claim and returns a
  `WorthQueryRecoveryReconcileAdmission` carrying the durable outbox binding;
  the owner's read-only reconciliation inquiry uses that binding to read the
  rail's own fate (`Completed`, `AcknowledgedPending`, or `UnresolvedNoRecord`)
  without redispatching a completed effect. If the external rail itself restarts
  with lost state, the Home never claims completion: the effect remains
  unresolved and the owner's reconciliation inquiry reports the restarted rail's
  contract (`NoRecord`).
- **A correction killed between authored completion and required derived settlement,
  a divergent-branch correction killed without close, or a live original owner
  held across a kill:** reopen preserves the exact split (authored correction
  complete, required derived settlement pending); replaying the correction key
  returns `RecoveredCommitted` and `recover_required_outputs` settles the
  derived output once matching the uninterrupted reference. A correction on a
  divergent branch keeps its branch-local lineage and refuses cross-branch
  alternatives (`ForeignAlternative`). A live original owner held across a kill
  without `handoff_for_reopen` or close readmits under fresh authenticated
  authority (`WorthQueryOriginalOperation::readmit_native`), survives a
  dependency edit, and advances/closes with continuous aggregate identities,
  while stale handles refuse `StaleIncarnation` and discharged originals refuse
  `OwnershipMaterialUnavailable`.
- **A kill while graceful close was publishing its checkpoint:** reopen recovers
  the state committed before that close.
- **Damage-shaped owner mismatches:** a Signal effect with no recorded open
  (`WorldReadmissionDenial::OrphanSignalAssociation`) and a Native branch fork
  with no creation intent (`WorldReadmissionDenial::MissingCreationIntent`)
  cannot arise from a process kill — only from damaged or forged journal files —
  and refuse before service as typed World readmission denials (the temporary
  D18 `UncleanRecoveryUnsupported` refusal surface has been retired). Close the
  refusal as above; a development Home can be discarded.
- **Contended or discard-pending location:** `refused.physical_refusal()` returns
  `Some(&host::ApplicationHomePhysicalRefusal)`; its `kind()` is
  `host::ApplicationHomePhysicalRefusalKind::MutationOwnerContended` (another
  owner holds the Home) or `DiscardPending` (finish it with `discard_home`).
- **Process-local custody of the ended process:** a managed run, a managed
  cleanup obligation or an application-resource request still outstanding when
  the process died (or the Home was dropped without close) is cold-released
  before service. The row keeps its original, recovery cause and recorded
  posture; `cold_release` on `managed_run_obligations()`,
  `managed_cleanup_obligations()` and `retained_state(..)` reports the terminal
  (`Abandoned` for running work or a request without a result, otherwise
  `ReleaseOutstanding`; never a completion). The released rows no longer refuse
  graceful close; discharge them through `discharge_managed_run`,
  `discharge_managed_cleanup` or `discharge_retained` as usual. No fresh
  provider is contacted, and World still decides each business original.
  Owner-held custody (a mandatory owned-async request) is never cold-released.

Track C1–C3 owner crash-recovery cases and C5 retirement are complete on
Directory Home; final adversarial-oracle and Tier-3 certification runs in Track
C4 / Landing 3 (see [Milestone 3 status](../plans/database-foundation/milestone-3.md#cleanup-track-status)).

## Discard

`host::discard_home(WorthQueryApplicationHomeLocation::Directory(path))` destroys
a Home's data under Store's mutation lease. Close the Home first; a held lease
returns `WorthQueryApplicationHomeDiscardDenial::MutationOwnerContended`. An
interrupted discard leaves a tombstone: opening refuses with `DiscardPending`
until you call `discard_home` again. The next open after a discard initializes a
new Home with a new identity. Discarding `MemoryOnly` does nothing.

## Checkpoints

A Directory Home checkpoints itself. After each advancement it runs checkpoint
maintenance when any Native or World journal reaches the threshold: 4096 entries
or 64 MiB of payload by default. Change the thresholds and the maintenance
allowance with
`request.with_checkpoint_limits(host::WorthQueryApplicationHomeCheckpointLimits::new(entries, payload_bytes))`
and its `with_maintenance_allowance(work, memory_bytes)`. Counted work and
memory bound maintenance and the close checkpoint; no wall-clock deadline
applies, so a slow machine never turns a graceful close into a failed
checkpoint.

`home.checkpoint(&request)` takes one explicitly, `home.inspect_checkpoint(&request)`
reports what is due, and `home.recover_checkpoint(&request)` finishes an accepted
attempt. On MemoryOnly they return
`WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired`.
`home.discard_derived_for_test(&request)` is a sealed D14 certification door
(recorded in `tools/boundary-check/config/control-callers.txt`) that discards
the Home's durable derived checkpoint (`home.checkpoint` and its anchored
members) so reopen reconstructs strictly from authoritative journals and
recomputes derived state only on demand.

Install a caller-pumped clock with `install_desktop_temporal_clock_operation`.
Resolve the installed binding with `home.runtime().conditional::<Binding>()`
and use `host::temporal_contribution::pump_temporal_catch_up_until` to advance
the clock and fire due occurrences on the selected branch under your
execution request and catch-up policy.

The default execution policy's work ceiling (`DEFAULT_HOST_EXECUTION_WORK_CEILING`)
covers reopening a Directory Home: reopen rebuilds its admission image from
retained history, and that reconstructive work grows with the retained commits,
so desktop applications do not need a custom policy to reopen, rebuild indexes
or run actions.

The interim D13 application-checkpoint restore surface has been retired.
Applications reopen their Directory Home directly through
`WorthQueryApplicationHomeRequest`.

## Bound the working set

A Home larger than memory stays usable when you give it a `ResidencyBudget`.
Set it once on the limits you open with; every open and reopen of that Home
enforces it:

```rust
let limits = limits.with_residency_budget(ResidencyBudget {
    head_working_set_bytes: 32 * 1024 * 1024,      // current branch heads
    historical_working_set_bytes: 8 * 1024 * 1024, // history, time travel
    max_in_flight_fill_bytes: 4 * 1024 * 1024,     // concurrent cold loads
    streaming_window_bytes: 256 * 1024,            // reserved for streaming
});
```

Cold regions load on demand and are evicted again under pressure. Head and
history reads have separate budgets, so a long historical scan never starves
current reads. Keeping a snapshot, observation or branch handle open costs only
what a read is borrowing *right now*. Region images are borrowed for one
operation and released when it ends. A scan reads one region at a time, so its
memory stays under the budget however large the Home grows. Without a budget,
residency is unbounded, as before.

Size `head_working_set_bytes` to cover what is borrowed at the same moment:
each concurrent reader's current region, plus the regions open transactions
are editing. `max_in_flight_fill_bytes` caps bytes being loaded from cold
storage at once. `streaming_window_bytes` is reserved for streamed large-value
reads and is not consumed yet.

When borrowed regions fill the whole budget and nothing can be evicted, the
read or write that needs more memory is refused with
`RelationalStorageUnavailableCause::MemoryExhausted`. Nothing has been
published or partly read, and no panic occurs. The refusal is transient.
Release borrows by finishing or dropping open reads, cursors and prepared
transactions, then retry the same request. A server can map it to retry or
HTTP 503. If the refusal keeps happening while the application is otherwise
idle, the budget is too small for the workload.
`RelationalRuntime::residency_pressure()` returns a
`ResidencyPressureSnapshot` for each pool: resident and borrowed bytes, bytes
in flight, completed evictions, and victims skipped because they were busy.

## Shared Scope Access

A single-owner Directory Home can delegate governed read (or read+watch) access
for one application scope to authenticated remote collaborators without ever
opening a second writer on the Directory:

- Declare the stock grant schema once with `worth_query_scope_access!`: typed
  grant records (access is `ScopeAccess`; action, purpose and disclosure are
  closed enums), read/watch capabilities and contracts, the governed
  disclosure (`access::disclosure()` with `access::scope_view()`) and
  grant/revoke actions. `worth_query_shared_access! { program P for Schema {
  access: access, payload, query, row, owner } }` then implements
  `host::shared_access::WorthQuerySharedAccessProgram<Schema>`, and
  `worth_query_remote_codec!(Row { field, .. })` derives each payload and row
  wire codec. The
  [stock scope-access court](../workspaces/worth-query/crates/worth-query-court-decl-host-macros/tests/worth_query_court_decl_host_macros/scope_access_dx/mod.rs)
  is the worked example. A custom grant schema implements the trait or uses
  the macro's explicit `read`/`watch`/`grant`/`revoke` form.
- **Native commit-backed grants and revocations:**
  `WorthQueryApplicationHome::grant_scope_access` and
  `WorthQueryApplicationHome::revoke_scope_access` execute your declared mutation
  intents directly through WORTH's own mutation runtime and refuse unless a
  published commit receipt with non-zero durable record changes is produced.
  Grants and revocations survive graceful close and reopen. On reopen, every
  request (including `WatchPoll` and `WatchClose`) authenticates first — so
  earlier incarnation-pinned tokens refuse with `StaleIncarnation` and unpinned
  tokens refuse with `IncarnationPinRequired` when
  `RemoteAuthConfig::with_require_incarnation_pin(true)` is set; when pinning is
  optional, valid unpinned tokens bound to the same `HomeIdentity` continue to
  authenticate, and once authenticated, prior incarnation-bound `RemoteWatchId`
  handles always fail closed with `WatchNotFound` even after replacement watches
  are opened.
- **Loopback-only TCP trust boundary:** `home.serve_remote(bind_addr, auth_config)?`
  and `WorthQueryRemoteClient::connect(addr, token)?` both fail closed with
  `SharedAccessDenial::NonLoopbackAddressRefused` for any non-loopback address,
  because direct frames carry reusable HMAC-SHA256 bearer tokens and row
  payloads without transport encryption. For cross-host deployments, tunnel
  loopback over an authenticated encrypted transport (TLS/mTLS, Tailscale, or
  SSH) or drive `WorthQueryRemoteServer::handle_frame` over an encrypted
  channel.
- **Bounded, nonblocking hosting and watch reclamation:** `RemoteServerLimits`
  and `RemoteClientLimits` enforce `MIN_MAX_FRAME_BYTES` (64 bytes, strictly
  above the stack-encoded refusal frame), intersect remote row limits with both
  graph cardinality and the installed query binding cap, require bounded
  `RemoteBinaryCodec::encode_remote_bounded` streaming through
  `RemoteByteWriter` before allocation, keep accepted TCP sockets strictly
  nonblocking with per-tick dispatch quotas and absolute per-frame/idle
  deadlines, verify the native admitted watch capability proof's resolved
  resource `EntityId` against the exact bound scope entity at both open and
  every poll, keep `active_watch_count` / `active_watches` strictly read-only,
  roll back failed watch opens before slot insertion, and reclaim watches on
  disconnect or token expiry via owner progress (`serve_pending`, `serve_once`,
  or `reap_expired_watches`).

## Examples

| Example | What it shows |
|---|---|
| [`shared_execution`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/examples/shared_execution.rs) | Two Directory Homes on one execution service; close, reopen, a keyed write, close. Run: `python scripts/application_build.py cargo run --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-topology-entry --example shared_execution` |
| [`h10_graceful_bank_home`](../workspaces/worth-query-bank-world/crates/bank-courtroom/examples/h10_graceful_bank_home/main.rs) | The full Bank journey across two real processes: workflows, dispatch, corrections, close, reopen and continuation. Driven by the [graceful reopen court](../workspaces/worth-query-bank-world/crates/worth-query-bank-court-external-rail-server/tests/worth_query_bank_court_external_rail_server/process/h10_graceful_bank_home.rs). |
| [Bank HTTP server configuration](../workspaces/worth-query-bank-world/crates/bank-http-adapter/src/http/process/configuration.rs) | A production server opening its Directory Home through `bank_server::open_bank_application_home`. |
| [`ordinary_product_workflow`](../workspaces/worth-query/crates/worth-query-certification/examples/ordinary_product_workflow/installation.rs), [`observable_workflow`](../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow/installation.rs), [`authored_workflow`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/installation.rs) | Workflow applications installed in a Directory Home. |

## Where to go deeper

- [Build an Application §4](build-an-application.md#4-install-or-reopen): the full
  installation contract, transitions and Home composition.
- [Query Host README](../workspaces/worth-query/crates/worth-query-host/README.md):
  contention, incarnation, retries, close, discard and workflow custody.
- [Capability Index](capability-index.md#task-index): one row per Home task.
- [Store workspace](../workspaces/worth-store/README.md): how Store keeps the files.
