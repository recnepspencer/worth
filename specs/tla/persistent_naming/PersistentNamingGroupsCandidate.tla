------------------ MODULE PersistentNamingGroupsCandidate --------------------
EXTENDS PersistentNamingGroups

(*
  Candidate-basis group reads (Phase 5E follow-up, #361).

  A coupled envelope whose earlier member (`first`) applied one group
  action to the unpublished candidate, and whose later member (a coupled
  producer or a later authored stage) then reads groups in its decision.
  The candidate has no group state of its own: commit finalization names a
  scheme's groups against the basis groups from the commit's partition
  claims and from every group losing a member the commit ends. The read
  answers from the derived index at the commit basis, admitted by Query's
  rule (`worth-query-execution-lineage` `operation_reader::group_naming`):

  - any read of the scheme refuses `RefusedClaim` (`CandidateGroupState`)
    when `first` claimed a partition, revival, split, merge or dissolve;
  - `group_name_of` a member whose name ended at the basis, and
    `resolve_group` of a held group that is unborn or not itself live
    (retired, split, merged), refuses `RefusedHistory`
    (`CandidateNameHistory`): the candidate may revive or regroup it;
  - a read refuses `RefusedEnded` (`EndedInCandidate`) when the candidate
    ended the member read or any member of a group the answer rests on.
    No member is filtered out, so no answer is ever stale membership;
  - otherwise the basis answer stands.

  A refused read vetoes the member's decision, so the whole envelope
  refuses with zero effects; an answered read lets the envelope commit
  `first`. `CandidateGroupReadSound`: an answered read is exactly what the
  committed envelope then makes true.

  A member claiming a scheme after a candidate read of it refuses the whole
  envelope (`LaterClaim`), so no committed envelope regroups what a
  decision read on its candidate.

  A field edit (`Edit`) changes no group and appends no event.
*)

VARIABLE lastRead     \* Ghost: every answered candidate group read so far was sound.

cvars == <<vars, lastRead>>

Refusals == {"RefusedClaim", "RefusedEnded", "RefusedHistory"}
Claims == {"Born", "Revive", "Split", "SplitAway", "Merge", "Dissolve"}
FirstKinds == Claims \cup {"Edit", "DeleteFree", "DeleteMember", "DeleteAnchor"}

\* The member liveness `first` leaves on the candidate, before publication.
CandMemberLive(kind, g, m) ==
    CASE kind \in {"DeleteFree", "DeleteMember"} -> [memberLive EXCEPT ![m] = FALSE]
      [] kind = "DeleteAnchor" -> [memberLive EXCEPT ![groups[g].anchor] = FALSE]
      [] OTHER -> memberLive

\* The commit an unclaimed `first` stands for. `Edit` publishes nothing; a
\* claim never commits under a candidate read (every read of its scheme
\* refuses), so its arguments never matter here.
CommitFirst(kind, g, m, n, a) ==
    CASE kind = "Edit" -> UNCHANGED vars
      [] kind = "DeleteFree" -> CommitDeleteFree(m)
      [] kind = "DeleteMember" -> CommitDeleteMember(g, m)
      [] kind = "DeleteAnchor" -> CommitDeleteAnchor(g, n, a)
      [] OTHER -> FALSE

Refused(s) == [status |-> s, targets |-> {}]

\* The live group holding `m` in group state `gs`.
HomeIn(gs, m) == {h \in GroupNames : gs[h].state = "live" /\ m \in gs[h].members}

\* Query's candidate-basis `group_name_of`: the member and every member of
\* the group holding it must be live at the basis and in the candidate.
ImplMemberOf(cand, m) ==
    IF ~memberLive[m] THEN Refused("RefusedHistory")
    ELSE LET home == HomeIn(groups, m)
             rests == {m} \cup UNION {groups[h].members : h \in home}
         IN IF \E r \in rests : ~cand[r] THEN Refused("RefusedEnded")
            ELSE [status |-> "member", targets |-> home]

\* Query's candidate-basis `resolve_group`: only a held group that is itself
\* the one live group answers, while the candidate leaves every member live.
ImplResolve(cand, g, sel) ==
    IF ~idx[g].born \/ ~idx[g].alive THEN Refused("RefusedHistory")
    ELSE IF \E r \in groups[g].members : ~cand[r] THEN Refused("RefusedEnded")
    ELSE ResolveFromIndex(g, sel)

ImplRead(kind, cand, rk, rm, rg, sel) ==
    IF kind \in Claims THEN Refused("RefusedClaim")
    ELSE IF rk = "MemberOf" THEN ImplMemberOf(cand, rm)
    ELSE ImplResolve(cand, rg, sel)

\* What the committed envelope makes true of the same read.
TruthAfter(rk, rm, rg, sel) ==
    IF rk = "MemberOf"
        THEN [status |-> "member", targets |-> HomeIn(groups', rm)]
        ELSE ResolveFromIndex(rg, sel)'

\* An answered read that equals the committed truth and rests on no history.
Sound(answer, rk, rm, rg, sel) ==
    /\ answer.status \notin {"retired", "unknown", "ambiguous", "plural"}
    /\ answer = TruthAfter(rk, rm, rg, sel)

\* One coupled envelope: `first` stages a group action, a later member reads.
CandidateEnvelope(kind, g, m, n, a, r) ==
    /\ kind \in FirstKinds
    /\ kind \in Claims \/ ENABLED CommitFirst(kind, g, m, n, a)
    /\ LET answer == ImplRead(kind, CandMemberLive(kind, g, m), r.rk, r.m, r.g, r.sel)
       IN IF answer.status \in Refusals
             THEN UNCHANGED cvars
             ELSE /\ CommitFirst(kind, g, m, n, a)
                  /\ lastRead' = (lastRead /\ Sound(answer, r.rk, r.m, r.g, r.sel))

\* A member that claims a scheme a decision already read on the candidate
\* refuses the envelope: nothing commits (a stutter of the model).
LaterClaim(kind) ==
    /\ kind \in Claims
    /\ UNCHANGED cvars

Reads ==
    [rk : {"MemberOf"}, m : Members, g : {None}, sel : {None}]
        \cup [rk : {"Resolve"}, m : {None}, g : GroupNames, sel : {"exact", "all"}]

CandidateStep ==
    \E r \in Reads :
        LET Env(kind, g, m, n, a) == CandidateEnvelope(kind, g, m, n, a, r)
            G == CHOOSE x \in GroupNames : TRUE
            M == CHOOSE x \in Members : TRUE
        IN
        \/ \E kind \in Claims \cup {"Edit"} : Env(kind, G, M, None, None)
        \/ \E m \in Members : Env("DeleteFree", G, m, None, None)
        \/ \E g \in GroupNames, m \in Members : Env("DeleteMember", g, m, None, None)
        \/ \E g \in GroupNames, n \in GroupNames \cup {None}, a \in Members \cup {None} :
              Env("DeleteAnchor", g, M, n, a)

CInit == Init /\ lastRead = TRUE

CNext ==
    \/ Next /\ UNCHANGED lastRead
    \/ CandidateStep
    \/ \E kind \in Claims : LaterClaim(kind)

CSpec == CInit /\ [][CNext]_cvars

\* Every answered candidate group read was exactly what the committed
\* envelope then made true, and none answered from group history: no stale
\* membership, no group of an ended member, no retired or split group.
CandidateGroupReadSound == lastRead

=============================================================================
