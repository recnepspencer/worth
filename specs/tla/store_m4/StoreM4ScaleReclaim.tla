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
    reclaimedEpochs     \* Set of epochs physically reclaimed

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

m4Vars == <<pinState, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs>>
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
    /\ UNCHANGED <<pinState, pinnedEpoch, liveChildPins, reclaimedEpochs>>

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
    /\ UNCHANGED <<m3Vars, reclaimedEpochs>>

AcquireReaderLoan ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ activeReaderLoans = 0
    /\ activeReaderLoans' = 1
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, liveChildPins, reclaimedEpochs>>

ReleaseReaderLoan ==
    /\ ~crashed
    /\ activeReaderLoans = 1
    /\ activeReaderLoans' = 0
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, liveChildPins, reclaimedEpochs>>

AttachChildPin ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ liveChildPins = 0
    /\ liveChildPins' = 1
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, reclaimedEpochs>>

ReleaseChildPin ==
    /\ ~crashed
    /\ liveChildPins = 1
    /\ liveChildPins' = 0
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, reclaimedEpochs>>

\* Active -> Expiring atomically fences NEW reader loans & child pins while
\* keeping `pinnedEpoch` protected until loans & child pins drain.
BeginExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Active"
    /\ pinState' = "Expiring"
    /\ UNCHANGED <<m3Vars, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs>>

\* Expiring -> Expired strictly requires zero active reader loans AND zero live child pins.
FinishExpireForkLease ==
    /\ ~crashed
    /\ pinState = "Expiring"
    /\ activeReaderLoans = 0
    /\ liveChildPins = 0
    /\ pinState' = "Expired"
    /\ UNCHANGED <<m3Vars, pinnedEpoch, activeReaderLoans, liveChildPins, reclaimedEpochs>>

\* Reclaim an obsolete epoch strictly outside ReachableEpochs.
ReclaimEpoch(ep) ==
    /\ ~crashed
    /\ ep \in Epochs
    /\ ep < retainedFloor
    /\ ~(ep \in ReachableEpochs)
    /\ ~(ep \in reclaimedEpochs)
    /\ reclaimedEpochs' = reclaimedEpochs \cup {ep}
    /\ UNCHANGED <<m3Vars, pinState, pinnedEpoch, activeReaderLoans, liveChildPins>>

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

M3NoRetiredHistoryGap == M3!NoRetiredHistoryGap

\* StoreGCPinSafety: no reachable epoch (including any Active or Expiring pin)
\* is ever reclaimed.
StoreGCPinSafety ==
    reclaimedEpochs \cap ReachableEpochs = {}

\* LeaseExpiryNoReaderLoss: a lease never reaches Expired while any reader loan
\* or nested child pin remains live.
LeaseExpiryNoReaderLoss ==
    (pinState = "Expired") => (activeReaderLoans = 0 /\ liveChildPins = 0)

\* Non-vacuity reachability witness predicate (verified via StoreM4ScaleReclaimWitness.cfg):
\* Proves the model reaches a state where a pinned epoch survived expiry while held,
\* then expired cleanly after draining, and was reclaimed afterward.
NonVacuityPinReclaimWitness ==
    pinState = "Expired" /\ pinnedEpoch \in reclaimedEpochs

\* Negation used by the dedicated non-vacuity witness check (must produce a trace!).
WitnessNotReached == ~NonVacuityPinReclaimWitness

=============================================================================
