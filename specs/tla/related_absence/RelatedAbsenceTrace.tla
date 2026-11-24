------------------------- MODULE RelatedAbsenceTrace ------------------------
EXTENDS RelatedAbsence, TraceData

(*
  Trace-validation refinement wrapper for RelatedAbsence.tla.

  The seeded Rust DST in `worth-query-court-decl-host-where-filter` emits one
  NDJSON record per step when `WORTH_DST_TRACE_DIR` is set, including:
    - Real `ObserveQuery` results & real governance denials (`allowRel`/`allowField`)
    - Real `CheckCurrentness` outcomes from `observe_ready_source` (Current vs
      SourceChanged) across in-scope edits, edge mutations, AND out-of-scope
      blocker mutations (`t6` outside `ScopeTasks`), ensuring a missing/empty
      footprint is rejected by TLC!
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "ToggleStatus" ->
            ToggleStatus(step.task)
      [] step.action = "AddEdge" ->
            AddEdge(step.from, step.to)
      [] step.action = "RemoveEdge" ->
            RemoveEdge(step.from, step.to)
      [] step.action = "ObserveQuery" ->
            /\ ObserveQuery(step.pol, step.dir, step.withPred, step.allowRel, step.allowField)
            /\ obs'.denied = step.denied
            /\ obs'.rows = step.rows
      [] step.action = "CheckCurrentness" ->
            /\ CheckCurrentness(step.stillCurrent)
            /\ currentCheck' = TRUE
      [] step.action = "PinnedRead" ->
            /\ PinnedRead(step.snapIdx, step.pol, step.dir, step.withPred)
            /\ GraphEval(history[step.snapIdx], step.pol, step.dir, step.withPred) = step.rows

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
