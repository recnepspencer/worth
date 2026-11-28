-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: workspaces/worth-store/crates/worth-store-physical-format/src/arena_frame/range.rs:52-56 sha256:7f3b7fb5f9e7c1bf57fe72868887494054aaa3c3eef85508086ffb435b42343f
-- proof-mirror: workspaces/worth-store/crates/worth-store-physical-format/src/arena_frame/allocation_generation.rs:11-49 sha256:0302471ceb8716aaa223bc3414b8834cf99c9418d360493661e01b5d51d7b3e1
-- proof-mirror: workspaces/worth-store/crates/worth-store-buffer-pool/src/allocation_generation_guard/mod.rs:20-50 sha256:0effd1cdee4ea30faadbb8588f82f92cb79b5aac9550b92a88f15c6d035b7fde
-- proof-mirror: workspaces/worth-store/crates/worth-store-buffer-pool/src/allocation_generation_guard/mod.rs:87-245 sha256:121eb23dc927a19d9735efcb945dc376d6f7a9519e7fc3e7916e5579557adafb
-- proof-mirror: workspaces/worth-store/crates/worth-store-buffer-pool/src/physical_residency/pool/arena_generation_ledger.rs:39-148 sha256:fa02f89c2f7e3e227bb6c9f2a4ed4d746225eecc42264cb450ed14d78a521680
-- proof-mirror: workspaces/worth-store/crates/worth-store-buffer-pool/src/physical_residency/pool/writeback_fence_drain.rs:24-120 sha256:7382c880b3a0a6a0fea758824aa6af5347d95a1b9ebc96e5ba90b96d4770bf0c
-- proof-mirror: workspaces/worth-store/crates/worth-store-buffer-pool/src/physical_residency/pool/writeback_fence_drain.rs:207-265 sha256:b7ff11a7b7444b1904b31e81ceea5d34de8f2b45b403be6aca7075f90da5eb8f

/-!
# `AddressReuseGenerationAndWriteFenceSound` (Store Milestone 4, Phase M4.5)

Model of extent-arena reuse in `worth-store` and `worth-store-buffer-pool`:

* `ArenaByteRange` is `ExtentArenaRange`; `Range.overlaps` is
  `ExtentArenaRange::overlaps` / the Verus kernel's `ranges_overlap` over
  arbitrary exact, split and coalesced sub/super-ranges.
* `Gen` is `ExtentAllocationGeneration`, ordered lexicographically by
  `(root_publication_generation, extent_fence_seq)`.
* `Step.admitRange` is `PhysicalResidencyPool::drain_writeback_fence` composed with
  allocator admission. Its generation is read from durable state: the current
  root generation and the WAL sequence of the reuse barrier's own durable
  record (retirement completion for Path A, the candidate cancellation for
  Path B). The barrier record and admission are one step here: every
  admission is preceded by its own fresh barrier record.
* `Step.crash` drops every volatile set (ledger, in-flight writebacks, resident
  frames) and keeps the durable root generation and WAL sequence.

The theorems hold for every interleaving of writeback issue/land, frame
admission, root publication, barrier records, admissions and crashes,
including candidate cancellations with no root publication at all.
-/

namespace StoreM4.ExtentGenerationSafety

/-- One arena byte range `[offset, offset + length)`. -/
structure Range where
  arena : Nat
  offset : Nat
  length : Nat
  deriving DecidableEq, Repr

def Range.overlaps (a b : Range) : Prop :=
  a.arena = b.arena ∧ a.offset < b.offset + b.length ∧ b.offset < a.offset + a.length

instance (a b : Range) : Decidable (a.overlaps b) := by
  unfold Range.overlaps; infer_instance

theorem Range.overlaps_symm {a b : Range} (h : a.overlaps b) : b.overlaps a :=
  ⟨h.1.symm, h.2.2, h.2.1⟩

/-- `(root_publication_generation, extent_fence_seq)`. -/
structure Gen where
  root : Nat
  seq : Nat
  deriving DecidableEq, Repr

def Gen.lt (a b : Gen) : Prop :=
  a.root < b.root ∨ (a.root = b.root ∧ a.seq < b.seq)

theorem Gen.lt_asymm {a b : Gen} (h : a.lt b) : ¬ b.lt a := by
  unfold Gen.lt at *; omega

theorem Gen.lt_ne {a b : Gen} (h : a.lt b) : a ≠ b := by
  intro e; subst e; unfold Gen.lt at h; omega

/-- `ArenaExtentFrameIdentity`. -/
structure Ident where
  range : Range
  gen : Gen
  deriving DecidableEq, Repr

/-- The kernel's `frame_lookup_matches`. -/
def lookupMatches (frame : Ident) (r : Range) (g : Gen) : Prop :=
  frame.range = r ∧ frame.gen = g

structure Sys where
  /-- Volatile `ArenaGenerationLedger` entries. -/
  seen : List Ident
  /-- Volatile in-flight writeback claims. -/
  inflight : List Ident
  /-- Volatile resident frames. -/
  frames : List Ident
  /-- Durable root publication generation. -/
  rootGen : Nat
  /-- Durable WAL sequence. -/
  walSeq : Nat
  /-- Durable reopen floor seeded on crash/reopen so frames never rewind. -/
  floorGen : Gen
  /-- Ghost: every generation ever admitted, across crashes. -/
  history : List Gen

/-- An identity at or above the reopen floor and superseded by no overlapping
admitted generation. -/
def current (s : Sys) (i : Ident) : Prop :=
  ¬ i.gen.lt s.floorGen ∧
    ∀ e ∈ s.seen, e.range.overlaps i.range → ¬ i.gen.lt e.gen

inductive Barrier where
  | retirementIncomplete
  | completedRetirement
  | cancelUncovered
  | checkpointCoveredCancel

def Barrier.admits : Barrier → Prop
  | .completedRetirement => True
  | .checkpointCoveredCancel => True
  | _ => False

/-- The generation the next admission's own durable barrier record names. -/
def Sys.nextGen (s : Sys) : Gen := ⟨s.rootGen, s.walSeq + 1⟩

/-- `WritebackFenceDrain` + `AdmitToAllocator`, after its fresh barrier record. -/
def Sys.admitRange (s : Sys) (r : Range) : Sys :=
  { s with
    seen := ⟨r, s.nextGen⟩ :: s.seen
    frames := s.frames.filter (fun f => !decide (f.range.overlaps r))
    walSeq := s.walSeq + 1
    history := s.nextGen :: s.history }

inductive Step : Sys → Sys → Prop
  | issue (s : Sys) (i : Ident) : current s i → Step s { s with inflight := i :: s.inflight }
  | land (s : Sys) (i : Ident) : i ∈ s.inflight → Step s { s with inflight := s.inflight.erase i }
  | load (s : Sys) (i : Ident) : current s i → Step s { s with frames := i :: s.frames }
  | publishRoot (s : Sys) : Step s { s with rootGen := s.rootGen + 1 }
  | barrier (s : Sys) : Step s { s with walSeq := s.walSeq + 1 }
  | admitRange (s : Sys) (r : Range) (b : Barrier) :
      b.admits →
      (∀ w ∈ s.inflight, ¬ r.overlaps w.range) →
      (∀ e ∈ s.seen, e.range.overlaps r → e.gen.lt s.nextGen) →
      Step s (s.admitRange r)
  | crash (s : Sys) :
      Step s { s with seen := [], inflight := [], frames := [], floorGen := ⟨s.rootGen, s.walSeq⟩ }

def init : Sys := ⟨[], [], [], 0, 0, ⟨0, 0⟩, []⟩

inductive Reachable : Sys → Prop
  | init : Reachable init
  | step {s s' : Sys} : Reachable s → Step s s' → Reachable s'

/-- Every in-flight writeback and resident frame is at or above the reopen floor
and its range's current generation, the reopen floor and every admitted
generation lie below the durable frontier, and the ledger only names admitted
generations. -/
structure Inv (s : Sys) : Prop where
  inflight_current : ∀ w ∈ s.inflight, current s w
  frames_current : ∀ f ∈ s.frames, current s f
  floor_below : s.floorGen.lt s.nextGen
  history_below : ∀ h ∈ s.history, h.lt s.nextGen
  seen_admitted : ∀ e ∈ s.seen, e.gen ∈ s.history

theorem nextGen_advances_root (s : Sys) (h : Gen) (hl : h.lt s.nextGen) :
    h.lt ({ s with rootGen := s.rootGen + 1 } : Sys).nextGen := by
  unfold Gen.lt Sys.nextGen at *; simp only at *; omega

theorem nextGen_advances_seq (s : Sys) (h : Gen) (hl : h.lt s.nextGen) :
    h.lt ({ s with walSeq := s.walSeq + 1 } : Sys).nextGen := by
  unfold Gen.lt Sys.nextGen at *; simp only at *; omega

theorem admit_current_preserved (s : Sys) (r : Range) (i : Ident)
    (hc : current s i) (hd : ¬ r.overlaps i.range) : current (s.admitRange r) i := by
  refine ⟨hc.1, ?_⟩
  intro e he ho
  simp only [Sys.admitRange, List.mem_cons] at he
  rcases he with rfl | he
  · exact absurd ho hd
  · exact hc.2 e he ho

theorem init_inv : Inv init :=
  ⟨by simp [init], by simp [init], by simp [init, Gen.lt, Sys.nextGen],
   by simp [init], by simp [init]⟩

theorem step_inv {s s' : Sys} (hi : Inv s) (hs : Step s s') : Inv s' := by
  cases hs with
  | issue i hc =>
    refine ⟨?_, hi.frames_current, hi.floor_below, hi.history_below, hi.seen_admitted⟩
    intro w hw
    simp only [List.mem_cons] at hw
    rcases hw with rfl | hw
    · exact hc
    · exact hi.inflight_current w hw
  | land i _ =>
    exact ⟨fun w hw => hi.inflight_current w (List.mem_of_mem_erase hw),
      hi.frames_current, hi.floor_below, hi.history_below, hi.seen_admitted⟩
  | load i hc =>
    refine ⟨hi.inflight_current, ?_, hi.floor_below, hi.history_below, hi.seen_admitted⟩
    intro f hf
    simp only [List.mem_cons] at hf
    rcases hf with rfl | hf
    · exact hc
    · exact hi.frames_current f hf
  | publishRoot =>
    exact ⟨hi.inflight_current, hi.frames_current,
      nextGen_advances_root s s.floorGen hi.floor_below,
      fun h hh => nextGen_advances_root s h (hi.history_below h hh), hi.seen_admitted⟩
  | barrier =>
    exact ⟨hi.inflight_current, hi.frames_current,
      nextGen_advances_seq s s.floorGen hi.floor_below,
      fun h hh => nextGen_advances_seq s h (hi.history_below h hh), hi.seen_admitted⟩
  | admitRange r _ _ hdrain _ =>
    refine ⟨?_, ?_, ?_, ?_, ?_⟩
    · intro w hw
      exact admit_current_preserved s r w (hi.inflight_current w hw) (hdrain w hw)
    · intro f hf
      simp only [Sys.admitRange, List.mem_filter, Bool.not_eq_true', decide_eq_false_iff_not] at hf
      exact admit_current_preserved s r f (hi.frames_current f hf.1)
        (fun ho => hf.2 (Range.overlaps_symm ho))
    · have hf := hi.floor_below
      simp only [Gen.lt, Sys.nextGen, Sys.admitRange] at hf ⊢
      omega
    · intro h hh
      simp only [Sys.admitRange, List.mem_cons] at hh
      rcases hh with rfl | hh
      · simp [Gen.lt, Sys.nextGen, Sys.admitRange]
      · have := hi.history_below h hh
        simp only [Gen.lt, Sys.nextGen, Sys.admitRange] at this ⊢
        omega
    · intro e he
      simp only [Sys.admitRange, List.mem_cons] at he ⊢
      rcases he with rfl | he
      · exact Or.inl rfl
      · exact Or.inr (hi.seen_admitted e he)
  | crash =>
    exact ⟨by simp, by simp, by simp [Gen.lt, Sys.nextGen], hi.history_below, by simp⟩

theorem Reachable.inv {s : Sys} (h : Reachable s) : Inv s := by
  induction h with
  | init => exact init_inv
  | step _ hs ih => exact step_inv ih hs

/-- The guard's generation premise is implied by the durable frontier: the
ledger's generations were all admitted, hence all lie below `nextGen`. -/
theorem guard_generation_holds {s : Sys} (h : Reachable s) (r : Range) :
    ∀ e ∈ s.seen, e.range.overlaps r → e.gen.lt s.nextGen :=
  fun e he _ => h.inv.history_below e.gen (h.inv.seen_admitted e he)

/-- Immediately after a crash/reopen, the seeded durable floor dominates every
previously admitted generation, so no newly loaded frame or issued writeback
can rewind below any pre-crash barrier generation. -/
theorem post_crash_frame_never_rewinds_below_history {s : Sys} (h : Reachable s) (i : Ident)
    (hc : current { s with seen := [], inflight := [], frames := [], floorGen := ⟨s.rootGen, s.walSeq⟩ } i) :
    ∀ g ∈ s.history, ¬ i.gen.lt g := by
  intro g hg hlt
  have hbelow := h.inv.history_below g hg
  have hfloor := hc.1
  simp only [Gen.lt, Sys.nextGen] at hbelow hfloor hlt
  omega

/-- **`AddressReuseGenerationAndWriteFenceSound`.** In every reachable state,
across arbitrary split/coalesced ranges, candidate cancellations without root
publication and crashes:

1. no in-flight writeback is below the reopen floor or superseded by a newer
   overlapping generation, so landing it never clobbers a reused range;
2. no resident frame is below the reopen floor or superseded, so a lookup never
   serves a stale generation's bytes for a reused range;
3. every admission's generation strictly exceeds every generation ever
   admitted before it, so a stale `(range, g)` lookup or submission can never
   match the reused identity. -/
theorem AddressReuseGenerationAndWriteFenceSound :
    ∀ s : Sys, Reachable s →
      (∀ w ∈ s.inflight, ¬ w.gen.lt s.floorGen ∧
        ∀ e ∈ s.seen, e.range.overlaps w.range → ¬ w.gen.lt e.gen) ∧
      (∀ f ∈ s.frames, ¬ f.gen.lt s.floorGen ∧
        ∀ e ∈ s.seen, e.range.overlaps f.range → ¬ f.gen.lt e.gen) ∧
      (∀ (r : Range), ∀ h ∈ s.history,
        h.lt s.nextGen ∧ ¬ lookupMatches ⟨r, s.nextGen⟩ r h) := by
  intro s hs
  have hi := hs.inv
  refine ⟨hi.inflight_current, hi.frames_current, ?_⟩
  intro r h hh
  have hlt := hi.history_below h hh
  exact ⟨hlt, fun hm => Gen.lt_ne hlt hm.2.symm⟩

/-! ## `M10` negative witnesses -/

/-- Admission with the drain premise removed. -/
def unfencedAdmitWitness : Sys := ⟨[], [⟨⟨1, 0, 8⟩, ⟨0, 0⟩⟩], [], 0, 0, ⟨0, 0⟩, []⟩

/-- M10a: removing `WritebackFenceDrain` lets a paused writeback survive a
split-range reuse; landing it would clobber the newer generation. -/
theorem M10_without_drain_paused_writeback_clobbers :
    Inv unfencedAdmitWitness ∧
      ∃ w ∈ (unfencedAdmitWitness.admitRange ⟨1, 4, 2⟩).inflight,
        ¬ current (unfencedAdmitWitness.admitRange ⟨1, 4, 2⟩) w := by
  refine ⟨⟨?_, by simp [unfencedAdmitWitness], by simp [unfencedAdmitWitness, Gen.lt, Sys.nextGen],
    ?_, by simp [unfencedAdmitWitness]⟩, ?_⟩
  · intro w hw
    simp only [unfencedAdmitWitness, List.mem_singleton] at hw
    subst hw
    exact ⟨by simp [Gen.lt, unfencedAdmitWitness], fun e he => by simp [unfencedAdmitWitness] at he⟩
  · intro h hh; simp [unfencedAdmitWitness] at hh
  · refine ⟨⟨⟨1, 0, 8⟩, ⟨0, 0⟩⟩, by simp [unfencedAdmitWitness, Sys.admitRange], ?_⟩
    intro hc
    have := hc.2 ⟨⟨1, 4, 2⟩, ⟨0, 1⟩⟩ (by simp [Sys.admitRange, Sys.nextGen, unfencedAdmitWitness])
      ⟨rfl, by decide, by decide⟩
    exact this (Or.inr ⟨rfl, by decide⟩)

/-- Admission that reuses the barrier's sequence without advancing it, as a
cancelled candidate would without its own fence record. -/
def Sys.admitNoAdvance (s : Sys) (r : Range) : Sys :=
  { s with seen := ⟨r, ⟨s.rootGen, s.walSeq⟩⟩ :: s.seen,
           history := ⟨s.rootGen, s.walSeq⟩ :: s.history }

/-- M10b: without the cancel-epoch advance, cancel → crash → reuse with no
root publication reissues the same identity, so a stale lookup or writeback
naming the cancelled generation matches the live one. -/
theorem M10_without_cancel_epoch_advance_aliases :
    let first := init.admitNoAdvance ⟨1, 0, 8⟩
    let reopened : Sys := { first with seen := [], inflight := [], frames := [] }
    let reused := reopened.admitNoAdvance ⟨1, 0, 8⟩
    reused.rootGen = init.rootGen ∧
      ∃ stale ∈ first.history,
        ∃ live ∈ reused.seen, lookupMatches live ⟨1, 0, 8⟩ stale := by
  refine ⟨rfl, ⟨0, 0⟩, by simp [Sys.admitNoAdvance, init], ⟨⟨1, 0, 8⟩, ⟨0, 0⟩⟩,
    by simp [Sys.admitNoAdvance, init], rfl, rfl⟩

end StoreM4.ExtentGenerationSafety
