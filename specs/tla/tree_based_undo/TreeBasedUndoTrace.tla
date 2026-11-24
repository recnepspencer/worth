------------------------- MODULE TreeBasedUndoTrace -------------------------
EXTENDS TreeBasedUndo, TraceData

(*
  Trace validation for TreeBasedUndo.tla.

  The Rust DST harness writes one NDJSON record per step. `run_tlc.py trace`
  turns the file into TraceData.tla (`Trace == << ... >>`) and runs TLC on
  this module. TLC passes only if every observed step is a transition the
  model allows from the state before it, with the outcome the harness saw,
  and reaches the post-state the harness observed.

  Record fields.
    action   "Envelope" | "Undo" | "Redo" | "Checkout" | "OutOfBand"
             | "Maintain" | "Discard" | "Close"
    outcome  what the entry returned (see StepAction)
    Envelope: kind ("edit" | "correction"), stages ([e1 |-> op, e2 |-> op],
             op is a value, 0 to retire, -1 for no stage), src and ent (the
             correction's source step and entity; 0 and "none" for an edit),
             key, and for outcome "AlreadyRecorded" the returned step, for
             outcome "Stopped" the point.
    Redo, Checkout: node.   OutOfBand: retireSet.
    Post-state: postCurrent, postNextNode, postLive, postGen, postVal,
             postOutA, postOutB, postHeadDiverged, postClosed, and
             optionally postParked (the number of parked candidates).

  Generations are the dense first-seen numbering of the real
  full-generation IDs, so equal numbers mean the same real ID.
*)

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

IntentOf(step) ==
    IF step.kind = "edit"
    THEN EditIntent(step.stages)
    ELSE CorrectionIntent(step.src, step.ent, step.stages)

EnvelopeStep(step) ==
    CASE step.outcome = "Recorded" ->
            IF step.kind = "edit"
            THEN EditEnvelope(step.stages, step.key)
            ELSE CorrectionEnvelope(step.src, step.ent, step.stages, step.key)
      [] step.outcome = "AlreadyRecorded" ->
            /\ AlreadyRecorded(step.key, IntentOf(step))
            /\ step.step = idem[step.key].node
      [] step.outcome = "IntentDrift" ->
            RefuseIntentDrift(step.key, IntentOf(step))
      [] step.outcome = "StageRefused" ->
            IF step.kind = "edit"
            THEN RefuseStages(step.stages, step.key)
            ELSE RefuseCorrectionStages(step.src, step.ent, step.stages, step.key)
      [] step.outcome = "SourceRefused" ->
            /\ step.kind = "correction"
            /\ RefuseCorrectionSource(step.src, step.ent)
      [] step.outcome = "HeadDiverged" -> RefuseHeadDiverged
      [] step.outcome = "Empty" -> RefuseEmpty
      [] step.outcome = "Stopped" -> WriterStopped(step.point)
      [] OTHER -> RefuseEnvironment(step.outcome)

NavigationStep(step) ==
    CASE step.outcome = "HeadDiverged" -> RefuseHeadDiverged
      [] step.action = "Undo" /\ step.outcome = "Moved" -> UndoStep
      [] step.action = "Undo" /\ step.outcome = "Refused" -> RefuseUndoAtRoot
      [] step.action = "Redo" /\ step.outcome = "Moved" -> RedoTo(step.node)
      [] step.action = "Redo" /\ step.outcome = "Refused" -> RefuseRedo(step.node)
      [] step.action = "Checkout" /\ step.outcome = "Moved" -> Checkout(step.node)
      [] step.action = "Checkout" /\ step.outcome = "Refused" ->
            RefuseCheckout(step.node)

StepAction(step) ==
    CASE step.action = "Envelope" -> EnvelopeStep(step)
      [] step.action \in {"Undo", "Redo", "Checkout"} -> NavigationStep(step)
      [] step.action = "OutOfBand" -> OutOfBandMutation(step.retireSet)
      [] step.action = "Maintain" ->
            IF step.outcome = "NoPendingRelease"
            THEN MaintainIdle
            ELSE \E c \in Candidates : MaintainHistory(c)
      [] step.action = "Discard" -> WriterDiscarded
      [] step.action = "Close" -> Close

MatchesPost(step) ==
    /\ current' = step.postCurrent
    /\ nextNode' = step.postNextNode
    /\ live' = step.postLive
    /\ gen' = step.postGen
    /\ val' = step.postVal
    /\ outA' = step.postOutA
    /\ outB' = step.postOutB
    /\ headDiverged' = step.postHeadDiverged
    /\ closed' = step.postClosed
    /\ ("postParked" \in DOMAIN step => Cardinality(parked') = step.postParked)

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
