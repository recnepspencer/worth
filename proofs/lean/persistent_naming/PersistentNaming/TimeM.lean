/-!
# Step-counted computations

`TimeM α` is a value with the number of unit steps spent computing it, as in
`proofs/lean/canonical_group_delta`. `ForkChildrenCost` and `CowMapForkCost`
count their steps in it.

Step-count infrastructure only: it models no production Rust, so it carries
no `proof-mirror:` pin.
-/
namespace PersistentNaming

/-- A value and the steps spent computing it. -/
structure TimeM (α : Type) where
  val : α
  steps : Nat

namespace TimeM

/-- One unit step producing `a`. -/
def tick {α : Type} (a : α) : TimeM α := ⟨a, 1⟩

/-- Sequential composition: the steps of both halves. -/
def bind {α β : Type} (m : TimeM α) (f : α → TimeM β) : TimeM β :=
  ⟨(f m.val).val, m.steps + (f m.val).steps⟩

@[simp] theorem tick_val {α : Type} (a : α) : (tick a).val = a := rfl

@[simp] theorem tick_steps {α : Type} (a : α) : (tick a).steps = 1 := rfl

@[simp] theorem bind_val {α β : Type} (m : TimeM α) (f : α → TimeM β) :
    (bind m f).val = (f m.val).val := rfl

@[simp] theorem bind_steps {α β : Type} (m : TimeM α) (f : α → TimeM β) :
    (bind m f).steps = m.steps + (f m.val).steps := rfl

end TimeM

end PersistentNaming
