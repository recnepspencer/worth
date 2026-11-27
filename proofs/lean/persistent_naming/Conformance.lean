import Lean.Data.Json
import PersistentNaming

/-!
# Executable Rust ↔ model conformance

Reads the NDJSON traces the real Rust emits with
`WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>`: `mint_order.ndjson` and
`name_index.ndjson` from `crates/worth-relational-lineage`, and
`wire_bound.ndjson` from `workspaces/worth-query`'s
`worth-query-declaration`. Every line is checked against the proved
definitions:

* `mint_order`: `dependency_order`'s minting order over a graph of births in
  canonical order is `MintOrder.dependencyOrder`'s: the Kahn loop popping
  the least ready birth, then the births left on a cycle;
* `index_batch`: a `PersistentNameBranchIndex::apply` batch leaves every
  touched name's timeline as `IndexDelta.batch` does over the batch's
  timeline operations, and refuses exactly when `applyAll` refuses (the
  refused batch's rollback restoring every timeline);
* `carriers`: an installed carrier set is `IndexDelta.carrierUpdate` of the
  prior set by the departed and joined members, and equals the declared set;
* `state_at`: `NameEntry::state_at` is `NameResolution.stateAt`, the
  `TimelineSearch.search` predecessor over the timeline;
* `resolve`: `PersistentNameBranchIndex::resolve`'s index probes, frontier
  pops, edge visits and live descendants are those of
  `NameResolution.resolve` over the recorded index;
* `wire`: an encoded discriminator value's length is `WireBound.encLen`,
  its type's `MAX_ENCODED_BYTES` is `WireBound.bound`, and a whole
  discriminator's length and bound are `discLen` and `fieldsBound`.

Exits 0 iff every line of every file refines the model. A checker, not a
model: it carries no `proof-mirror:` pin.
-/
open Lean PersistentNaming

def nat (j : Json) (key : String) : Except String Nat := do (← j.getObjVal? key).getNat?

def nats (j : Json) : Except String (List Nat) := do (← j.getArr?).toList.mapM (·.getNat?)

def natsAt (j : Json) (key : String) : Except String (List Nat) := do nats (← j.getObjVal? key)

def pair (j : Json) : Except String (Nat × Nat) := do
  match (← j.getArr?).toList with
  | [a, b] => pure (← a.getNat?, ← b.getNat?)
  | _ => throw s!"not a pair: {j.compress}"

def expect (ok : Bool) (what : String) : Except String Unit := unless ok do throw what

def checkMintOrder (j : Json) : Except String Unit := do
  let n ← nat j "births"
  let deps ← (← (← j.getObjVal? "deps").getArr?).toList.mapM nats
  let order ← natsAt j "order"
  let model := MintOrder.dependencyOrder n fun p => deps.getD p []
  expect (model == some order) s!"Rust minted {order}, the model {model}"

/-- A recorded timeline, `null` for an absent name. -/
def timeline (j : Json) : Except String (Option IndexDelta.Timeline) :=
  if j.isNull then pure none else do pure (some (← (← j.getArr?).toList.mapM pair))

def store (j : Json) (key : String) : Except String (List (Nat × Option IndexDelta.Timeline)) := do
  (← (← j.getObjVal? key).getArr?).toList.mapM fun row => do
    match (← row.getArr?).toList with
    | [k, t] => pure (← k.getNat?, ← timeline t)
    | _ => throw s!"not a [name, timeline] row: {row.compress}"

def op (j : Json) : Except String IndexDelta.Op := do
  match (← j.getArr?).toList with
  | [kind, k, x] =>
    match ← kind.getStr? with
    | "set" => pure (.set (← k.getNat?) (← x.getNat?))
    | "insert" => pure (.insert (← k.getNat?) (← x.getNat?))
    | other => throw s!"unknown operation {other}"
  | _ => throw s!"not an operation: {j.compress}"

def checkBatch (j : Json) : Except String Unit := do
  let c ← nat j "commit"
  let ops ← (← (← j.getObjVal? "ops").getArr?).toList.mapM op
  let refused ← (← j.getObjVal? "refused").getBool?
  let prior ← store j "prior"
  let post ← store j "post"
  let s : IndexDelta.Store := fun k => ((prior.find? (·.1 = k)).map (·.2)).join
  expect (prior.all fun (_, t) => t.all fun t => t.all (·.1 < c))
    "a touched timeline already holds the batch commit"
  let run := IndexDelta.applyAll c ops (IndexDelta.init s)
  expect (run.refused == refused) s!"Rust refused={refused}, the model {run.refused}"
  let installed := (IndexDelta.batch c ops s).1
  for (k, t) in post do
    expect (installed k == t) s!"name {k}'s timeline {t} is not the model's {installed k}"

def checkCarriers (j : Json) : Except String Unit := do
  let (prior, declared) := (← natsAt j "prior", ← natsAt j "declared")
  let (left, joined, post) := (← natsAt j "left", ← natsAt j "joined", ← natsAt j "post")
  let m : IndexDelta.Carriers := fun y => prior.contains y
  let updated := (IndexDelta.carrierUpdate m joined left).1
  let support := prior ++ declared ++ left ++ joined ++ post
  expect (support.all fun y => updated y == post.contains y)
    s!"installed carriers {post} are not the model's update of {prior}"
  expect (support.all fun y => declared.contains y == post.contains y)
    s!"installed carriers {post} are not the declared {declared}"

/-- A name's recorded entry as the resolution model reads it. -/
def entry (states : List (Nat × Nat)) (edges : List (Nat × List Nat)) : NameResolution.Entry :=
  { commits := fun i => (states.getD i (0, 0)).1, len := states.length,
    ended := fun i => (states.getD i (0, 1)).2 == 0,
    edgeCommits := fun i => (edges.getD i (0, [])).1, edgeLen := edges.length,
    edgeLists := fun i => (edges.getD i (0, [])).2 }

def found (f : NameResolution.Found) : Option (Nat × Nat) :=
  match f with
  | .absent => none
  | .live => some (0, 1)
  | .ended since => some (since, 0)

def checkStateAt (j : Json) : Except String Unit := do
  let states ← (← (← j.getObjVal? "timeline").getArr?).toList.mapM pair
  let basis ← nat j "basis"
  let rust ← do
    let f ← j.getObjVal? "found"
    if f.isNull then pure none else some <$> pair f
  -- A live state's commit is not part of the model's answer.
  let rust := rust.map fun (since, live) => if live == 1 then (0, 1) else (since, 0)
  let model := found (NameResolution.stateAt (entry states []) basis).1
  expect (rust == model) s!"state_at {basis} is {rust}, the model's {model}"

def edgeRow (j : Json) : Except String (Nat × List Nat) := do
  match (← j.getArr?).toList with
  | [c, to] => pure (← c.getNat?, ← nats to)
  | _ => throw s!"not an edge row: {j.compress}"

def checkResolve (j : Json) : Except String Unit := do
  let rows ← (← (← j.getObjVal? "names").getArr?).toList.mapM fun row => do
    match (← row.getArr?).toList with
    | [k, states, edges] =>
      pure (← k.getNat?, entry (← (← states.getArr?).toList.mapM pair)
        (← (← edges.getArr?).toList.mapM edgeRow))
    | _ => throw s!"not a name row: {row.compress}"
  let idx : Nat → Option NameResolution.Entry := fun k => (rows.find? (·.1 = k)).map (·.2)
  let (held, basis) := (← nat j "held", ← nat j "basis")
  let w := NameResolution.resolve idx basis held (rows.length + 1)
  expect w.queue.isEmpty "the model's walk did not finish"
  let events := (w.visited.filter fun s => (NameResolution.probe idx basis s).2).length
  let live := (w.visited.filter fun s => match idx s with
    | some e => match (NameResolution.stateAt e basis).1 with | .live => true | _ => false
    | none => false).mergeSort
  let rust := (← nat j "probes", ← nat j "events", ← nat j "edges", ← natsAt j "live")
  let model := (w.probes, events, w.edges, live)
  expect (rust == model)
    s!"resolve {held}@{basis}: Rust (probes, pops, edges, live) {rust}, the model {model}"

partial def ty (j : Json) : Except String WireBound.Ty :=
  match j.getStr? with
  | .ok "bool" => pure .bool
  | .ok "u64" => pure .u64
  | .ok "name" => pure .name
  | .ok "text" => pure .text
  | .ok other => throw s!"unknown type {other}"
  | .error _ => do
    match j.getObjVal? "opt", j.getObjVal? "pair" with
    | .ok t, _ => pure (.opt (← ty t))
    | _, .ok p =>
      match (← p.getArr?).toList with
      | [a, b] => pure (.pair (← ty a) (← ty b))
      | _ => throw s!"not a pair type: {j.compress}"
    | _, _ => throw s!"unknown type {j.compress}"

partial def val (j : Json) : Except String WireBound.Val :=
  match j.getStr? with
  | .ok "name" => pure .name
  | .ok "none" => pure .none
  | .ok other => throw s!"unknown value {other}"
  | .error _ => do
    if let .ok b := j.getObjValAs? Bool "bool" then return .bool b
    if let .ok n := j.getObjValAs? Nat "u64" then return .u64 n
    if let .ok n := j.getObjValAs? Nat "text" then return .text n
    if let .ok v := j.getObjVal? "some" then return .some (← val v)
    match (← (← j.getObjVal? "pair").getArr?).toList with
    | [a, b] => pure (.pair (← val a) (← val b))
    | _ => throw s!"unknown value {j.compress}"

/-- The model's bound beside Rust's `Option<usize>` (`null` when unbounded). -/
def bound (j : Json) : Except String (Option Nat) := do
  let b ← j.getObjVal? "bound"
  if b.isNull then pure none else some <$> b.getNat?

def checkWire (j : Json) : Except String Unit := do
  let (t, v) := (← ty (← j.getObjVal? "ty"), ← val (← j.getObjVal? "value"))
  let encoded ← nat j "encoded"
  expect (encoded == WireBound.encLen v) s!"Rust encoded {encoded} bytes, the model {WireBound.encLen v}"
  expect ((← bound j) == WireBound.bound t)
    s!"Rust's MAX_ENCODED_BYTES is not the model's {WireBound.bound t}"

def checkDisc (j : Json) : Except String Unit := do
  let fields ← (← (← j.getObjVal? "fields").getArr?).toList.mapM fun f => do
    match (← f.getArr?).toList with
    | [n, t] => pure (← n.getNat?, ← ty t)
    | _ => throw s!"not a field: {f.compress}"
  let values ← (← (← j.getObjVal? "values").getArr?).toList.mapM val
  let d : WireBound.Disc := ⟨← nat j "identity", fields⟩
  let encoded ← nat j "encoded"
  expect (encoded == WireBound.discLen d values)
    s!"Rust encoded {encoded} bytes, the model {WireBound.discLen d values}"
  let model := (WireBound.fieldsBound fields).map (WireBound.domainTagLen + 8 + d.identityLen + ·)
  expect ((← bound j) == model) s!"Rust's max_encoded_bytes is not the model's {model}"

def check (j : Json) : Except String Unit := do
  match ← (← j.getObjVal? "kind").getStr? with
  | "mint_order" => checkMintOrder j
  | "index_batch" => checkBatch j
  | "carriers" => checkCarriers j
  | "state_at" => checkStateAt j
  | "resolve" => checkResolve j
  | "wire" => checkWire j
  | "disc" => checkDisc j
  | other => throw s!"unknown line kind {other}"

def main (args : List String) : IO UInt32 := do
  if args.isEmpty then
    IO.eprintln "usage: conformance <trace.ndjson>..."
    return 2
  let mut checked := 0
  for path in args do
    let lines := (← IO.FS.lines path).toList.filter (· ≠ "")
    if lines.isEmpty then
      IO.eprintln s!"conformance: {path} is empty"
      return 1
    for (line, row) in lines.zipIdx do
      match Json.parse line >>= check with
      | .ok () => checked := checked + 1
      | .error e =>
        IO.eprintln s!"conformance: {path} line {row + 1} fails: {e}"
        return 1
  IO.println s!"conformance: {checked} lines refine the proved model"
  return 0
