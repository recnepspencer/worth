import CoupledSuccession.MultiChain
import CoupledSuccession.Guards

/-!
# The design's two guards do not suffice for T4m

Chain `X = 0` heads first and writes record `0`; chain `Y = 1` heads next (its head pre-pin
`B_Y` holds `X`'s write to `0`). `X` round 2 drops record `0` (retracted to `B_X(0) = none`).
`Y` round 2 writes record `0`: the backward and forward guards pass, since no unmasked segment
after `Y`'s head touches `0`. `Y` round 3 drops record `0`, and retraction restores
`B_Y(0) = some 1`: `X`'s superseded round-1 image, which no live segment writes. `R(E)` gives
`none`. The stale-base guard (`StaleBaseOk`) refuses `Y` round 2.
-/
namespace CoupledSuccession.MultiChainGap

open CoupledSuccession Guards

def e1 : Eff := ⟨single 0, fun _ => some 1⟩
def g1 : Eff := ⟨single 5, fun _ => some 5⟩
def e2 : Eff := ⟨single 6, fun _ => some 6⟩
def g2 : Eff := ⟨single 0, fun _ => some 7⟩
def g3 : Eff := ⟨fun _ => false, fun _ => none⟩

def steps : List Step := [.chain 0 e1, .chain 1 g1, .chain 0 e2, .chain 1 g2, .chain 1 g3]

theorem live_store_zero : (liveRun noneStore steps).store 0 = some 1 := by
  simp [liveRun, runFrom, steps, stepLive, Live.init, updateChain, apply, retract, e1, g1, e2,
    g2, g3, single]

theorem ref_store_zero :
    foldSegments (liveSegments (liveRun noneStore steps)) noneStore 0 = none := by
  simp [liveSegments, liveRun, runFrom, steps, stepLive, Live.init, updateChain, compactAll,
    liveOf, maskOwner, foldSegments, foldBy, apply, e1, g1, e2, g2, g3, single, noneStore]

theorem guards2 : WellGuarded2 noneStore steps := by
  simp [WellGuarded2, guardsFrom, steps, StepGuard2, stepLive, Live.init, updateChain,
    BackwardOk, ForwardOk, maskOwner, Disj, single, e1, g1, e2, g2, g3]
  repeat' apply And.intro
  all_goals
    intro i s hi hlt hne hm r hr
    rcases i with _ | _ | _ | _ | i <;> simp at hi <;> subst hi <;>
      simp_all [single, unionDom] <;> omega

/-- The stale-base guard refuses `Y` round 2 of this run. -/
theorem stale_guard_refuses : ¬ WellGuarded noneStore steps := by
  intro h
  simp only [WellGuarded, steps, guardsFrom] at h
  obtain ⟨_, _, _, h4, _⟩ := h
  simp [StepGuard, stepLive, Live.init, updateChain, maskOwner, e1, g1, e2, g2] at h4
  have hd := h4.2.2 2 _ 0 _ rfl (by decide) rfl (by decide) rfl rfl (by decide)
  have := hd 0 (by simp [unionDom, single])
  simp [single] at this

/-- The design's §5.1 `WellGuarded` (backward + forward only) does not imply T4m. -/
theorem two_guards_insufficient :
    ∃ (b : Store) (steps : List Step), WellGuarded2 b steps ∧
      (liveRun b steps).store ≠ foldSegments (liveSegments (liveRun b steps)) b := by
  refine ⟨noneStore, steps, guards2, fun h => ?_⟩
  have := congrFun h 0
  rw [live_store_zero, ref_store_zero] at this
  cases this

end CoupledSuccession.MultiChainGap
