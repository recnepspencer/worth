--------------------------- MODULE StoreDurability --------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Formal specification for Public WORTH Store Milestone 3 (M3 C6 Formal Gate).

  Models the intersection of Store M3 C1-C5 durability protocols:
  - Atomic commits across explicit I/O/crash barriers:
      BeginCommit -> FlushJournal -> SealManifest
  - Indeterminate pre-manifest crash barrier (FailBefore vs
    IndeterminateAfterEffect): crashing at journal_flushed_pre_manifest lets
    recovery resolve to either durableHead or pendingTarget, while crashing at
    journal_unflushed MUST resolve to durableHead. Once a decision settles at
    manifest_switch or completes reopen, no chained crash or subsequent reopen
    may flip or regress durableHead / recoveredHead.
  - Multi-step interruptible recovery & chained crashes (C4b.2 Bugs #6 & #7):
      CrashKill -> BeginRecover -> AdvanceRecoverStage -> FinishRecover
    with CrashKill enabled mid-recovery (staging / manifest_switch) up to
    MaxCrashes so safety covers chained crashes and liveness checks progress
    under fair, eventually crash-free retry assumptions.
  - Staged vs sealed checkpoint durability horizon (C5.2 D13 retirement) &
    nested-lease custody:
      BeginCheckpoint -> SealCheckpoint
    RetireHistoryPrefix is strictly bounded by the SEALED checkpointEpoch
    (never stagedCheckpoint), eliminating RetiredHistoryGap across
    mid-checkpoint crashes, and works both stand-alone (activeLease = "none")
    and nested under an active commit lease (activeLease = "commit_lease").
  - Orthogonal build-identity alternation & D14 derived-state discard (C4b.3):
    DiscardDerivedState moves derivedState to "discarded_blocked", which
    STRICTLY blocks in-session BeginCheckpoint (matching Store's
    DerivedStateDiscarded denial) until GracefulReopen or FinishRecover moves
    derivedState to "discarded_reopened", where a fresh checkpoint restores
    "covered".
  - Durable world/fork creation (fork_expected) with transient Busy blocking:
      Idle -> Creating -> ReopenRequired <-> ReopenBusy -> Published
    proving ForkEventuallyPublishes under fair retry (WF on clearing Busy,
    SF on ReopenForkToPublished).
*)

CONSTANTS
    MaxCommits,     \* e.g. 3 for model checking, 64 for trace validation
    MaxCrashes,     \* e.g. 3 for model checking (covers chained crashes), 128 for traces
    BuildIds,       \* e.g. {"b1", "b2"} for model checking, {"b1", "b2", "b3"} for traces
    InitialBuild    \* Explicit starting build identity (member of BuildIds)

ASSUME /\ MaxCommits >= 2
       /\ MaxCrashes >= 2
       /\ Cardinality(BuildIds) >= 2
       /\ InitialBuild \in BuildIds

Epochs == 0..MaxCommits
CrashCounts == 0..MaxCrashes
PendingKinds == {"none", "journal_unflushed", "journal_flushed_pre_manifest"}
LeaseModes == {"none", "commit_lease"}
ForkPhases == {"Idle", "Creating", "ReopenRequired", "ReopenBusy", "Published"}
RecoveryStages == {"none", "staging", "manifest_switch"}
DerivedStates == {"covered", "discarded_blocked", "discarded_reopened"}

VARIABLES
    durableHead,        \* Highest acknowledged (or settled-recovered) durable epoch
    checkpointEpoch,    \* Latest SEALED durable checkpoint epoch (<= durableHead)
    stagedCheckpoint,   \* In-flight unsealed checkpoint target (0 when none staged)
    retainedFloor,      \* Lowest epoch still retained in durable history log
    pendingWrite,       \* Current in-flight commit I/O barrier phase
    pendingTarget,      \* Target epoch of in-flight commit (0 when pendingWrite = "none")
    activeLease,        \* Active commit lease custody ("none" | "commit_lease")
    forkPhase,          \* Durable world creation / fork_expected phase (incl. ReopenBusy)
    crashed,            \* TRUE while crashed or mid-recovery until FinishRecover
    recoveryStage,      \* Multi-step recovery stage ("none" | "staging" | "manifest_switch")
    crashCount,         \* Crashes taken so far (<= MaxCrashes; bounds fault horizon)
    buildId,            \* Active ApplicationBuildIdentity
    derivedState,       \* D14 derived-state admission posture ("covered" | "discarded_blocked" | "discarded_reopened")
    recoveredHead,      \* Highest durableHead observed on completed reopen
    reopenSucceeded     \* TRUE iff latest reopen walked intact history to checkpointEpoch

vars == <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
          pendingWrite, pendingTarget, activeLease, forkPhase,
          crashed, recoveryStage, crashCount, buildId, derivedState,
          recoveredHead, reopenSucceeded>>

Init ==
    /\ durableHead = 0
    /\ checkpointEpoch = 0
    /\ stagedCheckpoint = 0
    /\ retainedFloor = 0
    /\ pendingWrite = "none"
    /\ pendingTarget = 0
    /\ activeLease = "none"
    /\ forkPhase = "Idle"
    /\ crashed = FALSE
    /\ recoveryStage = "none"
    /\ crashCount = 0
    /\ buildId = InitialBuild
    /\ derivedState = "covered"
    /\ recoveredHead = 0
    /\ reopenSucceeded = TRUE

\* ---------------------------------------------------------------------------
\* 1. Commit protocol across explicit I/O barriers (C4b fault-injection seams)
\* ---------------------------------------------------------------------------

\* Begin a commit under a root commit lease: writes unflushed journal bytes.
BeginCommit ==
    /\ ~crashed
    /\ pendingWrite = "none"
    /\ stagedCheckpoint = 0
    /\ activeLease = "none"
    /\ durableHead < MaxCommits
    /\ pendingWrite' = "journal_unflushed"
    /\ pendingTarget' = durableHead + 1
    /\ activeLease' = "commit_lease"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   forkPhase, crashed, recoveryStage, crashCount, buildId,
                   derivedState, recoveredHead, reopenSucceeded>>

\* fsync journal barrier (durable on disk, manifest pointer not yet swapped).
FlushJournal ==
    /\ ~crashed
    /\ pendingWrite = "journal_unflushed"
    /\ pendingWrite' = "journal_flushed_pre_manifest"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingTarget, activeLease, forkPhase, crashed, recoveryStage,
                   crashCount, buildId, derivedState, recoveredHead, reopenSucceeded>>

\* Atomic manifest seal: acknowledges durableHead' = pendingTarget, releases lease.
SealManifest ==
    /\ ~crashed
    /\ pendingWrite = "journal_flushed_pre_manifest"
    /\ durableHead' = pendingTarget
    /\ pendingWrite' = "none"
    /\ pendingTarget' = 0
    /\ activeLease' = "none"
    /\ UNCHANGED <<checkpointEpoch, stagedCheckpoint, retainedFloor, forkPhase,
                   crashed, recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

\* ---------------------------------------------------------------------------
\* 2. Staged vs Sealed Checkpointing & Bounded History Retirement (C5.2 D13)
\* ---------------------------------------------------------------------------

\* Stage a new checkpoint for epoch ep (unsealed until SealCheckpoint).
\* D14 law: strictly BLOCKED while derivedState = "discarded_blocked" (in-session
\* checkpointing refused with DerivedStateDiscarded until reopen).
BeginCheckpoint(ep) ==
    /\ ~crashed
    /\ pendingWrite = "none"
    /\ stagedCheckpoint = 0
    /\ derivedState # "discarded_blocked"
    /\ ep \in IF derivedState = "discarded_reopened" /\ checkpointEpoch > 0
              THEN checkpointEpoch..durableHead
              ELSE (checkpointEpoch + 1)..durableHead
    /\ stagedCheckpoint' = ep
    /\ UNCHANGED <<durableHead, checkpointEpoch, retainedFloor, pendingWrite,
                   pendingTarget, activeLease, forkPhase, crashed, recoveryStage,
                   crashCount, buildId, derivedState, recoveredHead, reopenSucceeded>>

\* Atomically seal the staged checkpoint, advancing checkpointEpoch and
\* restoring derived checkpoint coverage ("covered").
SealCheckpoint ==
    /\ ~crashed
    /\ stagedCheckpoint > 0
    /\ stagedCheckpoint <= durableHead
    /\ (stagedCheckpoint > checkpointEpoch \/ derivedState = "discarded_reopened")
    /\ checkpointEpoch' = stagedCheckpoint
    /\ stagedCheckpoint' = 0
    /\ derivedState' = "covered"
    /\ UNCHANGED <<durableHead, retainedFloor, pendingWrite, pendingTarget,
                   activeLease, forkPhase, crashed, recoveryStage, crashCount,
                   buildId, recoveredHead, reopenSucceeded>>

\* Retire durable history prefix up to newFloor.
\* Key C5 laws:
\*  - STRICTLY bounded by SEALED checkpointEpoch (NEVER stagedCheckpoint!),
\*    preventing RetiredHistoryGap even across mid-checkpoint crashes.
\*  - MAY run stand-alone (activeLease = "none") OR nested under an active
\*    commit lease (activeLease = "commit_lease").
RetireHistoryPrefix(newFloor) ==
    /\ ~crashed
    /\ pendingWrite \in {"none", "journal_unflushed", "journal_flushed_pre_manifest"}
    /\ newFloor \in (retainedFloor + 1)..checkpointEpoch
    /\ retainedFloor' = newFloor
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, pendingWrite,
                   pendingTarget, activeLease, forkPhase, crashed, recoveryStage,
                   crashCount, buildId, derivedState, recoveredHead, reopenSucceeded>>

\* ---------------------------------------------------------------------------
\* 3. D14 Derived-State Discard & Orthogonal Build-Identity Reopen (C4b.3)
\* ---------------------------------------------------------------------------

\* Discard derived checkpoint/index/signal caches (D14). Blocks in-session
\* checkpointing ("discarded_blocked") until reopen without touching
\* authoritative durableHead, checkpointEpoch, or retainedFloor.
DiscardDerivedState ==
    /\ ~crashed
    /\ pendingWrite = "none"
    /\ stagedCheckpoint = 0
    /\ derivedState = "covered"
    /\ derivedState' = "discarded_blocked"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, forkPhase, crashed,
                   recoveryStage, crashCount, buildId, recoveredHead, reopenSucceeded>>

\* Graceful close & fresh reopen under nextBuild. Reopen clears the in-session
\* D14 admission block ("discarded_blocked" -> "discarded_reopened") so a fresh
\* checkpoint can restore coverage.
GracefulReopen(nextBuild) ==
    /\ ~crashed
    /\ pendingWrite = "none"
    /\ stagedCheckpoint = 0
    /\ activeLease = "none"
    /\ nextBuild \in BuildIds
    /\ LET canWalk == (retainedFloor <= checkpointEpoch) /\ (checkpointEpoch <= durableHead)
       IN
       /\ buildId' = nextBuild
       /\ derivedState' = IF derivedState = "discarded_blocked"
                          THEN "discarded_reopened"
                          ELSE derivedState
       /\ recoveredHead' = durableHead
       /\ reopenSucceeded' = canWalk
       /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                      pendingWrite, pendingTarget, activeLease, forkPhase, crashed,
                      recoveryStage, crashCount>>

\* ---------------------------------------------------------------------------
\* 4. Durable World Creation / fork_expected Lifecycle (with transient Busy)
\* ---------------------------------------------------------------------------

BeginForkCreate ==
    /\ ~crashed
    /\ forkPhase \in {"Idle", "Published"}
    /\ forkPhase' = "Creating"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

ForkNeedsReopen ==
    /\ ~crashed
    /\ forkPhase = "Creating"
    /\ forkPhase' = "ReopenRequired"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

\* Transient Busy blocking state while physical fork write is Pending.
ForkReopenBusy ==
    /\ ~crashed
    /\ forkPhase = "ReopenRequired"
    /\ forkPhase' = "ReopenBusy"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

\* Pending physical fork write settles, unblocking retry back to ReopenRequired.
ForkBusyRetryClear ==
    /\ ~crashed
    /\ forkPhase = "ReopenBusy"
    /\ forkPhase' = "ReopenRequired"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

ReopenForkToPublished ==
    /\ ~crashed
    /\ forkPhase = "ReopenRequired"
    /\ forkPhase' = "Published"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

PublishForkDirect ==
    /\ ~crashed
    /\ forkPhase = "Creating"
    /\ forkPhase' = "Published"
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, crashed,
                   recoveryStage, crashCount, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

\* ---------------------------------------------------------------------------
\* 5. Adversarial Crash / Chained Recovery Kills & Multi-Step Recovery (C4b.2)
\* ---------------------------------------------------------------------------

\* Process killed at ANY live point OR mid-recovery (staging / manifest_switch)
\* while crash budget permits. Unsealed stagedCheckpoint is lost on crash;
\* Creating or ReopenBusy fork enters ReopenRequired cleanly.
CrashKill ==
    /\ crashCount < MaxCrashes
    /\ (~crashed) \/ (recoveryStage \in {"staging", "manifest_switch"})
    /\ crashed' = TRUE
    /\ recoveryStage' = "none"
    /\ crashCount' = crashCount + 1
    /\ stagedCheckpoint' = 0
    /\ forkPhase' = IF forkPhase \in {"Creating", "ReopenBusy"}
                    THEN "ReopenRequired"
                    ELSE forkPhase
    /\ UNCHANGED <<durableHead, checkpointEpoch, retainedFloor, pendingWrite,
                   pendingTarget, activeLease, buildId, derivedState,
                   recoveredHead, reopenSucceeded>>

\* Stage 1 of interruptible recovery: start reopen under nextBuild.
BeginRecover(nextBuild) ==
    /\ crashed
    /\ recoveryStage = "none"
    /\ nextBuild \in BuildIds
    /\ recoveryStage' = "staging"
    /\ buildId' = nextBuild
    /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                   pendingWrite, pendingTarget, activeLease, forkPhase, crashed,
                   crashCount, derivedState, recoveredHead, reopenSucceeded>>

\* Stage 2 of interruptible recovery: resolve in-flight WAL/manifest barrier.
\* - journal_unflushed -> discarded (atomicity: strictly resolves to durableHead)
\* - journal_flushed_pre_manifest -> indeterminate barrier: resolves to either
\*   FailBefore (durableHead) OR IndeterminateAfterEffect (pendingTarget)
\* - Once manifest_switch settles durableHead', any chained CrashKill preserves it.
AdvanceRecoverStage(resolvedHead) ==
    /\ crashed
    /\ recoveryStage = "staging"
    /\ resolvedHead \in
           IF pendingWrite = "journal_flushed_pre_manifest"
           THEN {durableHead, pendingTarget}
           ELSE {durableHead}
    /\ durableHead' = resolvedHead
    /\ pendingWrite' = "none"
    /\ pendingTarget' = 0
    /\ activeLease' = "none"
    /\ recoveryStage' = "manifest_switch"
    /\ UNCHANGED <<checkpointEpoch, stagedCheckpoint, retainedFloor, forkPhase,
                   crashed, crashCount, buildId, derivedState, recoveredHead,
                   reopenSucceeded>>

\* Stage 3 of interruptible recovery: walk activation history from durableHead
\* back to checkpointEpoch and hand off to serving (unblocking D14 session gate).
FinishRecover ==
    /\ crashed
    /\ recoveryStage = "manifest_switch"
    /\ LET canWalk == (retainedFloor <= checkpointEpoch) /\ (checkpointEpoch <= durableHead)
       IN
       /\ recoveredHead' = durableHead
       /\ reopenSucceeded' = canWalk
       /\ derivedState' = IF derivedState = "discarded_blocked"
                          THEN "discarded_reopened"
                          ELSE derivedState
       /\ crashed' = FALSE
       /\ recoveryStage' = "none"
       /\ UNCHANGED <<durableHead, checkpointEpoch, stagedCheckpoint, retainedFloor,
                      pendingWrite, pendingTarget, activeLease, forkPhase,
                      crashCount, buildId>>

Next ==
    \/ BeginCommit
    \/ FlushJournal
    \/ SealManifest
    \/ \E ep \in Epochs: BeginCheckpoint(ep)
    \/ SealCheckpoint
    \/ \E fl \in Epochs: RetireHistoryPrefix(fl)
    \/ DiscardDerivedState
    \/ \E b \in BuildIds: GracefulReopen(b)
    \/ BeginForkCreate
    \/ ForkNeedsReopen
    \/ ForkReopenBusy
    \/ ForkBusyRetryClear
    \/ ReopenForkToPublished
    \/ PublishForkDirect
    \/ CrashKill
    \/ \E b \in BuildIds: BeginRecover(b)
    \/ \E rh \in Epochs: AdvanceRecoverStage(rh)
    \/ FinishRecover

\* Explicit fairness under crash-free retry horizon (crashCount <= MaxCrashes):
\*  - Recovery steps make weak-fair progress whenever crashed
\*  - Transient Busy fork writes eventually settle (WF)
\*  - Retrying ReopenForkToPublished is strongly fair so ReopenRequired <->
\*    ReopenBusy cannot starve publication
RecoveryProgress ==
    (\E b \in BuildIds : BeginRecover(b))
    \/ (\E rh \in Epochs : AdvanceRecoverStage(rh))
    \/ FinishRecover

Spec ==
    /\ Init
    /\ [][Next]_vars
    /\ WF_vars(RecoveryProgress)
    /\ WF_vars(ForkBusyRetryClear)
    /\ SF_vars(ReopenForkToPublished)

\* ===========================================================================
\* Safety Invariants
\* ===========================================================================

TypeOK ==
    /\ durableHead \in Epochs
    /\ checkpointEpoch \in Epochs
    /\ stagedCheckpoint \in Epochs
    /\ retainedFloor \in Epochs
    /\ pendingWrite \in PendingKinds
    /\ pendingTarget \in Epochs
    /\ activeLease \in LeaseModes
    /\ forkPhase \in ForkPhases
    /\ crashed \in BOOLEAN
    /\ recoveryStage \in RecoveryStages
    /\ crashCount \in CrashCounts
    /\ buildId \in BuildIds
    /\ derivedState \in DerivedStates
    /\ recoveredHead \in Epochs
    /\ reopenSucceeded \in BOOLEAN

\* 1. Horizon safety: NEVER retire past SEALED checkpointEpoch or durableHead,
\*    stagedCheckpoint respects horizon bounds, and reopen walks intact history.
NoRetiredHistoryGap ==
    /\ retainedFloor <= checkpointEpoch
    /\ checkpointEpoch <= durableHead
    /\ (   stagedCheckpoint = 0
        \/ (   stagedCheckpoint <= durableHead
            /\ (   stagedCheckpoint > checkpointEpoch
                \/ (derivedState = "discarded_reopened" /\ stagedCheckpoint = checkpointEpoch /\ stagedCheckpoint > 0))))
    /\ reopenSucceeded = TRUE

\* 2. Durability: no acknowledged or recovered commit is ever lost across
\*    crash, chained recovery kills, build changes, or D14 derived discards.
DurabilityNoLoss ==
    recoveredHead <= durableHead

\* 3. Atomicity: no torn write/lease/recovery/staging state.
AtomicityCleanState ==
    /\ (pendingWrite = "none") <=> (pendingTarget = 0)
    /\ (pendingWrite = "none") <=> (activeLease = "none")
    /\ (~crashed) => (recoveryStage = "none")
    /\ (crashed /\ recoveryStage = "manifest_switch") => (pendingWrite = "none" /\ activeLease = "none")
    /\ crashed => (stagedCheckpoint = 0)

\* 4. D14 admission law: in-session checkpointing is strictly blocked after
\*    DiscardDerivedState until reopen transitions derivedState to
\*    "discarded_reopened".
D14BlocksInSessionCheckpoint ==
    (derivedState = "discarded_blocked") =>
        (stagedCheckpoint = 0 /\ ~(\E ep \in Epochs : ENABLED BeginCheckpoint(ep)))

\* ===========================================================================
\* Temporal Safety & Liveness Properties (checked under fair, crash-free retry)
\* ===========================================================================

\* 5. Temporal monotonicity: durableHead, checkpointEpoch, retainedFloor, and
\*    recoveredHead never regress across crashes, chained kills, or reopens.
MonotonicDurability ==
    [][ /\ durableHead' >= durableHead
        /\ checkpointEpoch' >= checkpointEpoch
        /\ retainedFloor' >= retainedFloor
        /\ recoveredHead' >= recoveredHead
      ]_vars

\* 6. Recovery liveness: every crashed state (including after chained crashes)
\*    eventually completes recovery with intact history.
EventuallyRecovers ==
    crashed ~> (~crashed /\ reopenSucceeded)

\* 7. Fork reopen liveness: even with transient ReopenBusy blocking and
\*    mid-recovery crashes, every fork in ReopenRequired or ReopenBusy
\*    eventually reaches Published (proves zero Busy livelock).
ForkEventuallyPublishes ==
    (forkPhase \in {"ReopenRequired", "ReopenBusy"}) ~> (forkPhase = "Published")

=============================================================================
