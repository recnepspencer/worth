------------------------ MODULE StoreDurabilityTrace ------------------------
EXTENDS StoreDurability, TraceData

(*
  Trace-validation refinement wrapper for StoreDurability.tla (Store M3 C6).
  Rust DST runs emit NDJSON records from observed Store barriers;
  `tools/tla/run_tlc.py trace` compiles them into TraceData.tla and invokes
  TLC on this spec to prove every real Store transition refines
  StoreDurability.tla.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

StepAction(step) ==
    CASE step.action = "BeginCommit" -> BeginCommit
      [] step.action = "FlushJournal" -> FlushJournal
      [] step.action = "SealManifest" -> SealManifest
      [] step.action = "BeginCheckpoint" -> BeginCheckpoint(step.epoch)
      [] step.action = "SealCheckpoint" -> SealCheckpoint
      [] step.action = "RetireHistoryPrefix" -> RetireHistoryPrefix(step.newFloor)
      [] step.action = "DiscardDerivedState" -> DiscardDerivedState
      [] step.action = "GracefulReopen" -> GracefulReopen(step.buildId)
      [] step.action = "BeginForkCreate" -> BeginForkCreate
      [] step.action = "ForkNeedsReopen" -> ForkNeedsReopen
      [] step.action = "ForkReopenBusy" -> ForkReopenBusy
      [] step.action = "ForkBusyRetryClear" -> ForkBusyRetryClear
      [] step.action = "ReopenForkToPublished" -> ReopenForkToPublished
      [] step.action = "PublishForkDirect" -> PublishForkDirect
      [] step.action = "CrashKill" -> CrashKill
      [] step.action = "BeginRecover" -> BeginRecover(step.buildId)
      [] step.action = "AdvanceRecoverStage" -> AdvanceRecoverStage(step.resolvedHead)
      [] step.action = "FinishRecover" -> FinishRecover

MatchesPost(step) ==
    /\ durableHead' = step.postDurableHead
    /\ checkpointEpoch' = step.postCheckpointEpoch
    /\ stagedCheckpoint' = step.postStagedCheckpoint
    /\ retainedFloor' = step.postRetainedFloor
    /\ pendingWrite' = step.postPendingWrite
    /\ activeLease' = step.postActiveLease
    /\ forkPhase' = step.postForkPhase
    /\ crashed' = step.postCrashed
    /\ recoveryStage' = step.postRecoveryStage
    /\ buildId' = step.postBuildId
    /\ derivedState' = step.postDerivedState
    /\ recoveredHead' = step.postRecoveredHead
    /\ reopenSucceeded' = step.postReopenSucceeded

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ MatchesPost(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
