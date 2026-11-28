----------------------- MODULE PersistentNamingGroups -----------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

(*
  Runtime group persistent names under `Anchor` continuity (Phase 5E,
  Slice 3C-2, #334).

  Ground truth is the group state Relational's group naming authority keeps
  (`groups`, `succ`): which members each live group holds, which member
  anchors it, which names are retired, and where an ended group's meaning
  went. The derived name index (`idx`) is built solely from the canonical
  group lineage events each commit publishes (`events`), exactly as the
  runtime ingests them: births and revivals, then retirements, then forward
  edges, all stamped with their commit.

  Safety:
    - GroupsWellFormed: live groups are nonempty, disjoint, and hold their
      anchor; no retired name is live.
    - DerivedIndexEquivalence: the incrementally maintained index equals a
      fresh rebuild from the event log alone (destroy + rebuild is a no-op).
    - ResolutionMatchesEventOracle: resolving every ever-born group name
      from the index agrees with ground truth.
    - NoSilentRebindOnAmbiguity: an `Exact` resolution of a name whose
      meaning reached two or more live groups refuses `Ambiguous`.
*)

\* Members are canonical keys. Model checking uses one entity and both
\* projected cases of that same carrier; trace refinement supplies its own
\* representationally opaque member keys.
CONSTANTS Members, GroupNames, MaxCommits

CanonicalMemberFixture ==
    {"Entity:face", "ProjectedCase:face:FaceSide:Negative",
     "ProjectedCase:face:FaceSide:Positive"}

None == "none"
Status == {"unborn", "live", "retired"}

VARIABLES
    memberLive, \* [Members -> BOOLEAN]
    groups,     \* [GroupNames -> [state, members, anchor, sources]]
    succ,       \* ground truth: [GroupNames -> SUBSET GroupNames] at its end
    events,     \* canonical group lineage events, in commit order
    idx,        \* derived index rebuilt from `events`
    clock       \* commits so far

vars == <<memberLive, groups, succ, events, idx, clock>>

EmptyEntry == [born |-> FALSE, alive |-> FALSE, endedAt |-> 0, edges |-> {}]

\* --- Derived index: built from events alone ---------------------------------

ApplyEvent(ix, e) ==
    CASE e.kind = "Born" ->
            [ix EXCEPT ![e.g] = [@ EXCEPT !.born = TRUE, !.alive = TRUE]]
      [] e.kind = "Revived" -> [ix EXCEPT ![e.g].alive = TRUE]
      [] e.kind = "Retired" ->
            [ix EXCEPT ![e.g] = [@ EXCEPT !.alive = FALSE, !.endedAt = e.at]]
      [] OTHER -> [ix EXCEPT ![e.g].edges = @ \cup {<<e.at, e.to>>}]

RECURSIVE Fold(_, _)
Fold(seq, ix) == IF seq = <<>> THEN ix ELSE Fold(Tail(seq), ApplyEvent(ix, Head(seq)))

Rebuild(log) == Fold(log, [g \in GroupNames |-> EmptyEntry])

\* One commit's events in ingestion order: lifecycle starts, ends, edges.
Ordered(batch) ==
    LET starts == SelectSeq(batch, LAMBDA e : e.kind \in {"Born", "Revived"})
        ends == SelectSeq(batch, LAMBDA e : e.kind = "Retired")
        edges == SelectSeq(batch, LAMBDA e : e.kind \in {"SplitFrom", "MergedInto"})
    IN starts \o ends \o edges

Publish(batch) ==
    LET ordered == Ordered(batch)
    IN /\ events' = events \o ordered
       /\ idx' = Fold(ordered, idx)
       /\ clock' = clock + 1

Ev(kind, g, to) == [kind |-> kind, g |-> g, to |-> to, at |-> clock + 1]

\* --- Resolution -----------------------------------------------------------------

RECURSIVE IdxFrontier(_, _)
IdxFrontier(g, seen) ==
    IF g \in seen THEN {}
    ELSE IF idx[g].alive THEN {g}
    ELSE LET next == {e[2] : e \in {x \in idx[g].edges : x[1] = idx[g].endedAt}}
         IN UNION {IdxFrontier(s, seen \cup {g}) : s \in next}

RECURSIVE TruthFrontier(_, _)
TruthFrontier(g, seen) ==
    IF g \in seen THEN {}
    ELSE IF groups[g].state = "live" THEN {g}
    ELSE UNION {TruthFrontier(s, seen \cup {g}) : s \in succ[g]}

Answer(frontier, sel) ==
    IF frontier = {} THEN [status |-> "retired", targets |-> {}]
    ELSE IF Cardinality(frontier) = 1 THEN [status |-> "exact", targets |-> frontier]
    ELSE IF sel = "exact" THEN [status |-> "ambiguous", targets |-> frontier]
    ELSE [status |-> "plural", targets |-> frontier]

ResolveFromIndex(g, sel) ==
    IF ~idx[g].born THEN [status |-> "unknown", targets |-> {}]
    ELSE Answer(IdxFrontier(g, {}), sel)

ResolveFromTruth(g, sel) ==
    IF groups[g].state = "unborn" THEN [status |-> "unknown", targets |-> {}]
    ELSE Answer(TruthFrontier(g, {}), sel)

\* --- Ground-truth helpers -------------------------------------------------------

Live == {g \in GroupNames : groups[g].state = "live"}
Grouped == UNION {groups[g].members : g \in Live}
Free == {m \in Members : memberLive[m] /\ m \notin Grouped}
Unborn == {g \in GroupNames : groups[g].state = "unborn"}

\* A group name is minted from its birth anchor and the prior groups it was
\* split from: the one name ever held under `(a, srcs)`.
KeyHolder(a, srcs) ==
    {h \in GroupNames : groups[h].state # "unborn" /\ groups[h].anchor = a /\ groups[h].sources = srcs}

\* `n` is the name `(a, srcs)` mints: a fresh name, or the retired one it
\* revives (a live holder would be a collision the authority refuses).
Mints(n, a, srcs) ==
    IF KeyHolder(a, srcs) = {}
        THEN n \in Unborn
        ELSE KeyHolder(a, srcs) = {n} /\ groups[n].state = "retired"

Start(n) == IF groups[n].state = "retired" THEN Ev("Revived", n, None) ELSE Ev("Born", n, None)

LiveGroup(ms, a, srcs) == [state |-> "live", members |-> ms, anchor |-> a, sources |-> srcs]

\* --- Commits ----------------------------------------------------------------------

Init ==
    /\ memberLive = [m \in Members |-> TRUE]
    /\ groups = [g \in GroupNames |-> [state |-> "unborn", members |-> {}, anchor |-> None, sources |-> {}]]
    /\ succ = [g \in GroupNames |-> {}]
    /\ events = <<>>
    /\ idx = [g \in GroupNames |-> EmptyEntry]
    /\ clock = 0

Room == clock < MaxCommits

\* A partition declares a new group of free members around anchor `a`.
CommitBorn(g, ms, a) ==
    /\ Room /\ g \in Unborn /\ ms # {} /\ ms \subseteq Free /\ a \in ms
    /\ Mints(g, a, {})
    /\ groups' = [groups EXCEPT ![g] = LiveGroup(ms, a, {})]
    /\ Publish(<<Ev("Born", g, None)>>)
    /\ UNCHANGED <<memberLive, succ>>

\* Re-forming a dissolved group around its anchor revives its exact name.
CommitRevive(g, ms) ==
    /\ Room /\ groups[g].state = "retired" /\ groups[g].sources = {}
    /\ ms # {} /\ ms \subseteq Free /\ groups[g].anchor \in ms
    /\ groups' = [groups EXCEPT ![g] = LiveGroup(ms, groups[g].anchor, {})]
    /\ succ' = [succ EXCEPT ![g] = {}]
    /\ Publish(<<Ev("Revived", g, None)>>)
    /\ UNCHANGED memberLive

\* The piece without the anchor leaves as `g`'s split successor; the anchor
\* keeps `g`. A piece re-split with the same anchor revives its old name.
CommitSplit(g, piece, n, a) ==
    /\ Room /\ g \in Live /\ n # g /\ Mints(n, a, {g})
    /\ piece # {} /\ piece \subseteq groups[g].members
    /\ groups[g].anchor \notin piece /\ a \in piece
    /\ groups' = [groups EXCEPT ![g].members = @ \ piece, ![n] = LiveGroup(piece, a, {g})]
    /\ succ' = [succ EXCEPT ![n] = {}]
    /\ Publish(<<Start(n), Ev("SplitFrom", g, n)>>)
    /\ UNCHANGED memberLive

\* The anchor is deleted and the rest re-partitioned in one commit: no
\* keeper exists, so the group ends with two successors.
CommitSplitAway(g, p1, p2, n1, n2, a1, a2) ==
    /\ Room /\ g \in Live /\ n1 # n2 /\ n1 # g /\ n2 # g
    /\ Mints(n1, a1, {g}) /\ Mints(n2, a2, {g})
    /\ p1 # {} /\ p2 # {} /\ p1 \cap p2 = {}
    /\ p1 \cup p2 = groups[g].members \ {groups[g].anchor}
    /\ a1 \in p1 /\ a2 \in p2
    /\ memberLive' = [memberLive EXCEPT ![groups[g].anchor] = FALSE]
    /\ groups' = [groups EXCEPT ![g] = [@ EXCEPT !.state = "retired", !.members = {}],
                                ![n1] = LiveGroup(p1, a1, {g}),
                                ![n2] = LiveGroup(p2, a2, {g})]
    /\ succ' = [succ EXCEPT ![g] = {n1, n2}, ![n1] = {}, ![n2] = {}]
    /\ Publish(<<Start(n1), Start(n2), Ev("Retired", g, None),
                 Ev("SplitFrom", g, n1), Ev("SplitFrom", g, n2)>>)

\* Two groups merge; `k` keeps its name and absorbs `x`.
CommitMerge(k, x) ==
    /\ Room /\ k \in Live /\ x \in Live /\ k # x
    /\ groups' = [groups EXCEPT ![k].members = @ \cup groups[x].members,
                                ![x] = [@ EXCEPT !.state = "retired", !.members = {}]]
    /\ succ' = [succ EXCEPT ![x] = {k}]
    /\ Publish(<<Ev("Retired", x, None), Ev("MergedInto", x, k)>>)
    /\ UNCHANGED memberLive

\* Dissolving a group retires its name; its members leave the kind.
CommitDissolve(g) ==
    /\ Room /\ g \in Live
    /\ groups' = [groups EXCEPT ![g] = [@ EXCEPT !.state = "retired", !.members = {}]]
    /\ succ' = [succ EXCEPT ![g] = {}]
    /\ Publish(<<Ev("Retired", g, None)>>)
    /\ UNCHANGED memberLive

\* Deleting an ungrouped member touches no group.
CommitDeleteFree(m) ==
    /\ Room /\ m \in Free
    /\ memberLive' = [memberLive EXCEPT ![m] = FALSE]
    /\ Publish(<<>>)
    /\ UNCHANGED <<groups, succ>>

\* Deleting a grouped non-anchor member shrinks its group, undeclared.
CommitDeleteMember(g, m) ==
    /\ Room /\ g \in Live /\ m \in groups[g].members /\ groups[g].anchor # m
    /\ memberLive' = [memberLive EXCEPT ![m] = FALSE]
    /\ groups' = [groups EXCEPT ![g].members = @ \ {m}]
    /\ Publish(<<>>)
    /\ UNCHANGED succ

\* Deleting a group's anchor leaves no keeper: the group retires, and its
\* remaining members, if any, are born as its one split successor.
CommitDeleteAnchor(g, n, a) ==
    /\ Room /\ g \in Live
    /\ LET m == groups[g].anchor
           rest == groups[g].members \ {m}
       IN /\ memberLive' = [memberLive EXCEPT ![m] = FALSE]
          /\ IF rest = {}
                THEN /\ n = None /\ a = None
                     /\ groups' = [groups EXCEPT ![g] = [@ EXCEPT !.state = "retired", !.members = {}]]
                     /\ succ' = [succ EXCEPT ![g] = {}]
                     /\ Publish(<<Ev("Retired", g, None)>>)
                ELSE /\ n # g /\ n \in GroupNames /\ a \in rest /\ Mints(n, a, {g})
                     /\ groups' = [groups EXCEPT ![g] = [@ EXCEPT !.state = "retired", !.members = {}],
                                                 ![n] = LiveGroup(rest, a, {g})]
                     /\ succ' = [succ EXCEPT ![g] = {n}, ![n] = {}]
                     /\ Publish(<<Start(n), Ev("Retired", g, None),
                                  Ev("SplitFrom", g, n)>>)

\* Typed refusal: declaring one member into two groups commits nothing.
RefuseOverlap(m) ==
    /\ m \in Free
    /\ UNCHANGED vars

\* The derived index is destroyed and rebuilt from canonical events.
DestroyAndRebuildIndex ==
    /\ idx' = Rebuild(events)
    /\ UNCHANGED <<memberLive, groups, succ, events, clock>>

Next ==
    \/ \E g \in GroupNames, ms \in SUBSET Members, a \in Members : CommitBorn(g, ms, a)
    \/ \E g \in GroupNames, ms \in SUBSET Members : CommitRevive(g, ms)
    \/ \E g, n \in GroupNames, piece \in SUBSET Members, a \in Members : CommitSplit(g, piece, n, a)
    \/ \E g, n1, n2 \in GroupNames, p1, p2 \in SUBSET Members, a1, a2 \in Members :
            CommitSplitAway(g, p1, p2, n1, n2, a1, a2)
    \/ \E k, x \in GroupNames : CommitMerge(k, x)
    \/ \E g \in GroupNames : CommitDissolve(g)
    \/ \E m \in Members : CommitDeleteFree(m) \/ RefuseOverlap(m)
    \/ \E g \in GroupNames, m \in Members : CommitDeleteMember(g, m)
    \/ \E g \in GroupNames, n \in GroupNames \cup {None}, a \in Members \cup {None} :
            CommitDeleteAnchor(g, n, a)
    \/ DestroyAndRebuildIndex

Spec == Init /\ [][Next]_vars

\* --- Invariants -------------------------------------------------------------------

TypeOK ==
    /\ memberLive \in [Members -> BOOLEAN]
    /\ \A g \in GroupNames : groups[g].state \in Status /\ groups[g].members \subseteq Members
    /\ clock \in 0..MaxCommits

GroupsWellFormed ==
    /\ \A g \in Live : groups[g].members # {} /\ groups[g].anchor \in groups[g].members
    /\ \A g, h \in Live : g # h => groups[g].members \cap groups[h].members = {}
    /\ \A g \in Live : \A m \in groups[g].members : memberLive[m]

DerivedIndexEquivalence == idx = Rebuild(events)

ResolutionMatchesEventOracle ==
    \A g \in GroupNames : \A sel \in {"exact", "all"} :
        ResolveFromIndex(g, sel) = ResolveFromTruth(g, sel)

NoSilentRebindOnAmbiguity ==
    \A g \in GroupNames :
        Cardinality(IdxFrontier(g, {})) > 1 /\ idx[g].born
            => ResolveFromIndex(g, "exact").status = "ambiguous"

=============================================================================
