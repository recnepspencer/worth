-- proof-mirror: crates/worth-relational-contracts/src/durability/canonical_commit_artifact/frame.rs:140-163 sha256:61b0088053bb14ff48287ae15529d1abc4829959965be66a47be9d03017125cc
-- proof-mirror: crates/worth-relational-durability/src/durability/log/native_checkpoint_codec.rs:25-52 sha256:594e87d1187a26633c7aee52a2aa58e83cc6379be2f8dfa91314b4c243df496b
-- proof-mirror: crates/worth-relational-durability/src/durability/canonical_commit_artifact/decoded_quote/envelope.rs:48-143 sha256:7b661f465105e8eaa7f29332c78ec1cf59da541652216e26a9ec3a76d9c95d80
/-! Exact partition of versioned read admission over the slot facts obtained
from the authenticated input. Passing this gate still requires full canonical
body readmission. A populated authority, parent branch, or base commit slot
has the same refusal in a checkpoint member and in a journal frame. -/

namespace LegacyMergeCodec

inductive Version where
  | current | legacy | unsupported
  deriving DecidableEq

inductive Slots where
  | absent | empty | populated
  deriving DecidableEq

inductive Disposition where
  | inspectOrdinary | unsupportedVersion | retiredMerge | unexpectedLegacySlot
  deriving DecidableEq

def dispatch : Version → Slots → Disposition
  | .unsupported, _ => .unsupportedVersion
  | _, .populated => .retiredMerge
  | .current, .empty => .unexpectedLegacySlot
  | _, _ => .inspectOrdinary

theorem ordinary_iff (v : Version) (slots : Slots) :
    dispatch v slots = .inspectOrdinary ↔
      (v = .legacy ∧ slots ≠ .populated) ∨ (v = .current ∧ slots = .absent) := by
  cases v <;> cases slots <;> simp [dispatch]

theorem retired_iff (v : Version) (slots : Slots) :
    dispatch v slots = .retiredMerge ↔ v ≠ .unsupported ∧ slots = .populated := by
  cases v <;> cases slots <;> simp [dispatch]

theorem unsupported_iff (v : Version) (slots : Slots) :
    dispatch v slots = .unsupportedVersion ↔ v = .unsupported := by
  cases v <;> cases slots <;> simp [dispatch]

theorem unexpected_iff (v : Version) (slots : Slots) :
    dispatch v slots = .unexpectedLegacySlot ↔ v = .current ∧ slots = .empty := by
  cases v <;> cases slots <;> simp [dispatch]

end LegacyMergeCodec
