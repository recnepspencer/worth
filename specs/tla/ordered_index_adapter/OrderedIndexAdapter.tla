------------------------ MODULE OrderedIndexAdapter ------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Full-stack formal specification of the WORTH Standard Checked Ordered
  Managed-Index Adapter (`worth_query_host::facade::ordered_index`).

  Models:
  - Authoritative source rows mapping RowId -> Absent | [lo, hi, title],
    covering BOTH calendar focus intervals [lo, hi] (with lo <= hi) AND
    ordered title-prefix groups `title \in 0..MaxTitle`.
  - Height-balanced binary source tree (AVL) with associative subtree
    summaries (`[lo, hi]` envelope merged bottom-up as Merge(own, left, right)).
  - Incremental Insert / Update (remove-before then insert-after) / Delete
    with summary-delta propagation across rotations and ancestors.
  - Top-down pruned query traversal (`PrunedQuery`) verified against a
    brute-force oracle scan (`OracleFocusQuery`), plus `OracleTitleQuery`.
  - Cold reopen lifecycle (`Created` -> typed `NotReady` refusal -> explicit
    `RebuildIndex` -> `Ready`).
  - Exact inverse watch routing: mutations mark a watch pending ONLY when
    the `before` or `after` row intersects that watch's probe, and delivery
    requires `watchPending = TRUE`.
  - Same-host metered budget exhaustion (`RefuseBudgetExhausted(r, val)`)
    bound to an attempted Ready warm update, preserving exact pre-state
    atomicity across rows, tree, lifecycle, and watches.
*)

CONSTANTS
    MaxRows,        \* Row identifiers are 1..MaxRows (0 is Null)
    MaxCoord,       \* Interval coordinates are 0..MaxCoord
    MaxTitle        \* Title prefix groups are 0..MaxTitle

ASSUME MaxRows >= 2
ASSUME MaxCoord >= 1
ASSUME MaxTitle >= 0

Null == 0
RowIds == 1..MaxRows
NodePtrs == RowIds \cup {Null}
Coords == 0..MaxCoord
TitleGroups == 0..MaxTitle
TitleProbes == TitleGroups \cup {-1, 99}
NoTitleProbe == -2
Intervals == { [lo |-> a, hi |-> b] : a \in Coords, b \in Coords }
ValidIntervals == { iv \in Intervals : iv.lo <= iv.hi }
AbsentInterval == [lo |-> -1, hi |-> -1]
ValidRows ==
    { [lo |-> iv.lo, hi |-> iv.hi, title |-> t] : iv \in ValidIntervals, t \in TitleGroups }
AbsentRow == [lo |-> -1, hi |-> -1, title |-> -1]
RowValues == ValidRows \cup {AbsentRow}
SummaryValues == ValidIntervals \cup {AbsentInterval}
Lifecycles == {"Created", "Ready"}

IsValidInterval(iv) ==
    /\ iv.lo \in Coords
    /\ iv.hi \in Coords
    /\ iv.lo <= iv.hi

IsSummaryValue(s) ==
    s = AbsentInterval \/ IsValidInterval(s)

IsValidRow(v) ==
    /\ v.lo \in Coords
    /\ v.hi \in Coords
    /\ v.lo <= v.hi
    /\ v.title \in TitleGroups

IsRowValue(v) ==
    v = AbsentRow \/ IsValidRow(v)

Min(a, b) == IF a <= b THEN a ELSE b
Max(a, b) == IF a >= b THEN a ELSE b

RowInterval(val) ==
    IF val = AbsentRow THEN AbsentInterval ELSE [lo |-> val.lo, hi |-> val.hi]

CombineSummary(s1, s2) ==
    [lo |-> Min(s1.lo, s2.lo), hi |-> Max(s1.hi, s2.hi)]

Encloses(outer, inner) ==
    /\ outer.lo <= inner.lo
    /\ inner.hi <= outer.hi

FocusMatches(probe, val) ==
    /\ val /= AbsentRow
    /\ val.lo <= val.hi
    /\ probe.lo <= probe.hi
    /\ val.lo <= probe.hi
    /\ probe.lo <= val.hi

TitleMatches(tProbe, val) ==
    /\ val /= AbsentRow
    /\ (tProbe = -1 \/ val.title = tProbe)

PruneAccepts(probe, summary) ==
    /\ summary /= AbsentInterval
    /\ summary.lo <= summary.hi
    /\ probe.lo <= probe.hi
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
     summary |-> [r \in RowIds |-> AbsentInterval]]

NodeHeight(T, n) == IF n = Null THEN 0 ELSE T.height[n]

RefreshNode(T, n, rows) ==
    LET l == T.left[n]
        r == T.right[n]
        h == 1 + Max(NodeHeight(T, l), NodeHeight(T, r))
        s0 == RowInterval(rows[n])
        s1 == IF l = Null THEN s0 ELSE CombineSummary(s0, T.summary[l])
        s2 == IF r = Null THEN s1 ELSE CombineSummary(s1, T.summary[r])
    IN [T EXCEPT !.height[n] = h, !.summary[n] = s2]

RotateLeftStep(T, n, p, rows) ==
    [tree |-> RefreshNode([T EXCEPT !.left[p] = n], p, rows), node |-> p]

RotateLeft(T, n, rows) ==
    LET p == T.right[n]
    IN RotateLeftStep(RefreshNode([T EXCEPT !.right[n] = T.left[p]], n, rows), n, p, rows)

RotateRightStep(T, n, p, rows) ==
    [tree |-> RefreshNode([T EXCEPT !.right[p] = n], p, rows), node |-> p]

RotateRight(T, n, rows) ==
    LET p == T.left[n]
    IN RotateRightStep(RefreshNode([T EXCEPT !.left[n] = T.right[p]], n, rows), n, p, rows)

FinishLR(r1, n, rows) ==
    RotateRight(RefreshNode([r1.tree EXCEPT !.left[n] = r1.node], n, rows), n, rows)

FinishRL(r1, n, rows) ==
    RotateLeft(RefreshNode([r1.tree EXCEPT !.right[n] = r1.node], n, rows), n, rows)

Balance(T, n, rows) ==
    LET lh == NodeHeight(T, T.left[n])
        rh == NodeHeight(T, T.right[n])
    IN IF lh - rh > 1 THEN
           LET c == T.left[n]
           IN IF NodeHeight(T, T.left[c]) < NodeHeight(T, T.right[c]) THEN
                  FinishLR(RotateLeft(T, c, rows), n, rows)
              ELSE RotateRight(T, n, rows)
       ELSE IF rh - lh > 1 THEN
           LET c == T.right[n]
           IN IF NodeHeight(T, T.right[c]) < NodeHeight(T, T.left[c]) THEN
                  FinishRL(RotateRight(T, c, rows), n, rows)
              ELSE RotateLeft(T, n, rows)
       ELSE [tree |-> T, node |-> n]

AttachLeftAndBalance(res, n, rows) ==
    Balance(RefreshNode([res.tree EXCEPT !.left[n] = res.node], n, rows), n, rows)

AttachRightAndBalance(res, n, rows) ==
    Balance(RefreshNode([res.tree EXCEPT !.right[n] = res.node], n, rows), n, rows)

FinishTreeRoot(res) ==
    [res.tree EXCEPT !.root = res.node]

RECURSIVE InsertAt(_, _, _, _)
InsertAt(T, n, r, rows) ==
    IF n = Null THEN
        [tree |-> [T EXCEPT !.left[r] = Null,
                            !.right[r] = Null,
                            !.height[r] = 1,
                            !.summary[r] = RowInterval(rows[r])],
         node |-> r]
    ELSE IF KeyLess(r, rows[r], n, rows[n]) THEN
        AttachLeftAndBalance(InsertAt(T, T.left[n], r, rows), n, rows)
    ELSE
        AttachRightAndBalance(InsertAt(T, T.right[n], r, rows), n, rows)

TreeInsert(T, r, rows) ==
    FinishTreeRoot(InsertAt(T, T.root, r, rows))

ClearNode(T, n) ==
    [T EXCEPT !.left[n] = Null,
              !.right[n] = Null,
              !.height[n] = 0,
              !.summary[n] = AbsentInterval]

PackMinBalance(bal, minNode) ==
    [tree |-> bal.tree, min |-> minNode, node |-> bal.node]

AttachMinLeftAndBalance(res, n, rows) ==
    PackMinBalance(
        Balance(RefreshNode([res.tree EXCEPT !.left[n] = res.node], n, rows), n, rows),
        res.min)

RECURSIVE RemoveMin(_, _, _)
RemoveMin(T, n, rows) ==
    IF T.left[n] = Null THEN
        [tree |-> ClearNode(T, n), min |-> n, node |-> T.right[n]]
    ELSE
        AttachMinLeftAndBalance(RemoveMin(T, T.left[n], rows), n, rows)

TransplantSuccessor(m, T, n, rows) ==
    Balance(
        RefreshNode(
            [ClearNode(m.tree, n) EXCEPT !.left[m.min] = T.left[n],
                                         !.right[m.min] = m.node],
            m.min,
            rows),
        m.min,
        rows)

RECURSIVE RemoveAt(_, _, _, _)
RemoveAt(T, n, r, rows) ==
    IF n = Null THEN [tree |-> T, node |-> Null]
    ELSE IF r = n THEN
        IF T.left[n] = Null THEN
            [tree |-> ClearNode(T, n), node |-> T.right[n]]
        ELSE IF T.right[n] = Null THEN
            [tree |-> ClearNode(T, n), node |-> T.left[n]]
        ELSE
            TransplantSuccessor(RemoveMin(T, T.right[n], rows), T, n, rows)
    ELSE IF KeyLess(r, rows[r], n, rows[n]) THEN
        AttachLeftAndBalance(RemoveAt(T, T.left[n], r, rows), n, rows)
    ELSE
        AttachRightAndBalance(RemoveAt(T, T.right[n], r, rows), n, rows)

TreeRemove(T, r, rows) ==
    FinishTreeRoot(RemoveAt(T, T.root, r, rows))

BuildStep(prev, r, rows) ==
    IF rows[r] = AbsentRow THEN prev ELSE TreeInsert(prev, r, rows)

RECURSIVE BuildUpTo(_, _, _)
BuildUpTo(T, r, rows) ==
    IF r = 0 THEN T
    ELSE BuildStep(BuildUpTo(T, r - 1, rows), r, rows)

RebuildTree(rows) == BuildUpTo(EmptyTree, MaxRows, rows)

LiveRows(rows) == { r \in RowIds : rows[r] /= AbsentRow }

OracleFocusQuery(rows, probe) ==
    { r \in RowIds : FocusMatches(probe, rows[r]) }

OracleTitleQuery(rows, tProbe) ==
    { r \in RowIds : TitleMatches(tProbe, rows[r]) }

RECURSIVE PrunedQueryAt(_, _, _, _)
PrunedQueryAt(T, n, probe, rows) ==
    IF n = Null THEN {}
    ELSE IF ~PruneAccepts(probe, T.summary[n]) THEN {}
    ELSE (IF FocusMatches(probe, rows[n]) THEN {n} ELSE {})
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
    rows,                 \* Authoritative source rows: [RowIds -> RowValues]
    lifecycle,            \* "Created" (cold, unbuilt) or "Ready" (checked tree live)
    tree,                 \* Derived AVL tree with associative summaries
    watchActive,          \* TRUE iff live watches are registered
    titleWatchProbe,      \* Active title watch probe (NoTitleProbe when inactive)
    focusWatchProbe,      \* Active focus watch probe (AbsentInterval when inactive)
    titleWatchPending,    \* TRUE iff a matching mutation occurred since last title delivery
    focusWatchPending,    \* TRUE iff a matching mutation occurred since last focus delivery
    titleWatchDelivered,  \* Last delivered title-watch matching row set
    focusWatchDelivered   \* Last delivered focus-watch matching row set

vars == <<rows, lifecycle, tree, watchActive,
          titleWatchProbe, focusWatchProbe,
          titleWatchPending, focusWatchPending,
          titleWatchDelivered, focusWatchDelivered>>

Init ==
    /\ rows = [r \in RowIds |-> AbsentRow]
    /\ lifecycle = "Created"
    /\ tree = EmptyTree
    /\ watchActive = FALSE
    /\ titleWatchProbe = NoTitleProbe
    /\ focusWatchProbe = AbsentInterval
    /\ titleWatchPending = FALSE
    /\ focusWatchPending = FALSE
    /\ titleWatchDelivered = {}
    /\ focusWatchDelivered = {}

\* ---------------------------------------------------------------------------
\* Actions
\* ---------------------------------------------------------------------------

\* 1. Insert a new authoritative row. Marks each active watch pending ONLY if
\*    the inserted row intersects that watch's probe.
InsertRow(r, val) ==
    /\ r \in RowIds
    /\ IsValidRow(val)
    /\ rows[r] = AbsentRow
    /\ LET nextRows == [rows EXCEPT ![r] = val]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeInsert(tree, r, nextRows)
                     ELSE tree
          /\ titleWatchPending' =
                IF watchActive /\ lifecycle = "Ready" /\ TitleMatches(titleWatchProbe, val)
                THEN TRUE ELSE titleWatchPending
          /\ focusWatchPending' =
                IF watchActive /\ lifecycle = "Ready" /\ FocusMatches(focusWatchProbe, val)
                THEN TRUE ELSE focusWatchPending
          /\ UNCHANGED <<lifecycle, watchActive, titleWatchProbe, focusWatchProbe,
                         titleWatchDelivered, focusWatchDelivered>>

\* 2. Update an existing authoritative row: pass 1 removes under `before`
\*    ordering/summaries, pass 2 inserts under `after` ordering/summaries.
\*    Marks a watch pending ONLY if `before` OR `after` intersects its probe.
UpdateRow(r, val) ==
    /\ r \in RowIds
    /\ IsValidRow(val)
    /\ rows[r] /= AbsentRow
    /\ LET nextRows == [rows EXCEPT ![r] = val]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeInsert(TreeRemove(tree, r, rows), r, nextRows)
                     ELSE tree
          /\ titleWatchPending' =
                IF watchActive /\ lifecycle = "Ready"
                   /\ (TitleMatches(titleWatchProbe, rows[r]) \/ TitleMatches(titleWatchProbe, val))
                THEN TRUE ELSE titleWatchPending
          /\ focusWatchPending' =
                IF watchActive /\ lifecycle = "Ready"
                   /\ (FocusMatches(focusWatchProbe, rows[r]) \/ FocusMatches(focusWatchProbe, val))
                THEN TRUE ELSE focusWatchPending
          /\ UNCHANGED <<lifecycle, watchActive, titleWatchProbe, focusWatchProbe,
                         titleWatchDelivered, focusWatchDelivered>>

\* 3. Delete an existing authoritative row. Marks a watch pending ONLY if the
\*    deleted `before` row intersected that watch's probe.
DeleteRow(r) ==
    /\ r \in RowIds
    /\ rows[r] /= AbsentRow
    /\ LET nextRows == [rows EXCEPT ![r] = AbsentRow]
       IN /\ rows' = nextRows
          /\ tree' = IF lifecycle = "Ready"
                     THEN TreeRemove(tree, r, rows)
                     ELSE tree
          /\ titleWatchPending' =
                IF watchActive /\ lifecycle = "Ready" /\ TitleMatches(titleWatchProbe, rows[r])
                THEN TRUE ELSE titleWatchPending
          /\ focusWatchPending' =
                IF watchActive /\ lifecycle = "Ready" /\ FocusMatches(focusWatchProbe, rows[r])
                THEN TRUE ELSE focusWatchPending
          /\ UNCHANGED <<lifecycle, watchActive, titleWatchProbe, focusWatchProbe,
                         titleWatchDelivered, focusWatchDelivered>>

\* 4. Cold Directory Home reopen: authoritative rows persist, derived tree
\*    is discarded, lifecycle returns to Created, active watches close.
ColdReopen ==
    /\ lifecycle' = "Created"
    /\ tree' = EmptyTree
    /\ watchActive' = FALSE
    /\ titleWatchProbe' = NoTitleProbe
    /\ focusWatchProbe' = AbsentInterval
    /\ titleWatchPending' = FALSE
    /\ focusWatchPending' = FALSE
    /\ titleWatchDelivered' = {}
    /\ focusWatchDelivered' = {}
    /\ UNCHANGED <<rows>>

\* 5. Explicit cold rebuild_index: rebuilds the checked tree from authoritative
\*    rows and transitions Created -> Ready.
RebuildIndex ==
    /\ lifecycle' = "Ready"
    /\ tree' = RebuildTree(rows)
    /\ UNCHANGED <<rows, watchActive, titleWatchProbe, focusWatchProbe,
                   titleWatchPending, focusWatchPending,
                   titleWatchDelivered, focusWatchDelivered>>

\* 6. Query while Created refuses typed NotReady with zero state effects.
RefuseNotReady(tProbe, fProbe) ==
    /\ lifecycle = "Created"
    /\ tProbe \in TitleProbes
    /\ IsValidInterval(fProbe)
    /\ UNCHANGED vars

\* 7. Ready query executes top-down pruned traversal with zero state mutation.
ExecuteQuery(tProbe, fProbe) ==
    /\ lifecycle = "Ready"
    /\ tProbe \in TitleProbes
    /\ IsValidInterval(fProbe)
    /\ PrunedQuery(tree, fProbe, rows) = OracleFocusQuery(rows, fProbe)
    /\ UNCHANGED vars

\* 8. Register live watches on a Ready index: delivers initial snapshots in Idle state.
WatchOpen(tProbe, fProbe) ==
    /\ lifecycle = "Ready"
    /\ ~watchActive
    /\ tProbe \in TitleProbes
    /\ IsValidInterval(fProbe)
    /\ watchActive' = TRUE
    /\ titleWatchProbe' = tProbe
    /\ focusWatchProbe' = fProbe
    /\ titleWatchPending' = FALSE
    /\ focusWatchPending' = FALSE
    /\ titleWatchDelivered' = OracleTitleQuery(rows, tProbe)
    /\ focusWatchDelivered' = PrunedQuery(tree, fProbe, rows)
    /\ UNCHANGED <<rows, lifecycle, tree>>

\* 9a. Deliver a pending title watch notification (strictly requires titleWatchPending).
WatchDeliverTitle ==
    /\ lifecycle = "Ready"
    /\ watchActive
    /\ titleWatchPending = TRUE
    /\ titleWatchPending' = FALSE
    /\ titleWatchDelivered' = OracleTitleQuery(rows, titleWatchProbe)
    /\ UNCHANGED <<rows, lifecycle, tree, watchActive, titleWatchProbe, focusWatchProbe,
                   focusWatchPending, focusWatchDelivered>>

\* 9b. Deliver a pending focus watch notification (strictly requires focusWatchPending).
WatchDeliverFocus ==
    /\ lifecycle = "Ready"
    /\ watchActive
    /\ focusWatchPending = TRUE
    /\ focusWatchPending' = FALSE
    /\ focusWatchDelivered' = PrunedQuery(tree, focusWatchProbe, rows)
    /\ UNCHANGED <<rows, lifecycle, tree, watchActive, titleWatchProbe, focusWatchProbe,
                   titleWatchPending, titleWatchDelivered>>

\* 9c. Poll active watches when neither watch has a pending notification.
WatchPollIdle ==
    /\ lifecycle = "Ready"
    /\ watchActive
    /\ ~titleWatchPending
    /\ ~focusWatchPending
    /\ UNCHANGED vars

\* 10. Close active watches.
WatchClose ==
    /\ watchActive
    /\ watchActive' = FALSE
    /\ titleWatchProbe' = NoTitleProbe
    /\ focusWatchProbe' = AbsentInterval
    /\ titleWatchPending' = FALSE
    /\ focusWatchPending' = FALSE
    /\ titleWatchDelivered' = {}
    /\ focusWatchDelivered' = {}
    /\ UNCHANGED <<rows, lifecycle, tree>>

\* 11. Same-host metered budget exhaustion during a Ready warm Update refuses
\*     cleanly with ZERO partial mutation across rows, AVL tree, lifecycle, or watches.
RefuseBudgetExhausted(r, val) ==
    /\ lifecycle = "Ready"
    /\ r \in RowIds
    /\ rows[r] /= AbsentRow
    /\ IsValidRow(val)
    /\ val /= rows[r]
    /\ UNCHANGED vars

Next ==
    \/ \E r \in RowIds, val \in ValidRows : InsertRow(r, val)
    \/ \E r \in RowIds, val \in ValidRows : UpdateRow(r, val)
    \/ \E r \in RowIds : DeleteRow(r)
    \/ ColdReopen
    \/ RebuildIndex
    \/ \E tp \in TitleProbes, fp \in ValidIntervals : RefuseNotReady(tp, fp)
    \/ \E tp \in TitleProbes, fp \in ValidIntervals : ExecuteQuery(tp, fp)
    \/ \E tp \in TitleProbes, fp \in ValidIntervals : WatchOpen(tp, fp)
    \/ WatchDeliverTitle
    \/ WatchDeliverFocus
    \/ WatchPollIdle
    \/ WatchClose
    \/ \E r \in RowIds, val \in ValidRows : RefuseBudgetExhausted(r, val)

Spec == Init /\ [][Next]_vars

\* ===========================================================================
\* Safety Invariants
\* ===========================================================================

TypeOK ==
    /\ DOMAIN rows = RowIds
    /\ \A r \in RowIds : IsRowValue(rows[r])
    /\ lifecycle \in Lifecycles
    /\ tree.root \in NodePtrs
    /\ DOMAIN tree.left = RowIds /\ \A r \in RowIds : tree.left[r] \in NodePtrs
    /\ DOMAIN tree.right = RowIds /\ \A r \in RowIds : tree.right[r] \in NodePtrs
    /\ DOMAIN tree.height = RowIds /\ \A r \in RowIds : tree.height[r] \in 0..MaxRows
    /\ DOMAIN tree.summary = RowIds /\ \A r \in RowIds : IsSummaryValue(tree.summary[r])
    /\ watchActive \in BOOLEAN
    /\ titleWatchProbe \in (TitleProbes \cup {NoTitleProbe})
    /\ IsSummaryValue(focusWatchProbe)
    /\ titleWatchPending \in BOOLEAN
    /\ focusWatchPending \in BOOLEAN
    /\ titleWatchDelivered \subseteq RowIds
    /\ focusWatchDelivered \subseteq RowIds

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
                    /\ tree.summary[r] /= AbsentInterval
                    /\ tree.left[r] /= r /\ tree.right[r] /= r
                    /\ (tree.left[r] /= Null => tree.left[r] /= tree.right[r])
                    /\ Subtree(tree, tree.left[r]) \cap Subtree(tree, tree.right[r]) = {}
                    /\ r \notin Subtree(tree, tree.left[r])
                    /\ r \notin Subtree(tree, tree.right[r])
                ELSE
                    /\ tree.left[r] = Null
                    /\ tree.right[r] = Null
                    /\ tree.height[r] = 0
                    /\ tree.summary[r] = AbsentInterval

BSTOrderingInvariant ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            /\ \A d \in Subtree(tree, tree.left[n]) :
                    KeyLess(d, rows[d], n, rows[n])
            /\ \A d \in Subtree(tree, tree.right[n]) :
                    KeyLess(n, rows[n], d, rows[d])

AVLBalanceInvariant ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            LET lh == NodeHeight(tree, tree.left[n])
                rh == NodeHeight(tree, tree.right[n])
            IN /\ tree.height[n] = 1 + Max(lh, rh)
               /\ lh - rh <= 1
               /\ rh - lh <= 1

SummaryExactnessAndEnclosure ==
    lifecycle = "Ready" =>
        \A n \in LiveRows(rows) :
            LET l == tree.left[n]
                r == tree.right[n]
                s0 == RowInterval(rows[n])
                s1 == IF l = Null THEN s0 ELSE CombineSummary(s0, tree.summary[l])
                expected == IF r = Null THEN s1 ELSE CombineSummary(s1, tree.summary[r])
            IN /\ tree.summary[n] = expected
               /\ Encloses(tree.summary[n], RowInterval(rows[n]))
               /\ (l /= Null => Encloses(tree.summary[n], tree.summary[l]))
               /\ (r /= Null => Encloses(tree.summary[n], tree.summary[r]))
               /\ \A d \in Subtree(tree, n) : Encloses(tree.summary[n], RowInterval(rows[d]))

PruningSoundnessAndQueryCompleteness ==
    lifecycle = "Ready" =>
        LET live == LiveRows(rows)
        IN /\ \A a \in live :
                \A d \in Subtree(tree, a) :
                    \A p \in ValidIntervals :
                        FocusMatches(p, rows[d]) => PruneAccepts(p, tree.summary[a])
           /\ \A p \in ValidIntervals :
                PrunedQuery(tree, p, rows) = OracleFocusQuery(rows, p)

WatchDeliveryConsistency ==
    /\ (watchActive /\ ~titleWatchPending) =>
        /\ lifecycle = "Ready"
        /\ titleWatchDelivered = OracleTitleQuery(rows, titleWatchProbe)
    /\ (watchActive /\ ~focusWatchPending) =>
        /\ lifecycle = "Ready"
        /\ focusWatchDelivered = OracleFocusQuery(rows, focusWatchProbe)

ColdNotReadySafety ==
    lifecycle = "Created" => tree.root = Null

=============================================================================
