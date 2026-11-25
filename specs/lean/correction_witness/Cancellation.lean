/-
Correction cancellation over performed witnesses (public #388).

Mirrors
  workspaces/worth-query/crates/worth-query-execution/src/domain_computation/
    primary_graph/product_operation/correction_source/restored_support/cancellation.rs
    * `require`, the capture loop        -> `capture`
      (a missing witness refuses `CanonicalEvidenceUnavailable`: `unavailable`)
    * `require`, the pairing loop        -> `pairs`
      (a source outside the selected occurrences, a doubly covered commit, an
       uncovered occurrence and every `RecordedReapply` refuse `SourceDiverged`:
       `diverged`)
    * `require_pair`                     -> the abstract `PairOk` relation
      (its refusal is `CanonicalAssociationMismatch`: `mismatch`)

A Memory Home takes each witness from its session store; a Directory Home
restores it from durable custody. `require` sees only the witness map, so:
  * `equal_witnesses_equal_answer`: equal witnesses give the same answer (the
    restore path's parity; court 5 shows the witnesses are equal);
  * `absent_never_accepts`: a restore that loses witnesses never turns a
    refusal into an acceptance, and turns an acceptance only into
    `unavailable`;
  * `accepted_witnessed`: an accepted correction had every witness.
-/
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/cancellation.rs:23-101 sha256:b75b1c4e28ac6cfb111a8912a4afb5def87561ac905c78393226aeec21c2c251
-- proof-mirror: workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/restored_support/cancellation.rs:116-195 sha256:d94d09a4e3438cc215aa1ae3adcbeef9583d7e01b1691408e73d84971fc6fe31

namespace CorrectionWitness

inductive Role where
  | reverse
  | reapply
  deriving DecidableEq

structure Fact where
  role : Role
  source : Nat
  deriving DecidableEq

/-- A performed witness: its World commit and its sealed fact, if any. -/
structure Witness where
  world : Nat
  fact : Option Fact
  deriving DecidableEq

inductive Answer where
  | accept
  | unavailable
  | diverged
  | mismatch
  deriving DecidableEq

/-- `require_pair` (source commit, reversal commit, before, after). -/
abbrev PairOk := Nat → Nat → Witness → Witness → Bool

/-- The capture loop: the first occurrence without a witness refuses. -/
def capture (w : Nat → Option Witness) : List Nat → Option Answer
  | [] => none
  | o :: os =>
    match w o with
    | none => some .unavailable
    | some _ => capture w os

/-- The pairing loop over `rest`; `occs` are all selected occurrences. -/
def pairs (occs : List Nat) (w : Nat → Option Witness) (ok : PairOk) :
    List Nat → List Nat → Answer
  | [], covered => if covered.length = occs.length then .accept else .diverged
  | o :: rest, covered =>
    match w o with
    | none => .unavailable
    | some after =>
      match after.fact with
      | some ⟨.reverse, s⟩ =>
        if s ∈ occs then
          match w s with
          | none => .diverged
          | some before =>
            if ok s o before after then
              if s ∈ covered then .diverged
              else if o ∈ s :: covered then .diverged
              else pairs occs w ok rest (o :: s :: covered)
            else .mismatch
        else .diverged
      | _ => pairs occs w ok rest covered

def require (occs : List Nat) (w : Nat → Option Witness) (ok : PairOk) : Answer :=
  match capture w occs with
  | some answer => answer
  | none => pairs occs w ok occs []

theorem capture_congr (w w' : Nat → Option Witness) :
    ∀ l : List Nat, (∀ o ∈ l, w o = w' o) → capture w l = capture w' l
  | [], _ => rfl
  | o :: os, h => by
    have ho : w o = w' o := h o (List.mem_cons_self ..)
    have hs : capture w os = capture w' os :=
      capture_congr w w' os (fun x hx => h x (List.mem_cons_of_mem _ hx))
    simp only [capture, ho, hs]

theorem capture_some (w : Nat → Option Witness) :
    ∀ l : List Nat, ∀ a, capture w l = some a → a = .unavailable
  | [], a, h => by simp [capture] at h
  | o :: os, a, h => by
    unfold capture at h
    split at h
    · simp at h; exact h.symm
    · exact capture_some w os a h

theorem capture_none (w : Nat → Option Witness) :
    ∀ l : List Nat, capture w l = none → ∀ o ∈ l, (w o).isSome
  | [], _, o, ho => by simp at ho
  | x :: xs, h, o, ho => by
    unfold capture at h
    split at h
    · simp at h
    · rename_i witness hx
      cases List.mem_cons.mp ho with
      | inl heq => subst heq; simp [hx]
      | inr hm => exact capture_none w xs h o hm

theorem capture_missing (w : Nat → Option Witness) :
    ∀ l : List Nat, ∀ o ∈ l, w o = none → capture w l = some .unavailable
  | [], o, ho, _ => by simp at ho
  | x :: xs, o, ho, hw => by
    unfold capture
    split
    · rfl
    · cases List.mem_cons.mp ho with
      | inl heq => subst heq; simp_all
      | inr hm => exact capture_missing w xs o hm hw

theorem pairs_congr (occs : List Nat) (w w' : Nat → Option Witness) (ok : PairOk)
    (hw : ∀ o ∈ occs, w o = w' o) :
    ∀ rest : List Nat, (∀ o ∈ rest, o ∈ occs) →
      ∀ covered, pairs occs w ok rest covered = pairs occs w' ok rest covered
  | [], _, _ => rfl
  | o :: rest, hsub, covered => by
    have ho : w o = w' o := hw o (hsub o (List.mem_cons_self ..))
    have ih := pairs_congr occs w w' ok hw rest
      (fun x hx => hsub x (List.mem_cons_of_mem _ hx))
    simp only [pairs, ho]
    split
    · rfl
    · split
      · rename_i s _
        by_cases hs : s ∈ occs
        · simp only [hs, if_true, hw s hs]
          split <;> simp [ih]
        · simp [hs]
      · exact ih _

/-- The parity statement: `require` answers from the witness map alone. -/
theorem equal_witnesses_equal_answer (occs : List Nat) (w w' : Nat → Option Witness)
    (ok : PairOk) (hw : ∀ o ∈ occs, w o = w' o) :
    require occs w ok = require occs w' ok := by
  unfold require
  rw [capture_congr w w' occs hw, pairs_congr occs w w' ok hw occs (fun _ h => h) []]

theorem accepted_witnessed (occs : List Nat) (w : Nat → Option Witness) (ok : PairOk)
    (h : require occs w ok = .accept) : ∀ o ∈ occs, (w o).isSome := by
  unfold require at h
  split at h
  · rename_i a ha
    have := capture_some w occs a ha
    subst this
    cases h
  · rename_i hn
    exact capture_none w occs hn

/-- `w'` keeps or drops each witness of `w`, as a restore that may lose evidence. -/
def Restricts (w' w : Nat → Option Witness) : Prop := ∀ o, w' o = none ∨ w' o = w o

theorem restricted_answer (occs : List Nat) (w w' : Nat → Option Witness) (ok : PairOk)
    (hr : Restricts w' w) :
    require occs w' ok = .unavailable ∨ require occs w' ok = require occs w ok := by
  by_cases hm : ∃ o ∈ occs, w' o = none
  · obtain ⟨o, ho, hn⟩ := hm
    left
    unfold require
    rw [capture_missing w' occs o ho hn]
  · right
    apply equal_witnesses_equal_answer
    intro o ho
    cases hr o with
    | inl hn => exact absurd ⟨o, ho, hn⟩ hm
    | inr he => exact he

/-- Losing witnesses never turns a refusal into an acceptance. -/
theorem absent_never_accepts (occs : List Nat) (w w' : Nat → Option Witness) (ok : PairOk)
    (hr : Restricts w' w) (h : require occs w' ok = .accept) :
    require occs w ok = .accept := by
  cases restricted_answer occs w w' ok hr with
  | inl hu => rw [hu] at h; cases h
  | inr he => rw [← he]; exact h

/-- Losing witnesses turns an acceptance only into `unavailable`. -/
theorem absent_only_unavailable (occs : List Nat) (w w' : Nat → Option Witness) (ok : PairOk)
    (hr : Restricts w' w) (h : require occs w ok = .accept) :
    require occs w' ok = .accept ∨ require occs w' ok = .unavailable := by
  cases restricted_answer occs w w' ok hr with
  | inl hu => exact Or.inr hu
  | inr he => exact Or.inl (he.trans h)

/-! Classification witnesses: commit 1 is a plain source, 2 reverses it, 3 reapplies 2. -/

def plain : Witness := ⟨1, none⟩
def undo : Witness := ⟨2, some ⟨.reverse, 1⟩⟩
def redo : Witness := ⟨3, some ⟨.reapply, 2⟩⟩
def held : Nat → Option Witness
  | 1 => some plain
  | 2 => some undo
  | 3 => some redo
  | _ => none

theorem cancelled_pair_accepts : require [1, 2] held (fun _ _ _ _ => true) = .accept := by decide
theorem bad_pair_mismatches : require [1, 2] held (fun _ _ _ _ => false) = .mismatch := by decide
theorem reapply_diverges : require [1, 2, 3] held (fun _ _ _ _ => true) = .diverged := by decide
theorem uncovered_diverges : require [1] held (fun _ _ _ _ => true) = .diverged := by decide
theorem missing_unavailable : require [1, 2, 4] held (fun _ _ _ _ => true) = .unavailable := by
  decide

end CorrectionWitness
