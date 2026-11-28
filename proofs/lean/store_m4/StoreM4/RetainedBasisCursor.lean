-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/token.rs:47-104 sha256:834a902441d8af9fd8958e2cb40d6d260c8ae7c0a6036da13fea0f943874c7a7
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/stream.rs:96-399 sha256:3c9700afe6ffaf4371b1862b22c0ccb63a03b7b9b6187040c893b67e48e67ad3
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/aux_index.rs:28-91 sha256:9837320f1bb8597ca6bcdfb3dcdff55bf7acf33fff618cf6f9bdd957a86f8e68
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/chunk_format.rs:133-254 sha256:2836d039018367b53049ced8a16d01b23b8d1c506be8a7cdb20977f5e5b88a51
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/chunk_scan.rs:26-385 sha256:ae4ca45ccdcd10f6e60007a467350c98fa6b0c91f17d6a4d17a32ce9d0a1c334
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/durable_reconcile.rs:44-298 sha256:8011d7c5e684f1da70d962347bb5f4ba301ae0a3d662a12bfc940cbaa958670a
-- proof-mirror: workspaces/worth-store/crates/worth-store-application/src/historical_cursor/durable_mutations.rs:28-360 sha256:1e4d9709f12d9752ee6960c05d4d5012c19d6ffd5aeafa4e68b02b5fde5d0c1d

/-!
# `RetainedBasisCursorImmutable` (Store Milestone 4, Phase M4.4)

Formal model of WORTH's durable retained-basis historical cursors:

* A [`Token`] (`RetainedBasisToken`) binds `(homeId, incarnation, branchOrigin,
  commitId, commitment, schemaVer, queryDigest, afterKey, mac)` where `afterKey`
  is strictly a canonical logical key (`Option Nat`), `commitment` depends solely
  on canonical logical rows (never physical chunk locators or layer boundaries),
  and `mac` is authenticated by the Home's persisted `macKey`.
* `consolidateChunk` flattens layered history chunks at arbitrary new physical
  locators while keeping `canonicalRows` and `rootCommitment` bit-identical.
* `chunkBoundAuxIndex` models the real per-chunk upper-bound (`maxKey`) auxiliary
  index directly over `chunks : List (List Row)` and proves exactness & discard
  invariance without any unproved `SoundAuxIndex` assumption.
* Explicit `durablePins : List OwnedPin` ownership models Store's `PhysicalRootPins`
  and EVERY two-phase durable crash cut (`R1`/`R2`/`R3` retain, `C1`/`C2`/`C3`/`C4`
  consolidation, `P1`/`P2` prune). `reconcileOnOpen` computes `boundPinIds` from
  reconciled catalog entries and expires every unbound durable pin, proving exact
  pin ownership without vacuous list clearing.
* `resumeToken` and `pruneBasis` serialize atomically under Store's reclaim
  fence: racing them yields exclusively two lawful outcomes — loan acquired
  before prune (exact read, reclamation blocked while `activeLoans > 0`) OR
  prune won before loan (`HistoryUnavailable` before admission, zero loans).
* `reconcileOnOpen` drops volatile `activeLoans` and increments `incarnation`.
  Resuming succeeds iff the token passes Home/MAC/branch/schema/query verification
  and the exact `(commitId, commitment)` is still independently retained — NEVER
  rebinding to `HEAD`.
* Explicit negative-mutant counterexamples certify `M7`.
-/

namespace StoreM4.RetainedBasisCursor

/-! ## Canonical rows, layered chunks & consolidation invariance -/

/-- One canonical logical row ordered by its logical `key`. -/
structure Row where
  key : Nat
  payload : Nat
  history : List Nat
  deriving DecidableEq, Repr

/-- One physical chunk: layered inline rows and physical locator. -/
structure LayeredChunk where
  locator : Nat
  layers : List (List Row)
  deriving DecidableEq, Repr

/-- Canonical row stream of a physical chunk: flattens layers in order, ignoring `locator`. -/
def LayeredChunk.canonicalRows (c : LayeredChunk) : List Row :=
  c.layers.flatten

/-- History consolidation: flattens layers into a single layer at a brand-new `newLocator`. -/
def LayeredChunk.consolidate (c : LayeredChunk) (newLocator : Nat) : LayeredChunk :=
  { locator := newLocator, layers := [c.layers.flatten] }

theorem consolidate_preserves_chunk_rows (c : LayeredChunk) (newLocator : Nat) :
    (c.consolidate newLocator).canonicalRows = c.canonicalRows := by
  simp [LayeredChunk.consolidate, LayeredChunk.canonicalRows]

/-- Canonical rows across an ordered list of physical chunks. -/
def basisCanonicalRows (chunks : List LayeredChunk) : List Row :=
  (chunks.map LayeredChunk.canonicalRows).flatten

/-- Consolidation across all chunks of a basis with arbitrary new physical locators. -/
def consolidateBasis (chunks : List LayeredChunk) (newLocator : LayeredChunk → Nat) :
    List LayeredChunk :=
  chunks.map fun c => c.consolidate (newLocator c)

theorem consolidateBasis_preserves_rows (chunks : List LayeredChunk)
    (newLocator : LayeredChunk → Nat) :
    basisCanonicalRows (consolidateBasis chunks newLocator) = basisCanonicalRows chunks := by
  unfold basisCanonicalRows consolidateBasis
  rw [List.map_map]
  congr 1
  apply List.map_congr_left
  intro c _
  simp only [Function.comp]
  exact consolidate_preserves_chunk_rows c (newLocator c)

/-- Any consolidation-invariant commitment hash over canonical rows is bit-identical
    before and after consolidating chunks to arbitrary new physical locators. -/
theorem consolidation_preserves_commitment {α : Type} (hash : List Row → α)
    (chunks : List LayeredChunk) (newLocator : LayeredChunk → Nat) :
    hash (basisCanonicalRows (consolidateBasis chunks newLocator)) =
      hash (basisCanonicalRows chunks) := by
  rw [consolidateBasis_preserves_rows]

/-! ## Canonical logical key resume & chunk-bound auxiliary index exactness -/

/-- Whether a row's canonical logical key sorts strictly after `pos`. -/
def keyAfter (pos : Option Nat) (r : Row) : Bool :=
  match pos with
  | none => true
  | some k => decide (k < r.key)

/-- Canonical suffix strictly after `pos` (independent of any physical index). -/
def canonicalSuffix (rows : List Row) (pos : Option Nat) : List Row :=
  rows.filter (keyAfter pos)

/-- Canonical prefix up to and including `pos`. -/
def canonicalPrefix (rows : List Row) (pos : Option Nat) : List Row :=
  rows.filter fun r => !keyAfter pos r

/-- Strictly ascending canonical key order on a row list. -/
def StrictlySorted : List Row → Prop
  | [] => True
  | [_] => True
  | r₁ :: r₂ :: rest => r₁.key < r₂.key ∧ StrictlySorted (r₂ :: rest)

theorem sorted_all_gt {k : Nat} : ∀ {xs : List Row},
    StrictlySorted (⟨k, 0, []⟩ :: xs) → ∀ r ∈ xs, k < r.key
  | [], _, _, hr => by cases hr
  | [_], h, x, hx => by
      simp only [List.mem_singleton] at hx; subst hx; exact h.1
  | r₁ :: r₂ :: rest, h, x, hx => by
      rcases List.mem_cons.mp hx with rfl | hx'
      · exact h.1
      · have hgt : r₁.key < x.key :=
          sorted_all_gt (k := r₁.key) (by exact ⟨h.2.1, h.2.2⟩) x hx'
        exact Nat.lt_trans h.1 hgt

theorem filter_false_nil : ∀ (xs : List Row), xs.filter (fun _ => false) = []
  | [] => rfl
  | _ :: xs => filter_false_nil xs

theorem filter_true_self : ∀ (xs : List Row), xs.filter (fun _ => true) = xs
  | [] => rfl
  | x :: xs => congrArg (x :: ·) (filter_true_self xs)

theorem canonicalPrefix_append_suffix_of_sorted :
    ∀ (rows : List Row) (pos : Option Nat),
      StrictlySorted rows →
      canonicalPrefix rows pos ++ canonicalSuffix rows pos = rows
  | [], _, _ => by simp [canonicalPrefix, canonicalSuffix]
  | [r], pos, _ => by
      unfold canonicalPrefix canonicalSuffix
      cases hk : keyAfter pos r <;> simp [hk]
  | r₁ :: r₂ :: rest, none, _ => by
      unfold canonicalPrefix canonicalSuffix keyAfter
      simp [filter_false_nil, filter_true_self]
  | r₁ :: r₂ :: rest, some k, ⟨hlt, hrest⟩ => by
      have ih := canonicalPrefix_append_suffix_of_sorted (r₂ :: rest) (some k) hrest
      by_cases h1 : k < r₁.key
      · have hall : ∀ r ∈ (r₂ :: rest), k < r.key := by
          intro r hr
          have hhead : StrictlySorted (⟨r₁.key, 0, []⟩ :: r₂ :: rest) := ⟨hlt, hrest⟩
          exact Nat.lt_trans h1 (sorted_all_gt hhead r hr)
        have hpref : canonicalPrefix (r₁ :: r₂ :: rest) (some k) = [] := by
          unfold canonicalPrefix
          rw [List.filter_eq_nil_iff]
          intro r hr
          rcases List.mem_cons.mp hr with rfl | hr'
          · simp [keyAfter, h1]
          · simp [keyAfter, hall r hr']
        have hsuff : canonicalSuffix (r₁ :: r₂ :: rest) (some k) = r₁ :: r₂ :: rest := by
          unfold canonicalSuffix
          rw [List.filter_eq_self]
          intro r hr
          rcases List.mem_cons.mp hr with rfl | hr'
          · simp [keyAfter, h1]
          · simp [keyAfter, hall r hr']
        simp [hpref, hsuff]
      · have hpref : canonicalPrefix (r₁ :: r₂ :: rest) (some k) =
            r₁ :: canonicalPrefix (r₂ :: rest) (some k) := by
          unfold canonicalPrefix
          rw [List.filter_cons]
          simp [keyAfter, h1]
        have hsuff : canonicalSuffix (r₁ :: r₂ :: rest) (some k) =
            canonicalSuffix (r₂ :: rest) (some k) := by
          unfold canonicalSuffix
          rw [List.filter_cons]
          simp [keyAfter, h1]
        rw [hpref, hsuff, List.cons_append, ih]

/-- Derived upper-bound key of one physical chunk (`aux_index.rs` `max_key`). -/
def chunkMaxKey : List Row → Option Nat
  | [] => none
  | r :: rest =>
      match chunkMaxKey rest with
      | none => some r.key
      | some m => some (max r.key m)

theorem chunkMaxKey_eq_none_iff : ∀ {c : List Row}, chunkMaxKey c = none ↔ c = []
  | [] => ⟨fun _ => rfl, fun _ => rfl⟩
  | r :: rest => by
      constructor
      · intro h
        unfold chunkMaxKey at h
        cases hrest : chunkMaxKey rest <;> rw [hrest] at h <;> cases h
      · intro h; cases h

theorem mem_le_chunkMaxKey :
    ∀ {c : List Row} {m : Nat}, chunkMaxKey c = some m → ∀ r ∈ c, r.key ≤ m
  | [], _, h, _, hr => by cases hr
  | x :: rest, m, h, r, hr => by
      unfold chunkMaxKey at h
      cases hrest : chunkMaxKey rest with
      | none =>
          rw [hrest] at h
          simp only [Option.some.injEq] at h
          have hnil : rest = [] := chunkMaxKey_eq_none_iff.mp hrest
          subst hnil
          simp only [List.mem_singleton] at hr
          subst hr; omega
      | some m' =>
          rw [hrest] at h
          simp only [Option.some.injEq] at h
          rcases List.mem_cons.mp hr with rfl | hr'
          · omega
          · have hle := mem_le_chunkMaxKey hrest r hr'
            omega

/-- Real chunk-bound auxiliary index predicate: skips a chunk iff its derived
    `chunkMaxKey` is `≤ k` (or empty), matching `aux_index.rs`. -/
def chunkBoundSkip (pos : Option Nat) (c : List Row) : Bool :=
  match pos, chunkMaxKey c with
  | none, _ => false
  | some _, none => true
  | some k, some m => decide (m ≤ k)

/-- Resuming with the real chunk-bound auxiliary index skips chunks whose
    upper bound is `≤ pos`, then filters the remaining chunks canonically. -/
def resumeWithChunkIndex (chunks : List (List Row)) (pos : Option Nat) : List Row :=
  ((chunks.filter (fun c => !chunkBoundSkip pos c)).flatten).filter (keyAfter pos)

theorem filter_chunk_nil_of_skip (pos : Option Nat) (c : List Row)
    (hskip : chunkBoundSkip pos c = true) : c.filter (keyAfter pos) = [] := by
  rw [List.filter_eq_nil_iff]
  intro r hr
  cases pos with
  | none => simp [chunkBoundSkip] at hskip
  | some k =>
      cases hmax : chunkMaxKey c with
      | none =>
          have hnil : c = [] := chunkMaxKey_eq_none_iff.mp hmax
          subst hnil; cases hr
      | some m =>
          simp only [chunkBoundSkip, hmax, decide_eq_true_eq] at hskip
          have hle := mem_le_chunkMaxKey hmax r hr
          simp [keyAfter]
          omega

theorem resumeWithChunkIndex_eq_canonicalSuffix (chunks : List (List Row)) (pos : Option Nat) :
    resumeWithChunkIndex chunks pos = canonicalSuffix chunks.flatten pos := by
  unfold resumeWithChunkIndex canonicalSuffix
  induction chunks with
  | nil => rfl
  | cons c cs ih =>
      rw [List.filter_cons, List.flatten_cons, List.filter_append]
      cases hskip : chunkBoundSkip pos c with
      | true =>
          have hnil := filter_chunk_nil_of_skip pos c hskip
          simp only [Bool.not_true, Bool.false_eq_true, ↓reduceIte, hnil, List.nil_append]
          exact ih
      | false =>
          simp only [Bool.not_false, ↓reduceIte, List.flatten_cons, List.filter_append]
          exact congrArg (c.filter (keyAfter pos) ++ ·) ih

/-- Exactness of the real chunk-bound auxiliary index and invariance under
    auxiliary index discard (`resumeWithChunkIndex` = `canonicalSuffix`),
    proved directly without any unproved `SoundAuxIndex` assumption. -/
theorem canonical_resume_exact_after_index_discard (chunks : List (List Row))
    (pos : Option Nat) (hSorted : StrictlySorted chunks.flatten) :
    resumeWithChunkIndex chunks pos = canonicalSuffix chunks.flatten pos ∧
    canonicalPrefix chunks.flatten pos ++ canonicalSuffix chunks.flatten pos = chunks.flatten :=
  ⟨resumeWithChunkIndex_eq_canonicalSuffix chunks pos,
   canonicalPrefix_append_suffix_of_sorted chunks.flatten pos hSorted⟩

/-! ## Home-MAC-authenticated `Token`, explicit `OwnedPin` ownership & two-phase cuts -/

/-- Pure deterministic Home-keyed MAC binding every authenticated token field. -/
def computeTokenMac (macKey homeId incarnation branchOrigin commitId commitment
    schemaVer queryDigest : Nat) (afterKey : Option Nat) : Nat :=
  let ak := match afterKey with | none => 0 | some k => k + 1
  macKey * 1000003 + homeId * 9176 + incarnation * 131 + branchOrigin * 257 +
    commitId * 521 + commitment * 1031 + schemaVer * 2053 + queryDigest * 4099 + ak * 8191

/-- Durable resumable cursor token (Home-MAC authenticated, never an unbounded pin). -/
structure Token where
  homeId : Nat
  incarnation : Nat
  branchOrigin : Nat
  commitId : Nat
  commitment : Nat
  schemaVer : Nat
  queryDigest : Nat
  afterKey : Option Nat
  mac : Nat
  deriving DecidableEq, Repr

/-- Issue a Home-MAC-authenticated token from the Home's secret `macKey`. -/
def issueToken (macKey homeId incarnation branchOrigin commitId commitment
    schemaVer queryDigest : Nat) (afterKey : Option Nat) : Token :=
  { homeId, incarnation, branchOrigin, commitId, commitment, schemaVer, queryDigest, afterKey
    mac := computeTokenMac macKey homeId incarnation branchOrigin commitId commitment
             schemaVer queryDigest afterKey }

/-- Verify a token's Home-keyed MAC against `macKey`. -/
def Token.verifyMac (macKey : Nat) (tok : Token) : Bool :=
  tok.mac == computeTokenMac macKey tok.homeId tok.incarnation tok.branchOrigin
    tok.commitId tok.commitment tok.schemaVer tok.queryDigest tok.afterKey

/-- One durable Store pin record (`WHPIN1` payload in `PhysicalRootPins`). -/
structure OwnedPin where
  pinId : Nat
  commitId : Nat
  epoch : Nat
  active : Bool
  deriving DecidableEq, Repr

/-- Retained basis entry tracked across two-phase pin + catalog mutations.
    * `pinId = 0`: staged in catalog before durable pin acquisition (`R1`).
    * `stagedConsolidate = true`: consolidated members published before new pin (`C1`/`C2`).
    * `prevPinId = some p`: old consolidation pin awaiting expiration (`C3`). -/
structure BasisEntry where
  commitId : Nat
  branchOrigin : Nat
  commitment : Nat
  pinId : Nat
  prevPinId : Option Nat
  epoch : Nat
  stagedConsolidate : Bool
  retained : Bool
  activeLoans : Nat
  reclaimed : Bool
  rows : List Row
  deriving DecidableEq, Repr

/-- Store runtime state governing Home identity, MAC key, retained bases,
    explicit durable pin ownership (`durablePins`), HEAD, and reclaim-fenced loans. -/
structure Sys where
  homeId : Nat
  macKey : Nat
  incarnation : Nat
  nextPinId : Nat
  headCommit : Nat
  headRows : List Row
  supportedSchema : Nat
  expectedQuery : Nat
  bases : List BasisEntry
  durablePins : List OwnedPin
  pendingRetirePins : List Nat
  deriving DecidableEq, Repr

/-- Outcome of `resumeToken`. -/
inductive ResumeOutcome where
  | admitted (sys' : Sys) (commitId : Nat) (rows : List Row)
  | historyUnavailable
  | unsupportedMeaning
  | corruptStorage
  deriving DecidableEq, Repr

def BasisEntry.independentlyRetained (b : BasisEntry) : Bool :=
  b.retained && !b.reclaimed && decide (0 < b.pinId)

/-- Pin IDs currently bound by retained catalog entries. -/
def boundPinIds (bases : List BasisEntry) : List Nat :=
  (bases.filter (fun b => decide (0 < b.pinId) && b.retained)).map BasisEntry.pinId

def expirePinInList (pid : Nat) (pins : List OwnedPin) : List OwnedPin :=
  pins.map fun p => if p.pinId == pid then { p with active := false } else p

def bumpLoan (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then { b with activeLoans := b.activeLoans + 1 } else b

def dropLoan (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then { b with activeLoans := b.activeLoans - 1 } else b

def markPrunedCatalog (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then { b with retained := false } else b

def markPrunedImmediate (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then
    { b with retained := false, reclaimed := b.reclaimed || (b.activeLoans == 0) }
  else b

def tryReclaim (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid && !b.retained && b.activeLoans == 0 then
    { b with reclaimed := true }
  else b

def bindPinToEntry (cid newPin : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then
    { b with pinId := newPin, retained := true, reclaimed := false }
  else b

def markConsolidateStaged (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then { b with stagedConsolidate := true } else b

def bindConsolidatePin (cid newPin : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then
    { b with
      prevPinId := some b.pinId
      pinId := newPin
      epoch := b.epoch + 1
      stagedConsolidate := false }
  else b

def finishConsolidateExpire (cid : Nat) (b : BasisEntry) : BasisEntry :=
  if b.commitId = cid then { b with prevPinId := none } else b

def reconcileEntry (b : BasisEntry) : BasisEntry :=
  { b with
    activeLoans := 0
    prevPinId := none
    stagedConsolidate := false
    reclaimed := b.reclaimed || !b.retained }

/-- Atomic `resume_cursor` under Store's reclaim fence: verifies Home-keyed MAC,
    `homeId`, `incarnation`, `schemaVer`, `queryDigest`, `branchOrigin`, and
    `commitment`, resolves exact `commitId` (NEVER `HEAD`), and atomically
    increments `activeLoans` iff independently retained. -/
def resumeToken (s : Sys) (tok : Token) : ResumeOutcome :=
  if !tok.verifyMac s.macKey then .corruptStorage
  else if tok.homeId != s.homeId then .corruptStorage
  else if s.incarnation < tok.incarnation then .corruptStorage
  else if tok.schemaVer != s.supportedSchema || tok.queryDigest != s.expectedQuery then
    .unsupportedMeaning
  else
    match s.bases.find? (fun b => b.commitId == tok.commitId) with
    | none => .historyUnavailable
    | some b =>
        if !b.independentlyRetained then .historyUnavailable
        else if b.branchOrigin != tok.branchOrigin || b.commitment != tok.commitment then
          .corruptStorage
        else
          .admitted
            { s with bases := s.bases.map (bumpLoan b.commitId) }
            b.commitId
            (canonicalSuffix b.rows tok.afterKey)

/-- Phase R1 (stage retain in catalog before pin): `pinId = 0`, `retained = false`. -/
def stageRetain (s : Sys) (cid origin comm : Nat) (rows : List Row) : Sys :=
  if s.bases.any (fun b => b.commitId == cid) then s
  else
    { s with
      bases := s.bases ++ [⟨cid, origin, comm, 0, none, 0, false, false, 0, false, rows⟩] }

/-- Phase R2 (pin acquired before catalog bind): records an unbound active `OwnedPin`. -/
def pinRetainOrphan (s : Sys) (cid : Nat) : Sys :=
  let newPin := max s.nextPinId 1
  { s with
    nextPinId := newPin + 1
    durablePins := ⟨newPin, cid, 0, true⟩ :: s.durablePins }

/-- Phase R3 (complete retain bind): allocates active `OwnedPin`, binds `pinId`, marks retained. -/
def bindRetain (s : Sys) (cid : Nat) : Sys :=
  let newPin := max s.nextPinId 1
  { s with
    nextPinId := newPin + 1
    bases := s.bases.map (bindPinToEntry cid newPin)
    durablePins := ⟨newPin, cid, 0, true⟩ :: s.durablePins }

/-- Atomic full retain (normal non-crashed retain). -/
def retainComplete (s : Sys) (cid origin comm : Nat) (rows : List Row) : Sys :=
  bindRetain (stageRetain s cid origin comm rows) cid

/-- Phase C1 (stage consolidation members before new pin). -/
def stageConsolidate (s : Sys) (cid : Nat) : Sys :=
  { s with bases := s.bases.map (markConsolidateStaged cid) }

/-- Phase C2 (pin replacement consolidated root before binding catalog): records unbound active pin. -/
def pinConsolidateReplacement (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      let newPin := max s.nextPinId 1
      { s with
        nextPinId := newPin + 1
        durablePins := ⟨newPin, cid, b.epoch + 1, true⟩ :: s.durablePins }

/-- Phase C3 (bind new consolidation pin while recording `prevPinId`). -/
def bindConsolidate (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      let newPin := max s.nextPinId 1
      { s with
        nextPinId := newPin + 1
        bases := s.bases.map (bindConsolidatePin cid newPin)
        durablePins := ⟨newPin, cid, b.epoch + 1, true⟩ :: s.durablePins }

/-- Phase C4 (complete consolidation): expire old `prevPinId` in `durablePins`. -/
def expireConsolidateOldPin (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      let pins' := match b.prevPinId with
        | none => s.durablePins
        | some oldPin => expirePinInList oldPin s.durablePins
      { s with
        bases := s.bases.map (finishConsolidateExpire cid)
        durablePins := pins' }

/-- Phase P1: remove basis from catalog and record `pendingRetirePins` before pin expire. -/
def pruneCatalogPendingExpire (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      { s with
        bases := s.bases.map (markPrunedCatalog cid)
        pendingRetirePins := b.pinId :: s.pendingRetirePins }

/-- Phase P2: atomic `prune_basis` (catalog + immediate pin expiry when drained). -/
def pruneBasis (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      let pins' := if b.activeLoans == 0 then expirePinInList b.pinId s.durablePins else s.durablePins
      { s with
        bases := s.bases.map (markPrunedImmediate cid)
        durablePins := pins' }

/-- Reclamation attempt under Store's reclaim fence: never frees a retained or loaned basis. -/
def reclaimBasis (s : Sys) (cid : Nat) : Sys :=
  match s.bases.find? (fun b => b.commitId == cid) with
  | none => s
  | some b =>
      let pins' := if !b.retained && b.activeLoans == 0 then expirePinInList b.pinId s.durablePins else s.durablePins
      { s with
        bases := s.bases.map (tryReclaim cid)
        durablePins := pins' }

/-- Release one in-process `ReaderLoan`. -/
def releaseLoan (s : Sys) (cid : Nat) : Sys :=
  { s with bases := s.bases.map (dropLoan cid) }

/-- Advance HEAD to a newer commit without altering retained historical bases. -/
def advanceHead (s : Sys) (newCommit : Nat) (newRows : List Row) : Sys :=
  { s with headCommit := newCommit, headRows := newRows }

/-- Real crash-cut reconciliation (`reconcile_on_open` in `durable_reconcile.rs`):
    * increments `incarnation` and drops volatile `activeLoans`,
    * drops uncommitted `R1` staged entries (`pinId == 0`),
    * clears `C1`/`C2` staged consolidation flag and `C3` `prevPinId`,
    * computes `boundPinIds` from the reconciled retained catalog entries and
      expires EVERY durable pin in `durablePins` not owned by a retained basis
      (catching `R2` orphans, `C2` replacement orphans, `C3` old pins, and `P1`
      pending-retire pins uniformly via pin-ownership diff!),
    * drains `pendingRetirePins`. -/
def reconcileOnOpen (s : Sys) : Sys :=
  let keptBases := (s.bases.filter (fun b => decide (0 < b.pinId))).map reconcileEntry
  let bound := boundPinIds keptBases
  { s with
    incarnation := s.incarnation + 1
    bases := keptBases
    durablePins := s.durablePins.map (fun p =>
      if bound.contains p.pinId then p else { p with active := false })
    pendingRetirePins := [] }

/-- System invariant preserved across every legal transition and crash cut. -/
structure Inv (s : Sys) : Prop where
  nodup : (s.bases.map BasisEntry.commitId).Nodup
  loanNotReclaimed : ∀ b ∈ s.bases, 0 < b.activeLoans → b.reclaimed = false
  retainedNotReclaimed : ∀ b ∈ s.bases, b.retained = true → b.reclaimed = false
  retainedHasPin : ∀ b ∈ s.bases, b.retained = true → 0 < b.pinId

inductive Step : Sys → Sys → Prop
  | resume {s s' : Sys} {tok : Token} {cid : Nat} {rows : List Row} :
      resumeToken s tok = .admitted s' cid rows → Step s s'
  | release (s : Sys) (cid : Nat) : Step s (releaseLoan s cid)
  | cutR1Stage (s : Sys) (cid origin comm : Nat) (rows : List Row) :
      Step s (stageRetain s cid origin comm rows)
  | cutR2PinOrphan (s : Sys) (cid : Nat) : Step s (pinRetainOrphan s cid)
  | bindRetainStep (s : Sys) (cid : Nat) : Step s (bindRetain s cid)
  | cutC1StageConsolidate (s : Sys) (cid : Nat) : Step s (stageConsolidate s cid)
  | cutC2PinConsolidate (s : Sys) (cid : Nat) : Step s (pinConsolidateReplacement s cid)
  | cutC3BindConsolidate (s : Sys) (cid : Nat) : Step s (bindConsolidate s cid)
  | finishConsolidate (s : Sys) (cid : Nat) : Step s (expireConsolidateOldPin s cid)
  | cutP1PruneCatalog (s : Sys) (cid : Nat) : Step s (pruneCatalogPendingExpire s cid)
  | prune (s : Sys) (cid : Nat) : Step s (pruneBasis s cid)
  | reclaim (s : Sys) (cid : Nat) : Step s (reclaimBasis s cid)
  | advance (s : Sys) (cid : Nat) (rows : List Row) : Step s (advanceHead s cid rows)
  | crashReopen (s : Sys) : Step s (reconcileOnOpen s)

inductive Reachable (init : Sys) : Sys → Prop
  | start : Inv init → Reachable init init
  | step {s s' : Sys} : Reachable init s → Step s s' → Reachable init s'

theorem ids_map_eq {f : BasisEntry → BasisEntry} (hf : ∀ b, (f b).commitId = b.commitId)
    (bs : List BasisEntry) : (bs.map f).map BasisEntry.commitId = bs.map BasisEntry.commitId := by
  rw [List.map_map]
  exact List.map_congr_left (fun b _ => hf b)

theorem nodup_filter_map_commitId (p : BasisEntry → Bool) :
    ∀ {bs : List BasisEntry}, (bs.map BasisEntry.commitId).Nodup →
      ((bs.filter p).map BasisEntry.commitId).Nodup
  | [], _ => List.nodup_nil
  | x :: xs, hnd => by
      rw [List.map_cons, List.nodup_cons] at hnd
      obtain ⟨hnotin, hnd'⟩ := hnd
      rw [List.filter_cons]
      cases hpx : p x with
      | false =>
          simp only [Bool.false_eq_true, ↓reduceIte]
          exact nodup_filter_map_commitId p hnd'
      | true =>
          simp only [↓reduceIte, List.map_cons, List.nodup_cons]
          refine ⟨?_, nodup_filter_map_commitId p hnd'⟩
          intro hmem
          obtain ⟨y, hy, heq⟩ := List.mem_map.mp hmem
          exact hnotin (List.mem_map.mpr ⟨y, (List.mem_filter.mp hy).1, heq⟩)

theorem eq_of_mem_commitId :
    ∀ {bs : List BasisEntry}, (bs.map BasisEntry.commitId).Nodup →
      ∀ {a b : BasisEntry}, a ∈ bs → b ∈ bs → a.commitId = b.commitId → a = b
  | [], _, _, _, ha, _, _ => by cases ha
  | x :: xs, hnd, a, b, ha, hb, heq => by
      rw [List.map_cons, List.nodup_cons] at hnd
      obtain ⟨hnotin, hnd⟩ := hnd
      rcases List.mem_cons.mp ha with rfl | ha' <;> rcases List.mem_cons.mp hb with rfl | hb'
      · rfl
      · exact absurd (heq ▸ List.mem_map_of_mem hb') hnotin
      · exact absurd (heq.symm ▸ List.mem_map_of_mem ha') hnotin
      · exact eq_of_mem_commitId hnd ha' hb' heq

theorem step_inv {s s' : Sys} (hi : Inv s) (hs : Step s s') : Inv s' := by
  cases hs with
  | resume hadm =>
      unfold resumeToken at hadm
      split at hadm; · cases hadm
      split at hadm; · cases hadm
      split at hadm; · cases hadm
      split at hadm; · cases hadm
      split at hadm; · cases hadm
      rename_i b hfind
      split at hadm; · cases hadm
      split at hadm; · cases hadm
      rename_i hret _
      simp only [ResumeOutcome.admitted.injEq] at hadm
      obtain ⟨rfl, _, _⟩ := hadm
      have hb_mem : b ∈ s.bases := List.mem_of_find?_eq_some hfind
      have hb_not_rec : b.reclaimed = false := by
        simp [BasisEntry.independentlyRetained] at hret
        exact hret.1.2
      refine ⟨?_, ?_, ?_, ?_⟩
      · rw [ids_map_eq (fun x => by unfold bumpLoan; split <;> rfl)]; exact hi.nodup
      · intro y hy hpos
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bumpLoan at hpos ⊢
        by_cases hxc : x.commitId = b.commitId
        · have hx_eq := eq_of_mem_commitId hi.nodup hx hb_mem hxc
          subst hx_eq; simp [hb_not_rec]
        · simp [hxc] at hpos ⊢; exact hi.loanNotReclaimed x hx hpos
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bumpLoan at hretY ⊢
        split at hretY <;> split <;> simp_all [hi.retainedNotReclaimed x hx]
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bumpLoan at hretY ⊢
        split at hretY <;> split <;> simp_all [hi.retainedHasPin x hx]
  | release cid =>
      unfold releaseLoan
      refine ⟨?_, ?_, ?_, ?_⟩
      · rw [ids_map_eq (fun x => by unfold dropLoan; split <;> rfl)]; exact hi.nodup
      · intro y hy hpos
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold dropLoan at hpos ⊢
        by_cases hxc : x.commitId = cid
        · simp [hxc] at hpos ⊢; exact hi.loanNotReclaimed x hx (by omega)
        · simp [hxc] at hpos ⊢; exact hi.loanNotReclaimed x hx hpos
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold dropLoan at hretY ⊢
        split at hretY <;> split <;> simp_all [hi.retainedNotReclaimed x hx]
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold dropLoan at hretY ⊢
        split at hretY <;> split <;> simp_all [hi.retainedHasPin x hx]
  | cutR1Stage cid origin comm rows =>
      unfold stageRetain
      split
      · exact hi
      · rename_i hnotany
        refine ⟨?_, ?_, ?_, ?_⟩
        · rw [List.map_append, List.map_singleton, List.nodup_append]
          refine ⟨hi.nodup, by simp, ?_⟩
          intro a ha b hb hab
          simp only [List.mem_singleton] at hb; subst hb; subst hab
          obtain ⟨x, hx, hxc⟩ := List.mem_map.mp ha
          have hany : s.bases.any (fun e => e.commitId == a) = true :=
            List.any_eq_true.mpr ⟨x, hx, beq_iff_eq.mpr hxc⟩
          exact hnotany hany
        · intro y hy hpos
          rcases List.mem_append.mp hy with hy' | hy'
          · exact hi.loanNotReclaimed y hy' hpos
          · simp only [List.mem_singleton] at hy'; subst hy'; simp at hpos
        · intro y hy hretY
          rcases List.mem_append.mp hy with hy' | hy'
          · exact hi.retainedNotReclaimed y hy' hretY
          · simp only [List.mem_singleton] at hy'; subst hy'; simp at hretY
        · intro y hy hretY
          rcases List.mem_append.mp hy with hy' | hy'
          · exact hi.retainedHasPin y hy' hretY
          · simp only [List.mem_singleton] at hy'; subst hy'; simp at hretY
  | cutR2PinOrphan _ =>
      exact ⟨hi.nodup, hi.loanNotReclaimed, hi.retainedNotReclaimed, hi.retainedHasPin⟩
  | bindRetainStep cid =>
      unfold bindRetain
      refine ⟨?_, ?_, ?_, ?_⟩
      · rw [ids_map_eq (fun x => by unfold bindPinToEntry; split <;> rfl)]; exact hi.nodup
      · intro y hy hpos
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bindPinToEntry at hpos ⊢
        by_cases hxc : x.commitId = cid
        · simp [hxc]
        · simp [hxc] at hpos ⊢; exact hi.loanNotReclaimed x hx hpos
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bindPinToEntry at hretY ⊢
        by_cases hxc : x.commitId = cid
        · simp [hxc]
        · simp [hxc] at hretY ⊢; exact hi.retainedNotReclaimed x hx hretY
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold bindPinToEntry at hretY ⊢
        by_cases hxc : x.commitId = cid
        · simp [hxc]; omega
        · simp [hxc] at hretY ⊢; exact hi.retainedHasPin x hx hretY
  | cutC1StageConsolidate cid =>
      unfold stageConsolidate
      refine ⟨?_, ?_, ?_, ?_⟩
      · rw [ids_map_eq (fun x => by unfold markConsolidateStaged; split <;> rfl)]; exact hi.nodup
      · intro y hy hpos
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold markConsolidateStaged at hpos ⊢
        split at hpos <;> simp_all [hi.loanNotReclaimed x hx]
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold markConsolidateStaged at hretY ⊢
        split at hretY <;> simp_all [hi.retainedNotReclaimed x hx]
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        unfold markConsolidateStaged at hretY ⊢
        split at hretY <;> simp_all [hi.retainedHasPin x hx]
  | cutC2PinConsolidate cid =>
      unfold pinConsolidateReplacement
      split
      · exact hi
      · exact ⟨hi.nodup, hi.loanNotReclaimed, hi.retainedNotReclaimed, hi.retainedHasPin⟩
  | cutC3BindConsolidate cid =>
      unfold bindConsolidate
      split
      · exact hi
      · refine ⟨?_, ?_, ?_, ?_⟩
        · rw [ids_map_eq (fun x => by unfold bindConsolidatePin; split <;> rfl)]; exact hi.nodup
        · intro y hy hpos
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold bindConsolidatePin at hpos ⊢
          split at hpos <;> simp_all [hi.loanNotReclaimed x hx]
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold bindConsolidatePin at hretY ⊢
          split at hretY <;> simp_all [hi.retainedNotReclaimed x hx]
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold bindConsolidatePin at hretY ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc]; omega
          · simp [hxc] at hretY ⊢; exact hi.retainedHasPin x hx hretY
  | finishConsolidate cid =>
      unfold expireConsolidateOldPin
      split
      · exact hi
      · refine ⟨?_, ?_, ?_, ?_⟩
        · rw [ids_map_eq (fun x => by unfold finishConsolidateExpire; split <;> rfl)]; exact hi.nodup
        · intro y hy hpos
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold finishConsolidateExpire at hpos ⊢
          split at hpos <;> simp_all [hi.loanNotReclaimed x hx]
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold finishConsolidateExpire at hretY ⊢
          split at hretY <;> simp_all [hi.retainedNotReclaimed x hx]
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold finishConsolidateExpire at hretY ⊢
          split at hretY <;> simp_all [hi.retainedHasPin x hx]
  | cutP1PruneCatalog cid =>
      unfold pruneCatalogPendingExpire
      split
      · exact hi
      · refine ⟨?_, ?_, ?_, ?_⟩
        · rw [ids_map_eq (fun x => by unfold markPrunedCatalog; split <;> rfl)]; exact hi.nodup
        · intro y hy hpos
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedCatalog at hpos ⊢
          split at hpos <;> simp_all [hi.loanNotReclaimed x hx]
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedCatalog at hretY ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc] at hretY
          · simp [hxc] at hretY ⊢; exact hi.retainedNotReclaimed x hx hretY
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedCatalog at hretY ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc] at hretY
          · simp [hxc] at hretY ⊢; exact hi.retainedHasPin x hx hretY
  | prune cid =>
      unfold pruneBasis
      split
      · exact hi
      · refine ⟨?_, ?_, ?_, ?_⟩
        · rw [ids_map_eq (fun x => by unfold markPrunedImmediate; split <;> rfl)]; exact hi.nodup
        · intro y hy hpos
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedImmediate at hpos ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc] at hpos ⊢; exact ⟨hi.loanNotReclaimed x hx hpos, Nat.ne_of_gt hpos⟩
          · simp [hxc] at hpos ⊢; exact hi.loanNotReclaimed x hx hpos
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedImmediate at hretY ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc] at hretY
          · simp [hxc] at hretY ⊢; exact hi.retainedNotReclaimed x hx hretY
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold markPrunedImmediate at hretY ⊢
          by_cases hxc : x.commitId = cid
          · simp [hxc] at hretY
          · simp [hxc] at hretY ⊢; exact hi.retainedHasPin x hx hretY
  | reclaim cid =>
      unfold reclaimBasis
      split
      · exact hi
      · refine ⟨?_, ?_, ?_, ?_⟩
        · rw [ids_map_eq (fun x => by unfold tryReclaim; split <;> rfl)]; exact hi.nodup
        · intro y hy hpos
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold tryReclaim at hpos ⊢
          split at hpos
          · rename_i hcond
            have hloans : x.activeLoans = 0 := by
              simp only [Bool.and_eq_true, beq_iff_eq] at hcond; exact hcond.2
            simp [hloans] at hpos
          · rename_i hcond; simp [hcond]; exact hi.loanNotReclaimed x hx hpos
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold tryReclaim at hretY ⊢
          split at hretY
          · rename_i hcond
            have hretF : x.retained = false := by
              simp only [Bool.and_eq_true, Bool.not_eq_true'] at hcond; exact hcond.1.2
            simp [hretF] at hretY
          · rename_i hcond; simp [hcond]; exact hi.retainedNotReclaimed x hx hretY
        · intro y hy hretY
          obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
          unfold tryReclaim at hretY ⊢
          split at hretY
          · rename_i hcond
            have hretF : x.retained = false := by
              simp only [Bool.and_eq_true, Bool.not_eq_true'] at hcond; exact hcond.1.2
            simp [hretF] at hretY
          · rename_i hcond; simp [hcond]; exact hi.retainedHasPin x hx hretY
  | advance _ _ =>
      exact ⟨hi.nodup, hi.loanNotReclaimed, hi.retainedNotReclaimed, hi.retainedHasPin⟩
  | crashReopen =>
      unfold reconcileOnOpen
      refine ⟨?_, ?_, ?_, ?_⟩
      · have hcid : ∀ x : BasisEntry, (reconcileEntry x).commitId = x.commitId := fun _ => rfl
        rw [ids_map_eq hcid]
        exact nodup_filter_map_commitId (fun b => decide (0 < b.pinId)) hi.nodup
      · intro y hy hpos
        obtain ⟨x, _, rfl⟩ := List.mem_map.mp hy
        simp [reconcileEntry] at hpos
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        have hx' := (List.mem_filter.mp hx).1
        simp only [reconcileEntry] at hretY ⊢
        simp [hretY, hi.retainedNotReclaimed x hx' hretY]
      · intro y hy hretY
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        have hx' := (List.mem_filter.mp hx).1
        simp only [reconcileEntry] at hretY ⊢
        exact hi.retainedHasPin x hx' hretY

theorem Reachable.inv {init s : Sys} (hr : Reachable init s) : Inv s := by
  induction hr with
  | start h => exact h
  | step _ hs ih => exact step_inv ih hs

/-- Post-reconcile pin ownership & crash-cut cleanliness across EVERY crash cut
    (`R1`/`R2`/`C1`/`C2`/`C3`/`P1`):
    * every still-active durable pin in `durablePins` is owned by a retained basis
      (`p.pinId ∈ boundPinIds s'.bases`), proving zero leaked orphan/old/retired pins,
    * `pendingRetirePins = []`,
    * every basis entry has `0 < b.pinId`, `prevPinId = none`, `stagedConsolidate = false`,
      and `activeLoans = 0`. -/
theorem reconcileOnOpen_pin_ownership_exact (s : Sys) :
    let s' := reconcileOnOpen s
    s'.pendingRetirePins = [] ∧
    (∀ p ∈ s'.durablePins, p.active = true → p.pinId ∈ boundPinIds s'.bases) ∧
    (∀ b ∈ s'.bases, 0 < b.pinId ∧ b.prevPinId = none ∧
      b.stagedConsolidate = false ∧ b.activeLoans = 0) := by
  refine ⟨rfl, ?_, ?_⟩
  · intro p hp hact
    simp only [reconcileOnOpen, List.mem_map] at hp ⊢
    obtain ⟨q, _, rfl⟩ := hp
    by_cases hcont : q.pinId ∈ boundPinIds ((s.bases.filter (fun b => decide (0 < b.pinId))).map reconcileEntry)
    · simp [hcont]
    · simp [hcont] at hact
  · intro b hb
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hb
    have hpin := (List.mem_filter.mp hx).2
    simp only [decide_eq_true_eq] at hpin
    exact ⟨hpin, rfl, rfl, rfl⟩

theorem find?_unique {bs : List BasisEntry} (hnd : (bs.map BasisEntry.commitId).Nodup)
    {b : BasisEntry} (hb : b ∈ bs) :
    bs.find? (fun x => x.commitId == b.commitId) = some b := by
  induction bs with
  | nil => cases hb
  | cons x xs ih =>
      rw [List.map_cons, List.nodup_cons] at hnd
      obtain ⟨hnotin, hnd'⟩ := hnd
      rcases List.mem_cons.mp hb with rfl | hb'
      · simp
      · have hne : x.commitId ≠ b.commitId := fun h => hnotin (h ▸ List.mem_map_of_mem hb')
        simp [hne, ih hnd' hb']

theorem find?_unique_eq {bs : List BasisEntry} (hnd : (bs.map BasisEntry.commitId).Nodup)
    {b : BasisEntry} {cid : Nat} (hb : b ∈ bs) (hcid : b.commitId = cid) :
    bs.find? (fun x => x.commitId == cid) = some b := by
  subst hcid; exact find?_unique hnd hb

/-- Atomic resume-vs-prune exclusivity under Store's reclaim fence. -/
theorem resume_vs_prune_exclusive (s : Sys) (tok : Token) (b : BasisEntry)
    (hi : Inv s) (hb : b ∈ s.bases)
    (hmac : tok.verifyMac s.macKey = true)
    (hhome : tok.homeId = s.homeId)
    (hinc : tok.incarnation ≤ s.incarnation)
    (hschema : tok.schemaVer = s.supportedSchema)
    (hquery : tok.queryDigest = s.expectedQuery)
    (hcid : b.commitId = tok.commitId)
    (horig : b.branchOrigin = tok.branchOrigin)
    (hcomm : b.commitment = tok.commitment)
    (hret : b.retained = true) :
    (∃ s', resumeToken s tok = .admitted s' b.commitId (canonicalSuffix b.rows tok.afterKey) ∧
      ∀ b' ∈ (reclaimBasis (pruneBasis s' b.commitId) b.commitId).bases,
        b'.commitId = b.commitId → b'.reclaimed = false ∧ 0 < b'.activeLoans) ∧
    resumeToken (pruneBasis s b.commitId) tok = .historyUnavailable := by
  have hnotlt : ¬ (s.incarnation < tok.incarnation) := Nat.not_lt.mpr hinc
  have hnotrec : b.reclaimed = false := hi.retainedNotReclaimed b hb hret
  have hpin : 0 < b.pinId := hi.retainedHasPin b hb hret
  have hfind : s.bases.find? (fun x => x.commitId == tok.commitId) = some b :=
    find?_unique_eq hi.nodup hb hcid
  have hfind_self : s.bases.find? (fun x => x.commitId == b.commitId) = some b :=
    find?_unique hi.nodup hb
  constructor
  · refine ⟨{ s with bases := s.bases.map (bumpLoan b.commitId) }, ?_, ?_⟩
    · unfold resumeToken
      simp [hmac, hhome, hnotlt, hschema, hquery, hfind, BasisEntry.independentlyRetained,
        hret, hnotrec, hpin, horig, hcomm]
    · intro b' hb' hcid'
      have hs1_nd : ((s.bases.map (bumpLoan b.commitId)).map BasisEntry.commitId).Nodup := by
        rw [ids_map_eq (fun u => by unfold bumpLoan; split <;> rfl)]; exact hi.nodup
      have hs1_mem : bumpLoan b.commitId b ∈ s.bases.map (bumpLoan b.commitId) :=
        List.mem_map_of_mem hb
      have hs1_find : (s.bases.map (bumpLoan b.commitId)).find? (fun x => x.commitId == b.commitId) =
          some (bumpLoan b.commitId b) :=
        find?_unique_eq hs1_nd hs1_mem (by simp [bumpLoan])
      have hs2_bases : (pruneBasis { s with bases := s.bases.map (bumpLoan b.commitId) } b.commitId).bases =
          (s.bases.map (bumpLoan b.commitId)).map (markPrunedImmediate b.commitId) := by
        unfold pruneBasis; rw [hs1_find]
      have hs2_nd : (((s.bases.map (bumpLoan b.commitId)).map (markPrunedImmediate b.commitId)).map BasisEntry.commitId).Nodup := by
        rw [ids_map_eq (fun u => by unfold markPrunedImmediate; split <;> rfl)]; exact hs1_nd
      have hs2_mem : markPrunedImmediate b.commitId (bumpLoan b.commitId b) ∈
          (s.bases.map (bumpLoan b.commitId)).map (markPrunedImmediate b.commitId) :=
        List.mem_map_of_mem hs1_mem
      have hs2_find : ((s.bases.map (bumpLoan b.commitId)).map (markPrunedImmediate b.commitId)).find?
          (fun x => x.commitId == b.commitId) =
          some (markPrunedImmediate b.commitId (bumpLoan b.commitId b)) :=
        find?_unique_eq hs2_nd hs2_mem (by simp [markPrunedImmediate, bumpLoan])
      have hs3_bases : (reclaimBasis (pruneBasis { s with bases := s.bases.map (bumpLoan b.commitId) } b.commitId) b.commitId).bases =
          ((s.bases.map (bumpLoan b.commitId)).map (markPrunedImmediate b.commitId)).map (tryReclaim b.commitId) := by
        unfold reclaimBasis; rw [hs2_bases, hs2_find]
      rw [hs3_bases] at hb'
      simp only [List.mem_map] at hb'
      obtain ⟨x, ⟨y, ⟨z, hz, rfl⟩, rfl⟩, rfl⟩ := hb'
      have hzc : z.commitId = b.commitId := by
        have h1 : ∀ u, (tryReclaim b.commitId u).commitId = u.commitId := fun u => by
          unfold tryReclaim; split <;> rfl
        have h2 : ∀ u, (markPrunedImmediate b.commitId u).commitId = u.commitId := fun u => by
          unfold markPrunedImmediate; split <;> rfl
        have h3 : ∀ u, (bumpLoan b.commitId u).commitId = u.commitId := fun u => by
          unfold bumpLoan; split <;> rfl
        rw [h1, h2, h3] at hcid'; exact hcid'
      have hz_eq := eq_of_mem_commitId hi.nodup hz hb hzc
      subst hz_eq
      simp [bumpLoan, markPrunedImmediate, tryReclaim, hnotrec]
  · have hpr_bases : (pruneBasis s b.commitId).bases = s.bases.map (markPrunedImmediate b.commitId) := by
      unfold pruneBasis; rw [hfind_self]
    have hnd' : ((pruneBasis s b.commitId).bases.map BasisEntry.commitId).Nodup := by
      rw [hpr_bases, ids_map_eq (fun x => by unfold markPrunedImmediate; split <;> rfl)]
      exact hi.nodup
    have hmem' : markPrunedImmediate b.commitId b ∈ (pruneBasis s b.commitId).bases := by
      rw [hpr_bases]; exact List.mem_map_of_mem hb
    have hfind' : (pruneBasis s b.commitId).bases.find? (fun x => x.commitId == tok.commitId) =
        some (markPrunedImmediate b.commitId b) :=
      find?_unique_eq hnd' hmem' (by simp [markPrunedImmediate, hcid])
    unfold resumeToken
    rw [hfind']
    have hpr_meta :
        (pruneBasis s b.commitId).macKey = s.macKey ∧
        (pruneBasis s b.commitId).homeId = s.homeId ∧
        (pruneBasis s b.commitId).incarnation = s.incarnation ∧
        (pruneBasis s b.commitId).supportedSchema = s.supportedSchema ∧
        (pruneBasis s b.commitId).expectedQuery = s.expectedQuery := by
      unfold pruneBasis; rw [hfind_self]; exact ⟨rfl, rfl, rfl, rfl, rfl⟩
    simp [hpr_meta, hmac, hhome, hnotlt, hschema, hquery, markPrunedImmediate,
      BasisEntry.independentlyRetained]

/-- Exact reopen semantics (`reconcileOnOpen`): after reopen (all volatile loans
    dropped, crash cuts reconciled, unbound pins expired, `incarnation` incremented),
    `resumeToken` succeeds iff the basis is still independently retained, and returns
    `HistoryUnavailable` otherwise — NEVER silently rebinding to `HEAD`. -/
theorem restart_resumes_iff_independently_retained (s : Sys) (tok : Token) (b : BasisEntry)
    (hi : Inv s) (hb : b ∈ s.bases)
    (hmac : tok.verifyMac s.macKey = true)
    (hhome : tok.homeId = s.homeId)
    (hinc : tok.incarnation ≤ s.incarnation)
    (hschema : tok.schemaVer = s.supportedSchema)
    (hquery : tok.queryDigest = s.expectedQuery)
    (hcid : b.commitId = tok.commitId)
    (horig : b.branchOrigin = tok.branchOrigin)
    (hcomm : b.commitment = tok.commitment) :
    (b.retained = true →
      ∃ s', resumeToken (reconcileOnOpen s) tok =
        .admitted s' b.commitId (canonicalSuffix b.rows tok.afterKey)) ∧
    (b.retained = false →
      resumeToken (reconcileOnOpen s) tok = .historyUnavailable) := by
  have hnotlt : ¬ (s.incarnation + 1 < tok.incarnation) := by omega
  have hnd' : ((reconcileOnOpen s).bases.map BasisEntry.commitId).Nodup := by
    unfold reconcileOnOpen
    have hcid_rc : ∀ x : BasisEntry, (reconcileEntry x).commitId = x.commitId := fun _ => rfl
    rw [ids_map_eq hcid_rc]
    exact nodup_filter_map_commitId (fun b => decide (0 < b.pinId)) hi.nodup
  constructor
  · intro hret
    have hnotrec : b.reclaimed = false := hi.retainedNotReclaimed b hb hret
    have hpin : 0 < b.pinId := hi.retainedHasPin b hb hret
    have hmem_filt : b ∈ s.bases.filter (fun x => decide (0 < x.pinId)) :=
      List.mem_filter.mpr ⟨hb, decide_eq_true hpin⟩
    have hmem' : reconcileEntry b ∈ (reconcileOnOpen s).bases :=
      List.mem_map_of_mem hmem_filt
    have hfind' : (reconcileOnOpen s).bases.find? (fun x => x.commitId == tok.commitId) =
        some (reconcileEntry b) := by
      have hcid' : (reconcileEntry b).commitId = tok.commitId := hcid
      rw [← hcid']; exact find?_unique hnd' hmem'
    refine ⟨{ reconcileOnOpen s with
              bases := (reconcileOnOpen s).bases.map (bumpLoan (reconcileEntry b).commitId) }, ?_⟩
    unfold resumeToken
    rw [hfind']
    simp [reconcileOnOpen, reconcileEntry, hmac, hhome, hnotlt, hschema, hquery,
      BasisEntry.independentlyRetained, hret, hnotrec, hpin, horig, hcomm]
  · intro hnotret
    by_cases hpin : 0 < b.pinId
    · have hmem_filt : b ∈ s.bases.filter (fun x => decide (0 < x.pinId)) :=
        List.mem_filter.mpr ⟨hb, decide_eq_true hpin⟩
      have hmem' : reconcileEntry b ∈ (reconcileOnOpen s).bases :=
        List.mem_map_of_mem hmem_filt
      have hfind' : (reconcileOnOpen s).bases.find? (fun x => x.commitId == tok.commitId) =
          some (reconcileEntry b) := by
        have hcid' : (reconcileEntry b).commitId = tok.commitId := hcid
        rw [← hcid']; exact find?_unique hnd' hmem'
      unfold resumeToken
      rw [hfind']
      simp [reconcileOnOpen, reconcileEntry, hmac, hhome, hnotlt, hschema, hquery,
        BasisEntry.independentlyRetained, hnotret]
    · have hfind_none :
          (reconcileOnOpen s).bases.find? (fun x => x.commitId == tok.commitId) = none := by
        rw [List.find?_eq_none]
        intro y hy
        obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hy
        have hx_mem := (List.mem_filter.mp hx).1
        have hx_pin := decide_eq_true_eq.mp (List.mem_filter.mp hx).2
        simp only [reconcileEntry, beq_iff_eq]
        intro hxc
        have heq := eq_of_mem_commitId hi.nodup hx_mem hb (hxc.trans hcid.symm)
        subst heq
        exact hpin hx_pin
      unfold resumeToken
      rw [hfind_none]
      simp [reconcileOnOpen, hmac, hhome, hnotlt, hschema, hquery]

/-- Forged-MAC or cross-Home tokens are unconditionally rejected before admission. -/
theorem forged_or_cross_home_token_rejected (s : Sys) (tok : Token)
    (hbad : tok.verifyMac s.macKey = false ∨ tok.homeId ≠ s.homeId) :
    resumeToken s tok = .corruptStorage := by
  unfold resumeToken
  rcases hbad with hmac | hhome
  · simp [hmac]
  · cases hmac : tok.verifyMac s.macKey with
    | false => simp
    | true => simp [hhome]

/-- **`RetainedBasisCursorImmutable`** (Milestone 4 §9.1 row 6):
    Complete end-to-end safety law combining ALL seven core guarantees directly
    inside the registered headline theorem:
    1. Reachable reclaim-fence & pin invariant across ALL two-phase cuts (`R1`–`R3`, `C1`–`C4`, `P1`–`P2`),
    2. Exact post-reconcile `durablePins` ownership (`p.active = true → p.pinId ∈ boundPinIds`),
    3. Consolidation commitment invariance,
    4. Forged/cross-Home MAC refusal before admission,
    5. Atomic resume-vs-prune exclusivity under Store's reclaim fence,
    6. Exact reopen-iff-independently-retained (never HEAD rebind),
    7. Chunk-bound auxiliary index exactness & discard invariance (zero unproved assumptions). -/
theorem RetainedBasisCursorImmutable :
    (∀ {init s : Sys}, Reachable init s →
      (∀ b ∈ s.bases, 0 < b.activeLoans → b.reclaimed = false) ∧
      (∀ b ∈ s.bases, b.retained = true → b.reclaimed = false) ∧
      (∀ b ∈ s.bases, b.retained = true → 0 < b.pinId)) ∧
    (∀ s : Sys,
      (reconcileOnOpen s).pendingRetirePins = [] ∧
      (∀ p ∈ (reconcileOnOpen s).durablePins, p.active = true →
        p.pinId ∈ boundPinIds (reconcileOnOpen s).bases) ∧
      (∀ b ∈ (reconcileOnOpen s).bases, 0 < b.pinId ∧ b.prevPinId = none ∧
        b.stagedConsolidate = false ∧ b.activeLoans = 0)) ∧
    (∀ (α : Type) (hash : List Row → α) (chunks : List LayeredChunk)
      (newLocator : LayeredChunk → Nat),
      hash (basisCanonicalRows (consolidateBasis chunks newLocator)) =
        hash (basisCanonicalRows chunks)) ∧
    (∀ (s : Sys) (tok : Token),
      (tok.verifyMac s.macKey = false ∨ tok.homeId ≠ s.homeId) →
      resumeToken s tok = .corruptStorage) ∧
    (∀ (s : Sys) (tok : Token) (b : BasisEntry),
      Inv s → b ∈ s.bases → tok.verifyMac s.macKey = true →
      tok.homeId = s.homeId → tok.incarnation ≤ s.incarnation →
      tok.schemaVer = s.supportedSchema → tok.queryDigest = s.expectedQuery →
      b.commitId = tok.commitId → b.branchOrigin = tok.branchOrigin →
      b.commitment = tok.commitment → b.retained = true →
      (∃ s', resumeToken s tok = .admitted s' b.commitId (canonicalSuffix b.rows tok.afterKey) ∧
        ∀ b' ∈ (reclaimBasis (pruneBasis s' b.commitId) b.commitId).bases,
          b'.commitId = b.commitId → b'.reclaimed = false ∧ 0 < b'.activeLoans) ∧
      resumeToken (pruneBasis s b.commitId) tok = .historyUnavailable) ∧
    (∀ (s : Sys) (tok : Token) (b : BasisEntry),
      Inv s → b ∈ s.bases → tok.verifyMac s.macKey = true →
      tok.homeId = s.homeId → tok.incarnation ≤ s.incarnation →
      tok.schemaVer = s.supportedSchema → tok.queryDigest = s.expectedQuery →
      b.commitId = tok.commitId → b.branchOrigin = tok.branchOrigin →
      b.commitment = tok.commitment →
      (b.retained = true →
        ∃ s', resumeToken (reconcileOnOpen s) tok =
          .admitted s' b.commitId (canonicalSuffix b.rows tok.afterKey)) ∧
      (b.retained = false →
        resumeToken (reconcileOnOpen s) tok = .historyUnavailable)) ∧
    (∀ (chunks : List (List Row)) (pos : Option Nat),
      StrictlySorted chunks.flatten →
      resumeWithChunkIndex chunks pos = canonicalSuffix chunks.flatten pos ∧
      canonicalPrefix chunks.flatten pos ++ canonicalSuffix chunks.flatten pos = chunks.flatten) :=
  ⟨fun hr => ⟨hr.inv.loanNotReclaimed, hr.inv.retainedNotReclaimed, hr.inv.retainedHasPin⟩,
   reconcileOnOpen_pin_ownership_exact,
   fun _ hash chunks newLocator => consolidation_preserves_commitment hash chunks newLocator,
   forged_or_cross_home_token_rejected,
   resume_vs_prune_exclusive,
   restart_resumes_iff_independently_retained,
   canonical_resume_exact_after_index_discard⟩

/-! ## Explicit `M7` negative-mutant counterexample witnesses -/

/-- Mutant `M7a`: silently rebinds an unretained/pruned token to `HEAD`. -/
def resumeMutantRebindToHead (s : Sys) (tok : Token) : ResumeOutcome :=
  match resumeToken s tok with
  | .historyUnavailable =>
      .admitted s s.headCommit (canonicalSuffix s.headRows tok.afterKey)
  | other => other

def m7HeadRows : List Row := [⟨1, 999, [999]⟩]
def m7OldRows : List Row := [⟨1, 10, [10]⟩]
def m7PrunedSys : Sys :=
  { homeId := 1, macKey := 55, incarnation := 2, nextPinId := 5
    headCommit := 10, headRows := m7HeadRows
    supportedSchema := 1, expectedQuery := 42
    bases := [⟨1, 9, 77, 1, none, 0, false, false, 0, true, m7OldRows⟩]
    durablePins := [⟨1, 1, 0, false⟩]
    pendingRetirePins := [] }
def m7Token : Token := issueToken 55 1 1 9 1 77 1 42 none

theorem M7_rebind_to_head_violates_immutability :
    resumeToken m7PrunedSys m7Token = .historyUnavailable ∧
    resumeMutantRebindToHead m7PrunedSys m7Token = .admitted m7PrunedSys 10 m7HeadRows ∧
    m7HeadRows ≠ m7OldRows := by
  decide

/-- Mutant `M7b`: couples physical chunk `locator` into the semantic commitment. -/
def locatorCoupledCommitment (c : LayeredChunk) : Nat × List Row :=
  (c.locator, c.canonicalRows)

theorem M7_locator_coupled_commitment_breaks_consolidation :
    let c : LayeredChunk := ⟨1, [[⟨1, 10, [1]⟩], [⟨2, 20, [2]⟩]]⟩
    (c.consolidate 2).canonicalRows = c.canonicalRows ∧
    locatorCoupledCommitment (c.consolidate 2) ≠ locatorCoupledCommitment c := by
  decide

/-- Mutant `M7c`: resumes by physical list offset instead of canonical logical key. -/
def m7CanonicalRows : List Row := [⟨1, 10, []⟩, ⟨2, 20, []⟩]
def m7PhysicalIndexRows : List Row := [⟨2, 20, []⟩, ⟨1, 10, []⟩]

theorem M7_physical_offset_resume_breaks_after_index_discard :
    let page1FromIndex := m7PhysicalIndexRows.take 1
    let page2ByOffsetAfterDiscard := m7CanonicalRows.drop 1
    page1FromIndex = [⟨2, 20, []⟩] ∧
    page2ByOffsetAfterDiscard = [⟨2, 20, []⟩] ∧
    page1FromIndex ++ page2ByOffsetAfterDiscard ≠ m7CanonicalRows := by
  decide

/-- `M7d`: forged-MAC token (signed with attacker key `99` instead of Home key `55`)
    is rejected with `CorruptStorage` even when the target basis is actively retained. -/
def m7ActiveSys : Sys :=
  { homeId := 1, macKey := 55, incarnation := 1, nextPinId := 5
    headCommit := 10, headRows := m7HeadRows
    supportedSchema := 1, expectedQuery := 42
    bases := [⟨1, 9, 77, 2, none, 0, false, true, 0, false, m7OldRows⟩]
    durablePins := [⟨2, 1, 0, true⟩]
    pendingRetirePins := [] }

theorem M7_forged_mac_token_rejected_before_admission :
    let forged := issueToken 99 1 1 9 1 77 1 42 none
    resumeToken m7ActiveSys forged = .corruptStorage ∧
    resumeToken m7ActiveSys m7Token =
      .admitted { m7ActiveSys with bases := m7ActiveSys.bases.map (bumpLoan 1) } 1 m7OldRows := by
  decide

end StoreM4.RetainedBasisCursor
