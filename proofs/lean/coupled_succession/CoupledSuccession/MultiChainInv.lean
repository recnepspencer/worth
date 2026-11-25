import CoupledSuccession.Structure

/-!
# The multi-chain invariant

For every chain `c`, the compacted log splits as `pre ++ y :: post` where `y` is `c`'s live
segment at its head position. The head pre-pin `B_c` equals the reference image before the head
(`foldSegments pre b`) on `c`'s live records, and on every record not touched by an
earlier-headed chain that re-rounded since `c`'s head (`RR`).
-/
namespace CoupledSuccession

theorem foldSegments_append (l l' : List Seg) (b : Store) :
    foldSegments (l ++ l') b = foldSegments l' (foldSegments l b) := foldBy_append _ _ _ _

theorem foldSegments_cons (a : Seg) (l : List Seg) (b : Store) :
    foldSegments (a :: l) b = foldSegments l (apply a.eff b) := rfl

theorem foldSegments_snoc (l : List Seg) (x : Seg) (b : Store) :
    foldSegments (l ++ [x]) b = apply x.eff (foldSegments l b) := by
  rw [foldSegments_append]; rfl

/-- `X` is an earlier-headed chain (head before `h`) that has re-rounded since `h`. -/
def RR (l : Live) (h : Nat) (c X : Nat) (csX : ChainSt) : Prop :=
  l.chains X = some csX ∧ X ≠ c ∧ csX.head < h ∧
    ∃ i s, l.log[i]? = some s ∧ h < i ∧ s.owner = .chain X ∧ s.masked = false

/-- The view of chain `c` in the compacted log. -/
structure ChainView (b : Store) (l : Live) (c : Nat) (cs : ChainSt)
    (pre : List Seg) (y : Seg) (post : List Seg) : Prop where
  split : compactAll l.log = pre ++ y :: post
  yOwner : y.owner = .chain c
  yEff : y.eff = cs.live
  preOwner : ∀ s ∈ pre, s.owner ≠ .chain c
  postOwner : ∀ s ∈ post, s.owner ≠ .chain c
  preHead : ∀ s ∈ pre, ∀ X, s.owner = .chain X → ∃ csX, l.chains X = some csX ∧ csX.head < cs.head
  postPos : ∀ s ∈ post, ∃ i, l.log[i]? = some s ∧ cs.head < i ∧ s.masked = false
  baseLive : ∀ r, cs.live.dom r = true → cs.base r = foldSegments pre b r
  baseRest : ∀ r, (∀ X csX, RR l cs.head c X csX → csX.touched r = false) →
    cs.base r = foldSegments pre b r

structure Inv (b : Store) (l : Live) : Prop where
  struct : StructInv l
  store : l.store = foldSegments (compactAll l.log) b
  chains : ∀ c cs, l.chains c = some cs → ∃ pre y post, ChainView b l c cs pre y post

theorem inv_init (b : Store) : Inv b (Live.init b) where
  struct := structInv_init b
  store := rfl
  chains := by intro c cs h; simp [Live.init] at h

/-- A chain's entry of the compacted log is its live segment, unmasked, at or after its head. -/
theorem slot_facts {l : Live} (hI : StructInv l) {s : Seg} (hs : s ∈ compactAll l.log) {X : Nat}
    (ho : s.owner = .chain X) {csX : ChainSt} (hX : l.chains X = some csX) :
    s.eff = csX.live ∧ ∃ i, l.log[i]? = some s ∧ csX.head ≤ i ∧ s.masked = false := by
  have hm := mem_compactAll hs
  rw [ho] at hm
  obtain ⟨i, y, hy, hiy, hh, heff⟩ := hI.liveSeg X csX hX
  rw [hm] at hy
  cases hy
  exact ⟨heff, i, hiy, hh, (liveOf_some hm).2.2⟩

theorem head_lt_length {l : Live} (hI : StructInv l) {c : Nat} {cs : ChainSt}
    (hc : l.chains c = some cs) : cs.head < l.log.length := by
  obtain ⟨i, _, _, hi, hh, _⟩ := hI.liveSeg c cs hc
  exact Nat.lt_of_le_of_lt hh (lt_length_of_getElem? hi)

/-- Appending a member keeps `RR` facts (old entries keep their index and mask bit). -/
theorem rr_mono_append {l l' : Live} {x : Seg} (hlog : l'.log = l.log ++ [x])
    (hch : ∀ X csX, l.chains X = some csX → l'.chains X = some csX)
    {h c X : Nat} {csX : ChainSt} (hr : RR l h c X csX) : RR l' h c X csX := by
  obtain ⟨h1, h2, h3, i, s, h4, h5, h6, h7⟩ := hr
  exact ⟨hch X csX h1, h2, h3, i, s, by rw [hlog]; exact getElem?_append_lt h4, h5, h6, h7⟩

/-- Shared part of the append cases: an existing chain's view survives a fresh append. -/
theorem view_append {b : Store} {l l' : Live} {x : Seg} (hI : StructInv l)
    (hlog : l'.log = l.log ++ [x]) (hcomp : compactAll l'.log = compactAll l.log ++ [x])
    (hch : ∀ X csX, l.chains X = some csX → l'.chains X = some csX)
    (hfresh : ∀ c, (l.chains c).isSome → x.owner ≠ .chain c) (hxm : x.masked = false)
    {c : Nat} {cs : ChainSt} (hc : l.chains c = some cs) {pre : List Seg} {y : Seg}
    {post : List Seg} (v : ChainView b l c cs pre y post) :
    ChainView b l' c cs pre y (post ++ [x]) where
  split := by rw [hcomp, v.split]; simp
  yOwner := v.yOwner
  yEff := v.yEff
  preOwner := v.preOwner
  postOwner := by
    intro s hs
    rcases List.mem_append.mp hs with hs | hs
    · exact v.postOwner s hs
    · rw [List.mem_singleton] at hs; subst hs; exact hfresh c (by rw [hc]; rfl)
  preHead := by
    intro s hs X hX
    obtain ⟨csX, h1, h2⟩ := v.preHead s hs X hX
    exact ⟨csX, hch X csX h1, h2⟩
  postPos := by
    intro s hs
    rcases List.mem_append.mp hs with hs | hs
    · obtain ⟨i, h1, h2, h3⟩ := v.postPos s hs
      exact ⟨i, by rw [hlog]; exact getElem?_append_lt h1, h2, h3⟩
    · rw [List.mem_singleton] at hs; subst hs
      exact ⟨l.log.length, by rw [hlog]; exact List.getElem?_concat_length,
        head_lt_length hI hc, hxm⟩
  baseLive := v.baseLive
  baseRest := by
    intro r hr
    exact v.baseRest r (fun X csX h => hr X csX (rr_mono_append hlog hch h))

theorem inv_foreign {b : Store} {l : Live} (hv : Inv b l) (f : Eff) :
    Inv b (stepLive l (.foreign f)) := by
  have hI := hv.struct
  have hfr : ∀ s ∈ l.log, s.owner ≠ (⟨.foreign l.log.length, f, true, false⟩ : Seg).owner := by
    intro s hs ho
    have := hI.foreignFresh s hs l.log.length ho
    omega
  have hcomp := compactAll_append_fresh hfr rfl rfl
  refine ⟨structInv_foreign hI f, ?_, ?_⟩
  · rw [stepLive_foreign]; dsimp only
    rw [hcomp, foldSegments_snoc, ← hv.store]
  · intro c cs hc
    rw [stepLive_foreign] at hc; dsimp only at hc
    obtain ⟨pre, y, post, v⟩ := hv.chains c cs hc
    refine ⟨pre, y, _, view_append hI (l' := stepLive l (.foreign f))
      (x := ⟨.foreign l.log.length, f, true, false⟩) ?_ ?_ ?_ ?_ rfl hc v⟩
    · rfl
    · exact hcomp
    · intro X csX h; exact h
    · intro c' _ h; cases h

theorem inv_head {b : Store} {l : Live} (hv : Inv b l) {c0 : Nat} (e : Eff)
    (hc0 : l.chains c0 = none) : Inv b (stepLive l (.chain c0 e)) := by
  have hI := hv.struct
  have hfr := chain_fresh hI hc0
  have hcomp := compactAll_append_fresh (x := ⟨.chain c0, e, true, false⟩) hfr rfl rfl
  have hl' := stepLive_head e hc0
  have hch : ∀ X csX, l.chains X = some csX → (stepLive l (.chain c0 e)).chains X = some csX := by
    intro X csX h
    rw [hl']; dsimp only
    have hne : X ≠ c0 := by intro hx; subst hx; rw [hc0] at h; cases h
    rw [updateChain_other _ hne]; exact h
  refine ⟨structInv_head hI e hc0, ?_, ?_⟩
  · rw [hl']; dsimp only
    rw [hcomp, foldSegments_snoc, ← hv.store]
  · intro c cs hc
    by_cases hcc : c = c0
    · subst hcc
      rw [hl'] at hc; dsimp only at hc
      rw [updateChain_self] at hc
      cases hc
      refine ⟨compactAll l.log, ⟨.chain c, e, true, false⟩, [], ?_⟩
      exact {
        split := by rw [hl']; dsimp only; rw [hcomp]
        yOwner := rfl
        yEff := rfl
        preOwner := fun s hs => hfr s (mem_log_of_mem_compactAll hs)
        postOwner := by intro s hs; cases hs
        preHead := by
          intro s hs X hX
          have hsl := mem_log_of_mem_compactAll hs
          obtain ⟨csX, hcsX⟩ := Option.isSome_iff_exists.mp (hI.chainKnown s hsl X hX)
          exact ⟨csX, hch X csX hcsX, head_lt_length hI hcsX⟩
        postPos := by intro s hs; cases hs
        baseLive := by intro r _; dsimp only; rw [hv.store]
        baseRest := by intro r _; dsimp only; rw [hv.store] }
    · have hc1 : l.chains c = some cs := by
        rw [hl'] at hc; dsimp only at hc; rwa [updateChain_other _ hcc] at hc
      obtain ⟨pre, y, post, v⟩ := hv.chains c cs hc1
      refine ⟨pre, y, _, view_append hI (l' := stepLive l (.chain c0 e))
        (x := ⟨.chain c0, e, true, false⟩) ?_ ?_ hch ?_ rfl hc1 v⟩
      · rw [hl']
      · rw [hl']; exact hcomp
      · intro c' hs ho
        simp only [Owner.chain.injEq] at ho
        subst ho; rw [hc0] at hs; cases hs

end CoupledSuccession
