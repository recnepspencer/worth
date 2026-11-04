# Milestone 10: Mechanically Enforced Runtime Lifecycle Composition

**Status:** design briefs; implementation not started. This is a group of eight
specifications, not a replacement executor or a reopening of accepted 9.x work.
The former Query 10–13 portfolio is removed. Physical execution, persistence,
restart and blob work remain in the [Store roadmap](../worth-store/runtime-integration-roadmap.md).

## Goal and scope

Make the correct integration of a new runtime operation mechanically follow
from its installed contract and the owners it receives. A feature author supplies
domain decisions, effects and supported interruption points. The framework carries
request controls, original identity, partial-effect ownership, delivery and
disposal through legal transitions. An omitted transition must fail compilation,
installation or a real effect-boundary check, rather than depend on an agent
discovering the right private state match.

The same principle applies to the three things every runtime feature currently
threads by hand:
- **Request stops:** carried as one value, never re-declared per owner (10.6).
- **Shared custody:** issued and tracked by its owner, never escaped as a raw `Arc` (10.7).
- **Retained funding:** bound to the original request in the type (10.8).

The measured goal is that adding a feature touches its own owner and the protocol it
joins, not every subsystem it passes through.

Scope includes public Execution, Native/Relational, Signal owner services,
Runtime World, Bridge and Query, plus the Store relational section's runtime-share
issuance boundary (10.7). Query owns their composition, not their truth.
Applications continue using Decl/Host. Public Bank and UI consumers qualify the
contracts where applicable; Proprietary and another machine are outside scope.
This group does not add durability, generic compensation, preemption of arbitrary
blocking code, an ambient executor, or mandatory resource invoices.

These are investigated design briefs: group boundaries, target contracts,
ordered phases and decisive acceptance cases are specified here and in the
children. Private algorithm choices and complete implementation plans are left
to the owning phase. Proposed names are targets, not existing APIs or passed tests.

## What the investigation established

Paths are relative to the repository. Existing production and courts, rather
than old milestone status labels, establish this starting point.

| Existing owner and source | Actual starting guarantee | Required change and owning spec |
|---|---|---|
| `crates/worth-execution/src/authority/{cancellation.rs,lease/limits.rs}` | Observe-only tokens; children retain ancestor cancellation, deadlines and execution constraints. | Reuse unchanged. 10.1 must prevent composition from substituting an unrelated request. |
| `crates/worth-execution/src/authority/request.rs` | `run_native_with_checkpoint` checks again after the callback; failed final polling can discard its returned value. | Keep this behavior for discardable computations; add an owner-preserving effectful handoff in 10.1. |
| `crates/worth-execution/src/backend/meter/checkpoint.rs` and `src/tests/nested_request_stops.rs` | A swallowed request stop remains latched, including nested execution. | Preserve this foundation; do not build a second interruption latch. |
| `crates/worth-relational/src/execution/{request_custody.rs,request_stage.rs}` | Native stages consult the original request and distinguish domain refusal from execution stop. | Carry original Native outcomes through 10.1, without adding post-effect admission or erasing an inner cause. |
| `crates/worth-signal/src/branch/owner_services/cancellation.rs` | `SignalOwnerMovementPermit` marks the synchronous cancellation cutoff. | Preserve its meaning; request stop cannot relabel an already performed movement. |
| `crates/worth-runtime-world/src/publication/{cancellation.rs,custody.rs}` | World cancellation reaches its own and Signal controls; active attempt custody is registered before effects and survives abandonment. | Reuse these catalogs and owners in 10.1/10.3. Another global recovery ledger is forbidden. |
| `crates/worth-runtime-world/src/lifecycle/owner/{execution_service/terminal.rs,current_read_publication.rs}` | No-effect and unpublished effects remain distinct; actual Native publication wins over a callback result; unwind retains native evidence. | Generalize ownership handoff without replacing effect cutoffs or guarded publication. |
| Query Execution `primary_graph/application_contribution/request_execution/advancement{.rs,/phase_custody.rs}` | Runtime-affine, nonescaping phases; final request checks can refuse after the body ran. | Add an owning counterpart in 10.1; retain ordinary read-only/discardable advancement. |
| Query Publication `application_entry/operation/{fixed_advance.rs,discovered/advance.rs,workflow/advance.rs}` | Owning public outcomes exist. Fixed/discovered move the owner out of an outer `Option`; workflow mutates a retained owner. | Standardize retention through final refusal in 10.1/10.2. The former pattern is an investigation target, not a claim of a reproduced defect. |
| Query Publication `application_entry/operation/{native_publication.rs,cleanup.rs,history_mode.rs}` | Shared Native state and cleanup already exist; composition still requires family-specific extraction, posture checks and disposal wiring. | Make the shared operation protocol total in 10.2–10.4; preserve native distinctions. |
| Query Execution `primary_graph/application_query/live/lease/lifecycle.rs` | Live termination releases queue/Bridge/query-read custody; `Drop` attempts abandonment termination. | 10.5 makes refused termination remain owned and integrates session close; it does not replace the live engine. |
| `crates/worth-runtime-world/src/lifecycle/close/{drain.rs,report.rs}` | Close excludes new admission and reports retained records/retirement work. Enumeration is not settlement. | 10.3/10.5 compose existing residual owners without claiming universal cleanup or crash survival. |
| `crates/worth-execution/src/backend/{meter.rs,meter/nested_stop.rs}` and owner refusal enums | `KernelStop`, `KernelFailure` and `NestedComputationStop` already own stop meaning. Yet 74 of 186 `worth-relational` refusal enums re-declare stops, there are 2,664 forwarding `map_err` calls, and refusal lines are 7.3% of changed lines (30 days to 2026-11-03). | 10.6 carries one stop value through every owner. |
| `crates/worth-relational/src/change_source/runtime_handle.rs` and the Milestone 3 Home close owners | Native runtime and medium shares escape as raw `Arc` clones; close refusals (`SharedRuntime`, `SharedBinding`, `Busy`) cannot name holders. The M3 integration found holders one rebuild at a time. | 10.7 owner-issued, role-tagged shares with a typed census. |
| Retained-owner charge/reserve sites (about 1,500 in about 930 files) | Payer choice and refunds are per-site conventions. 16 of 74 M3 integration commits corrected funding, and fixtures pin observed byte peaks. | 10.8 funding lease and declared footprints for retained owners only. |

`9.17.6.4` already specifies ordinary owning continuations, and current code
implements substantial parts of that contract. `9.17.6.6` carries runtime/action
affinity. Milestone 10 strengthens reusable framework construction underneath
and across those surfaces. It must not add a parallel ordinary operation API.
Current 9.18.2 implementation may finish independently; implementation planning
for this group starts from its then-current accepted source.

The current full advancement phase also reaches effects. A comment identifying
a callback as read-only is not mechanical enforcement. 10.1 must introduce an
observation-only view and restrict raw effect-phase issuance to the owning
runner; merely adding a convenient wrapper would leave the bypass open.

## Specification groups and dependencies

| Spec | Usable result | Requires | Establishes for successors | Parallel constraints |
|---|---|---|---|---|
| [10.1 Original-request and ownership handoffs](milestone-10.1.md) | Effects and original owners survive a stopped or unwinding handoff. | Existing Execution/Native/Signal/World contracts. | One reliable stage boundary for all later composition. | Freeze its handoff contract before changing operation drivers. |
| [10.2 Typed operation progression and delivery](milestone-10.2.md) | Source, fixed, discovered and workflow lanes share enforced ownership transitions. | 10.1. | Completed, terminal and delivery owners; descriptor-bound fresh admission. | Family adapters can proceed independently after the shared protocol freezes. |
| [10.3 Disposal, abandonment and shutdown](milestone-10.3.md) | Cleanup cannot resume business work; unresolved obligations survive abandonment/close. | 10.1 and 10.2 owner interfaces. | Disposal-only owners and typed residual close work. | Native adapter investigation can overlap 10.2; integrated cutover cannot. |
| [10.4 Declared composite operations](milestone-10.4.md) | Installation derives complete lifecycle wiring for groups/navigation/workflows. | 10.2 and 10.3. | New composites reuse the protocol without private native-state surgery. | Can run beside 10.5 after common interfaces freeze. |
| [10.5 Managed computations, outputs and live sessions](milestone-10.5.md) | Yielded work, output interests and live leases use enforced registration and close paths. | 10.1 and 10.3; 10.2 for operation/output adapters. | Full runtime lifecycle composition across one-shot and long-lived work. | Separate live/demand adapters may proceed beside 10.4, not edit its owners. |
| [10.6 Shared request stops in owner refusals](milestone-10.6.md) | One stop value crosses every owner; owner refusals stay owner-specific. | Existing `KernelStop` and nested-stop latch. | The stopped arm of 10.1's owning outcome; stop carriage for 10.7/10.8 refusals. | Freeze carriage first; owners migrate one per slice. |
| [10.7 Owner-issued shares and close custody](milestone-10.7.md) | Runtime/medium shares are issued, role-tagged and censused; one retirement contract per role. | Landed database-foundation Milestone 3 Home close. | The census in 10.3's residual close report; share issuance funded by 10.8. | Beside 10.1/10.2; Native first, then Bridge, Query and Store section. |
| [10.8 Original-request funding of retained owners](milestone-10.8.md) | Retained owners are funded through a lease from the original request, with declared footprints and phase storage. | 10.1; 10.7 for share issuance. | Footprint-derived fixture grants; structurally separated phase frames. | Owner by owner beside 10.4/10.5. |

### Order of the arc

1. **10.6** first. It is mostly mechanical, needs nothing, and every later spec returns its stop.
2. **10.1** next, with its stopped arm carrying 10.6's stop.
3. **10.7** beside 10.1 and 10.2, as soon as Milestone 3's Home close has landed.
4. **10.2**, then **10.3**. 10.3 Phase 3's close report carries 10.7's census, and its owner lifecycle states become types.
5. **10.8** owner by owner, beside **10.4** and **10.5**, which then consume funded, issued and stop-carrying owners rather than threading them.

This is not a requirement to finish 9.19–9.22 before work can begin. Dependencies
are the actual interfaces above. Their provider/access/reuse integrations consume
the same protocol when adopted; they do not require a second lifecycle design.

## Shared design decisions

- Keep request interruption, synchronous owner cancellation, workflow-instance
  cancellation and external resource cancellation distinct. Share transport and
  ownership mechanisms only where authority and failure meaning match.
- Reuse owner-issued proofs. A new adapter must not reconstruct permission,
  currentness, output completeness or recovery from a receipt, identifier,
  descriptive posture, boolean or matching label.
- Typestate protects stable legal transitions. Permissions, current heads,
  installed meaning and live resource availability still need owner-controlled
  runtime checks at their actual use boundary.
- An ordinary driver may keep a private exhaustive sum of states. Do not expose
  every internal pump state as another generic public type. New state/adapter
  membership must make incomplete framework construction fail compilation.
- Rust ownership is affine, not a guarantee that cleanup runs. `must_use` is
  supplementary. Preinstalled native custody and explicit session close protect
  real obligations. `mem::forget`, process death and losing the entire runtime
  cannot be described as successful cleanup or durable recovery.
- Share concrete lifecycle mechanics behind thin typed shells. No mandatory new
  work/memory quotes, per-feature pools, recurring global scans, or unrestricted
  callback/`Any` bag may become a new prerequisite for ordinary domain work.
  10.8 does not breach this: it types the funding that retained owners already
  carry, and ordinary domain work gains no quote.
- Bind at issuance, pass at use. Anything an owner hands out and later retires
  (shares, funding, disposal owners) is issued by that owner, typed, and bound to
  the original request or Home at issuance. The live request is still passed at
  each call for that call's cancellation and work budget, and never captured.
- Affinity is an owner-issued typed identity checked once at issuance, reusing
  9.17.6.6's runtime/action affinity. Do not spread lifetime brands through
  public signatures; a foreign share, lease or owner is refused before effects.
- Stops are carried, never re-declared (10.6). Owner refusals keep owner meaning;
  a request stop is never relabelled as a domain refusal or the reverse.
- Placement follows the repository rule: role and meaning shared across runtimes
  go in `worth-foundational`; registries, counters and anything with `Drop` go in
  the owning runtime (Execution for funding and stops, Native/Store for shares).

Architecture [9, 13, 16 and 19](../../docs/coding-guidelines/arch_laws.md) are
implemented by exhaustive adapter construction, derived installed wiring,
consuming transitions and existing resource owners respectively. Architecture
12 prevents false unification; 14/22 require honest partial-effect posture;
20 keeps inspection separate from authority. [DX 8, 17 and 19](../../docs/coding-guidelines/dx_laws.md)
require discoverable legal actions, actionable typed refusals and preserved
composition guarantees.

## Group closure and QA considerations

Each child closes its own phase matrices, real caller cutover and deletion of
the displaced path. Closed historical phases are not reopened. Cross-group
closure demonstrates an ordinary operation, a partially published composite,
a yielded computation and a cancelled live session using their real owners.
Each must retain exact effects and legal recovery after a stopped handoff.
New extensions cannot bypass the protocol through ordinary or advanced exports.

The binding specs (10.6–10.8) close on a demonstration, with source evidence, that adding a feature no longer edits the owners it passes through:
- a new `KernelStop` cause reaches all callers with no owner enum edited;
- a new holder of a covered subject adds one role and one retirement implementation, and a forgotten one appears in the close census;
- a new retained type declares its footprint and is constructible only through the original request's lease.

Record the touched-file count per feature, peak retained memory and frame sizes against the Milestone 3 baselines.

Reuse existing Execution tests, Native and World court targets, Query Decl/Host
court and compiled consumer fixture. Compile-fail cases protect actual public
boundaries with lawful counterparts; runtime tests use genuine issuance and
fault boundaries. No test inventory, proof ledger or tests-for-tests framework
is required. Independent numerical/tree oracles are unnecessary for these
ownership claims; native readback, real cleanup and handler invocation evidence
are decisive. Performance evidence separates touched owners from unrelated
retained population and follows the existing iteration/build configuration.

Documentation shipped with implementation teaches current calls and failure
handling in the existing contributor/application guides. These planning files
carry the schedule; public documentation does not carry milestone history.

## Adjacent work outside this group

- **Checkpoint basis composition.** `CapturedCheckpointBasis` (`crates/worth-relational/src/durability/authority/checkpoint_image.rs`) aggregates about 35 fields from every subsystem. Having each subsystem register its own section would stop new subsystems from editing that hub. That is durability, so it belongs in the database-foundation roadmap, and only if a measurement shows the hub actually drives cross-subsystem churn. It is not a Milestone 10 deliverable.
- **Store physical Serving census.** The Store roadmap adopts 10.7's share primitive as a committed successor handoff.
