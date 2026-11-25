/-!
# Coupled succession: the store model (design §5.1)

Absolute record images. A `Store` maps every record to its image (`none` = absent).
An `Eff` is one member's effect: the records it changes (`dom`) and their images.
-/
namespace CoupledSuccession

abbrev Rec := Nat
abbrev Img := Option Nat
abbrev Store := Rec → Img
abbrev Dom := Rec → Bool

/-- One member's (or one round's) effect on the store. -/
structure Eff where
  dom : Dom
  img : Rec → Img

/-- Apply an effect: records in `e.dom` take `e.img`, everything else keeps `s`. -/
def apply (e : Eff) (s : Store) : Store := fun r => if e.dom r then e.img r else s r

/-- Restore `w` from `b`; keep `s` elsewhere. -/
def retract (b : Store) (w : Dom) (s : Store) : Store := fun r => if w r then b r else s r

/-- Query's view of the chain's records: the head image `b` over `w`, the current image elsewhere. -/
def overlay (b : Store) (w : Dom) (cur : Store) : Store := retract b w cur

def noneStore : Store := fun _ => none

def Sub (a c : Dom) : Prop := ∀ r, a r = true → c r = true

def Disj (a c : Dom) : Prop := ∀ r, a r = true → c r = false

/-- A member appended to the coupled candidate: a round of chain `c`, or a foreign member. -/
inductive Step
  | chain (c : Nat) (e : Eff)
  | foreign (f : Eff)

/-- Foreign links must point at live records (non-vacuous). -/
def Integrity (s : Store) (links : Rec → Rec → Bool) : Prop :=
  ∀ src tgt, links src tgt = true → (s src).isSome ∧ (s tgt).isSome

/-- List permutation (core `List.Perm`). -/
def Perm {α : Type} (l₁ l₂ : List α) : Prop := List.Perm l₁ l₂

/-- No duplicates (core `List.Nodup`). -/
def NoDup {α : Type} (l : List α) : Prop := List.Nodup l

/-- Union of two record sets. -/
def unionDom (a c : Dom) : Dom := fun r => a r || c r

/-! ## Pointwise lemmas -/

theorem apply_of_dom {e : Eff} {s : Store} {r : Rec} (h : e.dom r = true) :
    apply e s r = e.img r := by
  simp [apply, h]

theorem apply_of_not {e : Eff} {s : Store} {r : Rec} (h : e.dom r = false) :
    apply e s r = s r := by
  simp [apply, h]

theorem apply_congr_pt (e : Eff) {s s' : Store} {r : Rec} (h : s r = s' r) :
    apply e s r = apply e s' r := by
  simp only [apply]; split <;> simp [h]

theorem disj_left {a c : Dom} {r : Rec} (h : Disj a c) (hc : c r = true) : a r = false := by
  cases ha : a r
  · rfl
  · have := h r ha; rw [hc] at this; cases this

theorem disj_right {a c : Dom} {r : Rec} (h : Disj a c) (ha : a r = true) : c r = false :=
  h r ha

theorem sub_false {a c : Dom} {r : Rec} (h : Sub a c) (hc : c r = false) : a r = false := by
  cases ha : a r
  · rfl
  · have := h r ha; rw [hc] at this; cases this

/-- Applying two effects with disjoint domains commutes. -/
theorem apply_comm_of_disj {e f : Eff} (h : Disj e.dom f.dom) (s : Store) :
    apply e (apply f s) = apply f (apply e s) := by
  funext r
  cases he : e.dom r with
  | true =>
    have hf : f.dom r = false := h r he
    simp [apply, he, hf]
  | false =>
    cases hf : f.dom r <;> simp [apply, he, hf]

/-! ## Folding a list of effects -/

/-- Apply the effects of a list in order (left to right). -/
def foldBy {α : Type} (f : α → Eff) (l : List α) (s : Store) : Store :=
  l.foldl (fun s a => apply (f a) s) s

theorem foldBy_nil {α : Type} (f : α → Eff) (s : Store) : foldBy f [] s = s := rfl

theorem foldBy_cons {α : Type} (f : α → Eff) (a : α) (l : List α) (s : Store) :
    foldBy f (a :: l) s = foldBy f l (apply (f a) s) := rfl

theorem foldBy_append {α : Type} (f : α → Eff) (l l' : List α) (s : Store) :
    foldBy f (l ++ l') s = foldBy f l' (foldBy f l s) := by
  simp [foldBy, List.foldl_append]

/-- The image of `r` after a fold depends only on the starting image of `r`. -/
theorem foldBy_congr_pt {α : Type} (f : α → Eff) (l : List α) :
    ∀ {s s' : Store} {r : Rec}, s r = s' r → foldBy f l s r = foldBy f l s' r := by
  induction l with
  | nil => intro s s' r h; exact h
  | cons a l ih =>
    intro s s' r h
    simp only [foldBy_cons]
    exact ih (apply_congr_pt (f a) h)

/-- Effects that do not write `r` leave it alone. -/
theorem foldBy_skip {α : Type} (f : α → Eff) (l : List α) :
    ∀ {s : Store} {r : Rec}, (∀ a ∈ l, (f a).dom r = false) → foldBy f l s r = s r := by
  induction l with
  | nil => intro s r _; rfl
  | cons a l ih =>
    intro s r h
    simp only [foldBy_cons]
    rw [ih (fun x hx => h x (List.mem_cons_of_mem a hx))]
    exact apply_of_not (h a (List.mem_cons_self))

/-- An effect disjoint from every effect of a list commutes past the whole list. -/
theorem foldBy_comm {α : Type} (f : α → Eff) (l : List α) (e : Eff) (s : Store)
    (h : ∀ a ∈ l, Disj (f a).dom e.dom) :
    foldBy f l (apply e s) = apply e (foldBy f l s) := by
  funext r
  cases he : e.dom r with
  | true =>
    have hs : ∀ a ∈ l, (f a).dom r = false := fun a ha => disj_left (h a ha) he
    rw [foldBy_skip f l hs, apply_of_dom he, apply_of_dom he]
  | false =>
    rw [apply_of_not he]
    exact foldBy_congr_pt f l (apply_of_not he)

/-- Replacing list entries that do not write `r` (before or after) does not change `r`. -/
theorem foldBy_map_pt {α : Type} (f : α → Eff) (g : α → α) (l : List α) :
    ∀ {s : Store} {r : Rec},
      (∀ a ∈ l, g a = a ∨ ((f a).dom r = false ∧ (f (g a)).dom r = false)) →
      foldBy f (l.map g) s r = foldBy f l s r := by
  induction l with
  | nil => intro s r _; rfl
  | cons a l ih =>
    intro s r h
    simp only [List.map_cons, foldBy_cons]
    have hl : ∀ x ∈ l, g x = x ∨ ((f x).dom r = false ∧ (f (g x)).dom r = false) :=
      fun x hx => h x (List.mem_cons_of_mem a hx)
    rcases h a List.mem_cons_self with hg | ⟨h1, h2⟩
    · rw [hg]; exact ih hl
    · rw [ih hl]
      apply foldBy_congr_pt
      rw [apply_of_not h1, apply_of_not h2]

end CoupledSuccession
