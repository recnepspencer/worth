------------------------ MODULE FamilySelectionWatch ------------------------
EXTENDS Integers, FiniteSets

(*
  A family selection read for public #429.

  A `current_output` read resolves an output family against the lineage
  heads of one partition. The outcome is Missing (no current candidate),
  Unique(e) (one current candidate e) or Ambiguous (more than one). Before
  #429 only a Unique read recorded anything a later write could invalidate:
  a Missing or Ambiguous read recorded no settlement. So a later first
  completion or readmission left the reader Clean on an outcome that had
  moved.

  Option B (plan r9, section 3.1) records the read as a FamilySelection fact.
  The fact posts the family's key, so every lineage head change in the
  partition wakes the reader. The fact is certified only by re-resolving the
  selection from the lineage heads at the certifier's coordinate. A
  certification is Clean when the re-resolved outcome equals the recorded
  one. Otherwise it is Moved, and the reader executes again.

  A reader records:
  - `part`: the coordinate it is registered at now;
  - `readPart`: the coordinate its read ran at;
  - `outcome`: what the read selected;
  - `vers`: the candidates' settlement identities when it read;
  - `status`: Clean, or Dirty once a wake or a registration demands a
    certification.

  `last` keeps only the most recent certification, so the invariants check
  each certification in the state it produces.

  `Mutant` names a defect to re-introduce. Each mutant module sets it, and
  TLC must refute that module's invariant.
  - FamilySelectionKeyedClean: the pre-#429 rule. Wakes are keyed on the
    selected candidate's settlement, and a Missing or Ambiguous reader
    certifies Clean with nothing to compare.
  - ReresolveAtReadCoordinate: certification re-resolves at the read's
    coordinate, not the certifier's. It is wrong once a fork splits them.
  - CompareBySettlementIdentity: certification also compares the selected
    candidate's settlement identity, not only the selection's meaning, so an
    equal adoption moves the reader.
*)

CONSTANTS
    Entities,     \* candidate outputs of the family, e.g. {"e1", "e2"}
    Readers,      \* consumer rows that read the family
    Partitions,   \* lineage partitions; the first is live, a fork adds one
    Origin,       \* the partition that is live initially
    MaxSteps,     \* bound on lineage, read, registration and fork events
    Mutant        \* "none" or a mutant name from the header

ASSUME Origin \in Partitions
ASSUME MaxSteps \in Nat
ASSUME Mutant \in {"none", "FamilySelectionKeyedClean",
                   "ReresolveAtReadCoordinate", "CompareBySettlementIdentity"}

Status == {"none", "current", "retired"}
Outcomes == {"Missing", "Ambiguous"} \cup Entities
Versions == 0..MaxSteps

VARIABLES
    live,      \* partitions that exist
    heads,     \* heads[p][e]: candidate e's lineage status in partition p
    vers,      \* vers[p][e]: candidate e's settlement identity in partition p
    readers,   \* readers[r]: the record above
    last,      \* the most recent certification
    steps

vars == <<live, heads, vers, readers, last, steps>>

Select(h) ==
    LET cur == {e \in Entities : h[e] = "current"}
    IN  IF cur = {} THEN "Missing"
        ELSE IF Cardinality(cur) = 1 THEN CHOOSE e \in cur : TRUE
        ELSE "Ambiguous"

NoCert == [kind |-> "none", recorded |-> "Missing", lineage |-> "Missing"]

TypeOK ==
    /\ live \subseteq Partitions /\ Origin \in live
    /\ heads \in [Partitions -> [Entities -> Status]]
    /\ vers \in [Partitions -> [Entities -> Versions]]
    /\ readers \in [Readers -> [part : Partitions, readPart : Partitions,
                                outcome : Outcomes,
                                vers : [Entities -> Versions],
                                status : {"Clean", "Dirty"}]]
    /\ last \in [kind : {"none", "clean", "moved"},
                 recorded : Outcomes, lineage : Outcomes]
    /\ steps \in 0..MaxSteps

Init ==
    /\ live = {Origin}
    /\ heads = [p \in Partitions |-> [e \in Entities |-> "none"]]
    /\ vers = [p \in Partitions |-> [e \in Entities |-> 0]]
    /\ readers = [r \in Readers |->
                    [part |-> Origin, readPart |-> Origin, outcome |-> "Missing",
                     vers |-> [e \in Entities |-> 0], status |-> "Clean"]]
    /\ last = NoCert
    /\ steps = 0

\* The FamilySelection key wakes every reader registered at p. The mutant
\* keys the wake on the selected candidate's settlement, so it reaches only
\* a Unique(e) reader whose e changed.
Woken(r, p, e) ==
    /\ readers[r].part = p
    /\ IF Mutant = "FamilySelectionKeyedClean"
       THEN readers[r].outcome = e
       ELSE TRUE

\* One lineage head change of candidate e in partition p.
Change(p, e, newStatus, newVer) ==
    /\ steps < MaxSteps
    /\ p \in live
    /\ heads' = [heads EXCEPT ![p][e] = newStatus]
    /\ vers' = [vers EXCEPT ![p][e] = newVer]
    /\ readers' = [r \in Readers |->
                     IF Woken(r, p, e)
                     THEN [readers[r] EXCEPT !.status = "Dirty"]
                     ELSE readers[r]]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, last>>

\* The producer's first completion: Missing can become Unique.
Complete(p, e) ==
    /\ heads[p][e] = "none"
    /\ Change(p, e, "current", vers[p][e] + 1)

\* A readmission issues a new settlement for a retired candidate.
Readmit(p, e) ==
    /\ heads[p][e] = "retired"
    /\ Change(p, e, "current", vers[p][e] + 1)

\* A recovery restores the retired candidate's prior settlement.
Recover(p, e) ==
    /\ heads[p][e] = "retired"
    /\ Change(p, e, "current", vers[p][e])

\* An equal adoption issues a new settlement with the same meaning.
EqualAdopt(p, e) ==
    /\ heads[p][e] = "current"
    /\ Change(p, e, "current", vers[p][e] + 1)

Retire(p, e) ==
    /\ heads[p][e] = "current"
    /\ Change(p, e, "retired", vers[p][e])

\* A fresh execution: the reader reads at its own coordinate and is Clean.
Execute(r) ==
    [readers[r] EXCEPT !.readPart = readers[r].part,
                       !.outcome = Select(heads[readers[r].part]),
                       !.vers = vers[readers[r].part],
                       !.status = "Clean"]

Read(r) ==
    /\ steps < MaxSteps
    /\ readers' = [readers EXCEPT ![r] = Execute(r)]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, heads, vers, last>>

\* A fork copies p's heads into a new partition q.
Fork(p, q) ==
    /\ steps < MaxSteps
    /\ p \in live /\ q \notin live
    /\ live' = live \cup {q}
    /\ heads' = [heads EXCEPT ![q] = heads[p]]
    /\ vers' = [vers EXCEPT ![q] = vers[p]]
    /\ steps' = steps + 1
    /\ UNCHANGED <<readers, last>>

\* The reader's recorded read is registered at another coordinate (a
\* consumer on a forked head, or the post-commit rebase). The registration
\* demands a certification there.
Register(r, q) ==
    /\ steps < MaxSteps
    /\ q \in live /\ q # readers[r].part
    /\ readers' = [readers EXCEPT ![r].part = q, ![r].status = "Dirty"]
    /\ steps' = steps + 1
    /\ UNCHANGED <<live, heads, vers, last>>

\* Re-resolution: the outcome at the certifier's coordinate. The mutant
\* re-resolves at the read's coordinate.
Reresolve(r) ==
    IF Mutant = "ReresolveAtReadCoordinate"
    THEN Select(heads[readers[r].readPart])
    ELSE Select(heads[readers[r].part])

CertifiesClean(r) ==
    CASE Mutant = "FamilySelectionKeyedClean"
            /\ readers[r].outcome \in {"Missing", "Ambiguous"} ->
            TRUE
      [] Mutant = "CompareBySettlementIdentity" ->
            LET o == readers[r].outcome
            IN  /\ Reresolve(r) = o
                /\ o \in Entities => vers[readers[r].part][o] = readers[r].vers[o]
      [] OTHER ->
            Reresolve(r) = readers[r].outcome

Certify(r) ==
    /\ readers[r].status = "Dirty"
    /\ LET lineage == Select(heads[readers[r].part])
       IN  IF CertifiesClean(r)
           THEN /\ readers' = [readers EXCEPT ![r].status = "Clean"]
                /\ last' = [kind |-> "clean", recorded |-> readers[r].outcome,
                            lineage |-> lineage]
           ELSE /\ readers' = [readers EXCEPT ![r] = Execute(r)]
                /\ last' = [kind |-> "moved", recorded |-> readers[r].outcome,
                            lineage |-> lineage]
    /\ UNCHANGED <<live, heads, vers, steps>>

Next ==
    \/ \E p \in Partitions, e \in Entities :
          \/ Complete(p, e) \/ Readmit(p, e) \/ Recover(p, e)
          \/ EqualAdopt(p, e) \/ Retire(p, e)
    \/ \E p, q \in Partitions : Fork(p, q)
    \/ \E r \in Readers : Read(r) \/ Certify(r)
    \/ \E r \in Readers, q \in Partitions : Register(r, q)

Spec == Init /\ [][Next]_vars /\ \A r \in Readers : WF_vars(Certify(r))

-----------------------------------------------------------------------------

\* Every Clean certification re-resolved the outcome it recorded, at the
\* certifier's coordinate.
CleanOnlyFromReresolution ==
    last.kind = "clean" => last.recorded = last.lineage

\* A Clean reader's outcome is the lineage outcome at its coordinate.
CleanMeansCurrentSelection ==
    \A r \in Readers :
        readers[r].status = "Clean" =>
            readers[r].outcome = Select(heads[readers[r].part])

\* A certification moves a reader only when the selection's meaning moved,
\* so an equal adoption never re-executes a current reader.
EqualAdoptionStaysCurrent ==
    last.kind = "moved" => last.recorded # last.lineage

\* No wedge: a woken or demanded reader eventually certifies or executes.
Converges ==
    \A r \in Readers : (readers[r].status = "Dirty") ~> (readers[r].status = "Clean")
=============================================================================
