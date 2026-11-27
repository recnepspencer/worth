import RelationalSymbolCommit.Dictionary

/-!
# Relational symbol commit: touched-only step bound (public #446)

`commit_steps_bounded`: one commit's dictionary work (reservation of each
touched text into the candidate, then the cutover import of the reserved
meanings into the owner) takes at most `13·|touched|` steps, for an owner
dictionary of any size: no term depends on the existing client keys.

`importFull_steps`: a successful full-snapshot import of a dictionary of
`N` meanings takes exactly `5·N` steps, the cost every commit paid for each
existing client key while a persistent-map path copy deep-copied each
sibling entry's text.

Proofs only: this file models no production Rust of its own, so it carries
no `proof-mirror:` pin; `Dictionary` pins the Rust its definitions model.
-/
namespace RelationalSymbolCommit

theorem importMeaning_steps_le (cap : Nat) (d : Dict) (s t : Nat) :
    (importMeaning cap d s t).2 ≤ 5 := by
  unfold importMeaning; split <;> simp

theorem importMeaning_ok_steps (cap : Nat) (d d' : Dict) (s t : Nat)
    (h : (importMeaning cap d s t).1 = some d') : (importMeaning cap d s t).2 = 5 := by
  unfold importMeaning at h ⊢; split at h <;> simp_all

/-- A batch import takes at most five steps per meaning, whatever `d` holds. -/
theorem importAll_steps_le (cap : Nat) : ∀ (d : Dict) (es : List (Nat × Nat)),
    (importAll cap d es).2 ≤ 5 * es.length
  | d, [] => by simp [importAll]
  | d, (s, t) :: es => by
    simp only [importAll]
    have hm := importMeaning_steps_le cap d s t
    split
    · simp only [List.length_cons]; rename_i k h; rw [h] at hm; simp at hm; omega
    · rename_i d' k h
      have ih := importAll_steps_le cap d' es
      rw [h] at hm; simp at hm
      simp only [List.length_cons]; omega

/-- A successful batch import takes exactly five steps per meaning. -/
theorem importAll_ok_steps (cap : Nat) : ∀ (d d' : Dict) (es : List (Nat × Nat)),
    (importAll cap d es).1 = some d' → (importAll cap d es).2 = 5 * es.length
  | d, d', [], _ => by simp [importAll]
  | d, d', (s, t) :: es, h => by
    simp only [importAll] at h ⊢
    split at h
    · simp at h
    · rename_i d₁ k hm
      simp only at h ⊢
      have h5 := importMeaning_ok_steps cap d d₁ s t (by rw [hm])
      rw [hm] at h5; simp only at h5
      have ih := importAll_ok_steps cap d₁ d' es h
      simp only [List.length_cons]; omega

/-- Rebuilding a dictionary of `N` meanings from its full snapshot. -/
theorem importFull_steps (cap : Nat) (snapshot : List (Nat × Nat)) (d : Dict)
    (h : (importFull cap snapshot).1 = some d) :
    (importFull cap snapshot).2 = 5 * snapshot.length :=
  importAll_ok_steps cap empty d snapshot h

theorem reserve_steps_le (cap : Nat) (owner cand : Dict) (t : Nat) :
    (reserve cap owner cand t).2 ≤ 8 := by
  unfold reserve
  split
  · simp
  · split
    · rename_i s _
      have := importMeaning_steps_le cap cand s t; simp only; omega
    · split
      · simp
      · have := importMeaning_steps_le cap cand owner.next t; simp only; omega

theorem reserveAll_steps_le (cap : Nat) : ∀ (owner cand : Dict) (ts : List Nat),
    (reserveAll cap owner cand ts).2 ≤ 8 * ts.length
  | _, _, [] => by simp [reserveAll]
  | owner, cand, t :: ts => by
    simp only [reserveAll]
    have hr := reserve_steps_le cap owner cand t
    split
    · rename_i k h; rw [h] at hr; simp only at hr; simp only [List.length_cons]; omega
    · rename_i owner' cand' s k h
      rw [h] at hr; simp only at hr
      have ih := reserveAll_steps_le cap owner' cand' ts
      simp only [List.length_cons]; omega

/-- Reservation returns one meaning per touched text. -/
theorem reserveAll_meanings_length (cap : Nat) : ∀ (owner cand : Dict) (ts : List Nat)
    (r : Dict × Dict × List (Nat × Nat)),
    (reserveAll cap owner cand ts).1 = some r → r.2.2.length = ts.length
  | _, _, [], r, h => by simp [reserveAll] at h; subst h; rfl
  | owner, cand, t :: ts, r, h => by
    simp only [reserveAll] at h
    split at h
    · simp at h
    · rename_i owner' cand' s k _
      cases hrest : (reserveAll cap owner' cand' ts).1 with
      | none => rw [hrest] at h; simp at h
      | some r' =>
        rw [hrest] at h; simp only [Option.map_some, Option.some.injEq] at h
        subst h
        simp [reserveAll_meanings_length cap owner' cand' ts r' hrest]

/-- One commit's dictionary work is at most `13·|touched|` steps, for an
owner dictionary of any size `N`. -/
theorem commit_steps_bounded (cap : Nat) (owner : Dict) (touched : List Nat) :
    (commit cap owner touched).2 ≤ 13 * touched.length := by
  unfold commit
  have hr := reserveAll_steps_le cap owner owner touched
  split
  · rename_i k h; rw [h] at hr; simp only at hr; simp only; omega
  · rename_i owner' cand meanings k h
    rw [h] at hr; simp only at hr
    have hl := reserveAll_meanings_length cap owner owner touched (owner', cand, meanings)
      (by rw [h])
    have hi := importAll_steps_le cap owner' meanings
    simp only at hl
    simp only; rw [hl] at hi; omega

/-- Two owner dictionaries of any sizes bound the same commit identically. -/
theorem commit_steps_bound_independent_of_dictionary (cap : Nat) (small large : Dict)
    (touched : List Nat) :
    (commit cap small touched).2 ≤ 13 * touched.length ∧
      (commit cap large touched).2 ≤ 13 * touched.length :=
  ⟨commit_steps_bounded cap small touched, commit_steps_bounded cap large touched⟩

end RelationalSymbolCommit
