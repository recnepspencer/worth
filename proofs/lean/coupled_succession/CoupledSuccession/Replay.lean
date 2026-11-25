import CoupledSuccession.Model

/-!
# Allocation evidence replay (design §2.5b step 3; manifest T14)

Recovery stages, per `(class, partition)`, the append-frontier slots of `live ∪ cancelled`:
no duplicates, and an exact contiguous block `[f0, f0 + n)` (`f0 = F_start`, `next_slots = F_end`).
Cancelled slots are burned. `reserve_exact` is then called for the live creates in
`merged_intents` order: each slot must be in the block, not cancelled and not yet consumed;
monotone order is not required. `finish_mutation` requires every staged live entry consumed.
-/
namespace CoupledSuccession

/-- The slots lie in `[f0, f0 + length)`; with `NoDup` they fill that block exactly. -/
def Block (f0 : Nat) (l : List Nat) : Prop := ∀ x ∈ l, f0 ≤ x ∧ x < f0 + l.length

/-- Executable duplicate check used by staging. -/
def noDupB : List Nat → Bool
  | [] => true
  | x :: xs => !xs.contains x && noDupB xs

structure Staged where
  start : Nat
  stop : Nat
  cancelled : List Nat
  consumed : List Nat

/-- `stage_replay_allocations(live, cancelled)`. -/
def stage (f0 : Nat) (live cancelled : List Nat) : Option Staged :=
  let all := live ++ cancelled
  if noDupB all && all.all (fun x => decide (f0 ≤ x ∧ x < f0 + all.length)) then
    some ⟨f0, f0 + all.length, cancelled, []⟩
  else none

/-- `reserve_exact` for one live `AppendFrontier` create. -/
def reserveExact (st : Staged) (slot : Nat) : Option Staged :=
  if decide (st.start ≤ slot ∧ slot < st.stop) && !st.cancelled.contains slot &&
      !st.consumed.contains slot then
    some { st with consumed := slot :: st.consumed }
  else none

def reserveAll : Staged → List Nat → Option Staged
  | st, [] => some st
  | st, x :: xs =>
    match reserveExact st x with
    | some st' => reserveAll st' xs
    | none => none

/-- `finish_mutation`: every staged live entry is consumed. -/
def finish (st : Staged) (live : List Nat) : Bool := live.all (fun x => st.consumed.contains x)

/-- Replay of one envelope: stage, reserve the live creates in the given order, finish. -/
def replay (f0 : Nat) (live cancelled : List Nat) : Bool :=
  match stage f0 live cancelled with
  | none => false
  | some st =>
    match reserveAll st live with
    | none => false
    | some st' => finish st' live

def ReplayAccepts (f0 : Nat) (live cancelled : List Nat) : Prop := replay f0 live cancelled = true

theorem noDupB_of_nodup : ∀ {l : List Nat}, List.Nodup l → noDupB l = true := by
  intro l
  induction l with
  | nil => intro _; rfl
  | cons x xs ih =>
    intro h
    rw [List.nodup_cons] at h
    simp [noDupB, h.1, ih h.2]

theorem reserveAll_ok (start stop : Nat) (cancelled : List Nat) : ∀ (xs consumed : List Nat),
    (∀ x ∈ xs, start ≤ x ∧ x < stop) → (∀ x ∈ xs, x ∉ cancelled) → List.Nodup xs →
    (∀ x ∈ xs, x ∉ consumed) →
    reserveAll ⟨start, stop, cancelled, consumed⟩ xs =
      some ⟨start, stop, cancelled, xs.reverse ++ consumed⟩ := by
  intro xs
  induction xs with
  | nil => intro consumed _ _ _ _; rfl
  | cons x xs ih =>
    intro consumed hb hc hn hu
    rw [List.nodup_cons] at hn
    have hx : reserveExact ⟨start, stop, cancelled, consumed⟩ x =
        some ⟨start, stop, cancelled, x :: consumed⟩ := by
      have h1 := hb x List.mem_cons_self
      have h2 := hc x List.mem_cons_self
      have h3 := hu x List.mem_cons_self
      simp [reserveExact, h1, h2, h3]
    simp only [reserveAll, hx]
    rw [ih (x :: consumed) (fun y hy => hb y (List.mem_cons_of_mem _ hy))
      (fun y hy => hc y (List.mem_cons_of_mem _ hy)) hn.2]
    · simp
    · intro y hy hy'
      rcases List.mem_cons.mp hy' with h | h
      · subst h; exact hn.1 hy
      · exact hu y (List.mem_cons_of_mem _ hy) h

namespace Ledger

/-- T14: recovery accepts the live evidence in any order when, together with the cancelled
slots, it forms one contiguous block from `f0` without duplicates. -/
theorem evidence_block_contiguous :
    ∀ (live cancelled : List Nat) (f0 : Nat), Block f0 (live ++ cancelled) →
      NoDup (live ++ cancelled) → ∀ perm, Perm perm live → ReplayAccepts f0 perm cancelled := by
  intro live cancelled f0 hblock hnodup perm hperm
  have hp : List.Perm (perm ++ cancelled) (live ++ cancelled) := List.Perm.append_right _ hperm
  have hn : List.Nodup (perm ++ cancelled) := hp.nodup_iff.mpr hnodup
  have hlen : (perm ++ cancelled).length = (live ++ cancelled).length := hp.length_eq
  have hb : ∀ x ∈ perm ++ cancelled, f0 ≤ x ∧ x < f0 + (perm ++ cancelled).length := by
    intro x hx
    rw [hlen]; exact hblock x (hp.mem_iff.mp hx)
  have hstage : stage f0 perm cancelled =
      some ⟨f0, f0 + (perm ++ cancelled).length, cancelled, []⟩ := by
    have h1 : noDupB (perm ++ cancelled) = true := noDupB_of_nodup hn
    have h2 : (perm ++ cancelled).all
        (fun x => decide (f0 ≤ x ∧ x < f0 + (perm ++ cancelled).length)) = true := by
      rw [List.all_eq_true]; intro x hx; simpa using hb x hx
    simp only [stage, h1, h2, Bool.and_self, ite_true]
  rw [List.nodup_append] at hn
  have hres := reserveAll_ok f0 (f0 + (perm ++ cancelled).length) cancelled perm []
    (fun x hx => hb x (List.mem_append_left _ hx))
    (fun x hx hc => hn.2.2 x hx x hc rfl) hn.1 (fun _ _ h => by cases h)
  unfold ReplayAccepts replay
  rw [hstage]
  dsimp only
  rw [hres]
  simp [finish]

end Ledger
end CoupledSuccession
