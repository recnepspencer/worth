------------------------ MODULE StoreM4ScaleReclaim -------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Milestone 4 Phase M4.0 Composed TLA+ Skeleton.

  Composes directly with `specs/tla/store_m3/StoreDurability.tla` via
  `tools/tla/run_tlc.py`'s `-DTLA-Library` cross-directory module resolution.

  Establishes the M4 pin/lease reachability and safe reclamation seam coupled
  to M3's commit, checkpoint, history-retirement, and interruptible crash /
  recovery barriers:
  - PublishForkPin: acquires a durable `Active` pin at `durableHead` under
    Store's reclamation fence (with optional nested child overlay + reader loan).
  - BeginExpireForkLease / FinishExpireForkLease: enforces
    `Active -> Expiring -> Expired` where `Expiring` fences new reader loans
    and `Expired` is reachable ONLY once `activeReaderLoans = 0 /\ liveChildPins = 0`.
  - ReclaimEpoch: reclaims an obsolete epoch strictly outside live head,
    checkpoint horizon, and active/expiring pin reachability (`StoreGCPinSafety`).
  - CrashAndRecover: composes M3's `CrashKill` / `BeginRecover` /
    `AdvanceRecoverStage` / `FinishRecover` while dropping volatile in-process
    reader loans on crash and preserving durable pins across reopen.
*)

CONSTANTS
    MaxCommits,
    MaxCrashes,
    BuildIds,
    InitialBuild

VARIABLES
    \* StoreDurability (M3) state variables
    durableHead,
    checkpointEpoch,
    stagedCheckpoint,
    retainedFloor,
    pendingWrite,
    pendingTarget,
    activeLease,
    forkPhase,
    crashed,
    recoveryStage,
    crashCount,
    buildId,
    derivedState,
    recoveredHead,
    reopenSucceeded,
    \* Milestone 4 pin/lease & reclamation state variables
    pinState,           \* "None" | "Active" | "Expiring" | "Expired"
    pinnedEpoch,        \* Epoch pinned by durable PinTable (0 when "None")
    activeReaderLoans,  \* Volatile in-process reader loans against the pin (0..1)
    liveChildPins,      \* Durable nested child overlay pins referencing this pin (0..1)
    reclaimedEpochs,    \* Set of epochs physically reclaimed
    everHeldWhileExpiring \* History flag: pin entered Expiring while held by a loan/child pin

M3 == INSTANCE StoreDurability
      WITH MaxCommits <- MaxCommits,
           MaxCrashes <- MaxCrashes,
           BuildIds <- BuildIds,
           InitialBuild <- InitialBuild,
           durableHead <- durableHead,
           checkpointEpoch <- checkpointEpoch,
           stagedCheckpoint <- stagedCheckpoint,
           retainedFloor <- retainedFloor,
           pendingWrite <- pendingWrite,
           pendingTarget <- pendingTarget,
           activeLease <- activeLease,
           forkPhase <- forkPhase,
           crashed <- crashed,
           recoveryStage <- recoveryStage,
           crashCount <- crashCount,
           buildId <- buildId,
           derivedState <- derivedState,
           recoveredHead <- recoveredHead,
           reopenSucceeded <- reopenSucceeded

m3Vars == <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
            pendingWrite, pendingTarget, activeLease, forkPhase,
            crashed, recoveryStage, crashCount, buildId, derivedState,
            recoveredHead, reopenSucceeded>>

m4Vars == <<pinState, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs, everHeldWhileExpiring>>
vars == <<m3Vars, m4Vars>>

PinStates == {"None", "Active", "Expiring", "Expired"}
Epochs == 0..MaxCommits

PinProtects(ep) ==
    pinState \in {"Active", "Expiring"} /\ pinnedEpoch = ep

ReachableEpochs ==
    { ep \in Epochs :
        \/ ep >= retainedFloor
        \/ ep >= checkpointEpoch
        \/ ep = durableHead
        \/ PinProtects(ep) }

Init ==
    /\ M3!Init
    /\ pinState = "None"
    /\ pinnedEpoch = 0
    /\ activeReaderLoans = 0
    /\ liveChildPins = 0
    /\ reclaimedEpochs = {}
    /\ everHeldWhileExpiring = FALSE

\* ---------------------------------------------------------------------------
\* Coupled M3 Durability Actions (preserving M4 durable pin/reclaim state)
\* ---------------------------------------------------------------------------

M3Step ==
    /\ ( \/ M3!BeginCommit
         \/ M3!FlushJournal
         \/ M3!SealManifest
         \/ \E ep \in Epochs : M3!BeginCheckpoint(ep)
         \/ M3!SealCheckpoint
         \/ \E fl \in Epochs : M3!RetireHistoryPrefix(fl)
         \/ \E b \in BuildIds : M3!BeginRecover(b)
         \/ \E rh \in Epochs : M3!AdvanceRecoverStage(rh)
         \/ M3!FinishRecover )
    /\ UNCHANGED m4Vars

\* Crash drops volatile in-process reader loans while preserving durable pins.
CrashWithVolatileDrop ==
    /\ M3!CrashKill
    /\ activeReaderLoans' = 0
    /\ UNCHANGED <<pinState, pinnedEpoch, liveChildPins, reclaimedEpochs, everHeldWhileExpiring>>

\* ---------------------------------------------------------------------------
\* M4 PinTable, Lease Expiry & Reachability-Guarded Reclamation Actions
\* ---------------------------------------------------------------------------

PublishForkPin ==
    /\ ~crashed
    /\ pinState \in {"None", "Expired"}
    /\ ~(durableHead \in reclaimedEpochs)
    /\ pinState' = "Active"
    /\ pinnedEpoch' = durableHead
    /\ activeReaderLoans' = 0
    /\ liveChildPins' = 0
    /\ UNCHANGED <<m3Vars, reclaimedEpochs, everHeldWhileExpiring>>

AcquireReaderLoan ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ activeReaderLoans = 0
    /\ activeReaderLoans' = 1
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, liveChildPins, reclaimedEpochs, everHeldWhileExpiring>>

ReleaseReaderLoan ==
    /\ ~crashed
    /\ activeReaderLoans = 1
    /\ activeReaderLoans' = 0
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, liveChildPins, reclaimedEpochs, everHeldWhileExpiring>>

AttachChildPin ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ liveChildPins = 0
    /\ liveChildPins' = 1
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, reclaimedEpochs, everHeldWhileExpiring>>

ReleaseChildPin ==
    /\ ~crashed
    /\ liveChildPins = 1
    /\ liveChildPins' = 0
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, reclaimedEpochs, everHeldWhileExpiring>>

\* Active -> Expiring atomically fences NEW reader loans & child pins while
\* keeping `pinnedEpoch` protected until loans & child pins drain.
BeginExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ pinState' = "Expiring"
    /\ everHeldWhileExpiring' = (everHeldWhileExpiring \/ activeReaderLoans > 0 \/ liveChildPins > 0)
    /\ UNCHANGED <<m3Vars, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs>>

\* Expiring -> Expired strictly requires zero active reader loans AND zero live child pins.
FinishExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Expiring"
    /\ activeReaderLoans = 0
    /\ liveChildPins = 0
    /\ pinState' = "Expired"
    /\ UNCHANGED <<m3Vars, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs, everHeldWhileExpiring>>

\* Reclaim an obsolete epoch strictly outside ReachableEpochs.
ReclaimEpoch(ep) ==
    /\ ~crashed
    /\ ep \in Epochs
    /\ ep < retainedFloor
    /\ ~(ep \in ReachableEpochs)
    /\ ~(ep \in reclaimedEpochs)
    /\ reclaimedEpochs' = reclaimedEpochs \cup {ep}
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, liveChildPins, everHeldWhileExpiring>>

Next ==
    \/ M3Step
    \/ CrashWithVolatileDrop
    \/ PublishForkPin
    \/ AcquireReaderLoan
    \/ ReleaseReaderLoan
    \/ AttachChildPin
    \/ ReleaseChildPin
    \/ BeginExpireForkLease
    \/ FinishExpireForkLease
    \/ \E ep \in Epochs : ReclaimEpoch(ep)

Spec == Init /\ [][Next]_vars

\* ===========================================================================
\* Safety Invariants (Composed M3 + M4 Laws)
\* ===========================================================================

TypeOK ==
    /\ M3!TypeOK
    /\ pinState \in PinStates
    /\ pinnedEpoch \in Epochs
    /\ activeReaderLoans \in 0..1
    /\ liveChildPins \in 0..1
    /\ reclaimedEpochs \subseteq Epochs
    /\ everHeldWhileExpiring \in BOOLEAN

M3NoRetiredHistoryGap == M3!NoRetiredHistoryGap

\* StoreGCPinSafety: independent direct assertion (does NOT rely solely on
\* `ReachableEpochs`) proving no Active or Expiring pinned epoch, nor any live
\* head/checkpoint/retained epoch, is ever reclaimed.
StoreGCPinSafety ==
    /\ (pinState \in {"Active", "Expiring"} => ~(pinnedEpoch \in reclaimedEpochs))
    /\ \A ep \in reclaimedEpochs :
        /\ ep < retainedFloor
        /\ ep < checkpointEpoch
        /\ ep /= durableHead

\* LeaseExpiryNoReaderLoss: a lease never reaches Expired while any reader loan
\* or nested child pin remains live.
LeaseExpiryNoReaderLoss ==
    (pinState = "Expired") => (activeReaderLoans = 0 /\ liveChildPins = 0)

\* Non-vacuity reachability witness predicate (verified via StoreM4ScaleReclaimWitness.cfg):
\* Proves the model reaches a state where a pinned epoch entered Expiring while
\* held by a live reader loan or child pin, survived until drained, expired
\* cleanly afterward, and was then physically reclaimed.
NonVacuityPinReclaimWitness ==
    /\ everHeldWhileExpiring
    /\ pinState = "Expired"
    /\ pinnedEpoch \in reclaimedEpochs

\* Negation used by the dedicated non-vacuity witness check (must produce a trace!).
WitnessNotReached == ~NonVacuityPinReclaimWitness

=============================================================================
