-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:13-18 sha256:c695fad3216b679b2baddf9ed1543e36c5ea3de5e447c0bb99e119037d7c2ba5
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:78-82 sha256:3e8f691aafa4c65343eec297dcb2d7d9863e762cec1f4935e66fa9f3c0dd1992
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:87-102 sha256:bdd8cbf1c3a03b493f179947a53785ca15bc7daa3ad7f34e74956062aa3171fe
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:106-115 sha256:a597d51a25d99e7725859501dff098320a19c3371381e8f2a235a9725adae19d
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:118-170 sha256:5737411d85e62fabacb05e64139153946a878a72bf37aaacc401425cef8ec46e
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/navigation.rs:154-168 sha256:a92f80c3782f4b668b81f1b6aca203ee37b7cc731dcde8ebb0c12a753baa2306
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:112-128 sha256:5472342abee3493323b9c10195ef05a6749216a757c87171f44336573a3529dd
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:143-144 sha256:7b79964869e5420cbe9abcdbd143672cef4d2e9d5aede269939aafad911b22ee
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/journal/sources.rs:182-200 sha256:0facd6e366a8d784c8c4aa94813ccaa7ac1066a24acd5b88e3d7cf0a450718bc
/-
The navigation supersession rule for a correction's restored support, unbounded.

Mirrors (TBU Phase 2, `6a420909dc`; supersession keeps line writes, public #453)
  workspaces/worth-query/crates/worth-query-execution/src/domain_computation/
  primary_graph/product_operation/correction_source/restored_support/navigation.rs
    * `OccurrenceKind`               navigation.rs:78-82  -> `Kind` (transliteration)
    * `superseded_through`           navigation.rs:87-102 -> `supersededThrough`
      (transliteration: filter_map to navigations, `max_by_key` on the commit,
      which keeps the LAST maximum, then the vouched basis kept beside the
      commit: `Option<(through, selected)>`)
    * `needs_witness`                navigation.rs:106-115 -> `needsWitness` (verbatim;
      `on_line` is the lineage's `wrote_on_line_through`, a parameter `onLine`)
    * `without_vouched_navigation`   navigation.rs:118-170 -> `kept` (the filter
      loop at navigation.rs:154-168; envelope reading is abstracted into `Occ`)
  and the vouch the journal answers,
  document_history/journal/sources.rs:112-128 (`restores_descendant_of`)
    -> `vouch` (abstraction: the two index lookups are given as the source step
       `S` and the basis's step `b`; the lift is parent-chain ancestry, proved
       equal to the code's loop in JumpLift.lean / jump_pointer_lift.rs).
  The journal's `wrote_on_line_through` (sources.rs:182-200) is left abstract as
  `onLine`: the theorems take what it answers as a hypothesis
  (`kept_line_writes`) and show which writes it must name
  (`since_change_is_a_line_write`). The kept loop's lookup of each
  occurrence's kind is total here (every occurrence has its kind); the code
  keeps an occurrence whose kind is missing.

The relational side is an abstraction: facts are keys `Nat`, a fact's support
is a record id `Nat`, an application commit overlays some keys, and a
navigation commit sets every key of the owned closure to the recorded state of
the step it selected ("a navigation restores the full recorded state of the
owned closure, and the original bases retain their payloads",
navigation.rs:13-18). The occurrence map is ordered by commit id (an
`ExecutionOrderedMap<CommitId, _>`), modelled as a list sorted strictly
increasing by id.

Results
  * `superseded_split`     the decision cancels a prefix that ends at the
    latest navigation, except the prefix's line writes, keeps exactly the
    suffix after it, and that suffix holds no navigation; the superseding
    navigation was vouched.
  * `kept_line_writes`     every write `onLine` names stays kept.
  * `kept_monotone`        the rule keeps everything the old rule kept.
  * `since_change_is_a_line_write`  a change at any step between the source
    and the selected basis was written by a step on the path from the source
    to that basis: a line write, so kept.
  * `old_rule_hides_a_line_write`  the old rule's counterexample (#453): it
    cancelled two line writes that the final comparison cannot see.
  * `cancelled_effect_is_restored_state`  soundness: the effect of the
    cancelled set on every owned fact IS the restored recorded state; the
    cancelled commits cannot influence the facts at the target.
  * `target_fact`          the target's owned fact is the restored recorded
    state followed by the suffix after the superseding navigation, which the
    kept set ends with.
  * `restored_differs_only_by_line_writes`  a write that makes the restored
    state differ from the source's is a recorded step on the selected line
    between the source and the selected basis, so it is in the state the
    final comparison (`eligibility::compare`) reads; nothing is hidden.
  * `fresh_fact_forces_descent`  the final comparison alone already implies
    the "descends from the source" half of the vouch whenever the source
    wrote one compared fact with a fresh record (why the ancestry-drop
    mutant survives in courts).
  * `nondescending_checkout_does_not_vouch`, `unvouched_keeps_everything`.
  * `outside_closure_counterexample`  the owned-closure premise is necessary:
    a cancelled write to a fact outside the owned closure does reach the
    target; there only the final comparison guards.
-/
set_option linter.deprecated false
set_option linter.unusedVariables false

namespace Supersession

/-! ## The unbounded history tree and its selected line -/

structure Tree where
  par : Nat → Nat
  par_root : par 0 = 0
  par_lt : ∀ n, 0 < n → par n < n

variable (T : Tree)

/-- Follow the parent link k times. -/
def up : Nat → Nat → Nat
  | 0, n => n
  | k + 1, n => up k (T.par n)

/-- `a` is `n` or one of its ancestors (sources.rs:143-144). -/
def AncOrSelf (a n : Nat) : Prop := ∃ k, up T k n = a

theorem anc_refl (n : Nat) : AncOrSelf T n n := ⟨0, rfl⟩

theorem anc_par (a n : Nat) (h : AncOrSelf T a (T.par n)) : AncOrSelf T a n := by
  obtain ⟨k, hk⟩ := h; exact ⟨k + 1, hk⟩

theorem up_root : ∀ k, up T k 0 = 0
  | 0 => rfl
  | k + 1 => by simp only [up]; rw [T.par_root]; exact up_root k

theorem anc_root (n : Nat) : AncOrSelf T 0 n := by
  induction n using Nat.strongRecOn with
  | _ n ih =>
  by_cases h : n = 0
  · subst h; exact anc_refl T 0
  · exact anc_par T 0 n (ih _ (T.par_lt n (by omega)))

/-- `up` never climbs above a smaller ordinal than it started from. -/
theorem up_le : ∀ k n, up T k n ≤ n
  | 0, n => Nat.le_refl n
  | k + 1, n => by
    simp only [up]
    have := up_le k (T.par n)
    by_cases h : n = 0
    · subst h; rw [T.par_root] at this ⊢; omega
    · have := T.par_lt n (by omega); omega

/-- The recorded state of a step: its writes over its parent's recorded state
(the history of recorded steps is a tree; `init` is the root's state). -/
def recorded (init : Nat → Nat) (wr : Nat → Nat → Option Nat) (n : Nat) (k : Nat) : Nat :=
  if h : n = 0 then init k else (wr n k).getD (recorded init wr (T.par n) k)
termination_by n
decreasing_by exact T.par_lt n (by omega)

theorem recorded_succ (init : Nat → Nat) (wr : Nat → Nat → Option Nat) {n : Nat} (h : n ≠ 0)
    (k : Nat) : recorded T init wr n k = (wr n k).getD (recorded T init wr (T.par n) k) := by
  rw [recorded, dif_neg h]

/-! ## Occurrences and the code's decision -/

/-- An occurrence as the rule sees it (navigation.rs:78-82). -/
inductive Kind where
  | navigation (selected : Option Nat)
  | other
  deriving DecidableEq

/-- An occurrence with its effect on the facts: a navigation commit (an
owned-graph selection naming the basis it selected) or any other write. -/
inductive Occ where
  | navigation (selected : Option Nat)
  | write (f : Nat → Option Nat)

def Occ.kind : Occ → Kind
  | .navigation s => .navigation s
  | .write _ => .other

/-- `.filter_map(|(commit, kind)| match kind { Navigation{selected} => Some((commit, selected)), Other => None })` -/
def navs : List (Nat × Occ) → List (Nat × Option Nat)
  | [] => []
  | (c, o) :: l =>
    match o.kind with
    | .navigation s => (c, s) :: navs l
    | .other => navs l

/-- `Iterator::max_by_key(|(commit, _)| *commit)`: the last maximum wins. -/
def maxStep (acc x : Nat × Option Nat) : Nat × Option Nat := if acc.1 ≤ x.1 then x else acc

def maxByKey : List (Nat × Option Nat) → Option (Nat × Option Nat)
  | [] => none
  | x :: l => some (l.foldl maxStep x)

/-- `superseded_through` (navigation.rs): the latest navigation's commit and
the basis it selected, when the vouch accepts that basis. -/
def supersededThrough (l : List (Nat × Occ)) (vouch : Nat → Bool) : Option (Nat × Nat) :=
  match maxByKey (navs l) with
  | some (c, some b) => if vouch b then some (c, b) else none
  | _ => none

/-- `needs_witness` (navigation.rs), verbatim: an occurrence after the latest
vouched navigation, or a write the journal answers is a recorded step's on the
path from the source to the selected basis (`onLine`, the abstract
`wrote_on_line_through`), still needs its witness. -/
def needsWitness (commit : Nat) (kind : Kind) (superseded : Option (Nat × Nat))
    (onLine : Nat → Nat → Bool) : Bool :=
  match superseded with
  | none => true
  | some (through, selected) => decide (commit > through) || (decide (kind = .other) && onLine commit selected)

/-- The occurrences `without_vouched_navigation` keeps (its filter loop). -/
def kept (l : List (Nat × Occ)) (vouch : Nat → Bool) (onLine : Nat → Nat → Bool) : List (Nat × Occ) :=
  l.filter (fun x => needsWitness x.1 x.2.kind (supersededThrough l vouch) onLine)

/-- The rule before public #453: everything up to the latest vouched navigation
was cancelled, line writes included. -/
def keptOld (l : List (Nat × Occ)) (vouch : Nat → Bool) : List (Nat × Occ) :=
  l.filter (fun x => match supersededThrough l vouch with
    | none => true
    | some (through, _) => decide (x.1 > through))

/-- The journal's vouch (sources.rs:112-128), over the index lookups' answers:
the source step `S` (never the root) is an ancestor-or-self of the selected
basis's step `b`, which is on the selected line (an ancestor-or-self of
`current`). -/
def Vouch (S current b : Nat) : Prop := S ≠ 0 ∧ AncOrSelf T S b ∧ AncOrSelf T b current

/-- The occurrence map iterates in strictly increasing commit order. -/
def Sorted (l : List (Nat × Occ)) : Prop := l.Pairwise (fun x y => x.1 < y.1)

/-! ## The decision cancels a prefix ending at the latest navigation -/

def lastNav : List (Nat × Occ) → Option (Nat × Option Nat)
  | [] => none
  | (c, o) :: l =>
    match lastNav l with
    | some r => some r
    | none => match o.kind with
      | .navigation s => some (c, s)
      | .other => none

theorem foldl_maxStep_sorted : ∀ (l : List (Nat × Option Nat)) (acc : Nat × Option Nat),
    (∀ y ∈ l, acc.1 < y.1) → l.Pairwise (fun x y => x.1 < y.1) →
    l.foldl maxStep acc = (acc :: l).getLast (by simp)
  | [], acc, _, _ => by simp
  | y :: l, acc, h, hp => by
    simp only [List.foldl]
    have hy : acc.1 ≤ y.1 := Nat.le_of_lt (h y (by simp))
    rw [show maxStep acc y = y by simp [maxStep, hy]]
    have hp' := List.pairwise_cons.mp hp
    rw [foldl_maxStep_sorted l y hp'.1 hp'.2]
    simp

theorem navs_sub : ∀ (l : List (Nat × Occ)) (x : Nat × Option Nat), x ∈ navs l → ∃ o, (x.1, o) ∈ l
  | [], x, h => by simp [navs] at h
  | (c, o) :: l, x, h => by
    unfold navs at h
    split at h
    · rcases List.mem_cons.mp h with h | h
      · subst h; exact ⟨o, by simp⟩
      · obtain ⟨o', ho⟩ := navs_sub l x h; exact ⟨o', by simp [ho]⟩
    · obtain ⟨o', ho⟩ := navs_sub l x h; exact ⟨o', by simp [ho]⟩

theorem navs_sorted : ∀ l : List (Nat × Occ), Sorted l → (navs l).Pairwise (fun x y => x.1 < y.1)
  | [], _ => by simp [navs]
  | (c, o) :: l, h => by
    have h' := List.pairwise_cons.mp h
    unfold navs
    split
    · refine List.pairwise_cons.mpr ⟨?_, navs_sorted l h'.2⟩
      intro y hy
      obtain ⟨o', ho⟩ := navs_sub l y hy
      exact h'.1 _ ho
    · exact navs_sorted l h'.2

theorem getLast_cons_eq_lastNav : ∀ (l : List (Nat × Occ)),
    (match navs l with | [] => none | x :: r => some ((x :: r).getLast (by simp))) = lastNav l
  | [] => by simp [navs, lastNav]
  | (c, o) :: l => by
    have ih := getLast_cons_eq_lastNav l
    unfold navs lastNav
    cases hk : o.kind with
    | other => simp only; rw [ih]; cases lastNav l <;> rfl
    | navigation s =>
      simp only
      cases hn : navs l with
      | nil =>
        rw [hn] at ih; simp only at ih; rw [← ih]; simp
      | cons x r =>
        rw [hn] at ih; simp only at ih; rw [← ih]
        simp [List.getLast_cons]

theorem maxByKey_eq_lastNav (l : List (Nat × Occ)) (h : Sorted l) :
    maxByKey (navs l) = lastNav l := by
  rw [← getLast_cons_eq_lastNav l]
  have hs := navs_sorted l h
  cases hn : navs l with
  | nil => rfl
  | cons x r =>
    rw [hn] at hs
    have hp := List.pairwise_cons.mp hs
    simp only [maxByKey]
    rw [foldl_maxStep_sorted r x hp.1 hp.2]

theorem lastNav_split : ∀ (l : List (Nat × Occ)) (c : Nat) (s : Option Nat),
    lastNav l = some (c, s) →
    ∃ pre post, l = pre ++ (c, .navigation s) :: post ∧ lastNav post = none
  | [], c, s, h => by simp [lastNav] at h
  | (c', o) :: l, c, s, h => by
    unfold lastNav at h
    cases hl : lastNav l with
    | some r =>
      rw [hl] at h; simp only at h
      obtain ⟨pre, post, he, hp⟩ := lastNav_split l c s (by rw [hl, h])
      exact ⟨(c', o) :: pre, post, by rw [he]; rfl, hp⟩
    | none =>
      rw [hl] at h; simp only at h
      cases o with
      | write f => simp [Occ.kind] at h
      | navigation s' =>
        simp only [Occ.kind, Option.some.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        exact ⟨[], l, rfl, hl⟩

theorem lastNav_none_no_navigation : ∀ (l : List (Nat × Occ)), lastNav l = none →
    ∀ x ∈ l, x.2.kind = .other
  | [], _, x, hx => by simp at hx
  | (c, o) :: l, h, x, hx => by
    unfold lastNav at h
    cases hl : lastNav l with
    | some r => rw [hl] at h; simp at h
    | none =>
      rw [hl] at h; simp only at h
      rcases List.mem_cons.mp hx with hx | hx
      · subst hx
        cases hk : o.kind with
        | other => rfl
        | navigation s => rw [hk] at h; simp at h
      · exact lastNav_none_no_navigation l hl x hx

/-- The code's decision, whenever it supersedes anything: the latest
navigation was vouched, the suffix after it holds no navigation, and the kept
set is the prefix's line writes followed by that whole suffix. -/
theorem superseded_split (l : List (Nat × Occ)) (vouch : Nat → Bool) (onLine : Nat → Nat → Bool)
    (c b : Nat) (hs : Sorted l) (h : supersededThrough l vouch = some (c, b)) :
    ∃ pre post, l = pre ++ (c, .navigation (some b)) :: post ∧ vouch b = true ∧
      (∀ x ∈ post, x.2.kind = .other) ∧
      kept l vouch onLine = pre.filter (fun x => decide (x.2.kind = .other) && onLine x.1 b) ++ post := by
  have h' := h
  unfold supersededThrough at h
  rw [maxByKey_eq_lastNav l hs] at h
  cases hl : lastNav l with
  | none => rw [hl] at h; simp at h
  | some r =>
    rw [hl] at h
    obtain ⟨c', s⟩ := r
    cases s with
    | none => simp at h
    | some b' =>
      simp only at h
      by_cases hv : vouch b' = true
      · rw [if_pos hv] at h
        simp only [Option.some.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        obtain ⟨pre, post, he, hp⟩ := lastNav_split l c' (some b') hl
        refine ⟨pre, post, he, hv, lastNav_none_no_navigation post hp, ?_⟩
        unfold kept
        rw [h']
        rw [he] at hs ⊢
        unfold Sorted at hs
        rw [List.pairwise_append] at hs
        obtain ⟨_, hpost, hcross⟩ := hs
        have hpost' := List.pairwise_cons.mp hpost
        rw [List.filter_append, List.filter_cons]
        have hpre : pre.filter (fun x => needsWitness x.1 x.2.kind (some (c', b')) onLine) =
            pre.filter (fun x => decide (x.2.kind = .other) && onLine x.1 b') := by
          apply List.filter_congr
          intro x hx
          have := hcross x hx (c', .navigation (some b')) (by simp)
          have hgt : decide (x.1 > c') = false := by simp; omega
          simp only [needsWitness, hgt, Bool.false_or]
        have hpost2 : post.filter (fun x => needsWitness x.1 x.2.kind (some (c', b')) onLine) = post := by
          rw [List.filter_eq_self]
          intro x hx
          have := hpost'.1 x hx
          simp [needsWitness]; omega
        rw [hpre, hpost2]
        simp [needsWitness, Occ.kind]
      · simp [hv] at h

/-- With no vouched navigation, every occurrence still needs its witness. -/
theorem unvouched_keeps_everything (l : List (Nat × Occ)) (vouch : Nat → Bool)
    (onLine : Nat → Nat → Bool) (h : supersededThrough l vouch = none) : kept l vouch onLine = l := by
  unfold kept; rw [h]; simp [needsWitness]

/-- Every write the journal answers is a line write of the vouched path is
kept, wherever it falls. -/
theorem kept_line_writes (l : List (Nat × Occ)) (vouch : Nat → Bool) (onLine : Nat → Nat → Bool)
    (x : Nat × Occ) (hx : x ∈ l) (hk : x.2.kind = .other)
    (hline : ∀ c b, supersededThrough l vouch = some (c, b) → onLine x.1 b = true) :
    x ∈ kept l vouch onLine := by
  unfold kept
  rw [List.mem_filter]
  refine ⟨hx, ?_⟩
  cases hsup : supersededThrough l vouch with
  | none => rfl
  | some p =>
    obtain ⟨c, b⟩ := p
    simp [needsWitness, hk, hline c b hsup]

/-- The new rule keeps everything the old one kept: every refusal before
public #453 is still a refusal. -/
theorem kept_monotone (l : List (Nat × Occ)) (vouch : Nat → Bool) (onLine : Nat → Nat → Bool) :
    ∀ x ∈ keptOld l vouch, x ∈ kept l vouch onLine := by
  intro x hx
  unfold keptOld at hx
  unfold kept
  rw [List.mem_filter] at hx ⊢
  refine ⟨hx.1, ?_⟩
  have h2 := hx.2
  cases hsup : supersededThrough l vouch with
  | none => rfl
  | some p =>
    obtain ⟨c, b⟩ := p
    rw [hsup] at h2
    simp only [decide_eq_true_eq] at h2
    simp [needsWitness, h2]

/-! ## Soundness: what the cancelled occurrences can and cannot affect -/

section State

variable (owned : Nat → Bool) (init : Nat → Nat) (wr : Nat → Nat → Option Nat)
  (emptyState : Nat → Nat)

/-- One occurrence's effect on the facts' supports. -/
def apply (σ : Nat → Nat) : Occ → (Nat → Nat)
  | .write f => fun k => (f k).getD (σ k)
  | .navigation (some b) => fun k => if owned k then recorded T init wr b k else σ k
  | .navigation none => fun k => if owned k then emptyState k else σ k

def run (σ : Nat → Nat) (l : List (Nat × Occ)) : Nat → Nat :=
  l.foldl (fun σ x => apply T owned init wr emptyState σ x.2) σ

theorem run_append (σ : Nat → Nat) (l₁ l₂ : List (Nat × Occ)) :
    run T owned init wr emptyState σ (l₁ ++ l₂) =
      run T owned init wr emptyState (run T owned init wr emptyState σ l₁) l₂ := by
  simp [run, List.foldl_append]

theorem cons_split {α : Type} (pre post : List α) (x : α) : pre ++ x :: post = (pre ++ [x]) ++ post := by
  simp

/-- The kept writes at one fact. -/
def runFact (v : Nat) (k : Nat) : List (Nat × Occ) → Nat
  | [] => v
  | (_, .write f) :: l => runFact ((f k).getD v) k l
  | (_, .navigation _) :: l => runFact v k l

theorem run_writes (k : Nat) : ∀ (post : List (Nat × Occ)) (σ : Nat → Nat),
    (∀ x ∈ post, x.2.kind = .other) →
    run T owned init wr emptyState σ post k = runFact (σ k) k post
  | [], σ, _ => rfl
  | (c, o) :: l, σ, h => by
    cases o with
    | navigation s => have := h (c, .navigation s) (by simp); simp [Occ.kind] at this
    | write f =>
      simp only [run, List.foldl] at *
      have := run_writes k l (apply T owned init wr emptyState σ (.write f))
        (fun x hx => h x (by simp [hx]))
      simp only [run] at this
      rw [this]
      rfl

/-- **Soundness.** The effect of the cancelled set (everything up to and
including the latest vouched navigation) on every owned fact is exactly the
recorded state of the basis that navigation restored, whatever the cancelled
commits wrote and whatever the state before them was. -/
theorem cancelled_effect_is_restored_state (pre : List (Nat × Occ)) (c b : Nat)
    (σ : Nat → Nat) (k : Nat) (hk : owned k = true) :
    run T owned init wr emptyState σ (pre ++ [(c, .navigation (some b))]) k =
      recorded T init wr b k := by
  simp [run, List.foldl_append, apply, hk]

theorem run_through_navigation (pre post : List (Nat × Occ)) (c b : Nat) (σ : Nat → Nat)
    (hpost : ∀ x ∈ post, x.2.kind = .other) (k : Nat) (hk : owned k = true) :
    run T owned init wr emptyState σ (pre ++ (c, .navigation (some b)) :: post) k =
      runFact (recorded T init wr b k) k post := by
  rw [cons_split pre post, run_append T owned init wr emptyState σ (pre ++ _) post,
    run_writes T owned init wr emptyState k post _ hpost,
    cancelled_effect_is_restored_state T owned init wr emptyState pre c b σ k hk]

/-- The target's owned facts are the restored recorded state followed by the
suffix after the latest vouched navigation, which the kept set ends with: no
occurrence before that navigation reaches them. -/
theorem target_fact (l : List (Nat × Occ)) (vouch : Nat → Bool) (onLine : Nat → Nat → Bool)
    (c b : Nat) (hs : Sorted l) (h : supersededThrough l vouch = some (c, b))
    (σ : Nat → Nat) (k : Nat) (hk : owned k = true) :
    vouch b = true ∧ ∃ pre post, l = pre ++ (c, .navigation (some b)) :: post ∧
      kept l vouch onLine = pre.filter (fun x => decide (x.2.kind = .other) && onLine x.1 b) ++ post ∧
      run T owned init wr emptyState σ l k = runFact (recorded T init wr b k) k post := by
  obtain ⟨pre, post, he, hv, hpost, hkept⟩ := superseded_split l vouch onLine c b hs h
  refine ⟨hv, pre, post, he, hkept, ?_⟩
  rw [he, run_through_navigation T owned init wr emptyState pre post c b σ hpost k hk]

/-- Two runs that differ only in their cancelled prefixes (and starting
states) agree on every owned fact at the target. -/
theorem cancelled_never_reaches_target (pre₁ pre₂ post : List (Nat × Occ)) (c b : Nat)
    (σ₁ σ₂ : Nat → Nat) (hpost : ∀ x ∈ post, x.2.kind = .other) (k : Nat) (hk : owned k = true) :
    run T owned init wr emptyState σ₁ (pre₁ ++ (c, .navigation (some b)) :: post) k =
      run T owned init wr emptyState σ₂ (pre₂ ++ (c, .navigation (some b)) :: post) k := by
  rw [run_through_navigation T owned init wr emptyState pre₁ post c b σ₁ hpost k hk,
    run_through_navigation T owned init wr emptyState pre₂ post c b σ₂ hpost k hk]

/-- A step strictly below `S` on the path up from `b`. -/
def OnPath (S b m : Nat) : Prop := ∃ j, up T j b = m ∧ ∀ i, i ≤ j → up T i b ≠ S

/-- The restored state differs from the source's recorded state at a fact only
through a recorded step on the path from the source (exclusive) to the
selected basis (inclusive). With a vouched basis that path lies on the
selected line. It feeds `since_change_is_a_line_write`. -/
theorem restored_differs_only_by_line_writes (S : Nat) (k : Nat) :
    ∀ b, AncOrSelf T S b →
      (∀ m, OnPath T S b m → wr m k = none) →
      recorded T init wr b k = recorded T init wr S k := by
  intro b
  induction b using Nat.strongRecOn with
  | _ b ih =>
  intro ⟨j, hj⟩ hnone
  cases j with
  | zero => simp [up] at hj; rw [hj]
  | succ j =>
    simp only [up] at hj
    by_cases hbS : b = S
    · rw [hbS]
    have hb0 : b ≠ 0 := by
      intro h0; subst h0; rw [T.par_root, up_root] at hj; subst hj; exact hbS rfl
    rw [recorded_succ T init wr hb0]
    have hw : wr b k = none :=
      hnone b ⟨0, rfl, fun i hi => by rw [show i = 0 by omega]; exact hbS⟩
    rw [hw]
    simp only [Option.getD_none]
    apply ih (T.par b) (T.par_lt b (by omega)) ⟨j, hj⟩
    intro m ⟨i, hi, hne⟩
    apply hnone m ⟨i + 1, hi, ?_⟩
    intro i' hi'
    cases i' with
    | zero => exact hbS
    | succ i' => exact hne i' (by omega)

theorem up_add (a j : Nat) : ∀ n, up T (a + j) n = up T j (up T a n) := by
  induction a with
  | zero => intro n; simp [up]
  | succ a ih =>
    intro n
    rw [show a + 1 + j = (a + j) + 1 by omega]
    simp only [up]
    exact ih (T.par n)

/-- **Line writes are what supersession must keep.** If any step `n` between
the source `S` and the selected basis `b` (the `Since` nodes up to `b`) holds a
fact differently from the source, a recorded step on the path from `S`
(exclusive) to `b` wrote that fact. With `kept_line_writes`, and the journal's
answer naming each such write's own step (the index's lower bound and its one
owner make that true), every such change stays kept. -/
theorem since_change_is_a_line_write (S b n k : Nat) (hSn : AncOrSelf T S n) (hnb : AncOrSelf T n b)
    (hne : recorded T init wr n k ≠ recorded T init wr S k) :
    ∃ m, OnPath T S b m ∧ wr m k ≠ none := by
  by_cases hnS : n = S
  · subst hnS; exact absurd rfl hne
  apply Classical.byContradiction
  intro hno
  apply hne
  apply restored_differs_only_by_line_writes T init wr S k n hSn
  intro m ⟨j, hj, hjS⟩
  obtain ⟨k0, hk0⟩ := hnb
  obtain ⟨jS, hjSn⟩ := hSn
  have hle : S ≤ n := by rw [← hjSn]; exact up_le T jS n
  apply Classical.byContradiction
  intro hw
  apply hno
  refine ⟨m, ⟨k0 + j, by rw [up_add, hk0, hj], ?_⟩, hw⟩
  intro i hi
  by_cases hik : k0 ≤ i
  · rw [show i = k0 + (i - k0) by omega, up_add, hk0]
    exact hjS (i - k0) (by omega)
  · intro hiS
    have : n = up T (k0 - i) S := by
      rw [← hiS, ← up_add, show i + (k0 - i) = k0 by omega, hk0]
    have hn : n ≤ S := by rw [this]; exact up_le T (k0 - i) S
    exact hnS (by omega)

/-- If the source wrote fact `k` with a fresh record (no other step writes that
record and the root does not hold it), then the final comparison passing on
`k` (the restored state still holds the source's record) already forces the
selected basis to descend from the source. -/
theorem fresh_fact_forces_descent (S k v : Nat) (hS : wr S k = some v)
    (fresh : ∀ m, m ≠ S → wr m k ≠ some v) (hinit : init k ≠ v) :
    ∀ b, recorded T init wr b k = v → AncOrSelf T S b := by
  intro b
  induction b using Nat.strongRecOn with
  | _ b ih =>
  intro hb
  by_cases hb0 : b = 0
  · subst hb0; rw [recorded, dif_pos rfl] at hb; exact absurd hb hinit
  rw [recorded_succ T init wr hb0] at hb
  cases hw : wr b k with
  | some v' =>
    rw [hw] at hb; simp only [Option.getD_some] at hb
    subst hb
    by_cases hbS : b = S
    · subst hbS; exact anc_refl T b
    · exact absurd hw (fresh b hbS)
  | none =>
    rw [hw] at hb; simp only [Option.getD_none] at hb
    exact anc_par T S b (ih (T.par b) (T.par_lt b (by omega)) hb)

/-- An undo to before the source, a checkout to a sibling branch, or any
other basis that does not descend from the source never vouches. -/
theorem nondescending_checkout_does_not_vouch (S current b : Nat)
    (h : ¬ AncOrSelf T S b) : ¬ Vouch T S current b := fun ⟨_, hd, _⟩ => h hd

end State

/-! ## The owned-closure premise is necessary -/

/-- A two-step line: root 0, step 1 (the source), step 2 (its child). -/
def line : Tree where
  par n := n - 1
  par_root := rfl
  par_lt n h := by omega

/-- Facts 0 (owned) and 1 (outside the owned closure). The source (step 1)
wrote record 10 to fact 0 and fact 1. A later write sets fact 1 to record 99,
then a navigation restores step 1. The rule cancels both, yet fact 1 at the
target is the cancelled write's record: outside the owned closure, only the
final comparison (`eligibility::compare`) guards. -/
theorem outside_closure_counterexample :
    let owned : Nat → Bool := fun k => k == 0
    let init : Nat → Nat := fun _ => 0
    let wr : Nat → Nat → Option Nat := fun n k => if n = 1 ∧ k ≤ 1 then some 10 else none
    let log : List (Nat × Occ) :=
      [(5, .write (fun k => if k = 1 then some 99 else none)), (6, .navigation (some 1))]
    supersededThrough log (fun b => b == 1) = some (6, 1) ∧
    kept log (fun b => b == 1) (fun _ _ => false) = [] ∧
    run line owned init wr (fun _ => 0) (fun k => recorded line init wr 1 k) log 1 = 99 ∧
    recorded line init wr 1 1 = 10 := by
  refine ⟨by decide, ?_, ?_, ?_⟩
  · simp [kept, supersededThrough, navs, maxByKey, List.foldl, Occ.kind, needsWitness]
  · simp [run, apply, List.foldl]
  · rw [recorded_succ line _ _ (by decide)]; simp

/-- **The rule before public #453 hid a line write** (court H, N2): step 1
makes the entity live, step 2 retires it, step 3 makes it live again (commit
13), step 4 retires it again (commit 14), then an undo (commit 15, basis 3)
and a redo (commit 16, basis 4). Correcting step 2's retirement: the restored
state at step 4 equals the source's, so the final comparison passes, yet step 3
held the fact differently. The old rule cancelled both line writes; the new
rule keeps them, and they then need their witnesses. -/
theorem old_rule_hides_a_line_write :
    let init : Nat → Nat := fun _ => 0
    let wr : Nat → Nat → Option Nat := fun n k =>
      if k = 0 then (if n = 1 then some 1 else if n = 2 then some 2
        else if n = 3 then some 3 else if n = 4 then some 2 else none) else none
    let log : List (Nat × Occ) :=
      [(13, .write (fun k => if k = 0 then some 3 else none)),
       (14, .write (fun k => if k = 0 then some 2 else none)),
       (15, .navigation (some 3)), (16, .navigation (some 4))]
    let vouch : Nat → Bool := fun b => b == 3 || b == 4
    let onLine : Nat → Nat → Bool := fun c _ => c == 13 || c == 14
    (keptOld log vouch).map Prod.fst = [] ∧
    (kept log vouch onLine).map Prod.fst = [13, 14] ∧
    recorded line init wr 4 0 = recorded line init wr 2 0 ∧
    recorded line init wr 3 0 ≠ recorded line init wr 2 0 := by
  refine ⟨by decide, by decide, ?_, ?_⟩
  · rw [recorded_succ line _ _ (n := 4) (by decide), recorded_succ line _ _ (n := 2) (by decide)]
    simp
  · rw [recorded_succ line _ _ (n := 3) (by decide), recorded_succ line _ _ (n := 2) (by decide)]
    simp

end Supersession
