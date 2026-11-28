-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/entry.rs:22-27 sha256:6cb1fef575318ae82ebd44dd64ba876716871427ee86ba96b2702df971681a40
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/entry.rs:184-186 sha256:4be772cacbea0fddd6908bdf1ad8c2db322e08006e4b1ed78f1f0371ec1bc1eb

/-!
# Timeline binary search: step bound and correctness (Phase 7B-2, RULES §4)

Models `NameEntry::state_at` of `crates/worth-relational-lineage/src/
lineage/index/entry.rs`: the predecessor of `basis` (the last entry whose
commit is at most `basis`) in one name's strictly ascending timeline of
length `t`. The Rust timeline is an `im::OrdMap` and the lookup is
`ordmap_prev`, one `range(..=basis).next_back()` B-tree descent. The model
finds the same entry by halving `[lo, lo + size)` at its midpoint: the
index of the first entry whose commit exceeds the basis, whose predecessor
is the answer. `search_eq_scan` proves that index is the linear-scan answer
(ascending order is all it needs), and `Conformance.lean` checks the Rust
answer against it on every recorded timeline and basis.
`search_steps_le_log` proves at most `log₂ t + 1` halving steps for every
timeline and basis: that is the design's cost model of one ordered
descent, not a proved property of the `im` B-tree, whose descent is
likewise logarithmic in `t`.
-/
namespace PersistentNaming.TimelineSearch

/-- `partition_point (|c| c ≤ basis)` over `a lo, …, a (lo + size - 1)`:
the index found and the halving iterations taken. -/
def search (a : Nat → Nat) (basis : Nat) (lo size : Nat) : Nat × Nat :=
  if size = 0 then (lo, 0)
  else if a (lo + size / 2) ≤ basis then
    let r := search a basis (lo + size / 2 + 1) (size - size / 2 - 1)
    (r.1, r.2 + 1)
  else
    let r := search a basis lo (size / 2)
    (r.1, r.2 + 1)
termination_by size
decreasing_by all_goals omega

/-- The linear-scan answer: the first index from `lo` whose commit exceeds
`basis`, or `lo + size`. -/
def scan (a : Nat → Nat) (basis : Nat) : Nat → Nat → Nat
  | lo, 0 => lo
  | lo, k + 1 => if a lo ≤ basis then scan a basis (lo + 1) k else lo

theorem log2_mono {m n : Nat} (h : m ≤ n) : Nat.log2 m ≤ Nat.log2 n := by
  by_cases hm : m = 0
  · subst hm; simp [Nat.log2_zero]
  · exact (Nat.le_log2 (by omega)).2 (Nat.le_trans (Nat.log2_self_le hm) h)

theorem log2_half {m n : Nat} (hm : m ≠ 0) (h : 2 * m ≤ n) : Nat.log2 m + 1 ≤ Nat.log2 n := by
  refine (Nat.le_log2 (by omega)).2 ?_
  rw [Nat.pow_succ]
  have := Nat.log2_self_le hm
  omega

theorem search_steps_aux (a : Nat → Nat) (basis : Nat) :
    ∀ n size lo, size ≤ n → (search a basis lo size).2 ≤ Nat.log2 size + 1 := by
  intro n
  induction n with
  | zero => intro size lo h; rw [search]; simp [show size = 0 by omega]
  | succ n ih =>
    intro size lo h
    rw [search]
    by_cases h0 : size = 0
    · simp [h0]
    · simp only [h0, ite_false]
      have key : ∀ s lo', 2 * s ≤ size → (search a basis lo' s).2 + 1 ≤ Nat.log2 size + 1 := by
        intro s lo' hs
        by_cases hs0 : s = 0
        · subst hs0; rw [search]; simp
        · have := ih s lo' (by omega)
          have := log2_half hs0 hs
          omega
      split
      · exact key _ _ (by omega)
      · exact key _ _ (by omega)

/-- Binary search over a timeline of length `size` takes at most
`log₂ size + 1` iterations, for every timeline and basis. -/
theorem search_steps_le_log (a : Nat → Nat) (basis lo size : Nat) :
    (search a basis lo size).2 ≤ Nat.log2 size + 1 :=
  search_steps_aux a basis size size lo (Nat.le_refl size)

theorem scan_skip (a : Nat → Nat) (basis : Nat) : ∀ j lo size,
    (∀ i, lo ≤ i → i < lo + j → a i ≤ basis) → j ≤ size →
    scan a basis lo size = scan a basis (lo + j) (size - j)
  | 0, lo, size, _, _ => by simp
  | j + 1, lo, size, h, hj => by
    obtain ⟨k, rfl⟩ : ∃ k, size = k + 1 := ⟨size - 1, by omega⟩
    simp only [scan, h lo (Nat.le_refl lo) (by omega), ↓reduceIte]
    rw [scan_skip a basis j (lo + 1) k (fun i h1 h2 => h i (by omega) (by omega)) (by omega)]
    congr 1 <;> omega

theorem scan_stop (a : Nat → Nat) (basis : Nat) : ∀ h lo size,
    basis < a (lo + h) → h < size → scan a basis lo size = scan a basis lo h
  | 0, lo, size, hb, hs => by
    obtain ⟨k, rfl⟩ : ∃ k, size = k + 1 := ⟨size - 1, by omega⟩
    simp at hb
    simp [scan, Nat.not_le.2 hb]
  | h + 1, lo, size, hb, hs => by
    obtain ⟨k, rfl⟩ : ∃ k, size = k + 1 := ⟨size - 1, by omega⟩
    simp only [scan]
    rw [scan_stop a basis h (lo + 1) k
      (by rw [show lo + 1 + h = lo + (h + 1) by omega]; exact hb) (by omega)]

theorem search_eq_scan_aux (a : Nat → Nat) (basis : Nat) :
    ∀ n size lo, size ≤ n → (∀ i j, lo ≤ i → i ≤ j → j < lo + size → a i ≤ a j) →
      (search a basis lo size).1 = scan a basis lo size := by
  intro n
  induction n with
  | zero => intro size lo h _; rw [search]; simp [show size = 0 by omega, scan]
  | succ n ih =>
    intro size lo h hm
    rw [search]
    by_cases h0 : size = 0
    · subst h0; simp [scan]
    · simp only [h0, ite_false]
      split
      · rename_i hle
        show (search a basis (lo + size / 2 + 1) (size - size / 2 - 1)).1 = _
        rw [ih _ _ (by omega) (fun i j h1 h2 _ => hm i j (by omega) h2 (by omega))]
        rw [scan_skip a basis (size / 2 + 1) lo size
          (fun i h1 h2 => Nat.le_trans (hm i (lo + size / 2) h1 (by omega) (by omega)) hle)
          (by omega)]
        congr 1 <;> omega
      · show (search a basis lo (size / 2)).1 = _
        rw [ih _ _ (by omega) (fun i j h1 h2 _ => hm i j h1 h2 (by omega))]
        rw [scan_stop a basis (size / 2) lo size (by omega) (by omega)]

/-- On an ascending timeline the binary search returns the linear-scan
answer: the index of the first entry whose commit exceeds the basis. -/
theorem search_eq_scan (a : Nat → Nat) (basis lo size : Nat)
    (hsorted : ∀ i j, lo ≤ i → i ≤ j → j < lo + size → a i ≤ a j) :
    (search a basis lo size).1 = scan a basis lo size :=
  search_eq_scan_aux a basis size size lo (Nat.le_refl size) hsorted

end PersistentNaming.TimelineSearch
