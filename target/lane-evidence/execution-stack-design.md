# Entry-point execution stack design (not implemented)

This is the separate batch after the P-C integration head is committed and
reported for H5. No frame or type shrinking is included.

## Boundary and owner

One implementation: `worth-execution/src/execution_stack.rs`, exported
through its existing lib facade as `run_with_execution_stack<R>(body: impl
FnOnce() -> R) -> R`. It calls `stacker::maybe_grow`, with stacker declared
at the same existing repository version in Execution's own manifest. Physical
stack allocation/switching is execution infrastructure. Foundation's existing
stacker calls serve its own value-shape operations and do not make Foundation
the owner of other runtimes' execution. No new live table, counter, platform
Drop type, proof or authority is introduced. All participating owners already
depend on Execution. Cargo updates required lock edges; no feature is added.
Foundation's existing value-shape implementation is unchanged.

The helper accepts borrowed, non-Send callbacks and returns their exact R;
no thread, Send/'static bound, error translation, catch or new API requirement.
For wasm32 it directly invokes the callback. No Cargo feature is added.

Named native constants: `EXECUTION_STACK_RED_ZONE_BYTES = 4 * 1024 * 1024`
and `EXECUTION_STACK_SEGMENT_BYTES = 8 * 1024 * 1024`.
The requested256KiB default would not by itself protect an opaque public call
whose internal uninterrupted peak exceeds that amount. The measured complete
consumer peak is3,075,736bytes and example peaks are about2MB;4MiB provides
over1MiB headroom and ensures a genuine1MiB main switches before deep work.
An8MiB segment allows nested ordinary entry calls to reuse its available
stack. This uses the actual measured requirement rather than assuming another
public barrier will appear within256KiB. Smaller/released frames are C2.

## Exact execution boundaries

Wrap actual execution, not builders or creation of unpolled futures:

- Query public bootstrap functions `with_bootstrap_advancement` and
  `with_bootstrap_advancement_from_request` in request_execution/advancement.rs
  and advancement/bootstrap_opening.rs. Original HRTB phases, request meter,
  OpeningCustody, panic/result precedence and callback completion remain inside
  the original operation.
- Query installed public entry methods in advancement/installed_opening.rs:
  `with_host_advancement`, `with_funded_host_advancement`,
  `with_application_advancement`, `with_funded_application_advancement`, and
  the product runtime's `with_advancement`/`with_request_advancement`.
  These are the shared public operation boundaries for mutations, query
  execution, retained/discovered continuation and publication; valid branded
  phases cannot escape the callback. No private advancement runner or kernel
  loop is wrapped.
- Host ordinary and transition `open_in_advancement` in opening.rs and
  transition/opening.rs. Both actual recipes, including shared successor
  compilation, run under the barrier. Host `close` in close.rs covers its real
  retirement operation, without changing refusal custody. Include public
  `WorthQueryApplicationHomeCloseRefused::try_close` in close/retry.rs and
  opening-refusal `close`/`try_close` in failure_close.rs. These execute retained
  retirement directly. Their complete Host barriers enclose Query's closed
  home_retirement preparation/advancement handoff; installed advancement is
  not claimed to protect retirement.
- Native public `commit_branch_transaction`/`prepare_branch_transaction` in
  mvcc/transaction/commit.rs, preparation port's `prepare_branch_transaction`,
  and publication port's `compare_and_publish`. Original candidate/body,
  validation, binding/epoch checks and durable/live ordering are unchanged.
- Native direct public routes also need barriers: publication port's
  `with_prepared_memory_cutover` in mvcc/publication/port_handoff.rs;
  runtime `validate_branch_transaction` and
  `validate_branch_transaction_with_source_context` in
  mvcc/validation/proposal_validation.rs; runtime `prepare_validated_proposal`
  in mvcc/publication/candidate_preparation.rs; preparation port's
  `prepare_retained_entity_selection` in transaction/preparation_port/selection_preparation.rs;
  transaction `prepare_merged_plan` in transaction/plan_preparation.rs. These
  perform actual current preparation/cutover rather than delegation into the
  other proposed entries. The returned merged-plan borrow keeps its exact
  original lifetime. Preserve the full memory-cutover callback and cleanup.
- Signal public `advance_signal_branch` in
  logic/transaction/runtime/state/branching/advancement.rs. Its operation,
  cancellation, owner-port forwarding and original transaction callback stay
  together. Include direct public mutation port `advance_exact` and
  `advance_exact_with_completion` in branch/owner_services/mutation_port.rs,
  plus conditional-definition publication port `advance_exact_with_completion`
  in branch/owner_services/definition_publication_port.rs. They bypass the
  runtime method and have distinct real advancement bodies. Keep complete
  catch/unwind, completion custody and returned unconsumed permit inside the
  barrier. No Signal internal propagation/planner function is wrapped.

The observed example/consumer peaks pass through Query's public installed
advancement entries. The bootstrap and Host barriers cover their earlier
installation/open/transition phases; Native/Signal public barriers protect
direct calls made from any existing caller thread. Already-running valid
advancement callbacks retain their original semantics and authority.

Owned-self methods must capture the complete owner, not independently capture
its fields and reorder retirement. Prefer wrapping their existing whole-owner
delegation. Where a whole body consumes fields, retain that body in a private
owner method and invoke it from the public stack barrier, preserving argument,
local, funding and Drop order. This is an entry adapter, not another phase
redesign. Borrowed methods keep their original body/result inside FnOnce.
Do not wrap asynchronous future construction as a substitute for execution;
if a measured path lacks an actual synchronous public entry, report that exact
boundary rather than claiming a construction-only wrapper protects polling.

## Consumer and spec cutover

Remove consumer_root main's8MiB wrapper thread and its matching lint expectation;
the genuine main calls the same run body directly. No other example meaning
changes. Update H10 with one sentence that entry-point stack growth satisfies
the genuine-main-thread requirement. Keep C2 and every type-size ceiling.

## One verification round

After review and the complete batch, one check per reached workspace, serial,
with the25GiB floor. One owned build per required workspace/selection, then:
the three unchanged dev example mains (invariant_reads, required_feature,
shared_execution), consumer-root's genuine main, both exact Bank parents at
default stack, helper borrowed/non-Send/result/unwind observations, and one
worth-signal-wasm wasm32 check. Use real production bodies, existing build
identity issuance and owned selections; no diagnostic64MiB harness, stack
environment override, new thread, ignored body, scale or exhaustive campaign.
Record native PE main-stack reservation and exact commands/results. Final
source review reads the diff and saved evidence without rebuilding.

## Design review correction

First design verdict REJECT: wrong helper owner and incomplete public boundary
inventory. These were design flaws, not test-fixture issues. All findings are
accepted and corrected above in one revision. The4MiB/8MiB mechanism and the
Query execution inventory were approved. No stack implementation or compiler
run has occurred for this design.
