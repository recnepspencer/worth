-------------------------- MODULE TemporalCatchup --------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Full-stack specification of desktop recurring & one-shot timed intents and
  bounded caller-pumped reopen catch-up (worth-query-host/src/temporal_contribution
  + worth-query-execution recurring temporal intent advancement).

  Models:
    - Authored timed intents with durable `(lifecycle, rev, due, interval,
      maxFires, firedCount)` stored in authoritative Relational state.
    - Desktop application lifecycle (`Open` vs `Closed`). Clock advances
      monotonically while Open or Closed; closing drops live clock leases
      (`observedClock' = 0`), and reopening reconstructs active intents from
      authoritative Relational state.
    - Bounded caller-pumped catch-up steps:
        * `PumpFire(i, targetNow)`: advances observed clock to `targetNow <= clock`
          and fires one due active intent `i` (`intentDue[i] <= targetNow`),
          atomically advancing `rev` and either rescheduling `due + interval`
          (if recurring with remaining fires) or transitioning to `Completed`.
        * `PumpCatchUpComplete(targetNow)`: advances observed clock to `targetNow`
          when no active intent has `due <= targetNow`, recording `CaughtUp`.
        * `PumpStepLimit(targetNow)`: bounded max-steps ceiling hit mid-catch-up
          while at least one active intent still has `due <= targetNow`.
        * `PumpBudgetExhausted` / `PumpCancelled` / `PumpUnauthorized`: refused
          or interrupted pump steps that leave durable intent state untouched.
    - Ordinary user mutations while Open: `InstallIntent`, `CancelIntent`,
      `AmendIntentDue`.
    - Principal authority lifecycle: `ExpireAuth`, `RefreshAuth`.

  Safety invariants:
    - TypeOK
    - NoPrematureFire: no occurrence ever fires before its due time.
    - NoDoubleFire: no `(id, rev)` occurrence fires twice across close/reopen,
      retries, or partial-batch stops (`Cardinality(firedKeys) = totalFired`).
    - NoSkippedOccurrenceWhenCaughtUp: when Open and `CaughtUp` at `clock`,
      every active intent has `intentDue[i] > clock`.
    - TerminalIntentsStopFiring: completed intents have `firedCount = maxFires`
      and no intent ever exceeds `maxFires`.
    - DurableStateConsistent: active/completed/cancelled/absent invariants hold
      and `observedClock <= clock` (0 when Closed).

  Liveness property:
    - EventuallyCaughtUp: once the app stays Open with valid authority and
      clock advances settle, fair bounded catch-up steps eventually reach
      `CaughtUp`.
*)

CONSTANTS
    Intents,        \* Finite set of intent identities, e.g. {"i1", "i2"}
    MaxTime,        \* Upper bound on clock coordinates in model checking
    MaxAmendRev,    \* Upper bound on pre-amend revision in model checking
    MaxFiresLimit   \* Upper bound on maxFires per intent in model checking

AppStates == {"Open", "Closed"}
Lifecycles == {"Absent", "Active", "Completed", "Cancelled"}
PumpStatuses == {
    "Idle",
    "Progress",
    "CaughtUp",
    "StepLimitReached",
    "BudgetExhausted",
    "Cancelled",
    "AuthorizationRefused",
    "Closed"
}

VARIABLES
    appState,          \* "Open" or "Closed"
    clock,             \* Authoritative monotonic host time (Nat)
    observedClock,     \* Last clock coordinate observed by live runtime (0 when Closed)
    authValid,         \* TRUE iff fresh execution authority / principal is valid
    intentLifecycle,   \* [Intents -> Lifecycles]
    intentRev,         \* [Intents -> Nat]
    intentDue,         \* [Intents -> Nat]
    intentInterval,    \* [Intents -> Nat] (0 = one-shot, > 0 = recurring interval)
    intentMaxFires,    \* [Intents -> Nat]
    intentFiredCount,  \* [Intents -> Nat]
    firedKeys,         \* Set of [id |-> i, rev |-> r, due |-> d, firedAt |-> t]
    totalFired,        \* Total number of fires ever performed (must equal Cardinality(firedKeys))
    lastPumpStatus     \* Most recent catch-up pump outcome

durableVars == <<intentLifecycle, intentRev, intentDue, intentInterval, intentMaxFires, intentFiredCount, firedKeys, totalFired>>
vars == <<appState, clock, observedClock, authValid, durableVars, lastPumpStatus>>

Init ==
    /\ appState = "Open"
    /\ clock = 0
    /\ observedClock = 0
    /\ authValid = TRUE
    /\ intentLifecycle = [i \in Intents |-> "Absent"]
    /\ intentRev = [i \in Intents |-> 0]
    /\ intentDue = [i \in Intents |-> 0]
    /\ intentInterval = [i \in Intents |-> 0]
    /\ intentMaxFires = [i \in Intents |-> 0]
    /\ intentFiredCount = [i \in Intents |-> 0]
    /\ firedKeys = {}
    /\ totalFired = 0
    /\ lastPumpStatus = "Idle"

\* ---------------------------------------------------------------------------
\* Helper predicates
\* ---------------------------------------------------------------------------

DueActiveIntents(t) ==
    {i \in Intents : intentLifecycle[i] = "Active" /\ intentDue[i] <= t}

CaughtUp ==
    /\ appState = "Open"
    /\ observedClock = clock
    /\ DueActiveIntents(clock) = {}
    /\ lastPumpStatus = "CaughtUp"

\* ---------------------------------------------------------------------------
\* Authoritative Intent Lifecycle (User / Application Mutations while Open)
\* ---------------------------------------------------------------------------

\* Author a new timed intent (one-shot if maxFires = 1 /\ interval = 0,
\* recurring if maxFires > 1 /\ interval > 0).
InstallIntent(i, due, interval, maxFires) ==
    /\ appState = "Open"
    /\ i \in Intents
    /\ intentLifecycle[i] = "Absent"
    /\ due \in 1..MaxTime
    /\ maxFires \in 1..MaxFiresLimit
    /\ IF maxFires = 1 THEN interval = 0 ELSE interval \in 1..MaxTime
    /\ intentLifecycle' = [intentLifecycle EXCEPT ![i] = "Active"]
    /\ intentRev' = [intentRev EXCEPT ![i] = 1]
    /\ intentDue' = [intentDue EXCEPT ![i] = due]
    /\ intentInterval' = [intentInterval EXCEPT ![i] = interval]
    /\ intentMaxFires' = [intentMaxFires EXCEPT ![i] = maxFires]
    /\ intentFiredCount' = [intentFiredCount EXCEPT ![i] = 0]
    /\ lastPumpStatus' = IF due <= clock THEN "Idle" ELSE lastPumpStatus
    /\ UNCHANGED <<appState, clock, observedClock, authValid, firedKeys, totalFired>>

\* User cancels an active intent before its next occurrence fires.
CancelIntent(i) ==
    /\ appState = "Open"
    /\ i \in Intents
    /\ intentLifecycle[i] = "Active"
    /\ intentLifecycle' = [intentLifecycle EXCEPT ![i] = "Cancelled"]
    /\ intentRev' = [intentRev EXCEPT ![i] = @ + 1]
    /\ UNCHANGED <<appState, clock, observedClock, authValid, intentDue, intentInterval, intentMaxFires, intentFiredCount, firedKeys, totalFired, lastPumpStatus>>

\* User reschedules an active intent to a new due coordinate.
AmendIntentDue(i, newDue) ==
    /\ appState = "Open"
    /\ i \in Intents
    /\ intentLifecycle[i] = "Active"
    /\ intentRev[i] <= MaxAmendRev
    /\ newDue \in 1..MaxTime
    /\ newDue # intentDue[i]
    /\ intentRev' = [intentRev EXCEPT ![i] = @ + 1]
    /\ intentDue' = [intentDue EXCEPT ![i] = newDue]
    /\ lastPumpStatus' = IF newDue <= clock THEN "Idle" ELSE lastPumpStatus
    /\ UNCHANGED <<appState, clock, observedClock, authValid, intentLifecycle, intentInterval, intentMaxFires, intentFiredCount, firedKeys, totalFired>>

\* ---------------------------------------------------------------------------
\* Desktop App Close / Reopen & Clock Progression
\* ---------------------------------------------------------------------------

\* Monotonic clock advance (may jump while Open or while Closed!).
AdvanceClock(newClock) ==
    /\ newClock \in (clock + 1)..MaxTime
    /\ clock' = newClock
    /\ lastPumpStatus' = IF appState = "Open" THEN "Idle" ELSE "Closed"
    /\ UNCHANGED <<appState, observedClock, authValid, durableVars>>

\* Close the desktop application: durable state persists, live clock leases drop.
CloseApp ==
    /\ appState = "Open"
    /\ appState' = "Closed"
    /\ observedClock' = 0
    /\ lastPumpStatus' = "Closed"
    /\ UNCHANGED <<clock, authValid, durableVars>>

\* Reopen the desktop application: durable intents survive intact; fresh clock starts at 0.
ReopenApp ==
    /\ appState = "Closed"
    /\ appState' = "Open"
    /\ observedClock' = 0
    /\ lastPumpStatus' = "Idle"
    /\ UNCHANGED <<clock, authValid, durableVars>>

\* Execution authority / principal expiry and refresh.
ExpireAuth ==
    /\ authValid
    /\ authValid' = FALSE
    /\ UNCHANGED <<appState, clock, observedClock, durableVars, lastPumpStatus>>

RefreshAuth ==
    /\ ~authValid
    /\ authValid' = TRUE
    /\ UNCHANGED <<appState, clock, observedClock, durableVars, lastPumpStatus>>

\* ---------------------------------------------------------------------------
\* Bounded Caller-Pumped Catch-Up Transitions
\* ---------------------------------------------------------------------------

\* Fire one due active intent `i` at target coordinate `targetNow <= clock`
\* within one bounded observation step (matching `processing.rs` single-commit
\* per-snapshot advancement). Strictly forbids premature firing
\* (`intentDue[i] <= targetNow`), double firing, firing while Closed, firing
\* without valid authority, or firing terminal intents.
PumpFire(i, targetNow) ==
    /\ appState = "Open"
    /\ authValid
    /\ i \in Intents
    /\ targetNow \in observedClock..clock
    /\ intentLifecycle[i] = "Active"
    /\ intentDue[i] <= targetNow
    /\ ~\E f \in firedKeys : f.id = i /\ f.rev = intentRev[i]
    /\ observedClock' = targetNow
    /\ intentLifecycle' = [intentLifecycle EXCEPT ![i] =
            IF intentFiredCount[i] + 1 = intentMaxFires[i] THEN "Completed" ELSE "Active"]
    /\ intentRev' = [intentRev EXCEPT ![i] = @ + 1]
    /\ intentDue' = [intentDue EXCEPT ![i] =
            IF intentFiredCount[i] + 1 < intentMaxFires[i]
            THEN @ + intentInterval[i]
            ELSE @]
    /\ intentFiredCount' = [intentFiredCount EXCEPT ![i] = @ + 1]
    /\ firedKeys' = firedKeys \union {
            [id |-> i, rev |-> intentRev[i], due |-> intentDue[i], firedAt |-> targetNow]
       }
    /\ totalFired' = totalFired + 1
    /\ lastPumpStatus' = "Progress"
    /\ UNCHANGED <<appState, clock, authValid, intentInterval, intentMaxFires>>

\* Catch-up observes `targetNow` with zero remaining due active intents at or before `targetNow`.
PumpCatchUpComplete(targetNow) ==
    /\ appState = "Open"
    /\ authValid
    /\ targetNow \in observedClock..clock
    /\ DueActiveIntents(targetNow) = {}
    /\ observedClock' = targetNow
    /\ lastPumpStatus' = "CaughtUp"
    /\ UNCHANGED <<appState, clock, authValid, durableVars>>

\* Bounded max-steps ceiling reached while at least one due active intent still remains.
PumpStepLimit(targetNow) ==
    /\ appState = "Open"
    /\ authValid
    /\ targetNow \in observedClock..clock
    /\ DueActiveIntents(targetNow) # {}
    /\ observedClock' = targetNow
    /\ lastPumpStatus' = "StepLimitReached"
    /\ UNCHANGED <<appState, clock, authValid, durableVars>>

\* Pump stopped due to Work/Memory budget exhaustion; durable state stays untouched.
\* If the clock observation was accepted before reentry stopped, observedClock advances monotonically.
PumpBudgetExhausted(targetNow) ==
    /\ appState = "Open"
    /\ targetNow \in observedClock..clock
    /\ observedClock' = targetNow
    /\ lastPumpStatus' = "BudgetExhausted"
    /\ UNCHANGED <<appState, clock, authValid, durableVars>>

\* Pump stopped due to caller cancellation; durable state stays untouched.
PumpCancelled(targetNow) ==
    /\ appState = "Open"
    /\ targetNow \in observedClock..clock
    /\ observedClock' = targetNow
    /\ lastPumpStatus' = "Cancelled"
    /\ UNCHANGED <<appState, clock, authValid, durableVars>>

\* Pump refused because execution authority / principal is expired or rejected;
\* durable state stays untouched.
PumpUnauthorized(targetNow) ==
    /\ appState = "Open"
    /\ ~authValid
    /\ targetNow \in observedClock..clock
    /\ observedClock' = targetNow
    /\ lastPumpStatus' = "AuthorizationRefused"
    /\ UNCHANGED <<appState, clock, authValid, durableVars>>

Next ==
    \/ \E i \in Intents, d \in 1..MaxTime, iv \in 0..MaxTime, mf \in 1..MaxFiresLimit :
            InstallIntent(i, d, iv, mf)
    \/ \E i \in Intents : CancelIntent(i)
    \/ \E i \in Intents, d \in 1..MaxTime : AmendIntentDue(i, d)
    \/ \E t \in 1..MaxTime : AdvanceClock(t)
    \/ CloseApp
    \/ ReopenApp
    \/ ExpireAuth
    \/ RefreshAuth
    \/ \E i \in Intents, t \in 0..MaxTime : PumpFire(i, t)
    \/ \E t \in 0..MaxTime : PumpCatchUpComplete(t)
    \/ \E t \in 0..MaxTime : PumpStepLimit(t)
    \/ \E t \in 0..MaxTime : PumpBudgetExhausted(t)
    \/ \E t \in 0..MaxTime : PumpCancelled(t)
    \/ \E t \in 0..MaxTime : PumpUnauthorized(t)

PumpFireProgress == \E i \in Intents : PumpFire(i, clock)
PumpCompleteProgress == PumpCatchUpComplete(clock)

\* Strong fairness on actual catch-up progress steps guarantees that once the
\* app stays Open with valid authority and the clock settles, all due
\* occurrences <= clock eventually fire and catch-up completes.
Fairness ==
    /\ SF_vars(PumpFireProgress)
    /\ SF_vars(PumpCompleteProgress)

Spec == Init /\ [][Next]_vars /\ Fairness

\* ===========================================================================
\* Safety Invariants & Liveness Property
\* ===========================================================================

TypeOK ==
    /\ appState \in AppStates
    /\ clock \in 0..MaxTime
    /\ observedClock \in 0..MaxTime
    /\ authValid \in BOOLEAN
    /\ intentLifecycle \in [Intents -> Lifecycles]
    /\ intentRev \in [Intents -> Nat]
    /\ intentDue \in [Intents -> Nat]
    /\ intentInterval \in [Intents -> 0..MaxTime]
    /\ intentMaxFires \in [Intents -> 0..MaxFiresLimit]
    /\ intentFiredCount \in [Intents -> 0..MaxFiresLimit]
    /\ totalFired \in Nat
    /\ lastPumpStatus \in PumpStatuses

\* (a) NEVER fire an occurrence before its due time.
NoPrematureFire ==
    \A f \in firedKeys : f.due <= f.firedAt /\ f.firedAt <= clock

\* (b) NEVER double-fire the same due occurrence `(id, rev)` across close/reopen/retry/partial-stop.
NoDoubleFire ==
    /\ Cardinality(firedKeys) = totalFired
    /\ \A f1, f2 \in firedKeys :
            (f1.id = f2.id /\ f1.rev = f2.rev) => f1 = f2
    /\ \A i \in Intents :
            Cardinality({f \in firedKeys : f.id = i}) = intentFiredCount[i]

\* (c) NEVER skip a missed active occurrence once caught up to `now`.
NoSkippedOccurrenceWhenCaughtUp ==
    (appState = "Open" /\ lastPumpStatus = "CaughtUp" /\ observedClock = clock) =>
        \A i \in Intents : intentLifecycle[i] = "Active" => intentDue[i] > clock

\* (d) Cancelled/completed intents stop firing and never exceed maxFires.
TerminalIntentsStopFiring ==
    \A i \in Intents :
        /\ intentFiredCount[i] <= intentMaxFires[i]
        /\ (intentLifecycle[i] = "Completed") => (intentFiredCount[i] = intentMaxFires[i])
        /\ (intentLifecycle[i] = "Absent") => (intentFiredCount[i] = 0 /\ intentRev[i] = 0)

\* (e) Durable state and clock observation remain consistent across close/reopen and refusals.
DurableStateConsistent ==
    /\ observedClock <= clock
    /\ (appState = "Closed") => (observedClock = 0)
    /\ \A i \in Intents :
            (intentLifecycle[i] = "Active") =>
                (intentRev[i] >= 1 /\ intentFiredCount[i] < intentMaxFires[i] /\ intentDue[i] >= 1)

\* Liveness: once the app stays Open with valid authority, bounded catch-up
\* eventually fires every due occurrence `<= clock` and reaches `CaughtUp`.
EventuallyCaughtUp ==
    (<>[] (appState = "Open" /\ authValid)) => <>CaughtUp

=============================================================================
