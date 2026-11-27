import Lean.Data.Json
import StoreM4

/-!
# Executable Rust <-> StoreM4 model conformance

Reads line-delimited JSON transition traces emitted by the real compiled Rust
production implementations in `worth-store` (`store_m4_lean_conformance_trace`)
and mechanically verifies every observed transition and decision against all
five proved modules of `StoreM4`:

* `pin_expire_step` (`StoreM4.PinSafety`): checks `PinTableKernel::expire_lease`
  and `is_root_protected` against `PinEntry.expireStep`, `PinTable.expireById`,
  and `PinTable.reachableRoot`.
* `gc_pin_transition` (`StoreM4.GCPinSafety`): checks `PinTableKernel`
  (`acquire_pin`, `acquire_reader_loan`, `release_reader_loan`, `expire_lease`)
  against `pinAcquire`, `loanAcquire`, `loanRelease`, `leaseExpire`, `childCount`,
  and root protection reachability.
* `chunked_partition` (`StoreM4.ChunkedCoverage`): checks fixed-size chunk
  sequence admission (`BlobChunkSequenceAdmission`) against `fixedPartition`,
  `fixedChunk`, and `chunkCount`, verifying complete non-overlapping coverage
  and exact refusal of gaps, overlaps, short interior chunks, and missing tails.
* `extent_overlap`, `extent_gen_order`, `extent_drain`, `extent_lookup`
  (`StoreM4.ExtentGenerationSafety`): checks `ExtentArenaRange::overlaps`,
  `ExtentAllocationGeneration` lexicographic ordering, `PhysicalResidencyPool`'s
  `drain_writeback_fence` and `require_arena_frame_identity` against
  `Range.overlaps`, `Gen.lt`, `Barrier.admits`, and `lookupMatches`.
* `reclaim_verdict`, `reclaim_resume` (`StoreM4.ReclamationSafety`): checks
  `plan_reclaim_intent` (and `plan_reclaim`) against `planVerdict`, and
  `resume_release` against `resumeRelease`.

Exits 0 iff every step refines the proved models. A checker, not a model: it
carries no `proof-mirror:` pin.
-/

open Lean StoreM4

private def expect (ok : Bool) (what : String) : Except String Unit :=
  unless ok do throw what

private def natAt (j : Json) (key : String) : Except String Nat := do
  (← j.getObjVal? key).getNat?

private def boolAt (j : Json) (key : String) : Except String Bool := do
  (← j.getObjVal? key).getBool?

private def strAt (j : Json) (key : String) : Except String String := do
  (← j.getObjVal? key).getStr?

private def optNatAt (j : Json) (key : String) : Except String (Option Nat) := do
  let v ← j.getObjVal? key
  if v.isNull then pure none else some <$> v.getNat?

private def parseLeaseState (s : String) : Except String PinSafety.LeaseState :=
  match s with
  | "active" => pure .active
  | "expiring" => pure .expiring
  | "expired" => pure .expired
  | other => throw s!"unknown lease state {other}"

private def parsePinEntry (j : Json) : Except String PinSafety.PinEntry := do
  pure {
    id := ← natAt j "id"
    root := ← natAt j "root"
    parent := ← optNatAt j "parent"
    state := ← parseLeaseState (← strAt j "state")
    activeReaderLoans := ← natAt j "loans"
    liveChildPins := ← natAt j "children"
  }

private def parsePinTable (j : Json) (key : String) : Except String PinSafety.PinTable := do
  (← (← j.getObjVal? key).getArr?).toList.mapM parsePinEntry

private def parseRootProbes (j : Json) (key : String) : Except String (List (Nat × Bool)) := do
  (← (← j.getObjVal? key).getArr?).toList.mapM fun row => do
    match (← row.getArr?).toList with
    | [r, p] => pure (← r.getNat?, ← p.getBool?)
    | _ => throw s!"not a [root, protected] probe: {row.compress}"

private def reachableRootBool (t : PinSafety.PinTable) (r : Nat) : Bool :=
  t.any fun e => e.root == r && e.state != PinSafety.LeaseState.expired

private def checkRootProbes (t : PinSafety.PinTable) (probes : List (Nat × Bool)) :
    Except String Unit := do
  for (r, observed) in probes do
    let expected := reachableRootBool t r
    expect (observed == expected)
      s!"root {r} protection observed={observed}, model={expected}"

private def checkPinExpireStep (j : Json) : Except String Unit := do
  let before ← parsePinTable j "before"
  let after ← parsePinTable j "after"
  let targetId ← natAt j "target_id"
  let probes ← parseRootProbes j "probes"
  let expected := PinSafety.PinTable.expireById before targetId
  expect (after == expected)
    s!"PinSafety.expireById mismatch: Rust={repr after}, model={repr expected}"
  checkRootProbes after probes

private def checkGCPinTransition (j : Json) : Except String Unit := do
  let op ← strAt j "op"
  let before ← parsePinTable j "before"
  let after ← parsePinTable j "after"
  let ok ← boolAt j "ok"
  let id ← natAt j "id"
  let probes ← parseRootProbes j "probes"
  let model : Option PinSafety.PinTable ← match op with
    | "acquire" =>
      pure (GCPinSafety.pinAcquire before id (← natAt j "root") (← optNatAt j "parent"))
    | "loan_acquire" =>
      pure (GCPinSafety.loanAcquire before id)
    | "loan_release" =>
      pure (GCPinSafety.loanRelease before id)
    | "expire" =>
      pure (GCPinSafety.leaseExpire before id)
    | other => throw s!"unknown gc_pin_transition op {other}"
  match model with
  | none =>
    expect (!ok) s!"Rust accepted {op} on pin {id} where GCPinSafety model refuses"
    expect (after == before) s!"refused {op} mutated pin table"
  | some expected =>
    expect ok s!"Rust refused {op} on pin {id} where GCPinSafety model accepts"
    expect (after == expected)
      s!"GCPinSafety {op} post-table mismatch: Rust={repr after}, model={repr expected}"
  checkRootProbes after probes
  for e in after do
    let liveChildren := GCPinSafety.childCount after e.id
    expect (liveChildren ≤ e.liveChildPins)
      s!"pin {e.id} childCount={liveChildren} exceeds liveChildPins={e.liveChildPins}"

private def parseSlotRange (j : Json) : Except String ChunkedCoverage.SlotRange := do
  match (← j.getArr?).toList with
  | [s, e] => pure ⟨← s.getNat?, ← e.getNat?⟩
  | _ => throw s!"not a [start, stop] slot range: {j.compress}"

private def parseSlotRanges (j : Json) (key : String) :
    Except String (List ChunkedCoverage.SlotRange) := do
  (← (← j.getObjVal? key).getArr?).toList.mapM parseSlotRange

private def rangeContainsBool (r : ChunkedCoverage.SlotRange) (slot : Nat) : Bool :=
  r.start ≤ slot && slot < r.stop

private def checkChunkedPartition (j : Json) : Except String Unit := do
  let c ← natAt j "chunk_width"
  let total ← natAt j "total"
  let submitted ← parseSlotRanges j "submitted"
  let admitted ← boolAt j "admitted"
  let outputRanges ← parseSlotRanges j "output_ranges"
  let reportedCount ← natAt j "chunk_count"
  let modelPart := if c == 0 then [] else ChunkedCoverage.fixedPartition c total
  let modelCount := if c == 0 then 0 else ChunkedCoverage.chunkCount c total
  let modelAdmits := c > 0 && total > 0 && submitted == modelPart
  expect (admitted == modelAdmits)
    s!"chunked_partition (c={c}, total={total}): Rust admitted={admitted}, model={modelAdmits}"
  if admitted then
    expect (outputRanges == modelPart)
      s!"chunked_partition output ranges {repr outputRanges} != model {repr modelPart}"
    expect (reportedCount == modelCount)
      s!"chunked_partition count {reportedCount} != model {modelCount}"
    for slot in List.range total do
      let hits := outputRanges.filter (fun r => rangeContainsBool r slot)
      let own := ChunkedCoverage.fixedChunk c total (slot / c)
      expect (hits == [own])
        s!"slot {slot} covered by {repr hits}, expected [{repr own}]"

private def parseRange (j : Json) : Except String ExtentGenerationSafety.Range := do
  match (← j.getArr?).toList with
  | [a, o, l] => pure ⟨← a.getNat?, ← o.getNat?, ← l.getNat?⟩
  | _ => throw s!"not a [arena, offset, length] range: {j.compress}"

private def parseGen (j : Json) : Except String ExtentGenerationSafety.Gen := do
  match (← j.getArr?).toList with
  | [r, s] => pure ⟨← r.getNat?, ← s.getNat?⟩
  | _ => throw s!"not a [root, seq] generation: {j.compress}"

private def parseOptGen (j : Json) (key : String) :
    Except String (Option ExtentGenerationSafety.Gen) := do
  let v ← j.getObjVal? key
  if v.isNull then pure none else some <$> parseGen v

private def parseIdent (j : Json) : Except String ExtentGenerationSafety.Ident := do
  pure ⟨← parseRange (← j.getObjVal? "range"), ← parseGen (← j.getObjVal? "gen")⟩

private def parseBarrier (s : String) : Except String ExtentGenerationSafety.Barrier :=
  match s with
  | "retirement_incomplete" => pure .retirementIncomplete
  | "completed_retirement" => pure .completedRetirement
  | "cancel_uncovered" => pure .cancelUncovered
  | "checkpoint_covered_cancel" => pure .checkpointCoveredCancel
  | other => throw s!"unknown extent reuse barrier {other}"

private local instance (a b : ExtentGenerationSafety.Gen) :
    Decidable (ExtentGenerationSafety.Gen.lt a b) := by
  unfold ExtentGenerationSafety.Gen.lt
  infer_instance

private local instance (f : ExtentGenerationSafety.Ident)
    (r : ExtentGenerationSafety.Range) (g : ExtentGenerationSafety.Gen) :
    Decidable (ExtentGenerationSafety.lookupMatches f r g) := by
  unfold ExtentGenerationSafety.lookupMatches
  infer_instance

private def barrierAdmitsBool : ExtentGenerationSafety.Barrier → Bool
  | .completedRetirement => true
  | .checkpointCoveredCancel => true
  | _ => false

private def checkExtentOverlap (j : Json) : Except String Unit := do
  let a ← parseRange (← j.getObjVal? "a")
  let b ← parseRange (← j.getObjVal? "b")
  let observed ← boolAt j "overlaps"
  let expected := decide (a.overlaps b)
  expect (observed == expected)
    s!"ExtentArenaRange.overlaps({repr a}, {repr b}): Rust={observed}, model={expected}"

private def checkExtentGenOrder (j : Json) : Except String Unit := do
  let a ← parseGen (← j.getObjVal? "a")
  let b ← parseGen (← j.getObjVal? "b")
  let observed ← boolAt j "lt"
  let expected := decide (a.lt b)
  expect (observed == expected)
    s!"ExtentAllocationGeneration.lt({repr a}, {repr b}): Rust={observed}, model={expected}"

private def checkExtentDrain (j : Json) : Except String Unit := do
  let range ← parseRange (← j.getObjVal? "range")
  let barrier ← parseBarrier (← strAt j "barrier")
  let nextGen ← parseGen (← j.getObjVal? "next_gen")
  let floorGen ← parseGen (← j.getObjVal? "floor_gen")
  let seen ← (← (← j.getObjVal? "seen").getArr?).toList.mapM parseIdent
  let inflight ← (← (← j.getObjVal? "inflight").getArr?).toList.mapM parseRange
  let admitted ← boolAt j "admitted"
  let returnedGen ← parseOptGen j "returned_gen"
  let activeAfter ← parseGen (← j.getObjVal? "active_after")
  let drained := inflight.all fun w => !decide (range.overlaps w)
  let exceedsSeen := seen.all fun e =>
    !decide (e.range.overlaps range) || decide (e.gen.lt nextGen)
  let exceedsFloor := (floorGen == ⟨0, 0⟩) || decide (floorGen.lt nextGen)
  let modelAdmits := barrierAdmitsBool barrier && drained && exceedsSeen && exceedsFloor
  expect (admitted == modelAdmits)
    s!"extent_drain on {repr range} at {repr nextGen}: Rust admitted={admitted}, model={modelAdmits}"
  if admitted then
    expect (returnedGen == some nextGen)
      s!"extent_drain returned {repr returnedGen}, expected {repr (some nextGen)}"
    expect (activeAfter == nextGen)
      s!"extent_drain active_after={repr activeAfter}, expected {repr nextGen}"
  else
    expect (returnedGen == none)
      s!"refused extent_drain unexpectedly returned {repr returnedGen}"
    expect (!decide (activeAfter.lt floorGen))
      s!"extent_drain active_after={repr activeAfter} rewound below floor {repr floorGen}"

private def checkExtentLookup (j : Json) : Except String Unit := do
  let frame ← parseIdent (← j.getObjVal? "frame")
  let qRange ← parseRange (← j.getObjVal? "query_range")
  let qGen ← parseGen (← j.getObjVal? "query_gen")
  let ok ← boolAt j "ok"
  let expected := decide (ExtentGenerationSafety.lookupMatches frame qRange qGen)
  expect (ok == expected)
    s!"extent_lookup({repr frame}, {repr qRange}, {repr qGen}): Rust={ok}, model={expected}"

private def parseSpan (j : Json) : Except String ReclamationSafety.Span := do
  match (← j.getArr?).toList with
  | [sp, k, f, l] => pure ⟨← sp.getNat?, ← k.getNat?, ← f.getNat?, ← l.getNat?⟩
  | _ => throw s!"not a [space, key, first, last] span: {j.compress}"

private def parseSpans (j : Json) (key : String) :
    Except String (List ReclamationSafety.Span) := do
  (← (← j.getObjVal? key).getArr?).toList.mapM parseSpan

private def parseCoverage (s : String) : Except String ReclamationSafety.Coverage :=
  match s with
  | "canonical_successor" => pure .canonicalSuccessor
  | "derived_only" => pure .derivedOnly
  | "uncovered" => pure .uncovered
  | other => throw s!"unknown reclaim coverage {other}"

private def parseVerdict (s : String) : Except String ReclamationSafety.Verdict :=
  match s with
  | "accept" => pure .accept
  | "reachable" => pure .reachable
  | "uncovered" => pure .uncovered
  | "obligated" => pure .obligated
  | other => throw s!"unknown reclaim verdict {other}"

private def parseReleaseBarrier (s : String) : Except String ReclamationSafety.Barrier :=
  match s with
  | "intent_durable" => pure .intentDurable
  | "free_space_published" => pure .freeSpacePublished
  | "completion_durable" => pure .completionDurable
  | other => throw s!"unknown release barrier {other}"

private def checkReclaimVerdict (j : Json) : Except String Unit := do
  let candidate ← parseSpans j "candidate"
  let reach ← parseSpans j "reach"
  let obligations ← parseSpans j "obligations"
  let coverage ← parseCoverage (← strAt j "coverage")
  let observed ← parseVerdict (← strAt j "verdict")
  let expected := ReclamationSafety.planVerdict candidate reach obligations coverage
  expect (observed == expected)
    s!"planVerdict mismatch: Rust={repr observed}, model={repr expected}"

private def checkReclaimResume (j : Json) : Except String Unit := do
  let barrier ← parseReleaseBarrier (← strAt j "barrier")
  let observed : ReclamationSafety.ResumePlan := {
    publish := ← boolAt j "publish"
    complete := ← boolAt j "complete"
    release := ← boolAt j "release"
  }
  let expected := ReclamationSafety.resumeRelease barrier
  expect (observed == expected)
    s!"resumeRelease({repr barrier}) mismatch: Rust={repr observed}, model={repr expected}"

private def checkStep (j : Json) : Except String Unit := do
  match ← strAt j "kind" with
  | "pin_expire_step" => checkPinExpireStep j
  | "gc_pin_transition" => checkGCPinTransition j
  | "chunked_partition" => checkChunkedPartition j
  | "extent_overlap" => checkExtentOverlap j
  | "extent_gen_order" => checkExtentGenOrder j
  | "extent_drain" => checkExtentDrain j
  | "extent_lookup" => checkExtentLookup j
  | "reclaim_verdict" => checkReclaimVerdict j
  | "reclaim_resume" => checkReclaimResume j
  | other => throw s!"unknown step kind {other}"

def main (args : List String) : IO UInt32 := do
  if args.isEmpty then
    IO.eprintln "usage: store_m4_core_conformance <trace.ndjson>..."
    return 2
  let mut totalChecked := 0
  for path in args do
    let lines := (← IO.FS.lines path).toList.filter (· ≠ "")
    if lines.isEmpty then
      IO.eprintln s!"conformance: {path} is empty"
      return 1
    let mut stepIndex := 0
    for line in lines do
      let result : Except String Unit := do
        let j ← Json.parse line
        let idx ← natAt j "step"
        unless idx == stepIndex do
          throw s!"step {idx} is out of sequence (expected {stepIndex})"
        checkStep j
      match result with
      | .ok () =>
        stepIndex := stepIndex + 1
        totalChecked := totalChecked + 1
      | .error e =>
        IO.eprintln s!"conformance: {path} step {stepIndex} fails: {e}"
        return 1
  IO.println s!"conformance: {totalChecked} steps refine the proved StoreM4 models"
  return 0
