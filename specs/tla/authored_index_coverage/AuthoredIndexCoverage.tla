------------------------- MODULE AuthoredIndexCoverage -------------------------
(***************************************************************************)
(* Authored index-write coverage across checkpoint restore and WAL replay  *)
(* (public #384). Bounded interleavings of registration, commits, legacy  *)
(* and adversarial packets, checkpoints and crash-reopens.                 *)
(*                                                                         *)
(* Recovery is modelled as paired views over the same history and the same *)
(* restored catalog: view k certifies commits 1..k (checkpoint-decoded) at *)
(* the receiving catalog and replay-mints commits k+1..n (WAL tail) at the *)
(* catalog installed when each was replayed. k = 0 is all-WAL and          *)
(* k = Len(commits) is all-checkpoint.                                     *)
(*                                                                         *)
(* Mirrors (Rust):                                                         *)
(*   Ext            catalog_coverage.rs `extends`                          *)
(*   Insert         definition_catalog.rs `insert` (epoch, introductions)  *)
(*   LastBelow      definition_catalog.rs `coverage_before`                *)
(*   Restore        catalog_publication.rs `readmit_checkpoint_catalog`    *)
(*                  and `readmit_catalog_delta` (next id only)             *)
(*   Certify        catalog_publication.rs `certify_readmitted_writes`     *)
(*   ReplayMint     pipeline index_catalog.rs + authored_index_writes.rs   *)
(*   Gen            authored_writes/indexed.rs `append`                    *)
(*   Admit          relevant_writes/cursor/admission.rs (index selectors)  *)
(* The Lean development specs/lean/authored_index_coverage proves the same *)
(* properties unbounded.                                                   *)
(***************************************************************************)
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS MaxDefs, MaxCommits, MaxReopens, Mutant

ASSUME Mutant \in {"none", "certifyNext", "legacyW", "remint", "outOfOrder"}

Ids == 1..MaxDefs
None == [ok |-> FALSE, inc |-> 0, epoch |-> 0]
Some(inc, epoch) == [ok |-> TRUE, inc |-> inc, epoch |-> epoch]
Ext(a, b) == a.inc = b.inc /\ a.epoch >= b.epoch

VARIABLES cat, commits, ckpt, reopens, injected
vars == <<cat, commits, ckpt, reopens, injected>>

Fresh(i) == [inc |-> i, epoch |-> 0, intro |-> [d \in Ids |-> 0], next |-> 1]
Insert(c, id) == [c EXCEPT !.epoch = c.epoch + 1, !.intro[id] = c.epoch + 1]

RECURSIVE InsertSeq(_, _)
InsertSeq(c, s) == IF s = <<>> THEN c ELSE InsertSeq(Insert(c, Head(s)), Tail(s))

Ascending(n) == [i \in 1..(n - 1) |-> i]
Descending(n) == [i \in 1..(n - 1) |-> n - i]
Restore(i, n) ==
    [InsertSeq(Fresh(i), IF Mutant = "outOfOrder" THEN Descending(n) ELSE Ascending(n))
        EXCEPT !.next = n]

RECURSIVE LastBelow(_, _)
LastBelow(c, w) ==
    IF w <= 1 THEN 0
    ELSE IF c.intro[w - 1] > 0 THEN c.intro[w - 1] ELSE LastBelow(c, w - 1)

CovBefore(c, w) == Some(c.inc, LastBelow(c, w))
Intro(c, d) == IF c.intro[d] > 0 THEN Some(c.inc, c.intro[d]) ELSE None

Honest(cm) == cm.packet.kind = "Covered" /\ cm.packet.b = cm.W

Certify(c, cm) ==
    IF Mutant = "legacyW" /\ cm.packet.kind = "Legacy" /\ cm.W <= c.next
        THEN CovBefore(c, cm.W)
    ELSE IF cm.packet.kind = "Covered" /\ cm.packet.b = cm.W /\ cm.packet.b <= c.next
        THEN IF Mutant = "certifyNext" THEN CovBefore(c, c.next) ELSE CovBefore(c, cm.packet.b)
    ELSE None

ReplayMint(c, cm) ==
    IF Mutant = "remint" /\ cm.packet.kind = "Legacy" /\ cm.W <= c.next
        THEN CovBefore(c, cm.W)
    ELSE IF cm.packet.kind = "Covered" /\ cm.W <= c.next /\ cm.packet.b = cm.W
        THEN CovBefore(c, cm.W)
    ELSE None

\* WAL replay readmits commit i's delta before preparing it, in the incarnation
\* the reopen restored.
ReplayCat(i) == Restore(cat.inc, commits[i].W)

Mint(k, i) == IF i <= k THEN Certify(cat, commits[i]) ELSE ReplayMint(ReplayCat(i), commits[i])

GenInit == [incomplete |-> {}, cov |-> [o \in 1..MaxCommits |-> None],
            runStart |-> 0, runLast |-> None]

RECURSIVE Gen(_, _)
Gen(k, n) ==
    IF n = 0 THEN GenInit
    ELSE LET g == Gen(k, n - 1)
             m == Mint(k, n)
         IN IF ~m.ok THEN [g EXCEPT !.incomplete = @ \cup {n}]
            ELSE [g EXCEPT !.cov[n] = m,
                           !.runStart = IF g.runLast.ok /\ Ext(m, g.runLast)
                                           THEN g.runStart ELSE n,
                           !.runLast = m]

\* One index selector for d over the open interval from commit s.
Admit(k, d, s) ==
    LET n == Len(commits)
        g == Gen(k, n)
        i == Intro(cat, d)
        covered == {o \in s..n : g.cov[o].ok}
    IN IF \E o \in g.incomplete : o >= s THEN "Refuse"
       ELSE IF ~i.ok THEN "Refuse"
       ELSE IF s > n THEN "Ok"
       ELSE IF s < g.runStart THEN "Refuse"
       ELSE IF covered = {} THEN "Refuse"
       ELSE LET f == CHOOSE o \in covered : \A p \in covered : o <= p
            IN IF Ext(g.cov[f], i) THEN "Ok" ELSE "Refuse"

Views == 0..Len(commits)
Spans == 1..(Len(commits) + 1)

Packet(kind, b) == [kind |-> kind, b |-> b]
Below(w) == {d \in Ids : d < w}

Init ==
    /\ cat = Fresh(0)
    /\ commits = <<>>
    /\ ckpt = 0
    /\ reopens = 0
    /\ injected = [legacy |-> 0, adversarial |-> 0]

Register ==
    /\ cat.next <= MaxDefs
    /\ cat' = [Insert(cat, cat.next) EXCEPT !.next = cat.next + 1]
    /\ UNCHANGED <<commits, ckpt, reopens, injected>>

AppendCommit(packet, writes) ==
    /\ Len(commits) < MaxCommits
    /\ commits' = Append(commits, [W |-> cat.next, packet |-> packet, writes |-> writes])

Commit(kind) ==
    /\ kind \in {"Covered", "Unavail"}
    /\ AppendCommit(Packet(kind, IF kind = "Covered" THEN cat.next ELSE 0),
                    IF kind = "Covered" THEN Below(cat.next) ELSE {})
    /\ UNCHANGED <<cat, ckpt, reopens, injected>>

InjectLegacy ==
    /\ injected.legacy < 1
    /\ AppendCommit(Packet("Legacy", 0), Below(cat.next))
    /\ injected' = [injected EXCEPT !.legacy = @ + 1]
    /\ UNCHANGED <<cat, ckpt, reopens>>

InjectCovered(b) ==
    /\ injected.adversarial < 1
    /\ b /= cat.next
    /\ AppendCommit(Packet("Covered", b), Below(cat.next))
    /\ injected' = [injected EXCEPT !.adversarial = @ + 1]
    /\ UNCHANGED <<cat, ckpt, reopens>>

Checkpoint ==
    /\ ckpt' = Len(commits)
    /\ UNCHANGED <<cat, commits, reopens, injected>>

CrashReopen ==
    /\ reopens < MaxReopens
    /\ cat' = Restore(cat.inc + 1, cat.next)
    /\ reopens' = reopens + 1
    /\ UNCHANGED <<commits, ckpt, injected>>

Next ==
    \/ Register
    \/ \E kind \in {"Covered", "Unavail"} : Commit(kind)
    \/ InjectLegacy
    \/ \E b \in 1..(MaxDefs + 1) : InjectCovered(b)
    \/ Checkpoint
    \/ CrashReopen

Spec == Init /\ [][Next]_vars

TypeOK ==
    /\ cat.next \in 1..(MaxDefs + 1)
    /\ \A d \in Ids : cat.intro[d] \in Nat
    /\ Len(commits) <= MaxCommits
    /\ ckpt \in 0..Len(commits)
    /\ reopens \in 0..MaxReopens

\* (a) Coverage never extends a later index, and an admitted span holds the writes.
CoverageSound ==
    \A k \in Views, d \in Ids :
        /\ \A i \in 1..Len(commits) :
               LET m == Mint(k, i) IN
               LET it == Intro(cat, d) IN
               (m.ok /\ it.ok /\ Ext(m, it)) => d < commits[i].W
        /\ \A s \in Spans :
               Admit(k, d, s) = "Ok" => \A o \in s..Len(commits) : d \in commits[o].writes

\* (b) Unavailable, legacy and adversarial packets mint nothing and refuse every span over them.
UnprovenancedRefuses ==
    \A k \in Views, i \in 1..Len(commits) :
        ~Honest(commits[i]) =>
            /\ ~Mint(k, i).ok
            /\ \A d \in Ids, s \in 1..i : Admit(k, d, s) = "Refuse"

\* (c) Every split between checkpoint and WAL tail mints and admits identically.
RecoveryParity ==
    \A k \in Views :
        /\ \A i \in 1..Len(commits) : Mint(k, i) = Mint(0, i)
        /\ \A d \in Ids, s \in Spans : Admit(k, d, s) = Admit(0, d, s)

=============================================================================
