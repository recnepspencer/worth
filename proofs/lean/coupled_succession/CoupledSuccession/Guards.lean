import CoupledSuccession.Retraction

/-!
# Guards (design §3; manifest T10, T11, T11f, T12)
-/
namespace CoupledSuccession

/-- `q` reads only the records in `supp`. -/
def DependsOnly (q : Store → Nat) (supp : Dom) : Prop :=
  ∀ s s' : Store, (∀ r, supp r = true → s r = s' r) → q s = q s'

namespace Guards

/-- T10: `PriorEffectRead` with exact touches. A read whose support avoids the chain's prior
touches `t ⊇ w ⊇ e.dom` cannot observe the superseded round. -/
theorem prior_effect_guard_sound :
    ∀ (b : Store) (w t supp : Dom) (e f : Eff) (q : Store → Nat), Sub e.dom w → Sub w t →
      Disj f.dom w → Disj supp t → DependsOnly q supp →
        q (apply f (apply e b)) = q (apply f b) := by
  intro b w t supp e f q he hw _ hs hq
  apply hq
  intro r hr
  have ht : t r = false := hs r hr
  have hwr : w r = false := sub_false hw ht
  have her : e.dom r = false := sub_false he hwr
  exact apply_congr_pt f (apply_of_not her)

/-- A one-record domain. -/
def single (r0 : Rec) : Dom := fun r => decide (r = r0)

/-- T11: the backward guard is necessary. -/
theorem intervened_necessary :
    ∃ (b : Store) (w : Dom) (e f : Eff), Sub e.dom w ∧ ¬ Disj f.dom w ∧
      retract b w (apply f (apply e b)) ≠ apply f b := by
  refine ⟨noneStore, single 0, ⟨single 0, fun _ => some 1⟩, ⟨single 0, fun _ => some 2⟩,
    fun _ h => h, ?_, ?_⟩
  · intro h
    have := h 0 (by decide)
    simp [single] at this
  · intro h
    have := congrFun h 0
    simp [retract, apply, single, noneStore] at this

/-- T11f: the forward guard (Option A) is necessary. -/
theorem forward_guard_necessary :
    ∃ (b : Store) (w : Dom) (e ek f : Eff), Sub e.dom w ∧ Disj f.dom w ∧ ¬ Disj ek.dom f.dom ∧
      apply ek (retract b w (apply f (apply e b))) ≠ apply f (apply ek b) := by
  refine ⟨noneStore, single 0, ⟨single 0, fun _ => some 1⟩, ⟨single 1, fun _ => some 3⟩,
    ⟨single 1, fun _ => some 2⟩, fun _ h => h, ?_, ?_, ?_⟩
  · intro r hr
    simp only [single, decide_eq_true_eq] at hr
    subst hr; decide
  · intro h
    have := h 1 (by decide)
    simp [single] at this
  · intro h
    have := congrFun h 1
    simp [apply, single] at this

/-- T12: cancelling records no live foreign link names leaves no dangling link. -/
theorem retired_reference_guard_sound :
    ∀ (s : Store) (links : Rec → Rec → Bool) (cancelled : Dom),
      (∀ src tgt, links src tgt = true → cancelled src = false ∧ cancelled tgt = false) →
      Integrity s links → Integrity (retract noneStore cancelled s) links := by
  intro s links cancelled hc hi src tgt hl
  obtain ⟨h1, h2⟩ := hc src tgt hl
  obtain ⟨h3, h4⟩ := hi src tgt hl
  simp [retract, h1, h2, h3, h4]

end Guards
end CoupledSuccession
