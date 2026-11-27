import Lean.Data.Json
import RelationalSymbolCommit.Dictionary

/-!
# Executable Rust ↔ model conformance

Reads the NDJSON trace the real Rust symbol dictionary emits (the
`symbol_commit_conformance_trace` court of `crates/worth-relational`, with
`WORTH_SYMBOL_COMMIT_TRACE=<path>`): one line per transition, carrying each
dictionary before and after as its `by_symbol` and `by_value` rows and its
`next_symbol` frontier. Every step is checked against the proved functions
of `RelationalSymbolCommit.Dictionary` at the Rust symbol ceiling
`u32::MAX`:

* every recorded dictionary is well formed: its two maps are inverse (one
  meaning per symbol and per text) and its frontier is above every symbol;
* `import_meaning` and `import_snapshot` steps install exactly what
  `importMeaning` and `importAll` install, and refuse exactly when they
  refuse, leaving the dictionary unchanged;
* `reserve` steps return `reserve`'s symbol, owner and candidate, and refuse
  exactly when it refuses;
* `commit` steps leave the owner exactly as `commit` does, and refuse
  exactly when it refuses;
* step indices run `0, 1, …` with none missing.

Exits 0 iff every step refines the model. A checker, not a model: it
carries no `proof-mirror:` pin.
-/
open Lean RelationalSymbolCommit

/-- Rust's exhausted symbol, `u32::MAX`. -/
def cap : Nat := 4294967295

/-- A dictionary as the trace records it. -/
structure Rows where
  bySym : List (Nat × Nat)
  byVal : List (Nat × Nat)
  next : Nat

def Rows.dict (r : Rows) : Dict :=
  { bySym := fun s => (r.bySym.find? (·.1 = s)).map (·.2),
    byVal := fun t => (r.byVal.find? (·.1 = t)).map (·.2),
    next := r.next }

def pair (j : Json) : Except String (Nat × Nat) := do
  match (← j.getArr?).toList with
  | [a, b] => pure (← a.getNat?, ← b.getNat?)
  | _ => throw s!"row is not a pair: {j.compress}"

def pairs (j : Json) : Except String (List (Nat × Nat)) := do
  (← j.getArr?).toList.mapM pair

def rows (j : Json) (key : String) : Except String Rows := do
  let d ← j.getObjVal? key
  pure { bySym := ← pairs (← d.getObjVal? "by_symbol"),
         byVal := ← pairs (← d.getObjVal? "by_value"),
         next := ← (← d.getObjVal? "next").getNat? }

/-- The two maps are inverse and the frontier is above every symbol. -/
def wellFormed (r : Rows) : Except String Unit := do
  unless r.bySym.length == r.byVal.length &&
      r.bySym.all (fun (s, t) => r.byVal.contains (t, s)) &&
      r.byVal.all (fun (t, s) => r.bySym.contains (s, t)) do
    throw "by_symbol and by_value are not inverse"
  unless r.bySym.all (·.1 < r.next) do throw "a symbol is at or above the frontier"
  unless (r.bySym.map (·.1)).Nodup && (r.byVal.map (·.1)).Nodup do
    throw "a symbol or text has two meanings"

/-- Every symbol and text either side mentions, plus the step's own. -/
def support (rs : List Rows) (extra : List Nat) : List Nat :=
  extra ++ rs.flatMap fun r => r.bySym.map (·.1) ++ r.byVal.map (·.1)

/-- `d` and the recorded dictionary agree on every mentioned key. -/
def agrees (d : Dict) (r : Rows) (keys : List Nat) : Bool :=
  d.next == r.next && keys.all fun k => d.bySym k == r.dict.bySym k && d.byVal k == r.dict.byVal k

def expect (ok : Bool) (what : String) : Except String Unit :=
  unless ok do throw what

def outcome (model : Option Dict) (ok : Bool) (before after : Rows) (keys : List Nat)
    (name : String) : Except String Unit :=
  match model with
  | none => do
    expect (!ok) s!"Rust installed what {name} refuses"
    expect (agrees before.dict after keys) s!"a refused {name} changed the dictionary"
  | some d => do
    expect ok s!"Rust refused what {name} installs"
    expect (agrees d after keys) s!"the dictionary after {name} is not the model's"

def check (j : Json) : Except String Unit := do
  let kind ← (← j.getObjVal? "kind").getStr?
  match kind with
  | "import_meaning" | "import_snapshot" =>
    let (before, after) := (← rows j "dict", ← rows j "after")
    wellFormed before; wellFormed after
    let ok ← (← j.getObjVal? "ok").getBool?
    let es ← if kind == "import_meaning" then
        pure [(← (← j.getObjVal? "symbol").getNat?, ← (← j.getObjVal? "text").getNat?)]
      else pairs (← j.getObjVal? "entries")
    let keys := support [before, after] (es.flatMap fun (s, t) => [s, t])
    let model := if kind == "import_meaning" then
        (importMeaning cap before.dict es.head!.1 es.head!.2).1
      else (importAll cap before.dict es).1
    outcome model ok before after keys kind
  | "reserve" =>
    let (owner, cand) := (← rows j "owner", ← rows j "cand")
    let (owner', cand') := (← rows j "owner_after", ← rows j "cand_after")
    for r in [owner, cand, owner', cand'] do wellFormed r
    let t ← (← j.getObjVal? "text").getNat?
    let symbol := (j.getObjValD "symbol").getNat?.toOption
    let keys := support [owner, cand, owner', cand'] (t :: symbol.toList)
    match (reserve cap owner.dict cand.dict t).1, symbol with
    | none, none => pure ()
    | none, some s => throw s!"Rust reserved {s} where the model refuses"
    | some _, none => throw "Rust refused a reservation the model makes"
    | some (o, c, s), some s' => do
      expect (s == s') s!"Rust reserved symbol {s'}, the model {s}"
      expect (agrees o owner' keys) "the owner after reserve is not the model's"
      expect (agrees c cand' keys) "the candidate after reserve is not the model's"
  | "commit" =>
    let (before, after) := (← rows j "owner", ← rows j "after")
    wellFormed before; wellFormed after
    let touched ← (← (← j.getObjVal? "touched").getArr?).toList.mapM (·.getNat?)
    let ok ← (← j.getObjVal? "ok").getBool?
    let keys := support [before, after] touched
    match (commit cap before.dict touched).1 with
    | none => expect (!ok) "Rust published a commit the model refuses"
    | some d => do
      expect ok "Rust refused a commit the model publishes"
      expect (agrees d after keys) "the owner after the commit is not the model's"
  | other => throw s!"unknown step kind {other}"

def main (args : List String) : IO UInt32 := do
  let [path] := args | IO.eprintln "usage: conformance <trace.ndjson>"; return 2
  let lines := (← IO.FS.lines path).toList.filter (· ≠ "")
  let mut checked := 0
  for line in lines do
    let result : Except String Unit := do
      let j ← Json.parse line
      let index ← (← j.getObjVal? "step").getNat?
      unless index == checked do throw s!"step {index} is out of sequence"
      check j
    match result with
    | .ok () => checked := checked + 1
    | .error e =>
      IO.eprintln s!"conformance: step {checked} fails: {e}"
      return 1
  if checked == 0 then
    IO.eprintln "conformance: empty trace"
    return 1
  IO.println s!"conformance: {checked} steps refine the proved model"
  return 0
