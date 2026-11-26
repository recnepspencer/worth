------------------------ MODULE StoreM4ScaleReclaim -------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Store Milestone 4 (M4.6a) composed split-barrier specification.

  Composes `specs/tla/store_m3/StoreDurability.tla` (resolved through
  `tools/tla/run_tlc.py`'s `-DTLA-Library`) with every M4 durable barrier of
  `plans/database-foundation/milestone-4.md` section 9.2. Each barrier is its
  own atomic action, so `Crash` interleaves between any two of them, including
  inside recovery.

  Coupling with M3 (shared variables, conjoined actions):
    BeginBranchCommit                = M3 BeginCommit (+ FIFO revalidation)
    PublishBranchCommit              = M3 SealManifest (+ frontier compose)
    BeginCheckpoint / SealCheckpoint = M3 checkpoint (+ canonical coverage)
    RetireHistoryPrefix              = M3 history consolidation
    Crash                            = M3 CrashKill (+ volatile drop)
    BeginRecover                     = M3 BeginRecover (current build)
    RecoverWal                       = M3 AdvanceRecoverStage (+ commit fate)
    RecoverServing                   = M3 FinishRecover (+ M4 stages done)
  M4 recovery stages RecoverRestorePins -> RecoverResumeFreeSpaceRoot ->
  RecoverResumeReclaimComplete run while M3 sits at "manifest_switch", where
  M3's CrashKill stays enabled, so chained crashes hit every M4 stage.
  M3's fork lifecycle, D14 derived discard, graceful reopen and build
  alternation touch no M4 state; StoreDurability.cfg checks them, and they are
  left out of this composition.

  Finite abstraction (fixed topology; DST fixtures bind to it, see README):
    Branches  A, B share root chunk "base"; a commit of b installs NewRoot(b)
              (every later commit of b keeps that abstract delta).
    Chunks    "base" (shared base, a physical extent), "gc" (unrelated
              displaced garbage, a physical extent), "a1"/"b1" (branch deltas,
              placement not modelled).
    Cells     1..4. Extent "base" = {1}; "gc" = {2,3}; "cand" = {4}, a
              candidate extent cancelled without root publication (Path B).
              Overlap is set intersection (see IoRanges / AllocRanges).
    Gens      ExtentAllocationGeneration = <<rootPubGen, fenceSeq>>, compared
              lexicographically.
    One whole-Home fork pin with a durable child overlay pin and one volatile
    reader loan, taken directly on the pin or (grandchild) through the child.

  Checking: Spec (full Next) drives the five witness modules and the trace
  validator; safety is checked exhaustively over LifecycleSpec (this
  module's .cfg) and PhysicalSpec (StoreM4ScaleReclaimPhysical.cfg), two
  projection-sound sub-relations of Next defined below.
*)

CONSTANTS
    MaxCommits,     \* M3: commit horizon
    MaxCrashes,     \* M3: crash budget (chained crashes)
    BuildIds,       \* M3: application build identities
    InitialBuild,   \* M3: starting build identity
    MaxInflight     \* bound on concurrently in-flight writeback claims

ASSUME MaxInflight \in Nat \ {0}

VARIABLES
    \* ---- StoreDurability (M3) ----
    durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
    pendingWrite, pendingTarget, activeLease, forkPhase, crashed,
    recoveryStage, crashCount, buildId, derivedState, recoveredHead,
    reopenSucceeded,
    \* ---- Relational branches & Home frontier (durable unless noted) ----
    branchState,    \* [Branches -> {"live","archived","retired"}]
    head,           \* [Branches -> Chunks] selected root per branch
    prep,           \* [Branches -> PrepStates] volatile prepared candidate: "none" | "current" (on the
                    \* current frontier) | "stale" (frontier advanced since preparation)
    frontierSeq,    \* Home frontier sequence (branch commits composed)
    pendingBranch,  \* branch whose commit occupies M3's pending write, or "none"
    retiredEver,    \* history: names ever retired
    \* ---- PinTable / lease ----
    pinState,           \* "None" | "Active" | "Expiring" | "Expired" (durable)
    pinnedRoots,        \* HomeRootManifest roots captured by the pin (durable)
    activeReaderLoans,  \* volatile reader loans, direct or grandchild through the child overlay (0..1)
    liveChildPins,      \* durable nested child overlay pins (0..1)
    pinsRestored,       \* volatile: FALSE after Crash until RecoverRestorePins restores PinTable
    \* ---- Physical extents, generations, writebacks, reclamation ----
    phase,          \* [Extents -> Phases] durable retirement / cancel record
    cov,            \* [Extents -> CovStates] canonical checkpoint coverage of the extent's removal
    cellState,      \* [Cells -> {"live","retired","free","fresh"}] durable
    cellGen,        \* [Cells -> Gens] generation of the cell's current/last allocation
    rootPubGen,     \* durable root_publication_generation
    fenceSeq,       \* durable extent_fence_seq
    inflight,       \* volatile set of [range, gen] writeback claims
    fsRootReleases, \* history: free-space root publications per extent
    allocatorAdmits,\* history: allocator admissions per extent
    \* ---- Retained-basis cursor ----
    tokenBasis,     \* RetainedBasisToken basis chunk held by the client, or "none"
    cursorLoan,     \* volatile ReaderLoan of a resumed cursor, or "none"
    \* ---- M4 recovery stage ----
    m4Rec           \* "none" | "pins_restored" | "fsroot_resumed" | "complete_resumed"

M3 == INSTANCE StoreDurability

m3Vars == <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
            pendingWrite, pendingTarget, activeLease, forkPhase, crashed,
            recoveryStage, crashCount, buildId, derivedState, recoveredHead,
            reopenSucceeded>>
branchVars == <<branchState, head, prep, frontierSeq, pendingBranch, retiredEver>>
pinVars == <<pinState, pinnedRoots, activeReaderLoans, liveChildPins, pinsRestored>>
extentVars == <<phase, cov, cellState, cellGen, rootPubGen, fenceSeq, inflight,
                fsRootReleases, allocatorAdmits>>
cursorVars == <<tokenBasis, cursorLoan>>
m4Vars == <<branchVars, pinVars, extentVars, cursorVars, m4Rec>>
vars == <<m3Vars, m4Vars>>

\* ---------------------------------------------------------------------------
\* Vocabulary
\* ---------------------------------------------------------------------------

Epochs == 0..MaxCommits
Branches == {"A", "B"}
NoBranch == "none"
NewRoot(b) == IF b = "A" THEN "a1" ELSE "b1"
Chunks == {"base", "gc", "a1", "b1"}
Extents == {"base", "gc", "cand"}
PathAExtents == {"base", "gc"}
Cells == 1..4
ExtentCells(e) == CASE e = "base" -> {1}
                    [] e = "gc"   -> {2, 3}
                    [] e = "cand" -> {4}
ExtentOf(c) == CASE c = 1 -> "base"
                 [] c \in {2, 3} -> "gc"
                 [] c = 4 -> "cand"
\* Writeback claims use a split range ({3} inside "gc") and a range coalesced
\* across extents ({3,4}, over "gc" and "cand"), so one claim can fence both
\* Path A and Path B. Reallocation reuses "gc" split ({3}) or whole ({2,3}).
\* "base" is reclaimed through Reach/pins only.
IoRanges == {{3}, {3, 4}}
AllocRanges == {{3}, {4}, {2, 3}, {3, 4}}
Overlaps(r1, r2) == r1 \cap r2 # {}

MaxPubGen == MaxCommits + 2
MaxFenceSeq == 3
Gens == (0..MaxPubGen) \X (0..MaxFenceSeq)
OrigGen == <<0, 0>>
GlobalGen == <<rootPubGen, fenceSeq>>
GenGT(a, b) == a[1] > b[1] \/ (a[1] = b[1] /\ a[2] > b[2])

BranchStates == {"live", "archived", "retired"}
PrepStates == {"none", "current", "stale"}
PinStates == {"None", "Active", "Expiring", "Expired"}
Phases == {"none", "intent", "fsroot", "fsroot_reopened", "complete", "cancelled", "admitted"}
RetiringPhases == {"intent", "fsroot", "fsroot_reopened", "complete", "cancelled"}
CovStates == {"live", "dropped", "staged", "covered"}
CellStates == {"live", "retired", "free", "fresh"}
M4RecStages == {"none", "pins_restored", "fsroot_resumed", "complete_resumed"}
WritebackClaims == [range : IoRanges, gen : Gens]

LiveRef(b) == branchState[b] \in {"live", "archived"}
CatalogRoots == {head[b] : b \in {x \in Branches : LiveRef(x)}}

\* The pin keeps its captured manifest reachable while Active/Expiring and
\* while any child overlay or loan still depends on it (transitive edges).
PinHolds ==
    \/ pinState \in {"Active", "Expiring"}
    \/ liveChildPins > 0
    \/ activeReaderLoans > 0
PinRoots == IF PinHolds THEN pinnedRoots ELSE {}

\* Complete independent reachability law (section 7.2 `Reach`).
\* Fail-closed: before PinTable restoration on reopen, every non-retiring extent
\* is treated as conservatively reachable so reclamation cannot admit new intents.
Reach ==
    CatalogRoots
    \cup (IF pinsRestored THEN PinRoots ELSE {e \in Extents : phase[e] = "none"})
    \cup (IF cursorLoan # "none" THEN {cursorLoan} ELSE {})
    \cup (IF phase["cand"] = "none" THEN {"cand"} ELSE {})   \* in-flight candidate decision

\* A token basis is independently retained iff some durable holder still
\* references it and it has not entered reclamation.
Retained(c) ==
    /\ c \in CatalogRoots \cup PinRoots
    /\ (c \in Extents => phase[c] = "none")

UniformGen(r, g) == \A c \in r : cellGen[c] = g
Writable(r, g) == \A c \in r : cellState[c] \in {"live", "fresh"} /\ cellGen[c] = g

\* Removing the last catalog reference to "base" starts its coverage clock.
CovAfter(h, bs) ==
    [cov EXCEPT !["base"] =
        IF cov["base"] = "live"
           /\ ~\E b \in Branches : bs[b] \in {"live", "archived"} /\ h[b] = "base"
        THEN "dropped" ELSE cov["base"]]

Init ==
    /\ M3!Init
    /\ branchState = [b \in Branches |-> "live"]
    /\ head = [b \in Branches |-> "base"]
    /\ prep = [b \in Branches |-> "none"]
    /\ frontierSeq = 0
    /\ pendingBranch = NoBranch
    /\ retiredEver = {}
    /\ pinState = "None"
    /\ pinnedRoots = {}
    /\ activeReaderLoans = 0
    /\ liveChildPins = 0
    /\ pinsRestored = TRUE
    /\ phase = [e \in Extents |-> "none"]
    /\ cov = [e \in Extents |-> IF e = "gc" THEN "covered" ELSE "live"]
    /\ cellState = [c \in Cells |-> "live"]
    /\ cellGen = [c \in Cells |-> OrigGen]
    /\ rootPubGen = 0
    /\ fenceSeq = 0
    /\ inflight = {}
    /\ fsRootReleases = [e \in Extents |-> 0]
    /\ allocatorAdmits = [e \in Extents |-> 0]
    /\ tokenBasis = "none"
    /\ cursorLoan = "none"
    /\ m4Rec = "none"

\* ===========================================================================
\* 1. Commits, FIFO revalidation & Home-frontier composition (M3 coupled)
\* ===========================================================================

\* Preparation runs concurrently per branch against the current frontier.
PrepareBranch(b) ==
    /\ ~crashed
    /\ branchState[b] = "live"
    /\ prep[b] = "none"
    /\ pendingBranch # b
    /\ durableHead + (IF pendingWrite = "none" THEN 0 ELSE 1) < MaxCommits  \* a commit is still possible
    /\ prep' = [prep EXCEPT ![b] = "current"]
    /\ UNCHANGED <<m3Vars, branchState, head, frontierSeq, pendingBranch,
                   retiredEver, pinVars, extentVars, cursorVars, m4Rec>>

\* FIFO patch-position reservation: revalidate own lifecycle, then take M3's
\* commit lease. The candidate composes onto whatever frontier is current at
\* publication (prep[b] = "stale": it was prepared on an older frontier).
BeginBranchCommit(b) ==
    /\ M3!BeginCommit
    /\ prep[b] # "none"
    /\ branchState[b] = "live"
    /\ pendingBranch' = b
    /\ prep' = [prep EXCEPT ![b] = "none"]
    /\ UNCHANGED <<branchState, head, frontierSeq, retiredEver, pinVars,
                   extentVars, cursorVars, m4Rec>>

\* Typed refusal at revalidation: BranchArchived / BranchDeleting.
RefuseStaleCandidate(b) ==
    /\ ~crashed
    /\ prep[b] # "none"
    /\ branchState[b] # "live"
    /\ prep' = [prep EXCEPT ![b] = "none"]
    /\ UNCHANGED <<m3Vars, branchState, head, frontierSeq, pendingBranch,
                   retiredEver, pinVars, extentVars, cursorVars, m4Rec>>

FlushJournal ==
    /\ M3!FlushJournal
    /\ UNCHANGED m4Vars

\* Applies pending branch delta onto the CURRENT frontier: only component b
\* changes, sibling components are preserved.
ApplyPending(b) ==
    /\ head' = [head EXCEPT ![b] = NewRoot(b)]
    /\ frontierSeq' = frontierSeq + 1
    /\ cov' = CovAfter([head EXCEPT ![b] = NewRoot(b)], branchState)

PublishBranchCommit ==
    /\ M3!SealManifest
    /\ pendingBranch # NoBranch
    /\ ApplyPending(pendingBranch)
    /\ prep' = [b \in Branches |-> IF prep[b] = "current" THEN "stale" ELSE prep[b]]
    /\ rootPubGen' = rootPubGen + 1
    /\ pendingBranch' = NoBranch
    /\ UNCHANGED <<branchState, retiredEver, pinVars, phase, cellState,
                   cellGen, fenceSeq, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Canonical coverage: a sealed checkpoint begun after the removal covers it.
BeginCheckpoint(ep) ==
    /\ M3!BeginCheckpoint(ep)
    /\ cov' = [e \in Extents |-> IF cov[e] = "dropped" THEN "staged" ELSE cov[e]]
    /\ UNCHANGED <<branchVars, pinVars, phase, cellState, cellGen, rootPubGen,
                   fenceSeq, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

SealCheckpoint ==
    /\ M3!SealCheckpoint
    /\ cov' = [e \in Extents |-> IF cov[e] = "staged" THEN "covered" ELSE cov[e]]
    /\ UNCHANGED <<branchVars, pinVars, phase, cellState, cellGen, rootPubGen,
                   fenceSeq, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Fuzzy maintenance checkpoint covering a cancelled candidate's WAL record
\* without requiring a branch commit or root publication (M4.5a Path B,
\* extent_copy/resolution.rs:158).
CoverCancelCheckpoint ==
    /\ ~crashed
    /\ phase["cand"] = "cancelled"
    /\ cov["cand"] = "dropped"
    /\ cov' = [cov EXCEPT !["cand"] = "covered"]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, phase, cellState, cellGen,
                   rootPubGen, fenceSeq, inflight, fsRootReleases,
                   allocatorAdmits, cursorVars, m4Rec>>

\* History consolidation (journal prefix retirement, physical locators move).
RetireHistoryPrefix(fl) ==
    /\ M3!RetireHistoryPrefix(fl)
    /\ UNCHANGED m4Vars

\* ===========================================================================
\* 2. Branch lifecycle: archive, permanent retirement
\* ===========================================================================

ArchiveBranch(b) ==
    /\ ~crashed
    /\ branchState[b] = "live"
    /\ pendingBranch # b
    /\ branchState' = [branchState EXCEPT ![b] = "archived"]
    /\ UNCHANGED <<m3Vars, head, prep, frontierSeq, pendingBranch, retiredEver,
                   pinVars, extentVars, cursorVars, m4Rec>>

\* Retires the name forever and drops only b's selected-root reference.
RetireBranchRef(b) ==
    /\ ~crashed
    /\ branchState[b] \in {"live", "archived"}
    /\ pendingBranch # b
    /\ branchState' = [branchState EXCEPT ![b] = "retired"]
    /\ retiredEver' = retiredEver \cup {b}
    /\ cov' = CovAfter(head, [branchState EXCEPT ![b] = "retired"])
    /\ UNCHANGED <<m3Vars, head, prep, frontierSeq, pendingBranch, pinVars,
                   phase, cellState, cellGen, rootPubGen, fenceSeq, inflight,
                   fsRootReleases, allocatorAdmits, cursorVars, m4Rec>>

\* Typed refusal: a retired name is never reused (no durable effect).
RefuseRetiredBranchName(b) ==
    /\ ~crashed
    /\ branchState[b] = "retired"
    /\ UNCHANGED vars

\* ===========================================================================
\* 3. PinTable: fork pin, nested overlays, reader-safe lease expiry
\* ===========================================================================

\* Captures the sealed HomeRootManifest (all catalog roots) atomically under
\* the reclamation fence.
PublishForkPin ==
    /\ ~crashed
    /\ pinState = "None"
    /\ pinState' = "Active"
    /\ pinnedRoots' = CatalogRoots
    /\ UNCHANGED <<m3Vars, branchVars, activeReaderLoans, liveChildPins,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

\* Direct reader loan on the fork pin: fenced once the lease is Expiring.
AcquireReaderLoan ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ activeReaderLoans = 0
    /\ activeReaderLoans' = 1
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, liveChildPins,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

\* Grandchild reader loan taken through the live child overlay; the child is
\* its own lease, so the parent's Expiring fence does not refuse it, and the
\* parent's roots stay reachable through the transitive overlay edge.
AcquireGrandchildLoan ==
    /\ ~crashed
    /\ liveChildPins = 1
    /\ activeReaderLoans = 0
    /\ activeReaderLoans' = 1
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, liveChildPins,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

ReleaseReaderLoan ==
    /\ ~crashed
    /\ activeReaderLoans = 1
    /\ activeReaderLoans' = 0
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, liveChildPins,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

AttachChildPin ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ liveChildPins = 0
    /\ liveChildPins' = 1
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, activeReaderLoans,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

\* Conservative drain: the child is released only once no loan is live.
ReleaseChildPin ==
    /\ ~crashed
    /\ liveChildPins = 1
    /\ activeReaderLoans = 0
    /\ liveChildPins' = 0
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, activeReaderLoans,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

\* Active -> Expiring fences new direct loans and child pins.
BeginExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ pinState' = "Expiring"
    /\ UNCHANGED <<m3Vars, branchVars, pinnedRoots, activeReaderLoans,
                   liveChildPins, pinsRestored, extentVars, cursorVars, m4Rec>>

\* Expiring -> Expired only once every loan and child overlay has drained.
FinishExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Expiring"
    /\ activeReaderLoans = 0
    /\ liveChildPins = 0
    /\ pinState' = "Expired"
    /\ pinnedRoots' = {}
    /\ UNCHANGED <<m3Vars, branchVars, activeReaderLoans, liveChildPins,
                   pinsRestored, extentVars, cursorVars, m4Rec>>

\* ===========================================================================
\* 4. P0 writeback fence & generation-checked I/O
\* ===========================================================================

\* Writeback claim admitted only at the range's active generation.
WritebackIssue(r, g) ==
    /\ ~crashed
    /\ r \in IoRanges
    /\ Cardinality(inflight) < MaxInflight
    /\ Writable(r, g)
    /\ [range |-> r, gen |-> g] \notin inflight
    /\ inflight' = inflight \cup {[range |-> r, gen |-> g]}
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, phase, cov, cellState, cellGen,
                   rootPubGen, fenceSeq, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Fail-closed: a submission whose generation is not the active one (or whose
\* range is retired / free) is refused with a typed generation mismatch.
RefuseStaleWriteback(r, g) ==
    /\ ~crashed
    /\ r \in IoRanges
    /\ ~Writable(r, g)
    /\ UNCHANGED vars

\* The paused claim reaches the device; it may only land on its generation.
WritebackLand(w) ==
    /\ ~crashed
    /\ w \in inflight
    /\ UniformGen(w.range, w.gen)
    /\ \A c \in w.range : cellState[c] # "free"
    /\ inflight' = inflight \ {w}
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, phase, cov, cellState, cellGen,
                   rootPubGen, fenceSeq, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Drain by cancellation.
WritebackCancel(w) ==
    /\ ~crashed
    /\ w \in inflight
    /\ inflight' = inflight \ {w}
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, phase, cov, cellState, cellGen,
                   rootPubGen, fenceSeq, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* ===========================================================================
\* 5. Split four-barrier reclamation (Path A) & candidate cancel (Path B)
\* ===========================================================================

\* Barrier 1: requires restored pins, range outside Reach, and canonical coverage.
ReclaimIntent(e) ==
    /\ ~crashed
    /\ pinsRestored
    /\ e \in PathAExtents
    /\ phase[e] = "none"
    /\ e \notin Reach
    /\ cov[e] = "covered"
    /\ phase' = [phase EXCEPT ![e] = "intent"]
    /\ cellState' = [c \in Cells |-> IF c \in ExtentCells(e) THEN "retired" ELSE cellState[c]]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, cellGen, rootPubGen,
                   fenceSeq, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Barrier 2: free-space authority root publication.
PublishFreeSpaceRoot(e) ==
    /\ ~crashed
    /\ e \in PathAExtents
    /\ phase[e] = "intent"
    /\ phase' = [phase EXCEPT ![e] = "fsroot"]
    /\ rootPubGen' = rootPubGen + 1
    /\ fsRootReleases' = [fsRootReleases EXCEPT ![e] = @ + 1]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, cellState, cellGen,
                   fenceSeq, inflight, allocatorAdmits, cursorVars, m4Rec>>

\* Barrier 3: durable RETIREMENT_EXTENT_COMPLETION; advances the fence.
ReclaimComplete(e) ==
    /\ ~crashed
    /\ e \in PathAExtents
    /\ phase[e] = "fsroot"
    /\ phase' = [phase EXCEPT ![e] = "complete"]
    /\ fenceSeq' = fenceSeq + 1
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, cellState, cellGen,
                   rootPubGen, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* Path B barrier 1: durable cancel WAL record, no root publication; advances
\* the fence so the range is never reissued at the same generation.
CancelCandidateExtent ==
    /\ ~crashed
    /\ phase["cand"] = "none"
    /\ phase' = [phase EXCEPT !["cand"] = "cancelled"]
    /\ cov' = [cov EXCEPT !["cand"] = "dropped"]
    /\ fenceSeq' = fenceSeq + 1
    /\ cellState' = [c \in Cells |-> IF c \in ExtentCells("cand") THEN "retired" ELSE cellState[c]]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cellGen, rootPubGen, inflight,
                   fsRootReleases, allocatorAdmits, cursorVars, m4Rec>>

\* Final barrier for both paths: (a) completed retirement or (b) checkpoint-
\* covered cancel; WritebackFenceDrain (no overlapping claim in flight); and a
\* strictly newer generation than any seen on the range.
AdmitToAllocator(e) ==
    /\ ~crashed
    /\ \/ e \in PathAExtents /\ phase[e] = "complete"
       \/ e = "cand" /\ phase[e] = "cancelled" /\ cov[e] = "covered"
    /\ ~\E w \in inflight : Overlaps(w.range, ExtentCells(e))
    /\ \A c \in ExtentCells(e) : GenGT(GlobalGen, cellGen[c])
    /\ phase' = [phase EXCEPT ![e] = "admitted"]
    /\ cellState' = [c \in Cells |-> IF c \in ExtentCells(e) THEN "free" ELSE cellState[c]]
    /\ cellGen' = [c \in Cells |-> IF c \in ExtentCells(e) THEN GlobalGen ELSE cellGen[c]]
    /\ allocatorAdmits' = [allocatorAdmits EXCEPT ![e] = @ + 1]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, rootPubGen, fenceSeq,
                   inflight, fsRootReleases, cursorVars, m4Rec>>

\* Reuse of admitted cells (split, or coalesced across ranges admitted at the
\* same generation) under a durable allocation record at the admitted gen'.
AllocateExtent(r) ==
    /\ ~crashed
    /\ r \in AllocRanges
    /\ \A c \in r : cellState[c] = "free"
    /\ \E c0 \in r : UniformGen(r, cellGen[c0])
    /\ cellState' = [c \in Cells |-> IF c \in r THEN "fresh" ELSE cellState[c]]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, phase, cov, cellGen, rootPubGen, fenceSeq,
                   inflight, fsRootReleases, allocatorAdmits, cursorVars, m4Rec>>

\* ===========================================================================
\* 6. Retained-basis cursor tokens & resume-vs-prune ReaderLoan
\* ===========================================================================

IssueRetainedBasisToken(b) ==
    /\ ~crashed
    /\ tokenBasis = "none"
    /\ LiveRef(b)
    /\ tokenBasis' = head[b]
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, extentVars, cursorLoan, m4Rec>>

\* Atomically resolve the exact basis and take a loan under the reclaim fence.
ResumeCursor ==
    /\ ~crashed
    /\ tokenBasis # "none"
    /\ cursorLoan = "none"
    /\ Retained(tokenBasis)
    /\ cursorLoan' = tokenBasis
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, extentVars, tokenBasis, m4Rec>>

\* Prune won: typed HistoryUnavailable, never a rebind to HEAD.
ResumeCursorRefused ==
    /\ ~crashed
    /\ tokenBasis # "none"
    /\ cursorLoan = "none"
    /\ ~Retained(tokenBasis)
    /\ UNCHANGED vars

ReleaseCursorLoan ==
    /\ ~crashed
    /\ cursorLoan # "none"
    /\ cursorLoan' = "none"
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, extentVars, tokenBasis, m4Rec>>

\* ===========================================================================
\* 7. Crash & chained-crash interruptible recovery
\* ===========================================================================

\* Drops all volatile state: loans, unlanded writebacks, preparations, cursor
\* loans, PinTable residency, and unsealed checkpoint coverage. Any extent at
\* "fsroot" becomes "fsroot_reopened" because bootstrap on reopen will already
\* load its published free-space root into the allocator's durable index.
Crash ==
    /\ M3!CrashKill
    /\ activeReaderLoans' = 0
    /\ pinsRestored' = FALSE
    /\ inflight' = {}
    /\ prep' = [b \in Branches |-> "none"]
    /\ cursorLoan' = "none"
    /\ cov' = [e \in Extents |-> IF cov[e] = "staged" THEN "dropped" ELSE cov[e]]
    /\ phase' = [e \in Extents |-> IF phase[e] = "fsroot" THEN "fsroot_reopened" ELSE phase[e]]
    /\ m4Rec' = "none"
    /\ UNCHANGED <<branchState, head, frontierSeq, pendingBranch, retiredEver,
                   pinState, pinnedRoots, liveChildPins, cellState,
                   cellGen, rootPubGen, fenceSeq, fsRootReleases,
                   allocatorAdmits, tokenBasis>>

BeginRecover(bid) ==
    /\ M3!BeginRecover(bid)
    /\ UNCHANGED m4Vars

\* WAL resolution decides the pending commit's fate (FailBefore or
\* IndeterminateAfterEffect); an applied branch commit composes onto the
\* current frontier exactly as PublishBranchCommit does.
RecoverWal(rh) ==
    /\ M3!AdvanceRecoverStage(rh)
    /\ IF rh # durableHead /\ pendingBranch # NoBranch
       THEN ApplyPending(pendingBranch)
       ELSE UNCHANGED <<head, frontierSeq, cov>>
    /\ rootPubGen' = IF rh # durableHead THEN rootPubGen + 1 ELSE rootPubGen
    /\ pendingBranch' = NoBranch
    /\ UNCHANGED <<branchState, prep, retiredEver, pinVars, phase, cellState,
                   cellGen, fenceSeq, inflight, fsRootReleases, allocatorAdmits,
                   cursorVars, m4Rec>>

\* PinTable is restored before any maintenance / reclamation resume.
RecoverRestorePins ==
    /\ crashed
    /\ recoveryStage = "manifest_switch"
    /\ m4Rec = "none"
    /\ pinsRestored' = TRUE
    /\ m4Rec' = "pins_restored"
    /\ UNCHANGED <<m3Vars, branchVars, pinState, pinnedRoots, activeReaderLoans,
                   liveChildPins, extentVars, cursorVars>>

\* Idempotent resume of barrier 2 for records still at "intent" only.
RecoverResumeFreeSpaceRoot ==
    /\ crashed
    /\ recoveryStage = "manifest_switch"
    /\ m4Rec = "pins_restored"
    /\ pinsRestored
    /\ LET pending == {e \in PathAExtents : phase[e] = "intent"}
       IN /\ phase' = [e \in Extents |-> IF e \in pending THEN "fsroot" ELSE phase[e]]
          /\ fsRootReleases' = [e \in Extents |->
                                  IF e \in pending THEN fsRootReleases[e] + 1
                                  ELSE fsRootReleases[e]]
          /\ rootPubGen' = IF pending # {} THEN rootPubGen + 1 ELSE rootPubGen
    /\ m4Rec' = "fsroot_resumed"
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, cellState, cellGen,
                   fenceSeq, inflight, allocatorAdmits, cursorVars>>

\* Idempotent resume of barrier 3 (matching Verus resume_release and Lean
\* ReclamationSafety):
\* - "fsroot" (this recovery just published the free-space root -> IntentDurable):
\*   records completion and moves to "complete" so this process performs
\*   AdmitToAllocator once after RecoverServing.
\* - "fsroot_reopened" (crashed after PublishFreeSpaceRoot -> FreeSpacePublished):
\*   bootstrap already loaded the published free-space root into the allocator;
\*   completion settles directly to "admitted" (free at GlobalGen') without a
\*   duplicate AdmitToAllocator (`M9`).
RecoverResumeReclaimComplete ==
    /\ crashed
    /\ recoveryStage = "manifest_switch"
    /\ m4Rec = "fsroot_resumed"
    /\ pinsRestored
    /\ LET ownFsRoot == {e \in PathAExtents : phase[e] = "fsroot"}
           reopenedFsRoot == {e \in PathAExtents : phase[e] = "fsroot_reopened"}
           pending == ownFsRoot \cup reopenedFsRoot
           nextFence == IF pending # {} THEN fenceSeq + 1 ELSE fenceSeq
           settledGen == <<rootPubGen, nextFence>>
       IN /\ phase' = [e \in Extents |->
                         IF e \in ownFsRoot THEN "complete"
                         ELSE IF e \in reopenedFsRoot THEN "admitted"
                         ELSE phase[e]]
          /\ fenceSeq' = nextFence
          /\ cellState' = [c \in Cells |->
                             IF ExtentOf(c) \in reopenedFsRoot THEN "free"
                             ELSE cellState[c]]
          /\ cellGen' = [c \in Cells |->
                           IF ExtentOf(c) \in reopenedFsRoot THEN settledGen
                           ELSE cellGen[c]]
          /\ allocatorAdmits' = [e \in Extents |->
                                   IF e \in reopenedFsRoot THEN allocatorAdmits[e] + 1
                                   ELSE allocatorAdmits[e]]
    /\ m4Rec' = "complete_resumed"
    /\ UNCHANGED <<m3Vars, branchVars, pinVars, cov, rootPubGen,
                   inflight, fsRootReleases, cursorVars>>

RecoverServing ==
    /\ M3!FinishRecover
    /\ m4Rec = "complete_resumed"
    /\ pinsRestored
    /\ m4Rec' = "none"
    /\ UNCHANGED <<branchVars, pinVars, extentVars, cursorVars>>

\* ===========================================================================
\* Next-state relation
\* ===========================================================================

CommitSteps ==
    \/ \E b \in Branches : PrepareBranch(b)
    \/ \E b \in Branches : BeginBranchCommit(b)
    \/ \E b \in Branches : RefuseStaleCandidate(b)
    \/ FlushJournal
    \/ PublishBranchCommit
    \/ \E ep \in Epochs : BeginCheckpoint(ep)
    \/ SealCheckpoint
    \/ CoverCancelCheckpoint
    \/ \E fl \in Epochs : RetireHistoryPrefix(fl)

RetireSteps ==
    \E b \in Branches : RetireBranchRef(b)

ArchiveSteps ==
    \/ \E b \in Branches : ArchiveBranch(b)
    \/ \E b \in Branches : RefuseRetiredBranchName(b)

PinSteps ==
    \/ PublishForkPin
    \/ AcquireReaderLoan
    \/ AcquireGrandchildLoan
    \/ ReleaseReaderLoan
    \/ AttachChildPin
    \/ ReleaseChildPin
    \/ BeginExpireForkLease
    \/ FinishExpireForkLease

\* Tokens are issued on B's shared base only: B's own delta "b1" is never
\* superseded within the model horizon, so a token on it cannot become
\* historical and adds no behaviour the cursor laws can distinguish.
CursorSteps ==
    \/ head["B"] = "base" /\ IssueRetainedBasisToken("B")
    \/ ResumeCursor
    \/ ResumeCursorRefused
    \/ ReleaseCursorLoan

\* Interface between the lifecycle and physical layers: the only steps that
\* move a phase away from "none".
IntentSteps ==
    \/ \E e \in PathAExtents : ReclaimIntent(e)
    \/ CancelCandidateExtent

WritebackSteps ==
    \/ \E r \in IoRanges, c \in Cells : WritebackIssue(r, cellGen[c])
    \/ \E r \in IoRanges, c \in Cells : RefuseStaleWriteback(r, cellGen[c])
    \/ \E w \in inflight : WritebackLand(w)
    \/ \E w \in inflight : WritebackCancel(w)

BarrierSteps ==
    \/ \E e \in PathAExtents : PublishFreeSpaceRoot(e)
    \/ \E e \in PathAExtents : ReclaimComplete(e)
    \/ \E e \in Extents : AdmitToAllocator(e)
    \/ \E r \in AllocRanges : AllocateExtent(r)

RecoverySteps ==
    \/ Crash
    \/ BeginRecover(buildId)
    \/ \E rh \in Epochs : RecoverWal(rh)
    \/ RecoverRestorePins
    \/ RecoverResumeFreeSpaceRoot
    \/ RecoverResumeReclaimComplete
    \/ RecoverServing

\* The full composed next-state relation (witnesses, trace validation).
Next ==
    \/ CommitSteps \/ RetireSteps \/ ArchiveSteps \/ PinSteps \/ CursorSteps
    \/ IntentSteps \/ WritebackSteps \/ BarrierSteps \/ RecoverySteps

Spec == Init /\ [][Next]_vars

(*
  Exhaustive safety checking is split into two sub-relations of Next whose
  union is Next; each is projection-sound for the invariants it is checked
  against (README.md, "Why two safety slices").

  LifecycleNext drops WritebackSteps and BarrierSteps. Those steps change
  only physical variables (cellState, cellGen, inflight, generations, phases
  that are already past "none"), and every lifecycle guard and invariant reads
  phases only through `phase[e] = "none"`. So every Spec behaviour, projected
  on lifecycle state, is a LifecycleSpec behaviour.

  PhysicalNext drops PinSteps, CursorSteps and ArchiveSteps. Those steps never
  write a physical variable; they only add Reach holders (pins, loans, cursor
  loans, archived roots), which can only disable ReclaimIntent("base"). Mapping
  "archived" to "live", projecting away volatile `prep` (just-in-time
  PrepareBranch immediately before BeginBranchCommit simulates every commit
  identically), and erasing pins/cursors, every Spec behaviour projected on
  physical state is a PhysicalSpec behaviour.
*)
LifecycleNext ==
    \/ CommitSteps \/ RetireSteps \/ ArchiveSteps \/ PinSteps \/ CursorSteps
    \/ IntentSteps \/ RecoverySteps

PhysicalNext ==
    \/ CommitSteps \/ RetireSteps
    \/ IntentSteps \/ WritebackSteps \/ BarrierSteps \/ RecoverySteps

LifecycleSpec == Init /\ [][LifecycleNext]_vars
PhysicalSpec == Init /\ [][PhysicalNext]_vars

\* ===========================================================================
\* Safety invariants
\* ===========================================================================

TypeOK ==
    /\ M3!TypeOK
    /\ branchState \in [Branches -> BranchStates]
    /\ head \in [Branches -> Chunks]
    /\ prep \in [Branches -> PrepStates]
    /\ frontierSeq \in 0..MaxCommits
    /\ pendingBranch \in Branches \cup {NoBranch}
    /\ retiredEver \subseteq Branches
    /\ pinState \in PinStates
    /\ pinnedRoots \subseteq Chunks
    /\ activeReaderLoans \in 0..1
    /\ liveChildPins \in 0..1
    /\ pinsRestored \in BOOLEAN
    /\ phase \in [Extents -> Phases]
    /\ cov \in [Extents -> CovStates]
    /\ cellState \in [Cells -> CellStates]
    /\ cellGen \in [Cells -> Gens]
    /\ rootPubGen \in 0..MaxPubGen
    /\ fenceSeq \in 0..MaxFenceSeq
    /\ inflight \subseteq WritebackClaims
    /\ Cardinality(inflight) <= MaxInflight
    /\ fsRootReleases \in [Extents -> Nat]
    /\ allocatorAdmits \in [Extents -> Nat]
    /\ tokenBasis \in Chunks \cup {"none"}
    /\ cursorLoan \in Chunks \cup {"none"}
    /\ m4Rec \in M4RecStages

M3NoRetiredHistoryGap == M3!NoRetiredHistoryGap
M3DurabilityNoLoss == M3!DurabilityNoLoss
M3AtomicityCleanState == M3!AtomicityCleanState

\* Lease pins, nested overlays and loans never lose a root to reclamation.
StoreGCPinSafety ==
    /\ PinHolds => \A e \in pinnedRoots \cap Extents : phase[e] = "none"
    /\ liveChildPins > 0 => pinState # "None"
    /\ pinState = "None" => pinnedRoots = {}

\* Expired only after every direct loan, child overlay and grandchild drained.
LeaseExpiryNoReaderLoss ==
    pinState = "Expired" =>
        /\ activeReaderLoans = 0
        /\ liveChildPins = 0

\* P0: no in-flight writeback ever targets a range at a different generation
\* or a range handed back to the allocator; every reissued range carries a
\* generation strictly newer than the one it was retired at.
AddressReuseGenerationAndWriteFenceSound ==
    /\ \A w \in inflight :
          \A c \in w.range : cellGen[c] = w.gen /\ cellState[c] # "free"
    /\ \A c \in Cells :
          cellState[c] \in {"free", "fresh"} =>
              /\ GenGT(cellGen[c], OrigGen)
              /\ ~GenGT(cellGen[c], GlobalGen)

\* Every extent under reclamation lies outside the complete Reach set
\* (catalog incl. archived roots, pins + transitive overlays + loans, cursor
\* loans, in-flight candidate decisions).
ReclaimPreservesAllReachableRoots ==
    /\ \A e \in Extents : phase[e] # "none" => e \notin Reach
    /\ \A c \in Cells : cellState[c] # "live" => ExtentOf(c) \notin Reach

\* Nothing is pruned without a sealed canonical checkpoint covering it.
CanonicalCoverageBeforePrune ==
    /\ \A e \in PathAExtents : phase[e] # "none" => cov[e] = "covered"
    /\ phase["cand"] = "admitted" => cov["cand"] = "covered"

\* Interrupted (and chained-crash interrupted) reclamation resumes without
\* double release: each extent reaches the free-space root and the allocator
\* at most once, and cell states agree with the durable retirement record.
InterruptedReclaimIdempotent ==
    /\ \A e \in Extents : fsRootReleases[e] <= 1 /\ allocatorAdmits[e] <= 1
    /\ \A e \in PathAExtents :
          fsRootReleases[e] =
              IF phase[e] \in {"fsroot", "fsroot_reopened", "complete", "admitted"}
              THEN 1 ELSE 0
    /\ \A e \in Extents : allocatorAdmits[e] = IF phase[e] = "admitted" THEN 1 ELSE 0
    /\ \A c \in Cells :
          /\ cellState[c] = "live" <=> phase[ExtentOf(c)] = "none"
          /\ cellState[c] = "retired" <=> phase[ExtentOf(c)] \in RetiringPhases
          /\ cellState[c] \in {"free", "fresh"} <=> phase[ExtentOf(c)] = "admitted"

\* Retired names stay retired and never carry a candidate or pending commit.
NoBranchResurrection ==
    \A b \in retiredEver :
        /\ branchState[b] = "retired"
        /\ pendingBranch # b

\* A live cursor loan always reads the token's exact basis, which is present.
RetainedBasisCursorImmutable ==
    cursorLoan # "none" =>
        /\ cursorLoan = tokenBasis
        /\ (cursorLoan \in Extents => phase[cursorLoan] = "none")

\* Every branch frontier component is either the shared base or that
\* branch's own delta (no sibling overwrite).
HomeFrontierComposed ==
    \A b \in Branches : head[b] \in {"base", NewRoot(b)}

\* Recovery maintenance and serving only run after the PinTable is restored.
PinsRestoredBeforeMaintenance ==
    /\ m4Rec # "none" => (crashed /\ recoveryStage = "manifest_switch")
    /\ (~crashed \/ m4Rec \in {"pins_restored", "fsroot_resumed", "complete_resumed"}) => pinsRestored

\* ===========================================================================
\* Temporal (action) safety properties
\* ===========================================================================

M3MonotonicDurability == M3!MonotonicDurability

GenerationMonotone ==
    [][ /\ rootPubGen' >= rootPubGen
        /\ fenceSeq' >= fenceSeq
        /\ \A c \in Cells : ~GenGT(cellGen[c], cellGen'[c]) ]_vars

NoBranchResurrectionStep ==
    [][ \A b \in Branches : branchState[b] = "retired" => branchState'[b] = "retired" ]_vars

HomeFrontierMonotone ==
    [][ \A b \in Branches : head[b] = NewRoot(b) => head'[b] = NewRoot(b) ]_vars

=============================================================================
