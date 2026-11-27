-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/mod.rs:275-306 sha256:cd0151b6797ba10f08c064d79e0cc52afff227bbede38fe897a1f14587398bf8
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/mod.rs:309-322 sha256:36f70c868ab7cc1a1816ee56c0144d5756e5cd7b0c2160c3c8d38575253e3b25
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/mod.rs:324-337 sha256:7f07bb9c4cadb347befd25e36ef7f01764572db972fa862b0f30296fffea8faa
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/entry.rs:255-267 sha256:343f792c71f5ee0a09dbeb7574ddb27a052080151ca30cdbe8981fd21ff756de
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/entry.rs:292-295 sha256:89edaad849ef8188a4c5cfd04d453385a411f93b18d365cc5cdb429085b1587c
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/journal.rs:73-94 sha256:2f26aa48a39db1132293297071143b41e70cec9518b797d141df30c9002b79f0
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/journal.rs:209-280 sha256:1407223c31dcbc36fb842d99bba3e24c283b0fcef973dc120162585826a2d96d

/-!
# Name-index batch delta: rollback and step bound (Phase 7B-2, RULES §4)

Models the delta-journal design of `crates/worth-relational-lineage/src/
lineage/index/`: `PersistentNameBranchIndex::apply` and `entry_mut` /
`insert_entry` (`mod.rs`), `NameEntry::set_state` (`entry.rs`),
`IngestionJournal::save_name` / `restore` (`journal.rs`), and the member
carrier update of `member_groups.rs`.

`apply` refuses a batch whose commit `c` is at or below the head, so every
timeline entry already stored has a commit below `c`. A key's timeline is
append-only: `set_state(c, x)` overwrites the tail only when the tail has
commit `c`, which is then an entry this same batch pushed. The journal
records, per touched key and before its first change, one undo entry
(`save_name`), never a copy of the prior timeline: remove the key's delta
slot (a key the batch makes new to this tier), restore the prior shared
`Arc` (a fork-shared row), or remove the batch commit's timeline entry in
place (`rollback_commit`). Every entry at commit `c` is this batch's, so
each undo gives back exactly the prior timeline; the model records that
prior timeline's length (`none` when the key is new) and rollback truncates
to it or removes the new key, the same store on every key.

A step is one operation (a saved-set probe, an O(1) expected `HashSet`
lookup, plus one timeline write), one refused operation, or one restored
journal entry. The write is one ordered insertion into the tiered name
table and the name's `OrdMap` timeline, logarithmic in their sizes; the
model charges it one step, the design's cost model. `rollback_restores`
proves rollback after any prefix of any operation sequence gives back the
prior store exactly, on every key, and
`batch_refused_restores` that a refused batch leaves the store unchanged.
`IndexDeltaCost` proves the step bound, its independence from history, and
the group carrier update cost; each member's tenure push in a carrier delta
is one more `set` operation here.

Simplifications: keys, commits and states are natural numbers; one store
stands for both `names` timelines and `member_groups` tenures; forward-edge
lists (`add_edge`, which also writes only at the batch commit) and the
lineage binding maps are not modelled.
-/
namespace PersistentNaming.IndexDelta

/-- A key's timeline of `(commit, state)` entries. -/
abbrev Timeline := List (Nat × Nat)

abbrev Store := Nat → Option Timeline

/-- `NameEntry::set_state`: replace a tail at the same commit, else push. -/
def setState (c x : Nat) (l : Timeline) : Timeline :=
  match l.getLast? with
  | some (a, _) => if a = c then l.dropLast ++ [(c, x)] else l ++ [(c, x)]
  | none => l ++ [(c, x)]

def update (s : Store) (k : Nat) (v : Option Timeline) : Store :=
  fun q => if q = k then v else s q

/-- `set k x`: `entry_mut(k).set_state(c, x)`, refused when `k` is absent;
`insert k x`: `insert_entry`, refused when `k` exists. -/
inductive Op where
  | set (key state : Nat)
  | insert (key state : Nat)

def Op.key : Op → Nat
  | .set k _ => k
  | .insert k _ => k

structure Run where
  state : Store
  journal : List (Nat × Option Nat)
  steps : Nat
  refused : Bool

/-- `save_name`: record the prior length before the first change only. -/
def save (j : List (Nat × Option Nat)) (k : Nat) (prior : Option Nat) : List (Nat × Option Nat) :=
  if (j.lookup k).isSome then j else (k, prior) :: j

def applyOp (c : Nat) (r : Run) : Op → Run
  | .set k x =>
    match r.state k with
    | none => { r with steps := r.steps + 1, refused := true }
    | some l => { r with state := update r.state k (some (setState c x l)),
                         journal := save r.journal k (some l.length), steps := r.steps + 2 }
  | .insert k x =>
    match r.state k with
    | some _ => { r with steps := r.steps + 1, refused := true }
    | none => { r with state := update r.state k (some [(c, x)]),
                       journal := save r.journal k none, steps := r.steps + 2 }

/-- Ingestion stops at the first refusal. -/
def applyAll (c : Nat) : List Op → Run → Run
  | [], r => r
  | op :: ops, r => if r.refused then r else applyAll c ops (applyOp c r op)

def init (s : Store) : Run := ⟨s, [], 0, false⟩

/-- `restore`: truncate to the prior length, or remove a new key. -/
def rollback : List (Nat × Option Nat) → Store → Store
  | [], s => s
  | (k, p) :: rest, s =>
    rollback rest (update s k (match p with
      | some n => (s k).map (List.take n)
      | none => none))

/-- One batch: the new store and its steps, rolled back when refused. -/
def batch (c : Nat) (ops : List Op) (s : Store) : Store × Nat :=
  let r := applyAll c ops (init s)
  if r.refused then (rollback r.journal r.state, r.steps + r.journal.length)
  else (r.state, r.steps)

theorem lookup_cons (q k : Nat) (p : Option Nat) (rest : List (Nat × Option Nat)) :
    List.lookup q ((k, p) :: rest) = if q = k then some p else List.lookup q rest := by
  simp [List.lookup]; split <;> simp_all

theorem lookup_none : ∀ (j : List (Nat × Option Nat)) (k : Nat),
    k ∉ j.map Prod.fst → j.lookup k = none
  | [], _, _ => rfl
  | (a, p) :: rest, k, h => by
    simp only [List.map_cons, List.mem_cons, not_or] at h
    rw [lookup_cons]
    simp only [h.1, ↓reduceIte]
    exact lookup_none rest k h.2

theorem not_mem_of_lookup_none : ∀ (j : List (Nat × Option Nat)) (k : Nat),
    j.lookup k = none → k ∉ j.map Prod.fst
  | [], _, _ => by simp
  | (a, p) :: rest, k, h => by
    rw [lookup_cons] at h
    split at h
    · cases h
    · rename_i hne
      have := not_mem_of_lookup_none rest k h
      simp_all

theorem lookup_save_ne (j : List (Nat × Option Nat)) (k q : Nat) (p : Option Nat) (h : q ≠ k) :
    (save j k p).lookup q = j.lookup q := by
  unfold save; split
  · rfl
  · rw [lookup_cons]; simp only [h, ↓reduceIte]

/-- Overwriting a tail never reaches entries older than the batch commit. -/
theorem setState_keeps_prefix (c x : Nat) (l0 t : Timeline) (hl0 : ∀ e, e ∈ l0 → e.1 < c) :
    ∃ t', setState c x (l0 ++ t) = l0 ++ t' := by
  rcases List.eq_nil_or_concat t with rfl | ⟨T, y, rfl⟩
  · rcases List.eq_nil_or_concat l0 with rfl | ⟨L, b, rfl⟩
    · exact ⟨[(c, x)], by simp [setState]⟩
    · have hb := hl0 b (by simp)
      refine ⟨[(c, x)], ?_⟩
      simp only [List.concat_eq_append, List.append_nil, setState, List.getLast?_concat]
      simp only [show ¬b.1 = c by omega, ↓reduceIte]
  · simp only [List.concat_eq_append, setState, ← List.append_assoc, List.getLast?_concat]
    split
    · exact ⟨T ++ [(c, x)], by simp⟩
    · exact ⟨T ++ [y] ++ [(c, x)], by simp⟩

/-- What the journal says about one key, against the prior store. -/
def Agree (prior : Option Timeline) : Option (Option Nat) → Option Timeline → Prop
  | none, cur => cur = prior
  | some none, _ => prior = none
  | some (some n), cur => ∃ l0 t, prior = some l0 ∧ l0.length = n ∧ cur = some (l0 ++ t)

def Saved (s0 : Store) (r : Run) : Prop :=
  (r.journal.map Prod.fst).Nodup ∧ ∀ k, Agree (s0 k) (r.journal.lookup k) (r.state k)

theorem save_saved (s0 : Store) (r : Run) (k : Nat) (p : Option Nat) (v : Option Timeline)
    (hr : Saved s0 r) (hk : Agree (s0 k) ((save r.journal k p).lookup k) v) :
    Saved s0 { r with state := update r.state k v, journal := save r.journal k p } := by
  obtain ⟨hn, ha⟩ := hr
  refine ⟨?_, fun q => ?_⟩
  · simp only [save]; split
    · exact hn
    · rename_i h
      exact List.nodup_cons.2 ⟨not_mem_of_lookup_none r.journal k
        (by cases hl : r.journal.lookup k <;> simp_all), hn⟩
  · by_cases hq : q = k
    · subst hq; simp only [update, ↓reduceIte]; exact hk
    · simp only [update, hq, ↓reduceIte, lookup_save_ne _ _ _ _ hq]; exact ha q

section Rollback
variable (c : Nat) (s0 : Store) (hc : ∀ k l, s0 k = some l → ∀ e, e ∈ l → e.1 < c)
include hc

theorem applyOp_saved (r : Run) (hr : Saved s0 r) : ∀ op, Saved s0 (applyOp c r op)
  | .set k x => by
    simp only [applyOp]
    split
    · exact hr
    · rename_i l hl
      apply save_saved _ _ _ _ _ hr
      have hak := hr.2 k
      unfold save
      split
      · rename_i hj
        cases hjk : r.journal.lookup k with
        | none => simp [hjk] at hj
        | some p =>
          cases p with
          | none => rw [hjk] at hak; exact hak
          | some n =>
            rw [hjk] at hak
            obtain ⟨l0, t, h0, hn, hcur⟩ := hak
            rw [hl] at hcur
            injection hcur with hcur
            obtain ⟨t', ht'⟩ := setState_keeps_prefix c x l0 t (hc k l0 h0)
            exact ⟨l0, t', h0, hn, by rw [hcur, ht']⟩
      · rename_i hj
        have hjk : r.journal.lookup k = none := by cases h : r.journal.lookup k <;> simp_all
        rw [hjk] at hak
        rw [lookup_cons]
        simp only [↓reduceIte]
        have h0 : s0 k = some l := by rw [← hak]; exact hl
        obtain ⟨t', ht'⟩ := setState_keeps_prefix c x l [] (hc k l h0)
        exact ⟨l, t', h0, rfl, by rw [← ht', List.append_nil]⟩
  | .insert k x => by
    simp only [applyOp]
    split
    · exact hr
    · rename_i hl
      apply save_saved _ _ _ _ _ hr
      have hak := hr.2 k
      unfold save
      split
      · rename_i hj
        cases hjk : r.journal.lookup k with
        | none => simp [hjk] at hj
        | some p =>
          cases p with
          | none => rw [hjk] at hak; exact hak
          | some n =>
            rw [hjk] at hak
            obtain ⟨l0, t, _, _, hcur⟩ := hak
            rw [hl] at hcur
            cases hcur
      · rename_i hj
        have hjk : r.journal.lookup k = none := by cases h : r.journal.lookup k <;> simp_all
        rw [hjk] at hak
        rw [lookup_cons]
        simp only [↓reduceIte, Agree]
        rw [← hak]; exact hl

theorem applyAll_saved : ∀ (ops : List Op) (r : Run), Saved s0 r → Saved s0 (applyAll c ops r)
  | [], _, hr => hr
  | op :: ops, r, hr => by
    simp only [applyAll]
    split
    · exact hr
    · exact applyAll_saved ops _ (applyOp_saved c s0 hc r hr op)

end Rollback

theorem rollback_correct (s0 : Store) : ∀ (j : List (Nat × Option Nat)) (st : Store),
    (j.map Prod.fst).Nodup → (∀ k, Agree (s0 k) (j.lookup k) (st k)) → rollback j st = s0
  | [], st, _, ha => funext fun k => ha k
  | (k, p) :: rest, st, hn, ha => by
    have hn' := List.nodup_cons.1 (show (k :: rest.map Prod.fst).Nodup from hn)
    simp only [rollback]
    apply rollback_correct s0 rest _ hn'.2
    intro q
    by_cases hq : q = k
    · subst hq
      rw [lookup_none rest q hn'.1]
      have hak := ha q
      rw [lookup_cons] at hak
      simp only [↓reduceIte] at hak
      simp only [update, ↓reduceIte, Agree]
      cases p with
      | none => exact hak.symm
      | some n =>
        obtain ⟨l0, t, h0, rfl, hcur⟩ := hak
        simp [hcur, h0]
    · have hak := ha q
      rw [lookup_cons] at hak
      simp only [hq, ↓reduceIte] at hak
      simp only [update, hq, ↓reduceIte]
      exact hak

/-- Rollback after any prefix of any operation sequence gives back the
prior store, on every key. -/
theorem rollback_restores (c : Nat) (ops : List Op) (s : Store)
    (hc : ∀ k l, s k = some l → ∀ e, e ∈ l → e.1 < c) :
    rollback (applyAll c ops (init s)).journal (applyAll c ops (init s)).state = s := by
  have h := applyAll_saved c s hc ops (init s) ⟨List.nodup_nil, fun _ => rfl⟩
  exact rollback_correct s _ _ h.1 h.2

/-- A refused batch leaves the store exactly as it was. -/
theorem batch_refused_restores (c : Nat) (ops : List Op) (s : Store)
    (hc : ∀ k l, s k = some l → ∀ e, e ∈ l → e.1 < c)
    (hrefused : (applyAll c ops (init s)).refused = true) : (batch c ops s).1 = s := by
  simp only [batch, hrefused, ↓reduceIte]
  exact rollback_restores c ops s hc

end PersistentNaming.IndexDelta
