import StoreM4.RetainedBasisCursor

/-!
# Executable Rust↔Lean Conformance Checker (`retained_basis_conformance`)

Reads a line-delimited trace emitted directly by WORTH's Rust runtime and steps
the EXACT proved Lean transition functions in `StoreM4.RetainedBasisCursor`,
checking observed cursor positions, canonical keys, payload digests, history
digests, AND actual durable pin/pending-retire/retained-basis counts.
-/

open StoreM4.RetainedBasisCursor

private def parseNat (s : String) : Except String Nat :=
  match s.toNat? with
  | some n => .ok n
  | none => .error s!"invalid nat: `{s}`"

private def parseOptNat (s : String) : Except String (Option Nat) :=
  if s == "-" || s == "none" then .ok none
  else return some (← parseNat s)

private def parseSemiNats (s : String) : Except String (List Nat) :=
  if s.isEmpty || s == "-" then .ok []
  else (s.splitOn ";").mapM parseNat

private def parseRow (item : String) : Except String Row := do
  match item.splitOn ":" with
  | [ks, ps, hs] => return ⟨← parseNat ks, ← parseNat ps, ← parseSemiNats hs⟩
  | [ks, ps] => return ⟨← parseNat ks, ← parseNat ps, []⟩
  | _ => throw s!"invalid row `{item}` (expected key:payload:h1;h2)"

private def parseRows (s : String) : Except String (List Row) :=
  if s.isEmpty || s == "-" then .ok []
  else (s.splitOn ",").mapM parseRow

private def formatRow (r : Row) : String :=
  let hs := if r.history.isEmpty then "-" else String.intercalate ";" (r.history.map toString)
  s!"{r.key}:{r.payload}:{hs}"

private def formatRows (rows : List Row) : String :=
  if rows.isEmpty then "-" else String.intercalate "," (rows.map formatRow)

private def formatOptNat : Option Nat → String
  | none => "-"
  | some k => toString k

private def formatOutcome (afterKey : Option Nat) : ResumeOutcome → String
  | .admitted _ cid rows =>
      let finalPos := match rows.getLast? with
        | some r => some r.key
        | none => afterKey
      s!"admitted:{cid}:{formatOptNat finalPos}:{formatRows rows}"
  | .historyUnavailable => "historyUnavailable"
  | .unsupportedMeaning => "unsupportedMeaning"
  | .corruptStorage => "corruptStorage"

private def stripLine (s : String) : String :=
  ((s.replace "\r" "").replace "\uFEFF" "").trimAscii.toString

private def stepLine (lineNo : Nat) (s : Sys) (line : String) : Except String Sys := do
  let trimmed := stripLine line
  if trimmed.isEmpty || trimmed.startsWith "#" then
    return s
  let parts := trimmed.splitOn " "
  match parts with
  | ["INIT", homeS, macS, incS, schemaS, queryS] =>
      return {
        homeId := ← parseNat homeS
        macKey := ← parseNat macS
        incarnation := ← parseNat incS
        nextPinId := 1
        headCommit := 0
        headRows := []
        supportedSchema := ← parseNat schemaS
        expectedQuery := ← parseNat queryS
        bases := []
        durablePins := []
        pendingRetirePins := []
      }
  | ["RETAIN", cidS, origS, commS, rowsS] =>
      return retainComplete s (← parseNat cidS) (← parseNat origS) (← parseNat commS) (← parseRows rowsS)
  | ["CUT_R1_STAGE", cidS, origS, commS, rowsS] =>
      return stageRetain s (← parseNat cidS) (← parseNat origS) (← parseNat commS) (← parseRows rowsS)
  | ["CUT_R2_ORPHAN", cidS] =>
      return pinRetainOrphan s (← parseNat cidS)
  | ["BIND_RETAIN", cidS] =>
      return bindRetain s (← parseNat cidS)
  | ["CONSOLIDATE", cidS] =>
      let cid ← parseNat cidS
      return expireConsolidateOldPin (bindConsolidate (stageConsolidate s cid) cid) cid
  | ["CUT_C1_STAGE_CONSOLIDATE", cidS] =>
      return stageConsolidate s (← parseNat cidS)
  | ["CUT_C2_PIN_CONSOLIDATE", cidS] =>
      return pinConsolidateReplacement s (← parseNat cidS)
  | ["CUT_C3_BIND_CONSOLIDATE", cidS] =>
      return bindConsolidate s (← parseNat cidS)
  | ["CUT_P1_PRUNE_CATALOG", cidS] =>
      return pruneCatalogPendingExpire s (← parseNat cidS)
  | ["PRUNE", cidS] =>
      return pruneBasis s (← parseNat cidS)
  | ["RECLAIM", cidS] =>
      return reclaimBasis s (← parseNat cidS)
  | ["RELEASE", cidS] =>
      return releaseLoan s (← parseNat cidS)
  | ["ADVANCE_HEAD", cidS, rowsS] =>
      return advanceHead s (← parseNat cidS) (← parseRows rowsS)
  | ["REOPEN"] =>
      return reconcileOnOpen s
  | ["RESUME", macKeyS, homeS, incS, origS, cidS, commS, schemaS, queryS, afterS, expectS] =>
      let afterKey ← parseOptNat afterS
      let tok := issueToken
        (← parseNat macKeyS)
        (← parseNat homeS)
        (← parseNat incS)
        (← parseNat origS)
        (← parseNat cidS)
        (← parseNat commS)
        (← parseNat schemaS)
        (← parseNat queryS)
        afterKey
      let outcome := resumeToken s tok
      let actual := formatOutcome afterKey outcome
      if actual != expectS then
        throw s!"line {lineNo}: RESUME mismatch: lean=`{actual}`, rust=`{expectS}`"
      match outcome with
      | .admitted s' _ _ => return s'
      | _ => return s
  | ["ASSERT_PINS", liveS, pendingS, retainedS] =>
      let rustLive ← parseNat liveS
      let rustPending ← parseNat pendingS
      let rustRetained ← parseNat retainedS
      let leanLive := (s.durablePins.filter (fun p => p.active)).length
      let leanPending := s.pendingRetirePins.length
      let leanRetained := (s.bases.filter (fun b => b.independentlyRetained)).length
      if leanLive != rustLive || leanPending != rustPending || leanRetained != rustRetained then
        throw s!"line {lineNo}: pin/basis count mismatch: lean=(live={leanLive},pending={leanPending},retained={leanRetained}) rust=(live={rustLive},pending={rustPending},retained={rustRetained})"
      let bound := boundPinIds s.bases
      if s.pendingRetirePins.isEmpty && s.durablePins.any (fun p => p.active && !bound.contains p.pinId) then
        throw s!"line {lineNo}: unbound active durable pin detected"
      return s
  | ["ASSERT_CHUNK_INDEX", chunksS, afterS, expectRowsS] =>
      let chunkStrs := if chunksS == "-" then [] else chunksS.splitOn "|"
      let chunks ← chunkStrs.mapM parseRows
      let after ← parseOptNat afterS
      let indexed := formatRows (resumeWithChunkIndex chunks after)
      let direct := formatRows (canonicalSuffix chunks.flatten after)
      if indexed != direct || indexed != expectRowsS then
        throw s!"line {lineNo}: chunk index mismatch: indexed=`{indexed}`, direct=`{direct}`, rust=`{expectRowsS}`"
      return s
  | _ =>
      throw s!"line {lineNo}: unknown trace command `{trimmed}`"

def main (args : List String) : IO UInt32 := do
  match args with
  | [tracePath] =>
      let contents ← IO.FS.readFile tracePath
      let lines := contents.splitOn "\n"
      let initSys : Sys := {
        homeId := 0, macKey := 0, incarnation := 0, nextPinId := 1
        headCommit := 0, headRows := [], supportedSchema := 0, expectedQuery := 0
        bases := [], durablePins := [], pendingRetirePins := []
      }
      let mut state := initSys
      let mut steps := 0
      let mut lineNo := 1
      for line in lines do
        match stepLine lineNo state line with
        | .ok next =>
            let trimmed := stripLine line
            if !trimmed.isEmpty && !trimmed.startsWith "#" then
              steps := steps + 1
            state := next
        | .error msg =>
            IO.eprintln s!"CONFORMANCE FAILED: {msg}"
            return 1
        lineNo := lineNo + 1
      IO.println s!"CONFORMANCE OK ({steps} steps verified)"
      return 0
  | _ =>
      IO.eprintln "usage: retained_basis_conformance <trace-file>"
      return 2
