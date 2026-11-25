import CoupledSuccession.RunLemmas

/-!
# Structural invariant of the live run, and compaction = `R(E)` as logs

`compactAll (liveRun b steps).log = refLog steps`: compacting the live, masked log yields the
reference log of `R(E)` built directly from the appended members. No guard is needed for this
syntactic fact; the guards are what make its fold equal the live store (`MultiChain`).
-/
namespace CoupledSuccession

theorem updateChain_self (m : Nat → Option ChainSt) (c : Nat) (v : Option ChainSt) :
    updateChain m c v c = v := by simp [updateChain]

theorem updateChain_other {m : Nat → Option ChainSt} {c c' : Nat} (v : Option ChainSt)
    (h : c' ≠ c) : updateChain m c v c' = m c' := by simp [updateChain, h]

theorem stepLive_foreign (l : Live) (f : Eff) :
    stepLive l (.foreign f) =
      { store := apply f l.store
        log := l.log ++ [⟨.foreign l.log.length, f, true, false⟩]
        chains := l.chains } := rfl

theorem stepLive_head {l : Live} {c : Nat} (e : Eff) (hc : l.chains c = none) :
    stepLive l (.chain c e) =
      { store := apply e l.store
        log := l.log ++ [⟨.chain c, e, true, false⟩]
        chains := updateChain l.chains c (some ⟨l.store, e.dom, l.log.length, e⟩) } := by
  simp [stepLive, hc]

theorem stepLive_succ {l : Live} {c : Nat} {cs : ChainSt} (e : Eff) (hc : l.chains c = some cs) :
    stepLive l (.chain c e) =
      { store := apply e (retract cs.base cs.live.dom l.store)
        log := maskOwner (.chain c) l.log ++ [⟨.chain c, e, false, false⟩]
        chains := updateChain l.chains c
          (some { cs with touched := unionDom cs.touched e.dom, live := e }) } := by
  simp [stepLive, hc]

/-- Structural invariant of the live run. -/
structure StructInv (l : Live) : Prop where
  foreignFresh : ∀ s ∈ l.log, ∀ n, s.owner = .foreign n → n < l.log.length
  chainKnown : ∀ s ∈ l.log, ∀ c, s.owner = .chain c → (l.chains c).isSome
  liveSeg : ∀ c cs, l.chains c = some cs → ∃ i y, liveOf l.log (.chain c) = some y ∧
    l.log[i]? = some y ∧ cs.head ≤ i ∧ y.eff = cs.live
  liveSub : ∀ c cs, l.chains c = some cs → Sub cs.live.dom cs.touched

theorem structInv_init (b : Store) : StructInv (Live.init b) where
  foreignFresh := by intro s hs; simp [Live.init] at hs
  chainKnown := by intro s hs; simp [Live.init] at hs
  liveSeg := by intro c cs h; simp [Live.init] at h
  liveSub := by intro c cs h; simp [Live.init] at h

theorem chain_fresh {l : Live} (hI : StructInv l) {c : Nat} (hc : l.chains c = none) :
    ∀ s ∈ l.log, s.owner ≠ .chain c := by
  intro s hs ho
  have := hI.chainKnown s hs c ho
  rw [hc] at this; cases this

theorem structInv_foreign {l : Live} (hI : StructInv l) (f : Eff) :
    StructInv (stepLive l (.foreign f)) := by
  rw [stepLive_foreign]
  constructor <;> dsimp only
  · intro s hs n hn
    simp only [List.length_append, List.length_singleton]
    rcases List.mem_append.mp hs with hs | hs
    · exact Nat.lt_succ_of_lt (hI.foreignFresh s hs n hn)
    · rw [List.mem_singleton] at hs; subst hs
      simp only [Owner.foreign.injEq] at hn; omega
  · intro s hs c hc
    rcases List.mem_append.mp hs with hs | hs
    · exact hI.chainKnown s hs c hc
    · rw [List.mem_singleton] at hs; subst hs; cases hc
  · intro c cs hc
    obtain ⟨i, y, h1, h2, h3, h4⟩ := hI.liveSeg c cs hc
    exact ⟨i, y, by rw [liveOf_append_other (by simp)]; exact h1, getElem?_append_lt h2, h3, h4⟩
  · exact hI.liveSub

theorem structInv_head {l : Live} (hI : StructInv l) {c : Nat} (e : Eff) (hc : l.chains c = none) :
    StructInv (stepLive l (.chain c e)) := by
  rw [stepLive_head e hc]
  have hfresh := chain_fresh hI hc
  constructor <;> dsimp only
  · intro s hs n hn
    simp only [List.length_append, List.length_singleton]
    rcases List.mem_append.mp hs with hs | hs
    · exact Nat.lt_succ_of_lt (hI.foreignFresh s hs n hn)
    · rw [List.mem_singleton] at hs; subst hs; cases hn
  · intro s hs c' hc'
    by_cases h : c' = c
    · subst h; simp [updateChain_self]
    · rw [updateChain_other _ h]
      rcases List.mem_append.mp hs with hs | hs
      · exact hI.chainKnown s hs c' hc'
      · rw [List.mem_singleton] at hs; subst hs
        simp only [Owner.chain.injEq] at hc'; exact absurd hc'.symm h
  · intro c' cs hc'
    by_cases h : c' = c
    · subst h
      rw [updateChain_self] at hc'
      cases hc'
      refine ⟨l.log.length, _, liveOf_append_new (liveOf_none_of_fresh hfresh) rfl rfl,
        List.getElem?_concat_length, Nat.le_refl _, rfl⟩
    · rw [updateChain_other _ h] at hc'
      obtain ⟨i, y, h1, h2, h3, h4⟩ := hI.liveSeg c' cs hc'
      refine ⟨i, y, ?_, getElem?_append_lt h2, h3, h4⟩
      rw [liveOf_append_other (by simp only [ne_eq, Owner.chain.injEq]; exact fun e => h e.symm)]
      exact h1
  · intro c' cs hc'
    by_cases h : c' = c
    · subst h
      rw [updateChain_self] at hc'
      cases hc'
      exact fun r hr => hr
    · rw [updateChain_other _ h] at hc'
      exact hI.liveSub c' cs hc'

theorem structInv_succ {l : Live} (hI : StructInv l) {c : Nat} {cs : ChainSt} (e : Eff)
    (hc : l.chains c = some cs) : StructInv (stepLive l (.chain c e)) := by
  rw [stepLive_succ e hc]
  obtain ⟨i0, y0, hy0, hi0, hh0, _⟩ := hI.liveSeg c cs hc
  have hlt0 := lt_length_of_getElem? hi0
  constructor <;> dsimp only
  · intro s hs n hn
    simp only [List.length_append, List.length_singleton, length_maskOwner]
    rcases List.mem_append.mp hs with hs | hs
    · obtain ⟨t, ht, hst⟩ := mem_maskOwner hs
      exact Nat.lt_succ_of_lt (hI.foreignFresh t ht n (hst ▸ hn))
    · rw [List.mem_singleton] at hs; subst hs; cases hn
  · intro s hs c' hc'
    by_cases h : c' = c
    · subst h; simp [updateChain_self]
    · rw [updateChain_other _ h]
      rcases List.mem_append.mp hs with hs | hs
      · obtain ⟨t, ht, hst⟩ := mem_maskOwner hs
        exact hI.chainKnown t ht c' (hst ▸ hc')
      · rw [List.mem_singleton] at hs; subst hs
        simp only [Owner.chain.injEq] at hc'; exact absurd hc'.symm h
  · intro c' cs' hc'
    by_cases h : c' = c
    · subst h
      rw [updateChain_self] at hc'
      cases hc'
      refine ⟨l.log.length, _, liveOf_append_new (liveOf_mask_self _ _) rfl rfl, ?_, ?_, rfl⟩
      · have := @List.getElem?_concat_length _ (maskOwner (.chain c') l.log)
          ⟨.chain c', e, false, false⟩
        rw [length_maskOwner] at this; exact this
      · exact Nat.le_of_lt (Nat.lt_of_le_of_lt hh0 hlt0)
    · rw [updateChain_other _ h] at hc'
      have hne : Owner.chain c' ≠ Owner.chain c := by
        simp only [ne_eq, Owner.chain.injEq]; exact h
      obtain ⟨i, y, h1, h2, h3, h4⟩ := hI.liveSeg c' cs' hc'
      have hyo := (liveOf_some h1).2.1
      refine ⟨i, y, ?_, getElem?_append_lt (getElem?_maskOwner_of_ne h2 (hyo ▸ hne)), h3, h4⟩
      rw [liveOf_append_other (by simp only [ne_eq, Owner.chain.injEq]; exact fun e => h e.symm),
        liveOf_mask_other hne]
      exact h1
  · intro c' cs' hc'
    by_cases h : c' = c
    · subst h
      rw [updateChain_self] at hc'
      cases hc'
      intro r hr
      dsimp only at hr
      simp [unionDom, hr]
    · rw [updateChain_other _ h] at hc'
      exact hI.liveSub c' cs' hc'

theorem structInv_step {l : Live} (hI : StructInv l) (st : Step) : StructInv (stepLive l st) := by
  cases st with
  | foreign f => exact structInv_foreign hI f
  | chain c e =>
    cases hc : l.chains c with
    | none => exact structInv_head hI e hc
    | some cs => exact structInv_succ hI e hc

theorem structInv_runFrom : ∀ (steps : List Step) {l : Live}, StructInv l →
    StructInv (runFrom l steps) := by
  intro steps
  induction steps with
  | nil => intro l h; exact h
  | cons st ss ih => intro l h; exact ih (structInv_step h st)

theorem structInv_liveRun (b : Store) (steps : List Step) : StructInv (liveRun b steps) :=
  structInv_runFrom steps (structInv_init b)

/-- Lockstep: compaction of the live log follows `refFrom`. -/
theorem compact_runFrom_eq_refFrom : ∀ (steps : List Step) (l : Live) (acc : List Seg) (n : Nat)
    (seen : Nat → Bool), StructInv l → compactAll l.log = acc → l.log.length = n →
    (∀ c, seen c = (l.chains c).isSome) →
    compactAll (runFrom l steps).log = refFrom acc n seen steps := by
  intro steps
  induction steps with
  | nil => intro l acc n seen _ h _ _; exact h
  | cons st ss ih =>
    intro l acc n seen hI hacc hn hseen
    cases st with
    | foreign f =>
      apply ih _ _ _ _ (structInv_foreign hI f)
      · rw [stepLive_foreign]
        simp only
        rw [compactAll_append_fresh _ rfl rfl, hacc, hn]
        intro s hs ho
        have := hI.foreignFresh s hs l.log.length ho
        omega
      · simp [stepLive_foreign, hn]
      · intro c; rw [hseen c]; rfl
    | chain c e =>
      cases hc : l.chains c with
      | none =>
        have hs : seen c = false := by rw [hseen c, hc]; rfl
        simp only [refFrom, hs]
        apply ih _ _ _ _ (structInv_head hI e hc)
        · rw [stepLive_head e hc]
          simp only
          rw [compactAll_append_fresh (chain_fresh hI hc) rfl rfl, hacc]
        · simp [stepLive_head e hc, hn]
        · intro c'
          rw [stepLive_head e hc]
          simp only
          by_cases h : c' = c
          · subst h; simp [updateChain_self]
          · rw [updateChain_other _ h, ← hseen c']; simp [h]
      | some cs =>
        have hs : seen c = true := by rw [hseen c, hc]; rfl
        simp only [refFrom, hs, ite_true]
        apply ih _ _ _ _ (structInv_succ hI e hc)
        · rw [stepLive_succ e hc]
          simp only
          obtain ⟨_, y, hy, _⟩ := hI.liveSeg c cs hc
          rw [compactAll_succ (o := .chain c) (x := ⟨.chain c, e, false, false⟩)
            (by rw [hy]; rfl) rfl rfl rfl, hacc]
        · simp [stepLive_succ e hc, length_maskOwner, hn]
        · intro c'
          rw [stepLive_succ e hc]
          simp only
          by_cases h : c' = c
          · subst h; simp [updateChain_self, hs]
          · rw [updateChain_other _ h, ← hseen c']

/-- Compaction of the live log is exactly the reference log of `R(E)`. -/
theorem compactAll_liveRun_eq_refLog (b : Store) (steps : List Step) :
    compactAll (liveRun b steps).log = refLog steps :=
  compact_runFrom_eq_refFrom steps _ _ _ _ (structInv_init b) rfl rfl (fun _ => rfl)

end CoupledSuccession
