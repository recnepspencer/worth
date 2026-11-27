/-!
# `mint_order::dependency_order` step bound (Phase 7B-2, RULES §4 layer 1)

Models the Kahn loop of `crates/worth-relational-lineage/src/lineage/
authority/mint_order.rs`: `waits[p]` counts the in-commit births `p` still
waits on, `releases[d]` lists the births waiting on `d`; the loop pops a
ready birth, then decrements `waits` of every birth it releases, readying
each that reaches zero. A step is one pop or one release visit.

The workspace builds with `-C overflow-checks=on`, so a decrement of a zero
`waits` panics: a run that completes never performs one. The model makes
that decrement abort the run (`none`). For every completed run, whatever
the dependency graph (cyclic or not, duplicated edges or not), the steps
taken are at most `|ready₀| + 2·D`, where `D = Σ waits₀` is the number of
declared in-commit dependencies, so at most `B + 2·D` for `B` births. No
other quantity, in particular no basis or branch name count, enters.
-/
namespace PersistentNaming.MintOrder

/-- `sumTo f n = f 0 + ⋯ + f (n - 1)`. -/
def sumTo (f : Nat → Nat) : Nat → Nat
  | 0 => 0
  | k + 1 => sumTo f k + f k

theorem sumTo_congr (f g : Nat → Nat) : ∀ n, (∀ q, q < n → f q = g q) → sumTo f n = sumTo g n
  | 0, _ => rfl
  | k + 1, h => by
    simp only [sumTo]
    rw [sumTo_congr f g k (fun q hq => h q (by omega)), h k (by omega)]

/-- `f` with position `p` decremented. -/
def dec (f : Nat → Nat) (p : Nat) : Nat → Nat :=
  fun q => if q = p then f p - 1 else f q

theorem sumTo_dec (f : Nat → Nat) (p : Nat) (hf : f p ≠ 0) :
    ∀ n, p < n → sumTo (dec f p) n + 1 = sumTo f n
  | 0, h => absurd h (Nat.not_lt_zero p)
  | k + 1, h => by
    simp only [sumTo]
    by_cases hk : p = k
    · subst hk
      have : sumTo (dec f p) p = sumTo f p :=
        sumTo_congr _ _ p (fun q hq => by simp [dec]; omega)
      simp [dec, this]; omega
    · have ih := sumTo_dec f p hf k (by omega)
      have : dec f p k = f k := by simp [dec]; omega
      rw [this]; omega

/-- Decrements `waits` of every listed birth; `none` on a zero decrement
(an overflow-checked panic); otherwise the new waits and the births that
reached zero. -/
def release (waits : Nat → Nat) : List Nat → Option ((Nat → Nat) × List Nat)
  | [] => some (waits, [])
  | p :: ps =>
    if waits p = 0 then none
    else
      match release (dec waits p) ps with
      | none => none
      | some (w, zeros) => some (w, if dec waits p p = 0 then p :: zeros else zeros)

theorem release_spec (n : Nat) : ∀ (l : List Nat) (waits w : Nat → Nat) (zeros : List Nat),
    (∀ p, p ∈ l → p < n) → release waits l = some (w, zeros) →
    sumTo waits n = sumTo w n + l.length ∧ zeros.length ≤ l.length
  | [], waits, w, zeros, _, h => by
    simp [release] at h
    obtain ⟨rfl, rfl⟩ := h
    simp
  | p :: ps, waits, w, zeros, hl, h => by
    simp only [release] at h
    split at h
    · cases h
    · rename_i hp
      split at h
      · cases h
      · rename_i w' zeros' hr
        injection h with h
        injection h with hw hz
        subst hw
        have ih := release_spec n ps (dec waits p) w' zeros'
          (fun q hq => hl q (List.mem_cons_of_mem p hq)) hr
        have hdec := sumTo_dec waits p hp n (hl p (List.mem_cons_self))
        subst hz
        constructor
        · simp; omega
        · split <;> simp <;> omega

/-- The loop state: the ready births, the waits, the steps taken. -/
structure State where
  ready : List Nat
  waits : Nat → Nat
  steps : Nat

/-- One loop iteration: pop a ready birth and release its dependents. -/
def step (releases : Nat → List Nat) (s : State) : Option State :=
  match s.ready with
  | [] => some s
  | x :: rest =>
    match release s.waits (releases x) with
    | none => none
    | some (w, zeros) =>
      some { ready := rest ++ zeros, waits := w, steps := s.steps + 1 + (releases x).length }

/-- `fuel` iterations, aborting on an overflow-checked panic. -/
def run (releases : Nat → List Nat) : Nat → State → Option State
  | 0, s => some s
  | k + 1, s =>
    match step releases s with
    | none => none
    | some s' => run releases k s'

/-- Potential: twice the outstanding waits plus the ready births. -/
def potential (n : Nat) (s : State) : Nat := 2 * sumTo s.waits n + s.ready.length

theorem step_spends_potential (n : Nat) (releases : Nat → List Nat)
    (hrel : ∀ x p, p ∈ releases x → p < n) (s s' : State) :
    step releases s = some s' → s'.steps + potential n s' ≤ s.steps + potential n s := by
  intro h
  simp only [step] at h
  split at h
  · injection h with h; subst h; omega
  · rename_i x rest hready
    split at h
    · cases h
    · rename_i w zeros hr
      injection h with h
      subst h
      obtain ⟨hsum, hzeros⟩ := release_spec n _ _ _ _ (hrel x) hr
      simp only [potential, hready, List.length_cons, List.length_append]
      omega

theorem run_spends_potential (n : Nat) (releases : Nat → List Nat)
    (hrel : ∀ x p, p ∈ releases x → p < n) :
    ∀ (fuel : Nat) (s s' : State), run releases fuel s = some s' →
      s'.steps + potential n s' ≤ s.steps + potential n s
  | 0, s, s', h => by simp [run] at h; subst h; omega
  | k + 1, s, s', h => by
    simp only [run] at h
    split at h
    · cases h
    · rename_i s₁ h₁
      have := step_spends_potential n releases hrel s s₁ h₁
      have := run_spends_potential n releases hrel k s₁ s' h
      omega

/-- Every completed run of the mint-order loop over `n` births with initial
ready set `ready₀` takes at most `|ready₀| + 2·Σ waits₀` steps, hence at most
`n + 2·D` for `D` in-commit dependencies. -/
theorem kahn_steps_bounded (n : Nat) (releases : Nat → List Nat)
    (hrel : ∀ x p, p ∈ releases x → p < n)
    (ready₀ : List Nat) (waits₀ : Nat → Nat) (hready : ready₀.length ≤ n)
    (fuel : Nat) (s' : State) :
    run releases fuel { ready := ready₀, waits := waits₀, steps := 0 } = some s' →
    s'.steps ≤ n + 2 * sumTo waits₀ n := by
  intro h
  have := run_spends_potential n releases hrel fuel _ s' h
  simp only [potential] at this
  omega

end PersistentNaming.MintOrder
