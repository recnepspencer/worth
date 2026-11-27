/-!
# Bounded discriminator wire size (Phase 7B-2, RULES §4 layer 1)

Models the persistent-name discriminator value algebra and the compile-time
wire bound `worth-query-declaration/.../discriminator/wire_bound.rs` derives
from field types. Byte counts mirror the encoder exactly: every tag is one
byte, every length prefix eight, a name reference a tag plus 32 bytes.

`encLen_le_bound` proves the derived bound is sound for every value of a
bounded type; `split_chain_birth_bounded` proves that a split chain of any
depth `d`, whose births are told apart by a bounded discriminator, records
births no larger than one constant independent of `d`.
-/
namespace PersistentNaming.WireBound

/-- Discriminator value types. `text` is the only unbounded one. -/
inductive Ty where
  | bool
  | u64
  | name
  | text
  | opt (inner : Ty)
  | pair (first second : Ty)

/-- Discriminator values; `text n` is a text of `n` bytes. -/
inductive Val where
  | bool (value : Bool)
  | u64 (value : Nat)
  | name
  | text (len : Nat)
  | none
  | some (inner : Val)
  | pair (first second : Val)

/-- `v` is a value of type `t`. -/
inductive HasType : Val → Ty → Prop where
  | bool (b : Bool) : HasType (.bool b) .bool
  | u64 (n : Nat) : HasType (.u64 n) .u64
  | name : HasType .name .name
  | text (n : Nat) : HasType (.text n) .text
  | none (t : Ty) : HasType .none (.opt t)
  | some {v : Val} {t : Ty} : HasType v t → HasType (.some v) (.opt t)
  | pair {a b : Val} {s t : Ty} : HasType a s → HasType b t → HasType (.pair a b) (.pair s t)

/-- Bytes the encoder writes for a value. -/
def encLen : Val → Nat
  | .bool _ => 2
  | .u64 _ => 9
  | .name => 33
  | .text n => 1 + 8 + n
  | .none => 1
  | .some v => 1 + encLen v
  | .pair a b => 1 + encLen a + encLen b

/-- The derived bound (`MAX_ENCODED_BYTES`); `none` when the type can grow. -/
def bound : Ty → Option Nat
  | .bool => .some 2
  | .u64 => .some 9
  | .name => .some 33
  | .text => .none
  | .opt t => (bound t).map (1 + ·)
  | .pair s t =>
    match bound s, bound t with
    | .some x, .some y => .some (1 + x + y)
    | _, _ => .none

/-- The derived bound is sound: no value of a bounded type encodes longer. -/
theorem encLen_le_bound : ∀ {v : Val} {t : Ty} {b : Nat},
    HasType v t → bound t = .some b → encLen v ≤ b := by
  intro v t b h
  induction h generalizing b with
  | bool => intro hb; simp [bound] at hb; simp [encLen]; omega
  | u64 => intro hb; simp [bound] at hb; simp [encLen]; omega
  | name => intro hb; simp [bound] at hb; simp [encLen]; omega
  | text => intro hb; simp [bound] at hb
  | none t =>
    intro hb
    simp only [bound] at hb
    cases h : bound t with
    | none => simp [h] at hb
    | some x => simp [h] at hb; simp [encLen]; omega
  | @some v t _ ih =>
    intro hb
    simp only [bound] at hb
    cases h : bound t with
    | none => simp [h] at hb
    | some x =>
      simp [h] at hb
      have := ih h
      simp [encLen]; omega
  | @pair a b s t _ _ iha ihb =>
    intro hb
    simp only [bound] at hb
    cases hs : bound s with
    | none => simp [hs] at hb
    | some x =>
      cases ht : bound t with
      | none => simp only [hs, ht] at hb; cases hb
      | some y =>
        simp only [hs, ht] at hb
        injection hb with hb
        have := iha hs
        have := ihb ht
        simp [encLen]; omega

/-- A text-carrying type is never bounded: ancestry text cannot pass the
structural-identity fence. -/
theorem text_unbounded : bound .text = .none := rfl

/-- A discriminator: its identity length, then named fields. -/
structure Disc where
  identityLen : Nat
  fields : List (Nat × Ty)

/-- Fields' encoded bytes: tag, length-prefixed name, value. -/
def fieldsLen : List (Nat × Ty) → List Val → Nat
  | (nameLen, _) :: fs, v :: vs => 1 + 8 + nameLen + encLen v + fieldsLen fs vs
  | _, _ => 0

/-- The derived bound of a field list. -/
def fieldsBound : List (Nat × Ty) → Option Nat
  | [] => .some 0
  | (nameLen, t) :: fs =>
    match bound t, fieldsBound fs with
    | .some x, .some y => .some (1 + 8 + nameLen + x + y)
    | _, _ => .none

/-- Values typed field by field. -/
inductive FieldsTyped : List (Nat × Ty) → List Val → Prop where
  | nil : FieldsTyped [] []
  | cons {nameLen : Nat} {t : Ty} {v : Val} {fs : List (Nat × Ty)} {vs : List Val} :
      HasType v t → FieldsTyped fs vs → FieldsTyped ((nameLen, t) :: fs) (v :: vs)

theorem fieldsLen_le_bound : ∀ {fs : List (Nat × Ty)} {vs : List Val} {b : Nat},
    FieldsTyped fs vs → fieldsBound fs = .some b → fieldsLen fs vs ≤ b := by
  intro fs vs b h
  induction h generalizing b with
  | nil => intro hb; simp [fieldsBound] at hb; simp [fieldsLen]
  | @cons nameLen t v fs vs hv _ ih =>
    intro hb
    simp only [fieldsBound] at hb
    cases ht : bound t with
    | none => simp [ht] at hb
    | some x =>
      cases hf : fieldsBound fs with
      | none => simp only [ht, hf] at hb; cases hb
      | some y =>
        simp only [ht, hf] at hb
        injection hb with hb
        have := encLen_le_bound hv ht
        have := ih hf
        simp [fieldsLen]; omega

/-- The domain tag `worth.query.name-discriminator.v1\0`. -/
def domainTagLen : Nat := 34

/-- Whole canonical discriminator bytes. -/
def discLen (d : Disc) (vs : List Val) : Nat :=
  domainTagLen + 8 + d.identityLen + fieldsLen d.fields vs

/-- One recorded birth: scheme, occurrence and role text, its parents (32
bytes each) and its discriminator. -/
def birthLen (schemeLen occurrenceLen roleLen parents discBytes : Nat) : Nat :=
  schemeLen + occurrenceLen + roleLen + 32 * parents + discBytes

/-- Every birth of a split chain of any depth `d`, each with at most one
parent and a discriminator of a bounded type, is at most one constant
`K`, which does not mention `d`. -/
theorem split_chain_birth_bounded
    (d : Disc) (b schemeLen occurrenceLen roleLen : Nat)
    (hb : fieldsBound d.fields = .some b) :
    ∃ K, ∀ (depth : Nat) (chain : Nat → List Val) (parents : Nat → Nat),
      (∀ i, i < depth → FieldsTyped d.fields (chain i) ∧ parents i ≤ 1) →
      ∀ i, i < depth →
        birthLen schemeLen occurrenceLen roleLen (parents i) (discLen d (chain i)) ≤ K := by
  refine ⟨schemeLen + occurrenceLen + roleLen + 32 + (domainTagLen + 8 + d.identityLen + b), ?_⟩
  intro depth chain parents hchain i hi
  obtain ⟨htyped, hparents⟩ := hchain i hi
  have := fieldsLen_le_bound htyped hb
  simp only [birthLen, discLen]
  omega

end PersistentNaming.WireBound
