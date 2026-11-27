/-
Standing grants, extent liveness and grant validity (W9A-2), with open
validity bounds (W10A).

Mirrors
  workspaces/worth-query/crates/worth-query-execution-lineage/src/domain_computation/
    authorization/capability_lowering.rs        (`compile_grant_path`)
    authorization/capability_lowering/extent_status.rs (`lower`)
  * the grant-level predicates (action, purpose, active status, validity)
                                          -> the abstract `base`
  * a workflow-tenure grant's field constraint `grant.workflow == scope.workflow`
                                          -> `Tenure.workflow`; a standing grant has none
  * each `require_active` binding becomes an equality predicate on every
    path ordinal from the head (ordinal 2) onward whose kind is the binding's
                                          -> `extentHolds`
  * relational predicates conjoin: a path admits only when all hold
                                          -> `admits`
and the validity comparison `not_before <= sample <= not_after` on one sample
of the admission instant (`grant_selection.rs`, `append_grant_predicates`).
An absent bound (W10A `from`, `until`, `unbounded`) adds no predicate
                                          -> `validOpen`
and a window open above reports the timeline's end as its expiry
(`expiry.rs`, `observe_grant_expiry`).

Properties:
  * `extent_narrows`: declaring liveness only removes admissions;
  * `inactive_root_revokes`: an extent entity on the path holding another
    status refuses every grant, principal and tenure (rooted revocation);
  * `standing_admits_iff`, `workflow_admits_standing`: a standing grant is a
    workflow grant minus the workflow equality, nothing else;
  * `expired_stays_expired`: on a UTC timeline expiry is permanent;
  * `expired_stays_expired_local`: on a local-date timeline it is permanent
    exactly when the calendar's date never moves backwards (`Monotone`);
  * `validOpen_closed`: with both bounds present, open validity is exactly
    the closed comparison, so closed grants are unchanged;
  * `open_above_never_expires`, `unbounded_always_valid`: an open upper
    bound never expires, and the unbounded window admits every sample;
  * `expired_stays_expired_open`: a present upper bound still expires
    permanently whatever the lower bound;
  * `open_bound_is_extreme`: an absent bound admits exactly what the
    timeline's extreme value admits, so the reported end-of-time expiry is
    the bound the predicate omits.
-/
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution-lineage/src/domain_computation/authorization/capability_lowering.rs:39-101 sha256:dbad2ea8453fb9ce731e9a38f4273e131d70d6f0006044cc2499cb1cda452e43
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution-attempt/src/domain_computation/authorization/capability_observation/grant_selection.rs:253-275 sha256:24d387defb8e67525afa5bb7c495959d04e66907abe0add2a8d0ced0e7cbe370
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution-attempt/src/domain_computation/authorization/capability_observation/expiry.rs:13-16 sha256:3370e8be9356dbf95b50e0b9301008d831870e718f4f64512cb64c9d5a25e725
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution-lineage/src/domain_computation/authorization/capability_lowering/extent_status.rs:22-57 sha256:59e0cf355edea31a10bd84a851189c5c93502f62c35d51caf54af6f4f000f19c

namespace CapabilityGrantPath

/-- One entity on the grant path: its kind and the value of a status field. -/
structure Node where
  kind : Nat
  status : Nat
  deriving DecidableEq

/-- `require_active`: every path entity of `kind` must hold `value`. -/
structure Extent where
  kind : Nat
  value : Nat
  deriving DecidableEq

/-- Principal (0), grant (1), then the head or root (2) and later steps. -/
abbrev Path := List Node

def nodeHolds (e : Extent) (n : Node) : Bool :=
  n.kind != e.kind || n.status == e.value

/-- The predicates lowered for one extent binding, from ordinal 2 onward. -/
def extentHolds (path : Path) (e : Extent) : Bool :=
  (path.drop 2).all (nodeHolds e)

inductive Tenure where
  | workflow (grantWorkflow scopeWorkflow : Nat)
  | standing

def tenureHolds : Tenure → Bool
  | .workflow g s => g == s
  | .standing => true

/-- One candidate grant path admits when every lowered constraint holds. -/
def admits (base : Bool) (tenure : Tenure) (extents : List Extent) (path : Path) : Bool :=
  base && tenureHolds tenure && extents.all (extentHolds path)

theorem extent_narrows (base : Bool) (t : Tenure) (e : Extent) (es : List Extent)
    (path : Path) (h : admits base t (e :: es) path = true) :
    admits base t es path = true := by
  simp only [admits, List.all_cons, Bool.and_eq_true] at h ⊢
  exact ⟨h.1, h.2.2⟩

theorem inactive_root_revokes (base : Bool) (t : Tenure) (es : List Extent) (e : Extent)
    (principal grant root : Node) (rest : Path) (member : e ∈ es)
    (kind : root.kind = e.kind) (inactive : root.status ≠ e.value) :
    admits base t es (principal :: grant :: root :: rest) = false := by
  have root_fails : nodeHolds e root = false := by
    simp [nodeHolds, kind, inactive]
  have extent_fails : extentHolds (principal :: grant :: root :: rest) e = false := by
    simp [extentHolds, List.all_cons, root_fails]
  have all_fails : es.all (extentHolds (principal :: grant :: root :: rest)) = false := by
    rw [List.all_eq_false]
    exact ⟨e, member, by simp [extent_fails]⟩
  simp [admits, all_fails]

theorem standing_admits_iff (base : Bool) (es : List Extent) (path : Path) :
    admits base .standing es path = (base && es.all (extentHolds path)) := by
  simp [admits, tenureHolds]

theorem workflow_admits_standing (base : Bool) (g s : Nat) (es : List Extent) (path : Path)
    (h : admits base (.workflow g s) es path = true) :
    admits base .standing es path = true := by
  simp only [admits, tenureHolds, Bool.and_eq_true] at h ⊢
  exact ⟨⟨h.1.1, trivial⟩, h.2⟩

/-- Inclusive validity of one sample on an ordered timeline. -/
def valid (notBefore notAfter sample : Int) : Prop :=
  notBefore ≤ sample ∧ sample ≤ notAfter

theorem expired_stays_expired (notBefore notAfter t t' : Int)
    (expired : notAfter < t) (later : t ≤ t') : ¬ valid notBefore notAfter t' := by
  intro h
  unfold valid at h
  omega

def Monotone (f : Int → Int) : Prop := ∀ a b, a ≤ b → f a ≤ f b

theorem expired_stays_expired_local (dateAt : Int → Int) (mono : Monotone dateAt)
    (firstDay lastDay t t' : Int) (expired : lastDay < dateAt t) (later : t ≤ t') :
    ¬ valid firstDay lastDay (dateAt t') := by
  intro h
  have := mono t t' later
  unfold valid at h
  omega

/-- Without monotonicity the local date can return: a midnight fall-back. -/
example : ∃ dateAt : Int → Int, ∃ t t' : Int, t ≤ t' ∧ 0 < dateAt t ∧ valid 0 0 (dateAt t') :=
  ⟨fun x => if x = 0 then 1 else 0, 0, 1, by decide, by decide, by simp [valid]⟩

example : admits true .standing [⟨7, 1⟩] [⟨0, 0⟩, ⟨1, 0⟩, ⟨7, 1⟩, ⟨9, 0⟩] = true := by decide
example : admits true .standing [⟨7, 1⟩] [⟨0, 0⟩, ⟨1, 0⟩, ⟨7, 0⟩, ⟨9, 0⟩] = false := by decide
example : admits true (.workflow 3 4) [] [⟨0, 0⟩, ⟨1, 0⟩, ⟨7, 1⟩] = false := by decide
-- A grant-kind node (ordinal 1) is not an extent ordinal.
example : admits true .standing [⟨1, 1⟩] [⟨0, 0⟩, ⟨1, 0⟩, ⟨7, 0⟩] = true := by decide

/-- Validity with optional bounds: an absent bound admits every sample on
its side, exactly as grant selection omits its predicate. -/
def validOpen (notBefore notAfter : Option Int) (sample : Int) : Prop :=
  (∀ b, notBefore = some b → b ≤ sample) ∧ (∀ a, notAfter = some a → sample ≤ a)

theorem validOpen_closed (notBefore notAfter sample : Int) :
    validOpen (some notBefore) (some notAfter) sample ↔ valid notBefore notAfter sample := by
  simp [validOpen, valid]

theorem open_above_never_expires (notBefore : Option Int) (t t' : Int)
    (current : validOpen notBefore none t) (later : t ≤ t') :
    validOpen notBefore none t' := by
  refine ⟨fun b hb => ?_, fun a ha => by simp at ha⟩
  have := current.1 b hb
  omega

theorem unbounded_always_valid (sample : Int) : validOpen none none sample := by
  simp [validOpen]

theorem expired_stays_expired_open (notBefore : Option Int) (notAfter t t' : Int)
    (expired : notAfter < t) (later : t ≤ t') : ¬ validOpen notBefore (some notAfter) t' := by
  intro h
  have := h.2 notAfter rfl
  omega

/-- On a timeline bounded by `lo` and `hi`, an absent bound admits exactly
what the extreme bound admits. -/
theorem open_bound_is_extreme (lo hi sample : Int) (inRange : lo ≤ sample ∧ sample ≤ hi)
    (notBefore notAfter : Option Int) :
    validOpen notBefore notAfter sample ↔
      validOpen (some (notBefore.getD lo)) (some (notAfter.getD hi)) sample := by
  cases notBefore <;> cases notAfter <;> simp [validOpen] <;> omega

example : validOpen (some 5) none 9 := by simp [validOpen]
example : ¬ validOpen none (some 5) 9 := by simp [validOpen]

end CapabilityGrantPath
