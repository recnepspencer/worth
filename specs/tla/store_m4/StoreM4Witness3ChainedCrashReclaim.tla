------------------ MODULE StoreM4Witness3ChainedCrashReclaim ------------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 3 (milestone-4.md section 9.2, P1): chained crash
  during recovery's reclaim resume, then reuse at g' > g with zero
  double-free across one more crash/reopen.

  1 Crash while "gc" is at ReclaimIntent ("intent").
  2 RecoverResumeFreeSpaceRoot moves "gc" intent -> fsroot.
  3 A SECOND Crash before RecoverResumeReclaimComplete; "gc" becomes
    "fsroot_reopened" because bootstrap on reopen already loads its published
    free-space root into the allocator.
  4 The next recovery's RecoverResumeReclaimComplete moves "gc" directly
    fsroot_reopened -> admitted (matching Verus resume_release(FreeSpacePublished)
    admit_to_allocator = false and Lean ReclamationSafety, with no second
    AdmitToAllocator).
  5 After RecoverServing (~ENABLED AdmitToAllocator("gc")), a writeback lands
    on reallocated "gc" cells at a generation > <<0,0>>.
  6 A third Crash with those cells reallocated ("fresh").
  7 RecoverServing completes; the cells are still fresh at the same
    generation (no double release; InterruptedReclaimIdempotent holds in
    every state).
  TLC must violate Witness3NotReached.
*)

VARIABLE w

w3Vars == <<vars, w>>

GcCells == ExtentCells("gc")
FreshGc == {c \in GcCells : cellState[c] = "fresh"}

Progress ==
    CASE w = 0 /\ crashCount' = crashCount + 1 /\ phase["gc"] = "intent"        -> 1
      [] w = 1 /\ m4Rec = "pins_restored" /\ m4Rec' = "fsroot_resumed"
               /\ phase["gc"] = "intent" /\ phase'["gc"] = "fsroot"             -> 2
      [] w = 2 /\ crashCount' = crashCount + 1 /\ m4Rec = "fsroot_resumed"
               /\ phase["gc"] = "fsroot" /\ phase'["gc"] = "fsroot_reopened"    -> 3
      [] w = 3 /\ m4Rec = "fsroot_resumed" /\ m4Rec' = "complete_resumed"
               /\ phase["gc"] = "fsroot_reopened" /\ phase'["gc"] = "admitted"  -> 4
      [] w = 5 /\ crashCount' = crashCount + 1 /\ FreshGc # {}                  -> 6
      [] w = 6 /\ crashed /\ ~crashed'
               /\ FreshGc # {}
               /\ \A c \in FreshGc : cellState'[c] = "fresh" /\ cellGen'[c] = cellGen[c]
               /\ phase'["gc"] = "admitted"
               /\ allocatorAdmits'["gc"] = 1                                    -> 7
      [] OTHER -> w

W3Init == Init /\ w = 0

W3Next ==
    \/ /\ w = 4
       /\ ~crashed
       /\ ~ENABLED AdmitToAllocator("gc")
       /\ \E x \in inflight :
             /\ x.range \subseteq GcCells
             /\ GenGT(x.gen, OrigGen)
             /\ \A c \in x.range : cellState[c] = "fresh"
             /\ WritebackLand(x)
       /\ w' = 5
    \/ Next /\ w' = Progress

W3Spec == W3Init /\ [][W3Next]_w3Vars

Witness3NotReached == w < 7

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W3Focus ==
    /\ durableHead = 0 /\ pendingWrite = "none"
    /\ pinState = "None"
    /\ tokenBasis = "none"
    /\ \A b \in Branches : branchState[b] = "live" /\ prep[b] = "none"
    /\ phase["base"] = "none" /\ phase["cand"] = "none"

=============================================================================
