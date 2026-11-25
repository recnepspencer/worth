import CoupledSuccession.MultiChain

/-!
# Compaction (design §2.7; manifest T5, T6)

`compactAll` drops masked entries and moves every chain's live segment to its head position.
It reproduces `R(E)`'s log (T5) and never keeps an entry that no live segment writes (T6, M-E2).
-/
namespace CoupledSuccession

/-- The accumulator entries of a segment list: who wrote what, in order. -/
def entries (segs : List Seg) : List (Owner × Eff) := segs.map (fun s => (s.owner, s.eff))

/-- Some entry of the list changes record `r`. -/
def InEntries (r : Rec) (segs : List Seg) : Prop := ∃ s ∈ segs, s.eff.dom r = true

namespace Elision

/-- T5: compaction across every chain equals `R(E)`'s log, up to permuting entries.
(Here the compacted order coincides with `R(E)`'s exactly, so the permutation is the identity.) -/
theorem compactAll_eq_reference_log :
    ∀ (b : Store) (steps : List Step), WellGuarded b steps →
      foldSegments (compactAll (liveRun b steps).log) b = foldSegments (refLog steps) b ∧
        Perm (entries (compactAll (liveRun b steps).log)) (entries (refLog steps)) := by
  intro b steps _
  rw [compactAll_liveRun_eq_refLog]
  exact ⟨rfl, List.Perm.refl _⟩

/-- T6 (M-E2): every compacted entry is a live segment's write. -/
theorem no_phantom_entry :
    ∀ (b : Store) (steps : List Step) (r : Rec), WellGuarded b steps →
      InEntries r (compactAll (liveRun b steps).log) →
        ∃ o e, LiveSeg o (liveRun b steps) = some e ∧ e.dom r = true := by
  intro b steps r _ ⟨s, hs, hr⟩
  refine ⟨s.owner, s.eff, ?_, hr⟩
  simp [LiveSeg, mem_compactAll hs]

end Elision
end CoupledSuccession
