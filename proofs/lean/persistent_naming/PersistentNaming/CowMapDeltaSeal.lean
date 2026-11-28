import PersistentNaming.CowMapForkCost

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/cow_map.rs:59-145 sha256:04b5f8a393f61c266156d421549d79bb09d4eeed782e7ee62cb9d57921f9a847
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/cow_map_diff.rs:17-261 sha256:a179af6e947c0b313705c1222a65753aa7245273334d957097d7e5c2e91bd062
-- proof-mirror: crates/worth-relational-lineage/src/lineage/naming_pages/store_delta.rs:84-226 sha256:be6f70b2fe9cb33a801a41120253c8a1d11f769f1a089e892ae5d1e3dd444e7d

/-!
# Structural-sharing cursor-stack `CowMap` diff & layered cold resolution (#480)

Models the production cursor-stack algorithm `CowMap::visit_diff_against_with_stats` /
`CowMap::diff_against_bounded` (`cow_map_diff.rs`) over explicit `NodeId`-tagged AVL
trees (`IdTree`, where `id` models `Arc::as_ptr` pointer identity and `h` models stored
AVL height), plus `ColdNamingStore::active_chain`, `get_root`, and `scan_root`
(`store_delta.rs`).

Every O(1) operation is charged explicitly — there are NO zero-cost recursive subtree
comparisons:
* `ptrEq` compares only `NodeId` (`id1 == id2`) in `1` step (`ptrChecks`), never
  recursing into subtrees.
* `pushLeft` charges `1` step (`pushSteps`) per pushed node.
* `dropSharedPrefix` charges `1` step (`ptrChecks`) per compared stack frame.
* Every non-shared node comparison charges `1` step (`visited`).

Main theorems:
* `cursorDiff_eq_bounded_listDiff` & `cursorDiff_sound_and_sorted`: under the
  pointer-sharing invariant `ValidSharing a b` (equal `NodeId` implies equal subtrees),
  `cursorDiff` produces the exact ordered `listDiff`, strictly sorted and reconstructing
  `idRead a` from `idRead b`, aborting iff the diff exceeds `limit`.
* `avl_height_le_two_log2`: every balanced `AVL` tree of `N` nodes has height
  `idHeight t ≤ 2 * Nat.log2 (idSize t + 1) + 1`.
* `avlInsert_preserves_invariants`: `avlInsert` (with real `rotLeft`/`rotRight`/`rebalance`
  allocating fresh `NodeId`s while sharing untouched subtrees) preserves `BST`, `AVL`,
  and `ValidSharing`.
* `cursorDiff_after_avlInsert_steps_bounded` & `cursorDiff_after_avlInsert_log_bound`:
  diffing after a balanced `avlInsert` visits `≤ idHeight t + 3` (`O(log N)`) non-shared
  nodes and executes `≤ 4 * (idHeight t + 3) * (idHeight t + 4)` (`O((log N)^2)`) total
  steps counting every `ptrEq` check, stack push, and node visit.
* `resolveActiveChain_eq_applyActiveChain` & `scanActiveChain_sound`:
  top-down `get_root` and merged `scan_root` across `active_chain` match layer-by-layer
  composition.
-/
namespace PersistentNaming.CowMapDeltaSeal

/-- One delta-root row operation: overwrite/insert (`put`) or shadow (`tombstone`). -/
inductive DeltaOp (α : Type) where
  | put (v : α)
  | tombstone
  deriving DecidableEq, Repr

/-- One structural diff item emitted by `CowMap::diff_against_bounded`. -/
inductive DiffItem (α : Type) where
  | set (k : Nat) (v : α) (prev : Option α)
  | removed (k : Nat) (prev : α)
  deriving DecidableEq, Repr

def DiffItem.key {α : Type} : DiffItem α → Nat
  | .set k _ _ => k
  | .removed k _ => k

/-- Explicit `Arc`-pointer-tagged persistent AVL tree (`cow_map.rs:Node`).
`id` models `Arc::as_ptr` pointer identity; `h` models stored AVL height. -/
inductive IdTree (α : Type) where
  | leaf
  | node (id : Nat) (h : Nat) (l : IdTree α) (k : Nat) (v : α) (r : IdTree α)
  deriving DecidableEq, Repr

/-- Height of an `IdTree`. -/
def idHeight {α : Type} : IdTree α → Nat
  | .leaf => 0
  | .node _ _ l _ _ r => max (idHeight l) (idHeight r) + 1

/-- Stored height field (mirrors `cow_map.rs:height`). -/
def storedHeight {α : Type} : IdTree α → Nat
  | .leaf => 0
  | .node _ h _ _ _ _ => h

/-- Node count of an `IdTree`. -/
def idSize {α : Type} : IdTree α → Nat
  | .leaf => 0
  | .node _ _ l _ _ r => idSize l + idSize r + 1

/-- BST lookup on `IdTree`. -/
def idRead {α : Type} : IdTree α → Nat → Option α
  | .leaf, _ => none
  | .node _ _ l k v r, q =>
    if q < k then idRead l q
    else if k < q then idRead r q
    else some v

/-- In-order traversal of an `IdTree`. -/
def inOrder {α : Type} : IdTree α → List (Nat × α)
  | .leaf => []
  | .node _ _ l k v r => inOrder l ++ (k, v) :: inOrder r

/-- `opt_ptr_eq` / `Arc::ptr_eq` (`cow_map_diff.rs:106-112`): O(1) pointer-identity
comparison on `NodeId` only (never inspects keys, values, or children). -/
def ptrEq {α : Type} : IdTree α → IdTree α → Bool
  | .leaf, .leaf => true
  | .node id1 _ _ _ _ _, .node id2 _ _ _ _ _ => decide (id1 = id2)
  | _, _ => false

theorem ptrEq_refl {α : Type} : ∀ (t : IdTree α), ptrEq t t = true
  | .leaf => rfl
  | .node _ _ _ _ _ _ => by simp [ptrEq]

/-- Subtree reachability relation. -/
def Subtree {α : Type} : IdTree α → IdTree α → Prop
  | s, t =>
    s = t ∨ match t with
      | .leaf => False
      | .node _ _ l _ _ r => Subtree s l ∨ Subtree s r

theorem Subtree.refl {α : Type} (t : IdTree α) : Subtree t t := by
  unfold Subtree; exact Or.inl rfl

theorem Subtree.left {α : Type} (id h k : Nat) (v : α) (l r : IdTree α) :
    Subtree l (.node id h l k v r) := by
  unfold Subtree; exact Or.inr (Or.inl (Subtree.refl l))

theorem Subtree.right {α : Type} (id h k : Nat) (v : α) (l r : IdTree α) :
    Subtree r (.node id h l k v r) := by
  unfold Subtree; exact Or.inr (Or.inr (Subtree.refl r))

theorem Subtree.trans {α : Type} : ∀ {s u t : IdTree α}, Subtree s u → Subtree u t → Subtree s t
  | s, u, .leaf, hsu, hut => by
    unfold Subtree at hut
    rcases hut with rfl | h
    · exact hsu
    · exact False.elim h
  | s, u, .node id h l k v r, hsu, hut => by
    unfold Subtree at hut
    rcases hut with rfl | (hul | hur)
    · exact hsu
    · unfold Subtree; exact Or.inr (Or.inl (Subtree.trans hsu hul))
    · unfold Subtree; exact Or.inr (Or.inr (Subtree.trans hsu hur))

/-- Sound `Arc::ptr_eq` pointer-sharing invariant between `a` and `b`:
whenever a subtree of `a` and a subtree of `b` share the same `NodeId`,
they are the exact same immutable tree node. -/
def ValidSharing {α : Type} (a b : IdTree α) : Prop :=
  ∀ s t, Subtree s a → Subtree t b → ptrEq s t = true → s = t

/-- Every node in `t` has `id < bound`. -/
def IdsLt {α : Type} (bound : Nat) : IdTree α → Prop
  | .leaf => True
  | .node id _ l _ _ r => id < bound ∧ IdsLt bound l ∧ IdsLt bound r

theorem IdsLt.mono {α : Type} {m n : Nat} (hmn : m ≤ n) :
    ∀ {t : IdTree α}, IdsLt m t → IdsLt n t
  | .leaf, _ => trivial
  | .node _ _ _ _ _ _, ⟨hid, hl, hr⟩ => ⟨Nat.lt_of_lt_of_le hid hmn, hl.mono hmn, hr.mono hmn⟩

theorem IdsLt.subtree {α : Type} {bound : Nat} :
    ∀ {s t : IdTree α}, Subtree s t → IdsLt bound t → IdsLt bound s
  | s, .leaf, hst, ht => by
    unfold Subtree at hst
    rcases hst with rfl | h
    · exact ht
    · exact False.elim h
  | s, .node id h l k v r, hst, ⟨hid, hl, hr⟩ => by
    unfold Subtree at hst
    rcases hst with rfl | (hsl | hsr)
    · exact ⟨hid, hl, hr⟩
    · exact IdsLt.subtree hsl hl
    · exact IdsLt.subtree hsr hr

/-- `push_left_arc` (`cow_map_diff.rs:95-104`): pushes the left spine onto the
cursor stack (head = top), charging `1` push step per pushed node. -/
def pushLeft {α : Type} : IdTree α → List (IdTree α) → List (IdTree α) × Nat
  | .leaf, s => (s, 0)
  | .node id h l k v r, s =>
    let p := pushLeft l (.node id h l k v r :: s)
    (p.1, p.2 + 1)

/-- Remaining in-order elements represented by a cursor stack. -/
def stackElems {α : Type} : List (IdTree α) → List (Nat × α)
  | [] => []
  | .leaf :: rest => stackElems rest
  | .node _ _ _ k v r :: rest => (k, v) :: (inOrder r ++ stackElems rest)

/-- Well-founded termination measure of a cursor stack. -/
def stackWeight {α : Type} : List (IdTree α) → Nat
  | [] => 0
  | .leaf :: rest => 1 + stackWeight rest
  | .node _ _ _ _ _ r :: rest => 1 + idSize r + stackWeight rest

/-- Every frame on stack `s` is a subtree of `root`. -/
def StackSubtree {α : Type} (root : IdTree α) (s : List (IdTree α)) : Prop :=
  ∀ x ∈ s, Subtree x root

theorem stackElems_pushLeft {α : Type} :
    ∀ (t : IdTree α) (s : List (IdTree α)),
      stackElems (pushLeft t s).1 = inOrder t ++ stackElems s
  | .leaf, _ => rfl
  | .node id h l k v r, s => by
    simp only [pushLeft, inOrder, stackElems_pushLeft l, stackElems, List.append_assoc, List.cons_append]

theorem stackWeight_pushLeft {α : Type} :
    ∀ (t : IdTree α) (s : List (IdTree α)),
      stackWeight (pushLeft t s).1 = idSize t + stackWeight s
  | .leaf, _ => by simp [pushLeft, idSize]
  | .node _ _ l _ _ r, s => by
    simp only [pushLeft, idSize, stackWeight_pushLeft l, stackWeight]
    omega

theorem pushLeft_steps_le_height {α : Type} :
    ∀ (t : IdTree α) (s : List (IdTree α)),
      (pushLeft t s).2 ≤ idHeight t
  | .leaf, _ => Nat.le_refl 0
  | .node id h l k v r, s => by
    have ih := pushLeft_steps_le_height l (.node id h l k v r :: s)
    simp only [pushLeft, idHeight]
    omega

theorem stackSubtree_pushLeft {α : Type} {root : IdTree α} :
    ∀ {t : IdTree α} {s : List (IdTree α)},
      Subtree t root → StackSubtree root s → StackSubtree root (pushLeft t s).1
  | .leaf, _, _, hs => hs
  | .node id h l k v r, s, ht, hs => by
    simp only [pushLeft]
    have hl : Subtree l root := Subtree.trans (Subtree.left id h k v l r) ht
    refine stackSubtree_pushLeft hl fun x hx => ?_
    rcases List.mem_cons.mp hx with rfl | hx'
    · exact ht
    · exact hs x hx'

/-- Shared-suffix truncation (`cow_map_diff.rs:150-163`), charging `1` O(1)
`ptrEq` check per compared frame (including the terminating mismatch check). -/
def dropSharedPrefix {α : Type} :
    List (IdTree α) → List (IdTree α) → List (IdTree α) × List (IdTree α) × Nat
  | a :: as, b :: bs =>
    if ptrEq a b then
      let p := dropSharedPrefix as bs
      (p.1, p.2.1, p.2.2 + 1)
    else
      (a :: as, b :: bs, 1)
  | as, bs => (as, bs, 0)

theorem stackWeight_dropSharedPrefix_le {α : Type} :
    ∀ (sa sb : List (IdTree α)),
      stackWeight (dropSharedPrefix sa sb).1 ≤ stackWeight sa ∧
      stackWeight (dropSharedPrefix sa sb).2.1 ≤ stackWeight sb
  | [], _ => ⟨Nat.le_refl _, Nat.le_refl _⟩
  | _ :: _, [] => ⟨Nat.le_refl _, Nat.le_refl _⟩
  | a :: as, b :: bs => by
    unfold dropSharedPrefix
    split
    · have ⟨h1, h2⟩ := stackWeight_dropSharedPrefix_le as bs
      cases a <;> cases b <;> simp only [stackWeight] <;> omega
    · exact ⟨Nat.le_refl _, Nat.le_refl _⟩

theorem stackSubtree_dropSharedPrefix {α : Type} {ra rb : IdTree α} :
    ∀ {sa sb : List (IdTree α)},
      StackSubtree ra sa → StackSubtree rb sb →
      StackSubtree ra (dropSharedPrefix sa sb).1 ∧ StackSubtree rb (dropSharedPrefix sa sb).2.1
  | [], _, hsa, hsb => ⟨hsa, hsb⟩
  | _ :: _, [], hsa, hsb => ⟨hsa, hsb⟩
  | a :: as, b :: bs, hsa, hsb => by
    unfold dropSharedPrefix
    split
    · exact stackSubtree_dropSharedPrefix
        (fun x hx => hsa x (List.mem_cons_of_mem _ hx))
        (fun x hx => hsb x (List.mem_cons_of_mem _ hx))
    · exact ⟨hsa, hsb⟩

/-- Exact execution trace of `CowMap::visit_diff_against_with_stats` /
`CowMap::diff_against_bounded_with_stats`. -/
structure DiffRun (α : Type) where
  val : Option (List (DiffItem α))
  visited : Nat
  ptrChecks : Nat
  pushSteps : Nat
  deriving DecidableEq, Repr

/-- Total explicit step cost charging every node visit, every O(1) `ptrEq`
check, and every `pushLeft` stack push. -/
def DiffRun.totalSteps {α : Type} (r : DiffRun α) : Nat :=
  r.visited + r.ptrChecks + r.pushSteps

/-- Cursor-stack diff loop (`cow_map_diff.rs:146-225`) with exact step counts
and remaining diff budget `rem`. -/
def cursorDiffLoop {α : Type} [DecidableEq α] :
    (sa sb : List (IdTree α)) → (rem : Nat) → DiffRun α
  | [], [], _ => { val := some [], visited := 0, ptrChecks := 0, pushSteps := 0 }
  | .leaf :: as, sb, rem => cursorDiffLoop as sb rem
  | [], .leaf :: bs, rem => cursorDiffLoop [] bs rem
  | .node id1 h1 l1 k1 v1 r1 :: as, .leaf :: bs, rem =>
    cursorDiffLoop (.node id1 h1 l1 k1 v1 r1 :: as) bs rem
  | .node id1 h1 l1 k1 v1 r1 :: as, .node id2 h2 l2 k2 v2 r2 :: bs, rem =>
    if ptrEq (.node id1 h1 l1 k1 v1 r1) (.node id2 h2 l2 k2 v2 r2) then
      let p := dropSharedPrefix as bs
      let res := cursorDiffLoop p.1 p.2.1 rem
      { res with ptrChecks := res.ptrChecks + 1 + p.2.2 }
    else if k1 < k2 then
      match rem with
      | 0 => { val := none, visited := 1, ptrChecks := 1, pushSteps := 0 }
      | r + 1 =>
        let pl := pushLeft r1 as
        let res := cursorDiffLoop pl.1 (.node id2 h2 l2 k2 v2 r2 :: bs) r
        { val := res.val.map (.set k1 v1 none :: ·)
          visited := res.visited + 1
          ptrChecks := res.ptrChecks + 1
          pushSteps := res.pushSteps + pl.2 }
    else if k2 < k1 then
      match rem with
      | 0 => { val := none, visited := 1, ptrChecks := 1, pushSteps := 0 }
      | r + 1 =>
        let pl := pushLeft r2 bs
        let res := cursorDiffLoop (.node id1 h1 l1 k1 v1 r1 :: as) pl.1 r
        { val := res.val.map (.removed k2 v2 :: ·)
          visited := res.visited + 1
          ptrChecks := res.ptrChecks + 1
          pushSteps := res.pushSteps + pl.2 }
    else if v1 = v2 then
      if ptrEq r1 r2 then
        let res := cursorDiffLoop as bs rem
        { res with visited := res.visited + 1, ptrChecks := res.ptrChecks + 2 }
      else
        let pl1 := pushLeft r1 as
        let pl2 := pushLeft r2 bs
        let res := cursorDiffLoop pl1.1 pl2.1 rem
        { val := res.val
          visited := res.visited + 1
          ptrChecks := res.ptrChecks + 2
          pushSteps := res.pushSteps + pl1.2 + pl2.2 }
    else
      match rem with
      | 0 => { val := none, visited := 1, ptrChecks := 1, pushSteps := 0 }
      | r + 1 =>
        if ptrEq r1 r2 then
          let res := cursorDiffLoop as bs r
          { val := res.val.map (.set k1 v1 (some v2) :: ·)
            visited := res.visited + 1
            ptrChecks := res.ptrChecks + 2
            pushSteps := res.pushSteps }
        else
          let pl1 := pushLeft r1 as
          let pl2 := pushLeft r2 bs
          let res := cursorDiffLoop pl1.1 pl2.1 r
          { val := res.val.map (.set k1 v1 (some v2) :: ·)
            visited := res.visited + 1
            ptrChecks := res.ptrChecks + 2
            pushSteps := res.pushSteps + pl1.2 + pl2.2 }
  | .node _ _ _ k1 v1 r1 :: as, [], rem =>
    match rem with
    | 0 => { val := none, visited := 1, ptrChecks := 0, pushSteps := 0 }
    | r + 1 =>
      let pl := pushLeft r1 as
      let res := cursorDiffLoop pl.1 [] r
      { val := res.val.map (.set k1 v1 none :: ·)
        visited := res.visited + 1
        ptrChecks := res.ptrChecks
        pushSteps := res.pushSteps + pl.2 }
  | [], .node _ _ _ k2 v2 r2 :: bs, rem =>
    match rem with
    | 0 => { val := none, visited := 1, ptrChecks := 0, pushSteps := 0 }
    | r + 1 =>
      let pl := pushLeft r2 bs
      let res := cursorDiffLoop [] pl.1 r
      { val := res.val.map (.removed k2 v2 :: ·)
        visited := res.visited + 1
        ptrChecks := res.ptrChecks
        pushSteps := res.pushSteps + pl.2 }
termination_by sa sb _ => stackWeight sa + stackWeight sb
decreasing_by
  all_goals
    simp_wf
    try (
      have ⟨h1, h2⟩ := stackWeight_dropSharedPrefix_le as bs
      simp only [stackWeight]
      omega)
    try (
      simp only [stackWeight_pushLeft, stackWeight]
      omega)
    try (
      simp only [stackWeight]
      omega)

/-- `CowMap::diff_against_bounded_with_stats` (`cow_map_diff.rs:125-248`). -/
def cursorDiff {α : Type} [DecidableEq α] (a b : IdTree α) (limit : Nat) : DiffRun α :=
  if ptrEq a b then
    { val := some [], visited := 0, ptrChecks := 1, pushSteps := 0 }
  else
    let pl1 := pushLeft a []
    let pl2 := pushLeft b []
    let res := cursorDiffLoop pl1.1 pl2.1 limit
    { res with ptrChecks := res.ptrChecks + 1, pushSteps := res.pushSteps + pl1.2 + pl2.2 }

/-- Canonical two-pointer ordered difference between two association lists. -/
def listDiff {α : Type} [DecidableEq α] :
    List (Nat × α) → List (Nat × α) → List (DiffItem α)
  | [], [] => []
  | (k, v) :: xs, [] => .set k v none :: listDiff xs []
  | [], (k, v) :: ys => .removed k v :: listDiff [] ys
  | (k1, v1) :: xs, (k2, v2) :: ys =>
    if k1 < k2 then .set k1 v1 none :: listDiff xs ((k2, v2) :: ys)
    else if k2 < k1 then .removed k2 v2 :: listDiff ((k1, v1) :: xs) ys
    else if v1 = v2 then listDiff xs ys
    else .set k1 v1 (some v2) :: listDiff xs ys

theorem listDiff_append_left_eq {α : Type} [DecidableEq α] :
    ∀ (p xs ys : List (Nat × α)),
      listDiff (p ++ xs) (p ++ ys) = listDiff xs ys
  | [], _, _ => rfl
  | (k, v) :: ps, xs, ys => by
    simp only [List.cons_append, listDiff, Nat.lt_irrefl, ite_false, ite_true]
    exact listDiff_append_left_eq ps xs ys

theorem listDiff_self {α : Type} [DecidableEq α] (xs : List (Nat × α)) :
    listDiff xs xs = [] := by
  have h := listDiff_append_left_eq xs [] []
  simpa [listDiff] using h

theorem listDiff_dropSharedPrefix {α : Type} [DecidableEq α] {ra rb : IdTree α}
    (hvs : ValidSharing ra rb) :
    ∀ {sa sb : List (IdTree α)},
      StackSubtree ra sa → StackSubtree rb sb →
      listDiff (stackElems (dropSharedPrefix sa sb).1) (stackElems (dropSharedPrefix sa sb).2.1) =
      listDiff (stackElems sa) (stackElems sb)
  | [], _, _, _ => rfl
  | _ :: _, [], _, _ => rfl
  | a :: as, b :: bs, hsa, hsb => by
    unfold dropSharedPrefix
    split
    · rename_i heq
      have hab : a = b := hvs a b (hsa a (List.mem_cons_self ..)) (hsb b (List.mem_cons_self ..)) heq
      subst hab
      rw [listDiff_dropSharedPrefix hvs
        (fun x hx => hsa x (List.mem_cons_of_mem _ hx))
        (fun x hx => hsb x (List.mem_cons_of_mem _ hx))]
      cases a with
      | leaf => rfl
      | node _ _ _ k v r =>
        simp only [stackElems]
        have h := listDiff_append_left_eq ((k, v) :: inOrder r) (stackElems as) (stackElems bs)
        simpa [List.cons_append] using h.symm
    · rfl

theorem cursorDiffLoop_val {α : Type} [DecidableEq α] {ra rb : IdTree α}
    (hvs : ValidSharing ra rb) :
    ∀ (sa sb : List (IdTree α)) (rem : Nat),
      StackSubtree ra sa → StackSubtree rb sb →
      (cursorDiffLoop sa sb rem).val =
        let d := listDiff (stackElems sa) (stackElems sb)
        if d.length ≤ rem then some d else none
  | [], [], rem, _, _ => by simp [cursorDiffLoop, stackElems, listDiff]
  | .leaf :: as, sb, rem, hsa, hsb => by
    unfold cursorDiffLoop
    exact cursorDiffLoop_val hvs as sb rem (fun x hx => hsa x (List.mem_cons_of_mem _ hx)) hsb
  | [], .leaf :: bs, rem, hsa, hsb => by
    unfold cursorDiffLoop
    exact cursorDiffLoop_val hvs [] bs rem hsa (fun x hx => hsb x (List.mem_cons_of_mem _ hx))
  | .node id1 h1 l1 k1 v1 r1 :: as, .leaf :: bs, rem, hsa, hsb => by
    unfold cursorDiffLoop
    exact cursorDiffLoop_val hvs (.node id1 h1 l1 k1 v1 r1 :: as) bs rem
      hsa (fun x hx => hsb x (List.mem_cons_of_mem _ hx))
  | .node id1 h1 l1 k1 v1 r1 :: as, .node id2 h2 l2 k2 v2 r2 :: bs, rem, hsa, hsb => by
    have ha : Subtree (.node id1 h1 l1 k1 v1 r1) ra := hsa _ (List.mem_cons_self ..)
    have hb : Subtree (.node id2 h2 l2 k2 v2 r2) rb := hsb _ (List.mem_cons_self ..)
    have has' : StackSubtree ra as := fun x hx => hsa x (List.mem_cons_of_mem _ hx)
    have hbs' : StackSubtree rb bs := fun x hx => hsb x (List.mem_cons_of_mem _ hx)
    have hr1 : Subtree r1 ra := Subtree.trans (Subtree.right id1 h1 k1 v1 l1 r1) ha
    have hr2 : Subtree r2 rb := Subtree.trans (Subtree.right id2 h2 k2 v2 l2 r2) hb
    unfold cursorDiffLoop
    by_cases heq : ptrEq (.node id1 h1 l1 k1 v1 r1) (.node id2 h2 l2 k2 v2 r2) = true
    · simp only [heq, ite_true]
      have ⟨hds1, hds2⟩ := stackSubtree_dropSharedPrefix has' hbs'
      rw [cursorDiffLoop_val hvs (dropSharedPrefix as bs).1 (dropSharedPrefix as bs).2.1 rem hds1 hds2]
      have hab := hvs _ _ ha hb heq
      injection hab with _ _ _ hk hv hr
      subst hk; subst hv; subst hr
      have hdrop := listDiff_dropSharedPrefix hvs has' hbs'
      have hpref := listDiff_append_left_eq ((k1, v1) :: inOrder r1) (stackElems as) (stackElems bs)
      simp only [List.cons_append] at hpref
      simp only [stackElems, hdrop, hpref]
    · simp only [heq, Bool.false_eq_true, ite_false]
      by_cases hlt1 : k1 < k2
      · simp only [hlt1, ite_true]
        have hld : listDiff (stackElems (.node id1 h1 l1 k1 v1 r1 :: as))
                            (stackElems (.node id2 h2 l2 k2 v2 r2 :: bs)) =
                   .set k1 v1 none :: listDiff (inOrder r1 ++ stackElems as)
                                               ((k2, v2) :: (inOrder r2 ++ stackElems bs)) := by
          simp [stackElems, listDiff, hlt1]
        rw [hld]
        cases rem with
        | zero => simp
        | succ r =>
          simp only [cursorDiffLoop_val hvs (pushLeft r1 as).1 (.node id2 h2 l2 k2 v2 r2 :: bs) r
              (stackSubtree_pushLeft hr1 has') hsb,
            stackElems_pushLeft, stackElems, List.length_cons, Nat.succ_le_succ_iff]
          split <;> rfl
      · simp only [hlt1, ite_false]
        by_cases hlt2 : k2 < k1
        · simp only [hlt2, ite_true]
          have hld : listDiff (stackElems (.node id1 h1 l1 k1 v1 r1 :: as))
                              (stackElems (.node id2 h2 l2 k2 v2 r2 :: bs)) =
                     .removed k2 v2 :: listDiff ((k1, v1) :: (inOrder r1 ++ stackElems as))
                                                (inOrder r2 ++ stackElems bs) := by
            simp [stackElems, listDiff, hlt1, hlt2]
          rw [hld]
          cases rem with
          | zero => simp
          | succ r =>
            simp only [cursorDiffLoop_val hvs (.node id1 h1 l1 k1 v1 r1 :: as) (pushLeft r2 bs).1 r
                hsa (stackSubtree_pushLeft hr2 hbs'),
              stackElems_pushLeft, stackElems, List.length_cons, Nat.succ_le_succ_iff]
            split <;> rfl
        · simp only [hlt2, ite_false]
          by_cases hv : v1 = v2
          · subst hv
            simp only [ite_true]
            have hld : listDiff (stackElems (.node id1 h1 l1 k1 v1 r1 :: as))
                                (stackElems (.node id2 h2 l2 k2 v1 r2 :: bs)) =
                       listDiff (inOrder r1 ++ stackElems as) (inOrder r2 ++ stackElems bs) := by
              simp [stackElems, listDiff, hlt1, hlt2]
            rw [hld]
            by_cases hreq : ptrEq r1 r2 = true
            · simp only [hreq, ite_true]
              have hrEq := hvs r1 r2 hr1 hr2 hreq
              subst hrEq
              rw [cursorDiffLoop_val hvs as bs rem has' hbs', listDiff_append_left_eq]
            · simp only [hreq, Bool.false_eq_true, ite_false]
              rw [cursorDiffLoop_val hvs (pushLeft r1 as).1 (pushLeft r2 bs).1 rem
                (stackSubtree_pushLeft hr1 has') (stackSubtree_pushLeft hr2 hbs'),
                stackElems_pushLeft, stackElems_pushLeft]
          · simp only [hv, ite_false]
            have hld : listDiff (stackElems (.node id1 h1 l1 k1 v1 r1 :: as))
                                (stackElems (.node id2 h2 l2 k2 v2 r2 :: bs)) =
                       .set k1 v1 (some v2) ::
                         listDiff (inOrder r1 ++ stackElems as) (inOrder r2 ++ stackElems bs) := by
              simp [stackElems, listDiff, hlt1, hlt2, hv]
            rw [hld]
            cases rem with
            | zero => simp
            | succ r =>
              by_cases hreq : ptrEq r1 r2 = true
              · simp only [hreq, ite_true]
                have hrEq := hvs r1 r2 hr1 hr2 hreq
                subst hrEq
                simp only [cursorDiffLoop_val hvs as bs r has' hbs', listDiff_append_left_eq,
                  List.length_cons, Nat.succ_le_succ_iff]
                split <;> rfl
              · simp only [hreq, Bool.false_eq_true, ite_false]
                simp only [cursorDiffLoop_val hvs (pushLeft r1 as).1 (pushLeft r2 bs).1 r
                    (stackSubtree_pushLeft hr1 has') (stackSubtree_pushLeft hr2 hbs'),
                  stackElems_pushLeft, List.length_cons, Nat.succ_le_succ_iff]
                split <;> rfl
  | .node id1 h1 l1 k1 v1 r1 :: as, [], rem, hsa, _ => by
    have ha : Subtree (.node id1 h1 l1 k1 v1 r1) ra := hsa _ (List.mem_cons_self ..)
    have has' : StackSubtree ra as := fun x hx => hsa x (List.mem_cons_of_mem _ hx)
    have hr1 : Subtree r1 ra := Subtree.trans (Subtree.right id1 h1 k1 v1 l1 r1) ha
    unfold cursorDiffLoop
    have hld : listDiff (stackElems (.node id1 h1 l1 k1 v1 r1 :: as)) (stackElems []) =
               .set k1 v1 none :: listDiff (inOrder r1 ++ stackElems as) [] := by
      simp [stackElems, listDiff]
    rw [hld]
    cases rem with
    | zero => simp
    | succ r =>
      simp only [cursorDiffLoop_val hvs (pushLeft r1 as).1 [] r
          (stackSubtree_pushLeft hr1 has') (fun _ hx => by cases hx),
        stackElems_pushLeft, stackElems, List.length_cons, Nat.succ_le_succ_iff]
      split <;> rfl
  | [], .node id2 h2 l2 k2 v2 r2 :: bs, rem, _, hsb => by
    have hb : Subtree (.node id2 h2 l2 k2 v2 r2) rb := hsb _ (List.mem_cons_self ..)
    have hbs' : StackSubtree rb bs := fun x hx => hsb x (List.mem_cons_of_mem _ hx)
    have hr2 : Subtree r2 rb := Subtree.trans (Subtree.right id2 h2 k2 v2 l2 r2) hb
    unfold cursorDiffLoop
    have hld : listDiff (stackElems []) (stackElems (.node id2 h2 l2 k2 v2 r2 :: bs)) =
               .removed k2 v2 :: listDiff [] (inOrder r2 ++ stackElems bs) := by
      simp [stackElems, listDiff]
    rw [hld]
    cases rem with
    | zero => simp
    | succ r =>
      simp only [cursorDiffLoop_val hvs [] (pushLeft r2 bs).1 r
          (fun _ hx => by cases hx) (stackSubtree_pushLeft hr2 hbs'),
        stackElems_pushLeft, stackElems, List.length_cons, Nat.succ_le_succ_iff]
      split <;> rfl
termination_by sa sb _ _ _ => stackWeight sa + stackWeight sb
decreasing_by
  all_goals
    simp_wf
    try (
      have ⟨h1, h2⟩ := stackWeight_dropSharedPrefix_le as bs
      simp only [stackWeight]
      omega)
    try (
      simp only [stackWeight_pushLeft, stackWeight]
      omega)
    try (
      simp only [stackWeight]
      omega)

/-- Exact ordered equivalence of `cursorDiff` (with O(1) `NodeId` skips) and
`listDiff` across arbitrary tree shapes whenever `ValidSharing a b` holds. -/
theorem cursorDiff_eq_bounded_listDiff {α : Type} [DecidableEq α]
    {a b : IdTree α} (hvs : ValidSharing a b) (limit : Nat) :
    (cursorDiff a b limit).val =
      let d := listDiff (inOrder a) (inOrder b)
      if d.length ≤ limit then some d else none := by
  unfold cursorDiff
  by_cases heq : ptrEq a b = true
  · have hab := hvs a b (Subtree.refl a) (Subtree.refl b) heq
    subst hab
    simp [heq, listDiff_self]
  · simp only [heq, Bool.false_eq_true, ite_false]
    rw [cursorDiffLoop_val hvs _ _ limit
      (stackSubtree_pushLeft (Subtree.refl a) (fun _ hx => by cases hx))
      (stackSubtree_pushLeft (Subtree.refl b) (fun _ hx => by cases hx)),
      stackElems_pushLeft, stackElems_pushLeft]
    simp [stackElems]

/-- Exact soundness and completeness of bounded budget refusal under `ValidSharing`. -/
theorem cursorDiff_budget_abort_iff {α : Type} [DecidableEq α]
    {a b : IdTree α} (hvs : ValidSharing a b) (limit : Nat) :
    (cursorDiff a b limit).val = none ↔ (listDiff (inOrder a) (inOrder b)).length > limit := by
  rw [cursorDiff_eq_bounded_listDiff hvs]
  dsimp only
  split <;> rename_i h
  · constructor
    · intro hc; cases hc
    · intro hgt; omega
  · constructor
    · intro _; omega
    · intro _; rfl

/-- Identical (`Arc::ptr_eq`) roots emit an empty diff with `0` node visits and
exactly `1` O(1) `ptrEq` check (`totalSteps = 1`). -/
theorem cursorDiff_skip_identical {α : Type} [DecidableEq α] (t : IdTree α) (limit : Nat) :
    (cursorDiff t t limit).val = some [] ∧
    (cursorDiff t t limit).visited = 0 ∧
    (cursorDiff t t limit).totalSteps = 1 := by
  unfold cursorDiff DiffRun.totalSteps
  simp [ptrEq_refl]

/-- All keys in an `IdTree` satisfy `P`. -/
def AllKeys {α : Type} (P : Nat → Prop) : IdTree α → Prop
  | .leaf => True
  | .node _ _ l k _ r => P k ∧ AllKeys P l ∧ AllKeys P r

/-- Binary search tree ordering invariant on `IdTree`. -/
def BST {α : Type} : IdTree α → Prop
  | .leaf => True
  | .node _ _ l k _ r => AllKeys (· < k) l ∧ AllKeys (k < ·) r ∧ BST l ∧ BST r

/-- AVL balance invariant (`|height l - height r| ≤ 1` and stored height matches). -/
def AVL {α : Type} : IdTree α → Prop
  | .leaf => True
  | .node _ h l _ _ r =>
    h = max (idHeight l) (idHeight r) + 1 ∧
    idHeight l ≤ idHeight r + 1 ∧
    idHeight r ≤ idHeight l + 1 ∧
    AVL l ∧ AVL r

/-- Minimum node count of an AVL tree of height `h` (Fibonacci recurrence). -/
def minAvlSize : Nat → Nat
  | 0 => 0
  | 1 => 1
  | h + 2 => minAvlSize (h + 1) + minAvlSize h + 1

theorem minAvlSize_mono : ∀ {a b : Nat}, a ≤ b → minAvlSize a ≤ minAvlSize b
  | 0, _, _ => Nat.zero_le _
  | 1, 0, h => by omega
  | 1, 1, _ => Nat.le_refl 1
  | 1, b + 2, _ => by
    have h1 := minAvlSize_mono (show 1 ≤ b + 1 by omega)
    simp only [minAvlSize]; omega
  | a + 2, b, hab => by
    cases b with
    | zero => omega
    | succ b' =>
      cases b' with
      | zero => omega
      | succ b'' =>
        have h1 := minAvlSize_mono (show a + 1 ≤ b'' + 1 by omega)
        have h2 := minAvlSize_mono (show a ≤ b'' by omega)
        simp only [minAvlSize]; omega

theorem avl_size_ge_minAvlSize {α : Type} :
    ∀ {t : IdTree α}, AVL t → minAvlSize (idHeight t) ≤ idSize t
  | .leaf, _ => Nat.le_refl 0
  | .node _ _ l _ _ r, ⟨_, hlr, hrl, hal, har⟩ => by
    have ihl := avl_size_ge_minAvlSize hal
    have ihr := avl_size_ge_minAvlSize har
    simp only [idHeight, idSize]
    by_cases h0 : max (idHeight l) (idHeight r) = 0
    · have hl0 : idHeight l = 0 := by omega
      have hr0 : idHeight r = 0 := by omega
      rw [h0]; simp [minAvlSize]
    · obtain ⟨m, hm⟩ := Nat.exists_eq_succ_of_ne_zero h0
      rw [hm]
      simp only [minAvlSize]
      by_cases hcmp : idHeight l ≥ idHeight r
      · have hl_eq : idHeight l = m + 1 := by omega
        have hr_ge : m ≤ idHeight r := by omega
        have hrm := minAvlSize_mono hr_ge
        rw [hl_eq] at ihl
        omega
      · have hr_eq : idHeight r = m + 1 := by omega
        have hl_ge : m ≤ idHeight l := by omega
        have hlm := minAvlSize_mono hl_ge
        rw [hr_eq] at ihr
        omega

theorem pow2_le_minAvlSize_succ : ∀ h : Nat, 2 ^ (h / 2) ≤ minAvlSize h + 1
  | 0 => by decide
  | 1 => by decide
  | h + 2 => by
    have h1 := pow2_le_minAvlSize_succ h
    have h2 := minAvlSize_mono (show h ≤ h + 1 by omega)
    have hdiv : (h + 2) / 2 = h / 2 + 1 := by omega
    rw [hdiv, Nat.pow_succ]
    simp only [minAvlSize]
    omega

/-- Exact `O(log N)` height bound for any balanced `AVL` tree. -/
theorem avl_height_le_two_log2 {α : Type} {t : IdTree α} (ha : AVL t) :
    idHeight t ≤ 2 * Nat.log2 (idSize t + 1) + 1 := by
  have h1 := pow2_le_minAvlSize_succ (idHeight t)
  have h2 := avl_size_ge_minAvlSize ha
  have hpow : 2 ^ (idHeight t / 2) ≤ idSize t + 1 := Nat.le_trans h1 (Nat.add_le_add_right h2 1)
  have hlog : idHeight t / 2 ≤ Nat.log2 (idSize t + 1) :=
    (Nat.le_log2 (by omega)).mpr hpow
  omega

/-- Strictly ascending keys above an optional lower bound. -/
def SortedKeysAbove {α : Type} (lo : Option Nat) : List (Nat × α) → Prop
  | [] => True
  | (k, _) :: rest => (match lo with | none => True | some m => m < k) ∧ SortedKeysAbove (some k) rest

def SortedKeys {α : Type} (xs : List (Nat × α)) : Prop := SortedKeysAbove none xs

/-- Strictly ascending diff-item keys above an optional lower bound. -/
def StrictlySortedDiffAbove {α : Type} (lo : Option Nat) : List (DiffItem α) → Prop
  | [] => True
  | item :: rest =>
    (match lo with | none => True | some m => m < item.key) ∧
    StrictlySortedDiffAbove (some item.key) rest

def StrictlySortedDiff {α : Type} (ds : List (DiffItem α)) : Prop :=
  StrictlySortedDiffAbove none ds

theorem sortedKeysAbove_mono {α : Type} {xs : List (Nat × α)} {lo₁ lo₂ : Option Nat}
    (h : SortedKeysAbove lo₂ xs)
    (hle : ∀ k, (match lo₂ with | none => True | some m => m < k) →
                (match lo₁ with | none => True | some m => m < k)) :
    SortedKeysAbove lo₁ xs := by
  cases xs with
  | nil => trivial
  | cons hd tl => exact ⟨hle hd.1 h.1, h.2⟩

theorem allKeys_inOrder {α : Type} {P : Nat → Prop} :
    ∀ {t : IdTree α}, AllKeys P t → ∀ p ∈ inOrder t, P p.1
  | .leaf, _, _, hp => by simp [inOrder] at hp
  | .node _ _ l k v r, ⟨hk, hl, hr⟩, p, hp => by
    simp only [inOrder, List.mem_append, List.mem_cons] at hp
    rcases hp with h | rfl | h
    · exact allKeys_inOrder hl p h
    · exact hk
    · exact allKeys_inOrder hr p h

theorem sortedKeysAbove_append_mid {α : Type} :
    ∀ {xs ys : List (Nat × α)} {lo : Option Nat} {mid : Nat} {v : α},
      SortedKeysAbove lo xs →
      (∀ p ∈ xs, p.1 < mid) →
      SortedKeysAbove (some mid) ys →
      (match lo with | none => True | some m => m < mid) →
      SortedKeysAbove lo (xs ++ (mid, v) :: ys)
  | [], _, _, _, _, _, _, hys, hmid => ⟨hmid, hys⟩
  | (k, kv) :: _xs, _ys, _, _mid, _v, ⟨hk, hxs⟩, hlt, hys, _ =>
    ⟨hk, sortedKeysAbove_append_mid hxs
      (fun p hp => hlt p (List.mem_cons_of_mem _ hp))
      hys
      (hlt (k, kv) (List.mem_cons_self ..))⟩

theorem inOrder_sortedAbove {α : Type} :
    ∀ {t : IdTree α} {lo : Option Nat},
      BST t →
      (match lo with | none => True | some m => AllKeys (m < ·) t) →
      SortedKeysAbove lo (inOrder t)
  | .leaf, _, _, _ => trivial
  | .node _ _ l k _v r, none, ⟨hlk, hkr, hbl, hbr⟩, _ => by
    simp only [inOrder]
    exact sortedKeysAbove_append_mid
      (inOrder_sortedAbove hbl trivial)
      (allKeys_inOrder hlk)
      (inOrder_sortedAbove hbr hkr)
      trivial
  | .node _ _ l k _v r, some m, ⟨hlk, hkr, hbl, hbr⟩, ⟨hmk, hml, _⟩ => by
    simp only [inOrder]
    exact sortedKeysAbove_append_mid
      (inOrder_sortedAbove hbl hml)
      (allKeys_inOrder hlk)
      (inOrder_sortedAbove hbr hkr)
      hmk

theorem inOrder_sorted {α : Type} {t : IdTree α} (hb : BST t) : SortedKeys (inOrder t) :=
  inOrder_sortedAbove hb trivial

/-- Association-list lookup. -/
def listLookup {α : Type} : List (Nat × α) → Nat → Option α
  | [], _ => none
  | (k, v) :: rest, q => if q = k then some v else listLookup rest q

theorem listLookup_append {α : Type} :
    ∀ (xs ys : List (Nat × α)) (q : Nat),
      listLookup (xs ++ ys) q = match listLookup xs q with | some v => some v | none => listLookup ys q
  | [], _, _ => rfl
  | (k, v) :: xs, ys, q => by
    simp only [List.cons_append, listLookup]
    by_cases h : q = k
    · simp [h]
    · simp only [h, ite_false]; exact listLookup_append xs ys q

theorem listLookup_none_of_not_mem {α : Type} :
    ∀ {xs : List (Nat × α)} {q : Nat}, (∀ p ∈ xs, p.1 ≠ q) → listLookup xs q = none
  | [], _, _ => rfl
  | (k, v) :: xs, q, h => by
    have hne : q ≠ k := Ne.symm (h (k, v) (List.mem_cons_self ..))
    simp only [listLookup, hne, ite_false]
    exact listLookup_none_of_not_mem fun p hp => h p (List.mem_cons_of_mem _ hp)

theorem listLookup_inOrder {α : Type} :
    ∀ {t : IdTree α}, BST t → ∀ q, listLookup (inOrder t) q = idRead t q
  | .leaf, _, _ => rfl
  | .node _ _ l k v r, ⟨hlk, hkr, hbl, hbr⟩, q => by
    simp only [inOrder, listLookup_append, listLookup, idRead]
    by_cases h1 : q < k
    · simp only [h1, ite_true]
      rw [listLookup_inOrder hbl q]
      cases idRead l q with
      | some _ => rfl
      | none =>
        have hne : q ≠ k := by omega
        simp only [hne, ite_false]
        exact listLookup_none_of_not_mem fun p hp => by
          have hpk : k < p.1 := allKeys_inOrder hkr p hp
          omega
    · by_cases h2 : k < q
      · simp only [h1, h2, ite_true, ite_false]
        have hl0 : listLookup (inOrder l) q = none :=
          listLookup_none_of_not_mem fun p hp => by
            have hpk : p.1 < k := allKeys_inOrder hlk p hp
            omega
        have hne : q ≠ k := by omega
        simp only [hl0, hne, ite_false]
        exact listLookup_inOrder hbr q
      · have hqk : q = k := by omega
        subst hqk
        have hl0 : listLookup (inOrder l) q = none :=
          listLookup_none_of_not_mem fun p hp => by
            have hpk : p.1 < q := allKeys_inOrder hlk p hp
            omega
        simp [h1, hl0]

/-- Applying an ordered diff list over `base` at key `q`. -/
def applyDiffItems {α : Type} (base : Nat → Option α) : List (DiffItem α) → Nat → Option α
  | [], q => base q
  | .set k v _ :: rest, q => if q = k then some v else applyDiffItems base rest q
  | .removed k _ :: rest, q => if q = k then none else applyDiffItems base rest q

theorem listDiff_strictly_sortedAbove {α : Type} [DecidableEq α] :
    ∀ {xs ys : List (Nat × α)} {lo : Option Nat},
      SortedKeysAbove lo xs → SortedKeysAbove lo ys →
      StrictlySortedDiffAbove lo (listDiff xs ys)
  | [], [], _, _, _ => by simp [listDiff, StrictlySortedDiffAbove]
  | (k, v) :: xs, [], _, ⟨hk, hxs⟩, _ => by
    simp only [listDiff, StrictlySortedDiffAbove, DiffItem.key]
    exact ⟨hk, listDiff_strictly_sortedAbove (ys := []) (lo := some k) hxs trivial⟩
  | [], (k, v) :: ys, _, _, ⟨hk, hys⟩ => by
    simp only [listDiff, StrictlySortedDiffAbove, DiffItem.key]
    exact ⟨hk, listDiff_strictly_sortedAbove (xs := []) (lo := some k) trivial hys⟩
  | (k1, v1) :: xs, (k2, v2) :: ys, lo, ⟨hk1, hxs⟩, ⟨hk2, hys⟩ => by
    unfold listDiff
    by_cases h1 : k1 < k2
    · simp only [h1, ite_true, StrictlySortedDiffAbove, DiffItem.key]
      exact ⟨hk1, listDiff_strictly_sortedAbove hxs ⟨h1, hys⟩⟩
    · by_cases h2 : k2 < k1
      · simp only [h1, h2, ite_true, ite_false, StrictlySortedDiffAbove, DiffItem.key]
        exact ⟨hk2, listDiff_strictly_sortedAbove ⟨h2, hxs⟩ hys⟩
      · have heq : k1 = k2 := by omega
        subst heq
        simp only [h1, ite_false]
        by_cases hv : v1 = v2
        · simp only [hv, ite_true]
          have hsub := listDiff_strictly_sortedAbove hxs hys
          cases hld : listDiff xs ys with
          | nil => trivial
          | cons hd tl =>
            rw [hld] at hsub
            refine ⟨?_, hsub.2⟩
            cases lo with
            | none => trivial
            | some m => exact Nat.lt_trans hk1 hsub.1
        · simp only [hv, ite_false, StrictlySortedDiffAbove, DiffItem.key]
          exact ⟨hk1, listDiff_strictly_sortedAbove hxs hys⟩

theorem sortedKeysAbove_gt {α : Type} :
    ∀ {xs : List (Nat × α)} {m : Nat},
      SortedKeysAbove (some m) xs → ∀ p ∈ xs, m < p.1
  | [], _, _, _, hp => by cases hp
  | (k, v) :: xs, m, ⟨hmk, hxs⟩, p, hp => by
    rcases List.mem_cons.mp hp with rfl | h
    · exact hmk
    · exact Nat.lt_trans hmk (sortedKeysAbove_gt hxs p h)

theorem listLookup_none_of_sortedAbove {α : Type} {xs : List (Nat × α)} {m q : Nat}
    (h : SortedKeysAbove (some m) xs) (hq : q ≤ m) :
    listLookup xs q = none :=
  listLookup_none_of_not_mem fun p hp => by
    have := sortedKeysAbove_gt h p hp; omega

theorem applyDiffItems_congr_base {α : Type} {f g : Nat → Option α} {q : Nat}
    (hfg : f q = g q) :
    ∀ ds : List (DiffItem α), applyDiffItems f ds q = applyDiffItems g ds q
  | [] => hfg
  | .set k v _ :: rest => by
    simp only [applyDiffItems]; split
    · rfl
    · exact applyDiffItems_congr_base hfg rest
  | .removed k _ :: rest => by
    simp only [applyDiffItems]; split
    · rfl
    · exact applyDiffItems_congr_base hfg rest

theorem apply_listDiff_reconstructs {α : Type} [DecidableEq α] :
    ∀ {xs ys : List (Nat × α)} {lo : Option Nat},
      SortedKeysAbove lo xs → SortedKeysAbove lo ys →
      ∀ q, applyDiffItems (listLookup ys) (listDiff xs ys) q = listLookup xs q
  | [], [], _, _, _, _ => by simp [listDiff, applyDiffItems]
  | (k, v) :: xs, [], _, ⟨_, hxs⟩, _, q => by
    simp only [listDiff, applyDiffItems, listLookup]
    by_cases h : q = k
    · simp [h]
    · simp only [h, ite_false]
      exact apply_listDiff_reconstructs (ys := []) (lo := some k) hxs trivial q
  | [], (k, v) :: ys, _, _, ⟨_, hys⟩, q => by
    simp only [listDiff, applyDiffItems, listLookup]
    by_cases h : q = k
    · simp [h]
    · simp only [h, ite_false]
      have ih := apply_listDiff_reconstructs (xs := []) (lo := some k) trivial hys q
      have hsub := applyDiffItems_congr_base (q := q)
        (f := fun x => if x = k then some v else listLookup ys x)
        (g := listLookup ys) (by simp [h]) (listDiff [] ys)
      exact hsub.trans ih
  | (k1, v1) :: xs, (k2, v2) :: ys, _, ⟨_, hxs⟩, ⟨_, hys⟩, q => by
    unfold listDiff
    by_cases h1 : k1 < k2
    · simp only [h1, ite_true, applyDiffItems, listLookup]
      by_cases hq : q = k1
      · simp [hq]
      · simp only [hq, ite_false]
        have hys' : SortedKeysAbove (some k1) ((k2, v2) :: ys) := ⟨h1, hys⟩
        exact apply_listDiff_reconstructs hxs hys' q
    · by_cases h2 : k2 < k1
      · simp only [h1, h2, ite_true, ite_false, applyDiffItems, listLookup]
        by_cases hq : q = k2
        · subst hq
          have hne : q ≠ k1 := by omega
          simp only [ite_true, hne, ite_false]
          exact (listLookup_none_of_sortedAbove hxs (by omega)).symm
        · simp only [hq, ite_false]
          have hxs' : SortedKeysAbove (some k2) ((k1, v1) :: xs) := ⟨h2, hxs⟩
          have ih := apply_listDiff_reconstructs hxs' hys q
          have hsub := applyDiffItems_congr_base (q := q)
            (f := fun x => if x = k2 then some v2 else listLookup ys x)
            (g := listLookup ys) (by simp [hq]) (listDiff ((k1, v1) :: xs) ys)
          simpa [listLookup] using hsub.trans ih
      · have heq : k1 = k2 := by omega
        subst heq
        simp only [h1, ite_false]
        by_cases hv : v1 = v2
        · subst hv
          simp only [ite_true]
          by_cases hq : q = k1
          · subst hq
            have hmem : ∀ ds, StrictlySortedDiffAbove (some q) ds →
                applyDiffItems (listLookup ((q, v1) :: ys)) ds q = some v1 := by
              intro ds hds
              induction ds with
              | nil => simp [applyDiffItems, listLookup]
              | cons d rest ihr =>
                cases d with
                | set k' v' _ =>
                  have hlt : q < k' := hds.1
                  have hne : q ≠ k' := by omega
                  simp only [applyDiffItems, hne, ite_false]
                  exact ihr (by
                    cases rest with
                    | nil => trivial
                    | cons r1 rs => exact ⟨Nat.lt_trans hlt hds.2.1, hds.2.2⟩)
                | removed k' _ =>
                  have hlt : q < k' := hds.1
                  have hne : q ≠ k' := by omega
                  simp only [applyDiffItems, hne, ite_false]
                  exact ihr (by
                    cases rest with
                    | nil => trivial
                    | cons r1 rs => exact ⟨Nat.lt_trans hlt hds.2.1, hds.2.2⟩)
            rw [hmem _ (listDiff_strictly_sortedAbove hxs hys)]
            simp [listLookup]
          · have ih := apply_listDiff_reconstructs hxs hys q
            have hsub := applyDiffItems_congr_base (q := q)
              (f := listLookup ((k1, v1) :: ys)) (g := listLookup ys)
              (by simp [listLookup, hq]) (listDiff xs ys)
            rw [hsub, ih]
            simp [listLookup, hq]
        · simp only [hv, ite_false, applyDiffItems, listLookup]
          by_cases hq : q = k1
          · simp [hq]
          · simp only [hq, ite_false]
            have ih := apply_listDiff_reconstructs hxs hys q
            have hsub := applyDiffItems_congr_base (q := q)
              (f := fun x => if x = k1 then some v2 else listLookup ys x) (g := listLookup ys)
              (by simp [hq]) (listDiff xs ys)
            exact hsub.trans ih

/-- Soundness and strict key ordering of `cursorDiff` for ANY two BST `IdTree`s
satisfying `ValidSharing` (including AVL-rotated trees with distinct upper spines). -/
theorem cursorDiff_sound_and_sorted {α : Type} [DecidableEq α]
    {fork base : IdTree α} (hf : BST fork) (hb : BST base) (hvs : ValidSharing fork base)
    {limit : Nat} {diffs : List (DiffItem α)}
    (h : (cursorDiff fork base limit).val = some diffs) :
    diffs.length ≤ limit ∧
    StrictlySortedDiff diffs ∧
    (∀ q, applyDiffItems (idRead base) diffs q = idRead fork q) := by
  rw [cursorDiff_eq_bounded_listDiff hvs] at h
  dsimp only at h
  split at h
  · rename_i hlen
    injection h with hEq
    subst hEq
    refine ⟨hlen, listDiff_strictly_sortedAbove (inOrder_sorted hf) (inOrder_sorted hb), fun q => ?_⟩
    have hbase : idRead base = listLookup (inOrder base) := by
      funext x; exact (listLookup_inOrder hb x).symm
    rw [hbase, apply_listDiff_reconstructs (inOrder_sorted hf) (inOrder_sorted hb) q,
      listLookup_inOrder hf q]
  · cases h

/-!
## Explicit-cost monotonicity, shared-subtree skip lemmas, and AVL insert cost
-/

theorem cursorDiffLoop_cost_mono {α : Type} [DecidableEq α] :
    ∀ (sa sb : List (IdTree α)) {r1 r2 : Nat},
      r1 ≤ r2 →
      (cursorDiffLoop sa sb r1).visited ≤ (cursorDiffLoop sa sb r2).visited ∧
      (cursorDiffLoop sa sb r1).ptrChecks ≤ (cursorDiffLoop sa sb r2).ptrChecks ∧
      (cursorDiffLoop sa sb r1).pushSteps ≤ (cursorDiffLoop sa sb r2).pushSteps
  | [], [], _, _, _ => by simp [cursorDiffLoop]
  | .leaf :: as, sb, _, _, hr => by
    unfold cursorDiffLoop; exact cursorDiffLoop_cost_mono as sb hr
  | [], .leaf :: bs, _, _, hr => by
    unfold cursorDiffLoop; exact cursorDiffLoop_cost_mono [] bs hr
  | .node id1 h1 l1 k1 v1 r1t :: as, .leaf :: bs, _, _, hr => by
    unfold cursorDiffLoop; exact cursorDiffLoop_cost_mono (.node id1 h1 l1 k1 v1 r1t :: as) bs hr
  | .node id1 h1 l1 k1 v1 r1t :: as, .node id2 h2 l2 k2 v2 r2t :: bs, r1, r2, hr => by
    unfold cursorDiffLoop
    by_cases heq : ptrEq (.node id1 h1 l1 k1 v1 r1t) (.node id2 h2 l2 k2 v2 r2t) = true
    · simp only [heq, ite_true]
      have ⟨h1, h2, h3⟩ := cursorDiffLoop_cost_mono (dropSharedPrefix as bs).1 (dropSharedPrefix as bs).2.1 hr
      exact ⟨h1, by omega, h3⟩
    · simp only [heq, Bool.false_eq_true, ite_false]
      by_cases hk1 : k1 < k2
      · simp only [hk1, ite_true]
        cases r1 with
        | zero => cases r2 <;> simp
        | succ m1 =>
          cases r2 with
          | zero => omega
          | succ m2 =>
            have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono
              (pushLeft r1t as).1 (.node id2 h2 l2 k2 v2 r2t :: bs) (Nat.le_of_succ_le_succ hr)
            simp only; exact ⟨by omega, by omega, by omega⟩
      · simp only [hk1, ite_false]
        by_cases hk2 : k2 < k1
        · simp only [hk2, ite_true]
          cases r1 with
          | zero => cases r2 <;> simp
          | succ m1 =>
            cases r2 with
            | zero => omega
            | succ m2 =>
              have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono
                (.node id1 h1 l1 k1 v1 r1t :: as) (pushLeft r2t bs).1 (Nat.le_of_succ_le_succ hr)
              simp only; exact ⟨by omega, by omega, by omega⟩
        · simp only [hk2, ite_false]
          by_cases hv : v1 = v2
          · simp only [hv, ite_true]
            by_cases hreq : ptrEq r1t r2t = true
            · simp only [hreq, ite_true]
              have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono as bs hr
              exact ⟨by omega, by omega, c3⟩
            · simp only [hreq, Bool.false_eq_true, ite_false]
              have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono (pushLeft r1t as).1 (pushLeft r2t bs).1 hr
              exact ⟨by omega, by omega, by omega⟩
          · simp only [hv, ite_false]
            cases r1 with
            | zero =>
              cases r2 with
              | zero => simp
              | succ _ =>
                by_cases hreq : ptrEq r1t r2t = true <;> simp [hreq]
            | succ m1 =>
              cases r2 with
              | zero => omega
              | succ m2 =>
                by_cases hreq : ptrEq r1t r2t = true
                · simp only [hreq, ite_true]
                  have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono as bs (Nat.le_of_succ_le_succ hr)
                  exact ⟨by omega, by omega, c3⟩
                · simp only [hreq, Bool.false_eq_true, ite_false]
                  have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono
                    (pushLeft r1t as).1 (pushLeft r2t bs).1 (Nat.le_of_succ_le_succ hr)
                  exact ⟨by omega, by omega, by omega⟩
  | .node _ _ _ _ _ r1t :: as, [], r1, r2, hr => by
    unfold cursorDiffLoop
    cases r1 with
    | zero => cases r2 <;> simp
    | succ m1 =>
      cases r2 with
      | zero => omega
      | succ m2 =>
        have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono (pushLeft r1t as).1 [] (Nat.le_of_succ_le_succ hr)
        simp only; exact ⟨by omega, c2, by omega⟩
  | [], .node _ _ _ _ _ r2t :: bs, r1, r2, hr => by
    unfold cursorDiffLoop
    cases r1 with
    | zero => cases r2 <;> simp
    | succ m1 =>
      cases r2 with
      | zero => omega
      | succ m2 =>
        have ⟨c1, c2, c3⟩ := cursorDiffLoop_cost_mono [] (pushLeft r2t bs).1 (Nat.le_of_succ_le_succ hr)
        simp only; exact ⟨by omega, c2, by omega⟩
termination_by sa sb _ _ _ => stackWeight sa + stackWeight sb
decreasing_by
  all_goals
    simp_wf
    try (
      have ⟨h1, h2⟩ := stackWeight_dropSharedPrefix_le as bs
      simp only [stackWeight]
      omega)
    try (
      simp only [stackWeight_pushLeft, stackWeight]
      omega)
    try (
      simp only [stackWeight]
      omega)

theorem cursorDiffLoop_leaf_left {α : Type} [DecidableEq α]
    (as sb : List (IdTree α)) (rem : Nat) :
    cursorDiffLoop (.leaf :: as) sb rem = cursorDiffLoop as sb rem := by
  rw [cursorDiffLoop.eq_def]

theorem cursorDiffLoop_leaf_right {α : Type} [DecidableEq α] :
    ∀ (sa bs : List (IdTree α)) (rem : Nat),
      cursorDiffLoop sa (.leaf :: bs) rem = cursorDiffLoop sa bs rem
  | [], _, _ => by rw [cursorDiffLoop.eq_def]
  | .leaf :: as, bs, rem => by
    rw [cursorDiffLoop_leaf_left, cursorDiffLoop_leaf_left]
    exact cursorDiffLoop_leaf_right as bs rem
  | .node _ _ _ _ _ _ :: _, _, _ => by rw [cursorDiffLoop.eq_def]

/-- No `.leaf` appears on a stack built by `pushLeft`. -/
def NoLeaf {α : Type} (s : List (IdTree α)) : Prop :=
  ∀ x ∈ s, x ≠ .leaf

theorem noLeaf_pushLeft {α : Type} :
    ∀ (t : IdTree α) {s : List (IdTree α)}, NoLeaf s → NoLeaf (pushLeft t s).1
  | .leaf, _, hs => hs
  | .node id h l k v r, s, hs =>
    noLeaf_pushLeft l fun x hx => by
      rcases List.mem_cons.mp hx with rfl | hx'
      · intro hc; cases hc
      · exact hs x hx'

/-- `dropSharedPrefix` on leaf-free stacks skips shared frames with `0` extra
node visits and `0` extra pushes, charging at most `1` extra `ptrEq` check
for the terminating mismatch. -/
theorem cursorDiffLoop_dropSharedPrefix_cost_noLeaf {α : Type} [DecidableEq α] :
    ∀ (sa sb : List (IdTree α)) (rem : Nat),
      NoLeaf sa → NoLeaf sb →
      (cursorDiffLoop (dropSharedPrefix sa sb).1 (dropSharedPrefix sa sb).2.1 rem).visited =
        (cursorDiffLoop sa sb rem).visited ∧
      (cursorDiffLoop (dropSharedPrefix sa sb).1 (dropSharedPrefix sa sb).2.1 rem).ptrChecks +
          (dropSharedPrefix sa sb).2.2 ≤ (cursorDiffLoop sa sb rem).ptrChecks + 1 ∧
      (cursorDiffLoop (dropSharedPrefix sa sb).1 (dropSharedPrefix sa sb).2.1 rem).pushSteps =
        (cursorDiffLoop sa sb rem).pushSteps
  | [], _, _, _, _ => by simp [dropSharedPrefix]
  | _ :: _, [], _, _, _ => by simp [dropSharedPrefix]
  | a :: as, b :: bs, rem, hsa, hsb => by
    unfold dropSharedPrefix
    by_cases heq : ptrEq a b = true
    · simp only [heq, ite_true]
      cases a with
      | leaf => exact False.elim (hsa .leaf (List.mem_cons_self ..) rfl)
      | node id1 h1 l1 k1 v1 r1 =>
        cases b with
        | leaf => exact False.elim (hsb .leaf (List.mem_cons_self ..) rfl)
        | node id2 h2 l2 k2 v2 r2 =>
          have hstep : cursorDiffLoop (.node id1 h1 l1 k1 v1 r1 :: as)
                                      (.node id2 h2 l2 k2 v2 r2 :: bs) rem =
            let p := dropSharedPrefix as bs
            let res := cursorDiffLoop p.1 p.2.1 rem
            { res with ptrChecks := res.ptrChecks + 1 + p.2.2 } := by
            rw [cursorDiffLoop.eq_def]; simp [heq]
          rw [hstep]
          exact ⟨rfl, by dsimp only; omega, rfl⟩
    · simp only [heq, Bool.false_eq_true, ite_false]
      exact ⟨trivial, by omega, trivial⟩

/-- Dropping the shared left spine of `t` from `(pushLeft t sa, pushLeft t sb)`
reaches `dropSharedPrefix sa sb` using exactly `(pushLeft t sa).2` extra `ptrEq` checks. -/
theorem dropSharedPrefix_pushLeft_self {α : Type} :
    ∀ (t : IdTree α) (sa sb : List (IdTree α)),
      (dropSharedPrefix (pushLeft t sa).1 (pushLeft t sb).1).1 = (dropSharedPrefix sa sb).1 ∧
      (dropSharedPrefix (pushLeft t sa).1 (pushLeft t sb).1).2.1 = (dropSharedPrefix sa sb).2.1 ∧
      (dropSharedPrefix (pushLeft t sa).1 (pushLeft t sb).1).2.2 =
        (dropSharedPrefix sa sb).2.2 + (pushLeft t sa).2
  | .leaf, _, _ => by simp [pushLeft]
  | .node id h l k v r, sa, sb => by
    simp only [pushLeft]
    have ih := dropSharedPrefix_pushLeft_self l (.node id h l k v r :: sa) (.node id h l k v r :: sb)
    have hstep : dropSharedPrefix (.node id h l k v r :: sa) (.node id h l k v r :: sb) =
        ((dropSharedPrefix sa sb).1, (dropSharedPrefix sa sb).2.1, (dropSharedPrefix sa sb).2.2 + 1) := by
      simp [dropSharedPrefix, ptrEq_refl]
    rw [hstep] at ih
    dsimp only at ih
    exact ⟨ih.1, ih.2.1, by omega⟩

theorem pushLeft_node_head {α : Type} :
    ∀ (l : IdTree α) (id h k : Nat) (v : α) (r : IdTree α) (sa sb : List (IdTree α)),
      ∃ (id0 h0 : Nat) (l0 : IdTree α) (k0 : Nat) (v0 : α) (r0 : IdTree α) (as bs : List (IdTree α)),
        (pushLeft (.node id h l k v r) sa).1 = .node id0 h0 l0 k0 v0 r0 :: as ∧
        (pushLeft (.node id h l k v r) sb).1 = .node id0 h0 l0 k0 v0 r0 :: bs
  | .leaf, id, h, k, v, r, sa, sb =>
    ⟨id, h, .leaf, k, v, r, sa, sb, rfl, rfl⟩
  | .node id1 h1 l1 k1 v1 r1, id, h, k, v, r, sa, sb => by
    simp only [pushLeft]
    exact pushLeft_node_head l1 id1 h1 k1 v1 r1 (.node id h (.node id1 h1 l1 k1 v1 r1) k v r :: sa)
      (.node id h (.node id1 h1 l1 k1 v1 r1) k v r :: sb)

/-- Skipping ANY identical pushed subtree `t` (`Arc::ptr_eq` match) on leaf-free stacks
incurs `0` node visits, `0` extra `pushSteps`, and at most `idHeight t + 1` O(1) `ptrEq` checks,
independent of `idSize t`. -/
theorem cursorDiffLoop_pushLeft_self_cost {α : Type} [DecidableEq α] :
    ∀ (t : IdTree α) {sa sb : List (IdTree α)} (rem : Nat),
      NoLeaf sa → NoLeaf sb →
      (cursorDiffLoop (pushLeft t sa).1 (pushLeft t sb).1 rem).visited =
        (cursorDiffLoop sa sb rem).visited ∧
      (cursorDiffLoop (pushLeft t sa).1 (pushLeft t sb).1 rem).ptrChecks ≤
        (cursorDiffLoop sa sb rem).ptrChecks + idHeight t + 1 ∧
      (cursorDiffLoop (pushLeft t sa).1 (pushLeft t sb).1 rem).pushSteps =
        (cursorDiffLoop sa sb rem).pushSteps
  | .leaf, _, _, _, _, _ => ⟨rfl, by simp [pushLeft]; omega, rfl⟩
  | .node id h l k v r, sa, sb, rem, hsa, hsb => by
    obtain ⟨id0, h0, l0, k0, v0, r0, as, bs, hpa, hpb⟩ :=
      pushLeft_node_head l id h k v r sa sb
    have hdrop := dropSharedPrefix_pushLeft_self (.node id h l k v r) sa sb
    rw [hpa, hpb] at hdrop
    simp only [dropSharedPrefix, ptrEq_refl, ite_true] at hdrop
    have hstep : cursorDiffLoop (.node id0 h0 l0 k0 v0 r0 :: as)
                                (.node id0 h0 l0 k0 v0 r0 :: bs) rem =
        let p := dropSharedPrefix as bs
        let res := cursorDiffLoop p.1 p.2.1 rem
        { res with ptrChecks := res.ptrChecks + 1 + p.2.2 } := by
      rw [cursorDiffLoop.eq_def]; simp [ptrEq_refl]
    rw [hpa, hpb, hstep]
    simp only [hdrop.1, hdrop.2.1]
    have hd := cursorDiffLoop_dropSharedPrefix_cost_noLeaf sa sb rem hsa hsb
    have hpl := pushLeft_steps_le_height (.node id h l k v r) sa
    exact ⟨hd.1, by omega, hd.2.2⟩

/-!
## Balanced AVL insert (`avlInsert`) with fresh `NodeId`s and `O(log N)` / `O((log N)^2)` diff cost
-/

/-- Smart AVL node constructor allocating fresh `NodeId = nid`. -/
def mkNode {α : Type} (nid : Nat) (l : IdTree α) (k : Nat) (v : α) (r : IdTree α) : IdTree α :=
  .node nid (max (storedHeight l) (storedHeight r) + 1) l k v r

/-- `rotate_right` (`cow_map.rs:59-76`), allocating 2 fresh `NodeId`s while
preserving all untouched subtrees `T1, T2, T3` by their original `NodeId`. -/
def rotRight {α : Type} (nid : Nat) : IdTree α → IdTree α × Nat
  | .node _ _ (.node _ _ t1 kx vx t2) ky vy t3 =>
    let y' := mkNode nid t2 ky vy t3
    let x' := mkNode (nid + 1) t1 kx vx y'
    (x', nid + 2)
  | t => (t, nid)

/-- `rotate_left` (`cow_map.rs:78-95`), allocating 2 fresh `NodeId`s while
preserving all untouched subtrees `T1, T2, T3` by their original `NodeId`. -/
def rotLeft {α : Type} (nid : Nat) : IdTree α → IdTree α × Nat
  | .node _ _ t1 kx vx (.node _ _ t2 ky vy t3) =>
    let x' := mkNode nid t1 kx vx t2
    let y' := mkNode (nid + 1) x' ky vy t3
    (y', nid + 2)
  | t => (t, nid)

/-- `rebalance` (`cow_map.rs:97-117`) allocating fresh `NodeId`s for rotated nodes. -/
def rebalance {α : Type} (nid : Nat) (l : IdTree α) (k : Nat) (v : α) (r : IdTree α) : IdTree α × Nat :=
  let hl := storedHeight l
  let hr := storedHeight r
  if hl > hr + 1 then
    match l with
    | .node _ _ ll _ _ lr =>
      if storedHeight ll < storedHeight lr then
        let (l', nid1) := rotLeft nid l
        rotRight nid1 (mkNode nid1 l' k v r)
      else
        rotRight nid (mkNode nid l k v r)
    | .leaf => (mkNode nid l k v r, nid + 1)
  else if hr > hl + 1 then
    match r with
    | .node _ _ rl _ _ rr =>
      if storedHeight rr < storedHeight rl then
        let (r', nid1) := rotRight nid r
        rotLeft nid1 (mkNode nid1 l k v r')
      else
        rotLeft nid (mkNode nid l k v r)
    | .leaf => (mkNode nid l k v r, nid + 1)
  else
    (mkNode nid l k v r, nid + 1)

/-- `insert_node` (`cow_map.rs:119-145`): AVL-rebalanced persistent insert
allocating fresh `NodeId`s `≥ nid` for path-copied / rotated nodes while keeping
every untouched subtree's `NodeId` intact. -/
def avlInsert {α : Type} : IdTree α → Nat → α → Nat → IdTree α × Nat
  | .leaf, k, v, nid => (mkNode nid .leaf k v .leaf, nid + 1)
  | .node _ _ l k' v' r, k, v, nid =>
    if k < k' then
      let (l', nid1) := avlInsert l k v nid
      rebalance nid1 l' k' v' r
    else if k' < k then
      let (r', nid1) := avlInsert r k v nid
      rebalance nid1 l k' v' r'
    else
      (mkNode nid l k' v r, nid + 1)

/-- Every subtree of `s` is either an untouched subtree of `orig` or a freshly
allocated node with `id ≥ lo`. -/
def OwnOrGe {α : Type} (orig : IdTree α) (lo : Nat) (s : IdTree α) : Prop :=
  ∀ u, Subtree u s → Subtree u orig ∨ match u with | .leaf => True | .node id _ _ _ _ _ => lo ≤ id

theorem ownOrGe_refl {α : Type} (orig : IdTree α) (lo : Nat) : OwnOrGe orig lo orig :=
  fun _ hu => Or.inl hu

theorem ownOrGe_mono_lo {α : Type} {orig s : IdTree α} {lo₁ lo₂ : Nat} (hle : lo₁ ≤ lo₂)
    (h : OwnOrGe orig lo₂ s) : OwnOrGe orig lo₁ s := by
  intro u hu
  rcases h u hu with hsub | hge
  · exact Or.inl hsub
  · right
    cases u with
    | leaf => trivial
    | node _ _ _ _ _ _ => exact Nat.le_trans hle hge

theorem ownOrGe_mono_orig {α : Type} {o1 o2 s : IdTree α} {lo : Nat} (hsub : Subtree o1 o2)
    (h : OwnOrGe o1 lo s) : OwnOrGe o2 lo s := by
  intro u hu
  rcases h u hu with ho | hge
  · exact Or.inl (Subtree.trans ho hsub)
  · exact Or.inr hge

theorem ownOrGe_mkNode {α : Type} {orig l r : IdTree α} {lo nid k : Nat} {v : α}
    (hlo : lo ≤ nid) (hl : OwnOrGe orig lo l) (hr : OwnOrGe orig lo r) :
    OwnOrGe orig lo (mkNode nid l k v r) := by
  intro u hu
  rcases hu with rfl | (hul | hur)
  · exact Or.inr hlo
  · exact hl u hul
  · exact hr u hur

theorem validSharing_of_ownOrGe {α : Type} {orig t' : IdTree α} {nid : Nat}
    (hself : ValidSharing orig orig) (hlt : IdsLt nid orig) (hown : OwnOrGe orig nid t') :
    ValidSharing t' orig := by
  intro s t hs ht heq
  cases s with
  | leaf =>
    cases t with
    | leaf => rfl
    | node _ _ _ _ _ _ => simp [ptrEq] at heq
  | node id1 _ _ _ _ _ =>
    cases t with
    | leaf => simp [ptrEq] at heq
    | node id2 _ _ _ _ _ =>
      simp only [ptrEq, decide_eq_true_eq] at heq
      have hlt2 : id2 < nid := (IdsLt.subtree ht hlt).1
      rcases hown _ hs with hsub | hge
      · exact hself _ _ hsub ht (by simp [ptrEq, heq])
      · omega

theorem avl_storedHeight {α : Type} : ∀ {t : IdTree α}, AVL t → storedHeight t = idHeight t
  | .leaf, _ => rfl
  | .node _ _ _ _ _ _, ⟨hh, _⟩ => hh

theorem allKeys_mono {α : Type} {P Q : Nat → Prop} (hpq : ∀ x, P x → Q x) :
    ∀ {t : IdTree α}, AllKeys P t → AllKeys Q t
  | .leaf, _ => trivial
  | .node _ _ _ _ _ _, ⟨hk, hl, hr⟩ => ⟨hpq _ hk, allKeys_mono hpq hl, allKeys_mono hpq hr⟩

/-- Combined structural invariants and exact height-growth shape of `avlInsert`. -/
def AvlInsertResult {α : Type} (t : IdTree α) (k : Nat) (v : α) (nid : Nat) : Prop :=
  let (t', nid') := avlInsert t k v nid
  BST t' ∧
  AVL t' ∧
  idHeight t ≤ idHeight t' ∧
  idHeight t' ≤ idHeight t + 1 ∧
  IdsLt nid' t' ∧
  nid ≤ nid' ∧
  OwnOrGe t nid t' ∧
  (∀ P : Nat → Prop, AllKeys P t → P k → AllKeys P t') ∧
  (idHeight t' = idHeight t + 1 →
    (t = .leaf ∧ t' = mkNode nid .leaf k v .leaf) ∨
    (∃ id h l k' v' r,
      t = .node id h l k' v' r ∧
      idHeight l = idHeight r ∧
      ((k < k' ∧ idHeight (avlInsert l k v nid).1 = idHeight l + 1 ∧
          t' = mkNode (avlInsert l k v nid).2 (avlInsert l k v nid).1 k' v' r) ∨
       (k' < k ∧ idHeight (avlInsert r k v nid).1 = idHeight r + 1 ∧
          t' = mkNode (avlInsert r k v nid).2 l k' v' (avlInsert r k v nid).1))))

theorem mkNode_avl {α : Type} {nid k : Nat} {v : α} {l r : IdTree α}
    (hal : AVL l) (har : AVL r) (h1 : idHeight l ≤ idHeight r + 1) (h2 : idHeight r ≤ idHeight l + 1) :
    AVL (mkNode nid l k v r) := by
  unfold mkNode AVL
  rw [avl_storedHeight hal, avl_storedHeight har]
  exact ⟨rfl, h1, h2, hal, har⟩

theorem mkNode_height {α : Type} (nid : Nat) (l : IdTree α) (k : Nat) (v : α) (r : IdTree α) :
    idHeight (mkNode nid l k v r) = max (idHeight l) (idHeight r) + 1 := rfl

theorem mkNode_idsLt {α : Type} {nid bound k : Nat} {v : α} {l r : IdTree α}
    (hnb : nid < bound) (hl : IdsLt bound l) (hr : IdsLt bound r) :
    IdsLt bound (mkNode nid l k v r) := ⟨hnb, hl, hr⟩

theorem avlInsert_spec {α : Type} :
    ∀ (t : IdTree α) (k : Nat) (v : α) (nid : Nat),
      BST t → AVL t → IdsLt nid t → AvlInsertResult t k v nid
  | .leaf, k, v, nid, _, _, _ => by
    unfold AvlInsertResult avlInsert
    refine ⟨⟨trivial, trivial, trivial, trivial⟩,
      mkNode_avl trivial trivial (by simp [idHeight]) (by simp [idHeight]),
      by simp [mkNode, idHeight], by simp [mkNode, idHeight],
      mkNode_idsLt (by omega) trivial trivial, by omega,
      ownOrGe_mkNode (Nat.le_refl _) (ownOrGe_refl .leaf nid) (ownOrGe_refl .leaf nid),
      fun P _ hpk => ⟨hpk, trivial, trivial⟩,
      fun _ => Or.inl ⟨rfl, rfl⟩⟩
  | .node id h l k' v' r, k, v, nid, ⟨hlk, hkr, hbl, hbr⟩, ⟨hh, hlr, hrl, hal, har⟩, ⟨hid, hidl, hidr⟩ => by
    have hsub_l : Subtree l (.node id h l k' v' r) := Subtree.left id h k' v' l r
    have hsub_r : Subtree r (.node id h l k' v' r) := Subtree.right id h k' v' l r
    have hown_l0 : OwnOrGe (.node id h l k' v' r) nid l := ownOrGe_mono_orig hsub_l (ownOrGe_refl l nid)
    have hown_r0 : OwnOrGe (.node id h l k' v' r) nid r := ownOrGe_mono_orig hsub_r (ownOrGe_refl r nid)
    unfold AvlInsertResult
    rw [avlInsert.eq_def]
    dsimp only
    by_cases h1 : k < k'
    · simp only [h1, ite_true]
      have ihl := avlInsert_spec l k v nid hbl hal hidl
      unfold AvlInsertResult at ihl
      generalize hres : avlInsert l k v nid = res at ihl ⊢
      rcases res with ⟨l', nid1⟩
      rcases ihl with ⟨hbl', hal', hhl_lo, hhl_hi, hidl', hnid1, hownl', halll', hgrowl⟩
      have hidr1 : IdsLt nid1 r := hidr.mono hnid1
      have hownl1 : OwnOrGe (.node id h l k' v' r) nid l' := ownOrGe_mono_orig hsub_l hownl'
      have hl'k : AllKeys (· < k') l' := halll' (· < k') hlk h1
      have hshl' := avl_storedHeight hal'
      have hshr := avl_storedHeight har
      unfold rebalance
      simp only [hshl', hshr]
      by_cases himb : idHeight l' > idHeight r + 1
      · simp only [himb, ite_true]
        have hgrow : idHeight l' = idHeight l + 1 := by omega
        have hl_eq_r1 : idHeight l = idHeight r + 1 := by omega
        cases l' with
        | leaf => simp [idHeight] at himb
        | node idl hl' ll kl vl lr =>
          rcases hbl' with ⟨hll_k, hlr_k, hbll, hblr⟩
          rcases hal' with ⟨hhl'_eq, hll_lr, hlr_ll, hall, halr⟩
          rcases hl'k with ⟨hkl_k', hll_k', hlr_k'⟩
          rcases hidl' with ⟨_, hidll, hidlr⟩
          have hown_ll : OwnOrGe (.node id h l k' v' r) nid ll := fun u hu =>
            hownl1 u (Subtree.trans hu (Subtree.left idl hl' kl vl ll lr))
          have hown_lr : OwnOrGe (.node id h l k' v' r) nid lr := fun u hu =>
            hownl1 u (Subtree.trans hu (Subtree.right idl hl' kl vl ll lr))
          have hsh_ll := avl_storedHeight hall
          have hsh_lr := avl_storedHeight halr
          simp only [hsh_ll, hsh_lr]
          by_cases hlr_tall : idHeight ll < idHeight lr
          · simp only [hlr_tall, ite_true]
            cases lr with
            | leaf => simp [idHeight] at hlr_tall
            | node idlr hlr_n t2 ky vy t3 =>
              rcases hblr with ⟨ht2_y, ht3_y, hbt2, hbt3⟩
              rcases halr with ⟨_, ht2_t3, ht3_t2, hat2, hat3⟩
              rcases hlr_k with ⟨hkl_y, hkl_t2, hkl_t3⟩
              rcases hlr_k' with ⟨hy_k', ht2_k', ht3_k'⟩
              rcases hidlr with ⟨_, hidt2, hidt3⟩
              have hown_t2 : OwnOrGe (.node id h l k' v' r) nid t2 := fun u hu =>
                hown_lr u (Subtree.trans hu (Subtree.left idlr hlr_n ky vy t2 t3))
              have hown_t3 : OwnOrGe (.node id h l k' v' r) nid t3 := fun u hu =>
                hown_lr u (Subtree.trans hu (Subtree.right idlr hlr_n ky vy t2 t3))
              simp only [rotLeft, rotRight, mkNode]
              have hhr_eq : idHeight ll = idHeight r := by
                simp only [idHeight] at himb hhl_hi hll_lr hlr_ll hlr_tall; omega
              have ht2_le : idHeight t2 ≤ idHeight r := by
                simp only [idHeight] at himb hhl_hi hll_lr hlr_ll hlr_tall; omega
              have ht3_le : idHeight t3 ≤ idHeight r := by
                simp only [idHeight] at himb hhl_hi hll_lr hlr_ll hlr_tall; omega
              have hmax23 : max (idHeight t2) (idHeight t3) = idHeight r := by
                simp only [idHeight] at himb hhl_hi hll_lr hlr_ll hlr_tall; omega
              have havl_x : AVL (mkNode nid1 ll kl vl t2) :=
                mkNode_avl hall hat2 (by omega) (by omega)
              have havl_z : AVL (mkNode (nid1 + 2) t3 k' v' r) :=
                mkNode_avl hat3 har (by omega) (by omega)
              have havl_y : AVL (mkNode (nid1 + 3) (mkNode nid1 ll kl vl t2) ky vy (mkNode (nid1 + 2) t3 k' v' r)) :=
                mkNode_avl havl_x havl_z (by simp [mkNode, idHeight]; omega) (by simp [mkNode, idHeight]; omega)
              have hbst_x : BST (mkNode nid1 ll kl vl t2) := ⟨hll_k, hkl_t2, hbll, hbt2⟩
              have hbst_z : BST (mkNode (nid1 + 2) t3 k' v' r) := ⟨ht3_k', hkr, hbt3, hbr⟩
              have hbst_y : BST (mkNode (nid1 + 3) (mkNode nid1 ll kl vl t2) ky vy (mkNode (nid1 + 2) t3 k' v' r)) :=
                ⟨⟨hkl_y, allKeys_mono (fun _ hx => Nat.lt_trans hx hkl_y) hll_k, ht2_y⟩,
                 ⟨hy_k', ht3_y, allKeys_mono (fun _ hx => Nat.lt_trans hy_k' hx) hkr⟩,
                 hbst_x, hbst_z⟩
              refine ⟨hbst_y, havl_y,
                by simp only [idHeight]; omega,
                by simp only [idHeight]; omega,
                mkNode_idsLt (by omega)
                  (mkNode_idsLt (by omega) (hidll.mono (by omega)) (hidt2.mono (by omega)))
                  (mkNode_idsLt (by omega) (hidt3.mono (by omega)) (hidr1.mono (by omega))),
                by omega,
                ownOrGe_mkNode (by omega)
                  (ownOrGe_mkNode (by omega) hown_ll hown_t2)
                  (ownOrGe_mkNode (by omega) hown_t3 hown_r0),
                fun P ⟨hpk', hpl, hpr⟩ hpk => by
                  have hpl' := halll' P hpl hpk
                  rcases hpl' with ⟨hpkl, hpll, hpy, hpt2, hpt3⟩
                  exact ⟨hpy, ⟨hpkl, hpll, hpt2⟩, ⟨hpk', hpt3, hpr⟩⟩,
                fun hgt => by simp only [idHeight] at hgt; omega⟩
          · simp only [hlr_tall, ite_false, rotRight, mkNode]
            have hll_eq : idHeight ll = idHeight r + 1 := by
              simp only [idHeight] at himb hhl_hi; omega
            have hlr_eq : idHeight lr = idHeight r := by
              rcases hgrowl hgrow with ⟨hl_leaf, _⟩ | ⟨_, _, la, ka, va, ra, hleq, heq_lr, hlr_cases⟩
              · subst hl_leaf; simp [idHeight] at hl_eq_r1
              · rcases hlr_cases with ⟨_, _, hmk⟩ | ⟨_, _, hmk⟩
                · simp only [mkNode] at hmk
                  injection hmk with _ _ hll_eq' _ _ hlr_eq'
                  subst hlr_eq'
                  simp only [hleq, idHeight] at hl_eq_r1; omega
                · simp only [mkNode] at hmk
                  injection hmk with _ _ hll_eq' _ _ hlr_eq'
                  subst hll_eq'
                  simp only [hleq, idHeight] at hl_eq_r1; omega
            have havl_y : AVL (mkNode nid1 lr k' v' r) :=
              mkNode_avl halr har (by omega) (by omega)
            have havl_x : AVL (mkNode (nid1 + 1) ll kl vl (mkNode nid1 lr k' v' r)) :=
              mkNode_avl hall havl_y (by simp [mkNode, idHeight]; omega) (by simp [mkNode, idHeight]; omega)
            have hbst_y : BST (mkNode nid1 lr k' v' r) := ⟨hlr_k', hkr, hblr, hbr⟩
            have hbst_x : BST (mkNode (nid1 + 1) ll kl vl (mkNode nid1 lr k' v' r)) :=
              ⟨hll_k, ⟨hkl_k', hlr_k, allKeys_mono (fun _ hx => Nat.lt_trans hkl_k' hx) hkr⟩, hbll, hbst_y⟩
            refine ⟨hbst_x, havl_x,
              by simp only [idHeight]; omega,
              by simp only [idHeight]; omega,
              mkNode_idsLt (by omega)
                (hidll.mono (by omega))
                (mkNode_idsLt (by omega) (hidlr.mono (by omega)) (hidr1.mono (by omega))),
              by omega,
              ownOrGe_mkNode (by omega) hown_ll (ownOrGe_mkNode (by omega) hown_lr hown_r0),
              fun P ⟨hpk', hpl, hpr⟩ hpk => by
                have hpl' := halll' P hpl hpk
                rcases hpl' with ⟨hpkl, hpll, hplr⟩
                exact ⟨hpkl, hpll, ⟨hpk', hplr, hpr⟩⟩,
              fun hgt => by simp only [idHeight] at hgt; omega⟩
      · have hnotr : ¬ (idHeight r > idHeight l' + 1) := by omega
        simp only [himb, hnotr, ite_false]
        refine ⟨⟨hl'k, hkr, hbl', hbr⟩,
          mkNode_avl hal' har (by omega) (by omega),
          by simp only [mkNode, idHeight]; omega,
          by simp only [mkNode, idHeight]; omega,
          mkNode_idsLt (by omega) (hidl'.mono (by omega)) (hidr1.mono (by omega)),
          by omega,
          ownOrGe_mkNode hnid1 hownl1 hown_r0,
          fun P ⟨hpk', hpl, hpr⟩ hpk => ⟨hpk', halll' P hpl hpk, hpr⟩,
          fun hgt => Or.inr ⟨id, h, l, k', v', r, rfl,
            by simp only [mkNode, idHeight] at hgt; omega,
            Or.inl ⟨h1, by rw [hres]; dsimp only; simp only [mkNode, idHeight] at hgt; omega,
              by rw [hres]⟩⟩⟩
    · by_cases h2 : k' < k
      · simp only [h1, h2, ite_true, ite_false]
        have ihr := avlInsert_spec r k v nid hbr har hidr
        unfold AvlInsertResult at ihr
        generalize hres : avlInsert r k v nid = res at ihr ⊢
        rcases res with ⟨r', nid1⟩
        rcases ihr with ⟨hbr', har', hhr_lo, hhr_hi, hidr', hnid1, hownr', hallr', hgrowr⟩
        have hidl1 : IdsLt nid1 l := hidl.mono hnid1
        have hownr1 : OwnOrGe (.node id h l k' v' r) nid r' := ownOrGe_mono_orig hsub_r hownr'
        have hk'r' : AllKeys (k' < ·) r' := hallr' (k' < ·) hkr h2
        have hshl := avl_storedHeight hal
        have hshr' := avl_storedHeight har'
        unfold rebalance
        simp only [hshl, hshr']
        have hnotl : ¬ (idHeight l > idHeight r' + 1) := by omega
        simp only [hnotl, ite_false]
        by_cases himb : idHeight r' > idHeight l + 1
        · simp only [himb, ite_true]
          have hgrow : idHeight r' = idHeight r + 1 := by omega
          have hr_eq_l1 : idHeight r = idHeight l + 1 := by omega
          cases r' with
          | leaf => simp [idHeight] at himb
          | node idr hr' rl kr vr rr =>
            rcases hbr' with ⟨hrl_k, hrr_k, hbrl, hbrr⟩
            rcases har' with ⟨hhr'_eq, hrl_rr, hrr_rl, harl, harr⟩
            rcases hk'r' with ⟨hk'_kr, hk'_rl, hk'_rr⟩
            rcases hidr' with ⟨_, hidrl, hidrr⟩
            have hown_rl : OwnOrGe (.node id h l k' v' r) nid rl := fun u hu =>
              hownr1 u (Subtree.trans hu (Subtree.left idr hr' kr vr rl rr))
            have hown_rr : OwnOrGe (.node id h l k' v' r) nid rr := fun u hu =>
              hownr1 u (Subtree.trans hu (Subtree.right idr hr' kr vr rl rr))
            have hsh_rl := avl_storedHeight harl
            have hsh_rr := avl_storedHeight harr
            simp only [hsh_rl, hsh_rr]
            by_cases hrl_tall : idHeight rr < idHeight rl
            · simp only [hrl_tall, ite_true]
              cases rl with
              | leaf => simp [idHeight] at hrl_tall
              | node idrl hrl_n t2 ky vy t3 =>
                rcases hbrl with ⟨ht2_y, ht3_y, hbt2, hbt3⟩
                rcases harl with ⟨_, ht2_t3, ht3_t2, hat2, hat3⟩
                rcases hrl_k with ⟨hy_kr, ht2_kr, ht3_kr⟩
                rcases hk'_rl with ⟨hk'_y, hk'_t2, hk'_t3⟩
                rcases hidrl with ⟨_, hidt2, hidt3⟩
                have hown_t2 : OwnOrGe (.node id h l k' v' r) nid t2 := fun u hu =>
                  hown_rl u (Subtree.trans hu (Subtree.left idrl hrl_n ky vy t2 t3))
                have hown_t3 : OwnOrGe (.node id h l k' v' r) nid t3 := fun u hu =>
                  hown_rl u (Subtree.trans hu (Subtree.right idrl hrl_n ky vy t2 t3))
                simp only [rotRight, rotLeft, mkNode]
                have hrr_eq : idHeight rr = idHeight l := by
                  simp only [idHeight] at himb hhr_hi hrl_rr hrr_rl hrl_tall; omega
                have ht2_le : idHeight t2 ≤ idHeight l := by
                  simp only [idHeight] at himb hhr_hi hrl_rr hrr_rl hrl_tall; omega
                have ht3_le : idHeight t3 ≤ idHeight l := by
                  simp only [idHeight] at himb hhr_hi hrl_rr hrr_rl hrl_tall; omega
                have hmax23 : max (idHeight t2) (idHeight t3) = idHeight l := by
                  simp only [idHeight] at himb hhr_hi hrl_rr hrr_rl hrl_tall; omega
                have havl_x : AVL (mkNode (nid1 + 2) l k' v' t2) :=
                  mkNode_avl hal hat2 (by omega) (by omega)
                have havl_z : AVL (mkNode nid1 t3 kr vr rr) :=
                  mkNode_avl hat3 harr (by omega) (by omega)
                have havl_y : AVL (mkNode (nid1 + 3) (mkNode (nid1 + 2) l k' v' t2) ky vy (mkNode nid1 t3 kr vr rr)) :=
                  mkNode_avl havl_x havl_z (by simp [mkNode, idHeight]; omega) (by simp [mkNode, idHeight]; omega)
                have hbst_x : BST (mkNode (nid1 + 2) l k' v' t2) := ⟨hlk, hk'_t2, hbl, hbt2⟩
                have hbst_z : BST (mkNode nid1 t3 kr vr rr) := ⟨ht3_kr, hrr_k, hbt3, hbrr⟩
                have hbst_y : BST (mkNode (nid1 + 3) (mkNode (nid1 + 2) l k' v' t2) ky vy (mkNode nid1 t3 kr vr rr)) :=
                  ⟨⟨hk'_y, allKeys_mono (fun _ hx => Nat.lt_trans hx hk'_y) hlk, ht2_y⟩,
                   ⟨hy_kr, ht3_y, allKeys_mono (fun _ hx => Nat.lt_trans hy_kr hx) hrr_k⟩,
                   hbst_x, hbst_z⟩
                refine ⟨hbst_y, havl_y,
                  by simp only [idHeight]; omega,
                  by simp only [idHeight]; omega,
                  mkNode_idsLt (by omega)
                    (mkNode_idsLt (by omega) (hidl1.mono (by omega)) (hidt2.mono (by omega)))
                    (mkNode_idsLt (by omega) (hidt3.mono (by omega)) (hidrr.mono (by omega))),
                  by omega,
                  ownOrGe_mkNode (by omega)
                    (ownOrGe_mkNode (by omega) hown_l0 hown_t2)
                    (ownOrGe_mkNode (by omega) hown_t3 hown_rr),
                  fun P ⟨hpk', hpl, hpr⟩ hpk => by
                    have hpr' := hallr' P hpr hpk
                    rcases hpr' with ⟨hpkr, ⟨hpy, hpt2, hpt3⟩, hprr⟩
                    exact ⟨hpy, ⟨hpk', hpl, hpt2⟩, ⟨hpkr, hpt3, hprr⟩⟩,
                  fun hgt => by simp only [idHeight] at hgt; omega⟩
            · simp only [hrl_tall, ite_false, rotLeft, mkNode]
              have hrr_eq : idHeight rr = idHeight l + 1 := by
                simp only [idHeight] at himb hhr_hi; omega
              have hrl_eq : idHeight rl = idHeight l := by
                rcases hgrowr hgrow with ⟨hr_leaf, _⟩ | ⟨_, _, la, ka, va, ra, hreq, heq_lr, hlr_cases⟩
                · subst hr_leaf; simp [idHeight] at hr_eq_l1
                · rcases hlr_cases with ⟨_, _, hmk⟩ | ⟨_, _, hmk⟩
                  · simp only [mkNode] at hmk
                    injection hmk with _ _ hrl_eq' _ _ hrr_eq'
                    subst hrr_eq'
                    simp only [hreq, idHeight] at hr_eq_l1; omega
                  · simp only [mkNode] at hmk
                    injection hmk with _ _ hrl_eq' _ _ hrr_eq'
                    subst hrl_eq'
                    simp only [hreq, idHeight] at hr_eq_l1; omega
              have havl_x : AVL (mkNode nid1 l k' v' rl) :=
                mkNode_avl hal harl (by omega) (by omega)
              have havl_y : AVL (mkNode (nid1 + 1) (mkNode nid1 l k' v' rl) kr vr rr) :=
                mkNode_avl havl_x harr (by simp [mkNode, idHeight]; omega) (by simp [mkNode, idHeight]; omega)
              have hbst_x : BST (mkNode nid1 l k' v' rl) := ⟨hlk, hk'_rl, hbl, hbrl⟩
              have hbst_y : BST (mkNode (nid1 + 1) (mkNode nid1 l k' v' rl) kr vr rr) :=
                ⟨⟨hk'_kr, allKeys_mono (fun _ hx => Nat.lt_trans hx hk'_kr) hlk, hrl_k⟩,
                 hrr_k, hbst_x, hbrr⟩
              refine ⟨hbst_y, havl_y,
                by simp only [idHeight]; omega,
                by simp only [idHeight]; omega,
                mkNode_idsLt (by omega)
                  (mkNode_idsLt (by omega) (hidl1.mono (by omega)) (hidrl.mono (by omega)))
                  (hidrr.mono (by omega)),
                by omega,
                ownOrGe_mkNode (by omega) (ownOrGe_mkNode (by omega) hown_l0 hown_rl) hown_rr,
                fun P ⟨hpk', hpl, hpr⟩ hpk => by
                  have hpr' := hallr' P hpr hpk
                  rcases hpr' with ⟨hpkr, hprl, hprr⟩
                  exact ⟨hpkr, ⟨hpk', hpl, hprl⟩, hprr⟩,
                fun hgt => by simp only [idHeight] at hgt; omega⟩
        · simp only [himb, ite_false]
          refine ⟨⟨hlk, hk'r', hbl, hbr'⟩,
            mkNode_avl hal har' (by omega) (by omega),
            by simp only [mkNode, idHeight]; omega,
            by simp only [mkNode, idHeight]; omega,
            mkNode_idsLt (by omega) (hidl1.mono (by omega)) (hidr'.mono (by omega)),
            by omega,
            ownOrGe_mkNode hnid1 hown_l0 hownr1,
            fun P ⟨hpk', hpl, hpr⟩ hpk => ⟨hpk', hpl, hallr' P hpr hpk⟩,
            fun hgt => Or.inr ⟨id, h, l, k', v', r, rfl,
              by simp only [mkNode, idHeight] at hgt; omega,
              Or.inr ⟨h2, by rw [hres]; dsimp only; simp only [mkNode, idHeight] at hgt; omega,
                by rw [hres]⟩⟩⟩
      · have hk_eq : k = k' := by omega
        subst hk_eq
        simp only [h1, ite_false]
        refine ⟨⟨hlk, hkr, hbl, hbr⟩,
          mkNode_avl hal har hlr hrl,
          by simp only [mkNode, idHeight]; omega,
          by simp only [mkNode, idHeight]; omega,
          mkNode_idsLt (by omega) (hidl.mono (by omega)) (hidr.mono (by omega)),
          by omega,
          ownOrGe_mkNode (Nat.le_refl _) hown_l0 hown_r0,
          fun P ⟨hpk', hpl, hpr⟩ _ => ⟨hpk', hpl, hpr⟩,
          fun hgt => by simp only [mkNode, idHeight] at hgt; omega⟩

/-- Balanced `avlInsert` preserves `BST`, `AVL`, and `ValidSharing` with the pre-insert tree. -/
theorem avlInsert_preserves_invariants {α : Type}
    {t : IdTree α} (hb : BST t) (ha : AVL t) (hself : ValidSharing t t)
    {nid : Nat} (hlt : IdsLt nid t) (k : Nat) (v : α) :
    let t' := (avlInsert t k v nid).1
    BST t' ∧ AVL t' ∧ ValidSharing t' t := by
  have h := avlInsert_spec t k v nid hb ha hlt
  unfold AvlInsertResult at h
  rcases h with ⟨hb', ha', _, _, _, _, hown, _, _⟩
  exact ⟨hb', ha', validSharing_of_ownOrGe hself hlt hown⟩

/-- Upper bound predicate on the top key of a cursor stack. -/
def StackBounded {α : Type} : Option Nat → List (IdTree α) → Prop
  | none, [] => True
  | none, _ :: _ => False
  | some _, [] => True
  | some _, .leaf :: _ => False
  | some m, .node _ _ _ k _ _ :: _ => m ≤ k

/-- Every node on stack `s` has `id < bound`. -/
def StackIdsLt {α : Type} (bound : Nat) (s : List (IdTree α)) : Prop :=
  ∀ x ∈ s, IdsLt bound x

theorem rightUb {α : Type} {id h k' k : Nat} {l r : IdTree α} {v' : α} :
    ∀ {ub : Option Nat},
      (match ub with | none => True | some m => AllKeys (· < m) (.node id h l k' v' r) ∧ k < m) →
      (match ub with | none => True | some m => AllKeys (· < m) r ∧ k < m)
  | none, _ => trivial
  | some _, ⟨⟨_, _, hr⟩, hk⟩ => ⟨hr, hk⟩

theorem ptrEq_fresh_false {α : Type} {id1 h1 k1 id2 h2 k2 nid : Nat}
    {l1 r1 l2 r2 : IdTree α} {v1 v2 : α}
    (hge : nid ≤ id1) (hlt : id2 < nid) :
    ptrEq (.node id1 h1 l1 k1 v1 r1) (.node id2 h2 l2 k2 v2 r2) = false := by
  simp only [ptrEq, decide_eq_false_iff_not]; omega

theorem cursorDiffLoop_same_kv_shared_r {α : Type} [DecidableEq α]
    {id1 h1 id2 h2 k : Nat} {l1 l2 r : IdTree α} {v : α}
    {sa sb : List (IdTree α)} (rem : Nat)
    (hne : ptrEq (.node id1 h1 l1 k v r) (.node id2 h2 l2 k v r) = false) :
    cursorDiffLoop (.node id1 h1 l1 k v r :: sa) (.node id2 h2 l2 k v r :: sb) rem =
      let res := cursorDiffLoop sa sb rem
      { res with visited := res.visited + 1, ptrChecks := res.ptrChecks + 2 } := by
  rw [cursorDiffLoop.eq_def]
  simp [hne, Nat.lt_irrefl, ptrEq_refl]

theorem cursorDiffLoop_same_kv_diff_r {α : Type} [DecidableEq α]
    {id1 h1 id2 h2 k : Nat} {l1 l2 r1 r2 : IdTree α} {v : α}
    {sa sb : List (IdTree α)} (rem : Nat)
    (hne : ptrEq (.node id1 h1 l1 k v r1) (.node id2 h2 l2 k v r2) = false)
    (hrne : ptrEq r1 r2 = false) :
    cursorDiffLoop (.node id1 h1 l1 k v r1 :: sa) (.node id2 h2 l2 k v r2 :: sb) rem =
      let pl1 := pushLeft r1 sa
      let pl2 := pushLeft r2 sb
      let res := cursorDiffLoop pl1.1 pl2.1 rem
      { val := res.val
        visited := res.visited + 1
        ptrChecks := res.ptrChecks + 2
        pushSteps := res.pushSteps + pl1.2 + pl2.2 } := by
  rw [cursorDiffLoop.eq_def]
  simp [hne, hrne, Nat.lt_irrefl]

theorem rotRight_nid_ge {α : Type} (nid : Nat) (t : IdTree α) :
    nid ≤ (rotRight nid t).2 := by
  cases t with
  | leaf => simp [rotRight]
  | node _ _ l _ _ _ =>
    cases l with
    | leaf => simp [rotRight]
    | node _ _ _ _ _ _ => simp [rotRight]

theorem rotLeft_nid_ge {α : Type} (nid : Nat) (t : IdTree α) :
    nid ≤ (rotLeft nid t).2 := by
  cases t with
  | leaf => simp [rotLeft]
  | node _ _ _ _ _ r =>
    cases r with
    | leaf => simp [rotLeft]
    | node _ _ _ _ _ _ => simp [rotLeft]

theorem rebalance_root_fresh {α : Type} (nid : Nat) (l : IdTree α) (k : Nat) (v : α) (r : IdTree α) :
    nid ≤ (rebalance nid l k v r).2 ∧
    ∃ id' h' l' k' v' r',
      (rebalance nid l k v r).1 = .node id' h' l' k' v' r' ∧ nid ≤ id' := by
  dsimp only [rebalance]
  by_cases h1 : storedHeight l > storedHeight r + 1
  · simp only [h1, ite_true]
    cases l with
    | leaf => simp only [mkNode]; exact ⟨by omega, nid, _, _, _, _, _, rfl, Nat.le_refl _⟩
    | node _ _ ll _ _ lr =>
      by_cases hll : storedHeight ll < storedHeight lr
      · simp only [hll, ite_true]
        cases lr with
        | leaf =>
          simp only [rotLeft, rotRight, mkNode]
          exact ⟨by omega, nid + 1, _, _, _, _, _, rfl, by omega⟩
        | node _ _ _ _ _ _ =>
          simp only [rotLeft, rotRight, mkNode]
          exact ⟨by omega, nid + 3, _, _, _, _, _, rfl, by omega⟩
      · simp only [hll, ite_false, rotRight, mkNode]
        exact ⟨by omega, nid + 1, _, _, _, _, _, rfl, by omega⟩
  · simp only [h1, ite_false]
    by_cases h2 : storedHeight r > storedHeight l + 1
    · simp only [h2, ite_true]
      cases r with
      | leaf => simp only [mkNode]; exact ⟨by omega, nid, _, _, _, _, _, rfl, Nat.le_refl _⟩
      | node _ _ rl _ _ rr =>
        by_cases hrr : storedHeight rr < storedHeight rl
        · simp only [hrr, ite_true]
          cases rl with
          | leaf =>
            simp only [rotRight, rotLeft, mkNode]
            exact ⟨by omega, nid + 1, _, _, _, _, _, rfl, by omega⟩
          | node _ _ _ _ _ _ =>
            simp only [rotRight, rotLeft, mkNode]
            exact ⟨by omega, nid + 3, _, _, _, _, _, rfl, by omega⟩
        · simp only [hrr, ite_false, rotLeft, mkNode]
          exact ⟨by omega, nid + 1, _, _, _, _, _, rfl, by omega⟩
    · simp only [h2, ite_false]
      exact ⟨by omega, nid, _, _, _, _, _, rfl, Nat.le_refl _⟩

theorem avlInsert_root_fresh {α : Type} :
    ∀ (t : IdTree α) (k : Nat) (v : α) (nid : Nat),
      nid ≤ (avlInsert t k v nid).2 ∧
      ∃ id' h' l' k' v' r',
        (avlInsert t k v nid).1 = .node id' h' l' k' v' r' ∧ nid ≤ id'
  | .leaf, k, v, nid => ⟨by simp [avlInsert], nid, _, .leaf, k, v, .leaf, rfl, Nat.le_refl _⟩
  | .node id h l k' v' r, k, v, nid => by
    rw [avlInsert.eq_def]; dsimp only
    by_cases h1 : k < k'
    · simp only [h1, ite_true]
      have ⟨hn1, _⟩ := avlInsert_root_fresh l k v nid
      have ⟨hn2, id', h', l', k'', v'', r', heq, hid'⟩ :=
        rebalance_root_fresh (avlInsert l k v nid).2 (avlInsert l k v nid).1 k' v' r
      exact ⟨Nat.le_trans hn1 hn2, id', h', l', k'', v'', r', heq, Nat.le_trans hn1 hid'⟩
    · by_cases h2 : k' < k
      · simp only [h1, h2, ite_true, ite_false]
        have ⟨hn1, _⟩ := avlInsert_root_fresh r k v nid
        have ⟨hn2, id', h', l', k'', v'', r', heq, hid'⟩ :=
          rebalance_root_fresh (avlInsert r k v nid).2 l k' v' (avlInsert r k v nid).1
        exact ⟨Nat.le_trans hn1 hn2, id', h', l', k'', v'', r', heq, Nat.le_trans hn1 hid'⟩
      · simp only [h1, h2, ite_false]
        exact ⟨by omega, nid, _, l, k', v, r, rfl, Nat.le_refl _⟩

theorem ptrEq_fresh_lt_false {α : Type} {a b : IdTree α} {nid : Nat}
    (ha : ∃ id h l k v r, a = .node id h l k v r ∧ nid ≤ id)
    (hb : IdsLt nid b) :
    ptrEq a b = false := by
  rcases ha with ⟨id1, h1, l1, k1, v1, r1, rfl, hge⟩
  cases b with
  | leaf => rfl
  | node id2 _ _ _ _ _ => exact ptrEq_fresh_false hge hb.1

theorem noLeaf_cons {α : Type} {id h k : Nat} {v : α} {l r : IdTree α} {s : List (IdTree α)}
    (hs : NoLeaf s) : NoLeaf (.node id h l k v r :: s) := by
  intro x hx
  rcases List.mem_cons.mp hx with rfl | hx'
  · intro hc; cases hc
  · exact hs x hx'

theorem stackIdsLt_cons {α : Type} {nid : Nat} {t : IdTree α} {s : List (IdTree α)}
    (ht : IdsLt nid t) (hs : StackIdsLt nid s) : StackIdsLt nid (t :: s) := by
  intro x hx
  rcases List.mem_cons.mp hx with rfl | hx'
  · exact ht
  · exact hs x hx'

theorem ptrEq_strict_subtree_false {α : Type} {a b t : IdTree α}
    (hself : ValidSharing t t) (ha : Subtree a t) (hb : Subtree b t)
    (hlt : idSize a < idSize b) : ptrEq a b = false := by
  cases h : ptrEq a b with
  | false => rfl
  | true =>
    have heq := hself a b ha hb h
    subst heq; omega

theorem validSharing_self_left {α : Type} {id h k : Nat} {v : α} {l r : IdTree α}
    (hs : ValidSharing (.node id h l k v r) (.node id h l k v r)) : ValidSharing l l :=
  fun s1 s2 h1 h2 heq =>
    hs s1 s2 (Subtree.trans h1 (Subtree.left id h k v l r))
      (Subtree.trans h2 (Subtree.left id h k v l r)) heq

theorem validSharing_self_right {α : Type} {id h k : Nat} {v : α} {l r : IdTree α}
    (hs : ValidSharing (.node id h l k v r) (.node id h l k v r)) : ValidSharing r r :=
  fun s1 s2 h1 h2 heq =>
    hs s1 s2 (Subtree.trans h1 (Subtree.right id h k v l r))
      (Subtree.trans h2 (Subtree.right id h k v l r)) heq

theorem quad_step1 {a b : Nat} (h : a + 1 ≤ b) :
    (a + 3) * (a + 4) + (2 * b + 6) ≤ (b + 3) * (b + 4) := by
  have hm : (a + 3) * (a + 4) ≤ (b + 2) * (b + 3) :=
    Nat.mul_le_mul (by omega) (by omega)
  have heq : (b + 3) * (b + 4) = (b + 2) * (b + 3) + (2 * b + 6) := by
    simp only [Nat.add_mul, Nat.mul_add]; omega
  omega

theorem quad_step2 {a b : Nat} (h : a + 2 ≤ b) :
    (a + 3) * (a + 4) + (4 * b + 10) ≤ (b + 3) * (b + 4) := by
  have hm : (a + 3) * (a + 4) ≤ (b + 1) * (b + 2) :=
    Nat.mul_le_mul (by omega) (by omega)
  have heq : (b + 3) * (b + 4) = (b + 1) * (b + 2) + (4 * b + 10) := by
    simp only [Nat.add_mul, Nat.mul_add]; omega
  omega

theorem quad_step3 {a b : Nat} (h : a + 3 ≤ b) :
    (a + 3) * (a + 4) + (6 * b + 12) ≤ (b + 3) * (b + 4) := by
  have hm : (a + 3) * (a + 4) ≤ b * (b + 1) :=
    Nat.mul_le_mul (by omega) (by omega)
  have heq : (b + 3) * (b + 4) = b * (b + 1) + (6 * b + 12) := by
    simp only [Nat.add_mul, Nat.mul_add]; omega
  omega

theorem quad_base {a : Nat} : a + 3 ≤ (a + 3) * (a + 4) := by
  have : (a + 3) * 1 ≤ (a + 3) * (a + 4) := Nat.mul_le_mul_left _ (by omega)
  omega

set_option maxHeartbeats 800000 in
set_option maxRecDepth 4000 in
/-- Explicit step cost of `cursorDiffLoop` after a balanced `avlInsert` (covering all
4 AVL rotation shapes LL, LR, RR, RL, and charging every `ptrEq` check and stack push). -/
theorem cursorDiffLoop_avlInsert_cost {α : Type} [DecidableEq α] :
    ∀ (t : IdTree α) (k : Nat) (v : α) (nid : Nat)
      (sa sb : List (IdTree α)) (ub : Option Nat) (rem : Nat),
      BST t → AVL t → ValidSharing t t → IdsLt nid t →
      (match ub with | none => True | some m => AllKeys (· < m) t ∧ k < m) →
      StackBounded ub sa → StackBounded ub sb →
      NoLeaf sa → NoLeaf sb → StackIdsLt nid sb →
      let t' := (avlInsert t k v nid).1
      (cursorDiffLoop (pushLeft t' sa).1 (pushLeft t sb).1 rem).visited ≤
        (cursorDiffLoop sa sb rem).visited + idHeight t + 3 ∧
      (cursorDiffLoop (pushLeft t' sa).1 (pushLeft t sb).1 rem).ptrChecks ≤
        (cursorDiffLoop sa sb rem).ptrChecks + (idHeight t + 3) * (idHeight t + 4) ∧
      (cursorDiffLoop (pushLeft t' sa).1 (pushLeft t sb).1 rem).pushSteps ≤
        (cursorDiffLoop sa sb rem).pushSteps + 2 * ((idHeight t + 3) * (idHeight t + 4))
  | .leaf, k, v, nid, sa, sb, ub, rem, _, _, _, _, hub, _, hsb, _, _, hidsb => by
    simp only [avlInsert, mkNode, idHeight, pushLeft]
    cases sb with
    | nil =>
      rw [cursorDiffLoop.eq_def]; dsimp only
      cases rem with
      | zero => simp
      | succ r =>
        simp only [pushLeft]
        have ⟨m1, m2, m3⟩ := cursorDiffLoop_cost_mono sa [] (Nat.le_succ r)
        change _ ≤ (cursorDiffLoop sa [] r.succ).visited + 0 + 3 ∧
               _ ≤ (cursorDiffLoop sa [] r.succ).ptrChecks + (0 + 3) * (0 + 4) ∧
               _ ≤ (cursorDiffLoop sa [] r.succ).pushSteps + 2 * ((0 + 3) * (0 + 4))
        exact ⟨by omega, by omega, by omega⟩
    | cons b bs =>
      cases ub with
      | none => cases hsb
      | some m =>
        cases b with
        | leaf => cases hsb
        | node idb hb lb kb vb rb =>
          have hlt : k < kb := Nat.lt_of_lt_of_le hub.2 hsb
          have hidb : idb < nid := (hidsb _ (List.mem_cons_self ..)).1
          have hne := ptrEq_fresh_false (l1 := .leaf) (r1 := .leaf) (l2 := lb) (r2 := rb)
            (h1 := max (storedHeight (.leaf : IdTree α)) (storedHeight (.leaf : IdTree α)) + 1)
            (h2 := hb) (k1 := k) (k2 := kb) (v1 := v) (v2 := vb) (Nat.le_refl nid) hidb
          rw [cursorDiffLoop.eq_def]
          simp only [hne, Bool.false_eq_true, ite_false, hlt, ite_true]
          cases rem with
          | zero => simp
          | succ r =>
            simp only [pushLeft]
            have ⟨m1, m2, m3⟩ := cursorDiffLoop_cost_mono sa (.node idb hb lb kb vb rb :: bs) (Nat.le_succ r)
            change _ ≤ (cursorDiffLoop sa (.node idb hb lb kb vb rb :: bs) r.succ).visited + 0 + 3 ∧
                   _ ≤ (cursorDiffLoop sa (.node idb hb lb kb vb rb :: bs) r.succ).ptrChecks + (0 + 3) * (0 + 4) ∧
                   _ ≤ (cursorDiffLoop sa (.node idb hb lb kb vb rb :: bs) r.succ).pushSteps + 2 * ((0 + 3) * (0 + 4))
            exact ⟨by omega, by omega, by omega⟩
  | .node id h l k' v' r, k, v, nid, sa, sb, ub, rem,
      ⟨hlk, hkr, hbl, hbr⟩, ⟨hh, hlr, hrl, hal, har⟩, hself, ⟨hid, hidl, hidr⟩,
      hub, hsa, hsb, hnsa, hnsb, hidsb => by
    have hself_l := validSharing_self_left hself
    have hself_r := validSharing_self_right hself
    dsimp only
    rw [avlInsert.eq_def]; dsimp only
    by_cases hk1 : k < k'
    · simp only [hk1, ite_true]
      have ispec := avlInsert_spec l k v nid hbl hal hidl
      unfold AvlInsertResult at ispec
      generalize hres : avlInsert l k v nid = res at ispec ⊢
      rcases res with ⟨l', nid1⟩
      rcases ispec with ⟨hbl', hal', hhl_lo, hhl_hi, hidl', hnid1, _, _, hgrowl⟩
      have hshl' := avl_storedHeight hal'
      have hshr := avl_storedHeight har
      unfold rebalance
      simp only [hshl', hshr]
      by_cases himb : idHeight l' > idHeight r + 1
      · simp only [himb, ite_true]
        have hgrow : idHeight l' = idHeight l + 1 := by omega
        have hl_eq_r1 : idHeight l = idHeight r + 1 := by omega
        rcases hgrowl hgrow with ⟨hl_leaf, _⟩ | ⟨idl0, hl0, l1, kx, vx, l2, hleq, heq_l12, hl_cases⟩
        · subst hl_leaf; simp [idHeight] at hl_eq_r1
        · subst hleq
          have hself_l1 := validSharing_self_left hself_l
          have hself_l2 := validSharing_self_right hself_l
          rcases hbl with ⟨hl1_x, hl2_x, hbl1, hbl2⟩
          rcases hal with ⟨_, _, _, hal1, hal2⟩
          rcases hlk with ⟨hkx_k', hl1_k', hl2_k'⟩
          rcases hidl with ⟨hidl0, hidl1, hidl2⟩
          rcases hl_cases with ⟨hk_lt_x, hgl1, hl'_eq⟩ | ⟨hx_lt_k, hgl2, hl'_eq⟩
          · -- LL rotation
            have hsh_l1' : storedHeight (avlInsert l1 k v nid).1 = idHeight (avlInsert l1 k v nid).1 := by
              rw [hl'_eq] at hal'; exact avl_storedHeight hal'.2.2.2.1
            have hsh_l2 := avl_storedHeight hal2
            have hnot_lr : ¬ (idHeight (avlInsert l1 k v nid).1 < idHeight l2) := by omega
            simp only [hl'_eq, mkNode, hsh_l1', hsh_l2, hnot_lr, ite_false, rotRight, pushLeft]
            let l1' := (avlInsert l1 k v nid).1
            let x' := mkNode (nid1 + 1) l1' kx vx (mkNode nid1 l2 k' v' r)
            let x0 := IdTree.node idl0 hl0 l1 kx vx l2
            let y0 := IdTree.node id h x0 k' v' r
            have hih := cursorDiffLoop_avlInsert_cost l1 k v nid (x' :: sa) (x0 :: y0 :: sb)
              (some kx) rem hbl1 hal1 hself_l1 hidl1 ⟨hl1_x, hk_lt_x⟩ (Nat.le_refl kx) (Nat.le_refl kx)
              (noLeaf_cons hnsa) (noLeaf_cons (noLeaf_cons hnsb))
              (stackIdsLt_cons ⟨hidl0, hidl1, hidl2⟩ (stackIdsLt_cons ⟨hid, ⟨hidl0, hidl1, hidl2⟩, hidr⟩ hidsb))
            have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hidl0
            have hyne : ptrEq (mkNode nid1 l2 k' v' r) y0 = false := ptrEq_fresh_false (by omega) hid
            have hrne : ptrEq (mkNode nid1 l2 k' v' r) l2 = false :=
              ptrEq_fresh_lt_false ⟨nid1, _, l2, k', v', r, rfl, Nat.le_refl nid1⟩ (hidl2.mono hnid1)
            have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := y0 :: sb) rem hxne hrne
            have hskip := cursorDiffLoop_pushLeft_self_cost l2 (sa := mkNode nid1 l2 k' v' r :: sa)
              (sb := y0 :: sb) rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
            have hstep2 := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hyne
            have hpl_l2 := pushLeft_steps_le_height l2 (mkNode nid1 l2 k' v' r :: sa)
            have hpl_l2b := pushLeft_steps_le_height l2 (y0 :: sb)
            dsimp only [l1', x', x0, y0, mkNode] at hih hstep1 hskip hstep2 hpl_l2 hpl_l2b ⊢
            rw [hstep1] at hih
            simp only [hsh_l1', hsh_l2, pushLeft] at hih hskip hstep2 hpl_l2 hpl_l2b ⊢
            rw [hstep2] at hskip
            dsimp only at hih hskip
            have hht : idHeight y0 = max (max (idHeight l1) (idHeight l2) + 1) (idHeight r) + 1 := rfl
            have hq := quad_step2 (a := idHeight l1) (b := idHeight y0) (by omega)
            dsimp only [y0, x0] at hht hq
            exact ⟨by omega, by omega, by omega⟩
          · -- LR rotation
            have il2 := avlInsert_spec l2 k v nid hbl2 hal2 hidl2
            unfold AvlInsertResult at il2
            generalize hres2 : avlInsert l2 k v nid = res2 at il2 hl'_eq hgl2 ⊢
            rcases res2 with ⟨l2', nid2⟩
            dsimp only at hgl2 hl'_eq
            rcases il2 with ⟨hbl2', hal2', _, _, _, hnid2, _, _, hgrowl2⟩
            have hsh_l1 := avl_storedHeight hal1
            have hsh_l2' := avl_storedHeight hal2'
            have hlr_tall : idHeight l1 < idHeight l2' := by omega
            let x0 := IdTree.node idl0 hl0 l1 kx vx l2
            let y0 := IdTree.node id h x0 k' v' r
            rcases hgrowl2 hgl2 with ⟨hl2_leaf, hl2'_eq⟩ | ⟨idy0, hy0, t2, ky, vy, t3, hl2_eq, heq_t23, ht_cases⟩
            · -- Base LR (`l2 = .leaf`)
              subst hl2_leaf
              have hl1_leaf : l1 = .leaf := by
                cases l1 with
                | leaf => rfl
                | node _ _ _ _ _ _ => simp [idHeight] at heq_l12
              have hr_leaf : r = .leaf := by
                cases r with
                | leaf => rfl
                | node _ _ _ _ _ _ => simp [idHeight] at hl_eq_r1 heq_l12; omega
              subst hl1_leaf; subst hr_leaf
              simp only [hl'_eq, mkNode, hsh_l1, hsh_l2', hlr_tall, ite_true]
              simp only [hl2'_eq, mkNode, rotLeft, rotRight, pushLeft]
              let x' := mkNode nid1 .leaf kx vx .leaf
              let z' := mkNode (nid1 + 2) .leaf k' v' .leaf
              let y' := mkNode (nid1 + 2 + 1) x' k v z'
              have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hidl0
              have hstep1 := cursorDiffLoop_same_kv_shared_r (sa := y' :: sa) (sb := y0 :: sb) rem hxne
              have hyne : ptrEq y' y0 = false := ptrEq_fresh_false (by omega) hid
              have hstep2 : cursorDiffLoop (y' :: sa) (y0 :: sb) rem =
                  match rem with
                  | 0 => { val := none, visited := 1, ptrChecks := 1, pushSteps := 0 }
                  | rem' + 1 =>
                    let res := cursorDiffLoop (z' :: sa) (y0 :: sb) rem'
                    { val := res.val.map (.set k v none :: ·)
                      visited := res.visited + 1
                      ptrChecks := res.ptrChecks + 1
                      pushSteps := res.pushSteps + 1 } := by
                rw [cursorDiffLoop.eq_def]
                dsimp only [y', z', y0, mkNode] at hyne ⊢
                rw [hyne]
                cases rem <;> simp [hk1, pushLeft]
              have hzne : ptrEq z' y0 = false := ptrEq_fresh_false (by omega) hid
              dsimp only [x', y', z', x0, y0, mkNode] at hstep1 hstep2 ⊢
              rw [hstep1, hstep2]
              cases rem with
              | zero => simp [idHeight]
              | succ rm =>
                dsimp only
                have hstep3' := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rm hzne
                dsimp only [z', y0, mkNode] at hstep3'
                rw [hstep3']
                have ⟨m1, m2, m3⟩ := cursorDiffLoop_cost_mono sa sb (show rm ≤ rm + 1 by omega)
                simp [idHeight]
                exact ⟨by omega, by omega, by omega⟩
            · -- Inductive LR (`l2 = node idy0 hy0 t2 ky vy t3`)
              subst hl2_eq
              have hself_t2 := validSharing_self_left hself_l2
              have hself_t3 := validSharing_self_right hself_l2
              rcases hbl2 with ⟨ht2_y, ht3_y, hbt2, hbt3⟩
              rcases hal2 with ⟨_, _, _, hat2, hat3⟩
              rcases hl2_x with ⟨hx_y, hx_t2, hx_t3⟩
              rcases hl2_k' with ⟨hy_k', ht2_k', ht3_k'⟩
              rcases hidl2 with ⟨hidy0, hidt2, hidt3⟩
              have hidsb_y0 : StackIdsLt nid (y0 :: sb) :=
                stackIdsLt_cons ⟨hid, ⟨hidl0, hidl1, ⟨hidy0, hidt2, hidt3⟩⟩, hidr⟩ hidsb
              rcases ht_cases with ⟨hk_lt_y, hgt2, hl2'_eq⟩ | ⟨hy_lt_k, hgt3, hl2'_eq⟩
              · -- LR-Left: `t2` grew
                simp only [hl'_eq, mkNode, hsh_l1, hsh_l2', hlr_tall, ite_true]
                simp only [hl2'_eq, mkNode, rotLeft, rotRight, pushLeft]
                let t2' := (avlInsert t2 k v nid).1
                let x' := mkNode nid1 l1 kx vx t2'
                let z' := mkNode (nid1 + 2) t3 k' v' r
                let y' := mkNode (nid1 + 2 + 1) x' ky vy z'
                let l2_0 := IdTree.node idy0 hy0 t2 ky vy t3
                have hskip1 := cursorDiffLoop_pushLeft_self_cost l1 (sa := x' :: y' :: sa) (sb := x0 :: y0 :: sb)
                  rem (noLeaf_cons (noLeaf_cons hnsa)) (noLeaf_cons (noLeaf_cons hnsb))
                have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hidl0
                have hxt2_ne : ptrEq t2' l2_0 = false :=
                  ptrEq_fresh_lt_false (avlInsert_root_fresh t2 k v nid).2 ⟨hidy0, hidt2, hidt3⟩
                have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := y0 :: sb) rem hxne hxt2_ne
                have hih := cursorDiffLoop_avlInsert_cost t2 k v nid (y' :: sa) (l2_0 :: y0 :: sb)
                  (some ky) rem hbt2 hat2 hself_t2 hidt2 ⟨ht2_y, hk_lt_y⟩ (Nat.le_refl ky) (Nat.le_refl ky)
                  (noLeaf_cons hnsa) (noLeaf_cons (noLeaf_cons hnsb))
                  (stackIdsLt_cons ⟨hidy0, hidt2, hidt3⟩ hidsb_y0)
                have hyne : ptrEq y' l2_0 = false := ptrEq_fresh_false (by omega) hidy0
                have hyt3_ne : ptrEq z' t3 = false :=
                  ptrEq_fresh_lt_false ⟨nid1 + 2, _, t3, k', v', r, rfl, Nat.le_refl _⟩
                    (hidt3.mono (Nat.le_trans hnid1 (Nat.le_add_right _ 2)))
                have hstep2 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := y0 :: sb) rem hyne hyt3_ne
                have hskip3 := cursorDiffLoop_pushLeft_self_cost t3 (sa := z' :: sa) (sb := y0 :: sb)
                  rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
                have hzne : ptrEq z' y0 = false := ptrEq_fresh_false (by omega) hid
                have hstep3 := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hzne
                have hpl_t2' := pushLeft_steps_le_height t2' (y' :: sa)
                have hpl_t2 := pushLeft_steps_le_height t2 (l2_0 :: y0 :: sb)
                have hpl_t3a := pushLeft_steps_le_height t3 (z' :: sa)
                have hpl_t3b := pushLeft_steps_le_height t3 (y0 :: sb)
                dsimp only [t2', x', y', z', x0, y0, l2_0, mkNode] at hskip1 hstep1 hih hstep2 hskip3 hstep3 hpl_t2' hpl_t2 hpl_t3a hpl_t3b ⊢
                rw [hstep1] at hskip1
                simp only [pushLeft] at hskip1 hstep2 hskip3 hstep3 hpl_t2' hpl_t2 hpl_t3a hpl_t3b ⊢
                rw [hstep2] at hih
                rw [hstep3] at hskip3
                dsimp only at hskip1 hih hskip3
                have hy0_h : idHeight y0 ≥ idHeight l1 + 2 ∧ idHeight y0 ≥ idHeight t2 + 3 ∧ idHeight y0 ≥ idHeight t3 + 3 := by
                  simp only [y0, x0, idHeight]; omega
                have hq := quad_step3 (a := idHeight t2) (b := idHeight y0) (by omega)
                dsimp only [y0, x0] at hy0_h hq
                exact ⟨by omega, by omega, by omega⟩
              · -- LR-Right: `t3` grew, `t2` unchanged
                simp only [hl'_eq, mkNode, hsh_l1, hsh_l2', hlr_tall, ite_true]
                simp only [hl2'_eq, mkNode, rotLeft, rotRight, pushLeft]
                let t3' := (avlInsert t3 k v nid).1
                let x' := mkNode nid1 l1 kx vx t2
                let z' := mkNode (nid1 + 2) t3' k' v' r
                let y' := mkNode (nid1 + 2 + 1) x' ky vy z'
                let l2_0 := IdTree.node idy0 hy0 t2 ky vy t3
                have hskip1 := cursorDiffLoop_pushLeft_self_cost l1 (sa := x' :: y' :: sa) (sb := x0 :: y0 :: sb)
                  rem (noLeaf_cons (noLeaf_cons hnsa)) (noLeaf_cons (noLeaf_cons hnsb))
                have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hidl0
                have hxt2_ne : ptrEq t2 l2_0 = false :=
                  ptrEq_strict_subtree_false hself_l2 (Subtree.left idy0 hy0 ky vy t2 t3) (Subtree.refl l2_0)
                    (by simp only [l2_0, idSize]; omega)
                have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := y0 :: sb) rem hxne hxt2_ne
                have hskip2 := cursorDiffLoop_pushLeft_self_cost t2 (sa := y' :: sa) (sb := l2_0 :: y0 :: sb)
                  rem (noLeaf_cons hnsa) (noLeaf_cons (noLeaf_cons hnsb))
                have hyne : ptrEq y' l2_0 = false := ptrEq_fresh_false (by omega) hidy0
                have hyt3_ne : ptrEq z' t3 = false :=
                  ptrEq_fresh_lt_false ⟨nid1 + 2, _, t3', k', v', r, rfl, Nat.le_refl _⟩
                    (hidt3.mono (Nat.le_trans hnid1 (Nat.le_add_right _ 2)))
                have hstep2 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := y0 :: sb) rem hyne hyt3_ne
                have hih := cursorDiffLoop_avlInsert_cost t3 k v nid (z' :: sa) (y0 :: sb)
                  (some k') rem hbt3 hat3 hself_t3 hidt3 ⟨ht3_k', hk1⟩ (Nat.le_refl k') (Nat.le_refl k')
                  (noLeaf_cons hnsa) (noLeaf_cons hnsb) hidsb_y0
                have hzne : ptrEq z' y0 = false := ptrEq_fresh_false (by omega) hid
                have hstep3 := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hzne
                have hpl_t2a := pushLeft_steps_le_height t2 (y' :: sa)
                have hpl_t2b := pushLeft_steps_le_height t2 (l2_0 :: y0 :: sb)
                have hpl_t3' := pushLeft_steps_le_height t3' (z' :: sa)
                have hpl_t3 := pushLeft_steps_le_height t3 (y0 :: sb)
                dsimp only [t3', x', y', z', x0, y0, l2_0, mkNode] at hskip1 hstep1 hskip2 hstep2 hih hstep3 hpl_t2a hpl_t2b hpl_t3' hpl_t3 ⊢
                rw [hstep1] at hskip1
                simp only [pushLeft] at hskip1 hstep2 hskip2 hih hstep3 hpl_t2a hpl_t2b hpl_t3' hpl_t3 ⊢
                rw [hstep2] at hskip2
                rw [hstep3] at hih
                dsimp only at hskip1 hskip2 hih
                have hy0_h : idHeight y0 ≥ idHeight l1 + 2 ∧ idHeight y0 ≥ idHeight t2 + 3 ∧ idHeight y0 ≥ idHeight t3 + 3 := by
                  simp only [y0, x0, idHeight]; omega
                have hq := quad_step3 (a := idHeight t3) (b := idHeight y0) (by omega)
                dsimp only [y0, x0] at hy0_h hq
                exact ⟨by omega, by omega, by omega⟩
      · have hnotr : ¬ (idHeight r > idHeight l' + 1) := by omega
        simp only [himb, hnotr, ite_false, mkNode, pushLeft]
        let y' := mkNode nid1 l' k' v' r
        let y0 := IdTree.node id h l k' v' r
        have hih := cursorDiffLoop_avlInsert_cost l k v nid (y' :: sa) (y0 :: sb)
          (some k') rem hbl hal hself_l hidl ⟨hlk, hk1⟩ (Nat.le_refl k') (Nat.le_refl k')
          (noLeaf_cons hnsa) (noLeaf_cons hnsb)
          (stackIdsLt_cons ⟨hid, hidl, hidr⟩ hidsb)
        rw [hres] at hih
        have hyne : ptrEq y' y0 = false := ptrEq_fresh_false (by omega) hid
        have hstep := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hyne
        dsimp only [y', y0, mkNode] at hih hstep ⊢
        rw [hstep] at hih
        dsimp only at hih
        have hy0_h : idHeight y0 ≥ idHeight l + 1 := by simp only [y0, idHeight]; omega
        have hq := quad_step1 (a := idHeight l) (b := idHeight y0) hy0_h
        dsimp only [y0] at hy0_h hq
        exact ⟨by omega, by omega, by omega⟩
    · by_cases hk2 : k' < k
      · simp only [hk1, hk2, ite_true, ite_false]
        have ispec := avlInsert_spec r k v nid hbr har hidr
        unfold AvlInsertResult at ispec
        generalize hres : avlInsert r k v nid = res at ispec ⊢
        rcases res with ⟨r', nid1⟩
        rcases ispec with ⟨hbr', har', hhr_lo, hhr_hi, hidr', hnid1, _, _, hgrowr⟩
        have hshl := avl_storedHeight hal
        have hshr' := avl_storedHeight har'
        unfold rebalance
        simp only [hshl, hshr']
        have hnotl : ¬ (idHeight l > idHeight r' + 1) := by omega
        simp only [hnotl, ite_false]
        by_cases himb : idHeight r' > idHeight l + 1
        · simp only [himb, ite_true]
          have hgrow : idHeight r' = idHeight r + 1 := by omega
          have hr_eq_l1 : idHeight r = idHeight l + 1 := by omega
          rcases hgrowr hgrow with ⟨hr_leaf, _⟩ | ⟨idr0, hr0, r1, kx, vx, r2, hreq, heq_r12, hr_cases⟩
          · subst hr_leaf; simp [idHeight] at hr_eq_l1
          · subst hreq
            have hself_r1 := validSharing_self_left hself_r
            have hself_r2 := validSharing_self_right hself_r
            rcases hbr with ⟨hr1_x, hr2_x, hbr1, hbr2⟩
            rcases har with ⟨_, _, _, har1, har2⟩
            rcases hkr with ⟨hk'_kx, hk'_r1, hk'_r2⟩
            rcases hidr with ⟨hidr0, hidr1, hidr2⟩
            rcases hr_cases with ⟨hk_lt_x, hgr1, hr'_eq⟩ | ⟨hx_lt_k, hgr2, hr'_eq⟩
            · -- RL rotation
              have ir1 := avlInsert_spec r1 k v nid hbr1 har1 hidr1
              unfold AvlInsertResult at ir1
              generalize hres1 : avlInsert r1 k v nid = res1 at ir1 hr'_eq hgr1 ⊢
              rcases res1 with ⟨r1', nid2⟩
              dsimp only at hgr1 hr'_eq
              rcases ir1 with ⟨hbr1', har1', _, _, _, hnid2, _, _, hgrowr1⟩
              have hsh_r1' := avl_storedHeight har1'
              have hsh_r2 := avl_storedHeight har2
              have hrl_tall : idHeight r2 < idHeight r1' := by omega
              let x0 := IdTree.node id h l k' v' (IdTree.node idr0 hr0 r1 kx vx r2)
              let z0 := IdTree.node idr0 hr0 r1 kx vx r2
              rcases hgrowr1 hgr1 with ⟨hr1_leaf, hr1'_eq⟩ | ⟨idy0, hy0, t2, ky, vy, t3, hr1_eq, heq_t23, ht_cases⟩
              · -- Base RL (`r1 = .leaf`, `r2 = .leaf`, `l = .leaf`)
                subst hr1_leaf
                have hr2_leaf : r2 = .leaf := by
                  cases r2 with
                  | leaf => rfl
                  | node _ _ _ _ _ _ => simp [idHeight] at heq_r12
                have hl_leaf : l = .leaf := by
                  cases l with
                  | leaf => rfl
                  | node _ _ _ _ _ _ => simp [idHeight] at hr_eq_l1 heq_r12; omega
                subst hr2_leaf; subst hl_leaf
                simp only [hr'_eq, mkNode, hsh_r1', hsh_r2, hrl_tall, ite_true]
                simp only [hr1'_eq, mkNode, rotRight, rotLeft, pushLeft]
                let x' := mkNode (nid1 + 2) .leaf k' v' .leaf
                let z' := mkNode nid1 .leaf kx vx .leaf
                let y' := mkNode (nid1 + 2 + 1) x' k v z'
                have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hid
                have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := sb) rem hxne
                  (by simp [ptrEq])
                have hyne : ptrEq y' z0 = false := ptrEq_fresh_false (by omega) hidr0
                have hstep2 : cursorDiffLoop (y' :: sa) (z0 :: sb) rem =
                    match rem with
                    | 0 => { val := none, visited := 1, ptrChecks := 1, pushSteps := 0 }
                    | rem' + 1 =>
                      let res := cursorDiffLoop (z' :: sa) (z0 :: sb) rem'
                      { val := res.val.map (.set k v none :: ·)
                        visited := res.visited + 1
                        ptrChecks := res.ptrChecks + 1
                        pushSteps := res.pushSteps + 1 } := by
                  rw [cursorDiffLoop.eq_def]
                  dsimp only [y', z', z0, mkNode] at hyne ⊢
                  rw [hyne]
                  cases rem <;> simp [hk_lt_x, pushLeft]
                have hzne : ptrEq z' z0 = false := ptrEq_fresh_false (by omega) hidr0
                dsimp only [x', y', z', x0, z0, mkNode] at hstep1 hstep2 ⊢
                rw [hstep1]
                simp only [pushLeft]
                rw [hstep2]
                cases rem with
                | zero => simp [idHeight]
                | succ rm =>
                  dsimp only
                  have hstep3' := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rm hzne
                  dsimp only [z', z0, mkNode] at hstep3'
                  rw [hstep3']
                  have ⟨m1, m2, m3⟩ := cursorDiffLoop_cost_mono sa sb (show rm ≤ rm + 1 by omega)
                  simp [idHeight]
                  exact ⟨by omega, by omega, by omega⟩
              · -- Inductive RL (`r1 = node idy0 hy0 t2 ky vy t3`)
                subst hr1_eq
                have hself_t2 := validSharing_self_left hself_r1
                have hself_t3 := validSharing_self_right hself_r1
                rcases hbr1 with ⟨ht2_y, ht3_y, hbt2, hbt3⟩
                rcases har1 with ⟨_, _, _, hat2, hat3⟩
                rcases hr1_x with ⟨hy_kx, ht2_kx, ht3_kx⟩
                rcases hk'_r1 with ⟨hk'_y, hk'_t2, hk'_t3⟩
                rcases hidr1 with ⟨hidy0, hidt2, hidt3⟩
                have hidsb_z0 : StackIdsLt nid (z0 :: sb) :=
                  stackIdsLt_cons ⟨hidr0, ⟨hidy0, hidt2, hidt3⟩, hidr2⟩ hidsb
                rcases ht_cases with ⟨hk_lt_y, hgt2, hr1'_eq⟩ | ⟨hy_lt_k, hgt3, hr1'_eq⟩
                · -- RL-Left: `t2` grew
                  simp only [hr'_eq, mkNode, hsh_r1', hsh_r2, hrl_tall, ite_true]
                  simp only [hr1'_eq, mkNode, rotRight, rotLeft, pushLeft]
                  let t2' := (avlInsert t2 k v nid).1
                  let x' := mkNode (nid1 + 2) l k' v' t2'
                  let z' := mkNode nid1 t3 kx vx r2
                  let y' := mkNode (nid1 + 2 + 1) x' ky vy z'
                  let r1_0 := IdTree.node idy0 hy0 t2 ky vy t3
                  have hskip1 := cursorDiffLoop_pushLeft_self_cost l (sa := x' :: y' :: sa) (sb := x0 :: sb)
                    rem (noLeaf_cons (noLeaf_cons hnsa)) (noLeaf_cons hnsb)
                  have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hid
                  have hxt2_ne : ptrEq t2' z0 = false :=
                    ptrEq_fresh_lt_false (avlInsert_root_fresh t2 k v nid).2 ⟨hidr0, ⟨hidy0, hidt2, hidt3⟩, hidr2⟩
                  have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := sb) rem hxne hxt2_ne
                  have hih := cursorDiffLoop_avlInsert_cost t2 k v nid (y' :: sa) (r1_0 :: z0 :: sb)
                    (some ky) rem hbt2 hat2 hself_t2 hidt2 ⟨ht2_y, hk_lt_y⟩ (Nat.le_refl ky) (Nat.le_refl ky)
                    (noLeaf_cons hnsa) (noLeaf_cons (noLeaf_cons hnsb))
                    (stackIdsLt_cons ⟨hidy0, hidt2, hidt3⟩ hidsb_z0)
                  have hyne : ptrEq y' r1_0 = false := ptrEq_fresh_false (by omega) hidy0
                  have hyt3_ne : ptrEq z' t3 = false :=
                    ptrEq_fresh_lt_false ⟨nid1, _, t3, kx, vx, r2, rfl, Nat.le_refl nid1⟩ (hidt3.mono hnid1)
                  have hstep2 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := z0 :: sb) rem hyne hyt3_ne
                  have hskip3 := cursorDiffLoop_pushLeft_self_cost t3 (sa := z' :: sa) (sb := z0 :: sb)
                    rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
                  have hzne : ptrEq z' z0 = false := ptrEq_fresh_false (by omega) hidr0
                  have hstep3 := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hzne
                  have hpl_t2' := pushLeft_steps_le_height t2' (y' :: sa)
                  have hpl_t2 := pushLeft_steps_le_height t2 (r1_0 :: z0 :: sb)
                  have hpl_t3a := pushLeft_steps_le_height t3 (z' :: sa)
                  have hpl_t3b := pushLeft_steps_le_height t3 (z0 :: sb)
                  dsimp only [t2', x', y', z', x0, z0, r1_0, mkNode] at hskip1 hstep1 hih hstep2 hskip3 hstep3 hpl_t2' hpl_t2 hpl_t3a hpl_t3b ⊢
                  rw [hstep1] at hskip1
                  simp only [pushLeft] at hskip1 hstep2 hskip3 hstep3 hpl_t2' hpl_t2 hpl_t3a hpl_t3b ⊢
                  rw [hstep2] at hih
                  rw [hstep3] at hskip3
                  dsimp only at hskip1 hih hskip3
                  have hx0_h : idHeight x0 ≥ idHeight l + 1 ∧ idHeight x0 ≥ idHeight t2 + 3 ∧ idHeight x0 ≥ idHeight t3 + 3 := by
                    simp only [x0, idHeight]; omega
                  have hq := quad_step3 (a := idHeight t2) (b := idHeight x0) (by omega)
                  dsimp only [x0] at hx0_h hq
                  exact ⟨by omega, by omega, by omega⟩
                · -- RL-Right: `t3` grew, `t2` unchanged
                  simp only [hr'_eq, mkNode, hsh_r1', hsh_r2, hrl_tall, ite_true]
                  simp only [hr1'_eq, mkNode, rotRight, rotLeft, pushLeft]
                  let t3' := (avlInsert t3 k v nid).1
                  let x' := mkNode (nid1 + 2) l k' v' t2
                  let z' := mkNode nid1 t3' kx vx r2
                  let y' := mkNode (nid1 + 2 + 1) x' ky vy z'
                  let r1_0 := IdTree.node idy0 hy0 t2 ky vy t3
                  have hskip1 := cursorDiffLoop_pushLeft_self_cost l (sa := x' :: y' :: sa) (sb := x0 :: sb)
                    rem (noLeaf_cons (noLeaf_cons hnsa)) (noLeaf_cons hnsb)
                  have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hid
                  have hxt2_ne : ptrEq t2 z0 = false :=
                    ptrEq_strict_subtree_false hself_r
                      (Subtree.trans (Subtree.left idy0 hy0 ky vy t2 t3) (Subtree.left idr0 hr0 kx vx r1_0 r2))
                      (Subtree.refl z0)
                      (by simp only [z0, idSize]; omega)
                  have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := sb) rem hxne hxt2_ne
                  have hskip2 := cursorDiffLoop_pushLeft_self_cost t2 (sa := y' :: sa) (sb := r1_0 :: z0 :: sb)
                    rem (noLeaf_cons hnsa) (noLeaf_cons (noLeaf_cons hnsb))
                  have hyne : ptrEq y' r1_0 = false := ptrEq_fresh_false (by omega) hidy0
                  have hyt3_ne : ptrEq z' t3 = false :=
                    ptrEq_fresh_lt_false ⟨nid1, _, t3', kx, vx, r2, rfl, Nat.le_refl nid1⟩ (hidt3.mono hnid1)
                  have hstep2 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := z0 :: sb) rem hyne hyt3_ne
                  have hih := cursorDiffLoop_avlInsert_cost t3 k v nid (z' :: sa) (z0 :: sb)
                    (some kx) rem hbt3 hat3 hself_t3 hidt3 ⟨ht3_kx, hk_lt_x⟩ (Nat.le_refl kx) (Nat.le_refl kx)
                    (noLeaf_cons hnsa) (noLeaf_cons hnsb) hidsb_z0
                  have hzne : ptrEq z' z0 = false := ptrEq_fresh_false (by omega) hidr0
                  have hstep3 := cursorDiffLoop_same_kv_shared_r (sa := sa) (sb := sb) rem hzne
                  have hpl_t2a := pushLeft_steps_le_height t2 (y' :: sa)
                  have hpl_t2b := pushLeft_steps_le_height t2 (r1_0 :: z0 :: sb)
                  have hpl_t3' := pushLeft_steps_le_height t3' (z' :: sa)
                  have hpl_t3 := pushLeft_steps_le_height t3 (z0 :: sb)
                  dsimp only [t3', x', y', z', x0, z0, r1_0, mkNode] at hskip1 hstep1 hskip2 hstep2 hih hstep3 hpl_t2a hpl_t2b hpl_t3' hpl_t3 ⊢
                  rw [hstep1] at hskip1
                  simp only [pushLeft] at hskip1 hstep2 hskip2 hih hstep3 hpl_t2a hpl_t2b hpl_t3' hpl_t3 ⊢
                  rw [hstep2] at hskip2
                  rw [hstep3] at hih
                  dsimp only at hskip1 hskip2 hih
                  have hx0_h : idHeight x0 ≥ idHeight l + 1 ∧ idHeight x0 ≥ idHeight t2 + 3 ∧ idHeight x0 ≥ idHeight t3 + 3 := by
                    simp only [x0, idHeight]; omega
                  have hq := quad_step3 (a := idHeight t3) (b := idHeight x0) (by omega)
                  dsimp only [x0] at hx0_h hq
                  exact ⟨by omega, by omega, by omega⟩
            · -- RR rotation
              have hsh_r1 := avl_storedHeight har1
              have hsh_r2' : storedHeight (avlInsert r2 k v nid).1 = idHeight (avlInsert r2 k v nid).1 := by
                rw [hr'_eq] at har'; exact avl_storedHeight har'.2.2.2.2
              have hnot_rl : ¬ (idHeight (avlInsert r2 k v nid).1 < idHeight r1) := by omega
              simp only [hr'_eq, mkNode, hsh_r1, hsh_r2', hnot_rl, ite_false, rotLeft, pushLeft]
              let r2' := (avlInsert r2 k v nid).1
              let x' := mkNode nid1 l k' v' r1
              let y' := mkNode (nid1 + 1) x' kx vx r2'
              let z0 := IdTree.node idr0 hr0 r1 kx vx r2
              let x0 := IdTree.node id h l k' v' z0
              have hskip1 := cursorDiffLoop_pushLeft_self_cost l (sa := x' :: y' :: sa) (sb := x0 :: sb)
                rem (noLeaf_cons (noLeaf_cons hnsa)) (noLeaf_cons hnsb)
              have hxne : ptrEq x' x0 = false := ptrEq_fresh_false (by omega) hid
              have hxr1_ne : ptrEq r1 z0 = false :=
                ptrEq_strict_subtree_false hself_r (Subtree.left idr0 hr0 kx vx r1 r2) (Subtree.refl z0)
                  (by simp only [z0, idSize]; omega)
              have hstep1 := cursorDiffLoop_same_kv_diff_r (sa := y' :: sa) (sb := sb) rem hxne hxr1_ne
              have hskip2 := cursorDiffLoop_pushLeft_self_cost r1 (sa := y' :: sa) (sb := z0 :: sb)
                rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
              have hyne : ptrEq y' z0 = false := ptrEq_fresh_false (by omega) hidr0
              have hyr2_ne : ptrEq r2' r2 = false :=
                ptrEq_fresh_lt_false (avlInsert_root_fresh r2 k v nid).2 hidr2
              have hstep2 := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := sb) rem hyne hyr2_ne
              have hih := cursorDiffLoop_avlInsert_cost r2 k v nid sa sb ub rem hbr2 har2 hself_r2 hidr2
                (rightUb (rightUb hub)) hsa hsb hnsa hnsb hidsb
              have hpl_r1a := pushLeft_steps_le_height r1 (y' :: sa)
              have hpl_r1b := pushLeft_steps_le_height r1 (z0 :: sb)
              have hpl_r2' := pushLeft_steps_le_height r2' sa
              have hpl_r2 := pushLeft_steps_le_height r2 sb
              dsimp only [r2', x', y', x0, z0, mkNode] at hskip1 hstep1 hskip2 hstep2 hih hpl_r1a hpl_r1b hpl_r2' hpl_r2 ⊢
              rw [hstep1] at hskip1
              simp only [hsh_r1, hsh_r2', pushLeft] at hskip1 hstep2 hskip2 hpl_r1a hpl_r1b hpl_r2' hpl_r2 ⊢
              rw [hstep2] at hskip2
              dsimp only at hskip1 hskip2 hih
              have hx0_h : idHeight x0 ≥ idHeight l + 1 ∧ idHeight x0 ≥ idHeight r1 + 2 ∧ idHeight x0 ≥ idHeight r2 + 2 := by
                simp only [x0, z0, idHeight]; omega
              have hq := quad_step2 (a := idHeight r2) (b := idHeight x0) (by omega)
              dsimp only [x0, z0] at hx0_h hq
              exact ⟨by omega, by omega, by omega⟩
        · simp only [himb, ite_false, mkNode, pushLeft]
          let y' := mkNode nid1 l k' v' r'
          let y0 := IdTree.node id h l k' v' r
          have hskip := cursorDiffLoop_pushLeft_self_cost l (sa := y' :: sa) (sb := y0 :: sb)
            rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
          have hyne : ptrEq y' y0 = false := ptrEq_fresh_false (by omega) hid
          have hrne : ptrEq r' r = false := by
            have hf := (avlInsert_root_fresh r k v nid).2
            rw [hres] at hf
            exact ptrEq_fresh_lt_false hf hidr
          have hstep := cursorDiffLoop_same_kv_diff_r (sa := sa) (sb := sb) rem hyne hrne
          have hih := cursorDiffLoop_avlInsert_cost r k v nid sa sb ub rem hbr har hself_r hidr
            (rightUb hub) hsa hsb hnsa hnsb hidsb
          rw [hres] at hih
          have hpl_r' := pushLeft_steps_le_height r' sa
          have hpl_r := pushLeft_steps_le_height r sb
          dsimp only [y', y0, mkNode] at hskip hstep hih hpl_r' hpl_r ⊢
          rw [hstep] at hskip
          dsimp only at hskip hih
          have hy0_h : idHeight y0 ≥ idHeight l + 1 ∧ idHeight y0 ≥ idHeight r + 1 := by
            simp only [y0, idHeight]; omega
          have hq := quad_step1 (a := idHeight r) (b := idHeight y0) hy0_h.2
          dsimp only [y0] at hy0_h hq
          exact ⟨by omega, by omega, by omega⟩
      · have hk_eq : k = k' := by omega
        subst hk_eq
        simp only [hk1, ite_false, mkNode, pushLeft]
        let y' := mkNode nid l k v r
        let y0 := IdTree.node id h l k v' r
        have hskip := cursorDiffLoop_pushLeft_self_cost l (sa := y' :: sa) (sb := y0 :: sb)
          rem (noLeaf_cons hnsa) (noLeaf_cons hnsb)
        have hyne : ptrEq y' y0 = false := ptrEq_fresh_false (Nat.le_refl nid) hid
        have hstep : (cursorDiffLoop (y' :: sa) (y0 :: sb) rem).visited ≤ (cursorDiffLoop sa sb rem).visited + 1 ∧
                     (cursorDiffLoop (y' :: sa) (y0 :: sb) rem).ptrChecks ≤ (cursorDiffLoop sa sb rem).ptrChecks + 2 ∧
                     (cursorDiffLoop (y' :: sa) (y0 :: sb) rem).pushSteps ≤ (cursorDiffLoop sa sb rem).pushSteps := by
          rw [cursorDiffLoop.eq_def]
          dsimp only [y', y0, mkNode] at hyne ⊢
          rw [hyne]
          simp only [Bool.false_eq_true, ite_false, Nat.lt_irrefl, ptrEq_refl, ite_true]
          by_cases hv : v = v'
          · simp only [hv, ite_true]; exact ⟨by omega, by omega, Nat.le_refl _⟩
          · simp only [hv, ite_false]
            cases rem with
            | zero => simp
            | succ rm =>
              dsimp only
              have ⟨m1, m2, m3⟩ := cursorDiffLoop_cost_mono sa sb (Nat.le_succ rm)
              change _ ≤ (cursorDiffLoop sa sb rm.succ).visited + 1 ∧
                     _ ≤ (cursorDiffLoop sa sb rm.succ).ptrChecks + 2 ∧
                     _ ≤ (cursorDiffLoop sa sb rm.succ).pushSteps
              exact ⟨by omega, by omega, m3⟩
        dsimp only [y', y0, mkNode] at hskip hstep ⊢
        have hy0_h : idHeight y0 ≥ idHeight l + 1 := by simp only [y0, idHeight]; omega
        have hqb := quad_base (a := idHeight y0)
        dsimp only [y0] at hy0_h hqb
        exact ⟨by omega, by omega, by omega⟩
termination_by t _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ => idSize t
decreasing_by
  all_goals
    simp_wf
    try subst_vars
    simp only [idSize]
    omega

/-- Exact height-bounded sharing cost of `cursorDiff` after a balanced `avlInsert`
(including rotations and charging every O(1) `ptrEq` check and `pushLeft` push). -/
theorem cursorDiff_after_avlInsert_steps_bounded {α : Type} [DecidableEq α]
    {t : IdTree α} (hb : BST t) (ha : AVL t) (hself : ValidSharing t t)
    {nid : Nat} (hlt : IdsLt nid t) (k : Nat) (v : α) (limit : Nat) :
    let r := cursorDiff (avlInsert t k v nid).1 t limit
    r.visited ≤ idHeight t + 3 ∧
    r.totalSteps ≤ 4 * ((idHeight t + 3) * (idHeight t + 4)) := by
  dsimp only
  unfold cursorDiff DiffRun.totalSteps
  split
  · simp only
    have hqb := quad_base (a := idHeight t)
    omega
  · dsimp only
    have h := cursorDiffLoop_avlInsert_cost t k v nid [] [] none limit
      hb ha hself hlt trivial trivial trivial (fun _ hx => by cases hx) (fun _ hx => by cases hx)
      (fun _ hx => by cases hx)
    have hspec := avlInsert_spec t k v nid hb ha hlt
    unfold AvlInsertResult at hspec
    have hpl1 := pushLeft_steps_le_height (avlInsert t k v nid).1 []
    have hpl2 := pushLeft_steps_le_height t []
    simp only [cursorDiffLoop] at h
    have hqb := quad_base (a := idHeight t)
    have hq3 : 3 * (idHeight t + 3) ≤ (idHeight t + 3) * (idHeight t + 4) := by
      rw [Nat.mul_comm 3]
      exact Nat.mul_le_mul_left _ (by omega)
    omega

/-- Composed `O(log N)` node-visit and `O((log N)^2)` total-step bound for
`cursorDiff` after a balanced `avlInsert`, charging every O(1) `ptrEq` check,
stack push, and node visit on an `N`-node AVL tree. -/
theorem cursorDiff_after_avlInsert_log_bound {α : Type} [DecidableEq α]
    {t : IdTree α} (hb : BST t) (ha : AVL t) (hself : ValidSharing t t)
    {nid : Nat} (hlt : IdsLt nid t) (k : Nat) (v : α) (limit : Nat) :
    let r := cursorDiff (avlInsert t k v nid).1 t limit
    let L := 2 * Nat.log2 (idSize t + 1) + 1
    r.visited ≤ L + 3 ∧
    r.totalSteps ≤ 4 * ((L + 3) * (L + 4)) := by
  have ⟨hv, ht⟩ := cursorDiff_after_avlInsert_steps_bounded hb ha hself hlt k v limit
  have hlog := avl_height_le_two_log2 ha
  have hm : (idHeight t + 3) * (idHeight t + 4) ≤
      (2 * Nat.log2 (idSize t + 1) + 1 + 3) * (2 * Nat.log2 (idSize t + 1) + 1 + 4) :=
    Nat.mul_le_mul (by omega) (by omega)
  exact ⟨by omega, by omega⟩

/-!
## Multi-layer cold-chain resolution (`store_delta.rs`)
-/

/-- One cold checkpoint naming root layer in a `delta_base` chain. -/
structure ColdLayer (α : Type) where
  activeDomain : Bool
  isDelta : Bool
  rows : List (Nat × DeltaOp α)

/-- `ColdNamingStore::active_chain` (`store_delta.rs:84-106`). -/
def activeChain {α : Type} : List (ColdLayer α) → List (ColdLayer α)
  | [] => []
  | l :: rest =>
    if !l.activeDomain || !l.isDelta then [l]
    else match rest with
      | [] => [l]
      | next :: _ =>
        if !next.activeDomain then [l]
        else l :: activeChain rest

/-- Point lookup inside one layer's sorted rows. -/
def layerLookup {α : Type} : List (Nat × DeltaOp α) → Nat → Option (DeltaOp α)
  | [], _ => none
  | (k, op) :: rest, q => if q = k then some op else layerLookup rest q

/-- Applying one delta layer over `base` at key `q`. -/
def applyDelta {α : Type} (base : Nat → Option α) (delta : Nat → Option (DeltaOp α)) (q : Nat) :
    Option α :=
  match delta q with
  | some (.put v) => some v
  | some .tombstone => none
  | none => base q

/-- Top-down lookup across a layer list (`ColdNamingStore::get_root`). -/
def resolveLayers {α : Type} : List (ColdLayer α) → Nat → Option α
  | [], _ => none
  | l :: rest, q =>
    match layerLookup l.rows q with
    | some (.put v) => some v
    | some .tombstone => none
    | none => resolveLayers rest q

/-- Bottom-up layer composition over a layer list. -/
def applyLayers {α : Type} : List (ColdLayer α) → Nat → Option α
  | [], _ => none
  | l :: rest, q => applyDelta (applyLayers rest) (layerLookup l.rows) q

/-- Top-down `get_root` resolution over `activeChain`. -/
def resolveActiveChain {α : Type} (layers : List (ColdLayer α)) (q : Nat) : Option α :=
  resolveLayers (activeChain layers) q

/-- Bottom-up composition over `activeChain`. -/
def applyActiveChain {α : Type} (layers : List (ColdLayer α)) (q : Nat) : Option α :=
  applyLayers (activeChain layers) q

theorem resolveLayers_eq_applyLayers {α : Type} :
    ∀ (layers : List (ColdLayer α)) (q : Nat),
      resolveLayers layers q = applyLayers layers q
  | [], _ => rfl
  | l :: rest, q => by
    simp only [resolveLayers, applyLayers, applyDelta]
    cases layerLookup l.rows q with
    | none => exact resolveLayers_eq_applyLayers rest q
    | some op => cases op <;> rfl

/-- `ColdNamingStore::get_root` over `active_chain` equals layer-by-layer composition. -/
theorem resolveActiveChain_eq_applyActiveChain {α : Type}
    (layers : List (ColdLayer α)) (q : Nat) :
    resolveActiveChain layers q = applyActiveChain layers q :=
  resolveLayers_eq_applyLayers (activeChain layers) q

/-- Sorted merge of two delta-op row lists, top layer shadowing bottom on equal keys. -/
def mergeLayerRows {α : Type} :
    List (Nat × DeltaOp α) → List (Nat × DeltaOp α) → List (Nat × DeltaOp α)
  | [], ys => ys
  | (k, op) :: xs, [] => (k, op) :: xs
  | (k1, op1) :: xs, (k2, op2) :: ys =>
    if k1 < k2 then (k1, op1) :: mergeLayerRows xs ((k2, op2) :: ys)
    else if k2 < k1 then (k2, op2) :: mergeLayerRows ((k1, op1) :: xs) ys
    else (k1, op1) :: mergeLayerRows xs ys

/-- Merged delta ops across a list of layers (newest first). -/
def mergeLayers {α : Type} : List (ColdLayer α) → List (Nat × DeltaOp α)
  | [] => []
  | l :: rest => mergeLayerRows l.rows (mergeLayers rest)

/-- Filter live `.put` rows from a merged op list (dropping `.tombstone`s). -/
def liveRows {α : Type} : List (Nat × DeltaOp α) → List (Nat × α)
  | [] => []
  | (k, .put v) :: rest => (k, v) :: liveRows rest
  | (_, .tombstone) :: rest => liveRows rest

/-- Ordered scan over `activeChain` (`ColdNamingStore::scan_root`). -/
def scanActiveChain {α : Type} (layers : List (ColdLayer α)) : List (Nat × α) :=
  liveRows (mergeLayers (activeChain layers))

theorem mergeLayerRows_sortedAbove {α : Type} :
    ∀ {xs ys : List (Nat × DeltaOp α)} {lo : Option Nat},
      SortedKeysAbove lo xs → SortedKeysAbove lo ys →
      SortedKeysAbove lo (mergeLayerRows xs ys)
  | [], ys, _, _, hys => by simpa [mergeLayerRows] using hys
  | (k, op) :: xs, [], _, hxs, _ => by simpa [mergeLayerRows] using hxs
  | (k1, op1) :: xs, (k2, op2) :: ys, _, ⟨hk1, hxs⟩, ⟨hk2, hys⟩ => by
    unfold mergeLayerRows
    by_cases h1 : k1 < k2
    · simp only [h1, ite_true]
      exact ⟨hk1, mergeLayerRows_sortedAbove hxs ⟨h1, hys⟩⟩
    · by_cases h2 : k2 < k1
      · simp only [h1, h2, ite_true, ite_false]
        exact ⟨hk2, mergeLayerRows_sortedAbove ⟨h2, hxs⟩ hys⟩
      · have heq : k1 = k2 := by omega
        subst heq
        simp only [h1, ite_false]
        exact ⟨hk1, mergeLayerRows_sortedAbove hxs hys⟩

theorem liveRows_sortedAbove {α : Type} :
    ∀ {xs : List (Nat × DeltaOp α)} {lo : Option Nat},
      SortedKeysAbove lo xs → SortedKeysAbove lo (liveRows xs)
  | [], _, _ => trivial
  | (k, .put v) :: xs, _, ⟨hk, hxs⟩ => ⟨hk, liveRows_sortedAbove hxs⟩
  | (k, .tombstone) :: xs, lo, ⟨hk, hxs⟩ => by
    simp only [liveRows]
    exact sortedKeysAbove_mono (lo₁ := lo) (lo₂ := some k) (liveRows_sortedAbove hxs) fun q hq => by
      cases lo with
      | none => trivial
      | some m => exact Nat.lt_trans hk hq

theorem layerLookup_none_of_sortedAbove {α : Type} :
    ∀ {xs : List (Nat × DeltaOp α)} {m q : Nat},
      SortedKeysAbove (some m) xs → q ≤ m → layerLookup xs q = none
  | [], _, _, _, _ => rfl
  | (k, _) :: xs, m, q, ⟨hmk, hxs⟩, hq => by
    have hne : q ≠ k := by omega
    simp only [layerLookup, hne, ite_false]
    exact layerLookup_none_of_sortedAbove hxs (by omega)

theorem layerLookup_mergeLayerRows {α : Type} :
    ∀ {xs ys : List (Nat × DeltaOp α)} {lo : Option Nat},
      SortedKeysAbove lo xs → SortedKeysAbove lo ys →
      ∀ q, layerLookup (mergeLayerRows xs ys) q =
        match layerLookup xs q with
        | some op => some op
        | none => layerLookup ys q
  | [], ys, _, _, _, q => by simp [mergeLayerRows, layerLookup]
  | (k, op) :: xs, [], _, _, _, q => by
    simp only [mergeLayerRows, layerLookup]
    cases (if q = k then some op else layerLookup xs q) <;> rfl
  | (k1, op1) :: xs, (k2, op2) :: ys, _, ⟨_, hxs⟩, ⟨_, hys⟩, q => by
    unfold mergeLayerRows
    by_cases h1 : k1 < k2
    · simp only [h1, ite_true, layerLookup]
      by_cases hq : q = k1
      · simp [hq]
      · simp only [hq, ite_false]
        have hys' : SortedKeysAbove (some k1) ((k2, op2) :: ys) := ⟨h1, hys⟩
        exact layerLookup_mergeLayerRows hxs hys' q
    · by_cases h2 : k2 < k1
      · simp only [h1, h2, ite_true, ite_false, layerLookup]
        by_cases hq : q = k2
        · subst hq
          have hne : q ≠ k1 := by omega
          simp only [ite_true, hne, ite_false]
          rw [layerLookup_none_of_sortedAbove hxs (by omega)]
        · simp only [hq, ite_false]
          have hxs' : SortedKeysAbove (some k2) ((k1, op1) :: xs) := ⟨h2, hxs⟩
          exact layerLookup_mergeLayerRows hxs' hys q
      · have heq : k1 = k2 := by omega
        subst heq
        simp only [h1, ite_false, layerLookup]
        by_cases hq : q = k1
        · simp [hq]
        · simp only [hq, ite_false]
          exact layerLookup_mergeLayerRows hxs hys q

theorem listLookup_liveRows {α : Type} :
    ∀ {xs : List (Nat × DeltaOp α)} {lo : Option Nat},
      SortedKeysAbove lo xs →
      ∀ q, listLookup (liveRows xs) q =
        match layerLookup xs q with
        | some (.put v) => some v
        | some .tombstone => none
        | none => none
  | [], _, _, _ => rfl
  | (k, .put v) :: xs, _, ⟨_, hxs⟩, q => by
    simp only [liveRows, listLookup, layerLookup]
    by_cases hq : q = k
    · simp [hq]
    · simp only [hq, ite_false]; exact listLookup_liveRows hxs q
  | (k, .tombstone) :: xs, _, ⟨_, hxs⟩, q => by
    simp only [liveRows, layerLookup]
    by_cases hq : q = k
    · subst hq
      simp only [ite_true]
      exact listLookup_none_of_sortedAbove (liveRows_sortedAbove hxs) (Nat.le_refl q)
    · simp only [hq, ite_false]; exact listLookup_liveRows hxs q

theorem activeChain_subset {α : Type} :
    ∀ (layers : List (ColdLayer α)) (l : ColdLayer α),
      l ∈ activeChain layers → l ∈ layers
  | [], _, hl => by simp [activeChain] at hl
  | x :: rest, l, hl => by
    unfold activeChain at hl
    split at hl
    · simp only [List.mem_singleton] at hl; subst hl; exact List.mem_cons_self ..
    · cases rest with
      | nil => simp only [List.mem_singleton] at hl; subst hl; exact List.mem_cons_self ..
      | cons next tail =>
        dsimp only at hl
        split at hl
        · simp only [List.mem_singleton] at hl; subst hl; exact List.mem_cons_self ..
        · rcases List.mem_cons.mp hl with rfl | htail
          · exact List.mem_cons_self ..
          · exact List.mem_cons_of_mem _ (activeChain_subset (next :: tail) l htail)

theorem mergeLayers_sorted {α : Type} :
    ∀ {layers : List (ColdLayer α)},
      (∀ l ∈ layers, SortedKeys l.rows) →
      SortedKeys (mergeLayers layers)
  | [], _ => trivial
  | l :: _rest, h =>
    mergeLayerRows_sortedAbove
      (h l (List.mem_cons_self ..))
      (mergeLayers_sorted fun x hx => h x (List.mem_cons_of_mem _ hx))

theorem resolveLayers_eq_live_merge {α : Type} :
    ∀ {layers : List (ColdLayer α)},
      (∀ l ∈ layers, SortedKeys l.rows) →
      ∀ q, listLookup (liveRows (mergeLayers layers)) q = resolveLayers layers q
  | [], _, _ => rfl
  | l :: _rest, h, q => by
    have hl : SortedKeys l.rows := h l (List.mem_cons_self ..)
    have hrest : ∀ x ∈ _rest, SortedKeys x.rows := fun x hx => h x (List.mem_cons_of_mem _ hx)
    have hmerge := mergeLayers_sorted hrest
    simp only [mergeLayers, resolveLayers]
    rw [listLookup_liveRows (mergeLayerRows_sortedAbove hl hmerge) q]
    rw [layerLookup_mergeLayerRows hl hmerge q]
    cases layerLookup l.rows q with
    | some op => cases op <;> rfl
    | none =>
      rw [← listLookup_liveRows hmerge q]
      exact resolveLayers_eq_live_merge hrest q

/-- Ordered `ColdNamingStore::scan_root` over `active_chain` produces strictly
ascending keys whose values agree identically with `get_root` (`resolveActiveChain`). -/
theorem scanActiveChain_sound {α : Type}
    {layers : List (ColdLayer α)} (h : ∀ l ∈ layers, SortedKeys l.rows) :
    SortedKeys (scanActiveChain layers) ∧
    (∀ q, listLookup (scanActiveChain layers) q = resolveActiveChain layers q) := by
  have hact : ∀ l ∈ activeChain layers, SortedKeys l.rows :=
    fun l hl => h l (activeChain_subset layers l hl)
  exact ⟨liveRows_sortedAbove (mergeLayers_sorted hact),
         resolveLayers_eq_live_merge hact⟩

end PersistentNaming.CowMapDeltaSeal
