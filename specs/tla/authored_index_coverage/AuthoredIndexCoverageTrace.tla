---------------------- MODULE AuthoredIndexCoverageTrace ----------------------
(***************************************************************************)
(* Trace validation for AuthoredIndexCoverage (public #384).               *)
(*                                                                         *)
(* The relational courts in authored_write_history/indexed/durable_coverage *)
(* write one NDJSON trace per recovery when WORTH_DST_TRACE_DIR is set.     *)
(* Each record is one of:                                                  *)
(*   {"action":"Register"}            a definition is installed            *)
(*   {"action":"Commit","packet":P,"bound":B}                             *)
(*                                    P is the packet the durable artifact *)
(*                                    holds: "Covered", "Unavail" or       *)
(*                                    "Legacy" (read from its bytes); B is *)
(*                                    a Covered packet's persisted bound   *)
(*                                    (0 otherwise), which must equal the  *)
(*                                    catalog's next id at that commit     *)
(*   {"action":"Checkpoint"}          the recovery's checkpoint covers     *)
(*                                    every commit so far                  *)
(*   {"action":"CrashReopen"}         recovery from that split             *)
(*   {"action":"Admit","index":d,"span":s,"outcome":"Ok"|"Refuse"}         *)
(*                                    the recovered cursor's answer for    *)
(*                                    index d over commits s..n            *)
(*   {"action":"Postings","index":d,"span":s,"commits":{"__set__":[...]}}  *)
(*                                    an admitted scan's commits           *)
(* Commits are numbered from 1 in commit order; the recovered view is the  *)
(* checkpoint frontier `ckpt`. A trace passes only if every record is a    *)
(* transition or observation the model allows.                             *)
(***************************************************************************)
EXTENDS AuthoredIndexCoverage, TraceData

VARIABLE pc

traceVars == <<vars, pc>>

TraceInit ==
    /\ Init
    /\ pc = 1

Observe(step) ==
    /\ UNCHANGED vars
    /\ CASE step.action = "Admit" ->
                Admit(ckpt, step.index, step.span) = step.outcome
         [] step.action = "Postings" ->
                /\ Admit(ckpt, step.index, step.span) = "Ok"
                /\ step.commits = {o \in step.span..Len(commits) :
                                       step.index \in commits[o].writes}

StepAction(step) ==
    CASE step.action = "Register" -> Register
      [] step.action = "Commit" /\ step.packet = "Legacy" -> InjectLegacy
      [] step.action = "Commit" ->
             /\ step.packet = "Covered" => step.bound = cat.next
             /\ Commit(step.packet)
      [] step.action = "Checkpoint" -> Checkpoint
      [] step.action = "CrashReopen" -> CrashReopen
      [] step.action \in {"Admit", "Postings"} -> Observe(step)

TraceStep ==
    /\ pc <= Len(Trace)
    /\ StepAction(Trace[pc])
    /\ pc' = pc + 1

TraceDone ==
    /\ pc > Len(Trace)
    /\ UNCHANGED traceVars

TraceNext == TraceStep \/ TraceDone

TraceSpec == TraceInit /\ [][TraceNext]_traceVars /\ WF_traceVars(TraceStep)

TraceAccepted == <>(pc = Len(Trace) + 1)

=============================================================================
