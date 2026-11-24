---------------------- MODULE OrderedIndexAdapterTrace ----------------------
EXTENDS OrderedIndexAdapter, TraceData

(*
  Strict trace-validation refinement wrapper for OrderedIndexAdapter.tla.

  Every step emitted by the Rust DST must match an exact legal action of
  OrderedIndexAdapter.tla AND its full observed post-state:
  - `postLifecycle` ("Created" or "Ready"),
  - `postRows` (exact observed Source values `[lo, hi, title]` for all RowIds),
  - `postWatchActive`, `postTitleWatchPending`, `postFocusWatchPending`,
  - exact `errorKind` ("NotReady" or "BudgetExhausted") on refusals,
  - BOTH title-prefix AND focus-interval observed match sets on Query,
    WatchOpen, WatchDeliver*, and RefuseBudgetExhausted.

  NO mismatched, unknown, or forbidden outcome is allowed to stutter:
  `TraceDone` is enabled ONLY after `pc > Len(Trace)`, and `TraceStep` has no
  fallback branch. With TLC's default deadlock check enabled, every state
  with `pc <= Len(Trace)` MUST advance via `TraceStep` (`pc' = pc + 1`), so
  any rejected step halts TLC immediately with `Error: Deadlock reached.`
  and acceptance proves the entire trace reached `pc = Len(Trace) + 1` with
  all 8 safety invariants holding at every step.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepRowVal(step) ==
    [lo |-> step.lo, hi |-> step.hi, title |-> step.title]

StepFocusProbe(step) ==
    [lo |-> step.probeLo, hi |-> step.probeHi]

StepAction(step) ==
    CASE step.action = "Insert" ->
            InsertRow(step.row, StepRowVal(step))
      [] step.action = "Update" ->
            UpdateRow(step.row, StepRowVal(step))
      [] step.action = "Delete" ->
            DeleteRow(step.row)
      [] step.action = "ColdReopen" ->
            ColdReopen
      [] step.action = "RebuildIndex" ->
            RebuildIndex
      [] step.action = "RefuseNotReady" ->
            /\ step.errorKind = "NotReady"
            /\ RefuseNotReady(step.titleProbe, StepFocusProbe(step))
      [] step.action = "Query" ->
            LET fp == StepFocusProbe(step)
            IN /\ ExecuteQuery(step.titleProbe, fp)
               /\ step.observedTitleMatches = OracleTitleQuery(rows, step.titleProbe)
               /\ step.observedFocusMatches = PrunedQuery(tree, fp, rows)
               /\ step.observedFocusMatches = OracleFocusQuery(rows, fp)
      [] step.action = "WatchOpen" ->
            LET fp == StepFocusProbe(step)
            IN /\ WatchOpen(step.titleProbe, fp)
               /\ step.observedTitleMatches = titleWatchDelivered'
               /\ step.observedTitleMatches = OracleTitleQuery(rows, step.titleProbe)
               /\ step.observedFocusMatches = focusWatchDelivered'
               /\ step.observedFocusMatches = OracleFocusQuery(rows, fp)
      [] step.action = "WatchDeliverTitle" ->
            /\ WatchDeliverTitle
            /\ step.observedMatches = titleWatchDelivered'
            /\ step.observedMatches = OracleTitleQuery(rows, titleWatchProbe)
      [] step.action = "WatchDeliverFocus" ->
            /\ WatchDeliverFocus
            /\ step.observedMatches = focusWatchDelivered'
            /\ step.observedMatches = OracleFocusQuery(rows, focusWatchProbe)
      [] step.action = "WatchPollIdle" ->
            WatchPollIdle
      [] step.action = "WatchClose" ->
            WatchClose
      [] step.action = "RefuseBudgetExhausted" ->
            LET fp == StepFocusProbe(step)
            IN /\ step.errorKind = "BudgetExhausted"
               /\ RefuseBudgetExhausted(step.row, StepRowVal(step))
               /\ step.observedTitleMatches = OracleTitleQuery(rows, step.titleProbe)
               /\ step.observedFocusMatches = PrunedQuery(tree, fp, rows)
               /\ step.observedFocusMatches = OracleFocusQuery(rows, fp)

MatchesPost(step) ==
    /\ lifecycle' = step.postLifecycle
    /\ \A r \in RowIds : rows'[r] = step.postRows[r]
    /\ watchActive' = step.postWatchActive
    /\ titleWatchPending' = step.postTitleWatchPending
    /\ focusWatchPending' = step.postFocusWatchPending

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ MatchesPost(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars

=============================================================================
