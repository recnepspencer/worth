------------------------- MODULE SharedAccessTrace --------------------------
EXTENDS SharedAccess, TraceData

(*
  Trace-validation refinement wrapper for SharedAccess.tla.

  The seeded Rust DST in `worth-query-court-decl-host-integration`
  (`host::shared_access`) emits one NDJSON record per step from strictly
  passive observations on the real Directory Home and authenticated loopback
  remote server. `tools/tla/run_tlc.py trace` compiles them into TraceData.tla
  and runs TLC on this spec. TLC accepts iff every observed Rust step matches
  the exact SharedAccess.tla transition, exact denial reason or delivery
  outcome, exact complete ordered scoped row sequence, and exact passive
  post-state while preserving every safety invariant.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "OwnerWrite" ->
            /\ OwnerWrite(step.scope)
            /\ step.outcome = "Ok"
            /\ step.rows = SingletonRow(step.scope, scopeValue'[step.scope])
      [] step.action = "GrantScopeAccess" ->
            /\ GrantScopeAccess(step.principal, step.scope, step.access)
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows
      [] step.action = "RevokeScopeAccess" ->
            /\ RevokeScopeAccess(step.principal, step.scope)
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows
      [] step.action = "RemoteRead" ->
            RemoteReadOutcome(step.principal, step.scope, step.outcome, step.rows)
      [] step.action = "RemoteWatchOpen" ->
            RemoteWatchOpenOutcome(step.principal, step.scope, step.outcome, step.rows)
      [] step.action = "RemoteWatchPoll" ->
            RemoteWatchPollOutcome(step.principal, step.scope, step.outcome, step.rows)
      [] step.action = "CloseHome" ->
            /\ CloseHome
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows
      [] step.action = "ReopenHome" ->
            /\ ReopenHome
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows
      [] step.action = "TamperAuthToken" ->
            /\ TamperAuthToken(step.principal)
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows
      [] step.action = "RestoreAuthToken" ->
            /\ RestoreAuthToken(step.principal)
            /\ step.outcome = "Ok"
            /\ step.rows = EmptyRows

MatchesPostState(step) ==
    /\ homeOpen' = step.postHomeOpen
    /\ incarnation' = step.postIncarnation
    /\ scopeValue'["Project1"] = step.postProject1Value
    /\ scopeValue'["Project2"] = step.postProject2Value
    /\ durableGrants'[<<"Alice", "Project1">>] = step.postGrantAliceProject1
    /\ durableGrants'[<<"Alice", "Project2">>] = step.postGrantAliceProject2
    /\ durableGrants'[<<"Bob", "Project1">>] = step.postGrantBobProject1
    /\ durableGrants'[<<"Bob", "Project2">>] = step.postGrantBobProject2
    /\ watches'[<<"Alice", "Project1">>] = step.postWatchAliceProject1
    /\ watches'[<<"Alice", "Project2">>] = step.postWatchAliceProject2
    /\ watches'[<<"Bob", "Project1">>] = step.postWatchBobProject1
    /\ watches'[<<"Bob", "Project2">>] = step.postWatchBobProject2

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ MatchesPostState(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
