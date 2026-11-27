/-!
# Relational symbol dictionary: model (public #446)

The pure transition of the Relational symbol dictionary
(`crates/worth-relational-contracts/src/symbols/data/string_interner.rs`
and the candidate reservations of
`crates/worth-relational-storage/src/runtime/state/subsystems/services/
symbol_table/reservations.rs`).

A dictionary maps symbols to texts and texts to symbols, with an identity
frontier. Texts are opaque values. Each persistent map answers point
lookups and takes point writes; the model counts one step per lookup and
one per write. That is the design's cost model, not a proved property of a
concrete persistent structure: the Rust `im` maps pay an `O(log N)` path
copy per write, and since `StoredSymbolText` clones share their bytes, that
path copy copies node pointers and handles, never an existing key's text.
-/
namespace RelationalSymbolCommit

/-- A symbol dictionary: `by_symbol`, `by_value` and `next_symbol`. -/
structure Dict where
  bySym : Nat → Option Nat
  byVal : Nat → Option Nat
  next : Nat

/-- The empty dictionary; symbol 0 is never issued. -/
def empty : Dict := { bySym := fun _ => none, byVal := fun _ => none, next := 1 }

/-- The two maps are inverse (one meaning per symbol and per text) and the
frontier is above every installed symbol. -/
def Wf (d : Dict) : Prop :=
  (∀ s t, d.bySym s = some t ↔ d.byVal t = some s) ∧
    ∀ s t, d.bySym s = some t → s < d.next

/-- Installing `(s, t)` would contradict an existing meaning. -/
def aliases (d : Dict) (s t : Nat) : Bool :=
  (d.bySym s).any (· != t) || (d.byVal t).any (· != s)

/-- The two map writes and the frontier raise of an install. -/
def write (d : Dict) (s t : Nat) : Dict :=
  { bySym := fun x => if x = s then some t else d.bySym x,
    byVal := fun y => if y = t then some s else d.byVal y,
    next := max d.next (s + 1) }

/-- `import_canonical_meaning`: two lookups, then a refusal (an alias, or
the exhausted symbol `cap`) or the three writes (`by_value` remove and
insert, `by_symbol` insert). -/
def importMeaning (cap : Nat) (d : Dict) (s t : Nat) : Option Dict × Nat :=
  if aliases d s t || s == cap then (none, 2) else (some (write d s t), 5)

/-- `import_canonical_snapshot`: each meaning in order over a staged copy;
the first refusal refuses the whole batch. -/
def importAll (cap : Nat) : Dict → List (Nat × Nat) → Option Dict × Nat
  | d, [] => (some d, 0)
  | d, (s, t) :: es =>
    match importMeaning cap d s t with
    | (none, k) => (none, k)
    | (some d', k) => ((importAll cap d' es).1, k + (importAll cap d' es).2)

/-- A full-snapshot import: the whole canonical dictionary `snapshot`
rebuilt from empty. -/
def importFull (cap : Nat) (snapshot : List (Nat × Nat)) : Option Dict × Nat :=
  importAll cap empty snapshot

/-- `PendingSymbolMeanings::reserve` of text `t`: a candidate lookup; on a
miss an owner lookup, then either the owner's meaning or a fresh identity
spent from the owner frontier (refused at `cap`), imported into the
candidate. Returns the owner, the candidate and the symbol. -/
def reserve (cap : Nat) (owner cand : Dict) (t : Nat) :
    Option (Dict × Dict × Nat) × Nat :=
  match cand.byVal t with
  | some s => (some (owner, cand, s), 1)
  | none =>
    match owner.byVal t with
    | some s =>
      ((importMeaning cap cand s t).1.map (fun c => (owner, c, s)),
        2 + (importMeaning cap cand s t).2)
    | none =>
      if owner.next = cap then (none, 3)
      else
        ((importMeaning cap cand owner.next t).1.map
            (fun c => ({ owner with next := owner.next + 1 }, c, owner.next)),
          3 + (importMeaning cap cand owner.next t).2)

/-- Reserving every touched text in order; returns the owner, the
candidate and the reserved meanings. -/
def reserveAll (cap : Nat) :
    Dict → Dict → List Nat → Option (Dict × Dict × List (Nat × Nat)) × Nat
  | owner, cand, [] => (some (owner, cand, []), 0)
  | owner, cand, t :: ts =>
    match reserve cap owner cand t with
    | (none, k) => (none, k)
    | (some (owner', cand', s), k) =>
      ((reserveAll cap owner' cand' ts).1.map
          (fun r => (r.1, r.2.1, (s, t) :: r.2.2)),
        k + (reserveAll cap owner' cand' ts).2)

/-- One commit's dictionary work over its touched texts: reservation into
the candidate, then the cutover import of the reserved meanings into the
owner (`PreparedSymbolPublication::publish` under `compare_and_move`). -/
def commit (cap : Nat) (owner : Dict) (touched : List Nat) : Option Dict × Nat :=
  match reserveAll cap owner owner touched with
  | (none, k) => (none, k)
  | (some (owner', _, meanings), k) =>
    ((importAll cap owner' meanings).1, k + (importAll cap owner' meanings).2)

end RelationalSymbolCommit
