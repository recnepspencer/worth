/-!
# `ChunkedCoverageAndSummarySound` (Store Milestone 4, Phase M4.3)

Model of `worth-relational-storage`'s chunked placement substrate
(`storage/residency/chunked`):

* `SlotRange::fixed_chunk_of` / `fixed_partition` cut `[0, total)` into
  fixed chunks `[k * c, min ((k + 1) * c) total)`.
* `ChunkCommitmentHasher` hashes only the canonical visit of a chunk: each
  slot's head, its logical history (spilled continuation pages, then inline
  history rows concatenated in layer order) and its adjacency. Physical
  locators never enter it; `consolidate_history_chunk` flattens layers and
  replaces only the binding's `history_locator`.
* `seal_chunk_image` derives every summary from the canonical records it
  hashes; `scan_chunks` skips a chunk only when its summary omits the key.

The theorems hold for every chunk width, slot count, hash function, layer
structure and record set.
-/

namespace StoreM4.ChunkedCoverage

/-! ## Fixed non-overlapping partition -/

/-- Half-open slot range `[start, stop)`. -/
structure SlotRange where
  start : Nat
  stop : Nat
  deriving DecidableEq, Repr

def SlotRange.contains (r : SlotRange) (slot : Nat) : Prop :=
  r.start ≤ slot ∧ slot < r.stop

/-- Chunk `k` of a `total`-slot domain with chunk width `c`. -/
def fixedChunk (c total k : Nat) : SlotRange :=
  ⟨k * c, min ((k + 1) * c) total⟩

/-- Number of chunks in the fixed partition. -/
def chunkCount (c total : Nat) : Nat :=
  (total + c - 1) / c

/-- The fixed partition, chunk indices ascending. -/
def fixedPartition (c total : Nat) : List SlotRange :=
  (List.range (chunkCount c total)).map (fixedChunk c total)

theorem slot_in_own_chunk (c total slot : Nat) (hc : 0 < c) (hs : slot < total) :
    (fixedChunk c total (slot / c)).contains slot := by
  refine ⟨Nat.div_mul_le_self slot c, ?_⟩
  have hlt : slot < slot / c * c + c := Nat.lt_div_mul_add hc
  have hmul : (slot / c + 1) * c = slot / c * c + c := Nat.succ_mul (slot / c) c
  simp only [fixedChunk, Nat.lt_min]
  exact ⟨hmul ▸ hlt, hs⟩

theorem chunk_of_slot_unique (c total slot k : Nat)
    (h : (fixedChunk c total k).contains slot) : k = slot / c := by
  rcases h with ⟨hlo, hhi⟩
  have hhi' : slot < (k + 1) * c := Nat.lt_of_lt_of_le hhi (Nat.min_le_left _ _)
  exact (Nat.div_eq_of_lt_le hlo hhi').symm

theorem own_chunk_in_partition (c total slot : Nat) (hc : 0 < c) (hs : slot < total) :
    slot / c < chunkCount c total := by
  unfold chunkCount
  apply (Nat.le_div_iff_mul_le hc).mpr
  have hle : slot / c * c ≤ slot := Nat.div_mul_le_self slot c
  show (slot / c + 1) * c ≤ total + c - 1
  rw [Nat.add_mul, Nat.one_mul]
  omega

/-- Complete coverage and disjointness: every valid slot lies in exactly one
    chunk of the fixed partition. -/
theorem fixed_partition_covers_each_slot_once (c total slot : Nat)
    (hc : 0 < c) (hs : slot < total) :
    (∃ r ∈ fixedPartition c total, r.contains slot) ∧
    (∀ k₁ k₂, (fixedChunk c total k₁).contains slot →
      (fixedChunk c total k₂).contains slot → k₁ = k₂) := by
  refine ⟨⟨fixedChunk c total (slot / c), ?_, slot_in_own_chunk c total slot hc hs⟩, ?_⟩
  · exact List.mem_map.mpr
      ⟨slot / c, List.mem_range.mpr (own_chunk_in_partition c total slot hc hs), rfl⟩
  · intro k₁ k₂ h₁ h₂
    rw [chunk_of_slot_unique c total slot k₁ h₁, chunk_of_slot_unique c total slot k₂ h₂]

/-! ## Consolidation-invariant semantic commitment -/

/-- One inline history row of a layer: a slot and its versions in that layer. -/
abbrev Row (V : Type) := Nat × List V

/-- A chunk's physical members. `historyLocator` and `headLocator` are the
    binding's content addresses; `spilled slot` is the slot's continuation
    pages in sequence order. -/
structure PhysicalChunk (V : Type) where
  slots : List Nat
  headLocator : Nat
  historyLocator : Nat
  layers : List (List (Row V))
  spilled : Nat → List V

/-- Inline history of `slot`: its rows concatenated in layer order. -/
def inlineHistory {V : Type} (layers : List (List (Row V))) (slot : Nat) : List V :=
  (layers.map fun layer => (layer.filter fun row => row.1 == slot).flatMap Prod.snd).flatten

/-- The canonical visit: per slot, spilled prefix then inline tail. -/
def canonical {V : Type} (p : PhysicalChunk V) : List (Nat × List V) :=
  p.slots.map fun slot => (slot, p.spilled slot ++ inlineHistory p.layers slot)

/-- `consolidate_history_chunk`: one flat layer in a new history member. -/
def consolidate {V : Type} (p : PhysicalChunk V) (newLocator : Nat) : PhysicalChunk V :=
  { p with layers := [p.layers.flatten], historyLocator := newLocator }

theorem inlineHistory_flatten {V : Type} (layers : List (List (Row V))) (slot : Nat) :
    inlineHistory [layers.flatten] slot = inlineHistory layers slot := by
  induction layers with
  | nil => simp [inlineHistory]
  | cons layer rest ih =>
      simp only [inlineHistory, List.map_cons, List.map_nil, List.flatten_cons,
        List.flatten_nil, List.append_nil, List.filter_append, List.flatMap_append] at ih ⊢
      rw [ih]

/-- Consolidation preserves canonical meaning. -/
theorem consolidate_preserves_canonical {V : Type} (p : PhysicalChunk V) (newLocator : Nat) :
    canonical (consolidate p newLocator) = canonical p := by
  simp only [canonical, consolidate]
  congr 1
  funext slot
  rw [inlineHistory_flatten]

/-- The semantic commitment of a chunk under any hash of its canonical
    meaning is bit-identical across consolidation, whatever the new locator. -/
theorem semantic_commitment_consolidation_invariant {V α : Type}
    (hash : List (Nat × List V) → α) (p : PhysicalChunk V) (newLocator : Nat) :
    hash (canonical (consolidate p newLocator)) = hash (canonical p) := by
  rw [consolidate_preserves_canonical]

/-- The semantic root folds only per-chunk commitments; consolidating any
    chunks into any new history members leaves the root unchanged. -/
theorem semantic_root_consolidation_invariant {V α β : Type}
    (hash : List (Nat × List V) → α) (root : List α → β)
    (chunks : List (PhysicalChunk V)) (newLocator : PhysicalChunk V → Nat) :
    root ((chunks.map fun p => consolidate p (newLocator p)).map (hash ∘ canonical)) =
      root (chunks.map (hash ∘ canonical)) := by
  congr 1
  simp only [List.map_map]
  apply List.map_congr_left
  intro p _
  simp only [Function.comp, consolidate_preserves_canonical]

/-! ## Producer-derived summary completeness -/

/-- A sealed chunk's canonical records and its summary. `keys r` is what a
    record contributes to the summary: its kind, or its incident relation kinds. -/
structure SummarizedChunk (R K : Type) where
  records : List R
  summary : List K

/-- The derivation `seal_chunk_image` performs over the records it hashes. -/
def derive {R K : Type} (keys : R → List K) (records : List R) : List K :=
  records.flatMap keys

def matching {R K : Type} [DecidableEq K] (keys : R → List K) (k : K) (records : List R) :
    List R :=
  records.filter fun r => decide (k ∈ keys r)

/-- Canonical full scan: every chunk is read. -/
def fullScan {R K : Type} [DecidableEq K] (keys : R → List K) (k : K)
    (chunks : List (SummarizedChunk R K)) : List R :=
  chunks.flatMap fun ch => matching keys k ch.records

/-- Summary-pruned scan: a chunk is read only if its summary may contain `k`. -/
def prunedScan {R K : Type} [DecidableEq K] (keys : R → List K) (k : K)
    (chunks : List (SummarizedChunk R K)) : List R :=
  (chunks.filter fun ch => decide (k ∈ ch.summary)).flatMap fun ch => matching keys k ch.records

theorem matching_nil_of_not_mem_derive {R K : Type} [DecidableEq K]
    (keys : R → List K) (k : K) (records : List R) (h : k ∉ derive keys records) :
    matching keys k records = [] := by
  apply List.filter_eq_nil_iff.mpr
  intro r hr hk
  exact h (List.mem_flatMap.mpr ⟨r, hr, of_decide_eq_true hk⟩)

/-- Summaries derived by the producer make pruning exact: the pruned scan
    returns identically what the canonical full scan returns. -/
theorem pruned_scan_agrees_with_full_scan {R K : Type} [DecidableEq K]
    (keys : R → List K) (k : K) (chunks : List (SummarizedChunk R K))
    (hDerived : ∀ ch ∈ chunks, ch.summary = derive keys ch.records) :
    prunedScan keys k chunks = fullScan keys k chunks := by
  induction chunks with
  | nil => rfl
  | cons ch rest ih =>
      have hRest := ih fun c hc => hDerived c (List.mem_cons_of_mem _ hc)
      have hCh := hDerived ch List.mem_cons_self
      simp only [prunedScan, fullScan] at hRest ⊢
      by_cases hk : k ∈ ch.summary
      · simp [hk, hRest]
      · have hNil : matching keys k ch.records = [] :=
          matching_nil_of_not_mem_derive keys k ch.records (hCh ▸ hk)
        simp [hk, hNil, hRest]

/-- Mutant `M3`: a summary omitting a key that a covered record carries
    violates the producer derivation invariant. -/
theorem omitted_key_violates_derivation {R K : Type}
    (keys : R → List K) (k : K) (ch : SummarizedChunk R K)
    (hRecord : ∃ r ∈ ch.records, k ∈ keys r) (hOmitted : k ∉ ch.summary) :
    ch.summary ≠ derive keys ch.records := by
  intro hEq
  rcases hRecord with ⟨r, hr, hk⟩
  exact hOmitted (hEq ▸ List.mem_flatMap.mpr ⟨r, hr, hk⟩)

/-- Mutant `M3`, observable form: with such an omission the pruned scan
    loses the record the canonical full scan returns. -/
theorem omitted_key_breaks_pruned_scan {R K : Type} [DecidableEq K]
    (keys : R → List K) (k : K) (ch : SummarizedChunk R K)
    (hRecord : ∃ r ∈ ch.records, k ∈ keys r) (hOmitted : k ∉ ch.summary) :
    prunedScan keys k [ch] ≠ fullScan keys k [ch] := by
  rcases hRecord with ⟨r, hr, hk⟩
  have hFull : r ∈ fullScan keys k [ch] := by
    simp only [fullScan, List.flatMap_cons, List.flatMap_nil, List.append_nil]
    exact List.mem_filter.mpr ⟨hr, decide_eq_true hk⟩
  intro hEq
  simp [prunedScan, hOmitted] at hEq
  rw [hEq] at hFull
  exact List.not_mem_nil hFull

/-! ## `ChunkedCoverageAndSummarySound` -/

/-- Milestone-4 §9.1 row 5: fixed non-overlapping coverage, consolidation
    invariance of the semantic commitment, and exact summary pruning. -/
theorem ChunkedCoverageAndSummarySound :
    (∀ c total slot, 0 < c → slot < total →
      (∃ r ∈ fixedPartition c total, r.contains slot) ∧
      (∀ k₁ k₂, (fixedChunk c total k₁).contains slot →
        (fixedChunk c total k₂).contains slot → k₁ = k₂)) ∧
    (∀ (V α : Type) (hash : List (Nat × List V) → α) (p : PhysicalChunk V) (newLocator : Nat),
      hash (canonical (consolidate p newLocator)) = hash (canonical p)) ∧
    (∀ (R K : Type) [DecidableEq K] (keys : R → List K) (k : K)
      (chunks : List (SummarizedChunk R K)),
      (∀ ch ∈ chunks, ch.summary = derive keys ch.records) →
      prunedScan keys k chunks = fullScan keys k chunks) :=
  ⟨fun c total slot hc hs => fixed_partition_covers_each_slot_once c total slot hc hs,
   fun _ _ hash p newLocator => semantic_commitment_consolidation_invariant hash p newLocator,
   fun _ _ _ keys k chunks hDerived => pruned_scan_agrees_with_full_scan keys k chunks hDerived⟩

end StoreM4.ChunkedCoverage
