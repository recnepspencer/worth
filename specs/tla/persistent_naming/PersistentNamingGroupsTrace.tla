--------------------- MODULE PersistentNamingGroupsTrace ---------------------
EXTENDS PersistentNamingGroupsCandidate, TraceData

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
  A `CandidateEnvelope` record (#361) is accepted iff each later-stage read
  on the candidate its first stage left is exactly Query's candidate rule
  (`ImplRead`), and the envelope then committed that first stage when every
  read answered, each answer equal to what the commit made true, and
  committed nothing otherwise.
*)

VARIABLE pc

traceVars == <<vars, lastRead, pc>>

TraceInit ==
    /\ CInit
    /\ pc = 1

ReadAnswer(r) == [status |-> r.status, targets |-> r.targets]

\* A candidate envelope as Query ran it: every read is the candidate rule's
\* answer; it commits its first stage exactly when every read answered.
CandidateTraceStep(step) ==
    LET cand == CandMemberLive(step.kind, step.g, step.m)
    IN /\ \A r \in step.reads :
              ImplRead(step.kind, cand, r.rk, r.m, r.g, r.sel) = ReadAnswer(r)
       /\ IF step.kind \in Claims \/ \E r \in step.reads : r.status \in Refusals
             THEN UNCHANGED vars
             ELSE /\ CommitFirst(step.kind, step.g, step.m, step.n, step.a)
                  /\ \A r \in step.reads :
                        ReadAnswer(r) = TruthAfter(r.rk, r.m, r.g, r.sel)

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
      [] step.action = "CandidateEnvelope" -> CandidateTraceStep(step)
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
    /\ UNCHANGED lastRead
    /\ MatchesPost(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
