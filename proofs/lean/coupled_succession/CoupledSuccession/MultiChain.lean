import CoupledSuccession.MultiChainSucc

/-!
# Several chains interleaved (manifest T4m)

Any number of chains and foreign members, interleaved in any order, including the
masked-segment case (`c2` round 2 reverts `E` before `c1` round 2). Under the runtime guards the
live store equals the fold of the live segments, each at its member's head position (`R(E)`).

Deviation from design §5.1: `WellGuarded` carries a third, stale-base guard (`StaleBaseOk`).
With only the backward and forward guards the statement is false; the machine-checked
counterexample is `CoupledSuccession.MultiChainGap.two_guards_insufficient`.
-/
namespace CoupledSuccession.MultiChain

open CoupledSuccession

/-- T4m: the live store is the composition of the live segments at their heads.
The statement is the design's; `WellGuarded` includes the added stale-base guard (see above). -/
theorem live_eq_compose_live_segments :
    ∀ (b : Store) (steps : List Step), WellGuarded b steps →
      (liveRun b steps).store = foldSegments (liveSegments (liveRun b steps)) b := by
  intro b steps hg
  exact (inv_liveRun hg).store

end CoupledSuccession.MultiChain
