-- proof-mirror: crates/worth-relational-branch/src/runtime/state/subsystems/indexing/catalog_coverage.rs:11-13 sha256:c1f568858b8c14f158517799ff6aa908bdeb51c6d6f1440e3297b438b4615054
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/definition_catalog.rs:71-126 sha256:31b5f2dfcdda365afcb7a63d04d9c164df97b46c426d19870453f9ffe670a5fe
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/definition_catalog.rs:156-165 sha256:6945457fd7708b7e1af8535bf89a1a529686ac516aa44f1fb5cc306ea04c45b9
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/definition_catalog.rs:183-214 sha256:ed830e1b4ed0658b1ebf274856db03de800a00b63462e87693b8b47e1d28aa2a
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/catalog_publication.rs:39-54 sha256:dcc0211bf6d6ad7e86f91d488872bb6fba0c64675a47d9d2c59bb8b87dc3ca1d
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/catalog_publication.rs:58-67 sha256:d949333acde29578902e43ffdbf0201652ec263c92e88aee4e348c55d23817b3
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/catalog_publication.rs:80-104 sha256:06f7ca5b68de1de50d5e29f479ace3affa4ffbca673f7548dc9852e8bb8433ae
-- proof-mirror: crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/catalog_publication.rs:105-129 sha256:96fffefb28e8f3ed59ee2d2803b7c4f53542e8d1ad3ed09b872a0b382afbf237
-- proof-mirror: crates/worth-relational-publication/src/authority/commit/pipeline/artifact_execution/authored_index_writes.rs:22-88 sha256:1b595631b231928db314206acce5f266266e3eada3bf19bdab30550ba0a72df6
-- proof-mirror: crates/worth-relational-publication/src/runtime/state/subsystems/canonical_publication_routes/authored_writes/indexed.rs:12-85 sha256:0c183dd0aa3ffb4c92c754727661d90f40f863edea6f08001e821205b079b8db
-- proof-mirror: crates/worth-relational-branch/src/history/access/relevant_writes/cursor/admission.rs:75-249 sha256:92e04c15b717f08fae5f5d7e1f8b224d25e0b16745d63d92b9aa4107b3f0b573
/-
Authored index-write coverage across checkpoint restore and WAL replay, unbounded
(public #384).

Mirrors
  crates/worth-relational-branch/src/runtime/state/subsystems/indexing/catalog_coverage.rs
    * `IndexCatalogCoverage::extends`          -> `Cov.ext` (verbatim)
  crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/definition_catalog.rs
    * `IndexDefinitionCatalog::default`        -> `fresh` (a fresh incarnation, epoch 0)
    * `IndexDefinitionCatalog::insert`         -> `Catalog.insert` (the epoch step and
      `introductions` update; the routing and roster maps are not read here)
    * `IndexDefinitionCatalog::coverage_before` -> `Catalog.coverageBefore`
      (`introductions.range(..next).next_back()` is the downward scan `lastBelow`)
    * `IndexDefinitionCatalog::introduction`   -> `Catalog.introduction`
  crates/worth-relational-indexes/src/runtime/state/subsystems/indexing/catalog_publication.rs
    * `readmit_catalog_delta`, `readmit_checkpoint_catalog` and live registration
      (definition_publication.rs) insert only the next issued id: `Catalog.register`.
      `readmit_checkpoint_catalog` refuses a non-fresh catalog, so a restore is
      `regN (W - 1) (fresh inc)`.
    * `CapturedIndexDefinitionCatalog::certify_readmitted_writes` -> `certify`
  crates/worth-relational-publication/src/authority/commit/pipeline/artifact_execution/
    index_catalog.rs (replay capture) and authored_index_writes.rs (`prepare`)
                                               -> `replayMint`
  crates/worth-relational-publication/src/runtime/state/subsystems/
    canonical_publication_routes/authored_writes/indexed.rs (`append`)
                                               -> `Gen.append`
  crates/worth-relational-branch/src/history/access/relevant_writes/cursor/admission.rs
    (the incomplete-index refusal, `index_run_start` and
     `index_coverage.first_at_or_after(next)`) -> `admitIndex`

Ordinal convention: the model numbers commits from 0 and `Gen.ordinal` counts the
appended commits, so the Rust `next > generation.ordinal` (the last appended
ordinal) is `g.ordinal ≤ next` here.

Results
  * `epoch_mono_of_ordered_prefix`  ordered insertion makes the epoch strictly
    monotone in id.
  * `coverageBefore_regN_stable`    prefix stability: appending ids `≥ W` never
    changes `coverageBefore W`.
  * `never_covers_later`            certified coverage never extends the
    introduction of an index registered at or after the commit's frontier.
  * `admit_compression_sound`       the production compression (incomplete set,
    run start, first coverage at or after `next`) proves every ordinal of the
    span covered by a coverage that extends the introduction.
  * `admit_sound`                   an admitted span holds the writes of every commit.
  * `legacy_refuses`, `legacy_in_span_refuses`, `mismatch_refuses`.
  * `stale_incarnation_refuses`     coverage of another incarnation never admits.
  * `reopen_eq_replay`              certify and the replay mint agree on every packet.
  * `admit_split_eq_replay`         for every split point `k` between checkpoint
    (certified) and WAL tail (replay-minted) the minted coverage and so every
    admission outcome are identical.
  * negative controls `mutant_*`: each mutant provably breaks its theorem.
-/
set_option linter.unusedVariables false

namespace Coverage

/-! ## Coverage and catalogs -/

structure Cov where
  inc : Nat
  epoch : Nat
deriving DecidableEq, Repr

/-- `IndexCatalogCoverage::extends`. -/
def Cov.ext (self prior : Cov) : Prop := self.inc = prior.inc ∧ prior.epoch ≤ self.epoch

instance (a b : Cov) : Decidable (a.ext b) := inferInstanceAs (Decidable (_ ∧ _))

theorem Cov.ext_refl (a : Cov) : a.ext a := ⟨rfl, Nat.le_refl _⟩

theorem Cov.ext_trans {a b c : Cov} (h1 : a.ext b) (h2 : b.ext c) : a.ext c :=
  ⟨h1.1.trans h2.1, Nat.le_trans h2.2 h1.2⟩

structure Catalog where
  inc : Nat
  epoch : Nat
  intro : Nat → Option Nat
  next : Nat

def fresh (inc : Nat) : Catalog := ⟨inc, 0, fun _ => none, 1⟩

def Catalog.insert (c : Catalog) (id : Nat) : Catalog :=
  { c with epoch := c.epoch + 1,
           intro := fun d => if d = id then some (c.epoch + 1) else c.intro d }

/-- Every installation path inserts exactly the next issued id. -/
def Catalog.register (c : Catalog) : Catalog := { c.insert c.next with next := c.next + 1 }

def Catalog.regN : Nat → Catalog → Catalog
  | 0, c => c
  | n + 1, c => (Catalog.regN n c).register

def lastBelow (f : Nat → Option Nat) : Nat → Nat
  | 0 => 0
  | w + 1 => (f w).getD (lastBelow f w)

def Catalog.coverageBefore (c : Catalog) (next : Nat) : Cov := ⟨c.inc, lastBelow c.intro next⟩

def Catalog.introduction (c : Catalog) (d : Nat) : Option Cov := (c.intro d).map (Cov.mk c.inc)

def Ordered (c : Catalog) : Prop :=
  (∀ d x, c.intro d = some x → 1 ≤ x ∧ x ≤ c.epoch ∧ d < c.next) ∧
  (∀ d e x y, c.intro d = some x → c.intro e = some y → d < e → x < y)

theorem ordered_fresh (inc : Nat) : Ordered (fresh inc) := by
  constructor <;> intros <;> simp_all [fresh]

theorem ordered_register {c : Catalog} (h : Ordered c) : Ordered c.register := by
  obtain ⟨h1, h2⟩ := h
  constructor
  · intro d x hd
    simp only [Catalog.register, Catalog.insert] at hd ⊢
    by_cases hdn : d = c.next
    · simp only [hdn, ite_true, Option.some.injEq] at hd; omega
    · simp only [hdn, ite_false] at hd; have := h1 d x hd; omega
  · intro d e x y hd he hde
    simp only [Catalog.register, Catalog.insert] at hd he
    by_cases hdn : d = c.next <;> by_cases hen : e = c.next
    · omega
    · simp only [hen, ite_false] at he; have := h1 e y he; omega
    · simp only [hdn, ite_false] at hd; simp only [hen, ite_true, Option.some.injEq] at he
      have := h1 d x hd; omega
    · simp only [hdn, ite_false] at hd; simp only [hen, ite_false] at he; exact h2 d e x y hd he hde

theorem ordered_regN {c : Catalog} (h : Ordered c) : ∀ n, Ordered (Catalog.regN n c)
  | 0 => h
  | n + 1 => ordered_register (ordered_regN h n)

theorem regN_inc (c : Catalog) : ∀ n, (Catalog.regN n c).inc = c.inc
  | 0 => rfl
  | n + 1 => regN_inc c n

theorem regN_next (c : Catalog) : ∀ n, (Catalog.regN n c).next = c.next + n
  | 0 => rfl
  | n + 1 => by simp only [Catalog.regN, Catalog.register, regN_next c n]; omega

/-- Ordered insertion of the issued prefix makes the epoch strictly monotone in id. -/
theorem epoch_mono_of_ordered_prefix (inc n d e x y : Nat)
    (hd : (Catalog.regN n (fresh inc)).intro d = some x)
    (he : (Catalog.regN n (fresh inc)).intro e = some y) (hde : d < e) : x < y :=
  (ordered_regN (ordered_fresh inc) n).2 d e x y hd he hde

/-! ## `coverage_before` -/

theorem lastBelow_cases (f : Nat → Option Nat) :
    ∀ w, lastBelow f w = 0 ∨ ∃ e, e < w ∧ f e = some (lastBelow f w)
  | 0 => Or.inl rfl
  | w + 1 => by
    simp only [lastBelow]
    cases h : f w with
    | some x => exact Or.inr ⟨w, by omega, by simp [h]⟩
    | none =>
      rcases lastBelow_cases f w with h0 | ⟨e, he, hfe⟩
      · exact Or.inl (by simp [h0])
      · exact Or.inr ⟨e, by omega, by simp [hfe]⟩

theorem lastBelow_congr (f g : Nat → Option Nat) :
    ∀ w, (∀ e, e < w → f e = g e) → lastBelow f w = lastBelow g w
  | 0, _ => rfl
  | w + 1, h => by
    simp only [lastBelow, h w (by omega), lastBelow_congr f g w (fun e he => h e (by omega))]

/-- Every id at or above `w` was introduced after everything `coverageBefore w` reads. -/
theorem lastBelow_lt {c : Catalog} (h : Ordered c) {d x w : Nat}
    (hd : c.intro d = some x) (hw : w ≤ d) : lastBelow c.intro w < x := by
  rcases lastBelow_cases c.intro w with h0 | ⟨e, he, hfe⟩
  · have := (h.1 d x hd).1; omega
  · exact h.2 e d _ x hfe hd (by omega)

/-- Prefix stability: registering an id `≥ w` leaves `coverageBefore w` unchanged. -/
theorem coverageBefore_register_stable {c : Catalog} {w : Nat} (hw : w ≤ c.next) :
    c.register.coverageBefore w = c.coverageBefore w := by
  simp only [Catalog.coverageBefore, Catalog.register, Catalog.insert, Cov.mk.injEq, true_and]
  exact lastBelow_congr _ _ w (fun e he => by simp only [show e ≠ c.next by omega, ite_false])

theorem coverageBefore_regN_stable {c : Catalog} {w : Nat} (hw : w ≤ c.next) :
    ∀ n, (Catalog.regN n c).coverageBefore w = c.coverageBefore w
  | 0 => rfl
  | n + 1 => by
    simp only [Catalog.regN]
    rw [coverageBefore_register_stable (by rw [regN_next]; omega)]
    exact coverageBefore_regN_stable hw n

/-! ## Packets, certification and the replay mint -/

inductive Packet where
  | unavail
  | legacy
  | covered (b : Nat)
deriving DecidableEq

/-- `W` is the commit's persisted `index_definition_delta.next_index_id`. -/
structure Commit where
  W : Nat
  packet : Packet

/-- `certify_readmitted_writes`: coverage only for a persisted bound equal to the
delta frontier and inside the installed prefix. -/
def certify (cat : Catalog) (c : Commit) : Option Cov :=
  match c.packet with
  | .covered b => if b = c.W ∧ b ≤ cat.next then some (cat.coverageBefore b) else none
  | _ => none

/-- WAL replay (§4.2): `Unavailable` and legacy stay unprovenanced; a `Covered`
original is re-prepared against `coverage_before(original.next_index_id)` after
the installed-prefix check, and must equal the original (bound and entries). -/
def replayMint (cat : Catalog) (c : Commit) : Option Cov :=
  match c.packet with
  | .covered b => if c.W ≤ cat.next ∧ b = c.W then some (cat.coverageBefore c.W) else none
  | _ => none

theorem certify_some {cat : Catalog} {c : Commit} {cov : Cov} (h : certify cat c = some cov) :
    c.packet = .covered c.W ∧ c.W ≤ cat.next ∧ cov = cat.coverageBefore c.W := by
  unfold certify at h
  split at h
  · rename_i b hp
    by_cases hc : b = c.W ∧ b ≤ cat.next
    · rw [if_pos hc, Option.some.injEq] at h
      obtain ⟨rfl, hle⟩ := hc
      exact ⟨hp, hle, h.symm⟩
    · simp only [hc, ite_false] at h; cases h
  · cases h

theorem reopen_eq_replay (cat : Catalog) (c : Commit) : certify cat c = replayMint cat c := by
  unfold certify replayMint
  split
  · rename_i b _
    by_cases hb : b = c.W
    · subst hb; simp only [true_and, and_true]
    · simp only [hb, false_and, and_false, ite_false]
  · rfl

theorem legacy_refuses (cat : Catalog) (c : Commit) (h : c.packet = .unavail ∨ c.packet = .legacy) :
    certify cat c = none ∧ replayMint cat c = none := by
  rcases h with h | h <;> simp [certify, replayMint, h]

theorem mismatch_refuses (cat : Catalog) (c : Commit) (b : Nat) (hp : c.packet = .covered b)
    (hb : b ≠ c.W) : certify cat c = none ∧ replayMint cat c = none := by
  simp [certify, replayMint, hp, hb]

/-- Coverage certified at `cat` never extends the introduction (at any later
receiving catalog `regN j cat`) of an index registered at or after `c.W`. -/
theorem never_covers_later {cat : Catalog} (hord : Ordered cat) (j : Nat) {c : Commit}
    {cov i : Cov} {d : Nat} (hc : certify cat c = some cov)
    (hi : (Catalog.regN j cat).introduction d = some i) (hext : cov.ext i) : d < c.W := by
  obtain ⟨_, hW, rfl⟩ := certify_some hc
  simp only [Catalog.introduction, Option.map_eq_some_iff] at hi
  obtain ⟨x, hx, rfl⟩ := hi
  rw [← coverageBefore_regN_stable hW j] at hext
  apply Nat.lt_of_not_le
  intro hle
  have := lastBelow_lt (ordered_regN hord j) hx hle
  simp only [Cov.ext, Catalog.coverageBefore] at hext
  omega

theorem stale_incarnation_not_ext {cov i : Cov} {rc : Catalog} {d : Nat}
    (hi : rc.introduction d = some i) (hstale : cov.inc ≠ rc.inc) : ¬ cov.ext i := by
  simp only [Catalog.introduction, Option.map_eq_some_iff] at hi
  obtain ⟨_, _, rfl⟩ := hi
  exact fun h => hstale h.1

/-! ## The generation and admission -/

structure Gen where
  ordinal : Nat
  incomplete : Nat → Bool
  cov : Nat → Option Cov
  runStart : Nat
  runLast : Option Cov

def Gen.empty : Gen := ⟨0, fun _ => false, fun _ => none, 0, none⟩

/-- `indexed::append`: an uncovered ordinal joins the incomplete set; a covered one
restarts the run unless it extends the previous run coverage. -/
def Gen.append (g : Gen) : Option Cov → Gen
  | none => { g with ordinal := g.ordinal + 1,
                     incomplete := fun o => if o = g.ordinal then true else g.incomplete o }
  | some c =>
    { g with ordinal := g.ordinal + 1,
             cov := fun o => if o = g.ordinal then some c else g.cov o,
             runStart := match g.runLast with
               | some prior => if c.ext prior then g.runStart else g.ordinal
               | none => g.ordinal,
             runLast := some c }

def buildN (m : Nat → Option Cov) : Nat → Gen
  | 0 => Gen.empty
  | n + 1 => (buildN m n).append (m n)

/-- The meaning of `first_at_or_after` over the `k` ordinals from `s`. -/
def scan (f : Nat → Option Cov) (s : Nat) : Nat → Option Cov
  | 0 => none
  | k + 1 => match f s with
    | some c => some c
    | none => scan f (s + 1) k

def anyFrom (f : Nat → Bool) (s : Nat) : Nat → Bool
  | 0 => false
  | k + 1 => f s || anyFrom f (s + 1) k

inductive Refusal where
  | incompleteIndexHistory
deriving DecidableEq

/-- admission.rs: one `EntityFieldIndex` selector for index `d` over the open
interval from ordinal `next`. -/
def admitIndex (g : Gen) (rc : Catalog) (d next : Nat) : Except Refusal Unit :=
  if anyFrom g.incomplete next (g.ordinal - next) then .error .incompleteIndexHistory else
  match rc.introduction d with
  | none => .error .incompleteIndexHistory
  | some i =>
    if g.ordinal ≤ next then .ok () else
    if next < g.runStart then .error .incompleteIndexHistory else
    match scan g.cov next (g.ordinal - next) with
    | none => .error .incompleteIndexHistory
    | some c => if c.ext i then .ok () else .error .incompleteIndexHistory

theorem buildN_ordinal (m : Nat → Option Cov) : ∀ n, (buildN m n).ordinal = n
  | 0 => rfl
  | n + 1 => by
    simp only [buildN]
    cases m n <;> simp [Gen.append, buildN_ordinal m n]

theorem buildN_cov (m : Nat → Option Cov) :
    ∀ n o, (buildN m n).cov o = if o < n then m o else none
  | 0, o => by simp [buildN, Gen.empty]
  | n + 1, o => by
    simp only [buildN]
    cases h : m n with
    | none =>
      simp only [Gen.append, buildN_cov m n o]
      by_cases ho : o = n
      · subst ho; simp [h]
      · by_cases hlt : o < n
        · simp [hlt, show o < n + 1 by omega]
        · simp [hlt, show ¬ o < n + 1 by omega]
    | some c =>
      simp only [Gen.append, buildN_ordinal, buildN_cov m n o]
      by_cases ho : o = n
      · subst ho; simp [h]
      · by_cases hlt : o < n
        · simp [ho, hlt, show o < n + 1 by omega]
        · simp [ho, hlt, show ¬ o < n + 1 by omega]

theorem buildN_incomplete (m : Nat → Option Cov) :
    ∀ n o, (buildN m n).incomplete o = true → o < n ∧ m o = none
  | 0, o, h => by simp [buildN, Gen.empty] at h
  | n + 1, o, h => by
    simp only [buildN] at h
    cases hm : m n with
    | none =>
      rw [hm] at h
      simp only [Gen.append, buildN_ordinal] at h
      by_cases ho : o = n
      · subst ho; exact ⟨by omega, hm⟩
      · simp only [ho, ite_false] at h
        have := buildN_incomplete m n o h; exact ⟨by omega, this.2⟩
    | some c =>
      rw [hm] at h
      simp only [Gen.append] at h
      have := buildN_incomplete m n o h; exact ⟨by omega, this.2⟩

theorem buildN_complete (m : Nat → Option Cov) :
    ∀ n o, o < n → m o = none → (buildN m n).incomplete o = true
  | 0, o, ho, _ => absurd ho (Nat.not_lt_zero _)
  | n + 1, o, ho, hm => by
    simp only [buildN]
    cases hmn : m n with
    | none =>
      simp only [Gen.append, buildN_ordinal]
      by_cases hon : o = n
      · simp [hon]
      · simp only [hon, ite_false]; exact buildN_complete m n o (by omega) hm
    | some c =>
      have : o ≠ n := fun e => by subst e; rw [hm] at hmn; cases hmn
      simp only [Gen.append]; exact buildN_complete m n o (by omega) hm

/-- The run invariant `indexed::append` maintains. -/
def RunInv (g : Gen) : Prop :=
  (∀ o x, g.runStart ≤ o → g.cov o = some x → ∃ r, g.runLast = some r ∧ r.ext x) ∧
  (∀ o p x y, g.runStart ≤ o → o ≤ p → g.cov o = some x → g.cov p = some y → y.ext x)

theorem buildN_runInv (m : Nat → Option Cov) : ∀ n, RunInv (buildN m n)
  | 0 => ⟨fun _ _ _ h => by simp [buildN, Gen.empty] at h,
          fun _ _ _ _ _ _ h => by simp [buildN, Gen.empty] at h⟩
  | n + 1 => by
    have ih := buildN_runInv m n
    have hcov := buildN_cov m n
    have hord := buildN_ordinal m n
    simp only [buildN]
    generalize hg : buildN m n = g at ih hcov hord
    cases m n with
    | none => exact ih
    | some c =>
      have later : ∀ o, n ≤ o → g.cov o = none := fun o ho => by
        rw [hcov o]; simp [show ¬ o < n by omega]
      by_cases hrun : ∃ r, g.runLast = some r ∧ c.ext r
      · obtain ⟨r, hr, hcr⟩ := hrun
        have hstart : (g.append (some c)).runStart = g.runStart := by
          simp [Gen.append, hr, hcr]
        constructor
        · intro o x ho hx
          rw [hstart] at ho
          simp only [Gen.append, hord] at hx ⊢
          refine ⟨c, rfl, ?_⟩
          by_cases hon : o = n
          · simp only [hon, ite_true, Option.some.injEq] at hx; subst hx; exact Cov.ext_refl _
          · simp only [hon, ite_false] at hx
            obtain ⟨r', hr', hrx⟩ := ih.1 o x ho hx
            rw [hr] at hr'; cases hr'; exact Cov.ext_trans hcr hrx
        · intro o p x y ho hop hx hy
          rw [hstart] at ho
          simp only [Gen.append, hord] at hx hy
          by_cases hpn : p = n
          · simp only [hpn, ite_true, Option.some.injEq] at hy; subst hy
            by_cases hon : o = n
            · simp only [hon, ite_true, Option.some.injEq] at hx; subst hx; exact Cov.ext_refl _
            · simp only [hon, ite_false] at hx
              obtain ⟨r', hr', hrx⟩ := ih.1 o x ho hx
              rw [hr] at hr'; cases hr'; exact Cov.ext_trans hcr hrx
          · simp only [hpn, ite_false] at hy
            have hon : o ≠ n := fun e => by
              subst e; rw [later p (by omega)] at hy; cases hy
            simp only [hon, ite_false] at hx
            exact ih.2 o p x y ho hop hx hy
      · have hstart : (g.append (some c)).runStart = n := by
          simp only [Gen.append, hord]
          cases hr : g.runLast with
          | none => rfl
          | some r =>
            have : ¬ c.ext r := fun h => hrun ⟨r, hr, h⟩
            simp [this]
        constructor
        · intro o x ho hx
          rw [hstart] at ho
          simp only [Gen.append, hord] at hx ⊢
          refine ⟨c, rfl, ?_⟩
          by_cases hon : o = n
          · simp only [hon, ite_true, Option.some.injEq] at hx; subst hx; exact Cov.ext_refl _
          · simp only [hon, ite_false] at hx; rw [later o ho] at hx; cases hx
        · intro o p x y ho hop hx hy
          rw [hstart] at ho
          simp only [Gen.append, hord] at hx hy
          by_cases hon : o = n
          · by_cases hpn : p = n
            · simp only [hon, hpn, ite_true, Option.some.injEq] at hx hy; subst hx; subst hy
              exact Cov.ext_refl _
            · simp only [hpn, ite_false] at hy; rw [later p (by omega)] at hy; cases hy
          · simp only [hon, ite_false] at hx; rw [later o ho] at hx; cases hx

theorem anyFrom_false {f : Nat → Bool} :
    ∀ {s k}, anyFrom f s k = false → ∀ o, s ≤ o → o < s + k → f o = false
  | _, 0, _, o, h1, h2 => absurd h2 (by omega)
  | s, k + 1, h, o, h1, h2 => by
    simp only [anyFrom, Bool.or_eq_false_iff] at h
    by_cases hos : o = s
    · subst hos; exact h.1
    · exact anyFrom_false h.2 o (by omega) (by omega)

theorem anyFrom_true {f : Nat → Bool} :
    ∀ {s k o}, s ≤ o → o < s + k → f o = true → anyFrom f s k = true
  | _, 0, _, h1, h2, _ => absurd h2 (by omega)
  | s, k + 1, o, h1, h2, h => by
    simp only [anyFrom, Bool.or_eq_true]
    by_cases hos : o = s
    · subst hos; exact Or.inl h
    · exact Or.inr (anyFrom_true (by omega) (by omega) h)

/-- The production compression is sound: an admitted non-empty span is covered at
every ordinal, each coverage extending the receiving catalog's introduction. -/
theorem admit_compression_sound (m : Nat → Option Cov) (n : Nat) (rc : Catalog)
    (d next : Nat) (h : admitIndex (buildN m n) rc d next = .ok ()) :
    ∀ o, next ≤ o → o < n → ∃ y i, m o = some y ∧ rc.introduction d = some i ∧ y.ext i := by
  intro o ho hon
  have hord := buildN_ordinal m n
  unfold admitIndex at h
  rw [hord] at h
  split at h
  · cases h
  rename_i hinc
  have hall : ∀ q, next ≤ q → q < n → ∃ y, m q = some y := by
    intro q hq1 hq2
    cases hmq : m q with
    | some y => exact ⟨y, rfl⟩
    | none =>
      have := anyFrom_true (f := (buildN m n).incomplete) (k := n - next) hq1 (by omega)
        (buildN_complete m n q hq2 hmq)
      simp_all
  split at h
  · cases h
  rename_i i hi
  split at h
  · omega
  split at h
  · cases h
  rename_i hstart
  obtain ⟨y0, hy0⟩ := hall next (Nat.le_refl _) (by omega)
  have hscan : scan (buildN m n).cov next (n - next) = some y0 := by
    obtain ⟨k, hk⟩ : ∃ k, n - next = k + 1 := ⟨n - next - 1, by omega⟩
    rw [hk]; simp only [scan, buildN_cov, show next < n by omega, ite_true, hy0]
  rw [hscan] at h
  simp only at h
  split at h
  · rename_i hext
    obtain ⟨y, hy⟩ := hall o ho hon
    refine ⟨y, i, hy, hi, Cov.ext_trans ?_ hext⟩
    exact (buildN_runInv m n).2 next o y0 y (by omega) ho
      (by rw [buildN_cov]; simp [show next < n by omega, hy0])
      (by rw [buildN_cov]; simp [hon, hy])
  · cases h

/-- A `Covered` packet whose bound equals its delta frontier was written by the
preparation owner from the same captured catalog: it holds the writes of every
index below the bound (the digest-covered trust boundary). -/
def WellIssued (writes : Nat → Prop) (c : Commit) : Prop :=
  ∀ b, c.packet = .covered b → b = c.W → ∀ d, d < b → writes d

/-- Soundness of an admitted span seeded at `cat` and admitted at any later
receiving catalog of the same incarnation. -/
theorem admit_sound (cat : Catalog) (hord : Ordered cat) (j : Nat) (hs : Nat → Commit)
    (writes : Nat → Nat → Prop) (hissued : ∀ o, WellIssued (writes o) (hs o))
    (n d next : Nat)
    (h : admitIndex (buildN (fun o => certify cat (hs o)) n) (Catalog.regN j cat) d next = .ok ()) :
    ∀ o, next ≤ o → o < n → writes o d := by
  intro o ho hon
  obtain ⟨y, i, hy, hi, hext⟩ := admit_compression_sound _ n _ d next h o ho hon
  have hlt := never_covers_later hord j hy hi hext
  exact hissued o (hs o).W (certify_some hy).1 rfl d hlt

theorem legacy_in_span_refuses (m : Nat → Option Cov) (n : Nat) (rc : Catalog) (d next o : Nat)
    (hm : m o = none) (h1 : next ≤ o) (h2 : o < n) :
    admitIndex (buildN m n) rc d next = .error .incompleteIndexHistory := by
  unfold admitIndex
  rw [anyFrom_true h1 (by rw [buildN_ordinal]; omega) (buildN_complete m n o h2 hm)]
  rfl

/-- Coverage minted under another incarnation (a catalog replacement renewed it)
never admits a non-empty span. -/
theorem stale_incarnation_refuses (m : Nat → Option Cov) (n : Nat) (rc : Catalog) (d next : Nat)
    (hstale : ∀ o y, m o = some y → y.inc ≠ rc.inc) (hspan : next < n) :
    admitIndex (buildN m n) rc d next ≠ .ok () := by
  intro h
  obtain ⟨y, i, hy, hi, hext⟩ := admit_compression_sound m n rc d next h next (Nat.le_refl _) hspan
  exact stale_incarnation_not_ext hi (hstale next y hy) hext

/-! ## Checkpoint and WAL-tail recovery agree at every split -/

/-- Split `k`: ordinals below `k` were restored from the checkpoint and are
certified at the final catalog; the rest are WAL-replayed against the
catalog installed at their replay. `k = 0` is all-WAL; `k ≥ n` is all-checkpoint. -/
def mintSplit (catF : Catalog) (cats : Nat → Catalog) (hs : Nat → Commit) (k o : Nat) :
    Option Cov :=
  if o < k then certify catF (hs o) else replayMint (cats o) (hs o)

/-- The final catalog extends each replay-time catalog by registrations (WAL
delta readmission, post-reopen registration), and replay readmits a commit's
delta before preparing it. -/
theorem admit_split_eq_replay (catF : Catalog) (cats : Nat → Catalog) (hs : Nat → Commit)
    (ext : Nat → Nat) (hext : ∀ o, catF = Catalog.regN (ext o) (cats o))
    (hreplay : ∀ o, (hs o).W ≤ (cats o).next) (k : Nat) :
    mintSplit catF cats hs k = mintSplit catF cats hs 0 ∧
    ∀ n rc d next, admitIndex (buildN (mintSplit catF cats hs k) n) rc d next =
      admitIndex (buildN (mintSplit catF cats hs 0) n) rc d next := by
  have hpt : mintSplit catF cats hs k = mintSplit catF cats hs 0 := by
    funext o
    simp only [mintSplit, Nat.not_lt_zero, ite_false]
    split
    · rw [reopen_eq_replay, hext o]
      unfold replayMint
      split
      · simp only [regN_next]
        rw [coverageBefore_regN_stable (hreplay o)]
        have := hreplay o
        by_cases hb : (hs o).W ≤ (cats o).next + ext o
        · simp [hb, this]
        · omega
      · rfl
    · rfl
  exact ⟨hpt, fun _ _ _ _ => by rw [hpt]⟩

/-! ## Negative controls: each mutant breaks its named theorem -/

/-- Mutant: certify mints from the reopen-time `next` instead of the bound. -/
def certifyFromNext (cat : Catalog) (c : Commit) : Option Cov :=
  match c.packet with
  | .covered b => if b = c.W ∧ b ≤ cat.next then some (cat.coverageBefore cat.next) else none
  | _ => none

theorem mutant_next_breaks_never_covers_later :
    ∃ cat c cov i d, Ordered cat ∧ certifyFromNext cat c = some cov ∧
      cat.introduction d = some i ∧ cov.ext i ∧ ¬ d < c.W :=
  ⟨Catalog.regN 2 (fresh 0), ⟨2, .covered 2⟩, ⟨0, 2⟩, ⟨0, 2⟩, 2,
    ordered_regN (ordered_fresh 0) 2, by decide, by decide, by decide, by decide⟩

/-- Mutant: certify accepts a legacy packet with the bound taken from `W_c`. -/
def certifyLegacyW (cat : Catalog) (c : Commit) : Option Cov :=
  match c.packet with
  | .covered b => if b = c.W ∧ b ≤ cat.next then some (cat.coverageBefore b) else none
  | .legacy => if c.W ≤ cat.next then some (cat.coverageBefore c.W) else none
  | .unavail => none

theorem mutant_legacy_breaks_legacy_refuses :
    ∃ cat c, c.packet = .legacy ∧ certifyLegacyW cat c ≠ none :=
  ⟨Catalog.regN 1 (fresh 0), ⟨2, .legacy⟩, rfl, by decide⟩

/-- Mutant: WAL replay re-mints a legacy packet as covered (the B1 regression). -/
def replayRemint (cat : Catalog) (c : Commit) : Option Cov :=
  match c.packet with
  | .unavail => none
  | _ => if c.W ≤ cat.next then some (cat.coverageBefore c.W) else none

theorem mutant_remint_breaks_reopen_eq_replay :
    ∃ cat c, certify cat c ≠ replayRemint cat c :=
  ⟨Catalog.regN 1 (fresh 0), ⟨2, .legacy⟩, by decide⟩

/-- Mutant: a restore inserts the prefix out of id order. -/
theorem mutant_out_of_order_breaks_epoch_mono :
    ∃ d e x y, ((fresh 0).insert 2 |>.insert 1).intro d = some x ∧
      ((fresh 0).insert 2 |>.insert 1).intro e = some y ∧ d < e ∧ ¬ x < y :=
  ⟨1, 2, 2, 1, by decide, by decide, by decide, by decide⟩

/-- Mutant: admission without the run-start check admits an uncovered ordinal. -/
def admitNoRunStart (g : Gen) (rc : Catalog) (d next : Nat) : Except Refusal Unit :=
  if anyFrom g.incomplete next (g.ordinal - next) then .error .incompleteIndexHistory else
  match rc.introduction d with
  | none => .error .incompleteIndexHistory
  | some i =>
    if g.ordinal ≤ next then .ok () else
    match scan g.cov next (g.ordinal - next) with
    | none => .error .incompleteIndexHistory
    | some c => if c.ext i then .ok () else .error .incompleteIndexHistory

theorem mutant_no_run_start_unsound :
    let m : Nat → Option Cov := fun o => if o = 0 then some ⟨0, 2⟩ else some ⟨0, 1⟩
    let rc := Catalog.regN 2 (fresh 0)
    admitNoRunStart (buildN m 2) rc 2 0 = .ok () ∧
      ∃ y i, m 1 = some y ∧ rc.introduction 2 = some i ∧ ¬ y.ext i := by
  refine ⟨rfl, ⟨0, 1⟩, ⟨0, 2⟩, by decide, by decide, by decide⟩

end Coverage
