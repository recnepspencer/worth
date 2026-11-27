-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/authority/posture_validation.rs:34-287 sha256:e2ead63a5134a298529f834ebf9ebf216d4164fb3722dda428455a07e594f993
-- proof-mirror: crates/worth-relational-lineage/src/lineage/authority/name_minting.rs:87-208 sha256:4459ab2ab45f28c435da02fbc021c8a53e65cb5c809f5d7e250effb781796dfb

/-!
# Commit naming step bound independent of history (Phase 7B-2, RULES §4)

Structural cost model of `posture_validation::validate_commit_postures`
and `name_minting::mint_commit_births_with_basis`: each loop iteration and
each point lookup into the branch basis (live holders, retired names) is
one step. The basis is an arbitrary lookup oracle of any size `N`: the
model lets it answer lookups, never drive iteration, exactly as the Rust
code reads it only through `LineageBasisView` point lookups.

`commit_naming_steps_bounded` proves the steps are at most
`3·|changed records| + Σ_claims (1 + fan-out + lookups) + mint-order steps
+ Σ_births (1 + parents + references + discriminator bytes)`, and
`commit_naming_steps_basis_independent` that two bases of any sizes give
the same count: no term grows with the branch's historical name count.

A structural cost model with no executable conformance trace: its steps are
not observable from the Rust without production counters. The
`proof-mirror:` pins bind it to the two functions it describes.
-/
namespace PersistentNaming.PostureCost

/-- A branch naming basis of any size, read only by point lookups. -/
structure Basis where
  live : Nat → Bool
  retired : Nat → Bool

/-- One posture claim: the records it fans out to (split children, merge
parents) and the names it looks up in the basis. -/
structure Claim where
  fanout : Nat
  lookups : List Nat

/-- One birth: parents, discriminator references, discriminator bytes. -/
structure Birth where
  parents : List Nat
  references : List Nat
  discBytes : Nat

/-- Point lookups: one step each, whatever the basis answers. -/
def lookupSteps (basis : Basis) : List Nat → Nat × Bool
  | [] => (0, true)
  | name :: rest =>
    let (steps, ok) := lookupSteps basis rest
    (steps + 1, ok && (basis.live name || basis.retired name))

/-- Validating one claim: its own step, its fan-out, its lookups. -/
def claimSteps (basis : Basis) (claim : Claim) : Nat :=
  1 + claim.fanout + (lookupSteps basis claim.lookups).1

/-- Minting one birth: its step, the parent and reference lookups, and the
discriminator scan. -/
def birthSteps (basis : Basis) (birth : Birth) : Nat :=
  1 + (lookupSteps basis birth.parents).1 + (lookupSteps basis birth.references).1
    + birth.discBytes

def sumMap {α : Type} (f : α → Nat) : List α → Nat
  | [] => 0
  | a :: rest => f a + sumMap f rest

/-- One commit's naming: the changed-record set, the created filter and
the missing-birth check (each over the changed records), every claim,
the mint order (bounded in `MintOrder`), and every birth. -/
def commitSteps (basis : Basis) (changed : Nat) (claims : List Claim)
    (mintOrderSteps : Nat) (births : List Birth) : Nat :=
  3 * changed + sumMap (claimSteps basis) claims + mintOrderSteps
    + sumMap (birthSteps basis) births

theorem lookupSteps_count (basis : Basis) : ∀ names : List Nat,
    (lookupSteps basis names).1 = names.length
  | [] => rfl
  | _ :: rest => by simp [lookupSteps, lookupSteps_count basis rest]

theorem commit_naming_steps_bounded (basis : Basis) (changed : Nat) (claims : List Claim)
    (mintOrderSteps : Nat) (births : List Birth) :
    commitSteps basis changed claims mintOrderSteps births =
      3 * changed + sumMap (fun c => 1 + c.fanout + c.lookups.length) claims + mintOrderSteps
        + sumMap (fun b => 1 + b.parents.length + b.references.length + b.discBytes) births := by
  have hc : ∀ cs : List Claim, sumMap (claimSteps basis) cs =
      sumMap (fun c => 1 + c.fanout + c.lookups.length) cs := by
    intro cs
    induction cs with
    | nil => rfl
    | cons c rest ih => simp [sumMap, claimSteps, lookupSteps_count, ih]
  have hb : ∀ bs : List Birth, sumMap (birthSteps basis) bs =
      sumMap (fun b => 1 + b.parents.length + b.references.length + b.discBytes) bs := by
    intro bs
    induction bs with
    | nil => rfl
    | cons b rest ih => simp [sumMap, birthSteps, lookupSteps_count, ih]
  simp only [commitSteps, hc, hb]

/-- Two bases of any sizes give one commit the same step count. -/
theorem commit_naming_steps_basis_independent (small large : Basis) (changed : Nat)
    (claims : List Claim) (mintOrderSteps : Nat) (births : List Birth) :
    commitSteps small changed claims mintOrderSteps births =
      commitSteps large changed claims mintOrderSteps births := by
  rw [commit_naming_steps_bounded, commit_naming_steps_bounded]

end PersistentNaming.PostureCost
