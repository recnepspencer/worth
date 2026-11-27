-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/reclamation_plan_validator/mod.rs:32-73 sha256:907e0df3a76edb0ee0f1888a30ad7b6b78a524be0f1fe961b0f4d3e826cdb963
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/reclamation_plan_validator/mod.rs:106-198 sha256:dd4a272b0d41d3041019cd05f57f46f969fcef8630b0ad06bef87ac76562daac
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/reclamation_plan_validator/resume.rs:13-34 sha256:ba7064218f2566d6675d733527773d926b8d17d89ceb0a312d5f872749fa54f6
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/reclamation_plan_validator/resume.rs:51-84 sha256:9fe19c4287006fad3edee3f6e750778a7c7108332fef1943a4bb4f7d98ae6090
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/retention/reclaim_plan.rs:90-172 sha256:3b44c758b95a25d063f269bc80c3709e39cfb4f59e6087163fdb8b4f1147bfca

/-!
# Safe physical reclamation (Store Milestone 4, Phase M4.5b)

Models of `worth-store`'s reclamation planner and its verified
`ReclamationPlanValidator` kernel (`reclamation_plan_validator/`):

* `Span` is the kernel's opaque `ReachSpan`: an exact `key` within a `space`
  and an inclusive range `[first, last]`. Store's planner
  (`durability/retention/reclaim_plan.rs`) maps root generations and inline
  segment tails to spans; upper owners project theirs the same way.
* `planVerdict` is the kernel's `plan_reclaim_intent`, with the same refusal
  priority (reach, then canonical coverage, then unresolved obligations).
* `Holders.reach` is the complete independent reachability law `Reach`: pins
  and leases, their transitive overlay ancestors, in-process registrations,
  `retain_members` cold custody, latest and staged checkpoint closures, and
  pending recovery obligations.
* `resumeRelease` is the kernel's `resume_release`, the chained-crash resume
  plan for Path A (`Intent -> PublishFreeSpaceRoot -> ReclaimComplete ->
  AdmitToAllocator`).

Three theorems: `ReclaimPreservesAllReachableRoots`,
`CanonicalCoverageBeforePrune` and `InterruptedReclaimIdempotent`, each with
a negative witness for mutants `M5`, `M8` and `M9`.
-/

namespace StoreM4.ReclamationSafety

/-! ## Reach and the reclaim-intent verdict -/

structure Span where
  space : Nat
  key : Nat
  first : Nat
  last : Nat
  deriving DecidableEq, Repr

def Span.overlaps (a b : Span) : Prop :=
  a.space = b.space ∧ a.key = b.key ∧ a.first ≤ b.last ∧ b.first ≤ a.last

instance (a b : Span) : Decidable (a.overlaps b) := by
  unfold Span.overlaps; infer_instance

/-- One physical point: a root generation or an inline tail. -/
structure Point where
  space : Nat
  key : Nat
  at_ : Nat
  deriving DecidableEq, Repr

def Span.covers (s : Span) (p : Point) : Prop :=
  s.space = p.space ∧ s.key = p.key ∧ s.first ≤ p.at_ ∧ p.at_ ≤ s.last

theorem covers_overlap {x y : Span} {p : Point} (hx : x.covers p) (hy : y.covers p) :
    x.overlaps y := by
  unfold Span.covers at hx hy
  exact ⟨hx.1.trans hy.1.symm, hx.2.1.trans hy.2.1.symm, by omega, by omega⟩

inductive Coverage where
  | canonicalSuccessor
  | derivedOnly
  | uncovered
  deriving DecidableEq, Repr

inductive Verdict where
  | accept
  | reachable
  | uncovered
  | obligated
  deriving DecidableEq, Repr

def spanClear (c : Span) (reach : List Span) : Bool :=
  reach.all fun y => !decide (c.overlaps y)

def candidateDisjoint (candidate reach : List Span) : Bool :=
  candidate.all fun c => spanClear c reach

def disjoint (candidate reach : List Span) : Prop :=
  ∀ x ∈ candidate, ∀ y ∈ reach, ¬ x.overlaps y

theorem candidateDisjoint_iff (candidate reach : List Span) :
    candidateDisjoint candidate reach = true ↔ disjoint candidate reach := by
  simp [candidateDisjoint, spanClear, disjoint, List.all_eq_true]

/-- The kernel's `plan_reclaim_intent`. -/
def planVerdict (candidate reach obligations : List Span) (coverage : Coverage) : Verdict :=
  if !candidateDisjoint candidate reach then .reachable
  else if coverage ≠ .canonicalSuccessor then .uncovered
  else if !candidateDisjoint candidate obligations then .obligated
  else .accept

def admissible (candidate reach obligations : List Span) (coverage : Coverage) : Prop :=
  disjoint candidate reach ∧ coverage = .canonicalSuccessor ∧ disjoint candidate obligations

/-- The executable verdict is exactly the admission law. -/
theorem planVerdict_admit_iff (candidate reach obligations : List Span) (coverage : Coverage) :
    planVerdict candidate reach obligations coverage = .accept ↔
      admissible candidate reach obligations coverage := by
  unfold planVerdict admissible
  rw [← candidateDisjoint_iff, ← candidateDisjoint_iff]
  by_cases h1 : candidateDisjoint candidate reach = true <;>
    by_cases h2 : coverage = Coverage.canonicalSuccessor <;>
    by_cases h3 : candidateDisjoint candidate obligations = true <;>
    simp [h1, h2, h3]

/-- Everything that still keeps physical bytes alive. -/
structure Holders where
  pins : List Span
  overlayAncestors : List Span
  registrations : List Span
  retainMembers : List Span
  checkpoints : List Span
  recovery : List Span

def Holders.reach (h : Holders) : List Span :=
  h.pins ++ h.overlayAncestors ++ h.registrations ++ h.retainMembers ++ h.checkpoints ++
    h.recovery

def Holders.protects (h : Holders) (p : Point) : Prop :=
  ∃ y ∈ h.reach, y.covers p

/-- A reclamation plan: candidates, each with its spans and coverage. -/
abbrev Plan := List (List Span × Coverage)

def Plan.admitted (plan : Plan) (reach obligations : List Span) : Prop :=
  ∀ c ∈ plan, planVerdict c.1 reach obligations c.2 = .accept

def Plan.frees (plan : Plan) (p : Point) : Prop :=
  ∃ c ∈ plan, ∃ x ∈ c.1, x.covers p

/-- `ReclaimPreservesAllReachableRoots`: a plan admitted against the complete
projected reach frees no point that any holder (pin, lease, overlay ancestor,
registration, `retain_members` custody, checkpoint or recovery obligation)
still protects, and every admitted candidate had canonical coverage. -/
theorem ReclaimPreservesAllReachableRoots (h : Holders) (obligations : List Span)
    (plan : Plan) (hadm : plan.admitted h.reach obligations) :
    (∀ p, h.protects p → ¬ plan.frees p) ∧
      (∀ c ∈ plan, c.2 = .canonicalSuccessor) := by
  constructor
  · rintro p ⟨y, hy, hyp⟩ ⟨c, hc, x, hx, hxp⟩
    have hd := ((planVerdict_admit_iff _ _ _ _).1 (hadm c hc)).1
    exact hd x hx y hy (covers_overlap hxp hyp)
  · intro c hc
    exact ((planVerdict_admit_iff _ _ _ _).1 (hadm c hc)).2.1

/-- Each holder category is part of `Reach`; dropping any one is unsound. -/
theorem retain_members_in_reach (h : Holders) : ∀ y ∈ h.retainMembers, y ∈ h.reach := by
  intro y hy; simp [Holders.reach, hy]

theorem overlay_ancestors_in_reach (h : Holders) :
    ∀ y ∈ h.overlayAncestors, y ∈ h.reach := by
  intro y hy; simp [Holders.reach, hy]

/-- A projector that forgets `retain_members` custody (mutant `M5`). -/
def Holders.reachWithoutRetain (h : Holders) : List Span :=
  h.pins ++ h.overlayAncestors ++ h.registrations ++ h.checkpoints ++ h.recovery

def m5Holders : Holders := ⟨[], [], [], [⟨1, 0, 5, 5⟩], [], []⟩
def m5Plan : Plan := [([⟨1, 0, 0, 9⟩], .canonicalSuccessor)]

/-- `M5`: against a reach missing live cold custody, the validator admits a
plan that frees a root a `retain_members` holder still reads. -/
theorem M5_forgetting_retain_members_frees_held_root :
    m5Plan.admitted m5Holders.reachWithoutRetain [] ∧
      m5Holders.protects ⟨1, 0, 5⟩ ∧ m5Plan.frees ⟨1, 0, 5⟩ := by
  refine ⟨?_, ?_, ?_⟩
  · intro c hc
    simp [m5Plan] at hc
    subst hc
    decide
  · exact ⟨⟨1, 0, 5, 5⟩, by simp [m5Holders, Holders.reach],
      by simp [Span.covers]⟩
  · exact ⟨_, List.mem_singleton_self _, ⟨1, 0, 0, 9⟩, List.mem_singleton_self _,
      by simp [Span.covers]⟩

/-! ## Canonical coverage before prune -/

/-- Journal segments carry semantic meanings; a sealed canonical checkpoint
covers a set of meanings; derived indexes are disposable accelerators. -/
structure Journal where
  segments : List Nat
  checkpoints : List (List Nat)
  derived : List (List Nat)

/-- The coverage the planner presents for pruning `m`: only a sealed canonical
checkpoint counts; a derived index alone is `derivedOnly`. -/
def Journal.coverage (j : Journal) (m : Nat) : Coverage :=
  if j.checkpoints.any (fun ck => ck.contains m) then .canonicalSuccessor
  else if j.derived.any (fun d => d.contains m) then .derivedOnly
  else .uncovered

/-- A live meaning survives in canonical storage. -/
def Journal.recoverable (j : Journal) (m : Nat) : Prop :=
  m ∈ j.segments ∨ ∃ ck ∈ j.checkpoints, m ∈ ck

inductive JStep (coverage : Journal → Nat → Coverage) : Journal → Journal → Prop
  | append (j : Journal) (m : Nat) : JStep coverage j { j with segments := m :: j.segments }
  | sealCheckpoint (j : Journal) (ck : List Nat) :
      (∀ m ∈ ck, j.recoverable m) → JStep coverage j { j with checkpoints := ck :: j.checkpoints }
  | buildDerived (j : Journal) (d : List Nat) :
      JStep coverage j { j with derived := d :: j.derived }
  | dropDerived (j : Journal) : JStep coverage j { j with derived := [] }
  | prune (j : Journal) (m : Nat) :
      planVerdict [] [] [] (coverage j m) = .accept →
      JStep coverage j { j with segments := j.segments.erase m }

inductive JReachable (coverage : Journal → Nat → Coverage) (live : List Nat) :
    Journal → Prop
  | init (j : Journal) : (∀ m ∈ live, m ∈ j.segments) → JReachable coverage live j
  | step {j j' : Journal} : JReachable coverage live j → JStep coverage j j' →
      JReachable coverage live j'

theorem erase_recoverable {j : Journal} {m n : Nat} (hcov : ∃ ck ∈ j.checkpoints, m ∈ ck)
    (hn : j.recoverable n) : ({ j with segments := j.segments.erase m } : Journal).recoverable n := by
  rcases hn with hn | hn
  · by_cases hnm : n = m
    · subst hnm; exact Or.inr hcov
    · exact Or.inl ((List.mem_erase_of_ne hnm).2 hn)
  · exact Or.inr hn

/-- `CanonicalCoverageBeforePrune`: a prune is admitted only while a sealed
canonical checkpoint covers the meaning, so along every interleaving of
appends, seals, derived-index builds and discards, and admitted prunes,
every live meaning stays recoverable from canonical storage alone, even with
every derived index discarded. -/
theorem CanonicalCoverageBeforePrune (live : List Nat) {j : Journal}
    (hr : JReachable Journal.coverage live j) :
    (∀ m ∈ live, j.recoverable m) ∧
      (∀ m ∈ live, ({ j with derived := [] } : Journal).recoverable m) := by
  have key : ∀ m ∈ live, j.recoverable m := by
    induction hr with
    | init j hj => exact fun m hm => Or.inl (hj m hm)
    | @step j _ _ hs ih =>
      cases hs with
      | append m' =>
        intro m hm
        rcases ih m hm with h | h
        · exact Or.inl (List.mem_cons_of_mem _ h)
        · exact Or.inr h
      | sealCheckpoint ck _ =>
        intro m hm
        rcases ih m hm with h | ⟨c, hc, hmc⟩
        · exact Or.inl h
        · exact Or.inr ⟨c, List.mem_cons_of_mem _ hc, hmc⟩
      | buildDerived d => exact ih
      | dropDerived => exact ih
      | prune m' hadm =>
        have hcov : ∃ ck ∈ j.checkpoints, m' ∈ ck := by
          unfold planVerdict Journal.coverage at hadm
          by_cases hany : j.checkpoints.any (fun ck => ck.contains m') = true
          · simp only [List.any_eq_true, List.contains_iff_mem] at hany
            exact hany
          · simp [candidateDisjoint] at hadm
            split at hadm <;> simp_all
        exact fun m hm => erase_recoverable hcov (ih m hm)
  exact ⟨key, fun m hm => key m hm⟩

/-- A planner that lets a derived index stand in for canonical coverage
(mutant `M8`). -/
def derivedCountsCoverage (j : Journal) (m : Nat) : Coverage :=
  if j.derived.any (fun d => d.contains m) then .canonicalSuccessor else j.coverage m

/-- `M8`: once a derived index counts as coverage, a pruned meaning is lost as
soon as that disposable index is discarded. -/
theorem M8_derived_coverage_loses_meaning :
    let j0 : Journal := ⟨[7], [], []⟩
    let j1 : Journal := { j0 with derived := [[7]] }
    let j2 : Journal := { j1 with segments := j1.segments.erase 7 }
    let j3 : Journal := { j2 with derived := [] }
    JReachable derivedCountsCoverage [7] j3 ∧ ¬ j3.recoverable 7 := by
  intro j0 j1 j2 j3
  refine ⟨?_, ?_⟩
  · refine JReachable.step (JReachable.step (JReachable.step
      (JReachable.init j0 (by simp [j0])) (JStep.buildDerived j0 [7])) ?_)
      (JStep.dropDerived j2)
    exact JStep.prune j1 7 (by decide)
  · simp [j3, j2, j1, j0, Journal.recoverable]

/-! ## Chained-crash interrupted reclamation -/

inductive Barrier where
  | intentDurable
  | freeSpacePublished
  | completionDurable
  deriving DecidableEq, Repr

structure ResumePlan where
  publish : Bool
  complete : Bool
  release : Bool
  deriving DecidableEq, Repr

/-- The kernel's `resume_release`. -/
def resumeRelease : Barrier → ResumePlan
  | .intentDurable => ⟨true, true, true⟩
  | .freeSpacePublished => ⟨false, true, false⟩
  | .completionDurable => ⟨false, false, false⟩

structure Sys where
  /-- Durable: the newest release barrier. -/
  barrier : Barrier
  /-- Durable: the published free-space root lists the retired range. -/
  durableFree : Bool
  /-- Durable: the generation of a live reallocation of the range. -/
  owner : Option Nat
  /-- Volatile: allocator free-list entries for the range. -/
  freeCount : Nat
  /-- Volatile: this incarnation's remaining release effects. -/
  plan : ResumePlan

inductive Step (resume : Barrier → ResumePlan) : Sys → Sys → Prop
  | publish (s : Sys) : s.barrier = .intentDurable → s.plan.publish = true →
      Step resume s { s with barrier := .freeSpacePublished, durableFree := true }
  | complete (s : Sys) : s.barrier = .freeSpacePublished → s.plan.complete = true →
      Step resume s { s with barrier := .completionDurable }
  /-- `WritebackFenceDrain` + `AdmitToAllocator`, only after completion. -/
  | admitToAllocator (s : Sys) : s.barrier = .completionDurable → s.plan.release = true →
      Step resume s { s with freeCount := s.freeCount + 1, plan := { s.plan with release := false } }
  /-- Ordinary publication resumes only after the release's last effect, then
  reissues the range at a newer generation `g'`. -/
  | reallocate (s : Sys) (g : Nat) : s.barrier = .completionDurable →
      s.plan.release = false → 0 < s.freeCount →
      Step resume s { s with freeCount := s.freeCount - 1, durableFree := false, owner := some g }
  /-- Crash, reopen (bootstrap loads the published free index), resume. -/
  | crash (s : Sys) :
      Step resume s { s with freeCount := if s.durableFree then 1 else 0, plan := resume s.barrier }

def start : Sys := ⟨.intentDurable, false, none, 0, resumeRelease .intentDurable⟩

inductive Reachable (resume : Barrier → ResumePlan) : Sys → Prop
  | init : Reachable resume start
  | step {s s' : Sys} : Reachable resume s → Step resume s s' → Reachable resume s'

structure Inv (s : Sys) : Prop where
  single : s.freeCount ≤ 1
  release_fresh : s.plan.release = true → s.freeCount = 0 ∧ s.owner = none
  owned_not_free : s.owner.isSome → s.freeCount = 0 ∧ s.durableFree = false
  intent_unpublished : s.barrier = .intentDurable → s.durableFree = false ∧ s.owner = none

theorem resume_release_only_from_intent {b : Barrier} (h : (resumeRelease b).release = true) :
    b = .intentDurable := by
  cases b <;> simp_all [resumeRelease]

theorem step_inv {s s' : Sys} (hi : Inv s) (hs : Step resumeRelease s s') : Inv s' := by
  cases hs with
  | publish hb _ =>
    have ho := (hi.intent_unpublished hb).2
    exact ⟨hi.single, hi.release_fresh, by simp [ho], by simp⟩
  | complete hb _ =>
    exact ⟨hi.single, hi.release_fresh, hi.owned_not_free, by simp⟩
  | admitToAllocator hb ha =>
    have ⟨hf, ho⟩ := hi.release_fresh ha
    exact ⟨by simp [hf], by simp, by simp [ho], by simp [hb]⟩
  | reallocate g hb ha hpos =>
    have := hi.single
    exact ⟨by simp; omega, by simp [ha], by simp; omega, by simp [hb]⟩
  | crash =>
    refine ⟨by split <;> simp, ?_, ?_, ?_⟩
    · intro ha
      have hb := resume_release_only_from_intent ha
      have ⟨hd, ho⟩ := hi.intent_unpublished hb
      simp [hd, ho]
    · intro ho
      have ⟨_, hd⟩ := hi.owned_not_free ho
      simp [hd]
    · exact hi.intent_unpublished

theorem Reachable.inv {s : Sys} (hr : Reachable resumeRelease s) : Inv s := by
  induction hr with
  | init => exact ⟨by simp [start], by simp [start, resumeRelease], by simp [start], by simp [start]⟩
  | step _ hs ih => exact step_inv ih hs

/-- `InterruptedReclaimIdempotent`: across any chain of crashes between
`Intent`, `PublishFreeSpaceRoot`, `ReclaimComplete` and `AdmitToAllocator`,
including a crash inside recovery's own resume after the free-space root is
published and before completion, the allocator never holds the retired range
twice, and a reallocated generation `g'` is never free again nor lost from
the durable free index view. -/
theorem InterruptedReclaimIdempotent {s : Sys} (hr : Reachable resumeRelease s) :
    s.freeCount ≤ 1 ∧ (s.owner.isSome → s.freeCount = 0 ∧ s.durableFree = false) :=
  ⟨hr.inv.single, hr.inv.owned_not_free⟩

/-- Non-vacuity: crash after the intent, resume and publish, crash again
before completion, complete, reallocate at `g' = 9`, crash and reopen. -/
theorem chained_crash_reaches_reallocated_range :
    ∃ s, Reachable resumeRelease s ∧ s.owner = some 9 ∧ s.barrier = .completionDurable := by
  let s2 : Sys := { start with barrier := .freeSpacePublished, durableFree := true }
  let s3 : Sys := { s2 with freeCount := 1, plan := resumeRelease .freeSpacePublished }
  let s4 : Sys := { s3 with barrier := .completionDurable }
  let s5 : Sys := { s4 with freeCount := 0, durableFree := false, owner := some 9 }
  let s6 : Sys := { s5 with freeCount := 0, plan := resumeRelease .completionDurable }
  have r1 : Reachable resumeRelease start :=
    Reachable.step Reachable.init (Step.crash start)
  have r2 : Reachable resumeRelease s2 := Reachable.step r1 (Step.publish start rfl rfl)
  have r3 : Reachable resumeRelease s3 := Reachable.step r2 (Step.crash s2)
  have r4 : Reachable resumeRelease s4 := Reachable.step r3 (Step.complete s3 rfl rfl)
  have r5 : Reachable resumeRelease s5 :=
    Reachable.step r4 (Step.reallocate s4 9 rfl rfl (by decide))
  have r6 : Reachable resumeRelease s6 := Reachable.step r5 (Step.crash s5)
  exact ⟨s6, r6, rfl, rfl⟩

/-- A resume that also re-admits a range whose free-space root was already
published (mutant `M9`). -/
def resumeReadmits : Barrier → ResumePlan
  | .intentDurable => ⟨true, true, true⟩
  | .freeSpacePublished => ⟨false, true, true⟩
  | .completionDurable => ⟨false, false, false⟩

/-- `M9`: replaying the release after a crash inside recovery's resume puts the
range on the free list twice, so two writers can be issued the same bytes. -/
theorem M9_readmitting_published_range_double_frees :
    ∃ s, Reachable resumeReadmits s ∧ s.freeCount = 2 := by
  let s1 : Sys := { start with plan := resumeReadmits .intentDurable }
  let s2 : Sys := { s1 with barrier := .freeSpacePublished, durableFree := true }
  let s3 : Sys := { s2 with freeCount := 1, plan := resumeReadmits .freeSpacePublished }
  let s4 : Sys := { s3 with barrier := .completionDurable }
  let s5 : Sys := { s4 with freeCount := 2, plan := { s4.plan with release := false } }
  have r1 : Reachable resumeReadmits s1 := Reachable.step Reachable.init (Step.crash start)
  have r2 : Reachable resumeReadmits s2 := Reachable.step r1 (Step.publish s1 rfl rfl)
  have r3 : Reachable resumeReadmits s3 := Reachable.step r2 (Step.crash s2)
  have r4 : Reachable resumeReadmits s4 := Reachable.step r3 (Step.complete s3 rfl rfl)
  have r5 : Reachable resumeReadmits s5 :=
    Reachable.step r4 (Step.admitToAllocator s4 rfl rfl)
  exact ⟨s5, r5, rfl⟩

end StoreM4.ReclamationSafety
