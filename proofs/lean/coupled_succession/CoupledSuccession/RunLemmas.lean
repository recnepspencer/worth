import CoupledSuccession.Run

/-!
# Structural lemmas about the live log

How `liveOf` and `compactAll` move when a member is appended, and the structural invariant
of `liveRun` (owners are fresh, every chain has exactly one live segment at or after its head).
-/
namespace CoupledSuccession

theorem liveOf_some {log : List Seg} {o : Owner} {y : Seg} (h : liveOf log o = some y) :
    y ∈ log ∧ y.owner = o ∧ y.masked = false := by
  refine ⟨List.mem_of_find?_eq_some h, ?_⟩
  have := List.find?_some h
  simpa using this

theorem liveOf_none_of_fresh {log : List Seg} {o : Owner} (h : ∀ s ∈ log, s.owner ≠ o) :
    liveOf log o = none := by
  unfold liveOf
  rw [List.find?_eq_none]
  intro x hx
  simp [h x hx]

theorem liveOf_append_other {log : List Seg} {o : Owner} {x : Seg} (h : x.owner ≠ o) :
    liveOf (log ++ [x]) o = liveOf log o := by
  unfold liveOf
  rw [List.find?_append]
  simp [h]

theorem liveOf_append_new {log : List Seg} {o : Owner} {x : Seg} (h0 : liveOf log o = none)
    (ho : x.owner = o) (hm : x.masked = false) : liveOf (log ++ [x]) o = some x := by
  unfold liveOf at *
  rw [List.find?_append, h0]
  simp [ho, hm]

theorem liveOf_mask_self (o : Owner) (log : List Seg) : liveOf (maskOwner o log) o = none := by
  unfold liveOf maskOwner
  rw [List.find?_eq_none]
  intro x hx
  rw [List.mem_map] at hx
  obtain ⟨s, _, rfl⟩ := hx
  by_cases hs : s.owner = o <;> simp [hs]

theorem liveOf_mask_other {o o' : Owner} (h : o' ≠ o) (log : List Seg) :
    liveOf (maskOwner o log) o' = liveOf log o' := by
  induction log with
  | nil => rfl
  | cons s log ih =>
    unfold liveOf maskOwner at *
    simp only [List.map_cons, List.find?_cons]
    by_cases hs : s.owner = o
    · have hoo : ¬ (o = o') := fun e => h e.symm
      simp [hs, hoo, ih]
    · simp [hs, ih]

theorem filterMap_congr' {α β : Type} {f g : α → Option β} :
    ∀ {l : List α}, (∀ a ∈ l, f a = g a) → l.filterMap f = l.filterMap g := by
  intro l
  induction l with
  | nil => intro _; rfl
  | cons a l ih =>
    intro h
    simp only [List.filterMap_cons]
    rw [h a List.mem_cons_self, ih (fun x hx => h x (List.mem_cons_of_mem a hx))]

/-- Entries of the compacted log are live segments of their owners. -/
theorem mem_compactAll {log : List Seg} {s : Seg} (h : s ∈ compactAll log) :
    liveOf log s.owner = some s := by
  unfold compactAll at h
  rw [List.mem_filterMap] at h
  obtain ⟨a, _, ha⟩ := h
  by_cases hh : a.head
  · simp only [hh, ite_true] at ha
    have := (liveOf_some ha).2.1
    rw [this]
    exact ha
  · simp [hh] at ha

theorem mem_log_of_mem_compactAll {log : List Seg} {s : Seg} (h : s ∈ compactAll log) :
    s ∈ log :=
  (liveOf_some (mem_compactAll h)).1

/-- Appending a fresh member's head segment appends it to the compacted log. -/
theorem compactAll_append_fresh {log : List Seg} {x : Seg} (hf : ∀ s ∈ log, s.owner ≠ x.owner)
    (hh : x.head = true) (hm : x.masked = false) :
    compactAll (log ++ [x]) = compactAll log ++ [x] := by
  unfold compactAll
  rw [List.filterMap_append]
  congr 1
  · apply filterMap_congr'
    intro a ha
    rw [liveOf_append_other (fun h => hf a ha h.symm)]
  · have h0 : liveOf log x.owner = none := liveOf_none_of_fresh hf
    simp [hh, liveOf_append_new h0 rfl hm]

theorem replaceOwner_owner {o : Owner} {x s : Seg} (hx : x.owner = o) :
    (replaceOwner o x s).owner = s.owner := by
  unfold replaceOwner
  by_cases h : s.owner = o
  · simp [h, hx]
  · simp [h]

theorem replaceOwner_of_ne {o : Owner} {x s : Seg} (h : s.owner ≠ o) : replaceOwner o x s = s := by
  simp [replaceOwner, h]

theorem replaceOwner_of_eq {o : Owner} {x s : Seg} (h : s.owner = o) : replaceOwner o x s = x := by
  simp [replaceOwner, h]

theorem map_replaceOwner_id {o : Owner} {x : Seg} {l : List Seg} (h : ∀ s ∈ l, s.owner ≠ o) :
    l.map (replaceOwner o x) = l := by
  conv => rhs; rw [← List.map_id l]
  apply List.map_congr_left
  intro s hs
  exact replaceOwner_of_ne (h s hs)

theorem maskOwner_owner (o : Owner) (s : Seg) :
    (if s.owner = o then { s with masked := true } else s).owner = s.owner := by
  by_cases h : s.owner = o <;> simp [h]

theorem maskOwner_head (o : Owner) (s : Seg) :
    (if s.owner = o then { s with masked := true } else s).head = s.head := by
  by_cases h : s.owner = o <;> simp [h]

/-- A successor round replaces the chain's entry of the compacted log by its new round. -/
theorem compactAll_succ {log : List Seg} {o : Owner} {x : Seg} (hl : (liveOf log o).isSome)
    (ho : x.owner = o) (hh : x.head = false) (hm : x.masked = false) :
    compactAll (maskOwner o log ++ [x]) = (compactAll log).map (replaceOwner o x) := by
  unfold compactAll
  rw [List.filterMap_append, List.map_filterMap]
  have hx : [x].filterMap (fun s => if s.head then liveOf (maskOwner o log ++ [x]) s.owner
      else none) = [] := by simp [hh]
  rw [hx, List.append_nil]
  unfold maskOwner
  rw [List.filterMap_map]
  apply filterMap_congr'
  intro s _
  simp only [Function.comp, maskOwner_head, maskOwner_owner]
  by_cases hsh : s.head
  · simp only [hsh, ite_true]
    by_cases hso : s.owner = o
    · rw [hso]
      have h0 := liveOf_mask_self o log
      unfold maskOwner at h0
      rw [liveOf_append_new h0 ho hm]
      · obtain ⟨y, hy⟩ := Option.isSome_iff_exists.mp hl
        rw [hy]
        simp [replaceOwner_of_eq (liveOf_some hy).2.1]
    · rw [liveOf_append_other (by rw [ho]; exact fun h => hso h.symm)]
      have := liveOf_mask_other hso log
      unfold maskOwner at this
      rw [this]
      cases hy : liveOf log s.owner with
      | none => rfl
      | some y =>
        have hyo := (liveOf_some hy).2.1
        simp [replaceOwner_of_ne (by rw [hyo]; exact hso)]
  · simp [hsh]

theorem getElem?_maskOwner_of_ne {o : Owner} {log : List Seg} {i : Nat} {s : Seg}
    (h : log[i]? = some s) (hs : s.owner ≠ o) : (maskOwner o log)[i]? = some s := by
  unfold maskOwner
  rw [List.getElem?_map, h]
  simp [hs]

theorem mem_maskOwner {o : Owner} {log : List Seg} {s : Seg} (h : s ∈ maskOwner o log) :
    ∃ t ∈ log, s.owner = t.owner := by
  unfold maskOwner at h
  rw [List.mem_map] at h
  obtain ⟨t, ht, rfl⟩ := h
  exact ⟨t, ht, maskOwner_owner o t⟩

theorem length_maskOwner (o : Owner) (log : List Seg) : (maskOwner o log).length = log.length := by
  simp [maskOwner]

theorem getElem?_append_lt {log : List Seg} {x : Seg} {i : Nat} {s : Seg} (h : log[i]? = some s) :
    (log ++ [x])[i]? = some s := by
  have hi : i < log.length := by
    rcases Nat.lt_or_ge i log.length with hi | hi
    · exact hi
    · rw [List.getElem?_eq_none hi] at h; cases h
  rw [List.getElem?_append_left hi, h]

theorem lt_length_of_getElem? {log : List Seg} {i : Nat} {s : Seg} (h : log[i]? = some s) :
    i < log.length := by
  rcases Nat.lt_or_ge i log.length with hi | hi
  · exact hi
  · rw [List.getElem?_eq_none hi] at h; cases h

end CoupledSuccession
