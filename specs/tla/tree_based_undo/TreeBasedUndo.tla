---------------------------- MODULE TreeBasedUndo ----------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Tree-Based Undo and Redo for a WORTH history document: Phase 1 (recorded
  edit envelopes, navigation; public #53, #54, #55, #57) and Phase 2 (recorded
  authored corrections and navigation across retirements; public #56).

  The model is reconciled with Phase 1 as built at 41755bbbc2:

  - One envelope is one history step. Its stages commit together on a private
    candidate and the document head moves through one adoption, or nothing
    changes at all.
  - A replay of a recorded key with the same intent returns AlreadyRecorded
    with the original step. The same key with another intent refuses
    IntentDrift. Metadata is not part of the intent.
  - A stopped request records nothing. Its candidate is parked for the
    document's own disposal and its journal record is released, so the key is
    free. Output trees it was recording stay bound to the parked candidate
    until the next writer, a discard, maintenance or close releases them.
  - Every typed refusal changes nothing.
  - HeadDiverged is terminal for the document. Only close succeeds afterwards.

  Phase 2 adds one transition: a correction envelope. Its first stage is the
  inverse of one recorded source stage on the document's selected line
  (retire what the source created, or restore what the source retired). It
  records through the same step recording as an edit envelope.

  An entity name is a stable authored key. A generation is one occurrence of
  that key: the full-generation ID. A recorded restore creates a fresh
  occurrence. Undo, redo and checkout select a recorded snapshot, so they
  bring back the original occurrence.

  A recorded correction also posts a reversal fact on the branch: "step c
  reversed source s on entity e" (the aftermath-causality fact). The
  candidate pass refuses SourceAlreadyReversed on an active fact. Navigation
  retires the facts of the steps it leaves and restores the facts of the
  steps it enters, so a fact is active exactly on the lines where its step is
  an ancestor of the selected step (spec-owner ruling (u)). FactBug restores
  the defect found in plan review r1: navigation leaves the facts alone.
*)

CONSTANTS
    Entities,       \* {"e1", "e2"}; "e2" is owned under "e1"
    Values,         \* e.g. {1, 2}
    Keys,           \* idempotency keys
    MaxNodes,       \* root = 0, up to MaxNodes - 1 recorded steps
    MaxStops,       \* stopped requests explored per behaviour
    MaxHeads,       \* queued past heads the document's release queue holds
    WedgeBug,       \* TRUE re-introduces Phase 1's stale-forest wedge
    FactBug         \* TRUE re-introduces r1's facts that ignore navigation

ASSUME Entities = {"e1", "e2"}
ASSUME MaxNodes >= 2 /\ MaxStops >= 0 /\ MaxHeads >= 1
ASSUME WedgeBug \in BOOLEAN
ASSUME FactBug \in BOOLEAN
ASSUME Values \subseteq (Nat \ {0})

NodeIds == 0..(MaxNodes - 1)
NullNode == -1
ValOrNone == Values \cup {0}
Candidates == 1..MaxStops

\* Stage operations of an envelope, one per entity.
Skip == -1          \* the envelope has no stage on this entity
Retire == 0         \* the stage retires the entity
StageOps == Values \cup {Skip, Retire}
NoStages == [e \in Entities |-> Skip]
StageMaps == [Entities -> StageOps]

\* What a recorded step did to each entity.
StepOps == {"none", "born", "set", "retired", "restored"}
NoOps == [e \in Entities |-> "none"]

\* Two generated output trees, both functions of the live authored state.
OutputsA(liveSet, valFn) ==
    [e \in Entities |-> IF e \in liveSet THEN valFn[e] ELSE 0]
OutputsB(liveSet, valFn) ==
    [e \in Entities |-> IF e \in liveSet THEN valFn[e] + 10 ELSE 0]

\* Ownership law: "e2" is live only while its owner "e1" is live.
DepsSatisfied(liveSet) == ("e2" \in liveSet) => ("e1" \in liveSet)

\* Fresh generations are handed out in a fixed entity order.
Rank(e) == IF e = "e1" THEN 0 ELSE 1

MakeSnapshot(liveSet, genFn, valFn) ==
    [live |-> liveSet,
     gen |-> genFn,
     val |-> valFn,
     outA |-> OutputsA(liveSet, valFn),
     outB |-> OutputsB(liveSet, valFn)]

EmptySnapshot ==
    MakeSnapshot({}, [e \in Entities |-> 0], [e \in Entities |-> 0])

VARIABLES
    nextNode,        \* next history step to allocate
    nextGen,         \* next fresh generation
    parent,          \* [NodeIds -> NodeIds \cup {NullNode}]
    snapshots,       \* [NodeIds -> snapshot]: retained state of each step
    ops,             \* [NodeIds -> [Entities -> StepOps]]
    current,         \* the document's selected step
    live,            \* live owned entities on the document branch
    gen,             \* [Entities -> Nat]: live occurrence, 0 when not live
    val,             \* [Entities -> ValOrNone]
    outA, outB,      \* generated output trees on the branch
    facts,           \* active reversal facts on the branch:
                     \*   [step |-> c, src |-> s, ent |-> e]
    idem,            \* [JournalKeys -> journal record]
    headDiverged,    \* the branch moved outside the document
    closed,          \* the document was closed
    nextCand,        \* next stopped candidate to allocate
    parked,          \* stopped candidates awaiting the document's disposal
    forestOrigin,    \* candidate the recording trees are bound to, 0 if none
    forestDisposing, \* a release of those trees was itself stopped
    writer,          \* an open .edit / .gesture holding the move-only document
    forked,          \* the document stands on a fork a navigation made, not
                     \* on its current step's own branch
    heads            \* past heads queued for maintain_history to release

treeVars == <<nextNode, parent, snapshots, ops, current>>
branchVars == <<live, gen, val, outA, outB, facts>>
stopVars == <<nextCand, parked, forestOrigin, forestDisposing>>
headVars == <<forked, heads>>
vars == <<treeVars, nextGen, branchVars, idem, headDiverged, closed, stopVars,
          writer, headVars>>

WriterClasses == {"edit", "gesture"}

\* Idempotency keys are scoped by who records them (refinement (v), ruled
\* option (2), 2026-11-24). An envelope's key is hashed under its own tag
\* (envelope_recording/admission.rs); a gesture writer is keyed by its group
\* id and an .edit by its operation's mutation key. The two never meet: an
\* envelope under a writer's key records a new step (Rust DST seeds
\* 0x7b020035, gesture, and 0x7b020108, .edit). Replay, drift and the
\* envelope refusals read only the "envelope" namespace.
Namespaces == {"envelope", "writer"}
JournalKeys == Namespaces \X Keys
Env(k) == <<"envelope", k>>
Wr(k) == <<"writer", k>>

NoIntent == [kind |-> "none", st |-> NoStages, src |-> 0, ent |-> "none"]
EditIntent(st) == [kind |-> "edit", st |-> st, src |-> 0, ent |-> "none"]
CorrectionIntent(src, e, st) ==
    [kind |-> "correction", st |-> st, src |-> src, ent |-> e]
\* A journal record is "unseen" or "recorded"; there is no in-flight status.
\* The document handle is move-only, so no second envelope can run while one
\* is in flight, and AttemptInFlight cannot be observed (review note 7).
Unseen == [status |-> "unseen", intent |-> NoIntent, node |-> NullNode]

Init ==
    /\ nextNode = 1
    /\ nextGen = 1
    /\ parent = [n \in NodeIds |-> NullNode]
    /\ snapshots = [n \in NodeIds |-> EmptySnapshot]
    /\ ops = [n \in NodeIds |-> NoOps]
    /\ current = 0
    /\ live = {}
    /\ gen = [e \in Entities |-> 0]
    /\ val = [e \in Entities |-> 0]
    /\ outA = EmptySnapshot.outA
    /\ outB = EmptySnapshot.outB
    /\ facts = {}
    /\ idem = [j \in JournalKeys |-> Unseen]
    /\ headDiverged = FALSE
    /\ closed = FALSE
    /\ nextCand = 1
    /\ parked = {}
    /\ forestOrigin = 0
    /\ forestDisposing = FALSE
    /\ writer = "none"
    /\ forked = FALSE
    /\ heads = 0

\* The head a recording or a move leaves behind (Rust DST, S1 probe): a fork
\* a navigation made is queued for maintain_history when the document leaves
\* it; the step's own branch is not.
LeaveHead == heads' = heads + (IF forked THEN 1 ELSE 0)

ApplySnapshot(s) ==
    /\ live' = s.live
    /\ gen' = s.gen
    /\ val' = s.val
    /\ outA' = s.outA
    /\ outB' = s.outB

\* The caller holds the document: it is open and no writer holds it.
Held == ~closed /\ writer = "none"

\* The document accepts work: it is held and its head has not diverged.
Writable == Held /\ ~headDiverged

RECURSIVE Line(_)
\* The selected line of a step: the step and all its ancestors.
Line(n) == IF n = 0 THEN {0} ELSE {n} \cup Line(parent[n])

\* Steps of the selected line from `src` to the current step.
Since(src) == {n \in Line(current) : src \in Line(n)}

\* A reversal fact: step `c` reversed source `s` on entity `e`.
Fact(c, s, e) == [step |-> c, src |-> s, ent |-> e]

\* Every fact a recorded correction step posted, active or not.
PostedFacts ==
    {Fact(idem[j].node, idem[j].intent.src, idem[j].intent.ent) :
        j \in {x \in JournalKeys : idem[x].status = "recorded"
                            /\ idem[x].intent.kind = "correction"}}

\* reverse_lineage: an active fact says the source is already reversed.
ReversedByFact(src, e) == \E f \in facts : f.src = src /\ f.ent = e

\* ---------------------------------------------------------------------------
\* Stage semantics shared by edit and correction envelopes
\* ---------------------------------------------------------------------------
Touched(st) == {e \in Entities : st[e] /= Skip}
Sets(st) == {e \in Entities : st[e] \in Values}
Retires(st) == {e \in Entities : st[e] = Retire}
NewLive(st) == (live \cup Sets(st)) \ Retires(st)
Births(st) == Sets(st) \ live

\* Every stage is legal and the envelope leaves the owned graph closed.
\* Retiring "e1" while "e2" stays live is a refused partial retirement.
StagesAdmissible(st) ==
    /\ Touched(st) /= {}
    /\ Retires(st) \subseteq live
    /\ DepsSatisfied(NewLive(st))

BirthIndex(e, born) == Cardinality({x \in born : Rank(x) < Rank(e)})

StepGen(st) ==
    [e \in Entities |->
        IF e \in Births(st) THEN nextGen + BirthIndex(e, Births(st))
        ELSE IF e \in Retires(st) THEN 0
        ELSE gen[e]]

StepVal(st) ==
    [e \in Entities |->
        IF e \in Sets(st) THEN st[e]
        ELSE IF e \in Retires(st) THEN 0
        ELSE val[e]]

\* `restored` names the entities a corrective stage brings back.
StepOpsOf(st, restored) ==
    [e \in Entities |->
        IF e \in restored THEN "restored"
        ELSE IF e \in Births(st) THEN "born"
        ELSE IF e \in Sets(st) THEN "set"
        ELSE IF e \in Retires(st) THEN "retired"
        ELSE "none"]

\* The one recording owner. Every recorded step, edit or correction, goes
\* through it: one new node, one adoption, one journal record. A forest that a
\* stopped request left bound is released by this writer.
RecordStep(st, restored, j, intent) ==
    LET snap == MakeSnapshot(NewLive(st), StepGen(st), StepVal(st)) IN
    /\ nextNode' = nextNode + 1
    /\ nextGen' = nextGen + Cardinality(Births(st))
    /\ parent' = [parent EXCEPT ![nextNode] = current]
    /\ snapshots' = [snapshots EXCEPT ![nextNode] = snap]
    /\ ops' = [ops EXCEPT ![nextNode] = StepOpsOf(st, restored)]
    /\ current' = nextNode
    /\ ApplySnapshot(snap)
    /\ facts' = IF intent.kind = "correction"
                THEN facts \cup {Fact(nextNode, intent.src, intent.ent)}
                ELSE facts
    /\ idem' = [idem EXCEPT ![j] =
                    [status |-> "recorded", intent |-> intent, node |-> nextNode]]
    /\ forestOrigin' = 0
    /\ forestDisposing' = FALSE
    /\ writer' = "none"
    /\ LeaveHead
    /\ forked' = FALSE
    /\ UNCHANGED <<headDiverged, closed, nextCand, parked>>

\* ---------------------------------------------------------------------------
\* Phase 1: edit envelope (document.edit_envelope ... execute_recorded)
\* ---------------------------------------------------------------------------
EditEnvelope(st, k) ==
    /\ Writable
    /\ st \in StageMaps /\ k \in Keys
    /\ idem[Env(k)].status = "unseen"
    /\ nextNode < MaxNodes
    /\ StagesAdmissible(st)
    /\ RecordStep(st, {}, Env(k), EditIntent(st))

\* ---------------------------------------------------------------------------
\* Phase 2: correction envelope (document.correction_envelope ...)
\* The source is the stage of recorded step `src` on entity `e`. `st` holds
\* the caller's coupled stages on the other entities.
\* ---------------------------------------------------------------------------

\* The source created `e` and that occurrence is still the live one.
ReversesCreation(src, e) ==
    /\ ops[src][e] = "born"
    /\ e \in live
    /\ gen[e] = snapshots[src].gen[e]

\* The source retired `e` and it has stayed retired on the selected line.
ReversesRetirement(src, e) ==
    /\ ops[src][e] = "retired"
    /\ \A n \in Since(src) : e \notin snapshots[n].live

\* The head pass (plan r2, 2.3): the source is a step this document recorded
\* on its selected line and its stage on `e` has a corrective binding. Only
\* then can the document derive the correction's meaning and consult its
\* journal. It reads nothing else of the branch.
MeaningDerivable(src, e) ==
    /\ src \in Line(current) \ {0}
    /\ ops[src][e] \in {"born", "retired"}

\* The candidate pass, after the journal found the key unseen.
SourceReversible(src, e) ==
    /\ MeaningDerivable(src, e)
    /\ ReversesCreation(src, e) \/ ReversesRetirement(src, e)

\* The corrective stage: retire what was created, or restore the value the
\* source's retirement removed.
CorrectiveOp(src, e) ==
    IF ops[src][e] = "born" THEN Retire ELSE snapshots[parent[src]].val[e]

\* Reversing the creation of "e1" also retires the owned child the same
\* source created, while that occurrence is still live (co_retire_created,
\* the owned-subgraph cascade). A child born in a later step is not part of
\* the source and stays, so the correction is a refused partial retirement
\* unless the caller retires it in a coupled stage.
CoRetired(src, e) ==
    /\ e = "e1"
    /\ ops[src]["e1"] = "born"
    /\ ops[src]["e2"] = "born"
    /\ "e2" \in live
    /\ gen["e2"] = snapshots[src].gen["e2"]

WithCorrective(src, e, st) ==
    [x \in Entities |->
        IF x = e THEN CorrectiveOp(src, e)
        ELSE IF x = "e2" /\ CoRetired(src, e) /\ st[x] = Skip THEN Retire
        ELSE st[x]]

RestoredBy(src, e) == IF ops[src][e] = "retired" THEN {e} ELSE {}

CorrectionEnvelope(src, e, st, k) ==
    /\ Writable
    /\ src \in 1..(nextNode - 1) /\ e \in Entities
    /\ st \in StageMaps /\ st[e] = Skip /\ k \in Keys
    /\ idem[Env(k)].status = "unseen"
    /\ nextNode < MaxNodes
    /\ SourceReversible(src, e)
    /\ ~ReversedByFact(src, e)
    /\ StagesAdmissible(WithCorrective(src, e, st))
    /\ RecordStep(WithCorrective(src, e, st), RestoredBy(src, e), Env(k),
                  CorrectionIntent(src, e, st))

\* ---------------------------------------------------------------------------
\* Replay and typed refusals. None of them changes any state.
\* ---------------------------------------------------------------------------

\* A correction's meaning is derived from its source before the journal is
\* consulted, so its replay needs the source on the selected line. Whether
\* the source is still reversible is not consulted (it is not, once the
\* correction is recorded on the line).
ReplayDisclosed(k) ==
    LET i == idem[Env(k)].intent IN
    i.kind = "edit" \/ MeaningDerivable(i.src, i.ent)

\* AlreadyRecorded { document, step }: the step is idem[Env(k)].node.
AlreadyRecorded(k, intent) ==
    /\ Writable
    /\ idem[Env(k)].status = "recorded"
    /\ idem[Env(k)].intent = intent
    /\ ReplayDisclosed(k)
    /\ UNCHANGED vars

\* The key is recorded with another intent.
RefuseIntentDrift(k, intent) ==
    /\ Writable
    /\ idem[Env(k)].status = "recorded"
    /\ idem[Env(k)].intent /= intent
    /\ (intent.kind = "correction" => MeaningDerivable(intent.src, intent.ent))
    /\ UNCHANGED vars

\* A stage refuses in its domain, or the envelope would leave a dependent
\* child live (Stage { ordinal, denial }).
RefuseStages(st, k) ==
    /\ Writable
    /\ idem[Env(k)].status = "unseen"
    /\ ~StagesAdmissible(st)
    /\ UNCHANGED vars

\* The head pass refuses a source off the selected line or a stage with no
\* corrective binding, whatever the key. With an unseen key, the candidate
\* pass refuses a source that is no longer current, or one an active reversal
\* fact says is already reversed (SourceAlreadyReversed).
RefuseCorrectionSource(src, e, k) ==
    /\ Writable
    /\ k \in Keys
    /\ \/ ~MeaningDerivable(src, e)
       \/ /\ idem[Env(k)].status = "unseen"
          /\ \/ ~SourceReversible(src, e)
             \/ ReversedByFact(src, e)
    /\ UNCHANGED vars

\* A correction whose source is reversible but whose stages are refused: the
\* refused partial retirement, or a restore whose owner is not live.
RefuseCorrectionStages(src, e, st, k) ==
    /\ Writable
    /\ idem[Env(k)].status = "unseen"
    /\ SourceReversible(src, e)
    /\ ~ReversedByFact(src, e)
    /\ ~StagesAdmissible(WithCorrective(src, e, st))
    /\ UNCHANGED vars

\* Refusals whose cause lies outside the modelled state.
EnvironmentRefusals ==
    {"MetadataTooLarge", "ForeignModel", "UnownedOperationScope",
     "OperationNotDeclared", "UnsupportedOperation", "WorkflowProducer",
     "ExternalEffectProducer", "OutputsIncomplete", "Budget",
     "StaleSelection", "ForeignRuntime", "RequestTargetMismatch"}

RefuseEnvironment(reason) ==
    /\ Writable
    /\ reason \in EnvironmentRefusals
    /\ UNCHANGED vars

\* An envelope with no stage refuses before the head is read.
RefuseEmpty == Held /\ UNCHANGED vars

\* Every document entry on a diverged head.
RefuseHeadDiverged == Held /\ headDiverged /\ UNCHANGED vars

RefuseUndoAtRoot == Writable /\ current = 0 /\ UNCHANGED vars
RefuseRedo(child) ==
    /\ Writable
    /\ ~(child \in 1..(nextNode - 1) /\ parent[child] = current)
    /\ UNCHANGED vars
RefuseCheckout(target) ==
    /\ Writable
    /\ target \notin 0..(nextNode - 1)
    /\ UNCHANGED vars

\* ---------------------------------------------------------------------------
\* Navigation: select a recorded snapshot. Crossing a recorded retirement
\* brings the retired entities back at their original generations.
\* ---------------------------------------------------------------------------
\* The move retires the facts of the steps it leaves and restores the facts
\* of the steps it enters, at their original IDs.
Select(target) ==
    LET leaving == Line(current) \ Line(target)
        entering == Line(target) \ Line(current)
    IN
    /\ current' = target
    \* A move forks a new branch. It keeps room in the release queue for the
    \* head it leaves and the one the next recording leaves.
    /\ heads + (IF forked THEN 1 ELSE 0) < MaxHeads
    /\ LeaveHead
    /\ forked' = TRUE
    /\ ApplySnapshot(snapshots[target])
    /\ facts' = IF FactBug THEN facts
                ELSE {f \in facts : f.step \notin leaving}
                        \cup {f \in PostedFacts : f.step \in entering}
    /\ UNCHANGED <<nextNode, nextGen, parent, snapshots, ops, idem,
                   headDiverged, closed, stopVars, writer>>

UndoStep == Writable /\ current /= 0 /\ Select(parent[current])

RedoTo(child) ==
    /\ Writable
    /\ child \in 1..(nextNode - 1)
    /\ parent[child] = current
    /\ Select(child)

Checkout(target) ==
    /\ Writable
    /\ target \in 0..(nextNode - 1)
    /\ target /= current
    /\ Select(target)

\* ---------------------------------------------------------------------------
\* A write or correction on the document branch outside the document
\* ---------------------------------------------------------------------------
\* Any admissible change, through request.mutation(..) or
\* request.corrections(..): create, update, retire, or restore (a fresh
\* occurrence, as a recorded restore makes). It needs no document, so it can
\* happen while a writer holds the document. The write and the settle of
\* its program outputs are one step: the model has no output wave or marks,
\* so that the settle ends is outside it (README; public #353).
OutOfBandMutation(st) ==
    /\ ~closed /\ ~headDiverged
    /\ st \in StageMaps
    /\ StagesAdmissible(st)
    /\ headDiverged' = TRUE
    /\ nextGen' = nextGen + Cardinality(Births(st))
    /\ ApplySnapshot(MakeSnapshot(NewLive(st), StepGen(st), StepVal(st)))
    /\ UNCHANGED <<treeVars, idem, closed, stopVars, writer, facts, headVars>>

\* ---------------------------------------------------------------------------
\* Stopped requests, parked candidates and the recording trees
\* ---------------------------------------------------------------------------

\* An envelope whose request stops before adoption. Nothing is recorded and
\* the key stays free. `point` is how far it got:
\*   "Admission" stopped before its candidate forked: nothing is parked and
\*               nothing changes (Rust DST seed 0x7b020000: a stop at the
\*               history read's admission). It lands before the head is
\*               read, so it needs only a held document, diverged or not
\*               (same seed, k7: a stop in the history read on a diverged
\*               head);
\*   "Stages"    stopped while its stages ran;
\*   "Recording" stopped while its output trees recorded, after releasing any
\*               trees an earlier stop left bound;
\*   "Release"   stopped while releasing trees an earlier stop left bound.
StopPoints == {"Admission", "Stages", "Recording", "Release"}

WriterStopped(point) ==
    /\ \E k \in Keys : idem[Env(k)].status = "unseen"
    /\ \/ /\ point = "Admission"
          /\ Held
          /\ UNCHANGED <<parked, nextCand, forestOrigin, forestDisposing>>
       \/ /\ point \in {"Stages", "Recording", "Release"}
          /\ Writable
          /\ nextCand <= MaxStops
          /\ parked' = parked \cup {nextCand}
          /\ nextCand' = nextCand + 1
          /\ \/ /\ point = "Stages"
                /\ UNCHANGED <<forestOrigin, forestDisposing>>
             \/ /\ point = "Recording"
                /\ forestOrigin' = nextCand
                /\ forestDisposing' = FALSE
             \/ /\ point = "Release"
                /\ forestOrigin /= 0
                /\ forestDisposing' = TRUE
                /\ UNCHANGED forestOrigin
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, closed,
                   writer, headVars>>

\* ---------------------------------------------------------------------------
\* Writers that hold the move-only document across advances: .edit and
\* .gesture. (An envelope runs in one call; its stops are WriterStopped.)
\* Phase 1 as fixed: the one recording owner releases trees an earlier stop
\* left bound when any writer begins, and discard / abort close the trees
\* against the candidate they are bound to. WedgeBug restores Phase 1's
\* defect: begin leaves them bound, and discard closes against the writer's
\* own candidate, which refuses ForeignDemand and keeps the document.
\* ---------------------------------------------------------------------------
WriterBegin(class) ==
    /\ Writable
    /\ class \in WriterClasses
    /\ writer' = class
    /\ IF WedgeBug
       THEN UNCHANGED <<forestOrigin, forestDisposing>>
       ELSE forestOrigin' = 0 /\ forestDisposing' = FALSE
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, closed,
                   nextCand, parked, headVars>>

\* The writer records its step. Trees bound to another candidate refuse
\* (ForeignDocument / NeedsDisposal) and the writer can only be discarded.
WriterCommit(st, k) ==
    /\ writer \in WriterClasses
    /\ ~closed /\ ~headDiverged
    /\ st \in StageMaps /\ k \in Keys
    /\ idem[Wr(k)].status = "unseen"
    /\ nextNode < MaxNodes
    /\ StagesAdmissible(st)
    /\ forestOrigin = 0 /\ ~forestDisposing
    /\ RecordStep(st, {}, Wr(k), EditIntent(st))

\* discard / abort return the document, releasing any trees.
WriterDiscard ==
    /\ writer \in WriterClasses
    /\ ~closed
    /\ WedgeBug => (forestOrigin = 0 /\ ~forestDisposing)
    /\ writer' = "none"
    /\ forestOrigin' = 0
    /\ forestDisposing' = FALSE
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, closed,
                   nextCand, parked, headVars>>

\* maintain_history: one parked candidate per call, trees first.
MaintainHistory(c) ==
    /\ Held
    /\ c \in parked
    /\ parked' = parked \ {c}
    /\ IF forestOrigin = c
       THEN forestOrigin' = 0 /\ forestDisposing' = FALSE
       ELSE UNCHANGED <<forestOrigin, forestDisposing>>
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, closed,
                   nextCand, writer, headVars>>

\* maintain_history: one queued past head per call.
MaintainHead ==
    /\ Held
    /\ heads > 0
    /\ heads' = heads - 1
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, closed,
                   stopVars, writer, forked>>

\* maintain_history with nothing parked or queued reports NoPendingRelease.
MaintainIdle == Held /\ parked = {} /\ heads = 0 /\ UNCHANGED vars

\* close: drains every parked candidate and the trees. It also succeeds on a
\* diverged head, and is the only document entry that does.
Close ==
    /\ Held
    /\ closed' = TRUE
    /\ parked' = {}
    /\ forestOrigin' = 0
    /\ forestDisposing' = FALSE
    /\ heads' = 0
    /\ UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged, nextCand,
                   writer, forked>>

ClosedStutter == closed /\ UNCHANGED vars

Intents ==
    {EditIntent(st) : st \in StageMaps}
        \cup {CorrectionIntent(n, e, st) :
                n \in NodeIds, e \in Entities, st \in StageMaps}

Next ==
    \/ \E st \in StageMaps, k \in Keys : EditEnvelope(st, k)
    \/ \E n \in NodeIds, e \in Entities, st \in StageMaps, k \in Keys :
            CorrectionEnvelope(n, e, st, k)
    \/ \E k \in Keys, i \in Intents : AlreadyRecorded(k, i)
    \/ \E k \in Keys, i \in Intents : RefuseIntentDrift(k, i)
    \/ \E st \in StageMaps, k \in Keys : RefuseStages(st, k)
    \/ \E n \in NodeIds, e \in Entities, k \in Keys :
            RefuseCorrectionSource(n, e, k)
    \/ \E n \in NodeIds, e \in Entities, st \in StageMaps, k \in Keys :
            RefuseCorrectionStages(n, e, st, k)
    \/ \E r \in EnvironmentRefusals : RefuseEnvironment(r)
    \/ RefuseEmpty
    \/ RefuseHeadDiverged
    \/ RefuseUndoAtRoot
    \/ \E c \in NodeIds : RefuseRedo(c)
    \/ \E n \in NodeIds \cup {MaxNodes} : RefuseCheckout(n)
    \/ UndoStep
    \/ \E c \in NodeIds : RedoTo(c)
    \/ \E n \in NodeIds : Checkout(n)
    \/ \E st \in StageMaps : OutOfBandMutation(st)
    \/ \E p \in StopPoints : WriterStopped(p)
    \/ \E c \in WriterClasses : WriterBegin(c)
    \/ \E st \in StageMaps, k \in Keys : WriterCommit(st, k)
    \/ WriterDiscard
    \/ \E c \in Candidates : MaintainHistory(c)
    \/ MaintainHead
    \/ MaintainIdle
    \/ Close
    \/ ClosedStutter

Spec == Init /\ [][Next]_vars

\* ===========================================================================
\* Safety invariants
\* ===========================================================================

Recorded == 1..(nextNode - 1)

TypeOK ==
    /\ nextNode \in 1..MaxNodes
    /\ nextGen \in 1..(1 + Cardinality(Entities) * (MaxNodes + 1))
    /\ current \in 0..(nextNode - 1)
    /\ live \subseteq Entities
    /\ gen \in [Entities -> 0..(nextGen - 1)]
    /\ val \in [Entities -> ValOrNone]
    /\ facts \subseteq [step : NodeIds, src : NodeIds, ent : Entities]
    /\ \A n \in NodeIds : ops[n] \in [Entities -> StepOps]
    /\ DOMAIN idem = JournalKeys
    /\ \A j \in JournalKeys : idem[j].status \in {"unseen", "recorded"}
    /\ headDiverged \in BOOLEAN /\ closed \in BOOLEAN
    /\ nextCand \in 1..(MaxStops + 1)
    /\ parked \subseteq Candidates
    /\ forestOrigin \in 0..MaxStops
    /\ forestDisposing \in BOOLEAN
    /\ writer \in {"none"} \cup WriterClasses
    /\ forked \in BOOLEAN
    /\ heads \in 0..MaxHeads

\* 1. While the document owns the head, the branch is exactly the selected
\*    step's snapshot: entities, generations, values and both output trees.
SnapshotExactness ==
    ~headDiverged =>
        /\ live = snapshots[current].live
        /\ gen = snapshots[current].gen
        /\ val = snapshots[current].val
        /\ outA = snapshots[current].outA
        /\ outB = snapshots[current].outB

\* 2. The owned graph is closed and the output forest follows it.
OwnershipAndOutputsConsistent ==
    /\ DepsSatisfied(live)
    /\ outA = OutputsA(live, val)
    /\ outB = OutputsB(live, val)
    /\ \A e \in live : gen[e] > 0 /\ val[e] \in Values
    /\ \A e \in Entities \ live : gen[e] = 0 /\ val[e] = 0
    /\ \A n \in 0..(nextNode - 1) :
        /\ DepsSatisfied(snapshots[n].live)
        /\ snapshots[n].outA = OutputsA(snapshots[n].live, snapshots[n].val)
        /\ snapshots[n].outB = OutputsB(snapshots[n].live, snapshots[n].val)

\* 3. The history is a rooted tree; sibling branches are kept.
TreeStructureValid ==
    /\ parent[0] = NullNode
    /\ \A n \in Recorded : parent[n] \in 0..(n - 1)
    /\ \A n \in nextNode..(MaxNodes - 1) : parent[n] = NullNode

\* 4. The journal and the tree agree: one key per recorded step, in each
\*    namespace, and no step under two keys of either. A writer records only
\*    edits: corrections are envelopes.
IdempotencyValid ==
    /\ \A j \in JournalKeys :
        idem[j].status = "recorded" => idem[j].node \in Recorded
    /\ \A j1, j2 \in JournalKeys :
        (idem[j1].status = "recorded" /\ idem[j2].status = "recorded"
            /\ j1 /= j2) => idem[j1].node /= idem[j2].node
    /\ \A n \in Recorded :
        \E j \in JournalKeys : idem[j].status = "recorded" /\ idem[j].node = n
    /\ \A k \in Keys :
        idem[Wr(k)].status = "recorded" => idem[Wr(k)].intent.kind = "edit"

\* 5. Generation custody. A generation names one occurrence of one entity.
\*    It is stable while the entity stays live, a birth or a recorded restore
\*    takes a generation no earlier step has seen, and each step's record of
\*    what it did matches its snapshots.
GenerationalCustody ==
    /\ \A n1, n2 \in 0..(nextNode - 1) : \A x, y \in Entities :
        (snapshots[n1].gen[x] > 0 /\ snapshots[n1].gen[x] = snapshots[n2].gen[y])
            => x = y
    /\ \A n \in 0..(nextNode - 1) : \A e \in Entities :
        snapshots[n].gen[e] < nextGen
    /\ \A n \in Recorded : \A e \in Entities :
        LET before == snapshots[parent[n]]
            after == snapshots[n]
        IN
        /\ ops[n][e] \in {"none", "set"} =>
            /\ (e \in before.live) = (e \in after.live)
            /\ after.gen[e] = before.gen[e]
        /\ ops[n][e] = "none" => after.val[e] = before.val[e]
        /\ ops[n][e] = "set" => e \in after.live
        /\ ops[n][e] = "retired" =>
            e \in before.live /\ e \notin after.live /\ after.gen[e] = 0
        /\ ops[n][e] \in {"born", "restored"} =>
            /\ e \notin before.live /\ e \in after.live
            /\ \A m \in 0..(n - 1) : \A x \in Entities :
                snapshots[m].gen[x] < after.gen[e]

\* 6. Stopped requests: trees are only ever bound to a parked candidate, and
\*    a closed document holds nothing.
StoppedCustody ==
    /\ parked \subseteq 1..(nextCand - 1)
    /\ forestOrigin /= 0 => forestOrigin \in parked
    /\ forestDisposing => forestOrigin /= 0
    /\ closed => (parked = {} /\ forestOrigin = 0 /\ writer = "none")

\* 6b. Every past head a move or recording leaves is queued for release: the
\*     queue keeps room for the one the document stands on, and close
\*     drains it.
QueuedHeads ==
    /\ forked => heads < MaxHeads
    /\ closed => heads = 0

\* 7. Nothing a stopped request leaves behind wedges the document. A writer
\*    that holds it can always give it back, and, on a head the document
\*    still owns with a free key and room for a step, can record. With
\*    WedgeBug = TRUE (TreeBasedUndoWedgeBug.cfg) TLC finds Phase 1's wedge:
\*    an envelope stops while recording its trees, then .edit begins.
NoWedge ==
    /\ (writer \in WriterClasses /\ ~closed) => ENABLED WriterDiscard
    /\ (writer \in WriterClasses /\ ~closed /\ ~headDiverged
            /\ nextNode < MaxNodes /\ \E k \in Keys : idem[Wr(k)].status = "unseen")
        => ENABLED (\E st \in StageMaps, k \in Keys : WriterCommit(st, k))

\* 8. Reversal facts follow navigation (spec-owner ruling (u)). A reversal's
\*    "already reversed" fact is active exactly on the lines where its step
\*    is an ancestor of the selected step, or the selected step itself. With
\*    FactBug = TRUE (TreeBasedUndoFactBug.cfg) TLC finds the r1 defect:
\*    delete by correction, undo, and the fact is still active.
FactsFollowLine == facts = {f \in PostedFacts : f.step \in Line(current)}

\* 9. Where a fact is active the snapshots agree the source is already
\*    reversed, so the fact never refuses what the selected line allows,
\*    and never allows what it refuses.
FactsAgreeWithSnapshots ==
    \A f \in facts :
        /\ MeaningDerivable(f.src, f.ent)
        /\ ~SourceReversible(f.src, f.ent)

\* ===========================================================================
\* Action properties
\* ===========================================================================

\* Recorded steps are never rewritten or erased.
AppendOnlyHistory ==
    [][\A n \in 0..(nextNode - 1) :
        /\ parent'[n] = parent[n]
        /\ snapshots'[n] = snapshots[n]
        /\ ops'[n] = ops[n]
    ]_vars

\* Once the head diverged the document never records, navigates or recovers.
OutOfBandHonesty ==
    [][headDiverged =>
        /\ headDiverged'
        /\ UNCHANGED <<treeVars, nextGen, branchVars, idem>>
    ]_vars

\* Recording a step, edit or correction, never diverges the head, and it
\* releases any trees a stopped request left bound.
RecordingKeepsCustody ==
    [][nextNode' /= nextNode =>
        /\ ~headDiverged'
        /\ forestOrigin' = 0
    ]_vars

\* A stopped request records nothing and frees its key.
StopsRecordNothing ==
    [][nextCand' /= nextCand =>
        UNCHANGED <<treeVars, nextGen, branchVars, idem, headDiverged>>
    ]_vars

\* A closed document does nothing more.
ClosedIsTerminal == [][closed => UNCHANGED vars]_vars

=============================================================================
