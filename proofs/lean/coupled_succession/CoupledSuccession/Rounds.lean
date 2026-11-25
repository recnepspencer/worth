import CoupledSuccession.Elision

/-!
# One chain, k ≥ 1 rounds, foreign members anywhere after the head (manifest T4)

`b` is the head pre-pin `B`: foreign members appended before the head are already folded into
it, so the chain's head is the first member of `steps`. Foreign members may be interleaved
anywhere between rounds.
-/
namespace CoupledSuccession

/-- Exactly one chain `c`, whose head is the first appended member. -/
def SingleChain (c : Nat) (steps : List Step) : Prop :=
  (∃ e rest, steps = .chain c e :: rest) ∧ ∀ c' e, Step.chain c' e ∈ steps → c' = c

/-- Apply the foreign members, in append order. -/
def applyForeign : List Step → Store → Store
  | [], s => s
  | .foreign f :: ss, s => applyForeign ss (apply f s)
  | .chain _ _ :: ss, s => applyForeign ss s

/-- The last round of chain `c` (or `d` if there is none). -/
def lastRound (c : Nat) (d : Option Eff) : List Step → Option Eff
  | [] => d
  | .foreign _ :: ss => lastRound c d ss
  | .chain c' e :: ss => lastRound c (if c' = c then some e else d) ss

def applyOpt : Option Eff → Store → Store
  | some e, s => apply e s
  | none, s => s

/-- Apply only chain `c`'s final round (the single-shot round). -/
def applyLast (c : Nat) (steps : List Step) (b : Store) : Store :=
  applyOpt (lastRound c none steps) b

theorem refFrom_single (b : Store) (c : Nat) : ∀ (rest : List Step) (y : Seg) (fs : List Seg)
    (n : Nat) (seen : Nat → Bool), seen c = true → y.owner = .chain c →
    (∀ s ∈ fs, s.owner ≠ .chain c) → (∀ c' e, Step.chain c' e ∈ rest → c' = c) →
    foldSegments (refFrom (y :: fs) n seen rest) b =
      applyForeign rest (foldSegments fs (applyOpt (lastRound c (some y.eff) rest) b)) := by
  intro rest
  induction rest with
  | nil => intro y fs n seen _ _ _ _; rfl
  | cons st ss ih =>
    intro y fs n seen hseen hy hfs hrest
    have hss : ∀ c' e, Step.chain c' e ∈ ss → c' = c :=
      fun c' e h => hrest c' e (List.mem_cons_of_mem _ h)
    cases st with
    | foreign f =>
      simp only [refFrom, List.cons_append, applyForeign, lastRound]
      rw [ih y _ _ _ hseen hy _ hss, foldSegments_snoc]
      intro s hs
      rcases List.mem_append.mp hs with hs | hs
      · exact hfs s hs
      · rw [List.mem_singleton] at hs; subst hs; simp
    | chain c' e =>
      have hc' : c' = c := hrest c' e List.mem_cons_self
      subst hc'
      simp only [refFrom, hseen, ite_true, List.map_cons, applyForeign, lastRound]
      rw [replaceOwner_of_eq hy, map_replaceOwner_id hfs]
      exact ih _ _ _ _ hseen rfl hfs hss

namespace Rounds

/-- T4: one chain with foreign members anywhere after its head equals the single-shot answer. -/
theorem live_eq_single_shot :
    ∀ (b : Store) (c : Nat) (steps : List Step), SingleChain c steps → WellGuarded b steps →
      (liveRun b steps).store = applyForeign steps (applyLast c steps b) := by
  intro b c steps ⟨⟨e0, rest, hsteps⟩, hall⟩ hg
  rw [MultiChain.live_eq_compose_live_segments b steps hg, liveSegments,
    compactAll_liveRun_eq_refLog, hsteps]
  have hrest : ∀ c' e, Step.chain c' e ∈ rest → c' = c :=
    fun c' e h => hall c' e (by rw [hsteps]; exact List.mem_cons_of_mem _ h)
  simp only [refLog, refFrom, Bool.false_eq_true, ite_false, List.nil_append, applyLast,
    lastRound, ite_true, applyForeign]
  exact refFrom_single b c rest _ [] 1 _ (by simp) rfl (by simp) hrest

end Rounds
end CoupledSuccession
