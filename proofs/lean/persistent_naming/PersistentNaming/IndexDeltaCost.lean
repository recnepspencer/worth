import PersistentNaming.IndexDelta

/-!
# Name-index batch delta: step bound and history independence (Phase 7B-2)

The cost half of the `IndexDelta` model of `PersistentNameBranchIndex::apply`
with its delta journal (`crates/worth-relational-lineage/src/lineage/index/
{mod.rs,entry.rs,journal.rs}`), and the member carrier update of
`member_groups.rs`. Steps are as in `IndexDelta`: one per applied or refused
operation (saved-set probe plus timeline write) and one per restored journal
entry.

`batch_steps_bounded`: a batch with its rollback takes at most `3·|ops|`
steps. `apply_steps_independent_of_history`: two stores that agree only on
which touched keys exist, whatever their prior timeline lengths or contents
and whatever other keys they hold, give the same step count.

Carriers: a group's carrier set is a persistent, structure-sharing set,
modelled as a membership function so one insert or erase is one step, the
design's cost model rather than a proved property of a concrete persistent
structure (the Rust `im::OrdSet` pays O(log M) per insert or erase, so there
each delta element costs a log factor in the member count `M`, and no step
copies or scans the set). `carrier_update_members` gives the result of a member delta
(departures before arrivals, as `ingest_carriers` orders them) and
`carrier_update_independent_of_group_size` its cost, `|added| + |removed| + 1`
for groups of any member count.
-/
namespace PersistentNaming.IndexDelta

theorem save_length (j : List (Nat × Option Nat)) (k : Nat) (p : Option Nat) :
    (save j k p).length ≤ j.length + 1 := by
  unfold save; split <;> simp

theorem applyAll_steps (c : Nat) : ∀ (ops : List Op) (r : Run),
    (applyAll c ops r).steps + (applyAll c ops r).journal.length ≤
      r.steps + r.journal.length + 3 * ops.length
  | [], r => by simp [applyAll]
  | op :: ops, r => by
    simp only [applyAll]
    split
    · simp only [List.length_cons]; omega
    · have ih := applyAll_steps c ops (applyOp c r op)
      have hop : (applyOp c r op).steps + (applyOp c r op).journal.length ≤
          r.steps + r.journal.length + 3 := by
        cases op with
        | set k x =>
          simp only [applyOp]; split
          · simp only; omega
          · rename_i l _
            have := save_length r.journal k (some l.length); simp only; omega
        | insert k x =>
          simp only [applyOp]; split
          · simp only; omega
          · have := save_length r.journal k none; simp only; omega
      simp only [List.length_cons]; omega

/-- A batch, with its rollback when refused, takes at most `3·|ops|` steps:
no prior timeline length and no other key's count appears. -/
theorem batch_steps_bounded (c : Nat) (ops : List Op) (s : Store) :
    (batch c ops s).2 ≤ 3 * ops.length := by
  have := applyAll_steps c ops (init s)
  have h0 : (init s).steps + (init s).journal.length = 0 := rfl
  simp only [batch]
  generalize applyAll c ops (init s) = r at this ⊢
  cases r.refused <;> simp <;> omega

/-- Two runs that differ only in timeline contents on the touched keys. -/
def Same (K : List Nat) (r r' : Run) : Prop :=
  r.steps = r'.steps ∧ r.refused = r'.refused ∧
    r.journal.map Prod.fst = r'.journal.map Prod.fst ∧
    ∀ k, k ∈ K → (r.state k).isSome = (r'.state k).isSome

theorem lookup_isSome_congr (k : Nat) : ∀ (j j' : List (Nat × Option Nat)),
    j.map Prod.fst = j'.map Prod.fst → (j.lookup k).isSome = (j'.lookup k).isSome
  | [], [], _ => rfl
  | [], _ :: _, h => by simp at h
  | _ :: _, [], h => by simp at h
  | (a, p) :: rest, (a', p') :: rest', h => by
    simp only [List.map_cons, List.cons.injEq] at h
    obtain ⟨rfl, h⟩ := h
    rw [lookup_cons, lookup_cons]
    split
    · rfl
    · exact lookup_isSome_congr k rest rest' h

theorem save_congr (j j' : List (Nat × Option Nat)) (k : Nat) (p p' : Option Nat)
    (h : j.map Prod.fst = j'.map Prod.fst) :
    (save j k p).map Prod.fst = (save j' k p').map Prod.fst := by
  unfold save
  rw [lookup_isSome_congr k j j' h]
  split <;> simp [h]

theorem update_isSome (K : List Nat) (s s' : Store) (k : Nat) (v v' : Option Timeline)
    (h : ∀ q, q ∈ K → (s q).isSome = (s' q).isSome) (hv : v.isSome = v'.isSome) :
    ∀ q, q ∈ K → (update s k v q).isSome = (update s' k v' q).isSome := by
  intro q hq
  simp only [update]
  split
  · exact hv
  · exact h q hq

theorem applyOp_same (c : Nat) (K : List Nat) (r r' : Run) (h : Same K r r') :
    ∀ op, op.key ∈ K → Same K (applyOp c r op) (applyOp c r' op)
  | .set k x, hk => by
    obtain ⟨h1, h2, h3, h4⟩ := h
    have hs := h4 k hk
    simp only [applyOp]
    cases hr : r.state k <;> cases hr' : r'.state k <;> simp only [hr, hr'] at hs ⊢
    · exact ⟨by simp [h1], by simp, h3, h4⟩
    · cases hs
    · cases hs
    · exact ⟨by simp [h1], h2, save_congr _ _ _ _ _ h3, update_isSome K _ _ _ _ _ h4 rfl⟩
  | .insert k x, hk => by
    obtain ⟨h1, h2, h3, h4⟩ := h
    have hs := h4 k hk
    simp only [applyOp]
    cases hr : r.state k <;> cases hr' : r'.state k <;> simp only [hr, hr'] at hs ⊢
    · exact ⟨by simp [h1], h2, save_congr _ _ _ _ _ h3, update_isSome K _ _ _ _ _ h4 rfl⟩
    · cases hs
    · cases hs
    · exact ⟨by simp [h1], by simp, h3, h4⟩

theorem applyAll_same (c : Nat) (K : List Nat) : ∀ (ops : List Op) (r r' : Run),
    (∀ op, op ∈ ops → op.key ∈ K) → Same K r r' → Same K (applyAll c ops r) (applyAll c ops r')
  | [], _, _, _, h => h
  | op :: ops, r, r', hK, h => by
    simp only [applyAll, h.2.1]
    split
    · exact h
    · exact applyAll_same c K ops _ _ (fun o ho => hK o (List.mem_cons_of_mem op ho))
        (applyOp_same c K r r' h op (hK op List.mem_cons_self))

/-- Two stores agreeing only on which touched keys exist — whatever their
prior timeline lengths and contents, and whatever other keys they hold —
give one batch the same step count. -/
theorem apply_steps_independent_of_history (c : Nat) (ops : List Op) (s s' : Store)
    (h : ∀ k, k ∈ ops.map Op.key → (s k).isSome = (s' k).isSome) :
    (batch c ops s).2 = (batch c ops s').2 := by
  have hsame := applyAll_same c (ops.map Op.key) ops (init s) (init s')
    (fun op hop => List.mem_map.2 ⟨op, hop, rfl⟩) ⟨rfl, rfl, rfl, h⟩
  simp only [batch]
  generalize applyAll c ops (init s) = r at hsame ⊢
  generalize applyAll c ops (init s') = r' at hsame ⊢
  obtain ⟨h1, h2, h3, _⟩ := hsame
  have hl : r.journal.length = r'.journal.length := by
    rw [← List.length_map (f := Prod.fst), h3, List.length_map]
  rw [h2]
  cases r'.refused <;> simp [h1, hl]

/-- A group's carrier set: a persistent set, read by membership. -/
abbrev Carriers := Nat → Bool

def eraseAll : Carriers → List Nat → Carriers × Nat
  | m, [] => (m, 0)
  | m, x :: xs =>
    let r := eraseAll (fun y => if y = x then false else m y) xs
    (r.1, r.2 + 1)

def insertAll : Carriers → List Nat → Carriers × Nat
  | m, [] => (m, 0)
  | m, x :: xs =>
    let r := insertAll (fun y => if y = x then true else m y) xs
    (r.1, r.2 + 1)

/-- One commit's carrier update by a member delta: departures, then
arrivals. -/
def carrierUpdate (m : Carriers) (added removed : List Nat) : Carriers × Nat :=
  let r := eraseAll m removed
  let a := insertAll r.1 added
  (a.1, a.2 + r.2 + 1)

theorem eraseAll_spec : ∀ (xs : List Nat) (m : Carriers),
    (eraseAll m xs).2 = xs.length ∧ ∀ y, (eraseAll m xs).1 y = (m y && !decide (y ∈ xs))
  | [], m => by simp [eraseAll]
  | x :: xs, m => by
    obtain ⟨h1, h2⟩ := eraseAll_spec xs (fun y => if y = x then false else m y)
    refine ⟨by simp only [eraseAll, h1, List.length_cons], fun y => ?_⟩
    simp only [eraseAll, h2]
    by_cases hy : y = x <;> simp [hy]

theorem insertAll_spec : ∀ (xs : List Nat) (m : Carriers),
    (insertAll m xs).2 = xs.length ∧ ∀ y, (insertAll m xs).1 y = (decide (y ∈ xs) || m y)
  | [], m => by simp [insertAll]
  | x :: xs, m => by
    obtain ⟨h1, h2⟩ := insertAll_spec xs (fun y => if y = x then true else m y)
    refine ⟨by simp only [insertAll, h1, List.length_cons], fun y => ?_⟩
    simp only [insertAll, h2]
    by_cases hy : y = x <;> simp [hy]

/-- The updated carriers are the prior ones minus the departures plus the
arrivals. -/
theorem carrier_update_members (m : Carriers) (added removed : List Nat) (y : Nat) :
    (carrierUpdate m added removed).1 y = (decide (y ∈ added) || (m y && !decide (y ∈ removed))) := by
  simp only [carrierUpdate, (insertAll_spec _ _).2, (eraseAll_spec _ _).2]

/-- The carrier update costs `|added| + |removed| + 1` steps for groups of
any sizes. -/
theorem carrier_update_independent_of_group_size (m m' : Carriers) (added removed : List Nat) :
    (carrierUpdate m added removed).2 = (carrierUpdate m' added removed).2 ∧
      (carrierUpdate m added removed).2 = added.length + removed.length + 1 := by
  constructor <;> simp only [carrierUpdate, (insertAll_spec _ _).1, (eraseAll_spec _ _).1]

end PersistentNaming.IndexDelta
