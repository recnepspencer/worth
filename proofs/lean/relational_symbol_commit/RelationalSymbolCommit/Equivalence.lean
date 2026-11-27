import RelationalSymbolCommit.Dictionary

/-!
# Relational symbol commit: delta import equals full-snapshot import (public #446)

`delta_import_eq_full_snapshot`: importing only a commit's touched meanings
into a dictionary gives exactly the dictionary that importing its whole
canonical snapshot followed by those meanings gives, and refuses exactly
when that full import refuses (`delta_import_refuses_iff_full_snapshot`).

`importAll_preserves`: a successful import keeps the two maps inverse (one
meaning per symbol and per text), keeps every existing meaning, never
lowers the identity frontier and installs every imported meaning.
`commit_preserves`: one commit (reservation, then the cutover import into
the owner) keeps those invariants and gives every touched text a symbol.
-/
namespace RelationalSymbolCommit

theorem aliases_false {d : Dict} {s t : Nat} (h : aliases d s t = false) :
    (∀ t', d.bySym s = some t' → t' = t) ∧ (∀ s', d.byVal t = some s' → s' = s) := by
  unfold aliases at h
  simp only [Bool.or_eq_false_iff] at h
  obtain ⟨h1, h2⟩ := h
  constructor
  · intro t' ht; rw [ht] at h1; simpa using h1
  · intro s' hs; rw [hs] at h2; simpa using h2

theorem importMeaning_ok {cap : Nat} {d d' : Dict} {s t : Nat}
    (h : (importMeaning cap d s t).1 = some d') :
    aliases d s t = false ∧ d' = write d s t := by
  unfold importMeaning at h
  split at h
  · simp at h
  · rename_i hc
    simp only [Option.some.injEq] at h
    simp only [Bool.or_eq_true, beq_iff_eq, not_or] at hc
    exact ⟨by simpa using hc.1, h.symm⟩

theorem write_wf {d : Dict} {s t : Nat} (hw : Wf d) (ha : aliases d s t = false) :
    Wf (write d s t) := by
  obtain ⟨hinv, hfront⟩ := hw
  obtain ⟨hs, ht⟩ := aliases_false ha
  constructor
  · intro x y
    simp only [write]
    by_cases hx : x = s <;> by_cases hy : y = t
    · subst hx; subst hy; simp
    · subst hx
      simp only [ite_true, hy, ite_false, Option.some.injEq]
      constructor
      · intro h; exact absurd h.symm hy
      · intro h; exact absurd (hs y ((hinv x y).mpr h)) hy
    · subst hy
      simp only [hx, ite_false, ite_true, Option.some.injEq]
      constructor
      · intro h; exact absurd (ht x ((hinv x y).mp h)) hx
      · intro h; exact absurd h.symm hx
    · simp only [hx, hy, ite_false]; exact hinv x y
  · intro x y h
    simp only [write] at h ⊢
    by_cases hx : x = s
    · omega
    · simp only [hx, ite_false] at h; have := hfront x y h; omega

/-- Writing a non-aliasing meaning keeps every existing one. -/
theorem write_keeps {d : Dict} {s t : Nat} (ha : aliases d s t = false) :
    ∀ x y, d.bySym x = some y → (write d s t).bySym x = some y := by
  intro x y h
  simp only [write]
  by_cases hx : x = s
  · subst hx; simp only [ite_true, Option.some.injEq]; exact ((aliases_false ha).1 y h).symm
  · simp only [hx, ite_false]; exact h

/-- What a successful batch import guarantees. -/
theorem importAll_preserves (cap : Nat) : ∀ (d d' : Dict) (es : List (Nat × Nat)),
    Wf d → (importAll cap d es).1 = some d' →
      Wf d' ∧ (∀ s t, d.bySym s = some t → d'.bySym s = some t) ∧ d.next ≤ d'.next ∧
        ∀ e, e ∈ es → d'.bySym e.1 = some e.2
  | d, d', [], _, h => by
    simp only [importAll, Option.some.injEq] at h; subst h
    exact ⟨by assumption, fun _ _ h => h, Nat.le_refl _, by simp⟩
  | d, d', (s, t) :: es, hw, h => by
    simp only [importAll] at h
    split at h
    · simp at h
    · rename_i d₁ k hm
      simp only at h
      obtain ⟨ha, rfl⟩ := importMeaning_ok (cap := cap) (by rw [hm])
      obtain ⟨hw', hkeep, hnext, hinst⟩ :=
        importAll_preserves cap (write d s t) d' es (write_wf hw ha) h
      refine ⟨hw', fun x y hxy => hkeep x y (write_keeps ha x y hxy), ?_, ?_⟩
      · simp only [write] at hnext; omega
      · intro e he
        simp only [List.mem_cons] at he
        cases he with
        | inl he => subst he; exact hkeep s t (by simp [write])
        | inr he => exact hinst e he

theorem importAll_append (cap : Nat) : ∀ (d : Dict) (L es : List (Nat × Nat)),
    (importAll cap d (L ++ es)).1 = (importAll cap d L).1.bind (fun d' => (importAll cap d' es).1)
  | d, [], es => by simp [importAll]
  | d, (s, t) :: L, es => by
    simp only [List.cons_append, importAll]
    split
    · simp
    · simp only; exact importAll_append cap _ L es

/-- Importing a commit's touched meanings into the dictionary that a full
snapshot `L` rebuilds equals importing the whole snapshot and them. -/
theorem delta_import_eq_full_snapshot (cap : Nat) (L es : List (Nat × Nat)) (d : Dict)
    (h : (importFull cap L).1 = some d) :
    (importFull cap (L ++ es)).1 = (importAll cap d es).1 := by
  unfold importFull at h ⊢
  rw [importAll_append, h]; rfl

/-- The delta import refuses exactly the aliases the full import refuses. -/
theorem delta_import_refuses_iff_full_snapshot (cap : Nat) (L es : List (Nat × Nat))
    (d : Dict) (h : (importFull cap L).1 = some d) :
    (importFull cap (L ++ es)).1 = none ↔ (importAll cap d es).1 = none := by
  rw [delta_import_eq_full_snapshot cap L es d h]

theorem empty_wf : Wf empty := by
  constructor
  · intro s t; simp [empty]
  · intro s t h; simp [empty] at h

/-- A full snapshot that rebuilds at all rebuilds a well-formed dictionary. -/
theorem importFull_wf (cap : Nat) (L : List (Nat × Nat)) (d : Dict)
    (h : (importFull cap L).1 = some d) : Wf d :=
  (importAll_preserves cap empty d L empty_wf h).1

/-- What one successful reservation guarantees: the owner changes at most
its frontier, both dictionaries stay well formed, and the candidate holds
the returned meaning. -/
theorem reserve_preserves {cap : Nat} {owner cand owner' cand' : Dict} {t s : Nat}
    (ho : Wf owner) (hc : Wf cand)
    (h : (reserve cap owner cand t).1 = some (owner', cand', s)) :
    Wf owner' ∧ owner'.bySym = owner.bySym ∧ owner.next ≤ owner'.next ∧ Wf cand' ∧
      cand'.bySym s = some t := by
  unfold reserve at h
  split at h
  · rename_i s₀ hs
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl, rfl⟩ := h
    exact ⟨ho, rfl, Nat.le_refl _, hc, (hc.1 _ _).mpr hs⟩
  · split at h
    · rename_i s₀ _
      cases hm : (importMeaning cap cand s₀ t).1 with
      | none => rw [hm] at h; simp at h
      | some c =>
        rw [hm] at h; simp only [Option.map_some, Option.some.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl, rfl⟩ := h
        obtain ⟨ha, rfl⟩ := importMeaning_ok hm
        exact ⟨ho, rfl, Nat.le_refl _, write_wf hc ha, by simp [write]⟩
    · split at h
      · simp at h
      · cases hm : (importMeaning cap cand owner.next t).1 with
        | none => rw [hm] at h; simp at h
        | some c =>
          rw [hm] at h; simp only [Option.map_some, Option.some.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl, rfl⟩ := h
          obtain ⟨ha, rfl⟩ := importMeaning_ok hm
          refine ⟨⟨ho.1, fun x y hxy => ?_⟩, rfl, by simp, write_wf hc ha, by simp [write]⟩
          have := ho.2 x y hxy; simp only; omega

/-- What a successful reservation of every touched text guarantees. -/
theorem reserveAll_preserves (cap : Nat) : ∀ (owner cand : Dict) (ts : List Nat)
    (r : Dict × Dict × List (Nat × Nat)), Wf owner → Wf cand →
    (reserveAll cap owner cand ts).1 = some r →
      Wf r.1 ∧ r.1.bySym = owner.bySym ∧ owner.next ≤ r.1.next ∧ r.2.2.map Prod.snd = ts
  | owner, cand, [], r, ho, _, h => by
    simp only [reserveAll, Option.some.injEq] at h; subst h
    exact ⟨ho, rfl, Nat.le_refl _, rfl⟩
  | owner, cand, t :: ts, r, ho, hc, h => by
    simp only [reserveAll] at h
    split at h
    · simp at h
    · rename_i owner₁ cand₁ s k hr
      obtain ⟨ho₁, hsym₁, hnext₁, hc₁, _⟩ := reserve_preserves ho hc (by rw [hr])
      cases hrest : (reserveAll cap owner₁ cand₁ ts).1 with
      | none => rw [hrest] at h; simp at h
      | some r' =>
        rw [hrest] at h; simp only [Option.map_some, Option.some.injEq] at h
        subst h
        obtain ⟨ho', hsym', hnext', htexts⟩ :=
          reserveAll_preserves cap owner₁ cand₁ ts r' ho₁ hc₁ hrest
        exact ⟨ho', hsym'.trans hsym₁, Nat.le_trans hnext₁ hnext', by simp [htexts]⟩

/-- One successful commit keeps the owner well formed, keeps every existing
meaning, never lowers the frontier and gives every touched text a symbol. -/
theorem commit_preserves (cap : Nat) (owner d' : Dict) (touched : List Nat)
    (ho : Wf owner) (h : (commit cap owner touched).1 = some d') :
    Wf d' ∧ (∀ s t, owner.bySym s = some t → d'.bySym s = some t) ∧
      owner.next ≤ d'.next ∧ ∀ t, t ∈ touched → ∃ s, d'.byVal t = some s := by
  unfold commit at h
  split at h
  · simp at h
  · rename_i owner₁ cand meanings k hr
    simp only at h
    obtain ⟨ho₁, hsym, hnext₁, htexts⟩ :=
      reserveAll_preserves cap owner owner touched (owner₁, cand, meanings) ho ho (by rw [hr])
    obtain ⟨hw, hkeep, hnext, hinst⟩ := importAll_preserves cap owner₁ d' meanings ho₁ h
    simp only at hsym hnext₁ htexts
    refine ⟨hw, fun s t hst => hkeep s t (by rw [hsym]; exact hst), Nat.le_trans hnext₁ hnext, ?_⟩
    intro t ht
    rw [← htexts] at ht
    obtain ⟨e, he, rfl⟩ := List.mem_map.mp ht
    exact ⟨e.1, (hw.1 _ _).mp (hinst e he)⟩

end RelationalSymbolCommit
