--------------------- MODULE PersistentNamingGroupsTrace ---------------------
EXTENDS PersistentNamingGroups, TraceData

(*
  Trace-validation refinement wrapper for PersistentNamingGroups.tla.

  The seeded Rust group DST in `worth-query-court-decl-host-persistent-naming`
  (`oracle::persistent_naming_groups_dst`) drives real installed commits and
  emits one NDJSON record per step; `tools/tla/run_tlc.py trace` compiles
  them into TraceData.tla (`Trace == << ... >>`) and runs TLC on this spec.
  TLC accepts iff every observed Rust transition is a step of
  PersistentNamingGroups.tla whose post-state matches the live loops (with
  their members) and retired loops the installed world answered, and every
  `Observe` record's resolutions (each ever-born loop, `Exact` and
  `AllDescendants`) equal the model's answers from the derived index.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "CommitBorn" -> CommitBorn(step.g, step.ms, step.a)
      [] step.action = "CommitRevive" -> CommitRevive(step.g, step.ms)
      [] step.action = "CommitSplit" -> CommitSplit(step.g, step.piece, step.n, step.a)
      [] step.action = "CommitSplitAway" ->
            CommitSplitAway(step.g, step.p1, step.p2, step.n1, step.n2, step.a1, step.a2)
      [] step.action = "CommitMerge" -> CommitMerge(step.k, step.x)
      [] step.action = "CommitDissolve" -> CommitDissolve(step.g)
      [] step.action = "CommitDeleteFree" -> CommitDeleteFree(step.m)
      [] step.action = "CommitDeleteMember" -> CommitDeleteMember(step.g, step.m)
      [] step.action = "CommitDeleteAnchor" -> CommitDeleteAnchor(step.g, step.n, step.a)
      [] step.action = "RefuseOverlap" -> RefuseOverlap(step.m)
      [] step.action = "Observe" ->
            /\ UNCHANGED vars
            /\ \A r \in step.resolved :
                    ResolveFromIndex(r.g, r.sel) = [status |-> r.status, targets |-> r.targets]

LiveView(gs) == {[g |-> g, members |-> gs[g].members] : g \in {h \in GroupNames : gs[h].state = "live"}}
RetiredView(gs) == {h \in GroupNames : gs[h].state = "retired"}

MatchesPost(step) ==
    /\ LiveView(groups') = step.postLive
    /\ RetiredView(groups') = step.postRetired

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
