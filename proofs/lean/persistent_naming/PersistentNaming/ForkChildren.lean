import PersistentNaming.TimeM

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/name_indexes.rs:147-180 sha256:124b9d4e0fdf34ab4a009bbfcc74b464f2e6d92b2050b72a1e7f396355629a07
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/naming_rebuild.rs:61-88 sha256:d9e62c0d572e07d41f4e24110a976d27149e76a7322429abef1f0c8283bf5a50
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/state.rs:27-27 sha256:7d73c54afb0ba858c6ac38747a1ea187bc0d874c356850b0a2cc049567a2af16

/-!
# Forks inheriting a commit: the walk (public #461)

The model of `LineageState::forks_inheriting` over the cutoff-indexed
`fork_children`, its independence of older forks and the bookkeeping of
its two nested visits. `ForkChildrenCost` states what the whole walk finds
and what it costs; its header describes the Rust and the cost model.
-/
namespace PersistentNaming.ForkChildren

/-- One branch's forks: `(cutoff, children)` rows. -/
abbrev ByCutoff := List (Nat × List Nat)

/-- `fork_children`: each branch's rows, `[]` for a branch without forks. -/
abbrev Forks := Nat → ByCutoff

/-- The rows `by_cutoff.range(c..)` yields. -/
def inheriting (c : Nat) (l : ByCutoff) : ByCutoff := l.filter fun e => decide (c ≤ e.1)

/-- The walk: stack of `(branch, inherited bound)`, marked forks, popped
branches, yielded `(branch, cutoff)` rows, pops and scan steps. -/
structure Walk where
  stack : List (Nat × Nat)
  marked : List Nat
  popped : List (Nat × Nat)
  visited : List (Nat × Nat)
  pops : Nat
  scan : Nat

/-- `for child in children`: one step each; a new child is marked and pushed. -/
def visitChildren (bound : Nat) : List Nat → Walk → Walk
  | [], w => w
  | x :: xs, w =>
    visitChildren bound xs
      (if x ∈ w.marked then { w with scan := w.scan + 1 }
       else { w with stack := (x, bound) :: w.stack, marked := x :: w.marked,
                     scan := w.scan + 1 })

/-- `for (fork_commit, children) in by_cutoff.range(c..)`: one step a row. -/
def visitRange (c p ib : Nat) : ByCutoff → Walk → Walk
  | [], w => w
  | (fc, xs) :: rest, w =>
    visitRange c p ib rest
      (if min ib fc < c then { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 }
       else visitChildren (min ib fc) xs
         { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 })

/-- The range seek over a branch's rows: one ordered descent. -/
def seekCost (l : ByCutoff) : Nat := if l.isEmpty then 0 else Nat.log2 l.length + 1

/-- One `stack.pop()`: the walk after it and the seek it paid. -/
def pop (F : Forks) (c : Nat) (w : Walk) : TimeM Walk :=
  match w.stack with
  | [] => ⟨w, 0⟩
  | (p, ib) :: rest =>
    ⟨visitRange c p ib (inheriting c (F p))
      { w with stack := rest, popped := (p, ib) :: w.popped, pops := w.pops + 1 },
     seekCost (F p)⟩

def run (F : Forks) (c : Nat) : Nat → Walk → TimeM Walk
  | 0, w => ⟨w, 0⟩
  | n + 1, w => TimeM.bind (pop F c w) (run F c n)

/-- The committing branches pushed in order, each with bound `top`. -/
def start (top : Nat) (seeds : List Nat) : Walk :=
  { stack := (seeds.map fun s => (s, top)).reverse, marked := [], popped := [],
    visited := [], pops := 0, scan := 0 }

def forksInheriting (F : Forks) (c top : Nat) (seeds : List Nat) (fuel : Nat) : TimeM Walk :=
  run F c fuel (start top seeds)

/-- Pops, seeks, and yielded rows and children. -/
def steps (r : TimeM Walk) : Nat := r.val.pops + r.steps + r.val.scan

/-- One step per inheriting row and per child in it. -/
def rowsWork (l : ByCutoff) : Nat := (l.map fun e => 1 + e.2.length).sum

/-- The inheriting work of the popped branches. -/
def work (F : Forks) (c : Nat) (popped : List (Nat × Nat)) : Nat :=
  (popped.map fun q => rowsWork (inheriting c (F q.1))).sum

/-! ## Independence of older forks -/

theorem inheriting_older (c : Nat) (o l : ByCutoff) (h : ∀ e, e ∈ o → e.1 < c) :
    inheriting c (o ++ l) = inheriting c l := by
  induction o with
  | nil => rfl
  | cons e o ih =>
    have he := h e List.mem_cons_self
    have : ¬ c ≤ e.1 := by omega
    simp only [inheriting, List.cons_append, List.filter_cons, this, decide_false] at ih ⊢
    exact ih fun e' he' => h e' (List.mem_cons_of_mem e he')

theorem run_walk_congr (F F' : Forks) (c : Nat) (h : ∀ p, inheriting c (F p) = inheriting c (F' p)) :
    ∀ (n : Nat) (w : Walk), (run F c n w).val = (run F' c n w).val
  | 0, _ => rfl
  | n + 1, w => by
    have hp : (pop F c w).val = (pop F' c w).val := by
      unfold pop; split
      · rfl
      · simp only [h]
    simp only [run, TimeM.bind_val, hp, run_walk_congr F F' c h n]

/-- Older forks, under any branches, change nothing but the seeks. -/
theorem forks_inheriting_independent_of_older_forks (F O : Forks) (c top : Nat)
    (seeds : List Nat) (fuel : Nat) (hO : ∀ p e, e ∈ O p → e.1 < c) :
    (forksInheriting (fun p => O p ++ F p) c top seeds fuel).val =
      (forksInheriting F c top seeds fuel).val :=
  run_walk_congr _ _ c (fun p => inheriting_older c (O p) (F p) (hO p)) fuel _

/-! ## Bookkeeping of the two visits -/

section Visits
variable (F : Forks) (c top : Nat) (seeds : List Nat)

theorem visitChildren_fixed (b : Nat) : ∀ (xs : List Nat) (w : Walk),
    (visitChildren b xs w).visited = w.visited ∧ (visitChildren b xs w).popped = w.popped ∧
      (visitChildren b xs w).pops = w.pops ∧
      (visitChildren b xs w).scan = w.scan + xs.length
  | [], w => by simp [visitChildren]
  | x :: xs, w => by
    simp only [visitChildren]
    split
    · obtain ⟨h1, h2, h3, h4⟩ := visitChildren_fixed b xs { w with scan := w.scan + 1 }
      exact ⟨h1, h2, h3, by rw [h4]; simp only [List.length_cons]; omega⟩
    · obtain ⟨h1, h2, h3, h4⟩ := visitChildren_fixed b xs
        { w with stack := (x, b) :: w.stack, marked := x :: w.marked, scan := w.scan + 1 }
      exact ⟨h1, h2, h3, by rw [h4]; simp only [List.length_cons]; omega⟩

/-- Marks only grow; the stack only grows; pushes are new marks; marks stay
distinct; every child is marked. -/
theorem visitChildren_marks (b : Nat) : ∀ (xs : List Nat) (w : Walk),
    (∀ x, x ∈ w.marked → x ∈ (visitChildren b xs w).marked) ∧
    (∀ q, q ∈ w.stack → q ∈ (visitChildren b xs w).stack) ∧
    (visitChildren b xs w).stack.length + w.marked.length =
      w.stack.length + (visitChildren b xs w).marked.length ∧
    (w.marked.Nodup → (visitChildren b xs w).marked.Nodup) ∧
    (∀ x, x ∈ xs → x ∈ (visitChildren b xs w).marked) ∧
    (∀ x, x ∈ (visitChildren b xs w).marked → x ∈ w.marked ∨
      ((x, b) ∈ (visitChildren b xs w).stack ∧ x ∈ xs)) ∧
    (∀ q, q ∈ (visitChildren b xs w).stack → q ∈ w.stack ∨ (q.2 = b ∧ q.1 ∈ xs))
  | [], w => by simp [visitChildren]
  | x :: xs, w => by
    simp only [visitChildren]
    by_cases hx : x ∈ w.marked
    · simp only [hx, ite_true]
      obtain ⟨h1, h2, h3, h4, h5, h6, h7⟩ := visitChildren_marks b xs { w with scan := w.scan + 1 }
      simp only at h1 h2 h3 h4 h5 h6 h7
      refine ⟨h1, h2, h3, h4, ?_, ?_, ?_⟩
      · intro y hy
        rcases List.mem_cons.1 hy with rfl | hy
        · exact h1 _ hx
        · exact h5 y hy
      · intro y hy
        rcases h6 y hy with h | ⟨h, h'⟩
        · exact Or.inl h
        · exact Or.inr ⟨h, List.mem_cons_of_mem _ h'⟩
      · intro q hq
        rcases h7 q hq with h | ⟨h, h'⟩
        · exact Or.inl h
        · exact Or.inr ⟨h, List.mem_cons_of_mem _ h'⟩
    · simp only [hx, ite_false]
      obtain ⟨h1, h2, h3, h4, h5, h6, h7⟩ := visitChildren_marks b xs
        { w with stack := (x, b) :: w.stack, marked := x :: w.marked, scan := w.scan + 1 }
      simp only [List.length_cons] at h1 h2 h3 h4 h5 h6 h7
      refine ⟨fun y hy => h1 y (List.mem_cons_of_mem _ hy),
        fun q hq => h2 q (List.mem_cons_of_mem _ hq), by omega,
        fun hn => h4 (List.nodup_cons.2 ⟨hx, hn⟩), ?_, ?_, ?_⟩
      · intro y hy
        rcases List.mem_cons.1 hy with rfl | hy
        · exact h1 _ List.mem_cons_self
        · exact h5 y hy
      · intro y hy
        rcases h6 y hy with h | ⟨h, h'⟩
        · rcases List.mem_cons.1 h with rfl | h
          · exact Or.inr ⟨h2 _ List.mem_cons_self, List.mem_cons_self⟩
          · exact Or.inl h
        · exact Or.inr ⟨h, List.mem_cons_of_mem _ h'⟩
      · intro q hq
        rcases h7 q hq with h | ⟨h, h'⟩
        · rcases List.mem_cons.1 h with rfl | h
          · exact Or.inr ⟨rfl, List.mem_cons_self⟩
          · exact Or.inl h
        · exact Or.inr ⟨h, List.mem_cons_of_mem _ h'⟩

theorem visitRange_fixed (p ib : Nat) : ∀ (l : ByCutoff) (w : Walk),
    (visitRange c p ib l w).popped = w.popped ∧ (visitRange c p ib l w).pops = w.pops ∧
      (visitRange c p ib l w).scan ≤ w.scan + rowsWork l ∧
      ∀ e, e ∈ (visitRange c p ib l w).visited → e ∈ w.visited ∨ (e.1 = p ∧ ∃ xs, (e.2, xs) ∈ l)
  | [], w => by simp [visitRange, rowsWork]
  | (fc, xs) :: rest, w => by
    simp only [visitRange]
    split
    · obtain ⟨h1, h2, h3, h4⟩ := visitRange_fixed p ib rest
        { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 }
      try simp only at h1 h2 h3 h4
      simp only [rowsWork, List.map_cons, List.sum_cons]
      refine ⟨h1, h2, by simp only [rowsWork] at h3; omega, fun e he => ?_⟩
      rcases h4 e he with h | ⟨hp, ys, hy⟩
      · rcases List.mem_cons.1 h with rfl | h
        · exact Or.inr ⟨rfl, xs, List.mem_cons_self⟩
        · exact Or.inl h
      · exact Or.inr ⟨hp, ys, List.mem_cons_of_mem _ hy⟩
    · have hc := visitChildren_fixed (min ib fc) xs
        { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 }
      obtain ⟨h1, h2, h3, h4⟩ := visitRange_fixed p ib rest
        (visitChildren (min ib fc) xs { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 })
      simp only at hc
      simp only [rowsWork, List.map_cons, List.sum_cons]
      rw [hc.2.1] at h1; rw [hc.2.2.1] at h2; rw [hc.2.2.2] at h3
      refine ⟨h1, h2, by simp only [rowsWork] at h3; omega, fun e he => ?_⟩
      rcases h4 e he with h | ⟨hp, ys, hy⟩
      · rw [hc.1] at h
        rcases List.mem_cons.1 h with rfl | h
        · exact Or.inr ⟨rfl, xs, List.mem_cons_self⟩
        · exact Or.inl h
      · exact Or.inr ⟨hp, ys, List.mem_cons_of_mem _ hy⟩

/-- The walk's mark invariants carried through one range. -/
theorem visitRange_marks (p ib : Nat) : ∀ (l : ByCutoff) (w : Walk),
    (∀ x, x ∈ w.marked → x ∈ (visitRange c p ib l w).marked) ∧
    (∀ q, q ∈ w.stack → q ∈ (visitRange c p ib l w).stack) ∧
    (visitRange c p ib l w).stack.length + w.marked.length =
      w.stack.length + (visitRange c p ib l w).marked.length ∧
    (w.marked.Nodup → (visitRange c p ib l w).marked.Nodup) ∧
    (∀ e, e ∈ l → ¬ min ib e.1 < c → ∀ x, x ∈ e.2 → x ∈ (visitRange c p ib l w).marked) ∧
    (∀ x, x ∈ (visitRange c p ib l w).marked → x ∈ w.marked ∨
      ((∃ b, (x, b) ∈ (visitRange c p ib l w).stack) ∧
        ∃ e, e ∈ l ∧ ¬ min ib e.1 < c ∧ x ∈ e.2)) ∧
    (∀ q, q ∈ (visitRange c p ib l w).stack → q ∈ w.stack ∨
      ∃ e, e ∈ l ∧ ¬ min ib e.1 < c ∧ q.2 = min ib e.1 ∧ q.1 ∈ e.2)
  | [], w => by simp [visitRange]
  | (fc, xs) :: rest, w => by
    simp only [visitRange]
    split
    · rename_i hlt
      obtain ⟨h1, h2, h3, h4, h5, h6, h7⟩ := visitRange_marks p ib rest
        { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 }
      simp only at h1 h2 h3 h4 h5 h6 h7
      refine ⟨h1, h2, h3, h4, ?_, ?_, ?_⟩
      · intro e he hn x hx
        rcases List.mem_cons.1 he with rfl | he
        · exact absurd hlt hn
        · exact h5 e he hn x hx
      · intro x hx
        rcases h6 x hx with h | ⟨hb, e, he, hn, hxe⟩
        · exact Or.inl h
        · exact Or.inr ⟨hb, e, List.mem_cons_of_mem _ he, hn, hxe⟩
      · intro q hq
        rcases h7 q hq with h | ⟨e, he, hn, hq2, hq1⟩
        · exact Or.inl h
        · exact Or.inr ⟨e, List.mem_cons_of_mem _ he, hn, hq2, hq1⟩
    · rename_i hge
      let w₁ : Walk := { w with visited := (p, fc) :: w.visited, scan := w.scan + 1 }
      obtain ⟨c1, c2, c3, c4, c5, c6, c7⟩ := visitChildren_marks (min ib fc) xs w₁
      obtain ⟨h1, h2, h3, h4, h5, h6, h7⟩ := visitRange_marks p ib rest
        (visitChildren (min ib fc) xs w₁)
      simp only [w₁] at c1 c2 c3 c4 c5 c6 c7 h1 h2 h3 h4 h5 h6 h7 ⊢
      refine ⟨fun x hx => h1 x (c1 x hx), fun q hq => h2 q (c2 q hq), by omega,
        fun hn => h4 (c4 hn), ?_, ?_, ?_⟩
      · intro e he hn x hx
        rcases List.mem_cons.1 he with rfl | he
        · exact h1 x (c5 x hx)
        · exact h5 e he hn x hx
      · intro x hx
        rcases h6 x hx with h | ⟨hb, e, he, hn, hxe⟩
        · rcases c6 x h with h | ⟨hs, hxs⟩
          · exact Or.inl h
          · exact Or.inr ⟨⟨_, h2 _ hs⟩, (fc, xs), List.mem_cons_self, hge, hxs⟩
        · exact Or.inr ⟨hb, e, List.mem_cons_of_mem _ he, hn, hxe⟩
      · intro q hq
        rcases h7 q hq with h | ⟨e, he, hn, hq2, hq1⟩
        · rcases c7 q h with h | ⟨hq2, hq1⟩
          · exact Or.inl h
          · exact Or.inr ⟨(fc, xs), List.mem_cons_self, hge, hq2, hq1⟩
        · exact Or.inr ⟨e, List.mem_cons_of_mem _ he, hn, hq2, hq1⟩

end Visits

end PersistentNaming.ForkChildren
