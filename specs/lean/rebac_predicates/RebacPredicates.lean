/-!
# ReBAC path predicates: authored conditions and their kernel lowering

Models `worth-query-declaration/src/application_schema/authorization_policy/`
(`condition.rs`, `expansion.rs`) against the relational kernel's predicate
semantics (`worth-relational-mutation/src/authorization/constraint.rs`).

* A kernel comparison is false on an absent value or a value of another scalar
  family; within a family it compares the stored scalar (`AspectValue`'s order).
* `eq`/`one_of` lower to `Equal` alternatives, `ne` to `StrictlyLess ∪
  StrictlyGreater`, and principal-field conditions to field constraints.
* A path's conditions expand to the cartesian product of their alternatives.
* Ordered comparisons mean the domain order only on order-faithful families.
-/

namespace RebacPredicates

inductive Fam where
  | int | uint | date | time | timestamp | decimal | bigint | rational | float | string
  deriving DecidableEq, Repr

/-- A stored scalar: its family and the key the kernel orders within the family. -/
structure Val where
  fam : Fam
  key : Int
  deriving DecidableEq, Repr

inductive Cmp where
  | eq | atMost | atLeast | lt | gt
  deriving DecidableEq, Repr

def cmpKeys : Cmp → Int → Int → Bool
  | .eq, a, b => decide (a = b)
  | .atMost, a, b => decide (a ≤ b)
  | .atLeast, a, b => decide (a ≥ b)
  | .lt, a, b => decide (a < b)
  | .gt, a, b => decide (a > b)

/-- Kernel `RelationalAuthorizationPredicate::matches`. -/
def kmatch (c : Cmp) (observed : Option Val) (expected : Val) : Bool :=
  match observed with
  | none => false
  | some o => if o.fam = expected.fam then cmpKeys c o.key expected.key else false

/-- Kernel `RelationalAuthorizationFieldConstraint::matches` over two read fields. -/
def kmatchField (c : Cmp) (left right : Option Val) : Bool :=
  match left, right with
  | some l, some r => if l.fam = r.fam then cmpKeys c l.key r.key else false
  | _, _ => false

/-- Authored conditions (`ApplicationAuthorizationFieldConditions`). -/
inductive Cond where
  | eq (v : Val)
  | ne (v : Val)
  | oneOf (vs : List Val)
  | ord (c : Cmp) (v : Val)

/-- The kernel alternatives a condition lowers to; it holds when any matches. -/
def lower : Cond → List (Cmp × Val)
  | .eq v => [(.eq, v)]
  | .ne v => [(.lt, v), (.gt, v)]
  | .oneOf vs => vs.map (fun v => (.eq, v))
  | .ord c v => [(c, v)]

def holds (c : Cond) (o : Option Val) : Bool :=
  (lower c).any (fun p => kmatch p.1 o p.2)

theorem val_eq_of_fam_key {a b : Val} (hf : a.fam = b.fam) (hk : a.key = b.key) : a = b := by
  cases a; cases b; simp_all

theorem eq_sound (v : Val) (o : Option Val) : holds (.eq v) o = true ↔ o = some v := by
  cases o with
  | none => simp [holds, lower, kmatch]
  | some w =>
    simp only [holds, lower, List.any_cons, List.any_nil, Bool.or_false, kmatch]
    constructor
    · intro h
      by_cases hf : w.fam = v.fam
      · simp [hf, cmpKeys] at h
        exact congrArg some (val_eq_of_fam_key hf h)
      · simp [hf] at h
    · intro h
      cases h
      simp [cmpKeys]

/-- `ne` lowered to `<` ∪ `>` holds exactly on a present, same-family, different
value. This needs nothing of the family but a total order consistent with
equality, so it is sound on every family. -/
theorem ne_sound (v : Val) (o : Option Val) :
    holds (.ne v) o = true ↔ ∃ w, o = some w ∧ w.fam = v.fam ∧ w ≠ v := by
  cases o with
  | none => simp [holds, lower, kmatch]
  | some w =>
    simp only [holds, lower, List.any_cons, List.any_nil, Bool.or_false, kmatch]
    by_cases hf : w.fam = v.fam
    · simp only [hf, ite_true, cmpKeys, Bool.or_eq_true, decide_eq_true_eq]
      constructor
      · intro h
        refine ⟨w, rfl, hf, ?_⟩
        intro hwv
        cases hwv
        omega
      · rintro ⟨w', hw', -, hne⟩
        simp only [Option.some.injEq] at hw'
        subst hw'
        have hk : w.key ≠ v.key := fun hk => hne (val_eq_of_fam_key hf hk)
        omega
    · simp only [hf, ite_false, Bool.or_false]
      constructor
      · intro h; exact absurd h (by simp)
      · rintro ⟨w', hw', hf', -⟩
        cases hw'
        exact absurd hf' hf

theorem oneOf_sound (vs : List Val) (o : Option Val) :
    holds (.oneOf vs) o = true ↔ ∃ v ∈ vs, o = some v := by
  simp only [holds, lower, List.any_map, List.any_eq_true, Function.comp_def]
  constructor
  · rintro ⟨v, hv, h⟩
    exact ⟨v, hv, (eq_sound v o).mp (by simpa [holds, lower] using h)⟩
  · rintro ⟨v, hv, h⟩
    exact ⟨v, hv, by simpa [holds, lower] using (eq_sound v o).mpr h⟩

/-- Fail closed: an absent field satisfies no authored condition. -/
theorem absent_never_holds (c : Cond) : holds c none = false := by
  simp [holds, kmatch]

/-- Fail closed: a value of another family satisfies no single-valued condition. -/
theorem other_family_never_holds (c : Cmp) (v w : Val) (h : w.fam ≠ v.fam) :
    kmatch c (some w) v = false := by
  simp [kmatch, h]

/-- Consequently a deny path over an absent optional field never fires; the
builder refuses deny conditions on optional fields for this reason. -/
theorem deny_on_absent_field_is_inert (c : Cond) : holds c none = false :=
  absent_never_holds c

/-- `ne_actor` lowered to two field constraints holds exactly when both fields
are present, share a family and differ. -/
theorem ne_actor_sound (l r : Option Val) :
    (kmatchField .lt l r || kmatchField .gt l r) = true ↔
      ∃ a b, l = some a ∧ r = some b ∧ a.fam = b.fam ∧ a ≠ b := by
  cases l with
  | none => simp [kmatchField]
  | some a =>
    cases r with
    | none => simp [kmatchField]
    | some b =>
      simp only [kmatchField]
      by_cases hf : a.fam = b.fam
      · simp only [hf, ite_true, cmpKeys, Bool.or_eq_true, decide_eq_true_eq]
        constructor
        · intro h
          refine ⟨a, b, rfl, rfl, hf, ?_⟩
          intro hab; cases hab; omega
        · rintro ⟨a', b', ha, hb, -, hne⟩
          simp only [Option.some.injEq] at ha hb
          subst ha; subst hb
          have hk : a.key ≠ b.key := fun hk => hne (val_eq_of_fam_key hf hk)
          omega
      · simp only [hf, ite_false, Bool.or_false]
        constructor
        · intro h; exact absurd h (by simp)
        · rintro ⟨a', b', ha, hb, hf', -⟩
          cases ha; cases hb
          exact absurd hf' hf

theorem eq_actor_sound (l r : Option Val) :
    kmatchField .eq l r = true ↔ ∃ a, l = some a ∧ r = some a := by
  cases l with
  | none => simp [kmatchField]
  | some a =>
    cases r with
    | none => simp [kmatchField]
    | some b =>
      simp only [kmatchField]
      by_cases hf : a.fam = b.fam
      · simp only [hf, ite_true, cmpKeys, decide_eq_true_eq]
        constructor
        · intro hk
          exact ⟨a, rfl, by rw [val_eq_of_fam_key hf hk]⟩
        · rintro ⟨a', ha, hb⟩
          cases ha; cases hb; rfl
      · simp only [hf, ite_false]
        constructor
        · intro h; exact absurd h (by simp)
        · rintro ⟨a', ha, hb⟩
          cases ha; cases hb
          exact absurd rfl hf

/-! ## Order faithfulness

`meaning f k` is the domain value a stored key `k` of family `f` encodes. A
family is order-faithful when the stored order is the domain order. -/

/-- Integers and canonical date/time/timestamp store their domain value as the
ordered key. Decimal and big-integer text order lexicographically: stored rank
0 is the text "10000" and rank 1 is "9". -/
def meaning : Fam → Int → Int
  | .int, k | .uint, k | .date, k | .time, k | .timestamp, k => k
  | .decimal, k | .bigint, k => if k = 0 then 10000 else if k = 1 then 9 else k
  | .rational, k | .float, k | .string, k => k

def Faithful (f : Fam) : Prop := ∀ a b : Int, a < b → meaning f a < meaning f b

/-- Exactly the families `ApplicationOrderFaithfulBinding` admits. -/
def orderFaithfulFamily : Fam → Bool
  | .int | .uint | .date | .time | .timestamp => true
  | _ => false

theorem admitted_families_are_faithful (f : Fam) (h : orderFaithfulFamily f = true) :
    Faithful f := by
  intro a b hab
  cases f <;> simp_all [orderFaithfulFamily, meaning]

theorem le_iff_of_faithful {f : Fam} (hf : Faithful f) (a b : Int) :
    a ≤ b ↔ meaning f a ≤ meaning f b := by
  constructor
  · intro h
    rcases Int.lt_or_eq_of_le h with hlt | heq
    · exact Int.le_of_lt (hf a b hlt)
    · rw [heq]; exact Int.le_refl _
  · intro h
    apply Int.not_lt.mp
    intro hba
    exact absurd (hf b a hba) (Int.not_lt.mpr h)

/-- On an admitted family, kernel `AtMost` is the domain `≤`. -/
theorem atMost_means_le (w v : Val) (h : orderFaithfulFamily v.fam = true) :
    kmatch .atMost (some w) v = true ↔
      w.fam = v.fam ∧ meaning v.fam w.key ≤ meaning v.fam v.key := by
  have hf := admitted_families_are_faithful v.fam h
  simp only [kmatch]
  by_cases hfam : w.fam = v.fam
  · simp only [hfam, ite_true, cmpKeys, decide_eq_true_eq, true_and]
    exact le_iff_of_faithful hf w.key v.key
  · simp [hfam]

/-- Why decimal text is excluded: the kernel finds "10000" at most "9". -/
theorem decimal_text_order_is_not_domain_order :
    kmatch .atMost (some ⟨.decimal, 0⟩) ⟨.decimal, 1⟩ = true ∧
      meaning .decimal 0 > meaning .decimal 1 := by
  decide

theorem decimal_text_is_not_faithful : ¬ Faithful .decimal := by
  intro h
  have := h 0 1 (by decide)
  simp [meaning] at this

/-! ## Path expansion

A path holds when every condition holds, each condition holding when any of
its alternatives does. Lowering installs the cartesian product of the
alternatives, one kernel path per choice; the policy admits when any path does. -/

def expand {α : Type} : List (List α) → List (List α)
  | [] => [[]]
  | xs :: rest => xs.flatMap (fun x => (expand rest).map (x :: ·))

theorem expand_iff {α : Type} (P : α → Prop) (xss : List (List α)) :
    (∃ path ∈ expand xss, ∀ a ∈ path, P a) ↔ ∀ xs ∈ xss, ∃ a ∈ xs, P a := by
  induction xss with
  | nil => simp [expand]
  | cons xs rest ih =>
    simp only [expand, List.mem_flatMap, List.mem_map, List.mem_cons, forall_eq_or_imp]
    constructor
    · rintro ⟨path, ⟨x, hx, tail, htail, rfl⟩, hall⟩
      refine ⟨⟨x, hx, hall x (List.mem_cons_self ..)⟩, ?_⟩
      exact ih.mp ⟨tail, htail, fun a ha => hall a (List.mem_cons_of_mem _ ha)⟩
    · rintro ⟨⟨x, hx, px⟩, hrest⟩
      obtain ⟨tail, htail, hall⟩ := ih.mpr hrest
      refine ⟨x :: tail, ⟨x, hx, tail, htail, rfl⟩, ?_⟩
      intro a ha
      rcases List.mem_cons.mp ha with rfl | ha
      · exact px
      · exact hall a ha

/-- The kernel alternatives of one authored condition over its observed value. -/
def alternatives (co : Cond × Option Val) : List ((Cmp × Val) × Option Val) :=
  (lower co.1).map (fun p => (p, co.2))

/-- The installed policy for one authored path admits exactly when every
authored condition holds on the observed values. -/
theorem lowered_path_sound (conds : List (Cond × Option Val)) :
    (∃ path ∈ expand (conds.map alternatives),
        ∀ a ∈ path, kmatch a.1.1 a.2 a.1.2 = true) ↔
      ∀ co ∈ conds, holds co.1 co.2 = true := by
  rw [expand_iff]
  unfold holds
  constructor
  · intro h co hco
    obtain ⟨a, ha, hm⟩ := h (alternatives co) (List.mem_map.mpr ⟨co, hco, rfl⟩)
    obtain ⟨p, hp, rfl⟩ := List.mem_map.mp ha
    exact List.any_eq_true.mpr ⟨p, hp, hm⟩
  · intro h xs hxs
    obtain ⟨co, hco, rfl⟩ := List.mem_map.mp hxs
    obtain ⟨p, hp, hm⟩ := List.any_eq_true.mp (h co hco)
    exact ⟨(p, co.2), List.mem_map.mpr ⟨p, hp, rfl⟩, hm⟩

end RebacPredicates
