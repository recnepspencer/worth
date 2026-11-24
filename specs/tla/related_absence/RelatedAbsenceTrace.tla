------------------------- MODULE RelatedAbsenceTrace ------------------------
EXTENDS RelatedAbsence, TraceData

(*
  Trace-validation refinement wrapper for RelatedAbsence.tla.

  The seeded Rust DST in `worth-query-court-decl-host-where-filter`
  (`oracle::related_absence_dst`) emits one NDJSON record per step when
  `WORTH_DST_TRACE_DIR` is set; `tools/tla/run_tlc.py trace` compiles those
  records into `TraceData.tla` (`Trace == << ... >>`) and checks that every
  observed Rust step refines `RelatedAbsence.tla` and satisfies all invariants.
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
