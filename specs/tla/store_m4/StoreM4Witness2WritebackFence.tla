------------------- MODULE StoreM4Witness2WritebackFence -------------------
EXTENDS StoreM4ScaleReclaim

(*
  Non-vacuity witness 2 (milestone-4.md section 9.2, P0): paused writeback
  fence and candidate cancel without root publication.

  fA  Path A: "gc" completed retirement while an overlapping split/coalesced
      claim is in flight, and AdmitToAllocator("gc") is NOT enabled.
  fB  Path B: "cand" cancelled and checkpoint-covered while an overlapping
      coalesced claim is in flight, and AdmitToAllocator("cand") is NOT enabled.
  aA  after fA, AdmitToAllocator("gc") succeeds at gen' > g.
  aB  after fB, AdmitToAllocator("cand") succeeds at gen' > g with ZERO root
      publication since cell 4's original allocation (rootPubGen unchanged
      from cellGen[4][1], so the generation advance comes strictly from
      extent_fence_seq; dropping fenceSeq++ in CancelCandidateExtent breaks
      this witness).
  st  after aA and aB, a fresh submission at the stale generation g on an
      overlapping range is refused (RefuseStaleWriteback).
  TLC must violate Witness2NotReached.
*)

VARIABLES fl, pubAtCancel

w2Vars == <<vars, fl, pubAtCancel>>

OverlapsClaim(e) == \E x \in inflight : Overlaps(x.range, ExtentCells(e)) /\ x.range # ExtentCells(e)

FencedA ==
    /\ phase["gc"] = "complete"
    /\ OverlapsClaim("gc")
    /\ ~ENABLED AdmitToAllocator("gc")

FencedB ==
    /\ phase["cand"] = "cancelled" /\ cov["cand"] = "covered"
    /\ OverlapsClaim("cand")
    /\ ~ENABLED AdmitToAllocator("cand")

AdmittedAfterFence(e) ==
    /\ phase'[e] = "admitted" /\ phase[e] # "admitted"
    /\ \A c \in ExtentCells(e) : GenGT(cellGen'[c], cellGen[c])

W2Init ==
    /\ Init
    /\ fl = [fA |-> FALSE, fB |-> FALSE, aA |-> FALSE, aB |-> FALSE, st |-> FALSE]
    /\ pubAtCancel = -1

W2Next ==
    \/ /\ Next
       /\ pubAtCancel' = IF phase["cand"] = "none" /\ phase'["cand"] = "cancelled"
                         THEN rootPubGen' ELSE pubAtCancel
       /\ fl' = [fA |-> fl.fA \/ FencedA,
                 fB |-> fl.fB \/ FencedB,
                 aA |-> fl.aA \/ (fl.fA /\ AdmittedAfterFence("gc")),
                 aB |-> fl.aB \/ (fl.fB /\ AdmittedAfterFence("cand")
                                        /\ rootPubGen' = pubAtCancel
                                        /\ cellGen'[4][1] = cellGen[4][1]
                                        /\ cellGen'[4][2] > cellGen[4][2]),
                 st |-> fl.st]
    \/ /\ fl.aA /\ fl.aB
       /\ \E r \in IoRanges :
             /\ Overlaps(r, ExtentCells("gc") \cup ExtentCells("cand"))
             /\ RefuseStaleWriteback(r, OrigGen)
             /\ \E c \in r : GenGT(cellGen[c], OrigGen)
       /\ fl' = [fl EXCEPT !.st = TRUE]
       /\ UNCHANGED pubAtCancel

W2Spec == W2Init /\ [][W2Next]_w2Vars

Witness2NotReached == ~(fl.fA /\ fl.fB /\ fl.aA /\ fl.aB /\ fl.st)

\* Search focus (prunes only; any trace found is still a Spec behaviour).
W2Focus ==
    /\ durableHead = 0 /\ pendingWrite = "none"
    /\ crashCount = 0
    /\ pinState = "None"
    /\ tokenBasis = "none"
    /\ \A b \in Branches : branchState[b] = "live" /\ prep[b] = "none"
    /\ phase["base"] = "none"

=============================================================================
