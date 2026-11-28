-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/transitions.rs:7-108 sha256:4805119d6cf9854b07661e6eb47eaba225184fb7d7ab7ae29b80e95dda67f786
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/transitions.rs:110-162 sha256:a236be84da0339827a016a34ccc1358effa70b98a418482d0c11ca823eacf446
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/transitions.rs:164-222 sha256:bfc5144a73023b130ab9a69436db88070b506b262c3a9052c2158e96611c98d6
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/transitions.rs:224-340 sha256:655032bffbe18ed09432badd6854747fdf99a79a07e7a689cec360e0a96951ac
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table/core.rs:120-162 sha256:aa878e944bd5ebb57d83d9f428c5f39421dd95fed5e031cc01bad23e77ad1cf1
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/physical_runtime/stability/retention/registrations/reclaim_guard.rs:9-39 sha256:dfd39db11714d79ce9571ece1dc27589a9ad21ad924f0cf396930d07fbb7aad2

import StoreM4.PinSafety

/-!
# `StoreGCPinSafety` and `LeaseExpiryNoReaderLoss` (Store Milestone 4, Phase M4.1)

Executable model of `worth-store`'s `PinTableKernel` transitions
(`acquire_pin`, `acquire_reader_loan`, `release_reader_loan`, `expire_lease`).
Every refusal returns `none` and leaves the table untouched, matching the
kernel's `Err(_) => entries unchanged` postconditions.

Production reclamation (`reclaim_guard` / displaced-segment retirement) asks only
whether a root is held by a *non-expired pin entry*. The theorems below certify that
this direct projection already covers the full reachability law of milestone-4 §9.1
row 1: every root reachable from a live pin or an active reader loan through any
chain of nested parent overlays stays protected, for every table reachable from
the empty table through the kernel transitions.
-/

namespace StoreM4.GCPinSafety

open StoreM4.PinSafety

/-- `c` is a live (non-expired) direct overlay child of the pin identified by `pid`. -/
def liveChildOf (pid : Nat) (c : PinEntry) : Bool :=
  c.parent == some pid && c.state != LeaseState.expired

/-- Number of live direct overlay children of pin `pid`. -/
def childCount (t : PinTable) (pid : Nat) : Nat :=
  t.countP (liveChildOf pid)

def freshPin (id root : Nat) (parent : Option Nat) : PinEntry :=
  { id := id, root := root, parent := parent, state := LeaseState.active,
    activeReaderLoans := 0, liveChildPins := 0 }

def bumpChild (pid : Nat) (p : PinEntry) : PinEntry :=
  if p.id = pid then { p with liveChildPins := p.liveChildPins + 1 } else p

/-- `PinTableKernel::acquire_pin`: fresh `Active` pin, parent must be `Active`. -/
def pinAcquire (t : PinTable) (id root : Nat) (parent : Option Nat) : Option PinTable :=
  if t.any (fun e => e.id == id) then none
  else
    match parent with
    | none => some (t ++ [freshPin id root none])
    | some pid =>
        if t.any (fun p => p.id == pid && p.state == LeaseState.active) then
          some (t.map (bumpChild pid) ++ [freshPin id root (some pid)])
        else none

def setLoans (id : Nat) (f : Nat → Nat) (e : PinEntry) : PinEntry :=
  if e.id = id then { e with activeReaderLoans := f e.activeReaderLoans } else e

/-- `PinTableKernel::acquire_reader_loan`: only `Active` pins admit new loans. -/
def loanAcquire (t : PinTable) (id : Nat) : Option PinTable :=
  if t.any (fun e => e.id == id && e.state == LeaseState.active) then
    some (t.map (setLoans id (· + 1)))
  else none

/-- `PinTableKernel::release_reader_loan` (saturating decrement). -/
def loanRelease (t : PinTable) (id : Nat) : Option PinTable :=
  if t.any (fun e => e.id == id) then some (t.map (setLoans id (· - 1))) else none

/-- Whether one expiry step on `e` completes `Expiring -> Expired`. -/
def completes (e : PinEntry) : Bool :=
  e.state == LeaseState.expiring && e.activeReaderLoans == 0 && e.liveChildPins == 0

def expireAt (id : Nat) (parent : Option Nat) (done : Bool) (x : PinEntry) : PinEntry :=
  if x.id = id then x.expireStep
  else if done && parent == some x.id then { x with liveChildPins := x.liveChildPins - 1 }
  else x

/-- `PinTableKernel::expire_lease`. Completing expiry releases one parent child slot. -/
def leaseExpire (t : PinTable) (id : Nat) : Option PinTable :=
  match t.find? (fun e => e.id == id) with
  | none => none
  | some e => some (t.map (expireAt id e.parent (completes e)))

/-- Table invariant maintained by every kernel transition. -/
structure WellFormed (t : PinTable) : Prop where
  nodup : (t.map PinEntry.id).Nodup
  childBound : ∀ e ∈ t, childCount t e.id ≤ e.liveChildPins
  parentLive : ∀ c ∈ t, c.protectsRoot → ∀ pid, c.parent = some pid →
    ∃ p ∈ t, p.id = pid ∧ p.protectsRoot
  noSelfParent : ∀ e ∈ t, e.parent ≠ some e.id
  expiredDrained : ∀ e ∈ t, e.state = LeaseState.expired →
    e.activeReaderLoans = 0 ∧ e.liveChildPins = 0

/-- Tables reachable from the empty table through the kernel transitions. -/
inductive Reachable : PinTable → Prop
  | empty : Reachable []
  | viaAcquire {t t' : PinTable} {id root : Nat} {parent : Option Nat} :
      Reachable t → pinAcquire t id root parent = some t' → Reachable t'
  | viaLoanAcquire {t t' : PinTable} {id : Nat} :
      Reachable t → loanAcquire t id = some t' → Reachable t'
  | viaLoanRelease {t t' : PinTable} {id : Nat} :
      Reachable t → loanRelease t id = some t' → Reachable t'
  | viaExpire {t t' : PinTable} {id : Nat} :
      Reachable t → leaseExpire t id = some t' → Reachable t'

/-- `a` is `e` itself or a transitive nested parent overlay of `e`. -/
inductive OverlayAncestor (t : PinTable) : PinEntry → PinEntry → Prop
  | refl (e : PinEntry) : OverlayAncestor t e e
  | step {e c p : PinEntry} :
      OverlayAncestor t e c → p ∈ t → c.parent = some p.id → OverlayAncestor t e p

/-- A pin holds its overlay chain when it is live or carries an active reader loan. -/
def holdsChain (e : PinEntry) : Prop :=
  e.protectsRoot ∨ e.activeReaderLoans > 0

/-- Full independent reachability law (milestone-4 §9.1 row 1): live roots, durable pins,
    active reader loans, and every transitive nested parent overlay. -/
def fullReach (liveRoots : Nat → Prop) (t : PinTable) (r : Nat) : Prop :=
  liveRoots r ∨ ∃ e ∈ t, holdsChain e ∧ ∃ a, OverlayAncestor t e a ∧ a.root = r

/-- What production reclamation actually consults: live roots and non-expired pins. -/
def projectedReach (liveRoots : Nat → Prop) (t : PinTable) (r : Nat) : Prop :=
  liveRoots r ∨ PinTable.reachableRoot t r

/-! ## Generic list lemmas -/

theorem eq_of_mem_of_id_eq :
    ∀ {t : PinTable}, (t.map PinEntry.id).Nodup →
      ∀ {x y : PinEntry}, x ∈ t → y ∈ t → x.id = y.id → x = y
  | [], _, _, _, hx, _, _ => by cases hx
  | a :: t, hnd, x, y, hx, hy, hid => by
      rw [List.map_cons, List.nodup_cons] at hnd
      obtain ⟨hnotin, hnd⟩ := hnd
      rcases List.mem_cons.mp hx with rfl | hx' <;> rcases List.mem_cons.mp hy with rfl | hy'
      · rfl
      · exact absurd (hid ▸ List.mem_map_of_mem hy') hnotin
      · exact absurd (hid.symm ▸ List.mem_map_of_mem hx') hnotin
      · exact eq_of_mem_of_id_eq hnd hx' hy' hid

theorem countP_map_le {g : PinEntry → PinEntry} {p : PinEntry → Bool} :
    ∀ (t : PinTable), (∀ x ∈ t, p (g x) = true → p x = true) →
      (t.map g).countP p ≤ t.countP p
  | [], _ => Nat.le_refl _
  | a :: t, h => by
      have ih := countP_map_le t (fun x hx => h x (List.mem_cons_of_mem _ hx))
      have ha := h a List.mem_cons_self
      simp only [List.map_cons, List.countP_cons]
      cases hga : p (g a) <;> cases hpa : p a <;> simp_all <;> omega

theorem countP_map_lt {g : PinEntry → PinEntry} {p : PinEntry → Bool} {e : PinEntry} :
    ∀ (t : PinTable), (∀ x ∈ t, p (g x) = true → p x = true) → e ∈ t →
      p e = true → p (g e) = false → (t.map g).countP p + 1 ≤ t.countP p
  | [], _, he, _, _ => by cases he
  | a :: t, h, he, hpe, hge => by
      have hrest : ∀ x ∈ t, p (g x) = true → p x = true :=
        fun x hx => h x (List.mem_cons_of_mem _ hx)
      have ha := h a List.mem_cons_self
      simp only [List.map_cons, List.countP_cons]
      rcases List.mem_cons.mp he with rfl | he'
      · have := countP_map_le t hrest
        simp only [hpe, hge, Bool.false_eq_true, ite_true, ite_false]; omega
      · have ih := countP_map_lt t hrest he' hpe hge
        cases hga : p (g a) <;> cases hpa : p a <;> simp_all <;> omega

theorem countP_map_eq {g : PinEntry → PinEntry} {p : PinEntry → Bool} (t : PinTable)
    (h : ∀ x ∈ t, p (g x) = p x) : (t.map g).countP p = t.countP p := by
  rw [List.countP_map]
  exact List.countP_congr (fun x hx => by simp [h x hx])

theorem ids_map {g : PinEntry → PinEntry} (hg : ∀ x, (g x).id = x.id) (t : PinTable) :
    (t.map g).map PinEntry.id = t.map PinEntry.id := by
  rw [List.map_map]
  exact List.map_congr_left (fun x _ => hg x)

theorem childCount_zero_of_fresh {t : PinTable} {id : Nat} (hwf : WellFormed t)
    (hfresh : ¬ ∃ e ∈ t, e.id = id) : childCount t id = 0 := by
  unfold childCount
  rw [List.countP_eq_zero]
  intro c hc hlive
  simp only [liveChildOf, Bool.and_eq_true, beq_iff_eq, bne_iff_ne, ne_eq] at hlive
  obtain ⟨p, hp, hpid, _⟩ := hwf.parentLive c hc hlive.2 id hlive.1
  exact hfresh ⟨p, hp, hpid⟩

theorem any_id_iff {t : PinTable} {id : Nat} :
    t.any (fun e => e.id == id) = true ↔ ∃ e ∈ t, e.id = id := by
  simp [List.any_eq_true]

/-! ## Invariant preservation -/

theorem wf_empty : WellFormed [] :=
  ⟨List.nodup_nil, by simp, by simp, by simp, by simp⟩

theorem liveChildOf_bump (pid q : Nat) (x : PinEntry) :
    liveChildOf q (bumpChild pid x) = liveChildOf q x := by
  unfold bumpChild; split <;> rfl

theorem pinAcquire_wf {t t' : PinTable} {id root : Nat} {parent : Option Nat}
    (hwf : WellFormed t) (h : pinAcquire t id root parent = some t') : WellFormed t' := by
  unfold pinAcquire at h
  split at h
  · cases h
  rename_i hnot
  have hfresh : ¬ ∃ e ∈ t, e.id = id := fun hex => hnot (any_id_iff.mpr hex)
  have hzero := childCount_zero_of_fresh hwf hfresh
  cases parent with
  | none =>
      simp only [Option.some.injEq] at h
      subst h
      have hcnt : ∀ q, childCount (t ++ [freshPin id root none]) q = childCount t q := by
        intro q; simp [childCount, List.countP_append, liveChildOf, freshPin]
      refine ⟨?_, ?_, ?_, ?_, ?_⟩
      · rw [List.map_append, List.nodup_append]
        refine ⟨hwf.nodup, (by simp), ?_⟩
        intro a ha b hb hab
        simp only [List.map_cons, List.map_nil, List.mem_singleton, freshPin] at hb
        subst hb; subst hab
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp ha
        exact hfresh ⟨x, hx, rfl⟩
      · intro e he
        rw [hcnt]
        rcases List.mem_append.mp he with he | he
        · exact hwf.childBound e he
        · simp only [List.mem_singleton] at he; subst he; simp [freshPin, hzero]
      · intro c hc hlive pid hpar
        rcases List.mem_append.mp hc with hc | hc
        · obtain ⟨p, hp, hpid, hpl⟩ := hwf.parentLive c hc hlive pid hpar
          exact ⟨p, List.mem_append_left _ hp, hpid, hpl⟩
        · simp only [List.mem_singleton] at hc; subst hc; simp [freshPin] at hpar
      · intro e he
        rcases List.mem_append.mp he with he | he
        · exact hwf.noSelfParent e he
        · simp only [List.mem_singleton] at he; subst he; simp [freshPin]
      · intro e he hexp
        rcases List.mem_append.mp he with he | he
        · exact hwf.expiredDrained e he hexp
        · simp only [List.mem_singleton] at he; subst he; simp [freshPin] at hexp
  | some pid =>
      simp only at h
      split at h
      · rename_i hpar
        simp only [Option.some.injEq] at h
        subst h
        obtain ⟨p0, hp0, hp0c⟩ := List.any_eq_true.mp hpar
        simp only [Bool.and_eq_true, beq_iff_eq] at hp0c
        obtain ⟨hp0id, hp0act⟩ := hp0c
        have hidne : id ≠ pid := fun heq => hfresh ⟨p0, hp0, hp0id.trans heq.symm⟩
        have hcnt : ∀ q, childCount (t.map (bumpChild pid) ++ [freshPin id root (some pid)]) q
            = childCount t q + (if q = pid then 1 else 0) := by
          intro q
          simp only [childCount, List.countP_append]
          rw [countP_map_eq t (fun x _ => liveChildOf_bump pid q x)]
          by_cases hq : q = pid
          · subst hq; simp [liveChildOf, freshPin]
          · have : pid ≠ q := fun e => hq e.symm
            simp [liveChildOf, freshPin, hq, this]
        have hbid : ∀ x, (bumpChild pid x).id = x.id := by
          intro x; unfold bumpChild; split <;> rfl
        have hbstate : ∀ x, (bumpChild pid x).state = x.state := by
          intro x; unfold bumpChild; split <;> rfl
        have hbpar : ∀ x, (bumpChild pid x).parent = x.parent := by
          intro x; unfold bumpChild; split <;> rfl
        refine ⟨?_, ?_, ?_, ?_, ?_⟩
        · rw [List.map_append, ids_map hbid, List.nodup_append]
          refine ⟨hwf.nodup, (by simp), ?_⟩
          intro a ha b hb hab
          simp only [List.map_cons, List.map_nil, List.mem_singleton, freshPin] at hb
          subst hb; subst hab
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp ha
          exact hfresh ⟨x, hx, rfl⟩
        · intro e he
          rw [hcnt]
          rcases List.mem_append.mp he with he | he
          · obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
            have hb := hwf.childBound x hx
            rw [hbid]
            unfold bumpChild
            by_cases hxp : x.id = pid
            · simp [hxp]; simp [hxp] at hb; omega
            · simp [hxp]; exact hb
          · simp only [List.mem_singleton] at he; subst he
            simp [freshPin, hzero, hidne]
        · intro c hc hlive q hpar
          rcases List.mem_append.mp hc with hc | hc
          · obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hc
            have hxl : x.protectsRoot := by
              unfold PinEntry.protectsRoot at *; rwa [hbstate] at hlive
            obtain ⟨p, hp, hpid, hpl⟩ := hwf.parentLive x hx hxl q (by rwa [hbpar] at hpar)
            refine ⟨bumpChild pid p, List.mem_append_left _ (List.mem_map_of_mem hp), ?_, ?_⟩
            · rw [hbid]; exact hpid
            · unfold PinEntry.protectsRoot at *; rwa [hbstate]
          · simp only [List.mem_singleton] at hc; subst hc
            simp only [freshPin, Option.some.injEq] at hpar; subst hpar
            refine ⟨bumpChild pid p0, List.mem_append_left _ (List.mem_map_of_mem hp0), ?_, ?_⟩
            · rw [hbid]; exact hp0id
            · unfold PinEntry.protectsRoot; rw [hbstate, hp0act]; intro h; cases h
        · intro e he
          rcases List.mem_append.mp he with he | he
          · obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
            rw [hbid, hbpar]; exact hwf.noSelfParent x hx
          · simp only [List.mem_singleton] at he; subst he
            simp [freshPin]; exact fun h => hidne h.symm
        · intro e he hexp
          rcases List.mem_append.mp he with he | he
          · obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
            rw [hbstate] at hexp
            obtain ⟨hl, hc⟩ := hwf.expiredDrained x hx hexp
            unfold bumpChild
            by_cases hxp : x.id = pid
            · exfalso
              have := eq_of_mem_of_id_eq hwf.nodup hx hp0 (hxp.trans hp0id.symm)
              subst this; rw [hp0act] at hexp; cases hexp
            · simp [hxp, hl, hc]
          · simp only [List.mem_singleton] at he; subst he; simp [freshPin] at hexp
      · cases h

theorem setLoans_wf {t : PinTable} {id : Nat} {f : Nat → Nat} (hwf : WellFormed t)
    (hexp : ∀ e ∈ t, e.id = id → e.state = LeaseState.expired → f e.activeReaderLoans = 0) :
    WellFormed (t.map (setLoans id f)) := by
  have hsid : ∀ x, (setLoans id f x).id = x.id := by intro x; unfold setLoans; split <;> rfl
  have hsst : ∀ x, (setLoans id f x).state = x.state := by intro x; unfold setLoans; split <;> rfl
  have hspar : ∀ x, (setLoans id f x).parent = x.parent := by
    intro x; unfold setLoans; split <;> rfl
  have hskids : ∀ x, (setLoans id f x).liveChildPins = x.liveChildPins := by
    intro x; unfold setLoans; split <;> rfl
  have hcnt : ∀ q, childCount (t.map (setLoans id f)) q = childCount t q := by
    intro q
    unfold childCount
    exact countP_map_eq t (fun x _ => by simp [liveChildOf, hsst, hspar])
  refine ⟨?_, ?_, ?_, ?_, ?_⟩
  · rw [ids_map hsid]; exact hwf.nodup
  · intro e he
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
    rw [hcnt, hsid, hskids]; exact hwf.childBound x hx
  · intro c hc hlive q hpar
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hc
    have hxl : x.protectsRoot := by unfold PinEntry.protectsRoot at *; rwa [hsst] at hlive
    obtain ⟨p, hp, hpid, hpl⟩ := hwf.parentLive x hx hxl q (by rwa [hspar] at hpar)
    refine ⟨setLoans id f p, List.mem_map_of_mem hp, by rw [hsid]; exact hpid, ?_⟩
    unfold PinEntry.protectsRoot at *; rwa [hsst]
  · intro e he
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
    rw [hsid, hspar]; exact hwf.noSelfParent x hx
  · intro e he hx
    obtain ⟨x, hxm, rfl⟩ := List.mem_map.mp he
    rw [hsst] at hx
    obtain ⟨hl, hc⟩ := hwf.expiredDrained x hxm hx
    refine ⟨?_, by rw [hskids]; exact hc⟩
    unfold setLoans
    by_cases hxid : x.id = id
    · simp [hxid]; exact hexp x hxm hxid hx
    · simp [hxid, hl]

theorem loanAcquire_wf {t t' : PinTable} {id : Nat}
    (hwf : WellFormed t) (h : loanAcquire t id = some t') : WellFormed t' := by
  unfold loanAcquire at h
  split at h
  · rename_i hany
    simp only [Option.some.injEq] at h; subst h
    obtain ⟨e0, he0, he0c⟩ := List.any_eq_true.mp hany
    simp only [Bool.and_eq_true, beq_iff_eq] at he0c
    refine setLoans_wf hwf (fun e he hid hexp => ?_)
    have := eq_of_mem_of_id_eq hwf.nodup he he0 (hid.trans he0c.1.symm)
    subst this; rw [he0c.2] at hexp; cases hexp
  · cases h

theorem loanRelease_wf {t t' : PinTable} {id : Nat}
    (hwf : WellFormed t) (h : loanRelease t id = some t') : WellFormed t' := by
  unfold loanRelease at h
  split at h
  · simp only [Option.some.injEq] at h; subst h
    refine setLoans_wf hwf (fun e he _ hexp => ?_)
    simp [(hwf.expiredDrained e he hexp).1]
  · cases h

theorem find_some {t : PinTable} {id : Nat} {e : PinEntry}
    (h : t.find? (fun e => e.id == id) = some e) : e ∈ t ∧ e.id = id :=
  ⟨List.mem_of_find?_eq_some h, by simpa using List.find?_some h⟩

theorem expireStep_id (x : PinEntry) : x.expireStep.id = x.id := by
  unfold PinEntry.expireStep; split <;> (try split) <;> rfl

theorem expireStep_parent (x : PinEntry) : x.expireStep.parent = x.parent := by
  unfold PinEntry.expireStep; split <;> (try split) <;> rfl

theorem expireStep_root (x : PinEntry) : x.expireStep.root = x.root := by
  unfold PinEntry.expireStep; split <;> (try split) <;> rfl

theorem expireStep_loans (x : PinEntry) :
    x.expireStep.activeReaderLoans = x.activeReaderLoans := by
  unfold PinEntry.expireStep; split <;> (try split) <;> rfl

theorem expireStep_kids (x : PinEntry) : x.expireStep.liveChildPins = x.liveChildPins := by
  unfold PinEntry.expireStep; split <;> (try split) <;> rfl

/-- An expiry step either keeps liveness or is exactly the completing transition. -/
theorem expireStep_live_iff (x : PinEntry) (hc : completes x = false) :
    (x.expireStep.state ≠ LeaseState.expired) ↔ (x.state ≠ LeaseState.expired) := by
  unfold PinEntry.expireStep
  unfold completes at hc
  cases hs : x.state <;> simp [hs] at hc ⊢
  split
  · rename_i hz; exfalso; exact absurd (hc hz.1) (by simp [hz.2])
  · simp [hs]

theorem expireAt_id (id : Nat) (par : Option Nat) (d : Bool) (x : PinEntry) :
    (expireAt id par d x).id = x.id := by
  unfold expireAt; split
  · exact expireStep_id x
  · split <;> rfl

theorem expireAt_parent (id : Nat) (par : Option Nat) (d : Bool) (x : PinEntry) :
    (expireAt id par d x).parent = x.parent := by
  unfold expireAt; split
  · exact expireStep_parent x
  · split <;> rfl

theorem expireAt_root (id : Nat) (par : Option Nat) (d : Bool) (x : PinEntry) :
    (expireAt id par d x).root = x.root := by
  unfold expireAt; split
  · exact expireStep_root x
  · split <;> rfl

theorem expireAt_loans (id : Nat) (par : Option Nat) (d : Bool) (x : PinEntry) :
    (expireAt id par d x).activeReaderLoans = x.activeReaderLoans := by
  unfold expireAt; split
  · exact expireStep_loans x
  · split <;> rfl

/-- Liveness only ever drops, and only on the expired entry itself. -/
theorem expireAt_live_mono (id : Nat) (par : Option Nat) (d : Bool) (x : PinEntry) :
    (expireAt id par d x).state ≠ LeaseState.expired → x.state ≠ LeaseState.expired := by
  intro h hx
  apply h
  unfold expireAt; split
  · unfold PinEntry.expireStep; split <;> (try split) <;> simp_all
  · split <;> exact hx

theorem expireAt_other_state {id : Nat} {par : Option Nat} {d : Bool} {x : PinEntry}
    (hx : x.id ≠ id) : (expireAt id par d x).state = x.state := by
  unfold expireAt; simp only [hx, ite_false]; split <;> rfl

theorem leaseExpire_wf {t t' : PinTable} {id : Nat}
    (hwf : WellFormed t) (h : leaseExpire t id = some t') : WellFormed t' := by
  unfold leaseExpire at h
  split at h
  · cases h
  rename_i e hfind
  simp only [Option.some.injEq] at h; subst h
  obtain ⟨he, heid⟩ := find_some hfind
  have hself : ∀ x ∈ t, x.id = id → x = e :=
    fun x hx hxid => eq_of_mem_of_id_eq hwf.nodup hx he (hxid.trans heid.symm)
  have hnodup : ((t.map (expireAt id e.parent (completes e))).map PinEntry.id).Nodup := by
    rw [ids_map (expireAt_id id e.parent (completes e))]; exact hwf.nodup
  have hmono : ∀ q, ∀ x ∈ t,
      liveChildOf q (expireAt id e.parent (completes e) x) = true → liveChildOf q x = true := by
    intro q x _ hl
    simp only [liveChildOf, Bool.and_eq_true, beq_iff_eq, bne_iff_ne, ne_eq,
      expireAt_parent] at hl ⊢
    exact ⟨hl.1, expireAt_live_mono id e.parent (completes e) x hl.2⟩
  have hle : ∀ q, childCount (t.map (expireAt id e.parent (completes e))) q ≤ childCount t q :=
    fun q => countP_map_le t (hmono q)
  cases hdone : completes e with
  | false =>
      rw [hdone] at hnodup hmono hle
      -- Liveness is unchanged everywhere; counters untouched.
      have hlive_eq : ∀ x ∈ t, (expireAt id e.parent false x).state ≠ LeaseState.expired ↔
          x.state ≠ LeaseState.expired := by
        intro x hx
        by_cases hxid : x.id = id
        · have := hself x hx hxid; subst this
          unfold expireAt; simp only [hxid, ite_true]; exact expireStep_live_iff _ hdone
        · rw [expireAt_other_state hxid]
      have hkids : ∀ x ∈ t, (expireAt id e.parent false x).liveChildPins = x.liveChildPins := by
        intro x _; unfold expireAt; split
        · exact expireStep_kids x
        · simp
      refine ⟨hnodup, ?_, ?_, ?_, ?_⟩
      · intro y hy
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        rw [expireAt_id, hkids x hx]
        exact Nat.le_trans (hle x.id) (hwf.childBound x hx)
      · intro c hc hlive q hpar
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hc
        rw [expireAt_parent] at hpar
        have hxl := (hlive_eq x hx).mp hlive
        obtain ⟨p, hp, hpid, hpl⟩ := hwf.parentLive x hx hxl q hpar
        exact ⟨_, List.mem_map_of_mem hp, by rw [expireAt_id]; exact hpid,
          (hlive_eq p hp).mpr hpl⟩
      · intro y hy
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        rw [expireAt_id, expireAt_parent]; exact hwf.noSelfParent x hx
      · intro y hy hexp
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        have hxexp : x.state = LeaseState.expired := by
          apply Classical.byContradiction; intro hne; exact (hlive_eq x hx).mpr hne hexp
        rw [expireAt_loans, hkids x hx]; exact hwf.expiredDrained x hx hxexp
  | true =>
      rw [hdone] at hnodup hmono hle
      unfold completes at hdone
      simp only [Bool.and_eq_true, beq_iff_eq] at hdone
      obtain ⟨⟨hes, hel⟩, hek⟩ := hdone
      have hnochild : childCount t e.id = 0 := by
        have := hwf.childBound e he; omega
      have hnolive : ∀ c ∈ t, c.parent = some e.id → c.state = LeaseState.expired := by
        intro c hc hpar
        apply Classical.byContradiction; intro hne
        have hpos : 0 < childCount t e.id := by
          unfold childCount
          exact List.countP_pos_iff.mpr ⟨c, hc, by simp [liveChildOf, hpar, hne]⟩
        omega
      have hparent_ne : ∀ pid, e.parent = some pid → pid ≠ id := by
        intro pid hp heq; subst heq; exact hwf.noSelfParent e he (heid ▸ hp)
      refine ⟨hnodup, ?_, ?_, ?_, ?_⟩
      · intro y hy
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        rw [expireAt_id]
        by_cases hxid : x.id = id
        · have := hself x hx hxid; subst this
          have hk : (expireAt id x.parent true x).liveChildPins = x.liveChildPins := by
            unfold expireAt; simp only [hxid, ite_true, expireStep_kids]
          rw [hk]
          have h1 := hle x.id
          omega
        · by_cases hxp : e.parent = some x.id
          · -- `x` is the parent: the expired child leaves its live-child count.
            have hstrict := countP_map_lt (g := expireAt id e.parent true)
              (p := liveChildOf x.id) t (hmono x.id) he
              (by simp [liveChildOf, hxp, hes])
              (by
                simp only [liveChildOf, expireAt_parent, hxp, beq_self_eq_true, Bool.true_and,
                  bne_eq_false_iff_eq]
                unfold expireAt; simp only [heid, ite_true]
                unfold PinEntry.expireStep; simp [hes, hel, hek])
            have hb := hwf.childBound x hx
            have hk : (expireAt id e.parent true x).liveChildPins = x.liveChildPins - 1 := by
              unfold expireAt; simp [hxid, hxp]
            rw [hk]
            unfold childCount at hb ⊢; omega
          · have hk : (expireAt id e.parent true x).liveChildPins = x.liveChildPins := by
              unfold expireAt
              simp only [hxid, ite_false]
              split
              · rename_i hc; simp at hc; exact absurd hc hxp
              · rfl
            rw [hk]
            exact Nat.le_trans (hle x.id) (hwf.childBound x hx)
      · intro c hc hlive q hpar
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hc
        rw [expireAt_parent] at hpar
        have hxl := expireAt_live_mono id e.parent true x hlive
        obtain ⟨p, hp, hpid, hpl⟩ := hwf.parentLive x hx hxl q hpar
        refine ⟨_, List.mem_map_of_mem hp, by rw [expireAt_id]; exact hpid, ?_⟩
        have hpne : p.id ≠ id := by
          intro hpi
          have := hself p hp hpi; subst this
          exact hxl (hnolive x hx (hpar.trans (congrArg some hpid.symm)))
        unfold PinEntry.protectsRoot; rw [expireAt_other_state hpne]; exact hpl
      · intro y hy
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        rw [expireAt_id, expireAt_parent]; exact hwf.noSelfParent x hx
      · intro y hy hexp
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        rw [expireAt_loans]
        by_cases hxid : x.id = id
        · have := hself x hx hxid; subst this
          unfold expireAt; simp only [hxid, ite_true, expireStep_kids]
          exact ⟨hel, hek⟩
        · rw [expireAt_other_state hxid] at hexp
          obtain ⟨hl, hk⟩ := hwf.expiredDrained x hx hexp
          refine ⟨hl, ?_⟩
          unfold expireAt; simp only [hxid, ite_false]; split <;> simp [hk]

theorem Reachable.wf {t : PinTable} (h : Reachable t) : WellFormed t := by
  induction h with
  | empty => exact wf_empty
  | viaAcquire _ hs ih => exact pinAcquire_wf ih hs
  | viaLoanAcquire _ hs ih => exact loanAcquire_wf ih hs
  | viaLoanRelease _ hs ih => exact loanRelease_wf ih hs
  | viaExpire _ hs ih => exact leaseExpire_wf ih hs

/-! ## Transitive overlay reachability -/

theorem holdsChain_live {t : PinTable} (hwf : WellFormed t) {e : PinEntry} (he : e ∈ t)
    (h : holdsChain e) : e.protectsRoot := by
  rcases h with h | h
  · exact h
  · intro hexp; have := (hwf.expiredDrained e he hexp).1; omega

/-- Every ancestor of a live pin is itself present and live. -/
theorem ancestor_live {t : PinTable} (hwf : WellFormed t) {e a : PinEntry}
    (he : e ∈ t) (hl : e.protectsRoot) (hanc : OverlayAncestor t e a) :
    a ∈ t ∧ a.protectsRoot := by
  induction hanc with
  | refl => exact ⟨he, hl⟩
  | step _ hp hpar ih =>
      obtain ⟨hc, hcl⟩ := ih
      obtain ⟨p', hp', hpid, hpl⟩ := hwf.parentLive _ hc hcl _ hpar
      have := eq_of_mem_of_id_eq hwf.nodup hp' hp hpid
      subst this
      exact ⟨hp', hpl⟩

/-- **`StoreGCPinSafety`** (milestone-4 §9.1 row 1).

For every pin table reachable through the kernel transitions, a reclamation set that is
disjoint from the *direct* projection consulted by production (`liveRoots` — current roots,
in-process read registrations and in-flight decisions — plus non-expired pin entries) never
reclaims any root in the full reachability law: live roots, durable pins, active reader
loans, and every transitive nested parent overlay. -/
theorem StoreGCPinSafety (liveRoots : Nat → Prop) (t : PinTable) (reclaimSet : Nat → Prop)
    (hreach : Reachable t)
    (hplan : ∀ r, reclaimSet r → ¬ projectedReach liveRoots t r) :
    ∀ r, fullReach liveRoots t r → ¬ reclaimSet r := by
  intro r hfull hrec
  apply hplan r hrec
  have hwf := hreach.wf
  rcases hfull with hlive | ⟨e, he, hchain, a, hanc, hroot⟩
  · exact Or.inl hlive
  · right
    obtain ⟨ha, hal⟩ := ancestor_live hwf he (holdsChain_live hwf he hchain) hanc
    exact ⟨a, ha, hroot, hal⟩

/-- **`LeaseExpiryNoReaderLoss`** (milestone-4 §3.2 / §4.1).

One `expire_lease` step on a reachable table:
1. never drops protection of a pin that still has an active reader loan or a live child
   overlay (its root stays protected with the same root);
2. reaches `Expired` only when `active_reader_loans == 0 && live_child_pins == 0` and no live
   child overlay remains;
3. fences new reader loans on the expired pin (`Expiring` and `Expired` refuse loans);
4. keeps every surviving live pin's whole transitive overlay chain live. -/
theorem LeaseExpiryNoReaderLoss (t t' : PinTable) (id : Nat)
    (hreach : Reachable t) (hstep : leaseExpire t id = some t') :
    (∀ e ∈ t, e.protectsRoot → (e.activeReaderLoans > 0 ∨ childCount t e.id > 0) →
        ∃ e' ∈ t', e'.id = e.id ∧ e'.root = e.root ∧ e'.protectsRoot) ∧
    (∀ e' ∈ t', e'.id = id → e'.state = LeaseState.expired →
        e'.activeReaderLoans = 0 ∧ e'.liveChildPins = 0 ∧ childCount t' id = 0) ∧
    (∀ e' ∈ t', e'.id = id → e'.state ≠ LeaseState.active → loanAcquire t' id = none) ∧
    (∀ e' ∈ t', e'.protectsRoot → ∀ a, OverlayAncestor t' e' a → a ∈ t' ∧ a.protectsRoot) := by
  have hwf := hreach.wf
  have hwf' := leaseExpire_wf hwf hstep
  have hstep' := hstep
  unfold leaseExpire at hstep
  split at hstep
  · cases hstep
  rename_i e0 hfind
  simp only [Option.some.injEq] at hstep; subst hstep
  obtain ⟨he0, he0id⟩ := find_some hfind
  refine ⟨?_, ?_, ?_, ?_⟩
  · intro e he hl hheld
    refine ⟨_, List.mem_map_of_mem he, expireAt_id _ _ _ e, expireAt_root _ _ _ e, ?_⟩
    by_cases heid : e.id = id
    · have := eq_of_mem_of_id_eq hwf.nodup he he0 (heid.trans he0id.symm); subst this
      unfold expireAt; simp only [heid, ite_true]
      have hkids : e.activeReaderLoans > 0 ∨ e.liveChildPins > 0 := by
        rcases hheld with h | h
        · exact Or.inl h
        · exact Or.inr (Nat.lt_of_lt_of_le h (hwf.childBound e he))
      exact expireStep_retains_protection_while_held e hl hkids
    · unfold PinEntry.protectsRoot; rw [expireAt_other_state heid]; exact hl
  · intro e' he' hid hexp
    obtain ⟨hl, hk⟩ := hwf'.expiredDrained e' he' hexp
    refine ⟨hl, hk, ?_⟩
    have := hwf'.childBound e' he'
    rw [hid] at this; omega
  · intro e' he' hid hna
    unfold loanAcquire
    have : ¬ (t.map (expireAt id e0.parent (completes e0))).any
        (fun e => e.id == id && e.state == LeaseState.active) = true := by
      intro hany
      obtain ⟨y, hy, hyc⟩ := List.any_eq_true.mp hany
      simp only [Bool.and_eq_true, beq_iff_eq] at hyc
      have := eq_of_mem_of_id_eq hwf'.nodup hy he' (hyc.1.trans hid.symm)
      subst this; exact hna hyc.2
    simp [this]
  · intro e' he' hl a hanc
    exact ancestor_live hwf' he' hl hanc

/-- Progress witness: a drained `Expiring` pin completes `Expiring -> Expired`. -/
theorem expire_completes_when_drained (t : PinTable) (e : PinEntry)
    (hreach : Reachable t) (he : e ∈ t) (hs : e.state = LeaseState.expiring)
    (hl : e.activeReaderLoans = 0) (hk : e.liveChildPins = 0) :
    ∃ t', leaseExpire t e.id = some t' ∧
      ∀ e' ∈ t', e'.id = e.id → e'.state = LeaseState.expired := by
  have hwf := hreach.wf
  unfold leaseExpire
  split
  · rename_i hnone
    exfalso
    have := List.find?_eq_none.mp hnone e he
    simp at this
  · rename_i e0 hfind
    obtain ⟨he0, he0id⟩ := find_some hfind
    have := eq_of_mem_of_id_eq hwf.nodup he0 he he0id; subst this
    refine ⟨_, rfl, ?_⟩
    intro y hy hyid
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
    rw [expireAt_id] at hyid
    have := eq_of_mem_of_id_eq hwf.nodup hx he0 hyid; subst this
    unfold expireAt; simp only [ite_true]
    unfold PinEntry.expireStep; simp [hs, hl, hk]

end StoreM4.GCPinSafety
