import PersistentNaming.TimelineSearch

/-!
# Held-name resolution step bound (Phase 7B-2, RULES §4 layer 1)

Models `PersistentNameBranchIndex::resolve` (`resolve_counted`,
`walk_forward`) of `crates/worth-relational-lineage/src/lineage/access/
name_resolution.rs` and `NameEntry::state_at` / `edges_at` /
`lineage_ended_at` of `crates/worth-relational-lineage/src/lineage/index/
entry.rs`.

A step is one loop iteration of a binary search (`TimelineSearch.search`),
one index probe (`self.names.get`, an O(1) expected hash lookup), one
popped frontier name (`event_visits`), or one traversed forward edge
(`edge_visits`). `state_at` is `partition_point` over a name's strictly
ascending timeline, bounded by `log₂ t + 1` and proved equal to the linear
scan in `TimelineSearch`. The walk is breadth first over a visited set from
the held name. Here `resolve_visits_lineage_path` proves the probes `V`
count distinct names, all on the forward lineage path `Reach`, and
`resolve_steps_independent_of_unrelated` that two indexes agreeing on that
path take the same steps, so total names and unrelated commits never enter;
`NameResolutionBound.resolve_steps_bounded` proves the steps are at most
`(V + E)·3·(log₂ Tmax + 2)` with `E` traversed edges and `Tmax` the longest
timeline or edge list among the names on the path.

Simplifications: the index is `Nat → Option Entry` with natural-number keys
and commits; `edges_at` (`binary_search_by_key`) and `lineage_ended_at`'s
`partition_point` (`<` rather than `<=`) are both modelled by the same
`search`; `lineage_ended_at`'s backwards `find_map` is charged one step,
which assumes the state before an `Ended` entry is live (ingestion ends only
live names; not proved here); the constant-time empty-filter and
pruned-basis refusals, the footprint and live-map insertions (O(log p)
each), and the final `select` over the found live set are not counted. The
walk runs on fuel; every theorem holds for every fuel, so a truncated walk
is covered too.
-/
namespace PersistentNaming.NameResolution

open PersistentNaming.TimelineSearch

/-- One name's entry: timeline commits and whether each state is `Ended`,
and the commits at which forward edges were recorded with their successor
lists. -/
structure Entry where
  commits : Nat → Nat
  len : Nat
  ended : Nat → Bool
  edgeCommits : Nat → Nat
  edgeLen : Nat
  edgeLists : Nat → List Nat

/-- What `state_at` found: born after the basis, live, or ended since. -/
inductive Found where
  | absent
  | live
  | ended (since : Nat)

/-- `state_at(basis)` and its search iterations. -/
def stateAt (e : Entry) (basis : Nat) : Found × Nat :=
  let r := search e.commits basis 0 e.len
  (if r.1 = 0 then .absent
    else if e.ended (r.1 - 1) then .ended (e.commits (r.1 - 1)) else .live, r.2)

/-- `edges_at(commit)` and its search iterations. -/
def edgesAt (e : Entry) (commit : Nat) : List Nat × Nat :=
  let r := search e.edgeCommits commit 0 e.edgeLen
  (if r.1 ≠ 0 ∧ e.edgeCommits (r.1 - 1) = commit then e.edgeLists (r.1 - 1) else [], r.2)

/-- The successors the walk follows from an entry: its edges at the commit
it ended, when it is ended at the basis. -/
def succs (e : Entry) (basis : Nat) : List Nat :=
  match (stateAt e basis).1 with
  | .ended since => (edgesAt e since).1
  | _ => []

/-- The forward lineage path from `held` at `basis`. -/
inductive Reach (idx : Nat → Option Entry) (basis held : Nat) : Nat → Prop where
  | root : Reach idx basis held held
  | edge {k s : Nat} {e : Entry} :
      Reach idx basis held k → idx k = some e → s ∈ succs e basis → Reach idx basis held s

/-- The walk: frontier, visited set, steps, `index_probes`, `edge_visits`. -/
structure Walk where
  queue : List Nat
  visited : List Nat
  steps : Nat
  probes : Nat
  edges : Nat

/-- Probing a name: its `state_at` cost and whether it is ended. -/
def probe (idx : Nat → Option Entry) (basis s : Nat) : Nat × Bool :=
  match idx s with
  | none => (0, false)
  | some e => ((stateAt e basis).2, match (stateAt e basis).1 with
      | .ended _ => true
      | _ => false)

/-- The inner `for successor in entry.edges_at(ended_at)` loop. -/
def visitEdges (idx : Nat → Option Entry) (basis : Nat) : List Nat → Walk → Walk
  | [], w => w
  | s :: ss, w =>
    if s ∈ w.visited then
      visitEdges idx basis ss { w with steps := w.steps + 1, edges := w.edges + 1 }
    else
      let p := probe idx basis s
      visitEdges idx basis ss
        { queue := if p.2 then w.queue ++ [s] else w.queue, visited := s :: w.visited,
          steps := w.steps + 2 + p.1, probes := w.probes + 1, edges := w.edges + 1 }

/-- One `frontier.pop_front()` iteration: the event visit, the footprint
lookup, `edges_at`, then the successors. -/
def pop (idx : Nat → Option Entry) (basis : Nat) (w : Walk) : Walk :=
  match w.queue with
  | [] => w
  | k :: rest =>
    match idx k with
    | none => { w with queue := rest }
    | some e =>
      match (stateAt e basis).1 with
      | .ended since =>
        visitEdges idx basis (edgesAt e since).1
          { w with queue := rest,
                   steps := w.steps + 2 + (search e.commits since 0 e.len).2
                     + (edgesAt e since).2 }
      | _ => { w with queue := rest }

def run (idx : Nat → Option Entry) (basis : Nat) : Nat → Walk → Walk
  | 0, w => w
  | n + 1, w => run idx basis n (pop idx basis w)

/-- `resolve`: probe the held name, then walk forward when it ended. -/
def resolve (idx : Nat → Option Entry) (basis held fuel : Nat) : Walk :=
  let p := probe idx basis held
  run idx basis fuel
    { queue := if p.2 then [held] else [], visited := [held], steps := 1 + p.1,
      probes := 1, edges := 0 }

section Proofs
variable (idx : Nat → Option Entry) (basis held : Nat)

/-- Every frontier and visited name is on the lineage path. -/
def OnPath (w : Walk) : Prop :=
  (∀ k, k ∈ w.queue → Reach idx basis held k) ∧ (∀ k, k ∈ w.visited → Reach idx basis held k)

theorem visitEdges_onPath : ∀ (ss : List Nat) (w : Walk),
    (∀ s, s ∈ ss → Reach idx basis held s) → OnPath idx basis held w →
    OnPath idx basis held (visitEdges idx basis ss w)
  | [], w, _, hw => hw
  | s :: ss, w, hs, ⟨hq, hv⟩ => by
    have hss : ∀ x, x ∈ ss → Reach idx basis held x := fun x hx => hs x (List.mem_cons_of_mem s hx)
    have h0 := hs s List.mem_cons_self
    simp only [visitEdges]
    split
    · exact visitEdges_onPath ss _ hss ⟨hq, hv⟩
    · refine visitEdges_onPath ss _ hss ⟨?_, ?_⟩
      · intro k hk
        split at hk
        · simp at hk; rcases hk with hk | hk
          · exact hq k hk
          · subst hk; exact h0
        · exact hq k hk
      · intro k hk
        simp at hk; rcases hk with hk | hk
        · subst hk; exact h0
        · exact hv k hk

theorem succs_of_ended {e : Entry} {since : Nat} (h : (stateAt e basis).1 = .ended since) :
    succs e basis = (edgesAt e since).1 := by
  simp [succs, h]

theorem pop_onPath (w : Walk) (hw : OnPath idx basis held w) :
    OnPath idx basis held (pop idx basis w) := by
  obtain ⟨hq, hv⟩ := hw
  unfold pop
  split
  · exact ⟨hq, hv⟩
  · rename_i k rest hqueue
    have hrest : ∀ x, x ∈ rest → Reach idx basis held x :=
      fun x hx => hq x (by rw [hqueue]; exact List.mem_cons_of_mem k hx)
    have hk : Reach idx basis held k := hq k (by rw [hqueue]; exact List.mem_cons_self)
    split
    · exact ⟨hrest, hv⟩
    · rename_i e he
      split
      · rename_i since hs
        refine visitEdges_onPath idx basis held _ _ ?_ ⟨hrest, hv⟩
        intro s hsm
        exact Reach.edge hk he (by rw [succs_of_ended basis hs]; exact hsm)
      · exact ⟨hrest, hv⟩

theorem run_onPath : ∀ (n : Nat) (w : Walk), OnPath idx basis held w →
    OnPath idx basis held (run idx basis n w)
  | 0, _, hw => hw
  | n + 1, w, hw => run_onPath n _ (pop_onPath idx basis held w hw)

theorem start_onPath :
    OnPath idx basis held
      { queue := if (probe idx basis held).2 then [held] else [], visited := [held],
        steps := 1 + (probe idx basis held).1, probes := 1, edges := 0 } := by
  constructor
  · intro k hk; split at hk <;> simp at hk; subst hk; exact Reach.root
  · intro k hk; simp at hk; subst hk; exact Reach.root

/-- Each probe inserts one new name into the visited set. -/
def Distinct (w : Walk) : Prop := w.probes = w.visited.length ∧ w.visited.Nodup

theorem visitEdges_distinct : ∀ (ss : List Nat) (w : Walk), Distinct w →
    Distinct (visitEdges idx basis ss w)
  | [], _, hw => hw
  | s :: ss, w, ⟨hp, hn⟩ => by
    simp only [visitEdges]
    split
    · exact visitEdges_distinct ss _ ⟨hp, hn⟩
    · rename_i hnot
      exact visitEdges_distinct ss _ ⟨by simp [hp], List.nodup_cons.2 ⟨hnot, hn⟩⟩

theorem pop_distinct (w : Walk) (hw : Distinct w) : Distinct (pop idx basis w) := by
  unfold pop
  split
  · exact hw
  · split
    · exact hw
    · split
      · exact visitEdges_distinct idx basis _ _ hw
      · exact hw

theorem run_distinct : ∀ (n : Nat) (w : Walk), Distinct w → Distinct (run idx basis n w)
  | 0, _, hw => hw
  | n + 1, w, hw => run_distinct n _ (pop_distinct idx basis w hw)

end Proofs

/-- The probes count distinct names, every one on the lineage path. -/
theorem resolve_visits_lineage_path (idx : Nat → Option Entry) (basis held fuel : Nat) :
    (∀ k, k ∈ (resolve idx basis held fuel).visited → Reach idx basis held k) ∧
      (resolve idx basis held fuel).probes = (resolve idx basis held fuel).visited.length ∧
      (resolve idx basis held fuel).visited.Nodup := by
  refine ⟨(run_onPath idx basis held fuel _ (start_onPath idx basis held)).2, ?_⟩
  exact run_distinct idx basis fuel _ ⟨rfl, List.nodup_cons.2 ⟨by simp, List.nodup_nil⟩⟩

section Unrelated
variable (idx idx' : Nat → Option Entry) (basis held : Nat)
  (hagree : ∀ k, Reach idx basis held k → idx k = idx' k)
include hagree

theorem probe_congr (s : Nat) (hs : Reach idx basis held s) :
    probe idx basis s = probe idx' basis s := by
  unfold probe; rw [hagree s hs]

theorem visitEdges_congr : ∀ (ss : List Nat) (w : Walk), (∀ s, s ∈ ss → Reach idx basis held s) →
    visitEdges idx basis ss w = visitEdges idx' basis ss w
  | [], _, _ => rfl
  | s :: ss, w, hs => by
    have hss : ∀ x, x ∈ ss → Reach idx basis held x := fun x hx => hs x (List.mem_cons_of_mem s hx)
    simp only [visitEdges, probe_congr idx idx' basis held hagree s (hs s List.mem_cons_self)]
    split
    · exact visitEdges_congr ss _ hss
    · exact visitEdges_congr ss _ hss

theorem pop_congr (w : Walk) (hw : OnPath idx basis held w) : pop idx basis w = pop idx' basis w := by
  unfold pop
  split
  · rfl
  · rename_i k rest hqueue
    have hk : Reach idx basis held k := hw.1 k (by rw [hqueue]; exact List.mem_cons_self)
    rw [← hagree k hk]
    split
    · rfl
    · rename_i e he
      split
      · rename_i since hs
        exact visitEdges_congr idx idx' basis held hagree _ _
          (fun s hsm => Reach.edge hk he (by rw [succs_of_ended basis hs]; exact hsm))
      · rfl

theorem run_congr : ∀ (n : Nat) (w : Walk), OnPath idx basis held w →
    run idx basis n w = run idx' basis n w
  | 0, _, _ => rfl
  | n + 1, w, hw => by
    simp only [run]
    rw [← pop_congr idx idx' basis held hagree w hw]
    exact run_congr n _ (pop_onPath idx basis held w hw)

/-- Two indexes that agree on the lineage path give the same walk. -/
theorem resolve_congr (fuel : Nat) : resolve idx basis held fuel = resolve idx' basis held fuel := by
  simp only [resolve]
  rw [← probe_congr idx idx' basis held hagree held Reach.root]
  exact run_congr idx idx' basis held hagree fuel _ (start_onPath idx basis held)

end Unrelated

/-- Indexes that agree on every name of the held name's lineage path take
the same steps, however many other names or commits either holds. -/
theorem resolve_steps_independent_of_unrelated (idx idx' : Nat → Option Entry)
    (basis held fuel : Nat) (hagree : ∀ k, Reach idx basis held k → idx k = idx' k) :
    (resolve idx basis held fuel).steps = (resolve idx' basis held fuel).steps := by
  rw [resolve_congr idx idx' basis held hagree fuel]

end PersistentNaming.NameResolution
