/-!
# Step-counted computations

`TimeM α` is a value with the number of unit steps spent computing it.
Each modelled loop ticks once per element it visits, so a theorem about
`steps` bounds exactly the work the Rust loop performs per visited row
(each row probe is one `O(log N)` tiered-map seek in the implementation).
-/
namespace CanonicalGroupDelta

/-- A value and the steps spent computing it. -/
structure TimeM (α : Type) where
  val : α
  steps : Nat

namespace TimeM

/-- Sequential composition: the steps of both halves. -/
def bind {α β : Type} (m : TimeM α) (f : α → TimeM β) : TimeM β :=
  ⟨(f m.val).val, m.steps + (f m.val).steps⟩

/-- One unit step producing `a`. -/
def tick {α : Type} (a : α) : TimeM α := ⟨a, 1⟩

@[simp] theorem bind_val {α β : Type} (m : TimeM α) (f : α → TimeM β) :
    (bind m f).val = (f m.val).val := rfl

@[simp] theorem bind_steps {α β : Type} (m : TimeM α) (f : α → TimeM β) :
    (bind m f).steps = m.steps + (f m.val).steps := rfl

@[simp] theorem tick_val {α : Type} (a : α) : (tick a).val = a := rfl

@[simp] theorem tick_steps {α : Type} (a : α) : (tick a).steps = 1 := rfl

end TimeM

/-- A left fold ticking once per element. -/
def foldlStep {σ α : Type} (f : σ → α → σ) : σ → List α → TimeM σ
  | s, [] => ⟨s, 0⟩
  | s, a :: as => TimeM.bind (TimeM.tick (f s a)) (fun s' => foldlStep f s' as)

/-- A right fold ticking once per element. -/
def foldrStep {σ α : Type} (f : α → σ → σ) (s : σ) : List α → TimeM σ
  | [] => ⟨s, 0⟩
  | a :: as => TimeM.bind (foldrStep f s as) (fun s' => TimeM.tick (f a s'))

@[simp] theorem foldlStep_val {σ α : Type} (f : σ → α → σ) :
    ∀ (s : σ) (l : List α), (foldlStep f s l).val = l.foldl f s
  | _, [] => rfl
  | s, a :: as => by simp [foldlStep, foldlStep_val f (f s a) as]

@[simp] theorem foldlStep_steps {σ α : Type} (f : σ → α → σ) :
    ∀ (s : σ) (l : List α), (foldlStep f s l).steps = l.length
  | _, [] => rfl
  | s, a :: as => by simp [foldlStep, foldlStep_steps f (f s a) as]; omega

@[simp] theorem foldrStep_val {σ α : Type} (f : α → σ → σ) (s : σ) :
    ∀ (l : List α), (foldrStep f s l).val = l.foldr f s
  | [] => rfl
  | a :: as => by simp [foldrStep, foldrStep_val f s as]

@[simp] theorem foldrStep_steps {σ α : Type} (f : α → σ → σ) (s : σ) :
    ∀ (l : List α), (foldrStep f s l).steps = l.length
  | [] => rfl
  | a :: as => by simp [foldrStep, foldrStep_steps f s as]

end CanonicalGroupDelta
