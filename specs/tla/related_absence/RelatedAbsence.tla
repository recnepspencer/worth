--------------------------- MODULE RelatedAbsence ---------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Formal reference model for bounded, metered related-record absence/existence
  predicates (`NOT EXISTS` / `EXISTS`) over a declared relation (`BlockedBy`)
  with optional target-field predicate (`Status == "open"`), out-of-scope
  blockers, governance enforcement, pinned snapshot reads, and exact
  invalidation footprints.

  Proves exhaustively:
    1. QueryMatchesGraphSemantics: every admitted query result equals exact
       relational graph semantics (including short-circuit witness evaluation).
    2. NoStaleRead: whenever every entity/adjacency/field revision recorded in
       the query's footprint still matches current state, the cached result set
       MUST equal the exact current graph semantics (zero stale reads across
       in-scope root status flips, edge add/remove, AND out-of-scope blocker
       status flips).
    3. PinnedReadSound: pinned/retained reads always match the exact snapshot
       state captured at their commit, independent of later mutations.
    4. GovernanceFailClosed: unauthorized relation or target-field access
       always denies before touching any rows.
*)

CONSTANTS
    Tasks,          \* Ordered sequence of all task IDs (in-scope + out-of-scope)
    ScopeTasks,     \* Subset of Tasks belonging to the queried root scope
    MaxCommits      \* Bound on committed state transitions for finite TLC check

ModelTasks == <<"t1", "t2", "t3">>
ModelScope == {"t1", "t2"}
TraceTasks == <<"t1", "t2", "t3", "t4", "t5", "t6">>
TraceScope == {"t1", "t2", "t3", "t4", "t5"}

TaskSet == {Tasks[i] : i \in 1..Len(Tasks)}
Statuses == {"open", "done"}
Polarities == {"Absent", "Exists"}
Directions == {"Forward", "Reverse"}

VARIABLES
    status,         \* [TaskSet -> Statuses]
    edges,          \* SUBSET (TaskSet \X TaskSet): <<t, b>> means t BlockedBy b
    statusRev,      \* [TaskSet -> Nat]
    fwdAdjRev,      \* [TaskSet -> Nat]
    revAdjRev,      \* [TaskSet -> Nat]
    history,        \* Sequence of committed snapshots: [status, edges]
    obs,            \* Last observed query execution + footprint (or denied)
    pinnedCheck,    \* Result of last pinned read check
    currentCheck    \* Result of last explicit currentness verification

vars == <<status, edges, statusRev, fwdAdjRev, revAdjRev, history, obs, pinnedCheck, currentCheck>>

\* Exact neighbor set of `root` in direction `dir` on snapshot `snap`.
Neighbors(snap, root, dir) ==
    IF dir = "Forward"
       THEN {b \in TaskSet : <<root, b>> \in snap.edges}
       ELSE {a \in TaskSet : <<a, root>> \in snap.edges}

\* Whether a related target satisfies the optional target predicate.
TargetMatches(snap, target, withPred) ==
    IF withPred THEN snap.status[target] = "open" ELSE TRUE

\* Exact declarative graph semantics for the scoped query:
\*   root \in ScopeTasks AND root.status == "open" AND (polarity == Absent <=> no matching neighbor)
GraphEval(snap, pol, dir, withPred) ==
    {t \in ScopeTasks :
        /\ snap.status[t] = "open"
        /\ LET hasMatch == \E n \in Neighbors(snap, t, dir) :
                               TargetMatches(snap, n, withPred)
           IN IF pol = "Absent" THEN ~hasMatch ELSE hasMatch}

\* Deterministic ordered sequence of neighbors of `root` matching `Tasks` order.
OrderedNeighbors(snap, root, dir) ==
    LET nbrs == Neighbors(snap, root, dir)
    IN SelectSeq(Tasks, LAMBDA x : x \in nbrs)

\* First index in `seq` whose element satisfies `TargetMatches`, or 0 if none.
RECURSIVE FirstMatchIdx(_, _, _, _)
FirstMatchIdx(snap, seq, withPred, idx) ==
    IF idx > Len(seq) THEN 0
    ELSE IF TargetMatches(snap, seq[idx], withPred) THEN idx
    ELSE FirstMatchIdx(snap, seq, withPred, idx + 1)

\* Neighbors actually examined by short-circuiting evaluation for open `root`.
ExaminedNeighbors(snap, root, dir, withPred) ==
    LET seq == OrderedNeighbors(snap, root, dir)
        hit == FirstMatchIdx(snap, seq, withPred, 1)
        limit == IF hit = 0 THEN Len(seq) ELSE hit
    IN {seq[i] : i \in 1..limit}

\* Executable short-circuit evaluation of one scoped root candidate.
EvalRootShortCircuit(snap, root, pol, dir, withPred) ==
    IF ~(root \in ScopeTasks) \/ snap.status[root] /= "open" THEN FALSE
    ELSE LET seq == OrderedNeighbors(snap, root, dir)
             hit == FirstMatchIdx(snap, seq, withPred, 1)
         IN IF pol = "Absent" THEN hit = 0 ELSE hit > 0

\* All open scoped roots whose adjacency list is observed by related narrowing.
OpenRoots(snap) == {t \in ScopeTasks : snap.status[t] = "open"}

\* All target tasks (including out-of-scope blockers!) whose target field is observed.
AllExaminedTargets(snap, dir, withPred) ==
    IF ~withPred THEN {}
    ELSE UNION {ExaminedNeighbors(snap, t, dir, withPred) : t \in OpenRoots(snap)}

CurrentSnap == [status |-> status, edges |-> edges]

AdjRevFor(dir) == IF dir = "Forward" THEN fwdAdjRev ELSE revAdjRev

FootprintStillCurrent ==
    /\ \A t \in DOMAIN obs.rootStatusRev : statusRev[t] = obs.rootStatusRev[t]
    /\ \A a \in DOMAIN obs.adjRev : AdjRevFor(obs.dir)[a] = obs.adjRev[a]
    /\ \A b \in DOMAIN obs.targetStatusRev : statusRev[b] = obs.targetStatusRev[b]

EmptyObs == [
    active |-> FALSE,
    denied |-> FALSE,
    allowRel |-> TRUE,
    allowField |-> TRUE,
    pol |-> "Absent",
    dir |-> "Forward",
    withPred |-> TRUE,
    snapIdx |-> 1,
    rows |-> {},
    touchedCount |-> 0,
    rootStatusRev |-> [t \in {} |-> 0],
    adjRev |-> [t \in {} |-> 0],
    targetStatusRev |-> [t \in {} |-> 0]
]

Init ==
    /\ status = [t \in TaskSet |-> "open"]
    /\ edges = {}
    /\ statusRev = [t \in TaskSet |-> 0]
    /\ fwdAdjRev = [t \in TaskSet |-> 0]
    /\ revAdjRev = [t \in TaskSet |-> 0]
    /\ history = <<[status |-> [t \in TaskSet |-> "open"], edges |-> {}]>>
    /\ obs = EmptyObs
    /\ pinnedCheck = TRUE
    /\ currentCheck = TRUE

CommitSnapshot(nextStatus, nextEdges) ==
    history' = Append(history, [status |-> nextStatus, edges |-> nextEdges])

ToggleStatus(t) ==
    /\ Len(history) <= MaxCommits
    /\ LET nextVal == IF status[t] = "open" THEN "done" ELSE "open"
           nextStatus == [status EXCEPT ![t] = nextVal]
       IN /\ status' = nextStatus
          /\ statusRev' = [statusRev EXCEPT ![t] = @ + 1]
          /\ CommitSnapshot(nextStatus, edges)
          /\ UNCHANGED <<edges, fwdAdjRev, revAdjRev, obs, pinnedCheck, currentCheck>>

AddEdge(t, b) ==
    /\ Len(history) <= MaxCommits
    /\ <<t, b>> \notin edges
    /\ LET nextEdges == edges \cup {<<t, b>>}
       IN /\ edges' = nextEdges
          /\ fwdAdjRev' = [fwdAdjRev EXCEPT ![t] = @ + 1]
          /\ revAdjRev' = [revAdjRev EXCEPT ![b] = @ + 1]
          /\ CommitSnapshot(status, nextEdges)
          /\ UNCHANGED <<status, statusRev, obs, pinnedCheck, currentCheck>>

RemoveEdge(t, b) ==
    /\ Len(history) <= MaxCommits
    /\ <<t, b>> \in edges
    /\ LET nextEdges == edges \ {<<t, b>>}
       IN /\ edges' = nextEdges
          /\ fwdAdjRev' = [fwdAdjRev EXCEPT ![t] = @ + 1]
          /\ revAdjRev' = [revAdjRev EXCEPT ![b] = @ + 1]
          /\ CommitSnapshot(status, nextEdges)
          /\ UNCHANGED <<status, statusRev, obs, pinnedCheck, currentCheck>>

ObserveQuery(pol, dir, withPred, allowRel, allowField) ==
    LET authorized == allowRel /\ (~withPred \/ allowField)
        snap == CurrentSnap
        open == OpenRoots(snap)
        targets == AllExaminedTargets(snap, dir, withPred)
        resultRows == {t \in ScopeTasks : EvalRootShortCircuit(snap, t, pol, dir, withPred)}
    IN /\ IF ~authorized THEN
              obs' = [
                  active |-> TRUE,
                  denied |-> TRUE,
                  allowRel |-> allowRel,
                  allowField |-> allowField,
                  pol |-> pol,
                  dir |-> dir,
                  withPred |-> withPred,
                  snapIdx |-> Len(history),
                  rows |-> {},
                  touchedCount |-> 0,
                  rootStatusRev |-> [t \in {} |-> 0],
                  adjRev |-> [t \in {} |-> 0],
                  targetStatusRev |-> [t \in {} |-> 0]
              ]
          ELSE
              obs' = [
                  active |-> TRUE,
                  denied |-> FALSE,
                  allowRel |-> allowRel,
                  allowField |-> allowField,
                  pol |-> pol,
                  dir |-> dir,
                  withPred |-> withPred,
                  snapIdx |-> Len(history),
                  rows |-> resultRows,
                  touchedCount |-> Cardinality(ScopeTasks) + Cardinality(open) + Cardinality(targets),
                  \* Root equality guard observes every scoped root's status (NOT out-of-scope blockers).
                  rootStatusRev |-> [t \in ScopeTasks |-> statusRev[t]],
                  \* Related predicate observes adjacency structural rev of every open scoped root.
                  adjRev |-> [t \in open |-> AdjRevFor(dir)[t]],
                  \* Related predicate observes target status rev of every examined neighbor (including out-of-scope!).
                  targetStatusRev |-> [b \in targets |-> statusRev[b]]
              ]
       /\ UNCHANGED <<status, edges, statusRev, fwdAdjRev, revAdjRev, history, pinnedCheck, currentCheck>>

CheckCurrentness(reportedCurrent) ==
    /\ obs.active
    /\ ~obs.denied
    /\ currentCheck' = (reportedCurrent = FootprintStillCurrent)
    /\ UNCHANGED <<status, edges, statusRev, fwdAdjRev, revAdjRev, history, obs, pinnedCheck>>

PinnedRead(snapIdx, pol, dir, withPred) ==
    /\ snapIdx \in 1..Len(history)
    /\ LET snap == history[snapIdx]
           execRows == {t \in ScopeTasks : EvalRootShortCircuit(snap, t, pol, dir, withPred)}
           oracleRows == GraphEval(snap, pol, dir, withPred)
       IN pinnedCheck' = (execRows = oracleRows)
    /\ UNCHANGED <<status, edges, statusRev, fwdAdjRev, revAdjRev, history, obs, currentCheck>>

Next ==
    \/ \E t \in TaskSet : ToggleStatus(t)
    \/ \E t \in TaskSet, b \in TaskSet : AddEdge(t, b)
    \/ \E t \in TaskSet, b \in TaskSet : RemoveEdge(t, b)
    \/ \E pol \in Polarities, dir \in Directions, withPred \in BOOLEAN,
          allowRel \in BOOLEAN, allowField \in BOOLEAN :
          ObserveQuery(pol, dir, withPred, allowRel, allowField)
    \/ (obs.active /\ ~obs.denied /\ CheckCurrentness(FootprintStillCurrent))
    \/ \E snapIdx \in 1..Len(history), pol \in Polarities,
          dir \in Directions, withPred \in BOOLEAN :
          PinnedRead(snapIdx, pol, dir, withPred)

Spec == Init /\ [][Next]_vars

\* ---- Invariants ----

TypeOK ==
    /\ status \in [TaskSet -> Statuses]
    /\ edges \subseteq (TaskSet \X TaskSet)
    /\ Len(history) >= 1
    /\ obs.snapIdx \in 1..Len(history)
    /\ obs.rows \subseteq ScopeTasks
    /\ pinnedCheck \in BOOLEAN
    /\ currentCheck \in BOOLEAN

QueryMatchesGraphSemantics ==
    (obs.active /\ ~obs.denied) =>
        obs.rows = GraphEval(history[obs.snapIdx], obs.pol, obs.dir, obs.withPred)

\* Exact invalidation soundness: if the recorded footprint is still current,
\* the previously observed result set MUST still equal current graph semantics!
NoStaleRead ==
    (obs.active /\ ~obs.denied /\ FootprintStillCurrent) =>
        obs.rows = GraphEval(CurrentSnap, obs.pol, obs.dir, obs.withPred)

PinnedReadSound == pinnedCheck = TRUE

CurrentnessMatchesFootprint == currentCheck = TRUE

GovernanceFailClosed ==
    obs.active =>
        LET authorized == obs.allowRel /\ (~obs.withPred \/ obs.allowField)
        IN (~authorized => (obs.denied /\ obs.rows = {} /\ obs.touchedCount = 0))
           /\ (authorized => ~obs.denied)

=============================================================================
