---------------------- MODULE OrderedIndexAdapterTrace ----------------------
EXTENDS OrderedIndexAdapter, TraceData

(*
  Strict trace-validation refinement wrapper for OrderedIndexAdapter.tla.

  Every step emitted by the Rust 64-seed DST (`ordered_index_dst`) must match
  an exact legal action of OrderedIndexAdapter.tla AND its observed post-state
  (lifecycle, live row ids, and exact query/watch match sets).

  NO mismatched, unknown, or forbidden outcome is allowed to stutter:
  `TraceDone` is enabled ONLY after `pc > Len(Trace)`, and `TraceStep` has no
  fallback branch, so any violation causes TLC deadlock / liveness failure.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "Insert" ->
            InsertRow(step.row, [lo |-> step.lo, hi |-> step.hi])
      [] step.action = "Update" ->
            UpdateRow(step.row, [lo |-> step.lo, hi |-> step.hi])
      [] step.action = "Delete" ->
            DeleteRow(step.row)
      [] step.action = "ColdReopen" ->
            ColdReopen
      [] step.action = "RebuildIndex" ->
            RebuildIndex
      [] step.action = "RefuseNotReady" ->
            RefuseNotReady([lo |-> step.probeLo, hi |-> step.probeHi])
      [] step.action = "Query" ->
            LET probe == [lo |-> step.probeLo, hi |-> step.probeHi]
            IN /\ ExecuteQuery(probe)
               /\ step.observedMatches = PrunedQuery(tree, probe, rows)
               /\ step.observedMatches = OracleQuery(rows, probe)
      [] step.action = "WatchOpen" ->
            LET probe == [lo |-> step.probeLo, hi |-> step.probeHi]
            IN /\ WatchOpen(probe)
               /\ step.observedMatches = watchDelivered'
               /\ step.observedMatches = OracleQuery(rows, probe)
      [] step.action = "WatchDeliver" ->
            /\ WatchDeliver
            /\ step.observedMatches = watchDelivered'
            /\ step.observedMatches = OracleQuery(rows, watchProbe)
      [] step.action = "WatchClose" ->
            WatchClose
      [] step.action = "RefuseBudgetExhausted" ->
            RefuseBudgetExhausted

MatchesPost(step) ==
    /\ lifecycle' = step.postLifecycle
    /\ LiveRows(rows') = step.postLiveRows

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
