------------------------ MODULE PersistentNamingTrace -----------------------
EXTENDS PersistentNaming, TraceData

(*
  Trace-validation refinement wrapper for PersistentNaming.tla.

  The seeded Rust DST in `worth-query-court-decl-host-persistent-naming`
  (`oracle::persistent_naming_dst`) emits one NDJSON record per step;
  `tools/tla/run_tlc.py trace` compiles them into TraceData.tla
  (`Trace == << ... >>`) and runs TLC on this spec. TLC accepts iff every
  observed Rust transition is a step of PersistentNaming.tla whose post-state
  matches the observed live and retired names and log length, and every
  `Observe` record's resolutions (each ever-minted name, unscoped and per
  owner, Exact and AllDescendants) equal the model's answers.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "CommitCreate" ->
            CommitCreate(step.name, step.owner, step.part)
      [] step.action = "RefuseDuplicateBirth" ->
            RefuseDuplicateBirth(step.name)
      [] step.action = "CommitRegeneratePreserve" ->
            CommitRegeneratePreserve(step.name, step.owner, step.part)
      [] step.action = "CommitSplit" ->
            CommitSplit(step.parent, step.c1, step.c2, step.o1, step.o2, step.part)
      [] step.action = "CommitMergeKeepParentInPlace" ->
            CommitMergeKeepParentInPlace(step.keeper, step.other)
      [] step.action = "CommitMergeMintNew" ->
            CommitMergeMintNew(step.p1, step.p2, step.minted, step.owner, step.part)
      [] step.action = "CommitRetire" ->
            CommitRetire(step.name)
      [] step.action = "CommitRestore" ->
            CommitRestore(step.name, step.owner, step.part)
      [] step.action = "DestroyAndRebuildIndex" ->
            DestroyAndRebuildIndex
      [] step.action = "Observe" ->
            /\ UNCHANGED vars
            /\ \A r \in step.resolved :
                    ResolveFromIndex(r.name, r.scope, r.sel)
                        = [status |-> r.status, targets |-> r.targets]

LiveNameSet(liveMap) == {n \in Names : liveMap[n].alive}

MatchesPost(step) ==
    /\ LiveNameSet(live') = step.postLive
    /\ retired' = step.postRetired
    /\ Len(events') = step.postEventCount

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
