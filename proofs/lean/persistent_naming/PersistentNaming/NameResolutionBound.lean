import PersistentNaming.NameResolution

/-!
# Held-name resolution: the step bound (Phase 7B-2, RULES §4 layer 1)

The potential argument for the model of `PersistentNameBranchIndex::resolve`
in `NameResolution`: every queued (ended) name prepays its later pop, so a
resolution at any fuel takes at most `(V + E)·3·(log₂ Tmax + 2)` steps, with
`V` index probes, `E` traversed edges and `Tmax` the longest timeline or edge
list among the names on the lineage path only. Steps are as in
`NameResolution`.

Proofs only: this file models no production Rust of its own, so it carries
no `proof-mirror:` pin; `NameResolution` pins the Rust its definitions model.
-/
namespace PersistentNaming.NameResolution

open PersistentNaming.TimelineSearch

section Proofs
variable (idx : Nat → Option Entry) (basis held : Nat)

/-- The potential argument: queued names prepay their pop. -/
def Counted (L : Nat) (w : Walk) : Prop :=
  w.steps + w.queue.length * (2 * L + 4) ≤ w.probes * (3 * L + 6) + w.edges

variable (Tmax : Nat)
  (hT : ∀ k e, Reach idx basis held k → idx k = some e → e.len ≤ Tmax ∧ e.edgeLen ≤ Tmax)
include hT

theorem probe_cost (s : Nat) (hs : Reach idx basis held s) :
    (probe idx basis s).1 ≤ Nat.log2 Tmax + 1 := by
  unfold probe
  split
  · omega
  · rename_i e he
    have := search_steps_le_log e.commits basis 0 e.len
    have := log2_mono (hT s e hs he).1
    simp only [stateAt]
    omega

theorem visitEdges_counted : ∀ (ss : List Nat) (w : Walk),
    (∀ s, s ∈ ss → Reach idx basis held s) → Counted (Nat.log2 Tmax) w →
    Counted (Nat.log2 Tmax) (visitEdges idx basis ss w)
  | [], w, _, hw => hw
  | s :: ss, w, hs, hc => by
    have hss : ∀ x, x ∈ ss → Reach idx basis held x := fun x hx => hs x (List.mem_cons_of_mem s hx)
    have hpc := probe_cost idx basis held Tmax hT s (hs s List.mem_cons_self)
    unfold Counted at hc
    simp only [visitEdges]
    split
    · exact visitEdges_counted ss _ hss (by unfold Counted; simp only; omega)
    · refine visitEdges_counted ss _ hss ?_
      unfold Counted
      simp only
      split
      · simp only [List.length_append, List.length_cons, List.length_nil, Nat.add_mul,
          Nat.one_mul, Nat.zero_add]
        omega
      · simp only [Nat.add_mul, Nat.one_mul]
        omega

theorem pop_counted (w : Walk) (ho : OnPath idx basis held w) (hw : Counted (Nat.log2 Tmax) w) :
    Counted (Nat.log2 Tmax) (pop idx basis w) := by
  have hc := hw
  unfold Counted at hc
  unfold pop
  split
  · exact hw
  · rename_i k rest hqueue
    rw [hqueue] at hc
    simp only [List.length_cons, Nat.add_mul, Nat.one_mul] at hc
    have hk : Reach idx basis held k := ho.1 k (by rw [hqueue]; exact List.mem_cons_self)
    split
    · unfold Counted; simp only; omega
    · rename_i e he
      split
      · rename_i since hs
        have hl := hT k e hk he
        refine visitEdges_counted idx basis held Tmax hT _ _ ?_ ?_
        · intro s hsm
          exact Reach.edge hk he (by rw [succs_of_ended basis hs]; exact hsm)
        · have h1 := search_steps_le_log e.commits since 0 e.len
          have h2 := search_steps_le_log e.edgeCommits since 0 e.edgeLen
          have h3 := log2_mono hl.1
          have h4 := log2_mono hl.2
          unfold Counted
          simp only [edgesAt]
          omega
      · unfold Counted; simp only; omega

theorem run_counted : ∀ (n : Nat) (w : Walk), OnPath idx basis held w →
    Counted (Nat.log2 Tmax) w → Counted (Nat.log2 Tmax) (run idx basis n w)
  | 0, _, _, hw => hw
  | n + 1, w, ho, hw =>
    run_counted n _ (pop_onPath idx basis held w ho) (pop_counted idx basis held Tmax hT w ho hw)

end Proofs

/-- Every resolution, at any fuel, takes at most `(V + E)·3·(log₂ Tmax + 2)`
steps: `V` index probes, `E` traversed edges, `Tmax` the longest timeline or
edge list among the names on the lineage path. -/
theorem resolve_steps_bounded (idx : Nat → Option Entry) (basis held fuel Tmax : Nat)
    (hT : ∀ k e, Reach idx basis held k → idx k = some e → e.len ≤ Tmax ∧ e.edgeLen ≤ Tmax) :
    (resolve idx basis held fuel).steps ≤
      ((resolve idx basis held fuel).probes + (resolve idx basis held fuel).edges)
        * (3 * (Nat.log2 Tmax + 2)) := by
  have hstart : Counted (Nat.log2 Tmax)
      { queue := if (probe idx basis held).2 then [held] else [], visited := [held],
        steps := 1 + (probe idx basis held).1, probes := 1, edges := 0 } := by
    have := probe_cost idx basis held Tmax hT held Reach.root
    unfold Counted
    simp only
    split <;> simp only [List.length_cons, List.length_nil] <;> omega
  have hc := run_counted idx basis held Tmax hT fuel _ (start_onPath idx basis held) hstart
  unfold Counted at hc
  simp only [resolve]
  generalize run idx basis fuel _ = w at hc ⊢
  have : w.edges ≤ w.edges * (3 * (Nat.log2 Tmax + 2)) :=
    Nat.le_mul_of_pos_right _ (by omega)
  rw [Nat.add_mul, show 3 * (Nat.log2 Tmax + 2) = 3 * Nat.log2 Tmax + 6 by omega]
  rw [show 3 * (Nat.log2 Tmax + 2) = 3 * Nat.log2 Tmax + 6 by omega] at this
  omega

end PersistentNaming.NameResolution
