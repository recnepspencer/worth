-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/mod.rs:21-53 sha256:86b58650ad9fca659eff97c17dda6a2f81c2e2d5f62045606245badfdb08826b
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/mod.rs:166-193 sha256:f6c5d79b599f81fddb14f2ec5d1f443dc83bebed0ba7807968ffc187fc898bdb
-- proof-mirror: workspaces/worth-store/crates/worth-store/src/pin_table_kernel/transitions.rs:282-328 sha256:679f25b271207da4c2a799036acaf6dd7d00cbad18b9667963f75691372d95d7

namespace StoreM4.PinSafety

/-- Lifecycle state of a durable whole-Home root pin lease (Milestone 4 §3.2 / §4.1). -/
inductive LeaseState where
  | active
  | expiring
  | expired
  deriving DecidableEq, Repr

/-- Pure mathematical model of one entry in `PinTable`. -/
structure PinEntry where
  id : Nat
  root : Nat
  parent : Option Nat
  state : LeaseState
  activeReaderLoans : Nat
  liveChildPins : Nat
  deriving DecidableEq, Repr

/-- Any non-expired pin (`Active` or `Expiring`) protects its pinned root from reclamation. -/
def PinEntry.protectsRoot (e : PinEntry) : Prop :=
  e.state ≠ LeaseState.expired

/-- Pure model of a durable `PinTable`. -/
abbrev PinTable := List PinEntry

/-- Root reachability closure contributed by live (`Active` or `Expiring`) pin entries. -/
def PinTable.reachableRoot (t : PinTable) (r : Nat) : Prop :=
  ∃ e ∈ t, e.root = r ∧ e.protectsRoot

/-- Acquire a fresh `Active` pin on `root` with optional `parent` overlay link. -/
def PinTable.acquire (t : PinTable) (id root : Nat) (parent : Option Nat) : PinTable :=
  { id := id
    root := root
    parent := parent
    state := LeaseState.active
    activeReaderLoans := 0
    liveChildPins := 0 } :: t

/-- Single lease-expiry step matching `Active -> Expiring -> Expired` (Milestone 4 §4.1). -/
def PinEntry.expireStep (e : PinEntry) : PinEntry :=
  match e.state with
  | LeaseState.active => { e with state := LeaseState.expiring }
  | LeaseState.expiring =>
      if e.activeReaderLoans = 0 ∧ e.liveChildPins = 0 then
        { e with state := LeaseState.expired }
      else
        e
  | LeaseState.expired => e

/-- Apply an expiry step to the pin entry identified by `targetId`. -/
def PinTable.expireById (t : PinTable) (targetId : Nat) : PinTable :=
  t.map (fun e => if e.id = targetId then e.expireStep else e)

/-- M4.0/M4.1 smoke theorem 1: every non-expired pin entry makes its root reachable. -/
theorem active_pin_protects_root
    (t : PinTable) (e : PinEntry)
    (hMem : e ∈ t) (hProt : e.protectsRoot) :
    PinTable.reachableRoot t e.root :=
  ⟨e, hMem, rfl, hProt⟩

/-- M4.0/M4.1 smoke theorem 2: acquiring a pin protects the new root and preserves all prior pins. -/
theorem acquire_protects_new_and_existing_roots
    (t : PinTable) (id root r : Nat) (parent : Option Nat)
    (h : r = root ∨ PinTable.reachableRoot t r) :
    PinTable.reachableRoot (PinTable.acquire t id root parent) r := by
  cases h with
  | inl hEq =>
      subst hEq
      refine ⟨{ id := id, root := r, parent := parent, state := LeaseState.active,
                activeReaderLoans := 0, liveChildPins := 0 }, ?_, rfl, ?_⟩
      · exact List.mem_cons_self
      · intro hAbsurd
        cases hAbsurd
  | inr hOld =>
      rcases hOld with ⟨e, hMem, hRoot, hProt⟩
      exact ⟨e, List.mem_cons_of_mem _ hMem, hRoot, hProt⟩

/-- M4.0/M4.1 smoke theorem 3 (`LeaseExpiryNoReaderLoss` shape):
    an expiry step never drops root protection while reader loans or child pins remain active. -/
theorem expireStep_retains_protection_while_held
    (e : PinEntry)
    (hProt : e.protectsRoot)
    (hHeld : e.activeReaderLoans > 0 ∨ e.liveChildPins > 0) :
    e.expireStep.protectsRoot := by
  unfold PinEntry.expireStep PinEntry.protectsRoot at *
  cases hState : e.state with
  | active =>
      intro hEq
      cases hEq
  | expiring =>
      have hNotZero : ¬ (e.activeReaderLoans = 0 ∧ e.liveChildPins = 0) := by
        intro hZero
        rcases hHeld with hLoans | hChildren
        · omega
        · omega
      simp [hNotZero, hState]
  | expired =>
      exact False.elim (hProt hState)

/-- M4.0/M4.1 smoke theorem 4: expiring a pin by id keeps its root reachable while held. -/
theorem expireById_preserves_held_reachability
    (t : PinTable) (targetId : Nat) (e : PinEntry)
    (hMem : e ∈ t)
    (hProt : e.protectsRoot)
    (hHeld : e.id = targetId → e.activeReaderLoans > 0 ∨ e.liveChildPins > 0) :
    PinTable.reachableRoot (PinTable.expireById t targetId) e.root := by
  by_cases hId : e.id = targetId
  · refine ⟨e.expireStep, ?_, ?_, expireStep_retains_protection_while_held e hProt (hHeld hId)⟩
    · unfold PinTable.expireById
      have hMap := List.mem_map_of_mem (f := fun x => if x.id = targetId then x.expireStep else x) hMem
      simpa [hId] using hMap
    · unfold PinEntry.expireStep
      cases e.state with
      | active => rfl
      | expiring =>
          by_cases hZero : e.activeReaderLoans = 0 ∧ e.liveChildPins = 0
          · simp [hZero]
          · simp [hZero]
      | expired => rfl
  · refine ⟨e, ?_, rfl, hProt⟩
    unfold PinTable.expireById
    have hMap := List.mem_map_of_mem (f := fun x => if x.id = targetId then x.expireStep else x) hMem
    simpa [hId] using hMap

/-- M4.0/M4.1 smoke theorem 5 (`StoreGCPinSafety` shape):
    any reclamation set disjoint from `reachableRoot` never reclaims a protected pin's root. -/
theorem reclaim_disjoint_preserves_active_pins
    (t : PinTable) (reclaimSet : Nat → Prop) (e : PinEntry)
    (hDisjoint : ∀ r, reclaimSet r → ¬ PinTable.reachableRoot t r)
    (hMem : e ∈ t)
    (hProt : e.protectsRoot) :
    ¬ reclaimSet e.root := by
  intro hReclaim
  exact hDisjoint e.root hReclaim (active_pin_protects_root t e hMem hProt)

end StoreM4.PinSafety
