import CoupledSuccession.MultiChainInv

/-!
# The multi-chain invariant across a successor round

Round `k` of chain `c` retracts `Σ_c` to `B_c`, masks `c`'s earlier entries and appends round
`k`. Backward, forward (Option A) and stale-base guards keep every chain's view exact.
-/
namespace CoupledSuccession

/-- The store step of a successor round, against the reference fold. -/
theorem succ_store (P B : Store) (T : Dom) (ℓ e : Eff) (post : List Seg)
    (hpost : ∀ s ∈ post, Disj s.eff.dom T ∧ Disj s.eff.dom e.dom) (hsub : Sub ℓ.dom T)
    (hbase : ∀ r, ℓ.dom r = true → B r = P r) :
    apply e (retract B ℓ.dom (foldSegments post (apply ℓ P))) = foldSegments post (apply e P) := by
  funext r
  unfold foldSegments
  cases hl : ℓ.dom r with
  | true =>
    have hT : T r = true := hsub r hl
    have hsk : ∀ s ∈ post, s.eff.dom r = false := fun s hs => disj_left (hpost s hs).1 hT
    rw [foldBy_skip _ _ hsk]
    apply apply_congr_pt
    simp [retract, hl, hbase r hl]
  | false =>
    have hre : retract B ℓ.dom (foldBy Seg.eff post (apply ℓ P)) r = foldBy Seg.eff post P r := by
      simp only [retract, hl]
      exact foldBy_congr_pt _ _ (apply_of_not hl)
    cases he : e.dom r with
    | true =>
      have hsk : ∀ s ∈ post, s.eff.dom r = false := fun s hs => disj_left (hpost s hs).2 he
      rw [foldBy_skip _ _ hsk, apply_of_dom he, apply_of_dom he]
    | false =>
      rw [apply_of_not he, hre]
      exact (foldBy_congr_pt _ _ (apply_of_not he)).symm

theorem inv_succ {b : Store} {l : Live} (hv : Inv b l) {c : Nat} {cs : ChainSt} (e : Eff)
    (hc : l.chains c = some cs) (hg : StepGuard l (.chain c e)) :
    Inv b (stepLive l (.chain c e)) := by
  have hI := hv.struct
  simp only [StepGuard, hc] at hg
  obtain ⟨hB, hF, hS⟩ := hg
  have hl' := stepLive_succ e hc
  -- names for the new state
  generalize hx : (⟨.chain c, e, false, false⟩ : Seg) = x at hl'
  generalize hcs' : ({ cs with touched := unionDom cs.touched e.dom, live := e } : ChainSt) = cs'
    at hl'
  have hxo : x.owner = .chain c := by rw [← hx]
  have hxe : x.eff = e := by rw [← hx]
  have hxm : x.masked = false := by rw [← hx]
  have hcsh : cs'.head = cs.head := by rw [← hcs']
  have hcsl : cs'.live = e := by rw [← hcs']
  have hcsb : cs'.base = cs.base := by rw [← hcs']
  have hcst : cs'.touched = unionDom cs.touched e.dom := by rw [← hcs']
  obtain ⟨i0, y0, hy0, hi0, hh0, _⟩ := hI.liveSeg c cs hc
  have hlen : cs.head < l.log.length := head_lt_length hI hc
  have hcomp : compactAll (stepLive l (.chain c e)).log = (compactAll l.log).map (replaceOwner (.chain c) x) := by
    rw [hl']; dsimp only
    exact compactAll_succ (by rw [hy0]; rfl) hxo (by rw [← hx]) hxm
  have hlog : (stepLive l (.chain c e)).log = maskOwner (.chain c) l.log ++ [x] := by rw [hl']
  have hchc : (stepLive l (.chain c e)).chains c = some cs' := by
    rw [hl']; exact updateChain_self _ _ _
  have hcho : ∀ X, X ≠ c → (stepLive l (.chain c e)).chains X = l.chains X := by
    intro X hX; rw [hl']; exact updateChain_other _ hX
  have hxidx : (stepLive l (.chain c e)).log[l.log.length]? = some x := by
    rw [hlog]
    have := @List.getElem?_concat_length _ (maskOwner (.chain c) l.log) x
    rwa [length_maskOwner] at this
  have hkeep : ∀ (i : Nat) (s : Seg), l.log[i]? = some s → s.owner ≠ .chain c →
      (stepLive l (.chain c e)).log[i]? = some s := by
    intro i s h1 h2; rw [hlog]; exact getElem?_append_lt (getElem?_maskOwner_of_ne h1 h2)
  -- `RR` facts survive the step
  have hrr : ∀ (h c' X : Nat) (csX : ChainSt), RR l h c' X csX → h < l.log.length →
      ∃ csX', RR (stepLive l (.chain c e)) h c' X csX' ∧
        ∀ r, csX'.touched r = false → csX.touched r = false := by
    intro h c' X csX ⟨h1, h2, h3, i, s, h4, h5, h6, h7⟩ hlt
    by_cases hX : X = c
    · subst hX
      rw [hc] at h1; cases h1
      refine ⟨cs', ⟨hchc, h2, by rw [hcsh]; exact h3, l.log.length, x, hxidx, hlt, hxo, hxm⟩, ?_⟩
      intro r hr
      rw [hcst] at hr
      simp only [unionDom, Bool.or_eq_false_iff] at hr
      exact hr.1
    · refine ⟨csX, ⟨by rw [hcho X hX]; exact h1, h2, h3, i, s,
        hkeep i s h4 (by rw [h6]; simp only [ne_eq, Owner.chain.injEq]; exact hX), h5, h6, h7⟩,
        fun _ h => h⟩
  obtain ⟨pre, y, post, v⟩ := hv.chains c cs hc
  have hpre : pre.map (replaceOwner (.chain c) x) = pre := map_replaceOwner_id v.preOwner
  have hpost : post.map (replaceOwner (.chain c) x) = post := map_replaceOwner_id v.postOwner
  have hy : replaceOwner (.chain c) x y = x := replaceOwner_of_eq v.yOwner
  have hpostD : ∀ s ∈ post, Disj s.eff.dom cs.touched ∧ Disj s.eff.dom e.dom := by
    intro s hs
    obtain ⟨i, h1, h2, h3⟩ := v.postPos s hs
    exact ⟨hB i s h1 h2 (v.postOwner s hs) h3, hF i s h1 h2 (v.postOwner s hs) h3⟩
  refine ⟨structInv_succ hI e hc, ?_, ?_⟩
  · -- the store equals the reference fold
    rw [hcomp, v.split, List.map_append, List.map_cons, hpre, hpost, hy]
    rw [hl']; dsimp only
    rw [hv.store, v.split, foldSegments_append, foldSegments_cons, foldSegments_append,
      foldSegments_cons, v.yEff, hxe]
    exact succ_store _ _ _ _ _ _ hpostD (hI.liveSub c cs hc) v.baseLive
  · intro c' cs'' hc'
    by_cases hcc : c' = c
    · subst hcc
      rw [hchc] at hc'; cases hc'
      refine ⟨pre, x, post, ?_⟩
      exact {
        split := by rw [hcomp, v.split, List.map_append, List.map_cons, hpre, hpost, hy]
        yOwner := hxo
        yEff := by rw [hxe, hcsl]
        preOwner := v.preOwner
        postOwner := v.postOwner
        preHead := by
          intro s hs X hX
          obtain ⟨csX, h1, h2⟩ := v.preHead s hs X hX
          have hXc : X ≠ c' := by intro h; subst h; exact v.preOwner s hs hX
          exact ⟨csX, by rw [hcho X hXc]; exact h1, by rw [hcsh]; exact h2⟩
        postPos := by
          intro s hs
          obtain ⟨i, h1, h2, h3⟩ := v.postPos s hs
          exact ⟨i, hkeep i s h1 (v.postOwner s hs), by rw [hcsh]; exact h2, h3⟩
        baseLive := by
          intro r hr
          rw [hcsl] at hr
          rw [hcsb]
          apply v.baseRest r
          intro X csX ⟨h1, h2, h3, i, s, h4, h5, h6, h7⟩
          exact disj_left (hS i s X csX h4 h5 h6 h2 h7 h1 h3) hr
        baseRest := by
          intro r hr
          rw [hcsb]
          apply v.baseRest r
          intro X csX hR
          obtain ⟨csX', hR', himp⟩ := hrr cs.head c' X csX hR hlen
          rw [← hcsh] at hR'
          exact himp r (hr X csX' hR') }
    · have hc1 : l.chains c' = some cs'' := by rw [← hcho c' hcc]; exact hc'
      obtain ⟨pre', y', post', v'⟩ := hv.chains c' cs'' hc1
      have hy' : replaceOwner (.chain c) x y' = y' :=
        replaceOwner_of_ne (by rw [v'.yOwner]; simp only [ne_eq, Owner.chain.injEq]; exact hcc)
      obtain ⟨hy'eff, i', hi', hh', hm'⟩ := slot_facts hI (by rw [v'.split]; simp) v'.yOwner hc1
      have hlen' : cs''.head < l.log.length := head_lt_length hI hc1
      -- an entry of `pre'` owned by `c` lies before `c'`'s head; its replacement is harmless at `r`
      have hown : ∀ s ∈ pre', s.owner = .chain c → cs.head < cs''.head := by
        intro s hs ho
        obtain ⟨csX, h1, h2⟩ := v'.preHead s hs c ho
        rw [hc] at h1; cases h1; exact h2
      have hslot : ∀ s ∈ pre', s.owner = .chain c → s.eff = cs.live := by
        intro s hs ho
        exact (slot_facts hI (by rw [v'.split]; simp [hs]) ho hc).1
      have hmapPt : ∀ r, cs.touched r = false → e.dom r = false →
          foldSegments (pre'.map (replaceOwner (.chain c) x)) b r = foldSegments pre' b r := by
        intro r ht he
        apply foldBy_map_pt
        intro s hs
        by_cases ho : s.owner = .chain c
        · right
          rw [replaceOwner_of_eq ho, hxe, hslot s hs ho]
          exact ⟨sub_false (hI.liveSub c cs hc) ht, he⟩
        · left; exact replaceOwner_of_ne ho
      refine ⟨pre'.map (replaceOwner (.chain c) x), y', post'.map (replaceOwner (.chain c) x), ?_⟩
      exact {
        split := by rw [hcomp, v'.split, List.map_append, List.map_cons, hy']
        yOwner := v'.yOwner
        yEff := v'.yEff
        preOwner := by
          intro s hs
          obtain ⟨t, ht, rfl⟩ := List.mem_map.mp hs
          rw [replaceOwner_owner hxo]; exact v'.preOwner t ht
        postOwner := by
          intro s hs
          obtain ⟨t, ht, rfl⟩ := List.mem_map.mp hs
          rw [replaceOwner_owner hxo]; exact v'.postOwner t ht
        preHead := by
          intro s hs X hX
          obtain ⟨t, ht, rfl⟩ := List.mem_map.mp hs
          rw [replaceOwner_owner hxo] at hX
          obtain ⟨csX, h1, h2⟩ := v'.preHead t ht X hX
          by_cases hXc : X = c
          · subst hXc; rw [hc] at h1; cases h1
            exact ⟨cs', hchc, by rw [hcsh]; exact h2⟩
          · exact ⟨csX, by rw [hcho X hXc]; exact h1, h2⟩
        postPos := by
          intro s hs
          obtain ⟨t, ht, rfl⟩ := List.mem_map.mp hs
          by_cases ho : t.owner = .chain c
          · rw [replaceOwner_of_eq ho]
            exact ⟨l.log.length, hxidx, hlen', hxm⟩
          · rw [replaceOwner_of_ne ho]
            obtain ⟨i, h1, h2, h3⟩ := v'.postPos t ht
            exact ⟨i, hkeep i t h1 ho, h2, h3⟩
        baseLive := by
          intro r hr
          rw [v'.baseLive r hr]
          have hcne : y'.owner ≠ .chain c := by
            rw [v'.yOwner]; simp only [ne_eq, Owner.chain.injEq]; exact hcc
          -- `c`'s guards see `y'` whenever `c`'s head precedes `c'`'s head
          by_cases hlt : cs.head < cs''.head
          · have hlt' : cs.head < i' := Nat.lt_of_lt_of_le hlt hh'
            have hr' : y'.eff.dom r = true := by rw [hy'eff]; exact hr
            have ht := disj_right (hB i' y' hi' hlt' hcne hm') hr'
            have he := disj_right (hF i' y' hi' hlt' hcne hm') hr'
            exact (hmapPt r ht he).symm
          · have : pre'.map (replaceOwner (.chain c) x) = pre' :=
              map_replaceOwner_id (fun s hs ho => hlt (hown s hs ho))
            rw [this]
        baseRest := by
          intro r hr
          have hold : cs''.base r = foldSegments pre' b r := by
            apply v'.baseRest r
            intro X csX hR
            obtain ⟨csX', hR', himp⟩ := hrr cs''.head c' X csX hR hlen'
            exact himp r (hr X csX' hR')
          rw [hold]
          by_cases hlt : cs.head < cs''.head
          · have hR : RR (stepLive l (.chain c e)) cs''.head c' c cs' :=
              ⟨hchc, fun h => hcc h.symm, by rw [hcsh]; exact hlt,
                l.log.length, x, hxidx, hlen', hxo, hxm⟩
            have ht := hr c cs' hR
            rw [hcst] at ht
            simp only [unionDom, Bool.or_eq_false_iff] at ht
            exact (hmapPt r ht.1 ht.2).symm
          · have : pre'.map (replaceOwner (.chain c) x) = pre' :=
              map_replaceOwner_id (fun s hs ho => hlt (hown s hs ho))
            rw [this] }

theorem inv_step {b : Store} {l : Live} (hv : Inv b l) (st : Step) (hg : StepGuard l st) :
    Inv b (stepLive l st) := by
  cases st with
  | foreign f => exact inv_foreign hv f
  | chain c e =>
    cases hc : l.chains c with
    | none => exact inv_head hv e hc
    | some cs => exact inv_succ hv e hc hg

theorem inv_runFrom {b : Store} : ∀ (steps : List Step) {l : Live}, Inv b l →
    guardsFrom StepGuard l steps → Inv b (runFrom l steps) := by
  intro steps
  induction steps with
  | nil => intro l h _; exact h
  | cons st ss ih =>
    intro l h hg
    exact ih (inv_step h st hg.1) hg.2

theorem inv_liveRun {b : Store} {steps : List Step} (hg : WellGuarded b steps) :
    Inv b (liveRun b steps) :=
  inv_runFrom steps (inv_init b) hg

end CoupledSuccession
