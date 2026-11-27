/-!
# Tracked-read currentness step bound (Phase 7B-2, RULES §4 layer 1)

Models `NameReadDependencies::currentness_after_lineage_events`
(`workspaces/worth-query/crates/worth-query-execution-naming/src/
tracked_resolution/dependencies.rs`) and `currentness_after_group_lineage`
(`dependencies/groups.rs`) in the probe shape being implemented: every
dependency is kept in a balanced ordered set or map (`BTreeSet`/`BTreeMap`),
and the commit's events are iterated, each probing the dependency sets once
per lineage, birth, group or member it touches.

A step is one event iteration, or one dependency probe, which costs
`log₂ D + 1` for `D` the total dependency count (the cost model of a B-tree
lookup, not a model of the tree). What each event probes (`touched`):
* `Create`: its birth digest in the awaited births;
* `Retire`/`Replace`/`Split`/`Merge`/`Restore`: each source that is not also
  a target, in the live records and in the ended lineages (`rests_on`);
* a group transition under `scheme`: each retired or assigned group in the
  group dependencies, in the awaited births, and in a reverse
  `(scheme, answered group)` index of member answers; and each member named
  in its assignments by one probe into the `(scheme, member)` map, never a
  scan over all member dependencies.

`currentness_steps_eq` proves the steps are exactly
`Σ_events |touched e| · (log₂ D + 1) + |events|`, and
`currentness_independent_of_unrelated_deps` that adding dependencies on keys
no event touches leaves the stale answer unchanged and moves the steps only
through the log factor, by `Σ |touched e| · (log₂ D' − log₂ D)`.

Simplifications: one branch (the per-event `branches.get` is folded into the
event step); dependency keys and digests are natural numbers; the
event-local work (filtering sources against the event's own targets,
building the touched-group set) and inserting a hit into the stale set are
not counted; the undecodable-frame path, which stales every group read by
design, is not modelled; the reverse answer index is part of the design
being implemented, not of the current Rust.
-/
namespace PersistentNaming.TrackedDeps

/-- One recorded dependency, by the set or map that holds it. -/
inductive Key where
  | live (lineage : Nat)
  | ended (lineage : Nat)
  | birth (digest : Nat)
  | group (group : Nat)
  | member (scheme member : Nat)
  | answer (scheme group : Nat)
  deriving DecidableEq

/-- One canonical lineage event or group transition of a commit. -/
inductive Event where
  | create (birth : Option Nat)
  | ending (sources targets : List Nat)
  | groupTransition (scheme : Nat) (retired : List Nat) (assignments : List (Nat × List Nat))

/-- The dependency probes an event makes. -/
def touched : Event → List Key
  | .create none => []
  | .create (some digest) => [.birth digest]
  | .ending sources targets =>
    (sources.filter fun s => !decide (s ∈ targets)).flatMap fun l => [.live l, .ended l]
  | .groupTransition scheme retired assignments =>
    ((retired ++ assignments.map Prod.fst).flatMap fun g =>
        [.group g, .birth g, .answer scheme g])
      ++ assignments.flatMap fun a => a.2.map (.member scheme)

/-- Probing keys against `deps`: the hits and the steps. -/
def probeAll (deps : List Key) : List Key → List Key × Nat
  | [] => ([], 0)
  | k :: ks =>
    let r := probeAll deps ks
    (if k ∈ deps then k :: r.1 else r.1, r.2 + (Nat.log2 deps.length + 1))

/-- The commit's verdict (the stale dependencies; none means `Clean`) and
its steps. -/
def currentness (deps : List Key) : List Event → List Key × Nat
  | [] => ([], 0)
  | e :: es =>
    let p := probeAll deps (touched e)
    let r := currentness deps es
    (p.1 ++ r.1, r.2 + p.2 + 1)

def sumTouched : List Event → Nat
  | [] => 0
  | e :: es => (touched e).length + sumTouched es

theorem probeAll_steps (deps : List Key) : ∀ ks : List Key,
    (probeAll deps ks).2 = ks.length * (Nat.log2 deps.length + 1)
  | [] => by simp [probeAll]
  | k :: ks => by
    simp only [probeAll, probeAll_steps deps ks, List.length_cons, Nat.succ_mul]

/-- The steps are `Σ_events |touched e| · (log₂ D + 1) + |events|`. -/
theorem currentness_steps_eq (deps : List Key) : ∀ events : List Event,
    (currentness deps events).2 =
      sumTouched events * (Nat.log2 deps.length + 1) + events.length
  | [] => by simp [currentness, sumTouched]
  | e :: es => by
    simp only [currentness, currentness_steps_eq deps es, probeAll_steps, sumTouched,
      List.length_cons, Nat.add_mul]
    omega

theorem probeAll_hits_congr (deps extra : List Key) : ∀ ks : List Key,
    (∀ k, k ∈ ks → k ∉ extra) → (probeAll (deps ++ extra) ks).1 = (probeAll deps ks).1
  | [] => fun _ => rfl
  | k :: ks => fun h => by
    have hk : (k ∈ deps ++ extra) = (k ∈ deps) := by
      simp [h k List.mem_cons_self]
    simp only [probeAll, hk, probeAll_hits_congr deps extra ks
      (fun x hx => h x (List.mem_cons_of_mem k hx))]

theorem currentness_hits_congr (deps extra : List Key) : ∀ events : List Event,
    (∀ k, k ∈ extra → ∀ e, e ∈ events → k ∉ touched e) →
      (currentness (deps ++ extra) events).1 = (currentness deps events).1
  | [] => fun _ => rfl
  | e :: es => fun h => by
    simp only [currentness]
    rw [probeAll_hits_congr deps extra (touched e)
        (fun k hk hx => h k hx e List.mem_cons_self hk),
      currentness_hits_congr deps extra es
        (fun k hx e' he' => h k hx e' (List.mem_cons_of_mem e he'))]

theorem log2_mono {m n : Nat} (h : m ≤ n) : Nat.log2 m ≤ Nat.log2 n := by
  by_cases hm : m = 0
  · subst hm; simp [Nat.log2_zero]
  · exact (Nat.le_log2 (by omega)).2 (Nat.le_trans (Nat.log2_self_le hm) h)

/-- Dependencies on keys no event touches never change the verdict, and
change the steps only through the log factor. -/
theorem currentness_independent_of_unrelated_deps (deps extra : List Key) (events : List Event)
    (h : ∀ k, k ∈ extra → ∀ e, e ∈ events → k ∉ touched e) :
    (currentness (deps ++ extra) events).1 = (currentness deps events).1 ∧
      (currentness (deps ++ extra) events).2 = (currentness deps events).2 +
        sumTouched events * (Nat.log2 (deps ++ extra).length - Nat.log2 deps.length) := by
  refine ⟨currentness_hits_congr deps extra events h, ?_⟩
  rw [currentness_steps_eq, currentness_steps_eq]
  have hle := log2_mono (show deps.length ≤ (deps ++ extra).length by simp)
  rw [show Nat.log2 (deps ++ extra).length + 1 =
      (Nat.log2 deps.length + 1) + (Nat.log2 (deps ++ extra).length - Nat.log2 deps.length)
      by omega, Nat.mul_add]
  omega

end PersistentNaming.TrackedDeps
