-------------------------- MODULE PersistentNaming --------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Persistent naming protocol of public WORTH (milestones/persistent-naming.md
  Phases 1-5, 5E): persistent names distinct from generational record
  identities, an append-only canonical lineage log (Create, Preserve, Split,
  MergeKeep, MergeMint, Retire, Restore), a derived name index rebuilt from
  that log, and held-name resolution exactly as Relational and Query answer it:

  - A live name answers itself. A name that is not live is followed forward
    only along the edges recorded by the event that last ended it, and every
    reached name repeats the rule (Relational `PersistentNameBranchIndex::resolve`).
  - Unscoped `Exact` over two or more live descendants refuses `Ambiguous`;
    `AllDescendants` answers `Plural`. Owner-scoped resolution (#133,
    Query `resolve_in`) filters the live descendants by owner and refuses
    `NoCandidateInOwner` when the owner holds none of them.
  - A `KeepParentInPlace` merge keeps the keeper's record untouched; its
    partition need not be touched by the merging commit (#314).
  - Re-minting a live name refuses typed (`DuplicateBirth`) with zero
    effects; re-minting a retired name restores it on a fresh generation.
  - Every installed scheme is complete (`require_all_births_named`): a
    commit creating a record of a schemed kind without naming it refuses
    typed (Relational `MissingBirth`) with zero effects, so no record of a
    schemed kind is ever live unnamed.
  - A coupled envelope (the authored stages of one edit envelope and the
    coupled producers it runs) is one commit: every member's naming plan is
    composed into the commit's one plan (Phase 5E Slice 3B-2), so all the
    members' births append atomically, and a name two members both mint, or
    a schemed record any member leaves unnamed, refuses the whole envelope
    with zero effects from every member.
*)

CONSTANTS
    Names,          \* e.g. {"n1", "n2", "n3"}
    Owners,         \* e.g. {"bodyA", "bodyB"}
    Partitions,     \* e.g. {"p1", "p2"}
    MaxEvents       \* bound on the canonical log length

ASSUME Cardinality(Names) >= 3
ASSUME Cardinality(Owners) >= 2
ASSUME Cardinality(Partitions) >= 2
ASSUME MaxEvents >= 2

Unowned == "none"
HolderOwners == Owners \cup {Unowned}
Scopes == Owners \cup {"all"}
Selections == {"Exact", "AllDescendants"}

EmptyHolder == [alive |-> FALSE, gen |-> 0, owner |-> Unowned, part |-> "none"]
Holder(g, o, p) == [alive |-> TRUE, gen |-> g, owner |-> o, part |-> p]
Ended(h) == [h EXCEPT !.alive = FALSE]
Max(S) == CHOOSE x \in S : \A y \in S : y <= x

VARIABLES
    nextGen,        \* Monotonic generational record counter
    live,           \* [Names -> Holder]: authoritative record state
    unnamedLive,    \* Unnamed live records of a completeness-guarded scheme
    events,         \* Append-only Seq of canonical lineage events
    derivedIndex,   \* [Names -> Holder]: the derived, rebuildable name index
    everBorn,       \* Names with a recorded birth (derived index membership)
    retired         \* Born names not live now

vars == <<nextGen, live, unnamedLive, events, derivedIndex, everBorn, retired>>

Init ==
    /\ nextGen = 1
    /\ live = [n \in Names |-> EmptyHolder]
    /\ unnamedLive = 0
    /\ events = <<>>
    /\ derivedIndex = [n \in Names |-> EmptyHolder]
    /\ everBorn = {}
    /\ retired = {}

\* ---------------------------------------------------------------------------
\* The canonical log is the only authority: replay rebuilds every holder.
\* ---------------------------------------------------------------------------
RECURSIVE ReplayEvents(_, _)
ReplayEvents(evSeq, idx) ==
    IF idx = 0 THEN [n \in Names |-> EmptyHolder]
    ELSE
        LET prev == ReplayEvents(evSeq, idx - 1)
            ev == evSeq[idx]
        IN
        CASE ev.kind \in {"Create", "Preserve", "Restore"} ->
                [prev EXCEPT ![ev.target] = Holder(ev.gen, ev.owner, ev.part)]
          [] ev.kind = "Split" ->
                [prev EXCEPT
                    ![ev.parent] = Ended(prev[ev.parent]),
                    ![ev.c1] = Holder(ev.g1, ev.o1, ev.part),
                    ![ev.c2] = Holder(ev.g2, ev.o2, ev.part)]
          [] ev.kind = "MergeKeep" ->
                [prev EXCEPT ![ev.other] = Ended(prev[ev.other])]
          [] ev.kind = "MergeMint" ->
                [prev EXCEPT
                    ![ev.p1] = Ended(prev[ev.p1]),
                    ![ev.p2] = Ended(prev[ev.p2]),
                    ![ev.minted] = Holder(ev.gen, ev.owner, ev.part)]
          [] ev.kind = "Retire" ->
                [prev EXCEPT ![ev.target] = Ended(prev[ev.target])]

\* Whether `ev` births `n` (a first birth, a split piece or a minted merge).
Births(ev, n) ==
    CASE ev.kind = "Create" -> ev.target = n
      [] ev.kind = "Split" -> n \in {ev.c1, ev.c2}
      [] ev.kind = "MergeMint" -> ev.minted = n
      [] OTHER -> FALSE

BornInLog(evSeq, n) == \E i \in 1..Len(evSeq) : Births(evSeq[i], n)

\* Whether `ev` ends the life of `n`.
Ends(ev, n) ==
    CASE ev.kind = "Split" -> ev.parent = n
      [] ev.kind = "MergeKeep" -> ev.other = n
      [] ev.kind = "MergeMint" -> n \in {ev.p1, ev.p2}
      [] ev.kind = "Retire" -> ev.target = n
      [] OTHER -> FALSE

\* The forward edges `ev` records from the name it ends.
Successors(ev) ==
    CASE ev.kind = "Split" -> {ev.c1, ev.c2}
      [] ev.kind = "MergeKeep" -> {ev.keeper}
      [] ev.kind = "MergeMint" -> {ev.minted}
      [] OTHER -> {}

LastEnd(evSeq, n) ==
    LET ends == {i \in 1..Len(evSeq) : Ends(evSeq[i], n)}
    IN IF ends = {} THEN 0 ELSE Max(ends)

\* The live names `n` designates under `state`: itself when live, else the
\* frontier of the edges recorded by the event that last ended it. Every
\* successor was live right after that event, so its own last end is later
\* and the walk terminates within Len(evSeq) hops.
RECURSIVE Frontier(_, _, _, _)
Frontier(evSeq, state, n, depth) ==
    IF state[n].alive THEN {n}
    ELSE IF depth = 0 THEN {}
    ELSE LET i == LastEnd(evSeq, n)
         IN IF i = 0 THEN {}
            ELSE UNION {Frontier(evSeq, state, s, depth - 1) : s \in Successors(evSeq[i])}

\* One resolution answer over log `evSeq`: unscoped (`scope = "all"`) or
\* within one owner.
AnswerIn(evSeq, born, state, n, scope, sel) ==
    IF ~born THEN [status |-> "UnknownName", targets |-> {}]
    ELSE
        LET matched == Frontier(evSeq, state, n, Len(evSeq))
            inScope == IF scope = "all" THEN matched
                       ELSE {c \in matched : state[c].owner = scope}
        IN
        IF matched = {} THEN [status |-> "Retired", targets |-> {}]
        ELSE IF inScope = {} THEN [status |-> "NoCandidateInOwner", targets |-> matched]
        ELSE IF Cardinality(inScope) = 1 THEN [status |-> "Exact", targets |-> inScope]
        ELSE IF sel = "Exact" THEN [status |-> "Ambiguous", targets |-> inScope]
        ELSE [status |-> "Plural", targets |-> inScope]

Answer(born, state, n, scope, sel) == AnswerIn(events, born, state, n, scope, sel)

\* Ground truth from the canonical log alone.
ResolveFromEvents(n, scope, sel) ==
    Answer(BornInLog(events, n), ReplayEvents(events, Len(events)), n, scope, sel)

\* The derived index's answer (must match the log's 100%).
ResolveFromIndex(n, scope, sel) ==
    Answer(n \in everBorn, derivedIndex, n, scope, sel)

\* ---------------------------------------------------------------------------
\* Actions: one canonical commit each, or a typed refusal with zero effects.
\* ---------------------------------------------------------------------------

\* 1. First birth of a name (the DuplicateBirth guard forbids live reuse).
CommitCreate(n, owner, part) ==
    /\ Len(events) < MaxEvents
    /\ n \in Names /\ owner \in HolderOwners /\ part \in Partitions
    /\ n \notin everBorn
    /\ LET h == Holder(nextGen, owner, part)
           ev == [kind |-> "Create", target |-> n, gen |-> nextGen, owner |-> owner, part |-> part]
       IN
       /\ nextGen' = nextGen + 1
       /\ live' = [live EXCEPT ![n] = h]
       /\ derivedIndex' = [derivedIndex EXCEPT ![n] = h]
       /\ events' = Append(events, ev)
       /\ everBorn' = everBorn \cup {n}
       /\ UNCHANGED <<unnamedLive, retired>>

\* 2. Re-minting a live name refuses typed with zero effects.
RefuseDuplicateBirth(n) ==
    /\ n \in Names /\ live[n].alive
    /\ UNCHANGED vars

\* 2b. A commit creating a record of a schemed kind (in `part`, held by
\*     `owner`) without naming it, beside any number of named births. The
\*     completeness guard refuses the whole commit typed (`MissingBirth`):
\*     no generation is issued, no event recorded, nothing goes live.
RefuseUnnamedBirth(owner, part) ==
    /\ owner \in HolderOwners /\ part \in Partitions
    /\ UNCHANGED vars

\* 3. In-commit regeneration: the holder is retired and the same name is
\*    re-minted onto a fresh record in one commit (Preserve).
CommitRegeneratePreserve(n, owner, part) ==
    /\ Len(events) < MaxEvents
    /\ n \in Names /\ owner \in HolderOwners /\ part \in Partitions
    /\ live[n].alive
    /\ LET h == Holder(nextGen, owner, part)
           ev == [kind |-> "Preserve", target |-> n, gen |-> nextGen, owner |-> owner, part |-> part]
       IN
       /\ nextGen' = nextGen + 1
       /\ live' = [live EXCEPT ![n] = h]
       /\ derivedIndex' = [derivedIndex EXCEPT ![n] = h]
       /\ events' = Append(events, ev)
       /\ UNCHANGED <<unnamedLive, everBorn, retired>>

\* 4. Split into two pieces created in one partition, possibly held by two
\*    distinct owners (#133 multi-body splits).
CommitSplit(parent, c1, c2, o1, o2, part) ==
    /\ Len(events) < MaxEvents
    /\ parent \in Names /\ c1 \in Names /\ c2 \in Names
    /\ o1 \in HolderOwners /\ o2 \in HolderOwners /\ part \in Partitions
    /\ live[parent].alive
    /\ c1 /= c2 /\ c1 \notin everBorn /\ c2 \notin everBorn
    /\ LET h1 == Holder(nextGen, o1, part)
           h2 == Holder(nextGen + 1, o2, part)
           ev == [kind |-> "Split", parent |-> parent, c1 |-> c1, c2 |-> c2,
                  g1 |-> nextGen, g2 |-> nextGen + 1, o1 |-> o1, o2 |-> o2, part |-> part]
       IN
       /\ nextGen' = nextGen + 2
       /\ live' = [live EXCEPT ![parent] = Ended(@), ![c1] = h1, ![c2] = h2]
       /\ derivedIndex' = [derivedIndex EXCEPT ![parent] = Ended(@), ![c1] = h1, ![c2] = h2]
       /\ events' = Append(events, ev)
       /\ everBorn' = everBorn \cup {c1, c2}
       /\ retired' = retired \cup {parent}
       /\ UNCHANGED unnamedLive

\* 5. Merge keeping `keeper`'s record in place: only `other` ends. The
\*    keeper's partition need not be touched by the commit (#314).
CommitMergeKeepParentInPlace(keeper, other) ==
    /\ Len(events) < MaxEvents
    /\ keeper \in Names /\ other \in Names /\ keeper /= other
    /\ live[keeper].alive /\ live[other].alive
    /\ LET ev == [kind |-> "MergeKeep", keeper |-> keeper, other |-> other,
                  untouchedKeeperPart |-> (live[keeper].part /= live[other].part)]
       IN
       /\ live' = [live EXCEPT ![other] = Ended(@)]
       /\ derivedIndex' = [derivedIndex EXCEPT ![other] = Ended(@)]
       /\ events' = Append(events, ev)
       /\ retired' = retired \cup {other}
       /\ UNCHANGED <<nextGen, unnamedLive, everBorn>>

\* 6. Merge ending both parents and minting a fresh name.
CommitMergeMintNew(p1, p2, minted, owner, part) ==
    /\ Len(events) < MaxEvents
    /\ p1 \in Names /\ p2 \in Names /\ minted \in Names
    /\ p1 /= p2 /\ live[p1].alive /\ live[p2].alive /\ minted \notin everBorn
    /\ owner \in HolderOwners /\ part \in Partitions
    /\ LET h == Holder(nextGen, owner, part)
           ev == [kind |-> "MergeMint", p1 |-> p1, p2 |-> p2, minted |-> minted,
                  gen |-> nextGen, owner |-> owner, part |-> part]
       IN
       /\ nextGen' = nextGen + 1
       /\ live' = [live EXCEPT ![p1] = Ended(@), ![p2] = Ended(@), ![minted] = h]
       /\ derivedIndex' = [derivedIndex EXCEPT ![p1] = Ended(@), ![p2] = Ended(@), ![minted] = h]
       /\ events' = Append(events, ev)
       /\ everBorn' = everBorn \cup {minted}
       /\ retired' = retired \cup {p1, p2}
       /\ UNCHANGED unnamedLive

\* 7. Retire a live name.
CommitRetire(n) ==
    /\ Len(events) < MaxEvents
    /\ n \in Names /\ live[n].alive
    /\ live' = [live EXCEPT ![n] = Ended(@)]
    /\ derivedIndex' = [derivedIndex EXCEPT ![n] = Ended(@)]
    /\ events' = Append(events, [kind |-> "Retire", target |-> n])
    /\ retired' = retired \cup {n}
    /\ UNCHANGED <<nextGen, unnamedLive, everBorn>>

\* 8. Restore a retired name onto a strictly fresh generation.
CommitRestore(n, owner, part) ==
    /\ Len(events) < MaxEvents
    /\ n \in retired /\ ~live[n].alive
    /\ owner \in HolderOwners /\ part \in Partitions
    /\ LET h == Holder(nextGen, owner, part)
           ev == [kind |-> "Restore", target |-> n, gen |-> nextGen, owner |-> owner, part |-> part]
       IN
       /\ nextGen' = nextGen + 1
       /\ live' = [live EXCEPT ![n] = h]
       /\ derivedIndex' = [derivedIndex EXCEPT ![n] = h]
       /\ events' = Append(events, ev)
       /\ retired' = retired \ {n}
       /\ UNCHANGED <<unnamedLive, everBorn>>

\* 10. One coupled envelope: two members (authored stages or coupled
\*     producers) each birth a fresh name. Their plans compose into the one
\*     commit's plan, so both births append atomically on fresh generations.
CommitCoupledCreate(n1, o1, p1, n2, o2, p2) ==
    /\ Len(events) + 2 <= MaxEvents
    /\ n1 \in Names /\ n2 \in Names /\ n1 /= n2
    /\ n1 \notin everBorn /\ n2 \notin everBorn
    /\ o1 \in HolderOwners /\ o2 \in HolderOwners
    /\ p1 \in Partitions /\ p2 \in Partitions
    /\ LET h1 == Holder(nextGen, o1, p1)
           h2 == Holder(nextGen + 1, o2, p2)
           e1 == [kind |-> "Create", target |-> n1, gen |-> nextGen, owner |-> o1, part |-> p1]
           e2 == [kind |-> "Create", target |-> n2, gen |-> nextGen + 1, owner |-> o2, part |-> p2]
       IN
       /\ nextGen' = nextGen + 2
       /\ live' = [live EXCEPT ![n1] = h1, ![n2] = h2]
       /\ derivedIndex' = [derivedIndex EXCEPT ![n1] = h1, ![n2] = h2]
       /\ events' = events \o <<e1, e2>>
       /\ everBorn' = everBorn \cup {n1, n2}
       /\ UNCHANGED <<unnamedLive, retired>>

\* 11. Two members of one envelope mint the same fresh name: the composed
\*     plan refuses typed (`DuplicateBirth`) and no member's effect commits.
\*     (A member leaving a schemed record unnamed is `RefuseUnnamedBirth`.)
RefuseCoupledDuplicateBirth(n) ==
    /\ n \in Names /\ n \notin everBorn
    /\ UNCHANGED vars

\* 9. Destroy the derived index and rebuild it from the canonical log.
DestroyAndRebuildIndex ==
    /\ derivedIndex' = ReplayEvents(events, Len(events))
    /\ UNCHANGED <<nextGen, live, unnamedLive, events, everBorn, retired>>

Next ==
    \/ \E n \in Names, o \in HolderOwners, p \in Partitions: CommitCreate(n, o, p)
    \/ \E n \in Names: RefuseDuplicateBirth(n)
    \/ \E o \in HolderOwners, p \in Partitions: RefuseUnnamedBirth(o, p)
    \/ \E n \in Names, o \in HolderOwners, p \in Partitions: CommitRegeneratePreserve(n, o, p)
    \/ \E par, c1, c2 \in Names, o1, o2 \in HolderOwners, p \in Partitions:
            CommitSplit(par, c1, c2, o1, o2, p)
    \/ \E k, o \in Names: CommitMergeKeepParentInPlace(k, o)
    \/ \E p1, p2, m \in Names, o \in HolderOwners, p \in Partitions:
            CommitMergeMintNew(p1, p2, m, o, p)
    \/ \E n \in Names: CommitRetire(n)
    \/ \E n \in Names, o \in HolderOwners, p \in Partitions: CommitRestore(n, o, p)
    \/ \E n1, n2 \in Names, o1, o2 \in HolderOwners, p1, p2 \in Partitions:
            CommitCoupledCreate(n1, o1, p1, n2, o2, p2)
    \/ \E n \in Names: RefuseCoupledDuplicateBirth(n)
    \/ DestroyAndRebuildIndex

Spec == Init /\ [][Next]_vars

\* ===========================================================================
\* Safety & temporal properties
\* ===========================================================================

TypeOK ==
    /\ nextGen >= 1
    /\ unnamedLive \in Nat
    /\ Len(events) <= MaxEvents
    /\ everBorn \subseteq Names
    /\ retired \subseteq everBorn
    /\ retired = {n \in everBorn : ~live[n].alive}

\* 1. Completeness guard: no unnamed live birth escapes a guarded scheme.
CompletenessGuard == unnamedLive = 0

\* 2. The derived index equals live state and a pure replay of the log.
DerivedIndexEquivalence ==
    /\ derivedIndex = live
    /\ derivedIndex = ReplayEvents(events, Len(events))
    /\ everBorn = {n \in Names : BornInLog(events, n)}

\* 3. Index resolution equals log resolution for every name, every scope
\*    (#133) and every selection.
ResolutionMatchesEventOracle ==
    \A n \in Names, scope \in Scopes, sel \in Selections:
        ResolveFromIndex(n, scope, sel) = ResolveFromEvents(n, scope, sel)

\* 4. Two or more live descendants never collapse to Exact.
NoSilentRebindOnAmbiguity ==
    \A n \in Names, scope \in Scopes:
        LET rAll == ResolveFromIndex(n, scope, "AllDescendants")
            rExact == ResolveFromIndex(n, scope, "Exact")
        IN (rAll.status = "Plural") => (rExact.status = "Ambiguous" /\ rExact.targets = rAll.targets)

\* 5. Multi-body owner scoping (#133): when an unscoped Exact is ambiguous,
\*    an owner holding exactly one candidate resolves it Exact.
OwnerScopedDisambiguation ==
    \A n \in Names, o \in Owners:
        LET unscoped == ResolveFromIndex(n, "all", "Exact")
            held == {c \in unscoped.targets : derivedIndex[c].owner = o}
        IN /\ (unscoped.status = "Ambiguous" /\ Cardinality(held) = 1) =>
                  ResolveFromIndex(n, o, "Exact") = [status |-> "Exact", targets |-> held]
           /\ (ResolveFromIndex(n, o, "AllDescendants").status = "Exact") =>
                  ResolveFromIndex(n, o, "Exact") = ResolveFromIndex(n, o, "AllDescendants")

\* 6. No two live records share a generation; every live generation was issued.
GenerationalSafety ==
    /\ \A n \in Names: live[n].alive => (live[n].gen >= 1 /\ live[n].gen < nextGen)
    /\ \A n1, n2 \in Names:
        (live[n1].alive /\ live[n2].alive /\ n1 /= n2) => live[n1].gen /= live[n2].gen

\* 7. The log only grows; a name coming back to life is a fresh generation;
\*    a KeepParentInPlace keeper keeps its exact record (#314).
AppendOnlyLineage ==
    [][ /\ Len(events') >= Len(events)
        /\ \A i \in 1..Len(events): events'[i] = events[i]
        /\ \A n \in Names:
            (~live[n].alive /\ live'[n].alive) => live'[n].gen > live[n].gen
        /\ (Len(events') > Len(events) /\ events'[Len(events')].kind = "MergeKeep") =>
                live'[events'[Len(events')].keeper] = live[events'[Len(events')].keeper]
      ]_vars

=============================================================================
