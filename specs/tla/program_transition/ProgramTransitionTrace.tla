------------------------ MODULE ProgramTransitionTrace -------------------------
EXTENDS ProgramTransition, TraceData

(*
  Trace-validation refinement wrapper for ProgramTransition.tla.

  The 64-seed Rust Differential DST in `worth-query-court-decl-host-integration`
  (`host::program_transition::dst`) drives real WORTH Directory Homes across
  v2 -> v3 transitions and emits one NDJSON record per step.
  `tools/tla/run_tlc.py trace` compiles them into TraceData.tla and runs TLC.

  TLC accepts iff EVERY observed Rust step is an enabled transition of
  ProgramTransition.tla whose post-state matches the real Directory Home's
  observed diskVersion, homeOpen, openVersion, indexState, transitionRuns,
  and complete row state (present, v2val, priority for every RowId).
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "CreateV2" -> CreateV2
      [] step.action = "WriteV2" -> WriteV2(step.row, step.v2val)
      [] step.action = "CloseHome" -> CloseHome
      [] step.action = "ReopenV2" -> ReopenV2
      [] step.action = "OpenV3WithoutTransition" -> OpenV3WithoutTransition
      [] step.action = "OpenV3WithTransition" -> OpenV3WithTransition
      [] step.action = "OpenV3WithTransitionBudgetStop" -> OpenV3WithTransitionBudgetStop
      [] step.action = "OpenV2OnV3" -> OpenV2OnV3
      [] step.action = "WriteV3" -> WriteV3(step.row, step.v2val, step.priority)
      [] step.action = "ReopenV3" -> ReopenV3

MatchesPost(step) ==
    /\ diskVersion' = step.postDiskVersion
    /\ homeOpen' = step.postHomeOpen
    /\ openVersion' = step.postOpenVersion
    /\ indexState' = step.postIndexState
    /\ transitionRuns' = step.postTransitionRuns
    /\ lastOutcome' = step.postOutcome
    /\ \A r \in RowIds :
            /\ rows'[r].present = step.postRows[r].present
            /\ rows'[r].v2val = step.postRows[r].v2val
            /\ rows'[r].priority = step.postRows[r].priority

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

================================================================================
