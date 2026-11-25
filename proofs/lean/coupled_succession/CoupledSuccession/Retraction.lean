import CoupledSuccession.Model

/-!
# Retraction (design §2.4; manifest T1–T3)

`retract b w` restores the chain's records `w` from the head image `b`
(`retract_chain_segment`). Round `k` is then applied as authored.
-/
namespace CoupledSuccession.Retraction

open CoupledSuccession

/-- T1: retraction undoes only the chain's own writes, beside a foreign write outside `w`. -/
theorem retract_cancels_chain_beside_foreign :
    ∀ (b : Store) (w : Dom) (e f : Eff), Sub e.dom w → Disj f.dom w →
      retract b w (apply f (apply e b)) = apply f b := by
  intro b w e f he hf
  funext r
  cases hw : w r with
  | true =>
    have hfr : f.dom r = false := disj_left hf hw
    simp [retract, apply, hw, hfr]
  | false =>
    have her : e.dom r = false := sub_false he hw
    simp [retract, apply, hw, her]

/-- T2: retract then re-apply round `k` equals the single-shot answer (Option A: `ek ⟂ f`). -/
theorem successor_eq_single_shot :
    ∀ (b : Store) (w : Dom) (e ek f : Eff), Sub e.dom w → Disj f.dom w → Disj ek.dom f.dom →
      apply ek (retract b w (apply f (apply e b))) = apply f (apply ek b) := by
  intro b w e ek f he hf hk
  rw [retract_cancels_chain_beside_foreign b w e f he hf]
  exact apply_comm_of_disj hk b

/-- T3: Query's overlay view equals Native's retracted image. -/
theorem view_eq_retracted :
    ∀ (b : Store) (w : Dom) (e f : Eff), Sub e.dom w → Disj f.dom w →
      overlay b w (apply f (apply e b)) = apply f b :=
  retract_cancels_chain_beside_foreign

end CoupledSuccession.Retraction
