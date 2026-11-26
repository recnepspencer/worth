------------------- MODULE StoreM4Witness5RetainedCursor -------------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 5 (milestone-4.md section 9.2): retained cursor across
  history consolidation and HEAD advance.

  1 A RetainedBasisToken is issued on B at "base".
  2 B's HEAD advances past the token's basis ("b1").
  3 History is consolidated (RetireHistoryPrefix moves the retained floor).
  4 Crash.
  5 Reopen completes (RecoverServing).
  6 ResumeCursor takes a ReaderLoan on exactly "base" (not B's HEAD "b1").
  7 After "base" is pruned (in reclamation), resuming is refused with
    HistoryUnavailable (ResumeCursorRefused); it never rebinds to HEAD.
  TLC must violate Witness5NotReached.
*)

VARIABLE w

w5Vars == <<vars, w>>

Progress ==
    CASE w = 0 /\ tokenBasis = "none" /\ tokenBasis' = "base"              -> 1
      [] w = 1 /\ head["B"] = "base" /\ head'["B"] = "b1"                  -> 2
      [] w = 2 /\ retainedFloor' > retainedFloor /\ head["B"] = "b1"       -> 3
      [] w = 3 /\ crashCount' = crashCount + 1                             -> 4
      [] w = 4 /\ crashed /\ ~crashed'                                     -> 5
      [] w = 5 /\ cursorLoan = "none" /\ cursorLoan' = "base"
               /\ head["B"] = "b1"                                         -> 6
      [] OTHER -> w

W5Init == Init /\ w = 0

W5Next ==
    \/ /\ w = 6
       /\ phase["base"] # "none"
       /\ ResumeCursorRefused
       /\ w' = 7
    \/ Next /\ w' = Progress

W5Spec == W5Init /\ [][W5Next]_w5Vars

Witness5NotReached == w < 7

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W5Focus ==
    /\ crashCount <= 1
    /\ inflight = {}
    /\ phase["gc"] = "none" /\ phase["cand"] = "none"
    /\ liveChildPins = 0 /\ activeReaderLoans = 0
    /\ \A b \in Branches : branchState[b] # "archived"

=============================================================================
