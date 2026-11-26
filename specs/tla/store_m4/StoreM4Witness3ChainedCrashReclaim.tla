------------------ MODULE StoreM4Witness3ChainedCrashReclaim ------------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 3 (milestone-4.md section 9.2, P1): chained crash
  during recovery's reclaim resume, then reuse at g' > g with zero
  double-free across one more crash/reopen.

  1 Crash while "gc" is at ReclaimIntent ("intent").
  2 RecoverResumeFreeSpaceRoot moves "gc" intent -> fsroot.
  3 A SECOND Crash before RecoverResumeReclaimComplete.
  4 The next recovery's RecoverResumeReclaimComplete moves fsroot -> complete.
  5 AdmitToAllocator("gc") after serving.
  6 A writeback lands on reallocated "gc" cells at a generation > <<0,0>>.
  7 A third Crash with those cells reallocated ("fresh").
  8 RecoverServing completes; the cells are still fresh at the same
    generation (no double release; InterruptedReclaimIdempotent holds in
    every state of the main check).
  TLC must violate Witness3NotReached.
*)

VARIABLE w

w3Vars == <<vars, w>>

GcCells == ExtentCells("gc")
FreshGc == {c \in GcCells : cellState[c] = "fresh"}

Progress ==
    CASE w = 0 /\ crashCount' = crashCount + 1 /\ phase["gc"] = "intent"   -> 1
      [] w = 1 /\ m4Rec = "pins_restored" /\ m4Rec' = "fsroot_resumed"
               /\ phase["gc"] = "intent" /\ phase'["gc"] = "fsroot"        -> 2
      [] w = 2 /\ crashCount' = crashCount + 1 /\ m4Rec = "fsroot_resumed"
               /\ phase["gc"] = "fsroot"                                   -> 3
      [] w = 3 /\ m4Rec = "fsroot_resumed" /\ m4Rec' = "complete_resumed"
               /\ phase["gc"] = "fsroot" /\ phase'["gc"] = "complete"      -> 4
      [] w = 4 /\ phase["gc"] = "complete" /\ phase'["gc"] = "admitted"    -> 5
      [] w = 6 /\ crashCount' = crashCount + 1 /\ FreshGc # {}             -> 7
      [] w = 7 /\ crashed /\ ~crashed'
               /\ FreshGc # {}
               /\ \A c \in FreshGc : cellState'[c] = "fresh" /\ cellGen'[c] = cellGen[c]
               /\ phase'["gc"] = "admitted"                                -> 8
      [] OTHER -> w

W3Init == Init /\ w = 0

W3Next ==
    \/ /\ w = 5
       /\ \E x \in inflight :
             /\ x.range \subseteq GcCells
             /\ GenGT(x.gen, OrigGen)
             /\ \A c \in x.range : cellState[c] = "fresh"
             /\ WritebackLand(x)
       /\ w' = 6
    \/ Next /\ w' = Progress

W3Spec == W3Init /\ [][W3Next]_w3Vars

Witness3NotReached == w < 8

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W3Focus ==
    /\ durableHead = 0 /\ pendingWrite = "none"
    /\ pinState = "None"
    /\ tokenBasis = "none"
    /\ \A b \in Branches : branchState[b] = "live" /\ prep[b] = "none"
    /\ phase["base"] = "none" /\ phase["cand"] = "none"

=============================================================================
