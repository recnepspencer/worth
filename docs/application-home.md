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

**Crash recovery is not yet certified.** A Directory Home survives graceful
close, and a process kill while no operation is executing. After a kill during
an operation, the named cases reopen and continue or release the interrupted
work, and a few damage-shaped cases still refuse to open; see
[After a crash](#after-a-crash-or-unclean-close).

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
workflows, consume the Home with
`home.retain_workflow_spec(installed_spec, signing_owner)`. It returns a
`host::WorthQueryWorkflowApplicationHome` with the same lifecycle, or a
`host::WorthQueryWorkflowHomeBindingRefused` that gives back the Home, the
installed spec and the signing owner.

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
`WorthQueryApplicationHomeOpenRefused::Physical`; its `detail()` names
`MutationOwnerContended`. If the owner process dies, the OS releases the lease and
another process can open the Home.

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
- The `ExecutionRequest` comes from a lease built with
  `host::runtime::{ExecutionResourceLease, LeaseRequest, CancellationToken}`; the
  [shared-execution example's `home.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/examples/shared_execution/home.rs)
  shows one.

## If opening is refused

`host::WorthQueryApplicationHomeOpenRefused` is an owning refusal: it can hold
the partly opened Home and its mutation lease. Do not treat it as permission to
initialize or delete the directory. Release it with `refused.close(&request)`,
which returns `WorthQueryApplicationHomeOpeningClosed` or
`WorthQueryApplicationHomeOpeningCloseRefused` (retry with `try_close`). Its
`detail()` (also its `Debug` output) describes the cause.

If the recorded program or schema differs from the request, opening refuses and
`refused.transition_required()` describes both bindings; reopen with
`with_program_transition(..)`.

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
- **An unfinished branch creation:** it reopens as recovered custody under its
  original identity, and the target branch name stays fenced. With no owner
  fork it can only be released; with every required fork durable it can only
  continue to exactly those forks. A partly performed creation refuses before
  service with `WorldReadmissionDenial::PartlyPerformedCreation`.
- **An interrupted program transition:** reopen with the same transition request
  and it resumes. Every step happens exactly once and the result equals an
  uninterrupted run. If the recomputed plan differs from the recorded one,
  opening refuses in phase `host::WorthQueryHomeProgramTransitionOpenPhase::Transition`
  with `TransitionPlanDrift` and extends nothing; a different successor refuses
  with `TransitionPending`.
- **A kill while graceful close was publishing its checkpoint:** reopen recovers
  the state committed before that close.
- **Still refused (D18):** an external dispatch attempt with no recorded
  observation (owner `Dispatch`), a Signal effect with no recorded open, and a
  Native branch fork with no creation intent. The last two arise only from
  damaged files, not from a kill. Opening refuses before service with a typed
  `UncleanRecoveryUnsupported { owner, effect }` (the `DurableOwnerFamily` and
  the effect digest); `refused.unclean_recovery_unsupported()` returns it in
  every case. Close the refusal as above; a development Home can be discarded.
- **Contended or discard-pending location:** `refused.physical_refusal()` returns
  `Some(&host::ApplicationHomePhysicalRefusal)`; its `kind()` is
  `host::ApplicationHomePhysicalRefusalKind::MutationOwnerContended` (another
  owner holds the Home) or `DiscardPending` (finish it with `discard_home`).
- **Known hold:** if the process died while an operation held a managed run,
  the reopened Home's graceful close refuses with
  `ManagedCleanup(ManagedRunPending { .. })` until the cold release contract lands
  (Milestone 3 cleanup C3).

Certification of crash recovery comes later in Milestone 3's cleanup track
(C4–C5; see [Milestone 3 status](../plans/database-foundation/milestone-3.md#cleanup-track-status)).

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
and its `with_maintenance_allowance(work, memory_bytes, timeout)`.

`home.checkpoint(&request)` takes one explicitly, `home.inspect_checkpoint(&request)`
reports what is due, and `home.recover_checkpoint(&request)` finishes an accepted
attempt. On MemoryOnly they return
`WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired`.

`host::application_installation::WorthQueryInterimCheckpointRestore` is an
internal certification door scheduled for deletion. Applications reopen their
Directory instead.

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
