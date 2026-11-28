import CanonicalGroupDelta.TimeM

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/group_read.rs:180-212 sha256:bcaba1614c24316c484e7c30bca404cc7c353df487f6485805d1b7854df9c4a6
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/group_install.rs:38-54 sha256:0f58c02bb07296bab67e20ad2c2dc68a42ade6e6dee397bef81b8128cb925183
-- proof-mirror: crates/worth-relational-mutation/src/aggregate_identity/transition/group/delta/naming.rs:166-211 sha256:51e2e866637caeb04a4143a23ed7bd1cf9b66f7264180ce9a51da2266e9f0443
-- proof-mirror: crates/worth-relational-mutation/src/aggregate_identity/transition/group/delta/scope.rs:226-283 sha256:4c643e6399b100b4d4a25f045228ed2ddfcd63cb139d8c8dbf7178e82ca33692

/-!
# Residual groups: ordered seeks and role counts without a member walk

The model of the incremental group authority's reads of a scoped prior
group in `crates/worth-relational-mutation/src/aggregate_identity/
transition/group/delta/{scope,naming}.rs` over the branch group state's
ordered member rows (`group_read.rs::least_member`).

A scoped group of `M` members keeps every member the commit does not touch
together in its residual. The authority never materialises the residual:

* `seek_val`, `seek_steps_bounded`: the residual's least member (or least
  member holding a role) is one ordered walk of the group's ascending member
  rows that passes only skipped (touched) members: it equals the least
  member of the materialised residual and takes at most `|skip| + 1` steps
  for any `M`.
* `residual_count`, `residualBoundary_val`, `residualBoundary_steps`: the
  residual's Boundary-role count is the group's stored count minus the
  touched Boundary members, the same number as counting the materialised
  residual, in `|touched| + 1` steps.
* `scope_steps_le`: naming a whole scope of groups costs at most
  `2·|touched| + 2` steps per scoped group: no member count appears.
-/
namespace CanonicalGroupDelta

/-- The least member of `l` (ascending) not in `skip`: one step per row
passed. -/
def seek (skip : List Nat) : List Nat → TimeM (Option Nat)
  | [] => TimeM.tick none
  | x :: xs =>
    if x ∈ skip then TimeM.bind (TimeM.tick ()) (fun _ => seek skip xs)
    else TimeM.tick (some x)

/-- The seek finds the materialised residual's least member. -/
theorem seek_val (skip : List Nat) :
    ∀ (l : List Nat), (seek skip l).val = (l.filter (fun x => decide (x ∉ skip))).head?
  | [] => rfl
  | x :: xs => by
    by_cases h : x ∈ skip
    · simp [seek, h, seek_val skip xs]
    · simp [seek, h]

theorem seek_steps_le_skipped (skip : List Nat) :
    ∀ (l : List Nat), (seek skip l).steps ≤ (l.filter (fun x => decide (x ∈ skip))).length + 1
  | [] => by simp [seek]
  | x :: xs => by
    by_cases h : x ∈ skip
    · have := seek_steps_le_skipped skip xs
      simp [seek, h]; omega
    · simp [seek, h]

/-- A duplicate-free list has at most `|t|` members of `t`. -/
theorem nodup_filter_mem_le :
    ∀ (l t : List Nat), l.Nodup → (l.filter (fun x => decide (x ∈ t))).length ≤ t.length
  | [], _, _ => by simp
  | x :: xs, t, hl => by
    rw [List.nodup_cons] at hl
    obtain ⟨hx, hxs⟩ := hl
    by_cases h : x ∈ t
    · have same : xs.filter (fun y => decide (y ∈ t)) =
          xs.filter (fun y => decide (y ∈ t.erase x)) := by
        apply List.filter_congr
        intro y hy
        have hne : y ≠ x := fun e => hx (e ▸ hy)
        simp [List.mem_erase_of_ne hne]
      have ih := nodup_filter_mem_le xs (t.erase x) hxs
      have hlen := List.length_erase_of_mem h
      have hpos : 0 < t.length := List.length_pos_of_mem h
      rw [List.filter_cons, ite_eq_left (by simpa using h), List.length_cons, same]
      omega
    · rw [List.filter_cons, ite_eq_right (by simpa using h)]
      exact nodup_filter_mem_le xs t hxs

/-- Over ascending (duplicate-free) member rows, a seek takes at most
`|skip| + 1` steps, whatever the group's size. -/
theorem seek_steps_bounded (skip l : List Nat) (hl : l.Nodup) :
    (seek skip l).steps ≤ skip.length + 1 := by
  have := seek_steps_le_skipped skip l
  have := nodup_filter_mem_le l skip hl
  omega

/-- Members split into those leaving and those staying. -/
theorem residual_count (p : Nat × Bool → Bool) (leaving : List Nat) :
    ∀ (l : List (Nat × Bool)),
      (l.filter (fun m => decide (m.1 ∉ leaving))).countP p +
          (l.filter (fun m => decide (m.1 ∈ leaving))).countP p = l.countP p
  | [] => by simp
  | m :: ms => by
    have ih := residual_count p leaving ms
    by_cases h : m.1 ∈ leaving
    · rw [List.filter_cons_of_neg (by simp [h]), List.filter_cons_of_pos (by simp [h]),
        List.countP_cons, List.countP_cons]
      omega
    · rw [List.filter_cons_of_pos (by simp [h]), List.filter_cons_of_neg (by simp [h]),
        List.countP_cons, List.countP_cons]
      omega

/-- The residual's Boundary count from the stored group count and the
touched members alone: one step per touched member, one subtraction. -/
def residualBoundary (stored : Nat) (touched : List (Nat × Bool)) : TimeM Nat :=
  TimeM.bind (foldlStep (fun c (m : Nat × Bool) => c + if m.2 then 1 else 0) 0 touched)
    fun t => TimeM.tick (stored - t)

theorem fold_counts :
    ∀ (l : List (Nat × Bool)) (c : Nat),
      l.foldl (fun c (m : Nat × Bool) => c + if m.2 then 1 else 0) c = c + l.countP (·.2)
  | [], c => by simp
  | m :: ms, c => by
    rw [List.foldl_cons, fold_counts ms, List.countP_cons]
    cases m.2 <;> simp <;> omega

/-- The incremental count equals counting the materialised residual. -/
theorem residualBoundary_val (l : List (Nat × Bool)) (leaving : List Nat) :
    (residualBoundary (l.countP (·.2)) (l.filter (fun m => decide (m.1 ∈ leaving)))).val =
      (l.filter (fun m => decide (m.1 ∉ leaving))).countP (·.2) := by
  have := residual_count (·.2) leaving l
  simp only [residualBoundary, TimeM.bind_val, TimeM.tick_val, foldlStep_val, fold_counts]
  omega

theorem residualBoundary_steps (stored : Nat) (touched : List (Nat × Bool)) :
    (residualBoundary stored touched).steps = touched.length + 1 := by
  simp [residualBoundary]

/-- One scoped prior group: its ascending member names (any number) and
the members the commit touches. -/
structure ScopedGroup where
  members : List Nat
  touched : List Nat

/-- The authority's reads of one scoped group: its header, one role read
per touched member, and the residual's least member. -/
def groupReads (g : ScopedGroup) : TimeM (Option Nat) :=
  TimeM.bind (TimeM.tick ()) fun _ =>
    TimeM.bind (foldlStep (fun c (_ : Nat) => c + 1) 0 g.touched) fun _ =>
      seek g.touched g.members

/-- Every scoped group's reads, in scope order. -/
def scopeReads : List ScopedGroup → TimeM (List (Option Nat))
  | [] => ⟨[], 0⟩
  | g :: gs => TimeM.bind (groupReads g) fun least =>
      TimeM.bind (scopeReads gs) fun rest => ⟨least :: rest, 0⟩

/-- Naming a scope costs `2·|touched| + 2` steps per scoped group at most:
the bound names no group's member count, so it holds for every `M`. -/
theorem scope_steps_le :
    ∀ (gs : List ScopedGroup), (∀ g, g ∈ gs → g.members.Nodup) →
      (scopeReads gs).steps ≤ (gs.map (fun g => 2 * g.touched.length + 2)).sum
  | [], _ => by simp [scopeReads]
  | g :: gs, h => by
    have hg := seek_steps_bounded g.touched g.members (h g (by simp))
    have ih := scope_steps_le gs (fun g' hg' => h g' (by simp [hg']))
    simp only [scopeReads, TimeM.bind_steps, groupReads, TimeM.tick_steps, foldlStep_steps,
      List.map_cons, List.sum_cons]
    omega

end CanonicalGroupDelta
