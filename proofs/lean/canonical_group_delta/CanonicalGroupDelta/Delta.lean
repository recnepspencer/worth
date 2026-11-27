import CanonicalGroupDelta.TimeM

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/group_install.rs:56-115 sha256:5aabe6e947e31cb375df580d67dad724583139f2ad0b1f0602445d914249cf82
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/group_install.rs:117-135 sha256:bf89330e2529e06eef917c15388ce3cc3a7c537afec578e4caa389f73277ba6b
-- proof-mirror: crates/worth-relational-contracts/src/aggregate_identity/transition/group/transition_result.rs:42-61 sha256:c6f10de59f81f0203d633fbb6991008c0687d71362b16aa19be112217e05d0f3
-- proof-mirror: crates/worth-relational-lineage/src/durability/naming/group_frame_encoding.rs:125-161 sha256:ae148932a87d1d1a47fade75fb7ab73d829fa8024ca8566d0101dd3beb8eb831
-- proof-mirror: crates/worth-relational-lineage/src/durability/naming/group_frame_encoding.rs:163-178 sha256:3569bf941924a055d8c1e85f6987c8160dd08b72a81dcca5b427c45c0b46b18c
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/branch_state.rs:314-342 sha256:1299b2237583a255b1c946e481ef7a88843c4f028fc3135154e06ddf10244441

/-!
# Kept-group member deltas: step bound, wire bound and replay equivalence

The model of `crates/worth-relational-lineage/src/lineage/groups/
group_install.rs` (`SchemeGroups::install` on `AggregateGroupChange::Kept`),
`transition_result.rs::member_delta` and the WPGL v2 kept body of
`durability/naming/group_frame_encoding.rs`.

A kept group's membership is a function from member name to its carrier and
role (`Members α`); the group may hold any number `M` of members. Its
canonical delta lists the members that left and those that joined or changed.

* `applyDelta_steps`: installing a delta takes `|removed| + |upserted| + 1`
  steps, a count in which no member count `M` appears (`∀ M`).
* `apply_diff`: installing the canonical delta between a group's complete
  prior and next memberships yields exactly the next membership: the delta
  path and the full-replacement path agree.
* `replay_eq_full`: replaying a history of deltas reaches the same state as
  replaying the history's complete memberships (`GROUP_LOG` replay).
* `diff_size`, `encode_diff_le_touched`: the delta and its wire bytes are
  bounded by the members whose place changed, never by the group's size.
-/
namespace CanonicalGroupDelta

/-- One group's members: name to its carrier and role. -/
abbrev Members (α : Type) := Nat → Option α

/-- A kept group's member delta. -/
structure Delta (α : Type) where
  removed : List Nat
  upserted : List (Nat × α)

def removeOne {α : Type} (m : Members α) (n : Nat) : Members α :=
  fun x => if x = n then none else m x

def upsertOne {α : Type} (p : Nat × α) (m : Members α) : Members α :=
  fun x => if x = p.1 then some p.2 else m x

/-- Installs a kept group's delta: one step per removed member, one per
upserted member and one for the group's header. -/
def applyDelta {α : Type} (m : Members α) (d : Delta α) : TimeM (Members α) :=
  TimeM.bind (foldlStep removeOne m d.removed) fun m' =>
    TimeM.bind (foldrStep upsertOne m' d.upserted) fun m'' => TimeM.tick m''

/-- The install's step count names only the delta: no member count. -/
theorem applyDelta_steps {α : Type} (m : Members α) (d : Delta α) :
    (applyDelta m d).steps = d.removed.length + d.upserted.length + 1 := by
  simp [applyDelta]; omega

theorem applyDelta_steps_independent {α : Type} (m m' : Members α) (d : Delta α) :
    (applyDelta m d).steps = (applyDelta m' d).steps := by
  simp [applyDelta_steps]

theorem removes_apply {α : Type} :
    ∀ (l : List Nat) (m : Members α) (x : Nat),
      (l.foldl removeOne m) x = if x ∈ l then none else m x
  | [], _, _ => by simp
  | a :: as, m, x => by
    rw [List.foldl_cons, removes_apply as (removeOne m a) x]
    by_cases h : x = a <;> simp [removeOne, h]

theorem upserts_apply {α : Type} :
    ∀ (l : List (Nat × α)) (m : Members α) (x : Nat),
      (l.foldr upsertOne m) x = match l.lookup x with
        | some v => some v
        | none => m x
  | [], _, _ => by simp
  | (k, v) :: as, m, x => by
    rw [List.foldr_cons, List.lookup_cons]
    by_cases h : x = k
    · simp [upsertOne, h]
    · have hb : (x == k) = false := by simp [h]
      simp only [upsertOne, h, ite_false, hb]
      exact upserts_apply as m x

theorem applyDelta_val {α : Type} (m : Members α) (d : Delta α) (x : Nat) :
    (applyDelta m d).val x = match d.upserted.lookup x with
      | some v => some v
      | none => if x ∈ d.removed then none else m x := by
  simp only [applyDelta, TimeM.bind_val, TimeM.tick_val, foldrStep_val, foldlStep_val]
  rw [upserts_apply, removes_apply]

variable {α : Type} [DecidableEq α]

/-- The member `x` holds next, when its place changes and it stays. -/
def upd (prior next : Members α) (x : Nat) : Option α :=
  match next x with
  | some v => if prior x = some v then none else some v
  | none => none

/-- The canonical delta from `prior` to `next` over the names `support`. -/
def diff (prior next : Members α) (support : List Nat) : Delta α where
  removed := support.filter (fun x => (prior x).isSome && (next x).isNone)
  upserted := support.filterMap (fun x => (upd prior next x).map (fun v => (x, v)))

theorem lookup_upserted (prior next : Members α) :
    ∀ (support : List Nat) (x : Nat),
      (support.filterMap (fun y => (upd prior next y).map (fun v => (y, v)))).lookup x =
        if x ∈ support then upd prior next x else none
  | [], _ => by simp
  | y :: ys, x => by
    have ih := lookup_upserted prior next ys x
    cases hy : upd prior next y with
    | none =>
      rw [List.filterMap_cons, hy]
      simp only [Option.map_none]
      rw [ih]
      by_cases h : x = y
      · subst h; simp [hy]
      · simp [h]
    | some v =>
      rw [List.filterMap_cons, hy]
      simp only [Option.map_some, List.lookup_cons]
      by_cases h : x = y
      · subst h; simp [hy]
      · have hb : (x == y) = false := by simp [h]
        simp only [hb]
        rw [ih]
        simp [h]

/-- The delta path equals the full-replacement path. -/
theorem apply_diff (prior next : Members α) (support : List Nat)
    (cover : ∀ x, x ∉ support → prior x = none ∧ next x = none) :
    (applyDelta prior (diff prior next support)).val = next := by
  funext x
  rw [applyDelta_val]
  simp only [diff]
  rw [lookup_upserted]
  by_cases hs : x ∈ support
  · simp only [hs, ite_true, List.mem_filter, true_and]
    unfold upd
    cases hn : next x with
    | none => cases hp : prior x <;> simp
    | some v =>
      by_cases hp : prior x = some v
      · simp [hp]
      · simp [hp]
  · obtain ⟨hp, hn⟩ := cover x hs
    simp [hs, hp, hn]

/-- Replays a group history by its canonical deltas. -/
def replayDelta (m : Members α) : List (Members α × List Nat) → TimeM (Members α)
  | [] => ⟨m, 0⟩
  | (n, s) :: rest => TimeM.bind (applyDelta m (diff m n s)) (fun m' => replayDelta m' rest)

/-- Replays a group history by its complete memberships. -/
def replayFull (m : Members α) : List (Members α × List Nat) → Members α
  | [] => m
  | (n, _) :: rest => replayFull n rest

/-- Every step's support names every member its prior or next state holds. -/
def Covered : Members α → List (Members α × List Nat) → Prop
  | _, [] => True
  | m, (n, s) :: rest => (∀ x, x ∉ s → m x = none ∧ n x = none) ∧ Covered n rest

/-- `GROUP_LOG` replay by deltas reaches the full-membership replay's state. -/
theorem replay_eq_full :
    ∀ (m : Members α) (hs : List (Members α × List Nat)),
      Covered m hs → (replayDelta m hs).val = replayFull m hs
  | _, [], _ => rfl
  | m, (n, s) :: rest, ⟨hc, hrest⟩ => by
    simp only [replayDelta, TimeM.bind_val, replayFull]
    rw [apply_diff m n s hc]
    exact replay_eq_full n rest hrest

/-- The members whose place `prior` to `next` changes. -/
def touched (prior next : Members α) (support : List Nat) : List Nat :=
  support.filter (fun x => decide (prior x ≠ next x))

/-- A canonical delta lists exactly the members whose place changed. -/
theorem diff_size (prior next : Members α) :
    ∀ (support : List Nat),
      (diff prior next support).removed.length + (diff prior next support).upserted.length =
        (touched prior next support).length
  | [] => by simp [diff, touched]
  | y :: ys => by
    have ih := diff_size prior next ys
    simp only [diff, touched] at ih
    simp only [diff, touched]
    rw [List.filter_cons, List.filterMap_cons, List.filter_cons]
    have key :
        (((prior y).isSome && (next y).isNone) = true ∧ upd prior next y = none ∧
            prior y ≠ next y) ∨
        (((prior y).isSome && (next y).isNone) = false ∧ (∃ v, upd prior next y = some v) ∧
            prior y ≠ next y) ∨
        (((prior y).isSome && (next y).isNone) = false ∧ upd prior next y = none ∧
            prior y = next y) := by
      unfold upd
      cases hn : next y with
      | none => cases hp : prior y <;> simp
      | some v =>
        by_cases hp : prior y = some v
        · simp [hp]
        · cases hq : prior y <;> simp_all
    rcases key with ⟨h1, h2, h3⟩ | ⟨h1, ⟨v, h2⟩, h3⟩ | ⟨h1, h2, h3⟩
    · rw [ite_eq_left h1, h2, ite_eq_left (decide_eq_true h3)]
      simp only [Option.map_none, List.length_cons]
      omega
    · rw [ite_eq_right (by simp [h1]), h2, ite_eq_left (decide_eq_true h3)]
      simp only [Option.map_some, List.length_cons]
      omega
    · rw [ite_eq_right (by simp [h1]), h2, ite_eq_right (by simp [h3])]
      simp only [Option.map_none]
      omega

/-- One step per changed member plus one per commit. -/
def replayCost : Members α → List (Members α × List Nat) → Nat
  | _, [] => 0
  | m, (n, s) :: rest => (touched m n s).length + 1 + replayCost (replayFull m [(n, s)]) rest

/-- Replaying a delta history costs one step per changed member plus one
per commit: never a group's untouched members. -/
theorem replay_steps :
    ∀ (m : Members α) (hs : List (Members α × List Nat)),
      Covered m hs → (replayDelta m hs).steps = replayCost m hs
  | _, [], _ => rfl
  | m, (n, s) :: rest, ⟨hc, hrest⟩ => by
    simp only [replayDelta, TimeM.bind_steps, applyDelta_steps, replayCost, replayFull]
    rw [apply_diff m n s hc, replay_steps n rest hrest, diff_size]

omit [DecidableEq α] in
/-- Big-endian four-byte count. -/
def u32 (n : Nat) : List Nat := [n / 2 ^ 24 % 256, n / 2 ^ 16 % 256, n / 2 ^ 8 % 256, n % 256]

omit [DecidableEq α] in
/-- WPGL v2 kept body: removed names, then upserted members. -/
def encode (name : Nat → List Nat) (member : Nat × α → List Nat) (d : Delta α) : List Nat :=
  u32 d.removed.length ++ d.removed.flatMap name ++ u32 d.upserted.length ++
    d.upserted.flatMap member

omit [DecidableEq α] in
theorem flatMap_length_const {β : Type} (f : β → List Nat) (w : Nat)
    (hf : ∀ b, (f b).length = w) : ∀ (l : List β), (l.flatMap f).length = w * l.length
  | [] => by simp
  | b :: bs => by
    rw [List.flatMap_cons, List.length_append, hf, flatMap_length_const f w hf bs]
    simp [Nat.mul_succ]; omega

omit [DecidableEq α] in
/-- A kept body's bytes: eight count bytes, 32 per removed name and `w`
per upserted member. -/
theorem encode_length (name : Nat → List Nat) (member : Nat × α → List Nat) (w : Nat)
    (hn : ∀ x, (name x).length = 32) (hm : ∀ p, (member p).length = w) (d : Delta α) :
    (encode name member d).length = 8 + 32 * d.removed.length + w * d.upserted.length := by
  simp only [encode, List.length_append, u32, List.length_cons, List.length_nil,
    flatMap_length_const name 32 hn, flatMap_length_const member w hm]
  omega

/-- The canonical delta's bytes are bounded by the changed members alone:
whatever the group's size `M`, a one-member edit encodes `8 + (32 + w)`
bytes at most. -/
theorem encode_diff_le_touched (name : Nat → List Nat) (member : Nat × α → List Nat) (w : Nat)
    (hn : ∀ x, (name x).length = 32) (hm : ∀ p, (member p).length = w)
    (prior next : Members α) (support : List Nat) :
    (encode name member (diff prior next support)).length ≤
      8 + (32 + w) * (touched prior next support).length := by
  rw [encode_length name member w hn hm, ← diff_size prior next support, Nat.add_mul]
  have h1 : w * (diff prior next support).upserted.length ≤
      w * (diff prior next support).upserted.length + 32 * 0 := by omega
  have := Nat.mul_le_mul_left 32 (Nat.le_add_right (diff prior next support).removed.length
    (diff prior next support).upserted.length)
  have := Nat.mul_le_mul_left w (Nat.le_add_left (diff prior next support).upserted.length
    (diff prior next support).removed.length)
  omega

end CanonicalGroupDelta
