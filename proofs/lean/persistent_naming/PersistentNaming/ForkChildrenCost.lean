import PersistentNaming.ForkChildren

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/name_indexes.rs:147-180 sha256:124b9d4e0fdf34ab4a009bbfcc74b464f2e6d92b2050b72a1e7f396355629a07
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/naming_rebuild.rs:61-88 sha256:d9e62c0d572e07d41f4e24110a976d27149e76a7322429abef1f0c8283bf5a50
-- proof-mirror: crates/worth-relational-lineage/src/runtime/state/subsystems/lineage/state.rs:27-27 sha256:7d73c54afb0ba858c6ac38747a1ea187bc0d874c356850b0a2cc049567a2af16

/-!
# Forks inheriting a commit: result and step bound (public #461)

Models `LineageState::forks_inheriting` of `crates/worth-relational-lineage/
src/runtime/state/subsystems/lineage/name_indexes.rs` over the cutoff-indexed
`fork_children` (`BTreeMap<BranchId, BTreeMap<CommitId, BTreeSet<BranchId>>>`
of `state.rs`, kept by `record_fork_ancestry_metadata` in `naming_rebuild.rs`
with one row per forked branch and no empty sets). After a commit `c`, the
walk finds every fork whose inherited history reaches `c`: from each
committing branch it takes the rows `by_cutoff.range(c..)` and, for each
child whose bound `min(inherited, cutoff)` still reaches `c`, marks it and
walks it in turn.

A branch's rows are `(cutoff, children)` pairs; `inheriting c` is the
`range(c..)` the `BTreeMap` yields (a filter: the map is ordered, so it is
that ordered suffix). Steps are one per popped branch, the range seek charged
`log₂ C + 1` for a branch with `C` cutoff rows (the design's cost model of
one ordered `BTreeMap` descent, as `NameResolution` charges its probes), one
per yielded row and one per visited child. The seeks are kept apart from the
walk (`run`'s `TimeM` steps) so the walk can be compared exactly.

* `forks_inheriting_independent_of_older_forks`: adding any older forks
  (cutoff before `c`) under any branches leaves the whole walk, its marked
  forks, popped branches, yielded rows, pops and scan steps, unchanged
  (`∀ O`); only the logarithmic seeks see them.
* `forks_inheriting_visits_no_older_fork`: every yielded row has cutoff `≥ c`.
* `forks_inheriting_steps_bounded`: steps are at most
  `(|seeds| + |marked|)·(log₂ C + 2)` plus one per inheriting row and child of
  the popped branches, with the marked forks distinct
  (`forks_inheriting_marked_nodup`).
* `forks_inheriting_sound`: every marked fork is a child, through an
  inheriting row, of a branch reached from a committing branch.
* `forks_inheriting_closed`: a finished walk has popped every committing
  branch and every marked fork, and marked every child of every inheriting
  row of every popped branch. With one row per branch (as `fork_children`
  keeps), that is every fork inheriting `c`.

Simplifications: the walk pushes every committing branch, where the Rust
first skips branches absent from `fork_children` (and returns at once when
the map is empty); such a branch pops with an empty range, so the result is
the same and the Rust takes at most the model's steps. Branch and commit ids
are natural numbers; `top` stands for `CommitId(u64::MAX)`. The walk runs on
fuel; every theorem holds for every fuel, and `Conformance.lean` checks that
the recorded walks finish.
-/
namespace PersistentNaming.ForkChildren

/-! ## The whole walk -/

/-- Branches reached from the committing branches through inheriting rows,
with their inherited bounds. -/
inductive Reach (F : Forks) (c top : Nat) (seeds : List Nat) : Nat → Nat → Prop where
  | seed {s : Nat} : s ∈ seeds → Reach F c top seeds s top
  | child {p ib fc x : Nat} {xs : List Nat} : Reach F c top seeds p ib → (fc, xs) ∈ F p →
      c ≤ min ib fc → x ∈ xs → Reach F c top seeds x (min ib fc)

/-- The walk's invariants from `start`. -/
structure Inv (F : Forks) (c top : Nat) (seeds : List Nat) (w : Walk) : Prop where
  nodup : w.marked.Nodup
  count : w.pops + w.stack.length ≤ seeds.length + w.marked.length
  scan : w.scan ≤ work F c w.popped
  rows : ∀ e, e ∈ w.visited → c ≤ e.2
  closed : ∀ q, q ∈ w.popped → ∀ fc xs, (fc, xs) ∈ F q.1 → c ≤ min q.2 fc →
    ∀ x, x ∈ xs → x ∈ w.marked
  pending : ∀ x, x ∈ w.marked → (∃ b, (x, b) ∈ w.popped) ∨ ∃ b, (x, b) ∈ w.stack
  seeded : ∀ s, s ∈ seeds → (s, top) ∈ w.popped ∨ (s, top) ∈ w.stack
  reachStack : ∀ q, q ∈ w.stack → Reach F c top seeds q.1 q.2
  reachPopped : ∀ q, q ∈ w.popped → Reach F c top seeds q.1 q.2
  reachMarked : ∀ x, x ∈ w.marked → ∃ p ib fc xs, Reach F c top seeds p ib ∧
    (fc, xs) ∈ F p ∧ c ≤ min ib fc ∧ x ∈ xs

theorem mem_inheriting {c : Nat} {l : ByCutoff} {e : Nat × List Nat} :
    e ∈ inheriting c l ↔ e ∈ l ∧ c ≤ e.1 := by
  simp [inheriting]

theorem start_inv (F : Forks) (c top : Nat) (seeds : List Nat) :
    Inv F c top seeds (start top seeds) where
  nodup := List.nodup_nil
  count := by simp [start]
  scan := Nat.zero_le _
  rows := by simp [start]
  closed := by simp [start]
  pending := by simp [start]
  seeded := fun s hs => Or.inr (by simp [start]; exact hs)
  reachStack := by
    intro q hq
    simp only [start, List.mem_reverse, List.mem_map] at hq
    obtain ⟨s, hs, rfl⟩ := hq
    exact Reach.seed hs
  reachPopped := by simp [start]
  reachMarked := by simp [start]

theorem pop_inv (F : Forks) (c top : Nat) (seeds : List Nat) (w : Walk)
    (h : Inv F c top seeds w) : Inv F c top seeds (pop F c w).val := by
  unfold pop
  split
  · exact h
  rename_i p ib rest hstack
  dsimp only
  let w₁ : Walk := { w with stack := rest, popped := (p, ib) :: w.popped, pops := w.pops + 1 }
  have hp : Reach F c top seeds p ib := h.reachStack (p, ib) (by rw [hstack]; exact List.mem_cons_self)
  obtain ⟨f1, f2, f3, f4⟩ := visitRange_fixed c p ib (inheriting c (F p)) w₁
  obtain ⟨m1, m2, m3, m4, m5, m6, m7⟩ := visitRange_marks c p ib (inheriting c (F p)) w₁
  simp only [w₁] at f1 f2 f3 f4 m1 m2 m3 m4 m5 m6 m7
  have hlen : w.stack.length = rest.length + 1 := by rw [hstack]; rfl
  refine ⟨m4 h.nodup, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_⟩
  · have := h.count; rw [f2]; omega
  · rw [f1]; have := h.scan; simp only [work, List.map_cons, List.sum_cons] at this ⊢; omega
  · intro e he
    rcases f4 e he with he | ⟨_, xs, hxs⟩
    · exact h.rows e he
    · exact (mem_inheriting.1 hxs).2
  · intro q hq fc xs hfx hc x hx
    rw [f1] at hq
    rcases List.mem_cons.1 hq with rfl | hq
    · exact m5 (fc, xs) (mem_inheriting.2 ⟨hfx, Nat.le_trans hc (Nat.min_le_right _ _)⟩)
        (by simp only; omega) x hx
    · exact m1 x (h.closed q hq fc xs hfx hc x hx)
  · intro x hx
    rw [f1]
    rcases m6 x hx with hx | ⟨hb, _⟩
    · rcases h.pending x hx with ⟨b, hb⟩ | ⟨b, hb⟩
      · exact Or.inl ⟨b, List.mem_cons_of_mem _ hb⟩
      · rw [hstack] at hb
        rcases List.mem_cons.1 hb with hb | hb
        · exact Or.inl ⟨b, by rw [hb]; exact List.mem_cons_self⟩
        · exact Or.inr ⟨b, m2 _ hb⟩
    · exact Or.inr hb
  · intro s hs
    rw [f1]
    rcases h.seeded s hs with hb | hb
    · exact Or.inl (List.mem_cons_of_mem _ hb)
    · rw [hstack] at hb
      rcases List.mem_cons.1 hb with hb | hb
      · exact Or.inl (by rw [hb]; exact List.mem_cons_self)
      · exact Or.inr (m2 _ hb)
  · intro q hq
    rcases m7 q hq with hq | ⟨e, he, hn, hq2, hq1⟩
    · exact h.reachStack q (by rw [hstack]; exact List.mem_cons_of_mem _ hq)
    · obtain ⟨hef, _⟩ := mem_inheriting.1 he
      have : q = (q.1, min ib e.1) := by rw [← hq2]
      rw [this]
      exact Reach.child hp hef (by omega) hq1
  · intro q hq
    rw [f1] at hq
    rcases List.mem_cons.1 hq with rfl | hq
    · exact hp
    · exact h.reachPopped q hq
  · intro x hx
    rcases m6 x hx with hx | ⟨_, e, he, hn, hxe⟩
    · exact h.reachMarked x hx
    · exact ⟨p, ib, e.1, e.2, hp, (mem_inheriting.1 he).1, by omega, hxe⟩

theorem run_inv (F : Forks) (c top : Nat) (seeds : List Nat) :
    ∀ (n : Nat) (w : Walk), Inv F c top seeds w → Inv F c top seeds (run F c n w).val
  | 0, _, h => h
  | n + 1, w, h => run_inv F c top seeds n _ (pop_inv F c top seeds w h)

theorem forks_inheriting_inv (F : Forks) (c top : Nat) (seeds : List Nat) (fuel : Nat) :
    Inv F c top seeds (forksInheriting F c top seeds fuel).val :=
  run_inv F c top seeds fuel _ (start_inv F c top seeds)

/-- No row with a cutoff before the commit is ever yielded. -/
theorem forks_inheriting_visits_no_older_fork (F : Forks) (c top : Nat) (seeds : List Nat)
    (fuel : Nat) : ∀ e, e ∈ (forksInheriting F c top seeds fuel).val.visited → c ≤ e.2 :=
  (forks_inheriting_inv F c top seeds fuel).rows

theorem forks_inheriting_marked_nodup (F : Forks) (c top : Nat) (seeds : List Nat)
    (fuel : Nat) : (forksInheriting F c top seeds fuel).val.marked.Nodup :=
  (forks_inheriting_inv F c top seeds fuel).nodup

/-- Every marked fork is a child, through a row whose bound reaches the
commit, of a branch reached from a committing branch. -/
theorem forks_inheriting_sound (F : Forks) (c top : Nat) (seeds : List Nat) (fuel : Nat) :
    ∀ x, x ∈ (forksInheriting F c top seeds fuel).val.marked → ∃ p ib fc xs,
      Reach F c top seeds p ib ∧ (fc, xs) ∈ F p ∧ c ≤ min ib fc ∧ x ∈ xs :=
  (forks_inheriting_inv F c top seeds fuel).reachMarked

/-- A finished walk has popped every committing branch and marked fork, and
marked every child of every inheriting row of a popped branch. -/
theorem forks_inheriting_closed (F : Forks) (c top : Nat) (seeds : List Nat) (fuel : Nat)
    (done : (forksInheriting F c top seeds fuel).val.stack = []) :
    (∀ s, s ∈ seeds → (s, top) ∈ (forksInheriting F c top seeds fuel).val.popped) ∧
    (∀ x, x ∈ (forksInheriting F c top seeds fuel).val.marked →
      ∃ b, (x, b) ∈ (forksInheriting F c top seeds fuel).val.popped) ∧
    (∀ q, q ∈ (forksInheriting F c top seeds fuel).val.popped → ∀ fc xs, (fc, xs) ∈ F q.1 →
      c ≤ min q.2 fc → ∀ x, x ∈ xs → x ∈ (forksInheriting F c top seeds fuel).val.marked) := by
  have h := forks_inheriting_inv F c top seeds fuel
  refine ⟨fun s hs => ?_, fun x hx => ?_, h.closed⟩
  · rcases h.seeded s hs with h' | h'
    · exact h'
    · rw [done] at h'; cases h'
  · rcases h.pending x hx with h' | ⟨b, h'⟩
    · exact h'
    · rw [done] at h'; cases h'

/-! ## Step bound -/

theorem log2_mono {a b : Nat} (h : a ≤ b) : Nat.log2 a ≤ Nat.log2 b := by
  rcases Nat.eq_zero_or_pos a with rfl | ha
  · simp [Nat.log2_zero]
  · exact (Nat.le_log2 (by omega)).2 (Nat.le_trans (Nat.log2_self_le (by omega)) h)

theorem seekCost_le {l : ByCutoff} {C : Nat} (h : l.length ≤ C) :
    seekCost l ≤ Nat.log2 C + 1 := by
  unfold seekCost
  split
  · omega
  · have := log2_mono h; omega

theorem run_seek (F : Forks) (c L : Nat) (hL : ∀ p, seekCost (F p) ≤ L) :
    ∀ (n : Nat) (w : Walk), (run F c n w).steps + w.pops * L ≤ (run F c n w).val.pops * L
  | 0, w => by simp [run]
  | n + 1, w => by
    have ih := run_seek F c L hL n (pop F c w).val
    have hpop : (pop F c w).steps + w.pops * L ≤ (pop F c w).val.pops * L := by
      unfold pop
      split
      · simp
      · rename_i p ib rest _
        have := (visitRange_fixed c p ib (inheriting c (F p))
          { w with stack := rest, popped := (p, ib) :: w.popped, pops := w.pops + 1 }).2.1
        simp only at this ⊢
        rw [this, Nat.succ_mul]
        have := hL p; omega
    simp only [run, TimeM.bind_val, TimeM.bind_steps]
    generalize (pop F c w).val.pops * L = a at *
    generalize w.pops * L = b at *
    omega

/-- Steps are at most `(|seeds| + |marked|)·(log₂ C + 2)` plus the inheriting
rows and children of the popped branches: no row before the commit counts. -/
theorem forks_inheriting_steps_bounded (F : Forks) (c top : Nat) (seeds : List Nat)
    (fuel C : Nat) (hC : ∀ p, (F p).length ≤ C) :
    steps (forksInheriting F c top seeds fuel) ≤
      (seeds.length + (forksInheriting F c top seeds fuel).val.marked.length) * (Nat.log2 C + 2) +
        work F c (forksInheriting F c top seeds fuel).val.popped := by
  have h := forks_inheriting_inv F c top seeds fuel
  have hs := run_seek F c (Nat.log2 C + 1) (fun p => seekCost_le (hC p)) fuel (start top seeds)
  rw [show (start top seeds).pops = 0 from rfl, Nat.zero_mul, Nat.add_zero] at hs
  have hcount := h.count
  have hscan := h.scan
  unfold steps forksInheriting at *
  generalize run F c fuel (start top seeds) = r at *
  have hp : r.val.pops ≤ seeds.length + r.val.marked.length := by omega
  have := Nat.mul_le_mul_right (Nat.log2 C + 1) hp
  rw [show Nat.log2 C + 2 = Nat.log2 C + 1 + 1 by omega, Nat.mul_succ]
  omega

end PersistentNaming.ForkChildren
