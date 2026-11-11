# Shared Home maintenance owner correction: independent design review

Decision: **APPROVE** for implementation of the proposed worker-owner correction.
No blocking design defect remains in the proposal as written. This is a design
decision, not implementation clearance or a `FUNDING LANDED` claim.

Reading boundary: repository AGENTS.md, every coding-guidelines document, M3
D11/D12 and the relevant coupled-checkpoint/finite-cadence owner contracts, the
exact orchestration funding grants, the proposed
`funding-close-owner-correction-design.md`, Store checkpoint maintenance,
operation, owner, limits, journey and close settlement, the exact coupled World
close capture call, Query Host maintenance resources/panic containment and
retirement adapter, the closed construction-factory rule/caller entry, the
failed Store court and its coupled Home fixture, and the full
`funding-finish-store-refusal-and-ack.log`. No source edits, builds or tests were
performed for this review.

1. **The stated composition defect is real, and occurs before cold reopen.**
   The failed court calls `home.checkpoint(&execution).unwrap()` immediately
   before the first close assertion at Store court
   `process/application/native_components/checkpoint.rs:180`. The log records
   three passes and that one failed assertion; it does not print the checkpoint
   denial itself. The denial is established by source tracing: the coupled Home
   fixture never installs Query Host maintenance, World close calls
   `checkpoint_for_close`, and `CloseCheckpointMaintenance::capture` returns
   `Maintenance::Unavailable` without its optional registered supplier. The
   factory inventory and exact caller list identify Query Host
   `application_home/maintenance.rs` as the sole external installation caller.
   This supports correcting the owner rather than changing the assertion.
   Governing requirements: D12's close trigger; Architecture 1 and 9.

2. **Store's same Home checkpoint owner is the correct placement.**
   `HomeCheckpointOwner` already owns the actual attempt, acknowledged coverage
   and close supplier. Replacing that supplier slot with a private worker slot
   under that owner supplies the missing capability to both compositions. The
   proposed worker retains only its ledger, cancellation, timeout and active
   state, with no Home, Serving, Native, World, operation or callback retained
   inside it. Borrowing the actual operation for one joined execution creates
   no strong cycle or second checkpoint authority. Runtime clocks, atomics and
   Drop belong here; no new legality witness belongs in a substrate crate.
   Existing Store manifest dependencies already include Execution, Foundational
   and Proof. Governing laws: Architecture 1, 10, 12 and 19; AGENTS type-placement
   invariant; Domain Structure 6, 9 and 10.

3. **Funding separation and the default path are sound as specified.**
   Funding grant 4 explicitly overrides the older same-caller maintenance
   wording: automatic close capture draws on the same Home-limits maintenance
   allowance through the same scoped worker. Explicit checkpoint/recovery
   remains caller-funded. The current exact close call constructs an operation
   with default limits, so lookup must prefer the already installed owner
   worker and preserve its actual custom Host allowance. Only an absent worker
   on direct Store close admits the finite default allowance. Admission failure
   is an ordinary typed checkpoint failure, not a fallback to caller funding.
   The proposal states all of these distinctions. Governing requirements:
   exact grant 4, D11 and D12; Architecture 10 and 22; Performance lane honesty.

4. **One worker must remain one actual shared worker across concurrency.**
   The proposed mutex-protected owner slot must serialize inspect/admit/install
   so racing lazy calls cannot create independent roots; duplicate explicit
   Host installation keeps typed Busy. Release the slot lock before executing
   the operation. Preserve the current worker compare-exchange and RAII active
   guard across setup, run, join and panic unwinding. Host retirement must read
   that same owner's active state through its original operation. The current
   Query retirement also requires unique maintenance-Arc access, while Store
   operation retirement discharges its Serving alias and retains the original
   owner. Moving the resources into that retained owner preserves this model.
   A boolean observation grants no authority. These are implementation details
   already required by the proposal, not requests for a new lifecycle design.
   Governing laws: Architecture 9, 19 and 22; Performance explicit coordination.

5. **The memory and cleanup contract survives the move.**
   Retain the existing finite root admission, 2 MiB joined stack, supported
   64 KiB rounding and checked quote for the actual task, join handle and worker
   Arc/control layout. Reserve before Arc/worker creation. The slot is part of
   the existing Home owner allocation; recompute its actual owner layout rather
   than introducing a second allocation lane. Keep the worker backing last;
   cancellation on final worker Drop and active-guard release remain runtime
   responsibilities. A joined worker cannot escape the borrowed operation.
   Query retains its separately funded actual adapter allocation, and retired
   operation metadata may lawfully retain the owner and its paid resources.
   Governing laws: Architecture 19 and 22; Performance allocation lifecycle and
   memory contracts. This preserves the existing funded mechanism; it does not
   authorize a new accounting framework.

6. **The proposed facade/cutover is appropriately narrow.**
   Keep the exact `install_home_checkpoint_maintenance` factory name and its
   existing closed caller entry; remove its supplier argument and the public
   supplier trait/export. Worker and registry types remain private. The new
   automatic method acts only on an actual Home-issued operation and grants no
   root, basis, selection, source or lifecycle authority. Store must contain
   setup/join/payload panics before Query deletes its sole containment module.
   No open request, branch service, lifecycle propagation or close progression
   change is required. D11's settled-failure reporting and retained-tail behavior
   remain in the unchanged close orchestrator; pending recovery still uses its
   existing custody path. Governing laws: Architecture 1, 5 and 15; MENTALITY 11;
   Composition 6 and 13; exact grants and current ownership boundary.

7. **Proof scope is proportionate, but remains outstanding.**
   After the coherent Store/Query cutover, rerun the failed Store court unchanged
   and the named affected Host funding/close controls, including custom tiny
   maintenance allowance, explicit tiny-caller refusal, and Host-funded close.
   Existing unaffected accepted evidence need not be rebuilt merely for this
   review. The three named general reopen controls whose worker reservations
   changed remain the proposal's required regression evidence. Inspect the
   implementation for single-slot admission, shared active state, exact layout
   quotes and panic containment, then run the required scoped constitutional,
   formatting and line-cap gates. Observe the one-build and 25 GB floor rules;
   inability to run a required proof is a reported blocker, not clearance.
   Governing laws: Testing 2, 3, 5, 17 and 28; AGENTS verification requirements.

The separate cold changed-region P1 storage-owner handoff is outside this
correction's causal claim and remains blocked. Neither this approval nor the
failed live-close court demonstrates its repair. No funding completion claim
is authorized by this design review alone.
