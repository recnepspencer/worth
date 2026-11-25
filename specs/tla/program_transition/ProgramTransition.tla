--------------------------- MODULE ProgramTransition ---------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Formal specification of in-place Directory Home program transitions (v2 -> v3).

  Models:
    - A Directory Home on disk at version 0 (absent), 2 (v2), or 3 (v3),
    - Surviving v2 rows (`v2val`) plus the v3-added defaulted field (`priority`),
    - Managed index readiness on the added v3 field (`"None"`, `"Cold"`, `"Ready"`),
    - Explicit authoritative pre/post row history witnesses (`lastV2Rows`, `lastV3Rows`)
      so any mutation or deletion during budget-stop rollback or v3 reopen violates
      safety invariants immediately,
    - Operations:
        CreateV2, WriteV2, CloseHome, ReopenV2,
        OpenV3WithoutTransition (must refuse!),
        OpenV3WithTransition (atomic v2 -> v3 upgrade populating DefaultPriority
                              on every v2 row and bringing v3 index to Ready),
        OpenV3WithTransitionBudgetStop (atomic rollback: disk remains valid v2!),
        OpenV2OnV3 (downgrade must refuse without mutating disk!),
        WriteV3, ReopenV3 (idempotent: never re-runs transition or clobbers v3 edits).

  Safety invariants:
    - TypeInvariant
    - NoDataLossAcrossUpgrade
    - NoDowngrade
    - AtomicRollbackOnFailure
    - AtomicRollbackPreservesV2Rows
    - IdempotentReopenAfterUpgrade
    - ReopenV3PreservesEditedPriorities
    - NoPriorityInV2
    - AllPresentRowsHavePriorityInV3

  Liveness:
    - EventuallyUpgradesToV3
*)

CONSTANTS
    RowIds,
    V2Values,
    Priorities,
    DefaultPriority

ASSUME DefaultPriority \in Priorities

VARIABLES
    diskVersion,
    homeOpen,
    openVersion,
    rows,
    indexState,
    transitionRuns,
    v2RowsAtUpgrade,
    v3EditedRows,
    lastV2Rows,
    lastV3Rows,
    downgradeEverSucceeded,
    lastOutcome

vars == <<
    diskVersion,
    homeOpen,
    openVersion,
    rows,
    indexState,
    transitionRuns,
    v2RowsAtUpgrade,
    v3EditedRows,
    lastV2Rows,
    lastV3Rows,
    downgradeEverSucceeded,
    lastOutcome
>>

EmptyRow == [present |-> FALSE, v2val |-> "None", priority |-> "None"]
EmptySnapshotRow == [present |-> FALSE, v2val |-> "None"]

RowStates == [
    present: BOOLEAN,
    v2val: V2Values \cup {"None"},
    priority: Priorities \cup {"None"}
]

SnapshotRowStates == [
    present: BOOLEAN,
    v2val: V2Values \cup {"None"}
]

Outcomes == {
    "Init",
    "CreatedV2",
    "WroteV2",
    "Closed",
    "ReopenedV2",
    "RefusedTransitionRequired",
    "UpgradedV2ToV3",
    "RefusedBudgetStop",
    "RefusedDowngrade",
    "WroteV3",
    "ReopenedV3"
}

Init ==
    /\ diskVersion = 0
    /\ homeOpen = FALSE
    /\ openVersion = 0
    /\ rows = [r \in RowIds |-> EmptyRow]
    /\ indexState = "None"
    /\ transitionRuns = 0
    /\ v2RowsAtUpgrade = [r \in RowIds |-> EmptySnapshotRow]
    /\ v3EditedRows = {}
    /\ lastV2Rows = [r \in RowIds |-> EmptyRow]
    /\ lastV3Rows = [r \in RowIds |-> EmptyRow]
    /\ downgradeEverSucceeded = FALSE
    /\ lastOutcome = "Init"

CreateV2 ==
    /\ diskVersion = 0
    /\ ~homeOpen
    /\ diskVersion' = 2
    /\ homeOpen' = TRUE
    /\ openVersion' = 2
    /\ indexState' = "None"
    /\ lastV2Rows' = rows
    /\ lastOutcome' = "CreatedV2"
    /\ UNCHANGED <<rows, transitionRuns, v2RowsAtUpgrade, v3EditedRows,
                   lastV3Rows, downgradeEverSucceeded>>

WriteV2(r, v) ==
    /\ homeOpen
    /\ openVersion = 2
    /\ diskVersion = 2
    /\ r \in RowIds
    /\ v \in V2Values
    /\ LET nextRows == [rows EXCEPT ![r] = [present |-> TRUE, v2val |-> v, priority |-> "None"]] IN
        /\ rows' = nextRows
        /\ lastV2Rows' = nextRows
    /\ lastOutcome' = "WroteV2"
    /\ UNCHANGED <<diskVersion, homeOpen, openVersion, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV3Rows, downgradeEverSucceeded>>

CloseHome ==
    /\ homeOpen
    /\ homeOpen' = FALSE
    /\ openVersion' = 0
    /\ lastOutcome' = "Closed"
    /\ UNCHANGED <<diskVersion, rows, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

ReopenV2 ==
    /\ ~homeOpen
    /\ diskVersion = 2
    /\ homeOpen' = TRUE
    /\ openVersion' = 2
    /\ lastOutcome' = "ReopenedV2"
    /\ UNCHANGED <<diskVersion, rows, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

OpenV3WithoutTransition ==
    /\ ~homeOpen
    /\ diskVersion = 2
    /\ lastOutcome' = "RefusedTransitionRequired"
    /\ UNCHANGED <<diskVersion, homeOpen, openVersion, rows, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

OpenV3WithTransition ==
    /\ ~homeOpen
    /\ diskVersion = 2
    /\ diskVersion' = 3
    /\ homeOpen' = TRUE
    /\ openVersion' = 3
    /\ LET upgradedRows == [r \in RowIds |->
            IF rows[r].present
               THEN [present |-> TRUE, v2val |-> rows[r].v2val, priority |-> DefaultPriority]
               ELSE rows[r]] IN
        /\ rows' = upgradedRows
        /\ lastV3Rows' = upgradedRows
    /\ v2RowsAtUpgrade' = [r \in RowIds |-> [present |-> rows[r].present, v2val |-> rows[r].v2val]]
    /\ v3EditedRows' = {}
    /\ indexState' = "Ready"
    /\ transitionRuns' = transitionRuns + 1
    /\ lastOutcome' = "UpgradedV2ToV3"
    /\ UNCHANGED <<lastV2Rows, downgradeEverSucceeded>>

OpenV3WithTransitionBudgetStop ==
    /\ ~homeOpen
    /\ diskVersion = 2
    /\ lastOutcome' = "RefusedBudgetStop"
    /\ UNCHANGED <<diskVersion, homeOpen, openVersion, rows, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

OpenV2OnV3 ==
    /\ ~homeOpen
    /\ diskVersion = 3
    /\ lastOutcome' = "RefusedDowngrade"
    /\ UNCHANGED <<diskVersion, homeOpen, openVersion, rows, indexState, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

WriteV3(r, v, p) ==
    /\ homeOpen
    /\ openVersion = 3
    /\ diskVersion = 3
    /\ r \in RowIds
    /\ v \in V2Values
    /\ p \in Priorities
    /\ LET nextRows == [rows EXCEPT ![r] = [present |-> TRUE, v2val |-> v, priority |-> p]] IN
        /\ rows' = nextRows
        /\ lastV3Rows' = nextRows
    /\ v3EditedRows' = v3EditedRows \cup {r}
    /\ indexState' = "Ready"
    /\ lastOutcome' = "WroteV3"
    /\ UNCHANGED <<diskVersion, homeOpen, openVersion, transitionRuns,
                   v2RowsAtUpgrade, lastV2Rows, downgradeEverSucceeded>>

ReopenV3 ==
    /\ ~homeOpen
    /\ diskVersion = 3
    /\ homeOpen' = TRUE
    /\ openVersion' = 3
    /\ indexState' = "Ready"
    /\ lastOutcome' = "ReopenedV3"
    /\ UNCHANGED <<diskVersion, rows, transitionRuns,
                   v2RowsAtUpgrade, v3EditedRows, lastV2Rows, lastV3Rows,
                   downgradeEverSucceeded>>

Next ==
    \/ CreateV2
    \/ \E r \in RowIds, v \in V2Values : WriteV2(r, v)
    \/ CloseHome
    \/ ReopenV2
    \/ OpenV3WithoutTransition
    \/ OpenV3WithTransition
    \/ OpenV3WithTransitionBudgetStop
    \/ OpenV2OnV3
    \/ \E r \in RowIds, v \in V2Values, p \in Priorities : WriteV3(r, v, p)
    \/ ReopenV3

\* Fairness: an open v2 session eventually closes, and whenever a closed v2 Home
\* is repeatedly available, a valid v2 -> v3 upgrade eventually occurs.
CloseV2 == homeOpen /\ openVersion = 2 /\ CloseHome

Fairness ==
    /\ WF_vars(CloseV2)
    /\ SF_vars(OpenV3WithTransition)

Spec == Init /\ [][Next]_vars /\ Fairness

\* ---- Safety Invariants ----

TypeInvariant ==
    /\ diskVersion \in {0, 2, 3}
    /\ homeOpen \in BOOLEAN
    /\ openVersion \in {0, 2, 3}
    /\ rows \in [RowIds -> RowStates]
    /\ indexState \in {"None", "Cold", "Ready"}
    /\ transitionRuns \in {0, 1}
    /\ v2RowsAtUpgrade \in [RowIds -> SnapshotRowStates]
    /\ v3EditedRows \subseteq RowIds
    /\ lastV2Rows \in [RowIds -> RowStates]
    /\ lastV3Rows \in [RowIds -> RowStates]
    /\ downgradeEverSucceeded \in BOOLEAN
    /\ lastOutcome \in Outcomes
    /\ (homeOpen => openVersion = diskVersion /\ diskVersion \in {2, 3})
    /\ (~homeOpen => openVersion = 0)

NoDataLossAcrossUpgrade ==
    diskVersion = 3 =>
        \A r \in RowIds :
            v2RowsAtUpgrade[r].present =>
                /\ rows[r].present
                /\ (r \notin v3EditedRows =>
                        /\ rows[r].v2val = v2RowsAtUpgrade[r].v2val
                        /\ rows[r].priority = DefaultPriority)

NoDowngrade ==
    /\ ~downgradeEverSucceeded
    /\ (diskVersion = 3 => openVersion # 2)
    /\ (lastOutcome = "RefusedDowngrade" => ~homeOpen /\ diskVersion = 3)

AtomicRollbackOnFailure ==
    lastOutcome = "RefusedBudgetStop" =>
        /\ diskVersion = 2
        /\ ~homeOpen
        /\ openVersion = 0
        /\ transitionRuns = 0
        /\ indexState = "None"
        /\ \A r \in RowIds : rows[r].priority = "None"

AtomicRollbackPreservesV2Rows ==
    diskVersion = 2 => rows = lastV2Rows

IdempotentReopenAfterUpgrade ==
    /\ transitionRuns <= 1
    /\ (diskVersion = 3 => transitionRuns = 1 /\ indexState = "Ready")
    /\ (lastOutcome = "ReopenedV3" => homeOpen /\ openVersion = 3 /\ transitionRuns = 1)

ReopenV3PreservesEditedPriorities ==
    diskVersion = 3 => rows = lastV3Rows

NoPriorityInV2 ==
    diskVersion = 2 => \A r \in RowIds : rows[r].priority = "None"

AllPresentRowsHavePriorityInV3 ==
    diskVersion = 3 => \A r \in RowIds : rows[r].present => rows[r].priority \in Priorities

\* ---- Liveness ----

EventuallyUpgradesToV3 ==
    []((diskVersion = 2 /\ ~homeOpen) => <>(diskVersion = 3 /\ indexState = "Ready"))

================================================================================
