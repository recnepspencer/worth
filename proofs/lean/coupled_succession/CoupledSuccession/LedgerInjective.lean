import CoupledSuccession.Ledger

/-!
# No ABA inside one candidate (design §2.5 M-N4; manifest T8i)

Withdraw, recreate, fresh reservation and completion never give one id to two keys: ids are
pairwise distinct and below the frontier, and cancelled entries keep holding their id.
-/
namespace CoupledSuccession.Ledger

open CoupledSuccession

/-- Ledger well-formedness: ids below the frontier and pairwise distinct. -/
structure WF (l : Ledger) : Prop where
  below : ∀ e ∈ l.entries, e.id < l.frontier
  distinct : l.entries.Pairwise (fun a b => a.id ≠ b.id)

theorem wf_empty : WF empty := ⟨by simp [empty], by simp [empty]⟩

theorem wf_map {l : Ledger} (h : WF l) (f : LedgerEntry → LedgerEntry) (hf : ∀ e, (f e).id = e.id) :
    WF { l with entries := l.entries.map f } := by
  constructor
  · intro e he
    obtain ⟨a, ha, rfl⟩ := List.mem_map.mp he
    rw [hf]; exact h.below a ha
  · rw [List.pairwise_map]
    exact h.distinct.imp (fun hab => by rw [hf, hf]; exact hab)

theorem wf_push {l : Ledger} (h : WF l) (k : Key) (n fr : Nat) (hn : n < fr) (hfr : l.frontier ≤ fr)
    (hfree : ∀ e ∈ l.entries, e.id ≠ n) : WF (l.push k n fr) := by
  unfold push
  constructor <;> dsimp only
  · intro e he
    rcases List.mem_append.mp he with he | he
    · exact Nat.lt_of_lt_of_le (h.below e he) hfr
    · rw [List.mem_singleton] at he; subst he; exact hn
  · rw [List.pairwise_append]
    refine ⟨h.distinct, List.pairwise_singleton _ _, ?_⟩
    intro a ha b hb
    rw [List.mem_singleton] at hb; subst hb
    exact hfree a ha

theorem wf_withdraw {l : Ledger} (h : WF l) (k : Key) : WF (l.withdraw k) :=
  wf_map h _ (fun e => by split <;> rfl)

theorem wf_rebind {l : Ledger} (h : WF l) (k : Key) : WF (l.rebind k) :=
  wf_map h _ (fun e => by split <;> rfl)

theorem wf_complete {l : Ledger} (h : WF l) (keep : Key → Bool) : WF (l.complete keep) :=
  wf_map h _ (fun e => by split <;> rfl)

theorem wf_recreate {l : Ledger} (h : WF l) (k : Key) : WF (l.recreate k).2 := by
  unfold recreate
  cases l.lookup k with
  | some id => exact wf_rebind h k
  | none =>
    exact wf_push h k _ _ (Nat.lt_succ_self _) (Nat.le_succ _)
      (fun e he heq => Nat.lt_irrefl _ (heq ▸ h.below e he))

theorem wf_fresh {l : Ledger} (h : WF l) (used : Nat → Bool) (k : Key) : WF (l.fresh used k).2 := by
  unfold fresh
  cases l.lookup k with
  | some _ => exact wf_recreate h k
  | none =>
    dsimp only
    cases hr : l.reclaimable used with
    | some n =>
      have hmem := List.mem_of_find?_eq_some hr
      have hp := List.find?_some hr
      rw [List.mem_range] at hmem
      simp only [Bool.and_eq_true, Bool.not_eq_true'] at hp
      refine wf_push h k _ _ hmem (Nat.le_refl _) ?_
      intro e he heq
      have : l.held n = true := by
        unfold held; rw [List.any_eq_true]; exact ⟨e, he, by simp [heq]⟩
      rw [this] at hp; cases hp.2
    | none =>
      exact wf_push h k _ _ (Nat.lt_succ_self _) (Nat.le_succ _)
        (fun e he heq => Nat.lt_irrefl _ (heq ▸ h.below e he))

theorem wf_step {l : Ledger} (h : WF l) (op : LedgerOp) : WF (l.step op) := by
  cases op with
  | withdraw k => exact wf_withdraw h k
  | recreate k => exact wf_recreate h k
  | fresh used k => exact wf_fresh h used k
  | complete keep => exact wf_complete h keep

theorem wf_foldl : ∀ (ops : List LedgerOp) {l : Ledger}, WF l → WF (ops.foldl step l) := by
  intro ops
  induction ops with
  | nil => intro l h; exact h
  | cons op ops ih => intro l h; exact ih (wf_step h op)

theorem wf_run (ops : List LedgerOp) : WF (run ops) := wf_foldl ops wf_empty

theorem eq_of_pairwise_id {l : List LedgerEntry} (h : l.Pairwise (fun a b => a.id ≠ b.id)) :
    ∀ {a b : LedgerEntry}, a ∈ l → b ∈ l → a.id = b.id → a = b := by
  induction l with
  | nil => intro a b ha; cases ha
  | cons x xs ih =>
    intro a b ha hb hab
    rw [List.pairwise_cons] at h
    rcases List.mem_cons.mp ha with ha' | ha' <;> rcases List.mem_cons.mp hb with hb' | hb'
    · rw [ha', hb']
    · rw [ha'] at hab; exact absurd hab (h.1 b hb')
    · rw [hb'] at hab; exact absurd hab.symm (h.1 a ha')
    · exact ih h.2 ha' hb' hab

theorem lookup_some {l : Ledger} {k : Key} {id : Nat} (h : l.lookup k = some id) :
    ∃ e ∈ l.entries, e.key = k ∧ e.id = id := by
  unfold lookup at h
  cases hf : l.entries.find? (isLive k) with
  | none => rw [hf] at h; cases h
  | some e =>
    rw [hf] at h
    cases h
    have hp := List.find?_some hf
    simp only [isLive, Bool.and_eq_true, decide_eq_true_eq] at hp
    exact ⟨e, List.mem_of_find?_eq_some hf, hp.1, rfl⟩

/-- T8i: no ABA across withdraw, recreate and fresh: one id, one key. -/
theorem ledger_injective :
    ∀ (ops : List LedgerOp) (k1 k2 : Key) (id : Nat), (run ops).lookup k1 = some id →
      (run ops).lookup k2 = some id → k1 = k2 := by
  intro ops k1 k2 id h1 h2
  obtain ⟨e1, he1, hk1, hi1⟩ := lookup_some h1
  obtain ⟨e2, he2, hk2, hi2⟩ := lookup_some h2
  have := eq_of_pairwise_id (wf_run ops).distinct he1 he2 (hi1.trans hi2.symm)
  rw [← hk1, ← hk2, this]

end CoupledSuccession.Ledger
