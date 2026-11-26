------------------ MODULE StoreM4Witness1GrandchildSurvival ------------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 1 (milestone-4.md section 9.2): grandchild survival and
  unrelated space return.

  Branch A is retired while sibling B and a grandchild reader loan (through
  the fork pin's child overlay) still share "base"; the fork lease enters
  "Expiring" and unrelated garbage "gc" is reclaimed all the way to
  allocator admission while that grandchild loan still holds "base"; finally
  "base" itself enters reclamation once the last loan, the child overlay and
  the lease have drained and B has moved on. TLC must violate
  Witness1NotReached, printing the trace.

  `viaChild` and `w` are witness-only history variables: every W1Next step is
  a Next step, so the trace is a behaviour of Spec.
*)

VARIABLES viaChild, w

w1Vars == <<vars, viaChild, w>>

Progress ==
    CASE w = 0
         /\ branchState["A"] # "retired" /\ branchState'["A"] = "retired"
         /\ head["A"] = "base"
         /\ branchState["B"] = "live" /\ head["B"] = "base"
         /\ viaChild /\ activeReaderLoans = 1 /\ liveChildPins = 1
         /\ "base" \in PinRoots                                        -> 1
      [] w = 1
         /\ pinState = "Expiring"
         /\ phase["gc"] # "admitted" /\ phase'["gc"] = "admitted"
         /\ viaChild /\ activeReaderLoans = 1
         /\ "base" \in PinRoots                                        -> 2
      [] w = 2
         /\ phase["base"] = "none" /\ phase'["base"] = "intent"
         /\ branchState["B"] = "live"                                  -> 3
      [] OTHER -> w

W1Init == Init /\ viaChild = FALSE /\ w = 0

W1Next ==
    /\ \/ AcquireGrandchildLoan /\ viaChild' = TRUE
       \/ AcquireReaderLoan /\ viaChild' = FALSE
       \/ /\ Next
          /\ ~(activeReaderLoans = 0 /\ activeReaderLoans' = 1)
          /\ viaChild' = (viaChild /\ activeReaderLoans' = 1)
    /\ w' = Progress

W1Spec == W1Init /\ [][W1Next]_w1Vars

Witness1NotReached == w < 3

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W1Focus ==
    /\ crashCount = 0
    /\ tokenBasis = "none"
    /\ inflight = {}
    /\ phase["cand"] = "none"
    /\ \A b \in Branches : branchState[b] # "archived"

=============================================================================
