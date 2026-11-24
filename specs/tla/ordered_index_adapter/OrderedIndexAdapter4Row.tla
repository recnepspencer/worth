---------------------- MODULE OrderedIndexAdapter4Row ----------------------
EXTENDS OrderedIndexAdapter, TLC

(*
  Bounded 4-row AVL structural & deletion-rebalance witness profile.

  With MaxRows = 4 and MaxCoord = 1, TLC exhaustively explores all 4-node AVL
  tree topologies and splits Delete into:
  - DeleteWithRemoveMinRecursion: 2-child deletion whose right subtree has a
    non-null left child (forcing recursive descent in `RemoveMin`).
  - DeleteWithRotation: deletion whose pre-rotation balance factor exceeds 1
    (forcing single or double AVL rotation + summary propagation during delete).
  - DeleteOrdinary: all other deletions.

  All 8 safety invariants from OrderedIndexAdapter are checked over every
  reachable 4-row state, and first-hit witnesses print confirmation when TLC
  reaches each deletion path.
*)

Init4Row ==
    /\ TLCSet(1, 0)
    /\ TLCSet(2, 0)
    /\ rows = [r \in RowIds |-> AbsentRow]
    /\ lifecycle = "Ready"
    /\ tree = EmptyTree
    /\ watchActive = FALSE
    /\ titleWatchProbe = NoTitleProbe
    /\ focusWatchProbe = AbsentInterval
    /\ titleWatchPending = FALSE
    /\ focusWatchPending = FALSE
    /\ titleWatchDelivered = {}
    /\ focusWatchDelivered = {}

TriggersRemoveMinRecursion(T, r) ==
    /\ T.left[r] /= Null
    /\ T.right[r] /= Null
    /\ T.left[T.right[r]] /= Null

TriggersDeleteRotation(T, r) ==
    /\ T.root /= Null
    /\ LET root == T.root
           lh == NodeHeight(T, T.left[root])
           rh == NodeHeight(T, T.right[root])
       IN \/ (lh = 1 /\ rh = 2 /\ r = T.left[root])
          \/ (rh = 1 /\ lh = 2 /\ r = T.right[root])

DeleteWithRemoveMinRecursion(r) ==
    /\ r \in RowIds
    /\ rows[r] /= AbsentRow
    /\ TriggersRemoveMinRecursion(tree, r)
    /\ DeleteRow(r)
    /\ LET prev == TLCGet(1)
       IN /\ TLCSet(1, prev + 1)
          /\ (prev = 0 => PrintT(<<"WITNESS_REACHED", "DeleteWithRemoveMinRecursion", r>>))

DeleteWithRotation(r) ==
    /\ r \in RowIds
    /\ rows[r] /= AbsentRow
    /\ ~TriggersRemoveMinRecursion(tree, r)
    /\ TriggersDeleteRotation(tree, r)
    /\ DeleteRow(r)
    /\ LET prev == TLCGet(2)
       IN /\ TLCSet(2, prev + 1)
          /\ (prev = 0 => PrintT(<<"WITNESS_REACHED", "DeleteWithRotation", r>>))

DeleteOrdinary(r) ==
    /\ r \in RowIds
    /\ rows[r] /= AbsentRow
    /\ ~TriggersRemoveMinRecursion(tree, r)
    /\ ~TriggersDeleteRotation(tree, r)
    /\ DeleteRow(r)

Next4Row ==
    \/ \E r \in RowIds, val \in ValidRows : InsertRow(r, val)
    \/ \E r \in RowIds, val \in ValidRows : UpdateRow(r, val)
    \/ \E r \in RowIds : DeleteWithRemoveMinRecursion(r)
    \/ \E r \in RowIds : DeleteWithRotation(r)
    \/ \E r \in RowIds : DeleteOrdinary(r)

Spec4Row == Init4Row /\ [][Next4Row]_vars

=============================================================================
