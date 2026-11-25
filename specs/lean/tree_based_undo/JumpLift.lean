-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/recording.rs:34-38 sha256:270717f3ae9c010eee32fa6619501a380ed4dde030149178e33e5a26d58e5645
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:29-49 sha256:109e3c8b3c8b373c2f9052841ea5efb632394bb92b6831e447e7e227b71aef1e
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:37-43 sha256:ba587940a6ee28d4fd7c4e9f4f3bbc88d5c22d1185535bc7e3441311a47009b2
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:135-158 sha256:29032e5260a39f9647d18cd17e63b31019a7139e39f852c828a21b689217fc0f
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:150-156 sha256:e1d85393d7dd13cedba09b046ea2921872307e1ccea67b64f65c4b6eeb1ed10b
/-
Skew-binary jump pointers and the level-ancestor lift, unbounded.

Mirrors (TBU Phase 2, `54a0be61a1`, unchanged at `6a420909dc`)
  workspaces/worth-query/crates/worth-query-execution/src/domain_computation/
  primary_graph/product_operation/document_history/journal/sources.rs
    * `child_position`        sources.rs:29-49   -> `jumpOf` (transliteration of
      the rule; the node map is abstracted to a parent function on Nat)
    * `is_ancestor_or_self`   sources.rs:135-158 -> `lift` / `isAncestorOrSelf`
      (transliteration of the loop as structural recursion on depth; the
      `None` arms are dropped because every Nat is a node here)

Faithfulness: an abstraction. The tree is unbounded (every Nat is a node,
`par n < n` for n > 0, the root 0 is its own parent). The jump rule and the lift
step are the code's, symbol for symbol. The depth-only table `J` is the same
recurrence the Verus model calls `jd`
(specs/verus/tree_based_undo/jump_pointer_lift.rs).

Main results
  * `jump_is_level_ancestor` : the jump pointer of n is n's ancestor at depth
    J (depth n): the invariant `child_position` establishes at reserve.
  * `lift_eq_up`, `isAncestorOrSelf_iff` : the lift is exactly parent-chain
    ancestry.
  * `liftCount_eq_steps`, `steps_log_bound` : the lift takes
    steps (depth n) (depth a) <= 4 * log2 (depth n + 1) + 2 iterations.
-/
set_option linter.deprecated false
set_option linter.unusedVariables false

namespace JumpLift

/-! ## The depth-only jump table -/

/-- `tab p i` is the jump depth of depth `i`, for `i ≤ p`, built depth by depth
with the rule of `child_position` (sources.rs:37-43). -/
def tab : Nat → Nat → Nat
  | 0, _ => 0
  | p + 1, i =>
    if i ≤ p then tab p i
    else if p - tab p p = tab p p - tab p (tab p p) then tab p (tab p p) else p

/-- The depth a node at depth `d` jumps to. -/
def J (d : Nat) : Nat := tab d d

theorem tab_le : ∀ p i, tab p i ≤ i
  | 0, i => by simp [tab]
  | p + 1, i => by
    have h1 := tab_le p i
    have h2 := tab_le p p
    have h3 := tab_le p (tab p p)
    unfold tab
    split
    · exact h1
    · split <;> omega

theorem tab_stable (i p : Nat) : ∀ q, i ≤ p → tab (p + q) i = tab p i
  | 0, _ => rfl
  | q + 1, h => by
    have := tab_stable i p q h
    show tab (p + q + 1) i = tab p i
    rw [tab.eq_2, if_pos (by omega)]
    exact this

theorem J_tab {i p : Nat} (h : i ≤ p) : tab p i = J i := by
  have := tab_stable i i (p - i) (Nat.le_refl i)
  rw [Nat.add_sub_cancel' h] at this
  exact this

theorem J_zero : J 0 = 0 := rfl

theorem J_le (d : Nat) : J d ≤ d := tab_le d d

/-- The recurrence `child_position` implements. -/
theorem J_succ (p : Nat) :
    J (p + 1) = if p - J p = J p - J (J p) then J (J p) else p := by
  show tab (p + 1) (p + 1) = _
  rw [tab.eq_2, if_neg (by omega)]
  have hp : tab p p = J p := rfl
  have hj : tab p (J p) = J (J p) := J_tab (J_le p)
  rw [hp, hj]

/-! ## Structure of the jumps: skew-binary weights -/

/-- Jump length. -/
def w (x : Nat) : Nat := x - J x

def Skew (n : Nat) : Prop := ∃ k, 0 < k ∧ n + 1 = 2 ^ k

/-- Everything the counting argument needs about one depth. -/
structure Good (x : Nat) : Prop where
  lt : 0 < x → J x < x
  skew : 0 < x → Skew (w x)
  s1 : J x ≠ 0 → w x ≤ w (J x)
  s2 : J x ≠ 0 → J (J x) ≠ 0 → w (J x) < w (J (J x))
  dec : 1 < w x → J (x - 1) ≠ 0 ∧ J (J (x - 1)) = J x ∧ w (x - 1) = w (J (x - 1)) ∧
      w x = 2 * w (x - 1) + 1

theorem skew_gap {u v : Nat} (hu : Skew u) (hv : Skew v) (h : v < u) : 2 * v + 1 ≤ u := by
  obtain ⟨i, _, hi⟩ := hu
  obtain ⟨j, _, hj⟩ := hv
  have hlt : 2 ^ j < 2 ^ i := by omega
  have hji : j < i := (Nat.pow_lt_pow_iff_right (a := 2) (by decide)).mp hlt
  have : 2 ^ (j + 1) ≤ 2 ^ i := Nat.pow_le_pow_right (by decide) hji
  rw [Nat.pow_succ] at this
  omega

theorem good : ∀ x, Good x := by
  intro x
  induction x using Nat.strongRecOn with
  | _ x ih =>
  match x with
  | 0 =>
    exact ⟨by omega, by omega, by simp [J_zero], by simp [J_zero],
      by simp [w, J_zero]⟩
  | p + 1 =>
    have hrec := J_succ p
    have ha : J p ≤ p := J_le p
    have hb : J (J p) ≤ J p := J_le (J p)
    have gp := ih p (by omega)
    have ga := ih (J p) (by omega)
    have gb := ih (J (J p)) (by omega)
    by_cases hE : p - J p = J p - J (J p)
    · rw [if_pos hE] at hrec
      by_cases hp0 : p = 0
      · subst hp0
        have h0 : J 1 = 0 := by rw [hrec]; rfl
        refine ⟨by omega, fun _ => ⟨1, by decide, by simp [w, h0]⟩, ?_, ?_, ?_⟩
        · intro h; exact absurd h0 h
        · intro h; exact absurd h0 h
        · intro h; simp [w, h0] at h
      · have hap : J p < p := gp.lt (by omega)
        have ha0 : J p ≠ 0 := by omega
        have hwx : w (p + 1) = 2 * w p + 1 := by unfold w; rw [hrec]; omega
        have hwa : w (J p) = w p := by unfold w; omega
        obtain ⟨k, hk0, hk⟩ := gp.skew (by omega)
        refine ⟨by omega, ?_, ?_, ?_, ?_⟩
        · intro _
          refine ⟨k + 1, by omega, ?_⟩
          rw [hwx, Nat.pow_succ]; omega
        · intro hb0
          rw [hrec] at hb0 ⊢
          have hlt : w (J p) < w (J (J p)) := gp.s2 ha0 hb0
          have hsb : Skew (w (J (J p))) := gb.skew (by omega)
          have hsp : Skew (w p) := ⟨k, hk0, hk⟩
          have := skew_gap hsb hsp (by omega)
          omega
        · intro hb0 hbb
          rw [hrec] at hb0 hbb ⊢
          exact ga.s2 hb0 hbb
        · intro _
          simp only [Nat.add_sub_cancel]
          exact ⟨ha0, hrec.symm, hwa.symm, hwx⟩
    · rw [if_neg hE] at hrec
      refine ⟨by omega, ?_, ?_, ?_, ?_⟩
      · intro _; exact ⟨1, by decide, by unfold w; rw [hrec]; omega⟩
      · intro hp0
        rw [hrec] at hp0 ⊢
        have := gp.lt (by omega)
        unfold w; rw [hrec]; omega
      · intro hp0 ha0
        rw [hrec] at hp0 ha0 ⊢
        have h1 := gp.s1 ha0
        have : w p ≠ w (J p) := by unfold w; omega
        omega
      · intro h; unfold w at h; rw [hrec] at h; omega

theorem J_lt {x : Nat} (h : 0 < x) : J x < x := (good x).lt h

/-! ## The depth-only query: how far one lift iteration moves -/

/-- One iteration of the lift toward depth `t`, on depths (sources.rs:150-156). -/
def qnext (x t : Nat) : Nat := if t ≤ J x then J x else x - 1

/-- Iterations of the lift from depth `x` to depth `t`. -/
def steps (x t : Nat) : Nat :=
  if h : x ≤ t then 0 else 1 + steps (qnext x t) t
termination_by x
decreasing_by
  have := J_lt (x := x) (by omega)
  unfold qnext; split <;> omega

theorem steps_eq (x t : Nat) (h : t < x) : steps x t = 1 + steps (qnext x t) t := by
  rw [steps]; rw [dif_neg (by omega)]

theorem steps_le (x t : Nat) (h : x ≤ t) : steps x t = 0 := by
  rw [steps]; rw [dif_pos h]

theorem pow_inj {a b : Nat} (h : 2 ^ a = 2 ^ b) : a = b := by
  rcases Nat.lt_trichotomy a b with hab | hab | hab
  · have := (Nat.pow_lt_pow_iff_right (a := 2) (by decide)).mpr hab; omega
  · exact hab
  · have := (Nat.pow_lt_pow_iff_right (a := 2) (by decide)).mpr hab; omega

/-- The level of a jump length. -/
def lev (x : Nat) : Nat := Nat.log2 (w x + 1)

theorem lev_pow {x : Nat} (h : 0 < x) : w x + 1 = 2 ^ lev x := by
  obtain ⟨k, _, hk⟩ := (good x).skew h
  unfold lev; rw [hk, Nat.log2_two_pow]

theorem lev_pos {x : Nat} (h : 0 < x) : 0 < lev x := by
  obtain ⟨k, hk0, hk⟩ := (good x).skew h
  unfold lev; rw [hk, Nat.log2_two_pow]; exact hk0

theorem lev_le {x D : Nat} (h : 0 < x) (hx : x ≤ D) : lev x ≤ Nat.log2 (D + 1) := by
  have hp := lev_pow h
  have hw : w x ≤ x := by unfold w; omega
  exact (Nat.le_log2 (by omega)).mpr (by omega)

theorem lev_mono {x y : Nat} (hx : 0 < x) (hy : 0 < y) (h : w x < w y) : lev x < lev y := by
  have px := lev_pow hx
  have py := lev_pow hy
  exact (Nat.pow_lt_pow_iff_right (a := 2) (by decide)).mp (by omega)

theorem lev_eq {x y : Nat} (hx : 0 < x) (hy : 0 < y) (h : w x = w y) : lev x = lev y := by
  unfold lev; rw [h]

/-- After a jump overshoots, each level costs at most two iterations. -/
theorem descend : ∀ x t, t ≤ x → (x = t ∨ J x < t) → steps x t ≤ 2 * lev x := by
  intro x
  induction x using Nat.strongRecOn with
  | _ x ih =>
  intro t htx hcase
  by_cases hxt : x = t
  · rw [steps_le x t (by omega)]; omega
  have hJ : J x < t := by omega
  have hx0 : 0 < x := by omega
  have gx := good x
  have hnext : qnext x t = x - 1 := by unfold qnext; rw [if_neg (by omega)]
  rw [steps_eq x t (by omega), hnext]
  -- a unit jump would land on x - 1 = J x < t ≤ x, so x = t
  have hw : 1 < w x := by
    have := lev_pow hx0
    have := gx.lt hx0
    unfold w; omega
  obtain ⟨hd0, hdd, hwd, hwx⟩ := gx.dec hw
  have hd : 0 < x - 1 := by omega
  have hlevd : lev x = lev (x - 1) + 1 := by
    have p1 := lev_pow hx0
    have p2 := lev_pow hd
    have : 2 ^ lev x = 2 ^ (lev (x - 1) + 1) := by rw [Nat.pow_succ]; omega
    exact pow_inj this
  by_cases hdt : x - 1 = t
  · rw [steps_le _ t (by omega)]
    have := lev_pos hx0; omega
  by_cases hjump : t ≤ J (x - 1)
  · have hn : qnext (x - 1) t = J (x - 1) := by unfold qnext; rw [if_pos hjump]
    rw [steps_eq (x - 1) t (by omega), hn]
    have hy : J (x - 1) < x - 1 := J_lt hd
    have := ih (J (x - 1)) (by omega) t hjump (by right; rw [hdd]; exact hJ)
    have hl : lev (J (x - 1)) = lev (x - 1) := (lev_eq hd (by omega) hwd).symm
    omega
  · have := ih (x - 1) (by omega) t (by omega) (by right; omega)
    omega

/-- Before the first overshoot the jumps climb levels, each at most twice. -/
theorem ascend (D : Nat) : ∀ x t, t ≤ x → x ≤ D →
    steps x t ≤ 2 * Nat.log2 (D + 1) + 2 * (Nat.log2 (D + 1) - lev x) +
      (if J x ≠ 0 ∧ w (J x) = w x then 2 else 1) := by
  intro x
  induction x using Nat.strongRecOn with
  | _ x ih =>
  intro t htx hxD
  by_cases hxt : x = t
  · rw [steps_le x t (by omega)]; omega
  have hx0 : 0 < x := by omega
  have gx := good x
  have hlx := lev_le hx0 hxD
  by_cases hjump : t ≤ J x
  · have hn : qnext x t = J x := by unfold qnext; rw [if_pos hjump]
    rw [steps_eq x t (by omega), hn]
    have hy : J x < x := gx.lt hx0
    by_cases hy0 : J x = 0
    · rw [steps_le (J x) t (by omega)]; split <;> omega
    have hyD : J x ≤ D := by omega
    have hly := lev_le (Nat.pos_of_ne_zero hy0) hyD
    have ihy := ih (J x) hy t hjump hyD
    have hs1 := gx.s1 hy0
    by_cases heq : w (J x) = w x
    · -- a repeated weight: the next jump climbs strictly
      have hl : lev (J x) = lev x := (lev_eq hx0 (Nat.pos_of_ne_zero hy0) heq.symm).symm
      have hr : ¬ (J (J x) ≠ 0 ∧ w (J (J x)) = w (J x)) := by
        intro ⟨h1, h2⟩
        have := gx.s2 hy0 h1
        omega
      rw [if_neg hr] at ihy
      rw [if_pos ⟨hy0, heq⟩]
      omega
    · have hl : lev x < lev (J x) := lev_mono hx0 (Nat.pos_of_ne_zero hy0) (by omega)
      have : (if J (J x) ≠ 0 ∧ w (J (J x)) = w (J x) then 2 else 1) ≤ 2 := by split <;> omega
      have : 1 ≤ (if J x ≠ 0 ∧ w (J x) = w x then 2 else 1) := by split <;> omega
      omega
  · have := descend x t htx (by right; omega)
    split <;> omega

/-- The lift takes O(log depth) iterations, unbounded. -/
theorem steps_log_bound (x t : Nat) : steps x t ≤ 4 * Nat.log2 (x + 1) + 2 := by
  by_cases h : x ≤ t
  · rw [steps_le x t h]; omega
  have := ascend x x t (by omega) (Nat.le_refl x)
  split at this <;> omega

/-! ## The unbounded history tree -/

/-- A history tree on all of Nat: node 0 is the root and its own parent; every
other node's parent has a smaller ordinal (reserve takes a fresh `next` above
the current step, recording.rs:34-38 and 69-83). -/
structure Tree where
  par : Nat → Nat
  par_root : par 0 = 0
  par_lt : ∀ n, 0 < n → par n < n

variable (T : Tree)

def depth (n : Nat) : Nat :=
  if h : n = 0 then 0 else depth (T.par n) + 1
termination_by n
decreasing_by exact T.par_lt n (by omega)

theorem depth_zero : depth T 0 = 0 := by rw [depth]; simp

theorem depth_succ {n : Nat} (h : 0 < n) : depth T n = depth T (T.par n) + 1 := by
  rw [depth]; rw [dif_neg (by omega)]

theorem depth_eq_zero {n : Nat} : depth T n = 0 ↔ n = 0 := by
  constructor
  · intro h; by_cases hn : n = 0
    · exact hn
    · rw [depth_succ T (by omega)] at h; omega
  · intro h; subst h; exact depth_zero T

/-- `up k n`: follow the parent link k times. -/
def up : Nat → Nat → Nat
  | 0, n => n
  | k + 1, n => up k (T.par n)

theorem up_add : ∀ a b n, up T (a + b) n = up T b (up T a n)
  | 0, b, n => by simp [up]
  | a + 1, b, n => by
    rw [show a + 1 + b = (a + b) + 1 by omega]
    simp only [up]
    exact up_add a b (T.par n)

theorem up_succ' (k n : Nat) : up T (k + 1) n = T.par (up T k n) := by
  rw [up_add T k 1 n]; rfl

theorem depth_up : ∀ k n, k ≤ depth T n → depth T (up T k n) = depth T n - k
  | 0, n, _ => by simp [up]
  | k + 1, n, h => by
    have hn : 0 < n := by
      cases Nat.eq_zero_or_pos n with
      | inl h0 => subst h0; rw [depth_zero] at h; omega
      | inr h => exact h
    have hd := depth_succ T hn
    simp only [up]
    rw [depth_up k (T.par n) (by omega)]
    omega

theorem up_root : ∀ k, up T k 0 = 0
  | 0 => rfl
  | k + 1 => by simp only [up]; rw [T.par_root]; exact up_root k

theorem up_past (k n : Nat) (h : depth T n ≤ k) : up T k n = 0 := by
  have h1 := depth_up T (depth T n) n (Nat.le_refl _)
  rw [Nat.sub_self, depth_eq_zero] at h1
  have := up_add T (depth T n) (k - depth T n) n
  rw [Nat.add_sub_cancel' h, h1, up_root] at this
  exact this

/-- The jump pointer, by the rule of `child_position` (sources.rs:29-49):
the child of `p` jumps to `jump (jump p)` when the two gaps above `p` are
equal, else to `p`. Bundled with `≤ n` so the nested recursion terminates. -/
def jumpB (n : Nat) : {q : Nat // q ≤ n} :=
  if h : n = 0 then ⟨0, by omega⟩ else
    have hp : T.par n < n := T.par_lt n (by omega)
    match jumpB (T.par n) with
    | ⟨a, ha⟩ =>
      match jumpB a with
      | ⟨b, hb⟩ =>
        if depth T (T.par n) - depth T a = depth T a - depth T b then ⟨b, by omega⟩
        else ⟨T.par n, by omega⟩
termination_by n
decreasing_by all_goals omega

def jumpOf (n : Nat) : Nat := (jumpB T n).1

theorem jumpOf_zero : jumpOf T 0 = 0 := by
  unfold jumpOf; rw [jumpB]; simp

theorem jumpOf_le (n : Nat) : jumpOf T n ≤ n := (jumpB T n).2

theorem jumpOf_succ {n : Nat} (h : 0 < n) :
    jumpOf T n =
      let p := T.par n
      let a := jumpOf T p
      let b := jumpOf T a
      if depth T p - depth T a = depth T a - depth T b then b else p := by
  unfold jumpOf
  rw [jumpB, dif_neg (by omega)]
  simp only
  generalize jumpB T (T.par n) = A
  obtain ⟨a, ha⟩ := A
  simp only
  generalize jumpB T a = B
  obtain ⟨b, hb⟩ := B
  simp only
  split <;> rfl

/-- The invariant `child_position` establishes at reserve: a node's jump is its
ancestor at depth `J (depth n)`. -/
theorem jump_is_level_ancestor : ∀ n,
    jumpOf T n = up T (depth T n - J (depth T n)) n := by
  intro n
  induction n using Nat.strongRecOn with
  | _ n ih =>
  by_cases hn : n = 0
  · subst hn; rw [jumpOf_zero, depth_zero, J_zero]; rfl
  have hn0 : 0 < n := by omega
  have hp : T.par n < n := T.par_lt n hn0
  have hdn := depth_succ T hn0
  rw [jumpOf_succ T hn0]
  simp only
  -- p's jump and its jump, as ancestors at known depths
  have hja := ih (T.par n) hp
  have ha_le : jumpOf T (T.par n) ≤ T.par n := jumpOf_le T _
  have hjb := ih (jumpOf T (T.par n)) (by omega)
  have hJp := J_le (depth T (T.par n))
  have hda : depth T (jumpOf T (T.par n)) = J (depth T (T.par n)) := by
    rw [hja, depth_up T _ _ (by omega)]; omega
  have hJa := J_le (depth T (jumpOf T (T.par n)))
  have hdb : depth T (jumpOf T (jumpOf T (T.par n))) = J (J (depth T (T.par n))) := by
    have := J_le (J (depth T (T.par n)))
    rw [hjb, depth_up T _ _ (by omega)]; rw [hda]; omega
  have hrec := J_succ (depth T (T.par n))
  rw [← hdn] at hrec
  rw [hda, hdb]
  split
  · rename_i hE
    rw [if_pos hE] at hrec
    have hbb := J_le (J (depth T (T.par n)))
    rw [hjb, hda, hja, ← up_add T, hrec, hdn]
    have e : depth T (T.par n) + 1 - J (J (depth T (T.par n))) =
        (depth T (T.par n) - J (depth T (T.par n)) +
          (J (depth T (T.par n)) - J (J (depth T (T.par n))))) + 1 := by omega
    rw [e]; rfl
  · rename_i hE
    rw [if_neg hE] at hrec
    rw [hrec, hdn, Nat.add_sub_cancel_left]
    rfl

/-! ## The lift (`is_ancestor_or_self`, sources.rs:135-158) -/

/-- The loop of `is_ancestor_or_self`, lifting `at` to depth `d`: returns the
node it stops at and the number of iterations. -/
def lift (d at_ : Nat) : Nat × Nat :=
  if h : depth T at_ ≤ d then (at_, 0) else
    let next := if d ≤ depth T (jumpOf T at_) then jumpOf T at_ else T.par at_
    let r := lift d next
    (r.1, r.2 + 1)
termination_by depth T at_
decreasing_by
  have h0 : 0 < at_ := by
    cases Nat.eq_zero_or_pos at_ with
    | inl h0 => subst h0; rw [depth_zero] at h; omega
    | inr h => exact h
  have hd := depth_succ T h0
  have hj := jump_is_level_ancestor T at_
  have hJ := J_lt (x := depth T at_) (by omega)
  split
  · rw [hj, depth_up T _ _ (by omega)]; omega
  · omega

/-- `is_ancestor_or_self(ancestor, node)`: lift `node` to the depth of
`ancestor` and compare (the loop's `depth == depth` and `depth < depth` exits). -/
def isAncestorOrSelf (a n : Nat) : Bool :=
  let r := lift T (depth T a) n
  depth T r.1 == depth T a && r.1 == a

theorem lift_spec : ∀ at_ d, d ≤ depth T at_ →
    (lift T d at_).1 = up T (depth T at_ - d) at_ ∧
    (lift T d at_).2 = steps (depth T at_) d := by
  intro at_
  induction at_ using Nat.strongRecOn with
  | _ x ih =>
  intro d hd
  by_cases hle : depth T x ≤ d
  · rw [lift, dif_pos hle, steps_le _ _ hle]
    have : depth T x - d = 0 := by omega
    rw [this]; exact ⟨rfl, rfl⟩
  have hx0 : 0 < x := by
    cases Nat.eq_zero_or_pos x with
    | inl h0 => subst h0; rw [depth_zero] at hle; omega
    | inr h => exact h
  have hdx := depth_succ T hx0
  have hj := jump_is_level_ancestor T x
  have hJ := J_lt (x := depth T x) (by omega)
  have hdj : depth T (jumpOf T x) = J (depth T x) := by
    rw [hj, depth_up T _ _ (by omega)]; omega
  rw [lift, dif_neg hle, steps_eq _ _ (by omega)]
  simp only
  unfold qnext
  rw [← hdj]
  split
  · rename_i hjd
    have hlt : jumpOf T x < x := by
      have := jumpOf_le T x
      rcases Nat.lt_or_eq_of_le this with h | h
      · exact h
      · rw [h] at hdj; omega
    obtain ⟨h1, h2⟩ := ih (jumpOf T x) hlt d hjd
    refine ⟨?_, by rw [h2]; omega⟩
    have hjd' : d ≤ J (depth T x) := hdj ▸ hjd
    rw [h1, hdj, hj, ← up_add T]
    congr 1
    omega
  · rename_i hjd
    obtain ⟨h1, h2⟩ := ih (T.par x) (T.par_lt x hx0) d (by omega)
    rw [hdj] at hjd
    refine ⟨?_, by rw [h2, hdx, Nat.add_sub_cancel]; omega⟩
    rw [h1]
    have : depth T x - d = (depth T (T.par x) - d) + 1 := by omega
    rw [this]
    rfl

/-- The lift is exactly parent-chain ancestry. -/
theorem isAncestorOrSelf_iff (a n : Nat) :
    isAncestorOrSelf T a n = true ↔ ∃ k, up T k n = a := by
  unfold isAncestorOrSelf
  simp only [Bool.and_eq_true, beq_iff_eq]
  by_cases hd : depth T a ≤ depth T n
  · obtain ⟨h1, _⟩ := lift_spec T n (depth T a) hd
    rw [h1]
    constructor
    · intro ⟨_, h⟩; exact ⟨_, h⟩
    · intro ⟨k0, hk0⟩
      obtain ⟨k, hk', hk⟩ : ∃ k, k ≤ depth T n ∧ up T k n = a := by
        by_cases hkn : k0 ≤ depth T n
        · exact ⟨k0, hkn, hk0⟩
        · rw [up_past T k0 n (by omega)] at hk0
          exact ⟨depth T n, Nat.le_refl _, by rw [up_past T _ n (Nat.le_refl _)]; exact hk0⟩
      have hda := depth_up T k n hk'
      rw [hk] at hda
      have : depth T n - depth T a = k := by omega
      rw [this, hk]
      exact ⟨rfl, rfl⟩
  · have hle : depth T n ≤ depth T a := by omega
    rw [lift, dif_pos hle]
    simp only
    constructor
    · intro ⟨h, _⟩; omega
    · intro ⟨k, hk⟩
      exfalso
      by_cases hkn : k ≤ depth T n
      · have := depth_up T k n hkn; rw [hk] at this; omega
      · rw [up_past T k n (by omega)] at hk
        subst hk; rw [depth_zero] at hd; omega

/-- The lift from `n` to an ancestor's depth reaches the level ancestor in
O(log depth) iterations. -/
theorem lift_reaches_level_ancestor_in_log_steps (n d : Nat) (hd : d ≤ depth T n) :
    (lift T d n).1 = up T (depth T n - d) n ∧
    depth T (lift T d n).1 = d ∧
    (lift T d n).2 ≤ 4 * Nat.log2 (depth T n + 1) + 2 := by
  obtain ⟨h1, h2⟩ := lift_spec T n d hd
  refine ⟨h1, ?_, ?_⟩
  · rw [h1, depth_up T _ _ (by omega)]; omega
  · rw [h2]; exact steps_log_bound _ _

end JumpLift
