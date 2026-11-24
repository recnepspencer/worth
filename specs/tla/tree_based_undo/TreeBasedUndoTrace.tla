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
             | "Begin" | "Commit" | "Discard" | "Maintain" | "Close"
    outcome  what the entry returned (see StepAction)
    Envelope: kind ("edit" | "correction"), stages ([e1 |-> op, e2 |-> op],
             op is a value, 0 to retire, -1 for no stage), src and ent (the
             correction's source step and entity; 0 and "none" for an edit),
             key, and for outcome "AlreadyRecorded" the returned step, for
             outcome "Stopped" the point when the harness can tell it.
    Redo, Checkout: node.   OutOfBand: stages (as an edit's).
    Begin: class ("edit" | "gesture").   Commit: stages, key (a writer's
             key, in the "writer" namespace; an Envelope's key is in
             "envelope").
    Post-state: postCurrent, postNextNode, postLive, postGen, postVal,
             postOutA, postOutB, postClosed, and optionally
             postHeadDiverged (left out while a writer holds the document,
             when no read can tell), postParked (the number of parked
             candidates) and postReversals (the active reversal facts on the
             document branch, a set of [step, src, ent], each attributed to
             its step through the fact IDs the journal records with it).

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
            /\ step.step = idem[Env(step.key)].node
      [] step.outcome = "IntentDrift" ->
            RefuseIntentDrift(step.key, IntentOf(step))
      [] step.outcome = "StageRefused" ->
            IF step.kind = "edit"
            THEN RefuseStages(step.stages, step.key)
            ELSE RefuseCorrectionStages(step.src, step.ent, step.stages, step.key)
      [] step.outcome = "SourceRefused" ->
            /\ step.kind = "correction"
            /\ RefuseCorrectionSource(step.src, step.ent, step.key)
      [] step.outcome = "HeadDiverged" -> RefuseHeadDiverged
      [] step.outcome = "Empty" -> RefuseEmpty
      [] step.outcome = "Stopped" ->
            IF "point" \in DOMAIN step
            THEN WriterStopped(step.point)
            ELSE \E p \in StopPoints : WriterStopped(p)
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
      [] step.action = "OutOfBand" -> OutOfBandMutation(step.stages)
      [] step.action = "Begin" -> WriterBegin(step.class)
      [] step.action = "Commit" -> WriterCommit(step.stages, step.key)
      [] step.action = "Maintain" ->
            IF step.outcome = "NoPendingRelease"
            THEN MaintainIdle
            ELSE \/ \E c \in Candidates : MaintainHistory(c)
                 \/ MaintainHead
      [] step.action = "Discard" -> WriterDiscard
      [] step.action = "Close" -> Close

MatchesPost(step) ==
    /\ current' = step.postCurrent
    /\ nextNode' = step.postNextNode
    \* Close retires the document branch: only a closed document may leave
    \* its rows and outputs out.
    /\ ("postLive" \in DOMAIN step => /\ live' = step.postLive
                                       /\ gen' = step.postGen
                                       /\ val' = step.postVal
                                       /\ outA' = step.postOutA
                                       /\ outB' = step.postOutB)
    /\ ~closed' => "postLive" \in DOMAIN step
    /\ ("postHeadDiverged" \in DOMAIN step => headDiverged' = step.postHeadDiverged)
    \* Only a held document may leave it out (review r2, note 4).
    /\ (writer' = "none" /\ ~closed') => "postHeadDiverged" \in DOMAIN step
    /\ closed' = step.postClosed
    /\ ("postParked" \in DOMAIN step => Cardinality(parked') = step.postParked)
    /\ ("postReversals" \in DOMAIN step => facts' = step.postReversals)

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
