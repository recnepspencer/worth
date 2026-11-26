----------------- MODULE StoreM4Witness4ConcurrentPrepArchive -----------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 4 (milestone-4.md section 9.2): concurrent branch
  preparation from one Home frontier, FIFO revalidation/composition, and an
  archive race.

  1 A and B are both prepared on the same (initial) frontier.
  2 A publishes; B's candidate becomes "stale".
  3 B passes FIFO revalidation with its stale candidate (own lifecycle live).
  4 B publishes onto A's frontier: A's component "a1" survives, B gets "b1".
  5 A prepares again, is archived, and its candidate is refused with a typed
    BranchArchived refusal (RefuseStaleCandidate) instead of publishing.
  TLC must violate Witness4NotReached.
*)

VARIABLE w

w4Vars == <<vars, w>>

Progress ==
    CASE w = 0 /\ prep'["A"] = "current" /\ prep'["B"] = "current"
               /\ head'["A"] = "base" /\ head'["B"] = "base"               -> 1
      [] w = 1 /\ head["A"] = "base" /\ head'["A"] = "a1"
               /\ prep["B"] = "current" /\ prep'["B"] = "stale"            -> 2
      [] w = 2 /\ pendingBranch = NoBranch /\ pendingBranch' = "B"
               /\ prep["B"] = "stale"                                      -> 3
      [] w = 3 /\ head["B"] = "base" /\ head'["B"] = "b1"
               /\ head'["A"] = "a1"                                        -> 4
      [] w = 4 /\ ~crashed /\ ~crashed'
               /\ branchState["A"] = "archived"
               /\ prep["A"] # "none" /\ prep'["A"] = "none"
               /\ pendingBranch' = NoBranch                                -> 5
      [] OTHER -> w

W4Init == Init /\ w = 0

W4Next == Next /\ w' = Progress

W4Spec == W4Init /\ [][W4Next]_w4Vars

Witness4NotReached == w < 5

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W4Focus ==
    /\ crashCount = 0
    /\ pinState = "None"
    /\ tokenBasis = "none"
    /\ inflight = {}
    /\ \A e \in Extents : phase[e] = "none"
    /\ checkpointEpoch = 0

=============================================================================
