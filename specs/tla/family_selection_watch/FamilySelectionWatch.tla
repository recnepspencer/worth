------------------------ MODULE FamilySelectionWatch ------------------------
EXTENDS Integers, FiniteSets

(*
  A family selection read for public #429.

  A `current_output` read resolves an output family against the lineage
  heads at one product coordinate. The outcome is Missing (no current
  candidate), Unique(e) (one current candidate e) or Ambiguous (more than
  one). Before #429 only a Unique read recorded anything a later write could
  invalidate: a Missing or Ambiguous read recorded no settlement, so its row
  stayed Clean by marks. A later first completion or readmission then left
  the reader serving an outcome that had moved.

  Option B (plan r9, sections 3.1-3.4) records the read as a FamilySelection
  fact. Marks never certify that fact: its row is UnsupportedFact, so every
  certification re-resolves the selection from the lineage heads at the
  certifier's coordinate. It is Clean when the re-resolved outcome equals the
  recorded one; otherwise it is Moved and the reader executes again. Wakes
  only drive convergence: a head change that reaches the reader's key sets a
  pending re-drive. Some head changes escape the wake (`record_restoration`,
  a failed republication registration); a reader they skip stays stale only
  until its next demand, which still re-resolves.

  A reader records:
  - `coord`: the product coordinate it is registered at now;
  - `readCoord`: the coordinate its read ran at;
  - `outcome`: what the read selected;
  - `vers`: the candidates' settlement identities when it read;
  - `pending`: a wake or a registration asked for a re-drive.

  Head statuses: `none` (never completed), `current`, `retired`, and
  `restored`: an unwitnessed restore head that is stale at the coordinate,
  which every verdict treats as not current.

  `last` keeps only the most recent certification (a demand or a re-drive),
  so the invariants check each certification in the state it produces.

  `Mutant` names a defect to re-introduce. Each mutant module sets it, and
  TLC must refute that module's property.
  - FamilySelectionKeyedClean: the pre-#429 rule. Wakes are keyed on the
    selected candidate, and a Missing or Ambiguous reader is Clean by marks,
    so a demand serves its recorded outcome without re-resolving.
  - ReresolveAtReadCoordinate: certification re-resolves at the read's
    coordinate, not the certifier's. It is wrong once a fork splits them.
  - CompareBySettlementIdentity: certification also compares the selected
    candidate's settlement identity, not only the selection's meaning, so an
    equal adoption moves the reader.
  - ImmediateMovedOnRestore: the r5 rule. Any unwitnessed restore head at
    the coordinate answers Moved. The re-execution leaves that head
    unwitnessed and the post-commit rebase re-drives, so it never converges.
*)

CONSTANTS
    Entities,     \* candidate outputs of the family, e.g. {"e1", "e2"}
    Readers,      \* consumer rows that read the family
    Coordinates,  \* product coordinates; the first is live, a fork adds one
    Origin,       \* the coordinate that is live initially
    MaxSteps,     \* bound on lineage, read, registration and fork events
    Mutant        \* "none" or a mutant name from the header

ASSUME Origin \in Coordinates
ASSUME MaxSteps \in Nat
ASSUME Mutant \in {"none", "FamilySelectionKeyedClean",
                   "ReresolveAtReadCoordinate", "CompareBySettlementIdentity",
                   "ImmediateMovedOnRestore"}

Status == {"none", "current", "retired", "restored"}
Outcomes == {"Missing", "Ambiguous"} \cup Entities
Versions == 0..MaxSteps

VARIABLES
    live,      \* coordinates that exist
    heads,     \* heads[c][e]: candidate e's lineage status at coordinate c
    vers,      \* vers[c][e]: candidate e's settlement identity at c
    readers,   \* readers[r]: the record above
    last,      \* the most recent certification
    steps

vars == <<live, heads, vers, readers, last, steps>>

Select(h) ==
    LET cur == {e \in Entities : h[e] = "current"}
    IN  IF cur = {} THEN "Missing"
        ELSE IF Cardinality(cur) = 1 THEN CHOOSE e \in cur : TRUE
        ELSE "Ambiguous"

NoCert == [kind |-> "none", recorded |-> "Missing", served |-> "Missing",
           lineage |-> "Missing"]

TypeOK ==
    /\ live \subseteq Coordinates /\ Origin \in live
    /\ heads \in [Coordinates -> [Entities -> Status]]
    /\ vers \in [Coordinates -> [Entities -> Versions]]
    /\ readers \in [Readers -> [coord : Coordinates, readCoord : Coordinates,
                                outcome : Outcomes,
                                vers : [Entities -> Versions],
                                pending : BOOLEAN]]
    /\ last \in [kind : {"none", "clean", "moved"}, recorded : Outcomes,
                 served : Outcomes, lineage : Outcomes]
    /\ steps \in 0..MaxSteps

Init ==
    /\ live = {Origin}
    /\ heads = [c \in Coordinates |-> [e \in Entities |-> "none"]]
    /\ vers = [c \in Coordinates |-> [e \in Entities |-> 0]]
    /\ readers = [r \in Readers |->
                    [coord |-> Origin, readCoord |-> Origin, outcome |-> "Missing",
                     vers |-> [e \in Entities |-> 0], pending |-> FALSE]]
    /\ last = NoCert
    /\ steps = 0

\* The FamilySelection key wakes every reader registered at c. The mutant
\* keys the wake on the selected candidate, so it reaches only a Unique(e)
\* reader whose e changed.
Woken(r, c, e) ==
    /\ readers[r].coord = c
    /\ IF Mutant = "FamilySelectionKeyedClean"
       THEN readers[r].outcome = e
       ELSE TRUE

\* One lineage head change of candidate e at coordinate c. `wake` is FALSE
\* for a change that escapes the wake.
Change(c, e, newStatus, newVer, wake) ==
    /\ steps < MaxSteps
    /\ c \in live
    /\ heads' = [heads EXCEPT ![c][e] = newStatus]
    /\ vers' = [vers EXCEPT ![c][e] = newVer]
    /\ readers' = [r \in Readers |->
                     IF wake /\ Woken(r, c, e)
                     THEN [readers[r] EXCEPT !.pending = TRUE]
                     ELSE readers[r]]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, last>>

\* The producer's first completion: Missing can become Unique.
Complete(c, e, wake) ==
    /\ heads[c][e] = "none"
    /\ Change(c, e, "current", vers[c][e] + 1, wake)

\* A readmission issues a new settlement for a retired candidate.
Readmit(c, e, wake) ==
    /\ heads[c][e] = "retired"
    /\ Change(c, e, "current", vers[c][e] + 1, wake)

\* A recovery restores the retired candidate's prior settlement.
Recover(c, e, wake) ==
    /\ heads[c][e] = "retired"
    /\ Change(c, e, "current", vers[c][e], wake)

\* `record_restoration` with no witness gives the retired candidate a head
\* that is stale at the coordinate; it never wakes.
Restore(c, e) ==
    /\ heads[c][e] = "retired"
    /\ Change(c, e, "restored", vers[c][e] + 1, FALSE)

\* An equal adoption issues a new settlement with the same meaning.
EqualAdopt(c, e, wake) ==
    /\ heads[c][e] = "current"
    /\ Change(c, e, "current", vers[c][e] + 1, wake)

Retire(c, e, wake) ==
    /\ heads[c][e] \in {"current", "restored"}
    /\ Change(c, e, "retired", vers[c][e], wake)

\* A fresh execution: the reader reads at its own coordinate.
Execute(r) ==
    [readers[r] EXCEPT !.readCoord = readers[r].coord,
                       !.outcome = Select(heads[readers[r].coord]),
                       !.vers = vers[readers[r].coord],
                       !.pending = FALSE]

Read(r) ==
    /\ steps < MaxSteps
    /\ readers' = [readers EXCEPT ![r] = Execute(r)]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, heads, vers, last>>

\* A fork copies c's heads into a new coordinate d.
Fork(c, d) ==
    /\ steps < MaxSteps
    /\ c \in live /\ d \notin live
    /\ live' = live \cup {d}
    /\ heads' = [heads EXCEPT ![d] = heads[c]]
    /\ vers' = [vers EXCEPT ![d] = vers[c]]
    /\ steps' = steps + 1
    /\ UNCHANGED <<readers, last>>

\* The reader's recorded read is registered at another coordinate (a
\* consumer on a forked head, or the post-commit rebase). The registration
\* asks for a re-drive there.
Register(r, d) ==
    /\ steps < MaxSteps
    /\ d \in live /\ d # readers[r].coord
    /\ readers' = [readers EXCEPT ![r].coord = d, ![r].pending = TRUE]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, heads, vers, last>>

\* Re-resolution: the outcome at the certifier's coordinate. The mutant
\* re-resolves at the read's coordinate.
Reresolve(r) ==
    IF Mutant = "ReresolveAtReadCoordinate"
    THEN Select(heads[readers[r].readCoord])
    ELSE Select(heads[readers[r].coord])

\* The pre-#429 marks: a Missing or Ambiguous read recorded no settlement,
\* so its row is Clean and a demand serves the recorded outcome.
CleanByMarks(r) ==
    /\ Mutant = "FamilySelectionKeyedClean"
    /\ readers[r].outcome \in {"Missing", "Ambiguous"}

HasRestored(c) == \E e \in Entities : heads[c][e] = "restored"

CertifiesClean(r) ==
    CASE CleanByMarks(r) -> TRUE
      [] Mutant = "CompareBySettlementIdentity" ->
            LET o == readers[r].outcome
            IN  /\ Reresolve(r) = o
                /\ o \in Entities => vers[readers[r].coord][o] = readers[r].vers[o]
      [] Mutant = "ImmediateMovedOnRestore" /\ HasRestored(readers[r].coord) ->
            FALSE
      [] OTHER ->
            Reresolve(r) = readers[r].outcome

\* One certification of r, by a demand or by its re-drive. Moved executes
\* the reader. Under the r5 mutant the rebase refuses the superseded
\* settlement while the unwitnessed head remains, so it re-drives.
Certification(r) ==
    LET lineage == Select(heads[readers[r].coord])
    IN  IF CertifiesClean(r)
        THEN /\ readers' = [readers EXCEPT ![r].pending = FALSE]
             /\ last' = [kind |-> "clean", recorded |-> readers[r].outcome,
                         served |-> readers[r].outcome, lineage |-> lineage]
        ELSE /\ readers' = [readers EXCEPT ![r] =
                    [Execute(r) EXCEPT !.pending =
                        Mutant = "ImmediateMovedOnRestore"
                            /\ HasRestored(readers[r].coord)]]
             /\ last' = [kind |-> "moved", recorded |-> readers[r].outcome,
                         served |-> lineage, lineage |-> lineage]

\* A consumer demands r: certification runs whether or not a wake reached it.
Demand(r) ==
    /\ Certification(r)
    /\ UNCHANGED <<live, heads, vers, steps>>

\* The re-drive of a pending reader.
Certify(r) ==
    /\ readers[r].pending
    /\ Certification(r)
    /\ UNCHANGED <<live, heads, vers, steps>>

Next ==
    \/ \E c \in Coordinates, e \in Entities, wake \in BOOLEAN :
          \/ Complete(c, e, wake) \/ Readmit(c, e, wake) \/ Recover(c, e, wake)
          \/ EqualAdopt(c, e, wake) \/ Retire(c, e, wake)
    \/ \E c \in Coordinates, e \in Entities : Restore(c, e)
    \/ \E c, d \in Coordinates : Fork(c, d)
    \/ \E r \in Readers : Read(r) \/ Demand(r) \/ Certify(r)
    \/ \E r \in Readers, d \in Coordinates : Register(r, d)

Spec == Init /\ [][Next]_vars /\ \A r \in Readers : WF_vars(Certify(r))

-----------------------------------------------------------------------------

\* Every Clean certification re-resolved the outcome it recorded, at the
\* certifier's coordinate.
CleanOnlyFromReresolution ==
    last.kind = "clean" => last.recorded = last.lineage

\* Every certification serves the lineage outcome at the reader's
\* coordinate, even for a reader an escaped head change never woke.
ServesCurrentSelection ==
    last.kind # "none" => last.served = last.lineage

\* A certification moves a reader only when the selection's meaning moved,
\* so an equal adoption never re-executes a current reader.
EqualAdoptionStaysCurrent ==
    last.kind = "moved" => last.recorded # last.lineage

\* No wedge and no re-execution loop: a pending reader eventually stops
\* being pending.
Converges ==
    \A r \in Readers : readers[r].pending ~> ~readers[r].pending
=============================================================================
