------------------------ MODULE OrderedIndexAdapter ------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Full-stack formal specification of the WORTH Standard Checked Ordered
  Managed-Index Adapter (`worth_query_host::facade::ordered_index`).

  Models:
  - Authoritative source rows mapping RowId -> Absent | [lo, hi] (covering
    both ordered point/prefix keys where lo = hi and interval keys [lo, hi]).
  - Height-balanced binary source tree (AVL) with associative subtree
    summaries (`[lo, hi]` envelope merged bottom-up as Merge(own, left, right)).
  - Incremental Insert / Update (remove-before then insert-after) / Delete
    with summary-delta propagation across rotations and ancestors.
  - Top-down pruned query traversal (`PrunedQuery`) verified against a
    brute-force oracle scan (`OracleQuery`), proving pruning soundness
    (`Matches(probe, entry) => PruneAccepts(probe, ancestor_summary)`).
  - Cold reopen lifecycle (`Created` -> `NotReady` refusal -> explicit
    `RebuildIndex` -> `Ready`) and watch delivery consistency.
  - Metered budget exhaustion (`RefuseBudgetExhausted`) preserving exact
    pre-state atomicity (zero half-mutated rows, tree, lifecycle, or watch).
*)

CONSTANTS
    MaxRows,        \* Row identifiers are 1..MaxRows (0 is Null)
    MaxCoord        \* Coordinates are 0..MaxCoord

ASSUME MaxRows >= 2
ASSUME MaxCoord >= 1

Null == 0
RowIds == 1..MaxRows
NodePtrs == RowIds \cup {Null}
Coords == 0..MaxCoord
Intervals == { [lo |-> a, hi |-> b] : a \in Coords, b \in Coords }
ValidIntervals == { iv \in Intervals : iv.lo <= iv.hi }
Absent == [lo |-> -1, hi |-> -1]
RowValues == ValidIntervals \cup {Absent}
Lifecycles == {"Created", "Ready"}

Min(a, b) == IF a <= b THEN a ELSE b
Max(a, b) == IF a >= b THEN a ELSE b

CombineSummary(s1, s2) ==
    [lo |-> Min(s1.lo, s2.lo), hi |-> Max(s1.hi, s2.hi)]

Encloses(outer, inner) ==
    /\ outer.lo <= inner.lo
    /\ inner.hi <= outer.hi

Matches(probe, val) ==
    /\ val /= Absent
    /\ val.lo <= probe.hi
    /\ probe.lo <= val.hi

PruneAccepts(probe, summary) ==
    /\ summary /= Absent
    /\ summary.lo <= probe.hi
    /\ probe.lo <= summary.hi

KeyLess(r1, v1, r2, v2) ==
    \/ v1.lo < v2.lo
    \/ (v1.lo = v2.lo /\ v1.hi < v2.hi)
    \/ (v1.lo = v2.lo /\ v1.hi = v2.hi /\ r1 < r2)

EmptyTree ==
    [root    |-> Null,
     left    |-> [r \in RowIds |-> Null],
     right   |-> [r \in RowIds |-> Null],
     height  |-> [r \in RowIds |-> 0],
     summary |-> [r \in RowIds |-> Absent]]

NodeHeight(T, n) == IF n = Null THEN 0 ELSE T.height[n]

RefreshNode(T, n, rows) ==
    LET l == T.left[n]
        r == T.right[n]
        h == 1 + Max(NodeHeight(T, l), NodeHeight(T, r))
        s0 == rows[n]
        s1 == IF l = Null THEN s0 ELSE CombineSummary(s0, T.summary[l])
        s2 == IF r = Null THEN s1 ELSE CombineSummary(s1, T.summary[r])
    IN [T EXCEPT !.height[n] = h, !.summary[n] = s2]

RotateLeft(T, n, rows) ==
    LET p == T.right[n]
        t1 == [T EXCEPT !.right[n] = T.left[p]]
        t2 == RefreshNode(t1, n, rows)
        t3 == [t2 EXCEPT !.left[p] = n]
        t4 == RefreshNode(t3, p, rows)
    IN [tree |-> t4, node |-> p]

RotateRight(T, n, rows) ==
    LET p == T.left[n]
        t1 == [T EXCEPT !.left[n] = T.right[p]]
        t2 == RefreshNode(t1, n, rows)
        t3 == [t2 EXCEPT !.right[p] = n]
        t4 == RefreshNode(t3, p, rows)
    IN [tree |-> t4, node |-> p]

Balance(T, n, rows) ==
    LET lh == NodeHeight(T, T.left[n])
        rh == NodeHeight(T, T.right[n])
    IN IF lh - rh > 1 THEN
           LET c == T.left[n]
           IN IF NodeHeight(T, T.left[c]) < NodeHeight(T, T.right[c]) THEN
                  LET r1 == RotateLeft(T, c, rows)
                      t1 == RefreshNode([r1.tree EXCEPT !.left[n] = r1.node], n, rows)
                  IN RotateRight(t1, n, rows)
              ELSE RotateRight(T, n, rows)
       ELSE IF rh - lh > 1 THEN
           LET c == T.right[n]
           IN IF NodeHeight(T, T.right[c]) < NodeHeight(T, T.left[c]) THEN
                  LET r1 == RotateRight(T, c, rows)
                      t1 == RefreshNode([r1.tree EXCEPT !.right[n] = r1.node], n, rows)
                  IN RotateLeft(t1, n, rows)
              ELSE RotateLeft(T, n, rows)
       ELSE [tree |-> T, node |-> n]

RECURSIVE InsertAt(_, _, _, _)
InsertAt(T, n, r, rows) ==
    IF n = Null THEN
        LET t1 == [T EXCEPT !.left[r] = Null,
                            !.right[r] = Null,
                            !.height[r] = 1,
                            !.summary[r] = rows[r]]
        IN [tree |-> t1, node |-> r]
    ELSE IF KeyLess(r, rows[r], n, rows[n]) THEN
        LET res == InsertAt(T, T.left[n], r, rows)
            t1 == RefreshNode([res.tree EXCEPT !.left[n] = res.node], n, rows)
        IN Balance(t1, n, rows)
    ELSE
        LET res == InsertAt(T, T.right[n], r, rows)
            t1 == RefreshNode([res.tree EXCEPT !.right[n] = res.node], n, rows)
        IN Balance(t1, n, rows)

TreeInsert(T, r, rows) ==
    LET res == InsertAt(T, T.root, r, rows)
    IN [res.tree EXCEPT !.root = res.node]

ClearNode(T, n) ==
    [T EXCEPT !.left[n] = Null,
              !.right[n] = Null,
              !.height[n] = 0,
              !.summary[n] = Absent]

RECURSIVE RemoveMin(_, _, _)
RemoveMin(T, n, rows) ==
    IF T.left[n] = Null THEN
        LET rightChild == T.right[n]
            t1 == ClearNode(T, n)
        IN [tree |-> t1, min |-> n, node |-> rightChild]
    ELSE
        LET res == RemoveMin(T, T.left[n], rows)
            t1 == RefreshNode([res.tree EXCEPT !.left[n] = res.node], n, rows)
            bal == Balance(t1, n, rows)
        IN [tree |-> bal.tree, min |-> res.min, node |-> bal.node]

RECURSIVE RemoveAt(_, _, _, _)
RemoveAt(T, n, r, rows) ==
    IF n = Null THEN [tree |-> T, node |-> Null]
    ELSE IF r = n THEN
        IF T.left[n] = Null THEN
            [tree |-> ClearNode(T, n), node |-> T.right[n]]
        ELSE IF T.right[n] = Null THEN
            [tree |-> ClearNode(T, n), node |-> T.left[n]]
        ELSE
            LET m == RemoveMin(T, T.right[n], rows)
                succ == m.min
                t1 == ClearNode(m.tree, n)
                t2 == [t1 EXCEPT !.left[succ] = T.left[n],
                                 !.right[succ] = m.node]
                t3 == RefreshNode(t2, succ, rows)
            IN Balance(t3, succ, rows)
    ELSE IF KeyLess(r, rows[r], n, rows[n]) THEN
        LET res == RemoveAt(T, T.left[n], r, rows)
            t1 == RefreshNode([res.tree EXCEPT !.left[n] = res.node], n, rows)
        IN Balance(t1, n, rows)
    ELSE
        LET res == RemoveAt(T, T.right[n], r, rows)
            t1 == RefreshNode([res.tree EXCEPT !.right[n] = res.node], n, rows)
        IN Balance(t1, n, rows)

TreeRemove(T, r, rows) ==
    LET res == RemoveAt(T, T.root, r, rows)
    IN [res.tree EXCEPT !.root = res.node]

RECURSIVE BuildUpTo(_, _, _)
BuildUpTo(T, r, rows) ==
    IF r = 0 THEN T
    ELSE LET prev == BuildUpTo(T, r - 1, rows)
         IN IF rows[r] = Absent THEN prev ELSE TreeInsert(prev, r, rows)

RebuildTree(rows) == BuildUpTo(EmptyTree, MaxRows, rows)

LiveRows(rows) == { r \in RowIds : rows[r] /= Absent }

OracleQuery(rows, probe) ==
    { r \in RowIds : Matches(probe, rows[r]) }

RECURSIVE PrunedQueryAt(_, _, _, _)
PrunedQueryAt(T, n, probe, rows) ==
    IF n = Null THEN {}
    ELSE IF ~PruneAccepts(probe, T.summary[n]) THEN {}
    ELSE (IF Matches(probe, rows[n]) THEN {n} ELSE {})
         \cup PrunedQueryAt(T, T.left[n], probe, rows)
         \cup PrunedQueryAt(T, T.right[n], probe, rows)

PrunedQuery(T, probe, rows) == PrunedQueryAt(T, T.root, probe, rows)

RECURSIVE ReachableAt(_, _, _)
ReachableAt(T, n, fuel) ==
    IF n = Null \/ fuel = 0 THEN {}
    ELSE {n} \cup ReachableAt(T, T.left[n], fuel - 1)
             \cup ReachableAt(T, T.right[n], fuel - 1)

Reachable(T) == ReachableAt(T, T.root, MaxRows + 1)

Subtree(T, n) == ReachableAt(T, n, MaxRows + 1)

\* ---------------------------------------------------------------------------
\* State variables
\* ---------------------------------------------------------------------------

VARIABLES
    rows,           \* Authoritative source rows: [RowIds -> RowValues]
    lifecycle,      \* "Created" (cold, unbuilt) or "Ready" (checked tree live)
    tree,           \* Derived AVL tree with associative summaries
    watchActive,    \* TRUE iff a live watch is registered
    watchProbe,     \* Active watch's probe interval (Absent when inactive)
    watchPending,   \* TRUE iff mutations occurred since last delivery
    watchDelivered  \* Last delivered matching row set

vars == <<rows, lifecycle, tree, watchActive, watchProbe, watchPending, watchDelivered>>

Init ==
    /\ rows = [r \in RowIds |-> Absent]
    /\ lifecycle = "Created"
    /\ tree = EmptyTree
    /\ watchActive = FALSE
    /\ watchProbe = Absent
    /\ watchPending = FALSE
    /\ watchDelivered = {}

\* ---------------------------------------------------------------------------
\* Actions
\* ---------------------------------------------------------------------------

\* 1. Insert a new authoritative row. If the index is Ready, incrementally
\*    insert into the AVL tree and mark active watch pending.
InsertRow(r, iv) ==
    /\ r \in RowIds
    /\ iv \in ValidIntervals
    /\ rows[r] = Absent
    /\ LET nextRows == [rows EXCEPT ![r] = iv]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeInsert(tree, r, nextRows)
                     ELSE tree
          /\ watchPending' = IF watchActive /\ lifecycle = "Ready"
                             THEN TRUE
                             ELSE watchPending
          /\ UNCHANGED <<lifecycle, watchActive, watchProbe, watchDelivered>>

\* 2. Update an existing authoritative row: pass 1 removes under `before`
\*    ordering/summaries, pass 2 inserts under `after` ordering/summaries.
UpdateRow(r, iv) ==
    /\ r \in RowIds
    /\ iv \in ValidIntervals
    /\ rows[r] /= Absent
    /\ LET nextRows == [rows EXCEPT ![r] = iv]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeInsert(TreeRemove(tree, r, rows), r, nextRows)
                     ELSE tree
          /\ watchPending' = IF watchActive /\ lifecycle = "Ready"
                             THEN TRUE
                             ELSE watchPending
          /\ UNCHANGED <<lifecycle, watchActive, watchProbe, watchDelivered>>

\* 3. Delete an existing authoritative row.
DeleteRow(r) ==
    /\ r \in RowIds
    /\ rows[r] /= Absent
    /\ LET nextRows == [rows EXCEPT ![r] = Absent]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeRemove(tree, r, rows)
                     ELSE tree
          /\ watchPending' = IF watchActive /\ lifecycle = "Ready"
                             THEN TRUE
                             ELSE watchPending
          /\ UNCHANGED <<lifecycle, watchActive, watchProbe, watchDelivered>>

\* 4. Cold Directory Home reopen: authoritative rows persist, derived tree
\*    is discarded, lifecycle returns to Created, active watches close.
ColdReopen ==
    /\ lifecycle' = "Created"
    /\ tree' = EmptyTree
    /\ watchActive' = FALSE
    /\ watchProbe' = Absent
    /\ watchPending' = FALSE
    /\ watchDelivered' = {}
    /\ UNCHANGED <<rows>>

\* 5. Explicit cold rebuild_index: rebuilds the checked tree from authoritative
\*    rows and transitions Created -> Ready.
RebuildIndex ==
    /\ lifecycle' = "Ready"
    /\ tree' = RebuildTree(rows)
    /\ watchPending' = IF watchActive THEN TRUE ELSE watchPending
    /\ UNCHANGED <<rows, watchActive, watchProbe, watchDelivered>>

\* 6. Query while Created refuses typed NotReady with zero state effects.
RefuseNotReady(probe) ==
    /\ lifecycle = "Created"
    /\ probe \in ValidIntervals
    /\ UNCHANGED vars

\* 7. Ready query executes top-down pruned traversal with zero state mutation.
ExecuteQuery(probe) ==
    /\ lifecycle = "Ready"
    /\ probe \in ValidIntervals
    /\ PrunedQuery(tree, probe, rows) = OracleQuery(rows, probe)
    /\ UNCHANGED vars

\* 8. Register a live watch on a Ready index: delivers initial snapshot.
WatchOpen(probe) ==
    /\ lifecycle = "Ready"
    /\ ~watchActive
    /\ probe \in ValidIntervals
    /\ watchActive' = TRUE
    /\ watchProbe' = probe
    /\ watchPending' = FALSE
    /\ watchDelivered' = PrunedQuery(tree, probe, rows)
    /\ UNCHANGED <<rows, lifecycle, tree>>

\* 9. Deliver the next watch update on a Ready index.
WatchDeliver ==
    /\ lifecycle = "Ready"
    /\ watchActive
    /\ watchPending' = FALSE
    /\ watchDelivered' = PrunedQuery(tree, watchProbe, rows)
    /\ UNCHANGED <<rows, lifecycle, tree, watchActive, watchProbe>>

\* 10. Close an active watch.
WatchClose ==
    /\ watchActive
    /\ watchActive' = FALSE
    /\ watchProbe' = Absent
    /\ watchPending' = FALSE
    /\ watchDelivered' = {}
    /\ UNCHANGED <<rows, lifecycle, tree>>

\* 11. Metered work or memory exhaustion refuses cleanly with ZERO partial mutation.
RefuseBudgetExhausted ==
    /\ UNCHANGED vars

Next ==
    \/ \E r \in RowIds, iv \in ValidIntervals : InsertRow(r, iv)
    \/ \E r \in RowIds, iv \in ValidIntervals : UpdateRow(r, iv)
    \/ \E r \in RowIds : DeleteRow(r)
    \/ ColdReopen
    \/ RebuildIndex
    \/ \E p \in ValidIntervals : RefuseNotReady(p)
    \/ \E p \in ValidIntervals : ExecuteQuery(p)
    \/ \E p \in ValidIntervals : WatchOpen(p)
    \/ WatchDeliver
    \/ WatchClose
    \/ RefuseBudgetExhausted

Spec == Init /\ [][Next]_vars

\* ===========================================================================
\* Safety Invariants & Temporal Properties
\* ===========================================================================

TypeOK ==
    /\ rows \in [RowIds -> RowValues]
    /\ lifecycle \in Lifecycles
    /\ tree.root \in NodePtrs
    /\ tree.left \in [RowIds -> NodePtrs]
    /\ tree.right \in [RowIds -> NodePtrs]
    /\ tree.height \in [RowIds -> 0..MaxRows]
    /\ tree.summary \in [RowIds -> RowValues]
    /\ watchActive \in BOOLEAN
    /\ watchProbe \in RowValues
    /\ watchPending \in BOOLEAN
    /\ watchDelivered \subseteq RowIds

\* 1. Source membership bijection & structural acyclicity/disjointness.
SourceMembershipBijection ==
    IF lifecycle = "Created" THEN
        /\ tree = EmptyTree
        /\ ~watchActive
    ELSE
        LET live == LiveRows(rows)
        IN /\ Reachable(tree) = live
           /\ (live = {}) <=> (tree.root = Null)
           /\ \A r \in RowIds :
                IF r \in live THEN
                    /\ tree.height[r] >= 1
                    /\ tree.summary[r] /= Absent
                    /\ tree.left[r] /= r /\ tree.right[r] /= r
                    /\ (tree.left[r] /= Null => tree.left[r] /= tree.right[r])
                    /\ Subtree(tree, tree.left[r]) \cap Subtree(tree, tree.right[r]) = {}
                    /\ r \notin Subtree(tree, tree.left[r])
                    /\ r \notin Subtree(tree, tree.right[r])
                ELSE
                    /\ tree.left[r] = Null
                    /\ tree.right[r] = Null
                    /\ tree.height[r] = 0
                    /\ tree.summary[r] = Absent

\* 2. Strict BST ordering invariant over (lo, hi, rowId).
BSTOrderingInvariant ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            /\ \A d \in Subtree(tree, tree.left[n]) :
                    KeyLess(d, rows[d], n, rows[n])
            /\ \A d \in Subtree(tree, tree.right[n]) :
                    KeyLess(n, rows[n], d, rows[d])

\* 3. AVL height and balance invariant.
AVLBalanceInvariant ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            LET lh == NodeHeight(tree, tree.left[n])
                rh == NodeHeight(tree, tree.right[n])
            IN /\ tree.height[n] = 1 + Max(lh, rh)
               /\ lh - rh <= 1
               /\ rh - lh <= 1

\* 4. Associative summary exactness & parent-child / leaf enclosure.
SummaryExactnessAndEnclosure ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            LET l == tree.left[n]
                r == tree.right[n]
                s0 == rows[n]
                s1 == IF l = Null THEN s0 ELSE CombineSummary(s0, tree.summary[l])
                expected == IF r = Null THEN s1 ELSE CombineSummary(s1, tree.summary[r])
            IN /\ tree.summary[n] = expected
               /\ Encloses(tree.summary[n], rows[n])
               /\ (l /= Null => Encloses(tree.summary[n], tree.summary[l]))
               /\ (r /= Null => Encloses(tree.summary[n], tree.summary[r]))
               /\ \A d \in Subtree(tree, n) : Encloses(tree.summary[n], rows[d])

\* 5. Pruning soundness & exact equivalence to brute-force oracle scan.
PruningSoundnessAndQueryCompleteness ==
    lifecycle = "Ready" =>
        \A p \in ValidIntervals :
            /\ \A a \in LiveRows(rows) :
                \A d \in Subtree(tree, a) :
                    Matches(p, rows[d]) => PruneAccepts(p, tree.summary[a])
            /\ PrunedQuery(tree, p, rows) = OracleQuery(rows, p)

\* 6. Watch delivery consistency when settled.
WatchDeliveryConsistency ==
    (watchActive /\ ~watchPending) =>
        /\ lifecycle = "Ready"
        /\ watchDelivered = OracleQuery(rows, watchProbe)

\* 7. Cold reopen never retains a live tree.
ColdNotReadySafety ==
    lifecycle = "Created" => tree.root = Null

=============================================================================
