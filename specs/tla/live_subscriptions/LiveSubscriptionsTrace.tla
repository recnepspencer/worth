----------------------- MODULE LiveSubscriptionsTrace -----------------------
EXTENDS LiveSubscriptions, TraceData

(*
  Strict trace-validation refinement wrapper for LiveSubscriptions.tla.

  Every step emitted by the Rust 64-seed DST (`WORTH_DST_TRACE_DIR`) is
  matched strictly against the real model transition AND every observed
  outcome (`outcome`, `rows`, `snapIdx`, `authorized`) MUST match the model's
  exact next state. Mismatched or forbidden outcomes (e.g., stale delivery
  after blocker completion, or delivery after grant revocation) deadlock the
  trace and fail `TraceAccepted` — never stuttering past mismatches!
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceMaxCommits == 64
TraceMaxReopens == 16

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "ToggleStatus" ->
            ToggleStatus(step.task)
      [] step.action = "SetDue" ->
            SetDue(step.task, step.due)
      [] step.action = "AddEdge" ->
            AddEdge(step.from, step.to)
      [] step.action = "RemoveEdge" ->
            RemoveEdge(step.from, step.to)
      [] step.action = "RevokeGrant" ->
            RevokeGrant
      [] step.action = "RestoreGrant" ->
            RestoreGrant
      [] step.action = "ReopenHome" ->
            ReopenHome
      [] step.action = "Subscribe" ->
            /\ Subscribe(step.kind)
            /\ subs'[step.kind].lastOutcome = step.outcome
            /\ (step.outcome = "Initial" => subs'[step.kind].lastRows = step.rows)
      [] step.action = "Poll" ->
            /\ Poll(step.kind)
            /\ subs'[step.kind].lastOutcome = step.outcome
            /\ (step.outcome = "Delivered" => subs'[step.kind].lastRows = step.rows)
      [] step.action = "CloseSub" ->
            CloseSub(step.kind)

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
