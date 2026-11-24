---------------------------- MODULE HostCustody ----------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Host source-custody state machine (worth-query-host/src/source_custody):
  models one move-only command slot across typed (`WorthQuerySourceCustody`)
  and type-erased (`WorthQueryAnySourceCustody`) ownership, ordinary and
  correction admission (`accept_correction`), lawful unexecuted withdrawal
  (`close_unexecuted`), typed downcast (`try_into_typed`), and cleanup close.

  Safety invariants:
    - NoDoubleExecution: the command commits at most once (`execCount <= 1`).
    - NoLostDurableOriginal: no terminal state leaks an undischarged original.
    - TerminalExclusivity: Committed has `execCount = 1 /\ committedValue`;
      Withdrawn and Denied have `execCount = 0 /\ ~committedValue`.
    - PendingClosePreservesCommit: a post-commit close refusal preserves both
      the single commit and the undischarged original until close succeeds.
    - UnexecutedNeverCommitted: pre-execution phases never observe a commit.

  Liveness property:
    - EventuallyTerminal: under weak fairness of non-fault progress steps
      (including typed downcast from erased `NativeObligationRetained`), every
      command reaches a terminal state with no leaked durable original.
*)

Phases == {
    "Empty",
    "Described",
    "PendingAdvance",
    "PendingCloseCommitted",
    "PendingCloseDeclined",
    "NativeObligationRetained",
    "TerminalCommitted",
    "TerminalWithdrawn",
    "TerminalDenied"
}

PendingPhases == {
    "Described",
    "PendingAdvance",
    "PendingCloseCommitted",
    "PendingCloseDeclined",
    "NativeObligationRetained"
}

TerminalPhases == {
    "TerminalCommitted",
    "TerminalWithdrawn",
    "TerminalDenied"
}

CorrectionOutcomes == {
    "CommittedClosed",
    "PendingClose",
    "PendingAdvance",
    "DeclinedClosed",
    "PublicationRefused"
}

VARIABLES
    phase,           \* Current custody phase
    erased,          \* FALSE = typed WorthQuerySourceCustody, TRUE = WorthQueryAnySourceCustody
    durableOriginal, \* TRUE iff a durable original record exists on disk awaiting discharge
    execCount,       \* Number of times the business mutation committed (must stay <= 1)
    correctionMode,  \* TRUE iff admitted via accept_correction
    committedValue   \* TRUE iff authoritative state reflects this command's commit

vars == <<phase, erased, durableOriginal, execCount, correctionMode, committedValue>>

Init ==
    /\ phase = "Empty"
    /\ erased = FALSE
    /\ durableOriginal = FALSE
    /\ execCount = 0
    /\ correctionMode = FALSE
    /\ committedValue = FALSE

\* ---------------------------------------------------------------------------
\* Transitions
\* ---------------------------------------------------------------------------

\* Admit a fresh ordinary command (typed or immediately erased).
AcceptOrdinary(isErased) ==
    /\ phase = "Empty"
    /\ isErased \in BOOLEAN
    /\ phase' = "Described"
    /\ erased' = isErased
    /\ durableOriginal' = FALSE
    /\ execCount' = 0
    /\ correctionMode' = FALSE
    /\ committedValue' = FALSE

\* Admit a prepared correction via accept_correction (typed or erased).
AcceptCorrection(isErased, outcome) ==
    /\ phase = "Empty"
    /\ isErased \in BOOLEAN
    /\ outcome \in CorrectionOutcomes
    /\ erased' = isErased
    /\ correctionMode' = TRUE
    /\ CASE outcome = "CommittedClosed" ->
            /\ phase' = "TerminalCommitted"
            /\ durableOriginal' = FALSE
            /\ execCount' = 1
            /\ committedValue' = TRUE
         [] outcome = "PendingClose" ->
            /\ phase' = "PendingCloseCommitted"
            /\ durableOriginal' = TRUE
            /\ execCount' = 1
            /\ committedValue' = TRUE
         [] outcome = "PendingAdvance" ->
            /\ phase' = "PendingAdvance"
            /\ durableOriginal' = TRUE
            /\ execCount' = 0
            /\ committedValue' = FALSE
         [] outcome = "DeclinedClosed" ->
            /\ phase' = "TerminalDenied"
            /\ durableOriginal' = FALSE
            /\ execCount' = 0
            /\ committedValue' = FALSE
         [] outcome = "PublicationRefused" ->
            /\ phase' = "TerminalDenied"
            /\ durableOriginal' = FALSE
            /\ execCount' = 0
            /\ committedValue' = FALSE

\* Erase a typed pending custody owner into WorthQueryAnySourceCustody.
Erase ==
    /\ phase \in PendingPhases
    /\ ~erased
    /\ erased' = TRUE
    /\ UNCHANGED <<phase, durableOriginal, execCount, correctionMode, committedValue>>

\* Consuming typed downcast (`try_into_typed`): succeeds on exact type match,
\* preserves exact erased owner (`Err(self)`) on mismatch.
Downcast(matched) ==
    /\ phase \in PendingPhases
    /\ erased
    /\ matched \in BOOLEAN
    /\ IF matched
          THEN /\ erased' = FALSE
               /\ UNCHANGED <<phase, durableOriginal, execCount, correctionMode, committedValue>>
          ELSE UNCHANGED vars

\* Described -> retention refused (cancelled or obligation pending after original written).
ResumeRetentionRefused(retainedOriginal) ==
    /\ phase = "Described"
    /\ retainedOriginal \in BOOLEAN
    /\ durableOriginal' = (durableOriginal \/ retainedOriginal)
    /\ UNCHANGED <<phase, erased, execCount, correctionMode, committedValue>>

\* Described -> durable original retained, then advance refused (e.g. expired principal).
ResumeRetainAndRefuse ==
    /\ phase = "Described"
    /\ phase' = "PendingAdvance"
    /\ durableOriginal' = TRUE
    /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>

\* Described or PendingAdvance -> executes once and closes cleanly.
ResumeCommitClosed ==
    /\ phase \in {"Described", "PendingAdvance"}
    /\ phase' = "TerminalCommitted"
    /\ durableOriginal' = FALSE
    /\ execCount' = execCount + 1
    /\ committedValue' = TRUE
    /\ UNCHANGED <<erased, correctionMode>>

\* Described or PendingAdvance -> executes once, then cleanup close refuses.
ResumeCommitPendingClose ==
    /\ phase \in {"Described", "PendingAdvance"}
    /\ phase' = "PendingCloseCommitted"
    /\ durableOriginal' = TRUE
    /\ execCount' = execCount + 1
    /\ committedValue' = TRUE
    /\ UNCHANGED <<erased, correctionMode>>

\* PendingAdvance -> advance refused again (e.g. expired principal or budget stop).
ResumeAdvanceRefused ==
    /\ phase = "PendingAdvance"
    /\ UNCHANGED vars

\* Described or PendingAdvance -> terminal decline (e.g. intent drift), closes cleanly.
ResumeDeclineClosed ==
    /\ phase \in {"Described", "PendingAdvance"}
    /\ phase' = "TerminalDenied"
    /\ durableOriginal' = FALSE
    /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>

\* Described or PendingAdvance -> terminal decline, cleanup close refuses.
ResumeDeclinePendingClose ==
    /\ phase \in {"Described", "PendingAdvance"}
    /\ phase' = "PendingCloseDeclined"
    /\ durableOriginal' = TRUE
    /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>

\* PendingAdvance -> native recovery obligation retained (post-execution / indeterminate).
ResumeEnterNativeObligation ==
    /\ phase = "PendingAdvance"
    /\ phase' = "NativeObligationRetained"
    /\ durableOriginal' = TRUE
    /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>

\* Lawful unexecuted withdrawal (`close_unexecuted`) on typed OR erased custody:
\* - On Described / PendingAdvance: discharges durable original if request succeeds,
\*   else preserves custody intact.
\* - On post-execution / NativeObligationRetained: MUST refuse and preserve custody!
WithdrawUnexecuted(succeeds) ==
    /\ phase \in PendingPhases
    /\ succeeds \in BOOLEAN
    /\ IF phase \in {"Described", "PendingAdvance"} /\ succeeds
          THEN /\ phase' = "TerminalWithdrawn"
               /\ durableOriginal' = FALSE
               /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>
          ELSE UNCHANGED vars

\* Close a pending post-execution owner (`resume` on PendingClose*, or typed
\* native recovery disposal after `try_into_typed` on NativeObligationRetained).
ClosePending(succeeds) ==
    /\ succeeds \in BOOLEAN
    /\ \/ /\ phase = "PendingCloseCommitted"
          /\ IF succeeds
                THEN /\ phase' = "TerminalCommitted"
                     /\ durableOriginal' = FALSE
                     /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>
                ELSE UNCHANGED vars
       \/ /\ phase = "PendingCloseDeclined"
          /\ IF succeeds
                THEN /\ phase' = "TerminalDenied"
                     /\ durableOriginal' = FALSE
                     /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>
                ELSE UNCHANGED vars
       \/ /\ phase = "NativeObligationRetained"
          /\ ~erased
          /\ IF succeeds
                THEN /\ phase' = "TerminalDenied"
                     /\ durableOriginal' = FALSE
                     /\ UNCHANGED <<erased, execCount, correctionMode, committedValue>>
                ELSE UNCHANGED vars

\* Terminal stuttering step (prevents artificial deadlock at terminal states).
Done ==
    /\ phase \in TerminalPhases
    /\ UNCHANGED vars

Next ==
    \/ \E e \in BOOLEAN: AcceptOrdinary(e)
    \/ \E e \in BOOLEAN, o \in CorrectionOutcomes: AcceptCorrection(e, o)
    \/ Erase
    \/ \E m \in BOOLEAN: Downcast(m)
    \/ \E r \in BOOLEAN: ResumeRetentionRefused(r)
    \/ ResumeRetainAndRefuse
    \/ ResumeCommitClosed
    \/ ResumeCommitPendingClose
    \/ ResumeAdvanceRefused
    \/ ResumeDeclineClosed
    \/ ResumeDeclinePendingClose
    \/ ResumeEnterNativeObligation
    \/ \E s \in BOOLEAN: WithdrawUnexecuted(s)
    \/ \E s \in BOOLEAN: ClosePending(s)
    \/ Done

\* Fair non-fault progress steps for liveness verification:
\* once faults stop, a caller can always drive any state to a terminal phase.
Fairness ==
    /\ WF_vars(AcceptOrdinary(FALSE))
    /\ WF_vars(ResumeCommitClosed)
    /\ WF_vars(WithdrawUnexecuted(TRUE))
    /\ SF_vars(ClosePending(TRUE))
    /\ WF_vars(Downcast(TRUE))

Spec == Init /\ [][Next]_vars /\ Fairness

\* ===========================================================================
\* Safety Invariants & Liveness Property
\* ===========================================================================

TypeOK ==
    /\ phase \in Phases
    /\ erased \in BOOLEAN
    /\ durableOriginal \in BOOLEAN
    /\ execCount \in {0, 1}
    /\ correctionMode \in BOOLEAN
    /\ committedValue \in BOOLEAN

NoDoubleExecution ==
    execCount <= 1

NoLostDurableOriginal ==
    (phase \in TerminalPhases) => ~durableOriginal

TerminalExclusivity ==
    /\ (phase = "TerminalCommitted") => (execCount = 1 /\ committedValue)
    /\ (phase \in {"TerminalWithdrawn", "TerminalDenied"}) => (execCount = 0 /\ ~committedValue)

PendingClosePreservesCommit ==
    (phase = "PendingCloseCommitted") => (execCount = 1 /\ committedValue /\ durableOriginal)

UnexecutedNeverCommitted ==
    (phase \in {"Empty", "Described", "PendingAdvance"}) => (execCount = 0 /\ ~committedValue)

EventuallyTerminal ==
    <>(phase \in TerminalPhases)

=============================================================================
