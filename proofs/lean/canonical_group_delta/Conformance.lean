import Lean.Data.Json
import CanonicalGroupDelta.Delta

/-!
# Executable Rust ↔ model conformance

Reads the NDJSON trace the real Rust group state emits (one line per
committed transition of one group: its prior membership, the recorded delta
or initial members, the installed and the `GROUP_LOG`-replayed memberships,
and the change's WPGL v3 wire bytes) and checks every step against the
proved functions of `CanonicalGroupDelta.Delta`:

* a kept step's installed membership is `applyDelta prior delta`, in
  `|removed| + |upserted| + 1` steps;
* its recorded delta is exactly `diff prior post` (canonical and minimal);
* its wire bytes are `encode`'s kept body minus the eight count bytes, with
  37-byte framed entity keys and 67-byte projected keys, plus 17 bytes
  for the native carrier and role on upserts;
* born and revived steps install exactly their initial members from an
  absent group, retirement leaves none, and replay agrees with the warm state.

Exits 0 iff every step refines the model. A checker, not a model: it
carries no `proof-mirror:` pin.
-/
open Lean CanonicalGroupDelta

/-- A member's carrier and role, as the trace numbers them. -/
abbrev Val := Nat × Nat

def membersOf (l : List (Nat × Val)) : Members Val :=
  fun x => (l.find? (·.1 = x)).map (·.2)

structure Step where
  index : Nat
  kind : String
  prior : List (Nat × Val)
  removed : List Nat
  upserted : List (Nat × Val)
  post : List (Nat × Val)
  replayed : List (Nat × Val)
  wire : Nat

def triple (j : Json) : Except String (Nat × Val) := do
  match (← j.getArr?).toList with
  | [n, c, r] => pure (← n.getNat?, (← c.getNat?, ← r.getNat?))
  | _ => throw s!"member is not [name, carrier, role]: {j.compress}"

def entries (j : Json) (key : String) : Except String (List (Nat × Val)) := do
  (← (← j.getObjVal? key).getArr?).toList.mapM triple

def parseStep (line : String) : Except String Step := do
  let j ← Json.parse line
  let removed ← (← (← j.getObjVal? "removed").getArr?).toList.mapM (·.getNat?)
  pure {
    index := ← (← j.getObjVal? "step").getNat?
    kind := ← (← j.getObjVal? "kind").getStr?
    prior := ← entries j "prior"
    removed
    upserted := ← entries j "upserted"
    post := ← entries j "post"
    replayed := ← entries j "replayed"
    wire := ← (← j.getObjVal? "wire_delta_bytes").getNat? }

/-- Every name the step mentions (trace names are single bytes). -/
def support (s : Step) : List Nat :=
  (List.range 256).filter fun x =>
    (s.prior ++ s.upserted ++ s.post ++ s.replayed).any (·.1 == x) || s.removed.contains x

def agree (f g : Members Val) (names : List Nat) : Bool :=
  names.all fun x => f x == g x

/-- The projected trace keys (200 and 201) use domain test/1/FaceSide
and the eight-byte Negative and Positive case names. -/
def keyWidth (key : Nat) : Nat :=
  4 + 1 + 32 + if key ≥ 200 then (4 + 2) + 4 + (8 + 2) + (8 + 2) else 0

/-- WPGL v3 kept-body bytes beyond its two counts. -/
def wireBytes (d : Delta Val) : Nat :=
  (encode (fun key => List.replicate (keyWidth key) 0)
    (fun member => List.replicate (keyWidth member.1 + 17) 0) d).length - 8

def check (s : Step) : Except String Unit := do
  let names := support s
  let prior := membersOf s.prior
  let post := membersOf s.post
  unless agree post (membersOf s.replayed) names do
    throw "GROUP_LOG replay disagrees with the installed state"
  match s.kind with
  | "kept" =>
    let delta : Delta Val := ⟨s.removed, s.upserted⟩
    let applied := applyDelta prior delta
    unless agree applied.val post names do
      throw "installed membership is not applyDelta prior delta"
    unless applied.steps == s.removed.length + s.upserted.length + 1 do
      throw "install steps exceed the delta"
    let canonical := diff prior post names
    unless canonical.removed == s.removed && canonical.upserted == s.upserted do
      throw "recorded delta is not the canonical diff"
    unless wireBytes delta == s.wire do
      throw s!"wire bytes {s.wire} are not the model's {wireBytes delta}"
  | "born" | "revived" =>
    let initial : Delta Val := ⟨[], s.upserted⟩
    unless s.prior.isEmpty do throw "a born or revived group was live"
    unless agree (applyDelta prior initial).val post names do
      throw "installed membership is not the initial members"
    unless wireBytes initial == s.wire do
      throw s!"wire bytes {s.wire} are not the model's {wireBytes initial}"
  | "retired" =>
    unless s.post.isEmpty && s.wire == 0 do throw "a retired group kept members"
  | kind => throw s!"unknown step kind {kind}"

def main (args : List String) : IO UInt32 := do
  let [path] := args | IO.eprintln "usage: conformance <trace.ndjson>"; return 2
  let lines := (← IO.FS.lines path).toList.filter (· ≠ "")
  let mut checked := 0
  for line in lines do
    match parseStep line >>= fun s => (check s).map fun _ => s with
    | .ok _ => checked := checked + 1
    | .error e =>
      IO.eprintln s!"conformance: step {checked} fails: {e}"
      return 1
  if checked == 0 then
    IO.eprintln "conformance: empty trace"
    return 1
  IO.println s!"conformance: {checked} steps refine the proved model"
  return 0
