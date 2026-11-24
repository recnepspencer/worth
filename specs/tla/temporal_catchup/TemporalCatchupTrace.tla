----------------------- MODULE TemporalCatchupTrace -----------------------
EXTENDS TemporalCatchup, TraceData

(*
  Strict trace-validation refinement wrapper for TemporalCatchup.tla.

  The seeded Rust DST in `worth-query-court-decl-host-integration`
  (`host::temporal_catchup::temporal_catchup_dst`) emits one NDJSON record per
  observed step when `WORTH_DST_TRACE_DIR` is set. `tools/tla/run_tlc.py trace`
  compiles the records into `TraceData.tla` (`Trace == << ... >>`) and checks
  that every real Rust transition refines `TemporalCatchup.tla`, satisfies every
  safety invariant, and matches the passively observed post-state.

  STRICT TRACE HONESTY:
    - Never stutters on unknown, mismatched, or forbidden outcomes.
    - Any forged double-fire, premature fire, skipped due occurrence on
      `CaughtUp`, or durable mutation during budget/cancel/auth refusal causes
      `TraceStep` to block and `TraceAccepted` to FAIL.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

\* Start the next independent DST seed cleanly after the prior seed closed or finished.
ResetForNextSeed ==
    /\ appState' = "Open"
    /\ clock' = 0
    /\ observedClock' = 0
    /\ authValid' = TRUE
    /\ intentLifecycle' = [i \in Intents |-> "Absent"]
    /\ intentRev' = [i \in Intents |-> 0]
    /\ intentDue' = [i \in Intents |-> 0]
    /\ intentInterval' = [i \in Intents |-> 0]
    /\ intentMaxFires' = [i \in Intents |-> 0]
    /\ intentFiredCount' = [i \in Intents |-> 0]
    /\ firedKeys' = {}
    /\ totalFired' = 0
    /\ lastPumpStatus' = "Idle"

StepAction(step) ==
    CASE step.action = "InstallIntent" ->
            InstallIntent(step.id, step.due, step.interval, step.maxFires)
      [] step.action = "CancelIntent" ->
            CancelIntent(step.id)
      [] step.action = "AmendIntentDue" ->
            AmendIntentDue(step.id, step.newDue)
      [] step.action = "AdvanceClock" ->
            AdvanceClock(step.newClock)
      [] step.action = "CloseApp" ->
            CloseApp
      [] step.action = "ReopenApp" ->
            ReopenApp
      [] step.action = "ExpireAuth" ->
            ExpireAuth
      [] step.action = "RefreshAuth" ->
            RefreshAuth
      [] step.action = "PumpFire" ->
            PumpFire(step.id, step.targetNow)
      [] step.action = "PumpCatchUpComplete" ->
            PumpCatchUpComplete(step.targetNow)
      [] step.action = "PumpStepLimit" ->
            PumpStepLimit(step.targetNow)
      [] step.action = "PumpBudgetExhausted" ->
            PumpBudgetExhausted
      [] step.action = "PumpCancelled" ->
            PumpCancelled
      [] step.action = "PumpUnauthorized" ->
            PumpUnauthorized
      [] step.action = "Reset" ->
            ResetForNextSeed

MatchesPost(step) ==
    /\ appState' = step.postAppState
    /\ clock' = step.postClock
    /\ observedClock' = step.postObservedClock
    /\ authValid' = step.postAuthValid
    /\ lastPumpStatus' = step.postLastPumpStatus
    /\ totalFired' = step.postTotalFired
    /\ \A i \in Intents :
            /\ intentLifecycle'[i] = step.postIntents[i].lifecycle
            /\ intentRev'[i] = step.postIntents[i].rev
            /\ intentDue'[i] = step.postIntents[i].due
            /\ intentInterval'[i] = step.postIntents[i].interval
            /\ intentMaxFires'[i] = step.postIntents[i].maxFires
            /\ intentFiredCount'[i] = step.postIntents[i].firedCount

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
