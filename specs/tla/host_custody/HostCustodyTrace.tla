------------------------- MODULE HostCustodyTrace -------------------------
EXTENDS HostCustody, TraceData

(*
  Trace-validation refinement wrapper for HostCustody.tla.

  The seeded Rust DST in `worth-query-court-decl-host-integration`
  (`host::source_custody::custody_dst`) emits one NDJSON record per custody
  transition when `WORTH_DST_TRACE_DIR` is set; `tools/tla/run_tlc.py trace`
  compiles the records into `TraceData.tla` (`Trace == << ... >>`) and checks
  that every observed Rust step refines `HostCustody.tla`, satisfies every
  safety invariant, and matches the observed post-state.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

\* Start the next independent command in the same trace once the prior command settled.
ResetForNextCommand ==
    /\ phase \in TerminalPhases
    /\ ~durableOriginal
    /\ phase' = "Empty"
    /\ erased' = FALSE
    /\ durableOriginal' = FALSE
    /\ execCount' = 0
    /\ correctionMode' = FALSE
    /\ committedValue' = FALSE

StepAction(step) ==
    CASE step.action = "AcceptOrdinary" ->
            AcceptOrdinary(step.erased)
      [] step.action = "AcceptCorrection" ->
            AcceptCorrection(step.erased, step.outcome)
      [] step.action = "Erase" ->
            Erase
      [] step.action = "Downcast" ->
            Downcast(step.matched)
      [] step.action = "ResumeRetentionRefused" ->
            ResumeRetentionRefused(step.retainedOriginal)
      [] step.action = "ResumeRetainAndRefuse" ->
            ResumeRetainAndRefuse
      [] step.action = "ResumeCommitClosed" ->
            ResumeCommitClosed
      [] step.action = "ResumeCommitPendingClose" ->
            ResumeCommitPendingClose
      [] step.action = "ResumeAdvanceRefused" ->
            ResumeAdvanceRefused
      [] step.action = "ResumeDeclineClosed" ->
            ResumeDeclineClosed
      [] step.action = "ResumeDeclinePendingClose" ->
            ResumeDeclinePendingClose
      [] step.action = "ResumeEnterNativeObligation" ->
            ResumeEnterNativeObligation
      [] step.action = "WithdrawUnexecuted" ->
            WithdrawUnexecuted(step.succeeds)
      [] step.action = "ClosePending" ->
            ClosePending(step.succeeds)
      [] step.action = "Reset" ->
            ResetForNextCommand

MatchesPost(step) ==
    /\ phase' = step.postPhase
    /\ erased' = step.postErased
    /\ durableOriginal' = step.postDurableOriginal
    /\ execCount' = step.postExecCount
    /\ committedValue' = step.postCommittedValue

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ MatchesPost(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
