---------------------- MODULE StoreM4ScaleReclaimTrace ----------------------
EXTENDS StoreM4ScaleReclaim, TraceData

(*
  Trace validation for StoreM4ScaleReclaim.tla (Store M4.6).

  `store_m4_dst` writes one NDJSON record per observed production barrier.
  `python tools/tla/run_tlc.py trace StoreM4ScaleReclaimTrace.tla <file>`
  turns the file into TraceData.tla (`Trace == << ... >>`, `{"__set__": [..]}`
  is a set, a JSON list is a tuple) and runs TLC here. A trace passes only if
  every record is a transition the composed model allows from the state
  before it, with exactly the observed post-state. The record schema is in
  README.md; every record carries every post field below.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

Claim(step) == [range |-> step.range, gen |-> step.gen]

StepAction(step) ==
    CASE step.action = "PrepareBranch"            -> PrepareBranch(step.branch)
      [] step.action = "BeginBranchCommit"        -> BeginBranchCommit(step.branch)
      [] step.action = "RefuseStaleCandidate"     -> RefuseStaleCandidate(step.branch)
      [] step.action = "FlushJournal"             -> FlushJournal
      [] step.action = "PublishBranchCommit"      -> PublishBranchCommit
      [] step.action = "BeginCheckpoint"          -> BeginCheckpoint(step.epoch)
      [] step.action = "SealCheckpoint"           -> SealCheckpoint
      [] step.action = "CoverCancelCheckpoint"    -> CoverCancelCheckpoint
      [] step.action = "RetireHistoryPrefix"      -> RetireHistoryPrefix(step.newFloor)
      [] step.action = "ArchiveBranch"            -> ArchiveBranch(step.branch)
      [] step.action = "RetireBranchRef"          -> RetireBranchRef(step.branch)
      [] step.action = "RefuseRetiredBranchName"  -> RefuseRetiredBranchName(step.branch)
      [] step.action = "PublishForkPin"           -> PublishForkPin
      [] step.action = "AcquireReaderLoan"        -> AcquireReaderLoan
      [] step.action = "AcquireGrandchildLoan"    -> AcquireGrandchildLoan
      [] step.action = "ReleaseReaderLoan"        -> ReleaseReaderLoan
      [] step.action = "AttachChildPin"           -> AttachChildPin
      [] step.action = "ReleaseChildPin"          -> ReleaseChildPin
      [] step.action = "BeginExpireForkLease"     -> BeginExpireForkLease
      [] step.action = "FinishExpireForkLease"    -> FinishExpireForkLease
      [] step.action = "WritebackIssue"           -> WritebackIssue(step.range, step.gen)
      [] step.action = "RefuseStaleWriteback"     -> RefuseStaleWriteback(step.range, step.gen)
      [] step.action = "WritebackLand"            -> WritebackLand(Claim(step))
      [] step.action = "WritebackCancel"          -> WritebackCancel(Claim(step))
      [] step.action = "ReclaimIntent"            -> ReclaimIntent(step.extent)
      [] step.action = "PublishFreeSpaceRoot"     -> PublishFreeSpaceRoot(step.extent)
      [] step.action = "ReclaimComplete"          -> ReclaimComplete(step.extent)
      [] step.action = "CancelCandidateExtent"    -> CancelCandidateExtent
      [] step.action = "AdmitToAllocator"         -> AdmitToAllocator(step.extent)
      [] step.action = "AllocateExtent"           -> AllocateExtent(step.range)
      [] step.action = "IssueRetainedBasisToken"  -> IssueRetainedBasisToken(step.branch)
      [] step.action = "ResumeCursor"             -> ResumeCursor
      [] step.action = "ResumeCursorRefused"      -> ResumeCursorRefused
      [] step.action = "ReleaseCursorLoan"        -> ReleaseCursorLoan
      [] step.action = "Crash"                    -> Crash
      [] step.action = "BeginRecover"             -> BeginRecover(step.buildId)
      [] step.action = "RecoverWal"               -> RecoverWal(step.resolvedHead)
      [] step.action = "RecoverRestorePins"       -> RecoverRestorePins
      [] step.action = "RecoverResumeFreeSpaceRoot"   -> RecoverResumeFreeSpaceRoot
      [] step.action = "RecoverResumeReclaimComplete" -> RecoverResumeReclaimComplete
      [] step.action = "RecoverServing"           -> RecoverServing
      [] OTHER                                    -> FALSE

MatchesPost(step) ==
    \* M3 durability barriers
    /\ durableHead' = step.postDurableHead
    /\ checkpointEpoch' = step.postCheckpointEpoch
    /\ stagedCheckpoint' = step.postStagedCheckpoint
    /\ retainedFloor' = step.postRetainedFloor
    /\ pendingWrite' = step.postPendingWrite
    /\ crashed' = step.postCrashed
    /\ recoveryStage' = step.postRecoveryStage
    /\ recoveredHead' = step.postRecoveredHead
    \* branches & Home frontier
    /\ branchState' = step.postBranchState
    /\ head' = step.postHead
    /\ prep' = step.postPrep
    /\ frontierSeq' = step.postFrontierSeq
    /\ pendingBranch' = step.postPendingBranch
    \* PinTable
    /\ pinState' = step.postPinState
    /\ pinnedRoots' = step.postPinnedRoots
    /\ activeReaderLoans' = step.postReaderLoans
    /\ liveChildPins' = step.postChildPins
    /\ pinsRestored' = step.postPinsRestored
    \* extents, generations, writebacks
    /\ phase' = step.postPhase
    /\ cov' = step.postCov
    /\ cellState' = step.postCellState
    /\ cellGen' = step.postCellGen
    /\ rootPubGen' = step.postRootPubGen
    /\ fenceSeq' = step.postFenceSeq
    /\ inflight' = step.postInflight
    \* cursor & recovery
    /\ tokenBasis' = step.postTokenBasis
    /\ cursorLoan' = step.postCursorLoan
    /\ m4Rec' = step.postM4Rec

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
