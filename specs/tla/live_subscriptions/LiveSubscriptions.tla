------------------------- MODULE LiveSubscriptions -------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

(*
  Formal reference model for Daymark Plan Wave 2 — PR 2C (`P2.3` / `G9`):
  Whole-Result Live Subscriptions preserving exact negative (`where_filter`)
  and related-record (`where_no_related`) invalidation evidence, ordered
  projected row values `<<task, status, due>>`, strict per-delivery authority
  re-admission with exact refusal reason, unrelated-project isolation, and
  Directory Home close/reopen + resubscribe.

  Safety invariants proved exhaustively by TLC:
    1. TypeOK: all variables and projected row sequences remain well-typed.
    2. DeliveredMatchesGraphSemantics: every delivered snapshot equals exact
       ordered projected relational graph semantics at its observed commit.
    3. NeverStaleWhenCaughtUp: whenever an active subscription has drained all
       committed batches (`cursor > Len(history)`), its last delivered ordered
       projected rows MUST equal exact current graph semantics (proving neither
       negative filter changes nor related-record blocker/edge changes nor
       projected field value edits can ever be missed by footprint verification).
    4. NeverDeliverUnauthorized: every initial, delivered, or pending outcome
       carries an active authorization proof (`lastAuthGrant = TRUE`).
    5. UnrelatedProjectIsolation: every projected row tuple belongs to `p1` and
       matches the exact status and due values of its observed commit.

  Liveness property proved exhaustively by TLC under weak fairness on `Poll`:
    - EventuallyDeliversCurrent: after any relevant mutation while subscribed
      with valid authority, polling eventually delivers the exact new ordered
      projected rows (unless authority is revoked or the subscription/home closes).
*)

CONSTANTS
    Tasks,          \* Ordered sequence of task IDs
    TaskProject,    \* [TaskSet -> {"p1", "p2"}]
    MaxCommits,     \* Bound on state mutations for finite TLC check
    MaxReopens      \* Bound on Directory Home reopens for finite TLC check

ModelTasks == <<"t1", "t2", "t3">>
ModelProject == ("t1" :> "p1") @@ ("t2" :> "p1") @@ ("t3" :> "p2")

TraceTasks == <<"t1", "t2", "t3", "t4", "t5", "t6">>
TraceProject ==
    ("t1" :> "p1") @@ ("t2" :> "p1") @@ ("t3" :> "p1")
    @@ ("t4" :> "p1") @@ ("t5" :> "p2") @@ ("t6" :> "p2")

TaskSet == {Tasks[i] : i \in 1..Len(Tasks)}
P1Tasks == {t \in TaskSet : TaskProject[t] = "p1"}
Statuses == {"open", "done"}
\* 0 = absent (None), 1 = due today (<= cutoff), 2 = future (> cutoff)
DueBuckets == {0, 1, 2}
Kinds == {"Today", "Ready"}

VARIABLES
    status,                 \* [TaskSet -> Statuses]
    due,                    \* [TaskSet -> DueBuckets]
    edges,                  \* SUBSET (TaskSet \X TaskSet): <<t, b>> means t BlockedBy b
    statusRev,              \* [TaskSet -> Nat]
    dueRev,                 \* [TaskSet -> Nat]
    fwdAdjRev,              \* [TaskSet -> Nat]
    grant,                  \* BOOLEAN: subscriber principal currently authorized
    homeEpoch,              \* Nat: Directory Home incarnation
    history,                \* Seq of committed snapshots
    subs                    \* [Kinds -> SubRecord]

vars == <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history, subs>>

Neighbors(snap, root) == {b \in TaskSet : <<root, b>> \in snap.edges}

OrderedNeighbors(snap, root) ==
    LET nbrs == Neighbors(snap, root)
    IN SelectSeq(Tasks, LAMBDA x : x \in nbrs)

RECURSIVE FirstOpenIdx(_, _, _)
FirstOpenIdx(snap, seq, idx) ==
    IF idx > Len(seq) THEN 0
    ELSE IF snap.status[seq[idx]] = "open" THEN idx
    ELSE FirstOpenIdx(snap, seq, idx + 1)

\* Neighbors actually examined by short-circuiting NOT EXISTS (status == "open").
ExaminedNeighbors(snap, root) ==
    LET seq == OrderedNeighbors(snap, root)
        hit == FirstOpenIdx(snap, seq, 1)
        limit == IF hit = 0 THEN Len(seq) ELSE hit
    IN {seq[i] : i \in 1..limit}

TodayKeys(snap) ==
    SelectSeq(Tasks, LAMBDA t : t \in P1Tasks /\ snap.status[t] = "open" /\ snap.due[t] = 1)

ReadyKeys(snap) ==
    SelectSeq(Tasks, LAMBDA t :
        /\ t \in P1Tasks
        /\ snap.status[t] = "open"
        /\ ~\E b \in Neighbors(snap, t) : snap.status[b] = "open")

ProjectRows(snap, keys) ==
    [i \in 1..Len(keys) |-> <<keys[i], snap.status[keys[i]], snap.due[keys[i]]>>]

Eval(kind, snap) ==
    ProjectRows(snap, IF kind = "Today" THEN TodayKeys(snap) ELSE ReadyKeys(snap))

OpenP1(snap) == {t \in P1Tasks : snap.status[t] = "open"}

ReadySet(snap) ==
    LET seq == ReadyKeys(snap)
    IN {seq[i] : i \in 1..Len(seq)}

ExaminedReadyTargets(snap) ==
    UNION {ExaminedNeighbors(snap, t) : t \in OpenP1(snap)}

\* Exact observed footprint captured by a whole-result read of `kind` at `snap`:
\*   - Today: every p1 task's status (index/equality guard) + every open p1
\*     task's due date (where_filter negative & positive evidence + projection).
\*   - Ready: every p1 task's status + every open p1 task's BlockedBy outgoing
\*     adjacency revision + every examined blocker target's status revision +
\*     every returned Ready row's projected due date revision.
CaptureFootprint(kind, snap) ==
    IF kind = "Today" THEN
        [
            statusDom |-> P1Tasks,
            statusRev |-> [t \in P1Tasks |-> snap.statusRev[t]],
            dueDom    |-> OpenP1(snap),
            dueRev    |-> [t \in OpenP1(snap) |-> snap.dueRev[t]],
            adjDom    |-> {},
            adjRev    |-> [t \in {} |-> 0]
        ]
    ELSE
        LET open == OpenP1(snap)
            targets == ExaminedReadyTargets(snap)
            sDom == P1Tasks \cup targets
            dDom == ReadySet(snap)
        IN [
            statusDom |-> sDom,
            statusRev |-> [t \in sDom |-> snap.statusRev[t]],
            dueDom    |-> dDom,
            dueRev    |-> [t \in dDom |-> snap.dueRev[t]],
            adjDom    |-> open,
            adjRev    |-> [t \in open |-> snap.fwdAdjRev[t]]
        ]

FootprintCurrentAt(fp, snap) ==
    /\ \A t \in fp.statusDom : snap.statusRev[t] = fp.statusRev[t]
    /\ \A t \in fp.dueDom    : snap.dueRev[t] = fp.dueRev[t]
    /\ \A t \in fp.adjDom    : snap.fwdAdjRev[t] = fp.adjRev[t]

\* Earliest commit index in `idx..Len(history)` that invalidates `fp`, or 0 if none.
RECURSIVE FirstInvalidated(_, _)
FirstInvalidated(fp, idx) ==
    IF idx > Len(history) THEN 0
    ELSE IF ~FootprintCurrentAt(fp, history[idx]) THEN idx
    ELSE FirstInvalidated(fp, idx + 1)

CurrentSnap == [
    status    |-> status,
    due       |-> due,
    edges     |-> edges,
    statusRev |-> statusRev,
    dueRev    |-> dueRev,
    fwdAdjRev |-> fwdAdjRev
]

EmptyFp == [
    statusDom |-> {},
    statusRev |-> [t \in {} |-> 0],
    dueDom    |-> {},
    dueRev    |-> [t \in {} |-> 0],
    adjDom    |-> {},
    adjRev    |-> [t \in {} |-> 0]
]

ClosedSub == [
    active           |-> FALSE,
    openedEpoch      |-> 0,
    cursor           |-> 1,
    lastSnapIdx      |-> 0,
    lastRows         |-> <<>>,
    lastOutcome      |-> "Closed",
    lastReason       |-> "Ok",
    lastAuthGrant    |-> FALSE,
    lastCloseOutcome |-> "None",
    fp               |-> EmptyFp
]

InitSnap == [
    status    |-> [t \in TaskSet |-> "open"],
    due       |-> [t \in TaskSet |-> 2],
    edges     |-> {},
    statusRev |-> [t \in TaskSet |-> 0],
    dueRev    |-> [t \in TaskSet |-> 0],
    fwdAdjRev |-> [t \in TaskSet |-> 0]
]

Init ==
    /\ status = InitSnap.status
    /\ due = InitSnap.due
    /\ edges = InitSnap.edges
    /\ statusRev = InitSnap.statusRev
    /\ dueRev = InitSnap.dueRev
    /\ fwdAdjRev = InitSnap.fwdAdjRev
    /\ grant = TRUE
    /\ homeEpoch = 0
    /\ history = <<InitSnap>>
    /\ subs = [k \in Kinds |-> ClosedSub]

CommitSnap(nextStatus, nextDue, nextEdges, nextSRev, nextDRev, nextARev) ==
    history' = Append(history, [
        status    |-> nextStatus,
        due       |-> nextDue,
        edges     |-> nextEdges,
        statusRev |-> nextSRev,
        dueRev    |-> nextDRev,
        fwdAdjRev |-> nextARev
    ])

\* ---- Mutations ----

ToggleStatus(t) ==
    /\ Len(history) <= MaxCommits
    /\ LET nextVal == IF status[t] = "open" THEN "done" ELSE "open"
           nextStatus == [status EXCEPT ![t] = nextVal]
           nextSRev == [statusRev EXCEPT ![t] = @ + 1]
       IN /\ status' = nextStatus
          /\ statusRev' = nextSRev
          /\ CommitSnap(nextStatus, due, edges, nextSRev, dueRev, fwdAdjRev)
          /\ UNCHANGED <<due, edges, dueRev, fwdAdjRev, grant, homeEpoch, subs>>

SetDue(t, bucket) ==
    /\ Len(history) <= MaxCommits
    /\ bucket \in DueBuckets
    /\ due[t] /= bucket
    /\ LET nextDue == [due EXCEPT ![t] = bucket]
           nextDRev == [dueRev EXCEPT ![t] = @ + 1]
       IN /\ due' = nextDue
          /\ dueRev' = nextDRev
          /\ CommitSnap(status, nextDue, edges, statusRev, nextDRev, fwdAdjRev)
          /\ UNCHANGED <<status, edges, statusRev, fwdAdjRev, grant, homeEpoch, subs>>

AddEdge(t, b) ==
    /\ Len(history) <= MaxCommits
    /\ <<t, b>> \notin edges
    /\ LET nextEdges == edges \cup {<<t, b>>}
           nextARev == [fwdAdjRev EXCEPT ![t] = @ + 1]
       IN /\ edges' = nextEdges
          /\ fwdAdjRev' = nextARev
          /\ CommitSnap(status, due, nextEdges, statusRev, dueRev, nextARev)
          /\ UNCHANGED <<status, due, statusRev, dueRev, grant, homeEpoch, subs>>

RemoveEdge(t, b) ==
    /\ Len(history) <= MaxCommits
    /\ <<t, b>> \in edges
    /\ LET nextEdges == edges \ {<<t, b>>}
           nextARev == [fwdAdjRev EXCEPT ![t] = @ + 1]
       IN /\ edges' = nextEdges
          /\ fwdAdjRev' = nextARev
          /\ CommitSnap(status, due, nextEdges, statusRev, dueRev, nextARev)
          /\ UNCHANGED <<status, due, statusRev, dueRev, grant, homeEpoch, subs>>

\* ---- Authority Grant / Revoke ----

RevokeGrant ==
    /\ grant = TRUE
    /\ grant' = FALSE
    /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, homeEpoch, history, subs>>

RestoreGrant ==
    /\ grant = FALSE
    /\ grant' = TRUE
    /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, homeEpoch, history, subs>>

\* ---- Directory Home Close / Reopen ----

ReopenHome ==
    /\ homeEpoch < MaxReopens
    /\ homeEpoch' = homeEpoch + 1
    /\ subs' = [k \in Kinds |-> ClosedSub]
    /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, history>>

\* ---- Whole-Result Live Subscription Lifecycle ----

Subscribe(kind) ==
    /\ ~subs[kind].active
    /\ IF ~grant THEN
           /\ subs' = [subs EXCEPT ![kind] = [
                  active           |-> FALSE,
                  openedEpoch      |-> homeEpoch,
                  cursor           |-> Len(history) + 1,
                  lastSnapIdx      |-> 0,
                  lastRows         |-> <<>>,
                  lastOutcome      |-> "AuthDenied",
                  lastReason       |-> "GrantRevoked",
                  lastAuthGrant    |-> FALSE,
                  lastCloseOutcome |-> "None",
                  fp               |-> EmptyFp
              ]]
           /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>
       ELSE
           LET snapIdx == Len(history)
               snap == history[snapIdx]
           IN /\ subs' = [subs EXCEPT ![kind] = [
                     active           |-> TRUE,
                     openedEpoch      |-> homeEpoch,
                     cursor           |-> snapIdx + 1,
                     lastSnapIdx      |-> snapIdx,
                     lastRows         |-> Eval(kind, snap),
                     lastOutcome      |-> "Initial",
                     lastReason       |-> "Ok",
                     lastAuthGrant    |-> TRUE,
                     lastCloseOutcome |-> "None",
                     fp               |-> CaptureFootprint(kind, snap)
                 ]]
              /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>

Poll(kind) ==
    /\ subs[kind].active
    /\ subs[kind].openedEpoch = homeEpoch
    /\ IF ~grant THEN
           \* Strict re-admission on every poll: fail closed immediately and retire custody!
           /\ subs' = [subs EXCEPT ![kind] = [
                  @ EXCEPT !.active           = FALSE,
                           !.lastOutcome      = "AuthDenied",
                           !.lastReason       = "GrantRevoked",
                           !.lastAuthGrant    = FALSE,
                           !.lastCloseOutcome = "None",
                           !.fp               = EmptyFp
              ]]
           /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>
       ELSE
           LET sub == subs[kind]
               hit == FirstInvalidated(sub.fp, sub.cursor)
           IN IF hit = 0 THEN
                  \* All unseen commits kept the observed footprint current -> Pending (no wake!)
                  /\ subs' = [subs EXCEPT ![kind] = [
                         @ EXCEPT !.cursor           = Len(history) + 1,
                                  !.lastOutcome      = "Pending",
                                  !.lastReason       = "Ok",
                                  !.lastAuthGrant    = TRUE,
                                  !.lastCloseOutcome = "None"
                     ]]
                  /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>
              ELSE
                  LET snap == history[hit]
                  IN /\ subs' = [subs EXCEPT ![kind] = [
                            @ EXCEPT !.cursor           = hit + 1,
                                     !.lastSnapIdx      = hit,
                                     !.lastRows         = Eval(kind, snap),
                                     !.lastOutcome      = "Delivered",
                                     !.lastReason       = "Ok",
                                     !.lastAuthGrant    = TRUE,
                                     !.lastCloseOutcome = "None",
                                     !.fp               = CaptureFootprint(kind, snap)
                        ]]
                     /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>

CloseSub(kind) ==
    /\ subs[kind].active
    /\ subs' = [subs EXCEPT ![kind] = [ClosedSub EXCEPT !.lastCloseOutcome = "Completed"]]
    /\ UNCHANGED <<status, due, edges, statusRev, dueRev, fwdAdjRev, grant, homeEpoch, history>>

Next ==
    \/ \E t \in TaskSet : ToggleStatus(t)
    \/ \E t \in TaskSet, b \in DueBuckets : SetDue(t, b)
    \/ \E t \in TaskSet, b \in TaskSet : AddEdge(t, b)
    \/ \E t \in TaskSet, b \in TaskSet : RemoveEdge(t, b)
    \/ RevokeGrant
    \/ RestoreGrant
    \/ ReopenHome
    \/ \E k \in Kinds : Subscribe(k)
    \/ \E k \in Kinds : Poll(k)
    \/ \E k \in Kinds : CloseSub(k)

Fairness == \A k \in Kinds : WF_vars(Poll(k))

Spec == Init /\ [][Next]_vars /\ Fairness

\* ---- Safety Invariants ----

ValidProjectedRow(row) ==
    /\ Len(row) = 3
    /\ row[1] \in P1Tasks
    /\ row[2] \in Statuses
    /\ row[3] \in DueBuckets

TypeOK ==
    /\ status \in [TaskSet -> Statuses]
    /\ due \in [TaskSet -> DueBuckets]
    /\ edges \subseteq (TaskSet \X TaskSet)
    /\ grant \in BOOLEAN
    /\ homeEpoch \in 0..MaxReopens
    /\ Len(history) >= 1
    /\ \A k \in Kinds :
        /\ subs[k].active \in BOOLEAN
        /\ subs[k].lastSnapIdx \in 0..Len(history)
        /\ subs[k].cursor \in 1..(Len(history) + 1)
        /\ subs[k].lastReason \in {"Ok", "GrantRevoked"}
        /\ subs[k].lastAuthGrant \in BOOLEAN
        /\ subs[k].lastCloseOutcome \in {"None", "Completed"}
        /\ \A i \in 1..Len(subs[k].lastRows) : ValidProjectedRow(subs[k].lastRows[i])

DeliveredMatchesGraphSemantics ==
    \A k \in Kinds :
        (subs[k].lastSnapIdx >= 1 /\ subs[k].lastOutcome \in {"Initial", "Delivered", "Pending"}) =>
            subs[k].lastRows = Eval(k, history[subs[k].lastSnapIdx])

\* Exact negative, related-record, & projected-value invalidation soundness:
\* If an active subscription has drained up to HEAD (`cursor > Len(history)`),
\* its last delivered ordered projected rows MUST match exact current semantics!
NeverStaleWhenCaughtUp ==
    \A k \in Kinds :
        (subs[k].active /\ subs[k].cursor > Len(history)) =>
            subs[k].lastRows = Eval(k, CurrentSnap)

\* No initial, delivered, or pending outcome may occur without valid authority.
NeverDeliverUnauthorized ==
    \A k \in Kinds :
        subs[k].lastOutcome \in {"Initial", "Delivered", "Pending"} =>
            subs[k].lastAuthGrant = TRUE

\* Value-level isolation: every projected row tuple belongs to p1 and matches
\* the exact projected status & due values of its observed commit.
UnrelatedProjectIsolation ==
    \A k \in Kinds :
        \A i \in 1..Len(subs[k].lastRows) :
            LET row == subs[k].lastRows[i]
            IN /\ row[1] \in P1Tasks
               /\ subs[k].lastSnapIdx >= 1
               /\ row[2] = history[subs[k].lastSnapIdx].status[row[1]]
               /\ row[3] = history[subs[k].lastSnapIdx].due[row[1]]

\* ---- Liveness Property ----

\* After any relevant mutation while subscribed with valid authority,
\* polling eventually delivers the exact current ordered projected rows
\* (unless closed or revoked).
EventuallyDeliversCurrent ==
    \A k \in Kinds :
        []((subs[k].active /\ grant /\ subs[k].lastRows /= Eval(k, CurrentSnap))
           => <>(~subs[k].active \/ ~grant \/ subs[k].lastRows = Eval(k, CurrentSnap)))

=============================================================================
