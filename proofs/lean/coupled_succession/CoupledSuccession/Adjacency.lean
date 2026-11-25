/-!
# Adjacency retraction (design §2.4 `AdjacencySet::retract_delta_for_request`; manifest T13)

One `(anchor, kind, direction)` triple: the ordered relation sets `current` and `historical`
(kept strictly sorted, like the production ordered sets) and `structural_revision_by_kind`.
-/
namespace CoupledSuccession

inductive AdjKind
  | created
  | deleted

/-- One adjacency delta, stamped with the proposed version. -/
structure AdjDelta where
  kind : AdjKind
  relation : Nat
  version : Nat

structure AdjState where
  current : List Nat
  historical : List Nat
  revision : Option Nat

/-- Ordered-set insertion. -/
def insertSorted (r : Nat) : List Nat → List Nat
  | [] => [r]
  | x :: xs => if r ≤ x then r :: x :: xs else x :: insertSorted r xs

def StrictSorted (l : List Nat) : Prop := List.Pairwise (· < ·) l

/-- Ordered-set insertion that keeps a present member (production sets are idempotent). -/
def insertSet (r : Nat) (l : List Nat) : List Nat := if r ∈ l then l else insertSorted r l

/-- Apply a delta: `Created` enters `current` and `historical` (an endpoint update re-creates a
relation the anchor may already hold historically); `Deleted` leaves `current` only. -/
def AdjState.apply (a : AdjState) (d : AdjDelta) : AdjState :=
  match d.kind with
  | .created =>
    { current := insertSet d.relation a.current
      historical := insertSet d.relation a.historical
      revision := some d.version }
  | .deleted =>
    { current := a.current.erase d.relation
      historical := a.historical
      revision := some d.version }

/-- Invert a delta. The revision returns to `B`'s value `rev0` iff no other unmasked member
after the head touched this triple (`otherTouch = false`); otherwise it keeps its stamp. A created
relation leaves `historical` only when `B` did not already hold it there (`historicalBefore`,
production's `historical_before`, read from the head image `B`). -/
def AdjState.retract (a : AdjState) (d : AdjDelta) (rev0 : Option Nat) (historicalBefore : Bool)
    (otherTouch : Bool) : AdjState :=
  let rev := if otherTouch then a.revision else rev0
  match d.kind with
  | .created =>
    { current := a.current.erase d.relation
      historical := if historicalBefore then a.historical else a.historical.erase d.relation
      revision := rev }
  | .deleted =>
    { current := insertSorted d.relation a.current
      historical := a.historical
      revision := rev }

/-- Well-formedness of a delta against the state it applies to (review note 4). -/
def AdjDelta.Fits (d : AdjDelta) (a : AdjState) : Prop :=
  match d.kind with
  | .created => d.relation ∉ a.current ∧ d.relation ∉ a.historical
  | .deleted => d.relation ∈ a.current ∧ d.relation ∈ a.historical

theorem erase_insertSorted (r : Nat) : ∀ l : List Nat, r ∉ l → (insertSorted r l).erase r = l := by
  intro l
  induction l with
  | nil => intro _; simp [insertSorted]
  | cons x xs ih =>
    intro h
    have hx : r ≠ x := fun e => h (e ▸ List.mem_cons_self)
    have hxs : r ∉ xs := fun m => h (List.mem_cons_of_mem _ m)
    unfold insertSorted
    by_cases hle : r ≤ x
    · simp [hle]
    · simp only [hle, ite_false]
      rw [List.erase_cons_tail (by simpa using fun e => hx e.symm), ih hxs]

theorem insertSorted_of_lt (r : Nat) (l : List Nat) (h : ∀ y ∈ l, r < y) :
    insertSorted r l = r :: l := by
  cases l with
  | nil => rfl
  | cons x xs =>
    have : r ≤ x := Nat.le_of_lt (h x List.mem_cons_self)
    simp [insertSorted, this]

theorem insertSorted_erase (r : Nat) : ∀ l : List Nat, StrictSorted l → r ∈ l →
    insertSorted r (l.erase r) = l := by
  intro l
  induction l with
  | nil => intro _ h; cases h
  | cons x xs ih =>
    intro hs hm
    unfold StrictSorted at hs
    rw [List.pairwise_cons] at hs
    obtain ⟨hlt, hs⟩ := hs
    by_cases hx : x = r
    · subst hx
      rw [List.erase_cons_head]
      exact insertSorted_of_lt x xs hlt
    · have hm' : r ∈ xs := by
        rcases List.mem_cons.mp hm with h | h
        · exact absurd h.symm hx
        · exact h
      have hxr : x < r := hlt r hm'
      rw [List.erase_cons_tail (by simpa using hx)]
      unfold insertSorted
      have : ¬ r ≤ x := Nat.not_le_of_lt hxr
      simp only [this, ite_false]
      rw [ih hs hm']

theorem not_mem_erase_of_sorted (r : Nat) : ∀ l : List Nat, StrictSorted l → r ∉ l.erase r := by
  intro l
  induction l with
  | nil => intro _; simp
  | cons x xs ih =>
    intro hs
    unfold StrictSorted at hs
    rw [List.pairwise_cons] at hs
    obtain ⟨hlt, hs⟩ := hs
    by_cases hx : x = r
    · subst hx
      rw [List.erase_cons_head]
      exact fun m => Nat.lt_irrefl x (hlt x m)
    · rw [List.erase_cons_tail (by simpa using hx)]
      intro m
      rcases List.mem_cons.mp m with h | h
      · exact hx h.symm
      · exact ih hs h

namespace Adjacency

/-- T13: retracting a delta restores the triple exactly; the revision is restored iff no other
unmasked member touched it. `historicalBefore` is whether `B` (here `a`) held the relation in its
historical bucket, as production reads it from the head image.

Changes from design §5.2 (review note 4): `rev0 : Option Nat` instead of `Nat` (`B` may have no
revision for the triple, and `rev0 = a.revision` only typechecks as `Option Nat`); added the
well-formedness premises `rev0 = a.revision`, `StrictSorted a.current` (ordered sets: re-inserting
a deleted relation must land in its old place) and `d.Fits a` (`Created r`: `r ∉ current ∧
r ∉ historical`; `Deleted r`: `r ∈ current ∧ r ∈ historical`). -/
theorem invert_adjacency_exact :
    ∀ (a : AdjState) (d : AdjDelta) (rev0 : Option Nat), rev0 = a.revision →
      StrictSorted a.current → d.Fits a →
      (a.apply d).retract d rev0 (decide (d.relation ∈ a.historical)) (otherTouch := false) = a ∧
        ((a.apply d).retract d rev0 (decide (d.relation ∈ a.historical))
          (otherTouch := true)).revision = (a.apply d).revision := by
  intro a d rev0 hrev hsort hfit
  obtain ⟨cur, hist, rev⟩ := a
  obtain ⟨kind, r, v⟩ := d
  cases kind with
  | created =>
    simp only [AdjDelta.Fits] at hfit
    simp only [AdjState.apply, AdjState.retract, insertSet, hfit.1, hfit.2, decide_false,
      Bool.false_eq_true, ite_false, ite_true]
    refine ⟨?_, trivial⟩
    rw [erase_insertSorted r cur hfit.1, erase_insertSorted r hist hfit.2, hrev]
  | deleted =>
    simp only [AdjDelta.Fits] at hfit
    simp only [AdjState.apply, AdjState.retract, Bool.false_eq_true, ite_false, ite_true]
    refine ⟨?_, trivial⟩
    rw [insertSorted_erase r cur hsort hfit.1, hrev]

/-- T13u (Neon code review r1, blocker 1): an endpoint update on an anchor that keeps the relation
(`Deleted r` then `Created r`, as `UpdateEndpoints` emits on a shared endpoint), retracted in
reverse order with `historicalBefore` read from `B`, restores the triple exactly: the relation
stays in `historical`, which already held it before the chain. Pruning it unconditionally would
leave `r ∈ current` with `r ∉ historical`. -/
theorem invert_endpoint_move_exact :
    ∀ (a : AdjState) (r v : Nat) (rev0 : Option Nat), rev0 = a.revision →
      StrictSorted a.current → r ∈ a.current → r ∈ a.historical →
      let deleted : AdjDelta := ⟨.deleted, r, v⟩
      let created : AdjDelta := ⟨.created, r, v⟩
      (((a.apply deleted).apply created).retract created rev0 (decide (r ∈ a.historical))
          (otherTouch := false)).retract deleted rev0 (decide (r ∈ a.historical))
          (otherTouch := false) = a := by
  intro a r v rev0 hrev hsort hcur hhist
  obtain ⟨cur, hist, rev⟩ := a
  have hout : r ∉ cur.erase r := not_mem_erase_of_sorted r cur hsort
  simp only at hcur hhist hrev
  simp only [AdjState.apply, AdjState.retract, insertSet, hout, hhist, decide_true,
    Bool.false_eq_true, ite_false, ite_true]
  rw [erase_insertSorted r (cur.erase r) hout, insertSorted_erase r cur hsort hcur, hrev]

end Adjacency
end CoupledSuccession
