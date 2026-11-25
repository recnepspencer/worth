/-!
# The chain ledger `L_c` (design §2.5; manifest T7, T7d, T8, T8i, T9)

`L_c` maps a created key `(is_relation, partition, kind, scoped key)` to its `RecordRef` and a
reservation state. Kept keys keep their id across rounds (also `A → B → A`); new keys take an
id no one holds (a reclaimed slot or the append frontier); ids are never reassigned to another
key inside the candidate (cancelled reservations are held, not dropped: no ABA).
-/
namespace CoupledSuccession

structure Key where
  isRelation : Bool
  partition : Nat
  kind : Nat
  scopedKey : String
  deriving DecidableEq

inductive ResState
  | bound
  | dormant
  | cancelled
  deriving DecidableEq

structure LedgerEntry where
  key : Key
  id : Nat
  state : ResState

structure Ledger where
  entries : List LedgerEntry
  /-- The append frontier: every id the ledger (or the store) holds lies below it. -/
  frontier : Nat

inductive LedgerOp
  | withdraw (k : Key)
  | recreate (k : Key)
  | fresh (used : Nat → Bool) (k : Key)
  | complete (keep : Key → Bool)

namespace Ledger

def empty : Ledger := ⟨[], 0⟩

/-- An entry for `k` that is still reserved (Bound or Dormant). -/
def isLive (k : Key) (e : LedgerEntry) : Bool := decide (e.key = k) && decide (e.state ≠ .cancelled)

def lookup (l : Ledger) (k : Key) : Option Nat := (l.entries.find? (isLive k)).map (·.id)

/-- Bound → Dormant at retraction (`withdraw`). -/
def withdraw (l : Ledger) (k : Key) : Ledger :=
  { l with entries := l.entries.map (fun e =>
      if decide (e.key = k) && decide (e.state = .bound) then { e with state := .dormant } else e) }

/-- Dormant → Bound when round `k` creates the key again (`prebind`). -/
def rebind (l : Ledger) (k : Key) : Ledger :=
  { l with entries := l.entries.map (fun e =>
      if decide (e.key = k) && decide (e.state = .dormant) then { e with state := .bound } else e) }

/-- Append a Bound entry. -/
def push (l : Ledger) (k : Key) (n : Nat) (frontier : Nat) : Ledger :=
  ⟨l.entries ++ [⟨k, n, .bound⟩], frontier⟩

/-- Round `k` creates key `k`: the ledger's id if it has one, else an append-frontier slot. -/
def recreate (l : Ledger) (k : Key) : Nat × Ledger :=
  match l.lookup k with
  | some id => (id, l.rebind k)
  | none => (l.frontier, l.push k l.frontier (l.frontier + 1))

def held (l : Ledger) (n : Nat) : Bool := l.entries.any (fun e => decide (e.id = n))

/-- A reclaimed slot below the frontier that nobody uses and the ledger does not hold. -/
def reclaimable (l : Ledger) (used : Nat → Bool) : Option Nat :=
  (List.range l.frontier).find? (fun n => !used n && !l.held n)

/-- Ordinary reservation for a key: prebind if known, else a reclaimed slot, else the frontier. -/
def fresh (l : Ledger) (used : Nat → Bool) (k : Key) : Nat × Ledger :=
  match l.lookup k with
  | some _ => l.recreate k
  | none =>
    match l.reclaimable used with
    | some n => (n, l.push k n l.frontier)
    | none => (l.frontier, l.push k l.frontier (l.frontier + 1))

/-- Completion: keys the final rounds keep stay Bound; everything else is cancelled (held). -/
def complete (l : Ledger) (keep : Key → Bool) : Ledger :=
  { l with entries := l.entries.map (fun e =>
      if decide (e.state = .bound) && keep e.key then e else { e with state := .cancelled }) }

def bound (l : Ledger) (k : Key) : Bool :=
  l.entries.any (fun e => decide (e.key = k) && decide (e.state = .bound))

def step (l : Ledger) : LedgerOp → Ledger
  | .withdraw k => l.withdraw k
  | .recreate k => (l.recreate k).2
  | .fresh used k => (l.fresh used k).2
  | .complete keep => l.complete keep

def run (ops : List LedgerOp) : Ledger := ops.foldl step empty

/-- T7 (M-ID): a kept key keeps its id. -/
theorem kept_key_keeps_id :
    ∀ (l : Ledger) (k : Key) (id : Nat), l.lookup k = some id → (l.recreate k).1 = id := by
  intro l k id h
  simp [recreate, h]

theorem lookup_withdraw (l : Ledger) (k : Key) : (l.withdraw k).lookup k = l.lookup k := by
  unfold lookup withdraw
  dsimp only
  rw [List.find?_map, Option.map_map]
  have hp : (isLive k ∘ fun e : LedgerEntry =>
      if decide (e.key = k) && decide (e.state = .bound) then { e with state := .dormant } else e)
      = isLive k := by
    funext e
    by_cases h1 : e.key = k <;> by_cases h2 : e.state = .bound <;> simp [isLive, h1, h2]
  rw [hp]
  congr 1
  funext e
  by_cases h1 : e.key = k <;> by_cases h2 : e.state = .bound <;> simp [h1, h2]

/-- T7d: create, omit, re-create (`A → B → A`) keeps the id. -/
theorem dormant_recreate_keeps_id :
    ∀ (l : Ledger) (k : Key) (id : Nat), l.lookup k = some id →
      ((l.withdraw k).recreate k).1 = id := by
  intro l k id h
  apply kept_key_keeps_id
  rw [lookup_withdraw]; exact h

/-- T8 (corrected: frontier premise). A new key never takes an id that is in use.
The premise states the allocator invariant that every used slot lies below the frontier;
without it the statement is false (take `used := fun _ => true`). -/
theorem fresh_id_distinct :
    ∀ (l : Ledger) (used : Nat → Bool) (k : Key), l.lookup k = none →
      (∀ n, used n = true → n < l.frontier) → used (l.fresh used k).1 = false := by
  intro l used k h hf
  unfold fresh
  rw [h]
  dsimp only
  cases hr : l.reclaimable used with
  | some n =>
    have := List.find?_some hr
    simp only [Bool.and_eq_true, Bool.not_eq_true'] at this
    exact this.1
  | none =>
    cases hu : used l.frontier
    · rfl
    · exact absurd (hf _ hu) (Nat.lt_irrefl _)

/-- T9 (N2): a key the final rounds do not keep is not bound after completion. -/
theorem cancelled_unbound :
    ∀ (l : Ledger) (keep : Key → Bool) (k : Key), keep k = false → (l.complete keep).bound k = false := by
  intro l keep k hk
  unfold bound complete
  dsimp only
  rw [List.any_eq_false]
  intro e he
  rw [List.mem_map] at he
  obtain ⟨a, _, rfl⟩ := he
  by_cases h1 : a.state = .bound <;> by_cases h2 : a.key = k
  · subst h2; simp [h1, hk]
  · cases keep a.key <;> simp [h1, h2]
  · subst h2; simp [h1]
  · simp [h1, h2]

end Ledger
end CoupledSuccession
