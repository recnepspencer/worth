import PersistentNaming.TimeM

-- Production Rust this model mirrors (`python3 specs/proof_mirror_check.py`):
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/cow_map.rs:119-145 sha256:dffa69ebc29f864974d3b8eaf22c9181dabbfba1883b7a812975022c5fcfac2e
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/cow_map.rs:223-230 sha256:1d248a4fcef3c47ee91c829b1094a5357665021c254cd679c6c668dc905d7d66
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/cow_map.rs:274-288 sha256:b318d21405a33395a95eb938c1f31fcd7fe8576e6df470510b616b8197bd3005
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/mod.rs:204-211 sha256:e9d95e43dc68c32f5c55e61be4a3c4d156e0430f4aec9871edc75f14cd7dbf92
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/mod.rs:255-269 sha256:3c1734572f2eda0c0bad13e8a4a67f2f3e045e369cb512c8ac3b056e4a94c074
-- proof-mirror: crates/worth-relational-lineage/src/lineage/tiered/mod.rs:301-311 sha256:b3eb33cd63de0b3b81b0db0d0c005c7bacc70599e135a52e7148ef12386379c5
-- proof-mirror: crates/worth-relational-lineage/src/lineage/index/mod.rs:152-165 sha256:c390d89294e46c18684b839a077fd90e4a47fc5a82fa80db4664b9e1c9cf1341
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/branch_state.rs:109-118 sha256:23c9e34f8ce510777bfd43a604c5a5d8b2f7617c3baae2a4bd9380b3936a4836
-- proof-mirror: crates/worth-relational-lineage/src/lineage/groups/branch_state.rs:228-260 sha256:1d60b1af082df5639b3b1dcf54ae54bfc20a01a82e352dc2ed1baff6b4771d19

/-!
# Copy-on-write fork: snapshot isolation and constant fork cost (Store M4.3f P0-B)

Models the persistent naming tables of `crates/worth-relational-lineage/src/
lineage/tiered/`: `CowMap` (`cow_map.rs`), an `Arc`-node search tree whose
`clone` copies the root handle, and `TieredMap` (`mod.rs`), a cold base under
that tree as a delta where a `None` slot is a tombstone. A branch's name
index forks its five tables (`PersistentNameBranchIndex::fork_to`,
`index/mod.rs`) and its group state forks four tables per installed scheme
(`SchemeGroups::fork`, `BranchGroupState::fork`, `groups/branch_state.rs`).

The tree is a value: a fork is the same tree under a second handle, and a
write rebuilds the nodes on its search path into a new tree, leaving the old
one (and every handle to it) unchanged. That is what `Arc` sharing with
`Arc::make_mut` path copying implements.

* `get_insert`: a lookup after a write sees the write and nothing else
  changed; `get_steps_le_height`, `insert_steps_le_height`: lookups visit and
  writes copy at most one node per tree level.
* `read_apply`: a table's reads after a write sequence are the plain map's
  (`spec`): the delta-over-base layering is exactly a map.
* `fork_reads`: a fork reads every key as its source did.
* `fork_isolation`: after a fork, each side's reads are the pre-fork reads
  under that side's own writes only, whatever the other side writes.
* `fork_steps`, `fork_index_steps_independent_of_size`: a table fork is one
  step and an index fork five, for tables of any size (`∀ large small`);
  `fork_groups_steps_independent_of_members`: a group state fork is four
  steps per scheme, whatever its groups' member counts.

Simplifications: keys and values are natural numbers; AVL rebalancing
(rotations, which keep the in-order contents and path-copy only nodes next to
the search path) is not modelled, so heights here are search-path lengths and
the `O(log N)` height is the AVL bound, not proved; a delta row removal
(`remove_blind` without a base, `remove_delta`) is not modelled: over a base,
`remove_blind` writes the tombstone modelled here; cloning a scheme's id
string and the `O(1)` base link are not counted. `Conformance.lean` checks the
real `CowMap`: a fork allocates no node and shares its source's root, and
both sides' slots after their own writes are this model's.
-/
namespace PersistentNaming.CowMapFork

/-- `CowMap`'s tree: `node left key value right`. -/
inductive Tree (α : Type) where
  | leaf
  | node (l : Tree α) (k : Nat) (v : α) (r : Tree α)

variable {α : Type}

/-- The search path's length. -/
def height : Tree α → Nat
  | .leaf => 0
  | .node l _ _ r => max (height l) (height r) + 1

/-- `get_arc_and_shared`'s descent: one step per visited node. -/
def get : Tree α → Nat → TimeM (Option α)
  | .leaf, _ => ⟨none, 0⟩
  | .node l k v r, q =>
    if q < k then ⟨(get l q).val, (get l q).steps + 1⟩
    else if k < q then ⟨(get r q).val, (get r q).steps + 1⟩
    else ⟨some v, 1⟩

/-- `insert_node`: rebuilds the nodes on the search path, one step each. -/
def insert : Tree α → Nat → α → TimeM (Tree α)
  | .leaf, k, v => ⟨.node .leaf k v .leaf, 1⟩
  | .node l k' v' r, k, v =>
    if k < k' then ⟨.node (insert l k v).val k' v' r, (insert l k v).steps + 1⟩
    else if k' < k then ⟨.node l k' v' (insert r k v).val, (insert r k v).steps + 1⟩
    else ⟨.node l k' v r, 1⟩

theorem get_insert : ∀ (t : Tree α) (k : Nat) (v : α) (q : Nat),
    (get (insert t k v).val q).val = if q = k then some v else (get t q).val
  | .leaf, k, v, q => by
    simp only [insert, get]
    by_cases h1 : q < k
    · simp only [h1, ite_true]; split <;> first | omega | rfl
    · by_cases h2 : k < q
      · simp only [h1, h2, ite_true, ite_false]; split <;> first | omega | rfl
      · simp only [h1, h2, ite_false]; split <;> first | rfl | omega
  | .node l k' v' r, k, v, q => by
    simp only [insert]
    by_cases hk1 : k < k'
    · simp only [hk1, ite_true, get]
      by_cases h1 : q < k'
      · simp only [h1, ite_true, get_insert l k v q]
      · simp only [h1, ite_false]; split <;> split <;> first | omega | rfl
    · by_cases hk2 : k' < k
      · simp only [hk1, hk2, ite_true, ite_false, get]
        by_cases h1 : q < k'
        · simp only [h1, ite_true]; split <;> first | omega | rfl
        · by_cases h2 : k' < q
          · simp only [h1, h2, ite_true, ite_false, get_insert r k v q]
          · simp only [h1, h2, ite_false]; split <;> first | omega | rfl
      · have : k = k' := by omega
        subst this
        simp only [hk1, ite_false, get]
        by_cases h1 : q < k
        · simp only [h1, ite_true]; split <;> first | omega | rfl
        · by_cases h2 : k < q
          · simp only [h1, h2, ite_true, ite_false]; split <;> first | omega | rfl
          · simp only [h1, h2, ite_false]; split <;> first | rfl | omega

theorem get_steps_le_height : ∀ (t : Tree α) (q : Nat), (get t q).steps ≤ height t
  | .leaf, _ => Nat.le_refl _
  | .node l k _ r, q => by
    have := get_steps_le_height l q
    have := get_steps_le_height r q
    simp only [get, height]
    split
    · simp only; omega
    · split
      · simp only; omega
      · simp only; omega

theorem insert_steps_le_height : ∀ (t : Tree α) (k : Nat) (v : α),
    (insert t k v).steps ≤ height t + 1
  | .leaf, _, _ => Nat.le_refl _
  | .node l k' _ r, k, v => by
    have := insert_steps_le_height l k v
    have := insert_steps_le_height r k v
    simp only [insert, height]
    split
    · simp only; omega
    · split
      · simp only; omega
      · simp only; omega

/-- `TieredMap`: a cold base and the delta, `none` a tombstone. -/
structure Table (α : Type) where
  base : Nat → Option α
  delta : Tree (Option α)

/-- `get_in`: the delta's slot when it has one, else the base row. -/
def read (t : Table α) (q : Nat) : Option α :=
  match (get t.delta q).val with
  | some slot => slot
  | none => t.base q

/-- `insert`, and `remove_blind` over a base (a tombstone). -/
inductive Op (α : Type) where
  | write (k : Nat) (v : α)
  | erase (k : Nat)

def Op.slot : Op α → Nat × Option α
  | .write k v => (k, some v)
  | .erase k => (k, none)

def apply (t : Table α) : List (Op α) → Table α
  | [] => t
  | op :: ops => apply { t with delta := (insert t.delta op.slot.1 op.slot.2).val } ops

/-- The same writes on a plain map. -/
def spec (m : Nat → Option α) : List (Op α) → Nat → Option α
  | [] => m
  | op :: ops => spec (fun q => if q = op.slot.1 then op.slot.2 else m q) ops

theorem read_apply : ∀ (ops : List (Op α)) (t : Table α) (q : Nat),
    read (apply t ops) q = spec (read t) ops q
  | [], _, _ => rfl
  | op :: ops, t, q => by
    simp only [apply, spec]
    rw [read_apply ops]
    congr 1
    funext q'
    simp only [read, get_insert]
    by_cases h : q' = op.slot.1 <;> simp [h]

/-- `TieredMap::fork`: the child's base link and the same delta root, one
step whatever the table holds. -/
def fork (t : Table α) (base : Nat → Option α) : TimeM (Table α) :=
  TimeM.tick { base := base, delta := t.delta }

theorem fork_steps (t : Table α) (base : Nat → Option α) : (fork t base).steps = 1 := rfl

/-- A fork whose link reads its source's base rows reads every key as its
source did. -/
theorem fork_reads (t : Table α) (base : Nat → Option α) (hb : ∀ q, base q = t.base q) :
    read (fork t base).val = read t := by
  funext q
  simp only [fork, TimeM.tick_val, read, hb]

/-- Snapshot isolation: after a fork, each side reads the pre-fork table
under its own writes only. -/
theorem fork_isolation (t : Table α) (base : Nat → Option α) (hb : ∀ q, base q = t.base q)
    (childOps parentOps : List (Op α)) :
    read (apply (fork t base).val childOps) = spec (read t) childOps ∧
      read (apply t parentOps) = spec (read t) parentOps := by
  constructor
  · funext q; rw [read_apply, fork_reads t base hb]
  · funext q; rw [read_apply]

/-- A name index's five tables. -/
structure Index where
  names : Table Nat
  lineageNames : Table Nat
  lineageHolders : Table Nat
  holderLineages : Table Nat
  memberGroups : Table Nat

/-- `fork_to`: each table forked over the child's link (`link t` is the
rows table `t` reads through it). -/
def forkIndex (x : Index) (link : Table Nat → Nat → Option Nat) : TimeM Index :=
  TimeM.bind (fork x.names (link x.names)) fun names =>
  TimeM.bind (fork x.lineageNames (link x.lineageNames)) fun lineageNames =>
  TimeM.bind (fork x.lineageHolders (link x.lineageHolders)) fun lineageHolders =>
  TimeM.bind (fork x.holderLineages (link x.holderLineages)) fun holderLineages =>
  TimeM.bind (fork x.memberGroups (link x.memberGroups)) fun memberGroups =>
  ⟨{ names, lineageNames, lineageHolders, holderLineages, memberGroups }, 0⟩

/-- An index fork is five steps, for indexes of any size. -/
theorem fork_index_steps_independent_of_size (large small : Index)
    (link link' : Table Nat → Nat → Option Nat) :
    (forkIndex large link).steps = (forkIndex small link').steps ∧
      (forkIndex large link).steps = 5 := by
  simp [forkIndex, fork]

/-- One scheme's four group tables. -/
structure SchemeTables where
  live : Table Nat
  members : Table Nat
  retired : Table Nat
  memberOwner : Table Nat

def forkScheme (s : SchemeTables) (link : Table Nat → Nat → Option Nat) : TimeM SchemeTables :=
  TimeM.bind (fork s.live (link s.live)) fun live =>
  TimeM.bind (fork s.members (link s.members)) fun members =>
  TimeM.bind (fork s.retired (link s.retired)) fun retired =>
  TimeM.bind (fork s.memberOwner (link s.memberOwner)) fun memberOwner =>
  ⟨{ live, members, retired, memberOwner }, 0⟩

/-- `BranchGroupState::fork`: every scheme's tables. -/
def forkGroups (link : Table Nat → Nat → Option Nat) : List SchemeTables → TimeM (List SchemeTables)
  | [] => ⟨[], 0⟩
  | s :: ss =>
    TimeM.bind (forkScheme s link) fun s' =>
    TimeM.bind (forkGroups link ss) fun ss' => ⟨s' :: ss', 0⟩

theorem fork_groups_steps (link : Table Nat → Nat → Option Nat) :
    ∀ ss : List SchemeTables, (forkGroups link ss).steps = 4 * ss.length
  | [] => rfl
  | s :: ss => by
    have := fork_groups_steps link ss
    simp [forkGroups, forkScheme, fork, this]
    omega

/-- A group state fork is four steps per scheme, whatever its groups hold. -/
theorem fork_groups_steps_independent_of_members (large small : List SchemeTables)
    (link link' : Table Nat → Nat → Option Nat) (h : large.length = small.length) :
    (forkGroups link large).steps = (forkGroups link' small).steps := by
  rw [fork_groups_steps, fork_groups_steps, h]

end PersistentNaming.CowMapFork
