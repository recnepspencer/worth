# Diagnostic stack peak boundary and plan

Scope: actual high-water bytes of unchanged entry bodies, not static frame sums
or a repair. No product/court/profile/manifest changes in measurement. The
existing dev artifacts are reused: accepted final census verifies their hashes;
post-build source changes are scalar cfg(test) observations and fixtures only.
Release requires consumer and Bank workspace builds, serialized as one release
batch (two native Cargo invocations because these are separate workspace roots).
No extra target directory or artifact-changing flags. Save transient entry COFF
objects read-only during linking for any otherwise anonymous peak frame.

One Python harness under target/lane-evidence/stack_peak uses Windows Debug API.
At the genuine body entry, leave its caller suspended, spawn an OS thread with
64MiB reserve and call exactly the same compiled function. Preserve argument and
return ABI registers. Call GetCurrentThreadStackLimits on that worker, commit
and paint only unused stack below its entry RSP, leave the bottom guard page,
then execute its unmodified body. Manual commitment also updates the worker's
NT_TIB.StackLimit to agree with that commitment. At its native return breakpoint, scan before
stack destruction. Report whole touched 8-byte slots, including the worker
startup baseline. The first pass locates the low-water slot; a second hardware
write watchpoint relative to body-entry RSP captures the actual peak call path.
A stack-free native minimum recorder around __chkstk avoids millions of debugger
events; a post-allocation trap records the actual reserved RSP. Windows startup
padding is retained in absolute samples and normalized only for path matching.
Report max across both passes; per-entry metadata peak is the path-pass sample.
Painting is touched high-water at8-byte resolution, not an instruction-by-
instruction bound for untouched subpage reservations.
No observer function allocates slots or calls on the measured stack during the body. Restore
the original caller with the result once; no duplicate body execution in a run.
Debugger code breakpoints are restored before the body, and disk artifacts stay
unchanged. A second run is a distinct peak-path diagnostic, not certification.

For Bank, debug child processes too and relocate the selected parent function
in each child. Report the maximum across the parent and every width-child entry
thread, with individual process results. Physical execution worker stacks are
outside this named entry-thread measurement. Execute no other test/ignored,
scale, exhaustive oracle or scenario. All test assertions remain in original
compiled code. A successful diagnostic never certifies the 1MiB genuine main.

Frame sizes come from actual unwind SP deltas at the watched write. Exact PE
function ranges validate PDB symbol addresses; unresolved names stay explicit,
never nearest-symbol guesses. Preserve raw event/call-path and artifact data.
Report precision and any instability across the two painting passes.

Then trace all10Store warnings as one family. Recovery-only constructors are
live with recovery-runtime-owner; deleting them globally would break C8. Bare
Store has no mint for recovered custody. Remove the impossible default recovery
payload graph structurally, keep shared opaque carrier/read paths honest, no
allow/expect, fake use, widened visibility or new feature. Review complete
consumer and field/census cutover before one bare Store check. No stack repairs.

Final boundary: consumer-root measures its genuine run body, preserving its
existing main/thread wrapper. _NO_DEBUG_HEAP=1 removes debugger-only heap
checking; no compiler/profile override. All12 entries completed. Read
stack-peak.md for measured values, failures, exact lane logs and scope limits.
