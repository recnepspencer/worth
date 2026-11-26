--------------------- MODULE PersistentNamingCandidate ----------------------
EXTENDS PersistentNaming

(*
  Candidate-basis persistent-name reads (Phase 5E Slice 3C-1, #345).

  A coupled envelope whose earlier member (`first`) applied one mutation to
  the unpublished candidate, and whose later member (a coupled producer or a
  later authored stage) then reads a name in its decision. The candidate has
  no name index of its own: the read answers from the derived index at the
  commit basis, admitted by Query's rule
  (`worth-query-execution-lineage` `operation_reader::candidate_naming`),
  overlaid with the names the envelope's own naming plan gives the records
  its members created (`operation_reader::pending_naming`, #426):

  - `name_of` a record not live at the basis answers with the name the
    envelope gives it (`CandNames`: the candidate's own holders of its new
    generation), which its commit then records; one the envelope names
    nothing refuses `RefusedUnpublished`;
  - `name_of` a basis record the candidate ended refuses `RefusedEnded`;
  - `resolve` refuses `RefusedEnded` when any live record its basis answer
    rests on is not live at the same identity (generation) in the
    candidate. No such record is ever filtered out, so a sibling set never
    silently collapses;
  - `resolve` of an unborn name, or one whose basis walk crossed an ended
    name (a retired name or a split / merged parent), refuses
    `RefusedHistory`: the candidate may bear, regenerate or restore it;
  - otherwise the basis answer stands, with owners read on the candidate.

  A refused read vetoes the member's decision, so the whole envelope refuses
  with zero effects. An answered read lets the envelope commit `first`.
  `CandidateReadSound` states the safety claim: an answered read equals what
  the committed envelope then makes true. It never returns a stale or ended
  record, never answers a pending name other than the one the commit
  records, and never collapses an ambiguous sibling set after an
  unpublished retirement.

  A field edit (`Edit`) changes no lineage: it appends no event. Re-owning a
  record is not modelled (the owner lives in Native adjacency, not the
  lineage log); Query courts cover it.
*)

VARIABLE lastRead     \* Ghost: the last candidate read and its soundness.

cvars == <<vars, lastRead>>

NoRead == [status |-> "None", sound |-> TRUE]

Refusals == {"RefusedEnded", "RefusedHistory", "RefusedUnpublished"}
Creating == {"Create", "Preserve", "Restore", "Split"}
FirstKinds == {"Edit", "Create", "Preserve", "Restore", "Retire", "Split", "MergeKeep"}
ReadKinds == {"Resolve", "NameOf", "NameOfCreated"}

\* Whether a live record of generation `g` exists in holder map `state`.
RecordLiveIn(state, g) == \E m \in Names : state[m].alive /\ state[m].gen = g

\* The candidate holder map `first` leaves, before anything is published.
CandLive(kind, x, y, z, o1, o2, p) ==
    CASE kind = "Edit" -> live
      [] kind \in {"Create", "Preserve", "Restore"} ->
            [live EXCEPT ![x] = Holder(nextGen, o1, p)]
      [] kind = "Retire" -> [live EXCEPT ![x] = Ended(@)]
      [] kind = "Split" ->
            [live EXCEPT ![x] = Ended(@), ![y] = Holder(nextGen, o1, p),
                         ![z] = Holder(nextGen + 1, o2, p)]
      [] kind = "MergeKeep" -> [live EXCEPT ![y] = Ended(@)]

\* The guards of the commit `first` stands for.
FirstOK(kind, x, y, z, o1, o2, p) ==
    /\ kind \in FirstKinds
    /\ x \in Names /\ y \in Names /\ z \in Names
    /\ o1 \in HolderOwners /\ o2 \in HolderOwners /\ p \in Partitions
    /\ (kind /= "Edit") => Len(events) < MaxEvents
    /\ CASE kind \in {"Edit", "Preserve", "Retire"} -> live[x].alive
         [] kind = "Create" -> x \notin everBorn
         [] kind = "Restore" -> x \in retired
         [] kind = "Split" -> live[x].alive /\ y /= z /\ y \notin everBorn /\ z \notin everBorn
         [] kind = "MergeKeep" -> x /= y /\ live[x].alive /\ live[y].alive

\* The commit `first` stands for, once the later member's read answered.
CommitFirst(kind, x, y, z, o1, o2, p) ==
    CASE kind = "Edit" -> UNCHANGED vars
      [] kind = "Create" -> CommitCreate(x, o1, p)
      [] kind = "Preserve" -> CommitRegeneratePreserve(x, o1, p)
      [] kind = "Restore" -> CommitRestore(x, o1, p)
      [] kind = "Retire" -> CommitRetire(x)
      [] kind = "Split" -> CommitSplit(x, y, z, o1, o2, p)
      [] kind = "MergeKeep" -> CommitMergeKeepParentInPlace(x, y)

Refused(s) == [status |-> s, targets |-> {}]

\* The names a holder map gives the live records of generation `g`.
NamesOfGen(state, g) == {m \in Names : state[m].alive /\ state[m].gen = g}

\* Query's candidate-basis `resolve`: the basis answer, admitted only when
\* every live record it rests on is unchanged and no ended or unborn name
\* was walked.
ImplResolve(cand, n, scope, sel) ==
    IF n \notin everBorn THEN Refused("RefusedHistory")
    ELSE
        LET matched == Frontier(events, derivedIndex, n, Len(events))
        IN IF \E c \in matched : ~RecordLiveIn(cand, derivedIndex[c].gen)
              THEN Refused("RefusedEnded")
           ELSE IF ~derivedIndex[n].alive THEN Refused("RefusedHistory")
           ELSE AnswerIn(events, TRUE, cand, n, scope, sel)

\* Query's candidate-basis `name_of` of the basis record live name `n` holds.
ImplNameOf(cand, n) ==
    IF RecordLiveIn(cand, derivedIndex[n].gen)
        THEN [status |-> "Named", targets |-> {n}]
        ELSE Refused("RefusedEnded")

\* Query's `name_of` of the record `first` created (generation `nextGen`):
\* the name the envelope's own plan gives it on the candidate, before any
\* commit (#426). A created record it names nothing refuses.
ImplNameOfCreated(cand) ==
    IF NamesOfGen(cand, nextGen) = {} THEN Refused("RefusedUnpublished")
    ELSE [status |-> "Named", targets |-> NamesOfGen(cand, nextGen)]

ImplRead(cand, rk, rn, scope, sel) ==
    CASE rk = "Resolve" -> ImplResolve(cand, rn, scope, sel)
      [] rk = "NameOf" -> ImplNameOf(cand, rn)
      [] rk = "NameOfCreated" -> ImplNameOfCreated(cand)

\* What the committed envelope makes true of the same read (primed state).

TruthAfter(rk, rn, scope, sel, basisGen, createdGen) ==
    CASE rk = "Resolve" -> AnswerIn(events', rn \in everBorn', live', rn, scope, sel)
      [] rk = "NameOf" -> [status |-> "Named", targets |-> NamesOfGen(live', basisGen)]
      [] rk = "NameOfCreated" -> [status |-> "Named", targets |-> NamesOfGen(live', createdGen)]

ReadOK(kind, rk, rn, scope, sel) ==
    /\ rk \in ReadKinds /\ rn \in Names /\ scope \in Scopes /\ sel \in Selections
    /\ (rk = "NameOf") => derivedIndex[rn].alive
    /\ (rk = "NameOfCreated") => kind \in Creating

\* One coupled envelope: `first` stages a mutation, a later member reads.
CandidateEnvelope(kind, x, y, z, o1, o2, p, rk, rn, scope, sel) ==
    /\ FirstOK(kind, x, y, z, o1, o2, p)
    /\ ReadOK(kind, rk, rn, scope, sel)
    /\ LET answer == ImplRead(CandLive(kind, x, y, z, o1, o2, p), rk, rn, scope, sel)
       IN IF answer.status \in Refusals
             THEN /\ UNCHANGED vars
                  /\ lastRead' = [status |-> answer.status, sound |-> TRUE]
             ELSE /\ CommitFirst(kind, x, y, z, o1, o2, p)
                  /\ lastRead' = [status |-> answer.status,
                                  sound |-> answer = TruthAfter(rk, rn, scope, sel,
                                                derivedIndex[rn].gen, nextGen)]

\* Every candidate envelope over bounded arguments: only the arguments a
\* kind uses vary.
Default == CHOOSE n \in Names : TRUE
DefaultPart == CHOOSE p \in Partitions : TRUE

CandidateStep ==
    \E rk \in ReadKinds, rn \in Names, scope \in Scopes, sel \in Selections:
        LET Env(kind, x, y, z, o1, o2, p) ==
                CandidateEnvelope(kind, x, y, z, o1, o2, p, rk, rn, scope, sel)
        IN
        \/ \E x \in Names: Env("Edit", x, x, x, Unowned, Unowned, DefaultPart)
        \/ \E x \in Names: Env("Retire", x, x, x, Unowned, Unowned, DefaultPart)
        \/ \E x, y \in Names: Env("MergeKeep", x, y, y, Unowned, Unowned, DefaultPart)
        \/ \E k \in {"Create", "Preserve", "Restore"}, x \in Names,
              o \in HolderOwners, p \in Partitions: Env(k, x, x, x, o, o, p)
        \/ \E x, y, z \in Names, o1, o2 \in HolderOwners, p \in Partitions:
              Env("Split", x, y, z, o1, o2, p)

CInit == Init /\ lastRead = NoRead

CNext == (Next /\ UNCHANGED lastRead) \/ CandidateStep

CSpec == CInit /\ [][CNext]_cvars

\* An answered candidate read is exactly what the committed envelope makes
\* true: no stale or ended record, no fabricated unpublished name, no
\* collapsed sibling set.
CandidateReadSound == lastRead.sound

\* A candidate read never answers from a name's ended or unborn state: the
\* candidate may already have restored, regenerated or borne it.
NoStaleHistoryAnswer == lastRead.status \notin {"Retired", "UnknownName"}

=============================================================================
