//! Verus model of the history journal's skew-binary jump pointers (TBU Phase 2,
//! `54a0be61a1`, unchanged in `6a420909dc`).
//!
//! Mirrors
//!   workspaces/worth-query/crates/worth-query-execution/src/domain_computation/
//!   primary_graph/product_operation/document_history/journal/sources.rs
//!     - `LinePosition` and `LinePosition::ROOT`            sources.rs:11-24
//!     - `HistoryJournal::child_position`                   sources.rs:29-49
//!     - `HistoryJournal::is_ancestor_or_self`              sources.rs:132-155
//!     - `HistoryJournal::position`                         sources.rs:157-160
//!   and the reserve call site recording.rs:69-83 (`line = child_position(current)`,
//!   then `nodes.insert(next, ..line..)`).
//!
//! Faithfulness:
//!   - `child_position` and `is_ancestor_or_self` are line-by-line
//!     transliterations: same reads in the same order, same u64 arithmetic,
//!     same branch structure, same `expect`s (vstd's `Option::expect` spec
//!     requires `is_some`, so verifying them proves they never panic).
//!   - Abstraction: the journal's `BTreeMap<u64, HistoryNode>` is the trait
//!     `Nodes`, whose `line(ordinal)` returns `Some(node.line)` exactly when the
//!     ordinal is present (`position`, sources.rs:157-160). `self.access(..)`
//!     cost accounting is replaced by a ghost probe counter.
//!   - Ghost-only additions (erased): the `Ghost<u64>` new ordinal passed to
//!     `child_position`, and the ghost probe/step count returned by
//!     `is_ancestor_or_self`.
//!
//! Proved:
//!   1. `root_wf`: the journal's initial one-node tree (journal.rs:203-221) is
//!      well formed.
//!   2. `child_position` (reserve): never panics, never underflows or
//!      overflows, and inserting its result at the fresh ordinal keeps the tree
//!      well formed, which includes the skew-binary jump invariant
//!      depth(jump(v)) == J(depth(v)) and "jump(v) is v's ancestor at that depth".
//!   3. `is_ancestor_or_self`: never panics, terminates (decreasing depth), and
//!      returns exactly `exists k. parent^k(node) == ancestor` (parent-chain
//!      ancestry, with `node` present).
//!   4. Its loop takes exactly `qsteps(depth(node), depth(ancestor))`
//!      iterations of the depth-only skew-binary query, and makes at most
//!      2 * qsteps + 2 node probes. `qsteps(x, t) <= 4 * log2(x + 1) + 2` is
//!      proved unbounded in Lean (specs/lean/tree_based_undo/JumpLift.lean,
//!      `steps_log_bound`), from the same recurrence `jd` below.
use vstd::prelude::*;

verus! {

// ---------------------------------------------------------------------------
// The depth-only skew-binary jump table.
// jd(d) is the depth a node at depth d jumps to. It is built by exactly the
// rule `child_position` applies (sources.rs:37-43): for a child of a node at
// depth p, with a = jd(p) and b = jd(a), the jump goes to b if p - a == a - b,
// else to p.
// ---------------------------------------------------------------------------

pub open spec fn jtab(d: nat) -> Seq<nat>
    decreases d,
{
    if d == 0 {
        seq![0nat]
    } else {
        let s = jtab((d - 1) as nat);
        let p = (d - 1) as nat;
        let a = s[p as int];
        let b = s[a as int];
        s.push(if p - a == a - b { b } else { p })
    }
}

pub open spec fn jd(d: nat) -> nat {
    jtab(d)[d as int]
}

proof fn lemma_jtab(d: nat)
    ensures
        jtab(d).len() == d + 1,
        forall|i: int| 0 <= i <= d ==> #[trigger] jtab(d)[i] <= i,
        forall|i: int| 0 < i <= d ==> #[trigger] jtab(d)[i] < i,
        jtab(d)[0] == 0,
    decreases d,
{
    if d > 0 {
        let p = (d - 1) as nat;
        lemma_jtab(p);
        let s = jtab(p);
        let a = s[p as int];
        let b = s[a as int];
        assert(a <= p);
        assert(b <= a);
        assert(jtab(d) == s.push(if p - a == a - b { b } else { p }));
        assert forall|i: int| 0 <= i <= d implies #[trigger] jtab(d)[i] <= i by {
            if i < d {
                assert(jtab(d)[i] == s[i]);
            }
        }
        assert forall|i: int| 0 < i <= d implies #[trigger] jtab(d)[i] < i by {
            if i < d {
                assert(jtab(d)[i] == s[i]);
            }
        }
        assert(jtab(d)[0] == s[0]);
    }
}

proof fn lemma_jtab_prefix(d: nat, e: nat, i: nat)
    requires
        i <= d <= e,
    ensures
        jtab(e)[i as int] == jtab(d)[i as int],
    decreases e,
{
    if e > d {
        let p = (e - 1) as nat;
        lemma_jtab(p);
        lemma_jtab_prefix(d, p, i);
        let s = jtab(p);
        let a = s[p as int];
        let b = s[a as int];
        assert(jtab(e) == s.push(if p - a == a - b { b } else { p }));
        assert(jtab(e)[i as int] == s[i as int]);
    }
}

pub proof fn lemma_jd_bounds(d: nat)
    ensures
        jd(d) <= d,
        d > 0 ==> jd(d) < d,
        jd(0) == 0,
{
    lemma_jtab(d);
    lemma_jtab(0);
}

/// The recurrence `child_position` implements, stated on `jd`.
pub proof fn lemma_jd_rec(p: nat)
    ensures
        jd(p + 1) == (if p - jd(p) == jd(p) - jd(jd(p)) { jd(jd(p)) } else { p }),
{
    let d = p + 1;
    lemma_jtab(p);
    let s = jtab(p);
    let a = s[p as int];
    assert(a == jd(p));
    lemma_jd_bounds(p);
    lemma_jtab_prefix(a, p, a);
    let b = s[a as int];
    assert(b == jd(a));
    assert(jtab(d) == s.push(if p - a == a - b { b } else { p }));
    assert(jd(d) == jtab(d)[d as int]);
}

// ---------------------------------------------------------------------------
// The tree the journal's node map describes.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct LinePosition {
    pub parent: Option<u64>,
    pub depth: u64,
    pub jump: u64,
}

/// `LinePosition::ROOT` (sources.rs:19-23).
pub open spec fn root_spec() -> LinePosition {
    LinePosition { parent: None, depth: 0, jump: 0 }
}

pub fn root() -> (r: LinePosition)
    ensures
        r == root_spec(),
{
    LinePosition { parent: None, depth: 0, jump: 0 }
}

/// `parent^k(n)`: follow the parent link k times.
pub open spec fn up(m: Map<u64, LinePosition>, n: u64, k: nat) -> Option<u64>
    decreases k,
{
    if k == 0 {
        Some(n)
    } else {
        match up(m, n, (k - 1) as nat) {
            Some(x) => if m.contains_key(x) {
                m[x].parent
            } else {
                None
            },
            None => None,
        }
    }
}

/// Parent-chain ancestry, the meaning `is_ancestor_or_self` claims.
pub open spec fn ancestor_or_self(m: Map<u64, LinePosition>, a: u64, n: u64) -> bool {
    m.contains_key(n) && exists|k: nat| up(m, n, k) == Some(a)
}

pub open spec fn node_ok(m: Map<u64, LinePosition>, k: u64) -> bool {
    match m[k].parent {
        Some(p) => m.contains_key(p) && p < k && m[k].depth == m[p].depth + 1 && m[k].depth <= k,
        None => false,
    }
}

pub open spec fn jump_ok(m: Map<u64, LinePosition>, k: u64) -> bool {
    let j = m[k].jump;
    &&& m.contains_key(j)
    &&& m[j].depth <= m[k].depth
    &&& m[j].depth as nat == jd(m[k].depth as nat)
    &&& up(m, k, (m[k].depth - m[j].depth) as nat) == Some(j)
}

/// The journal's node map is a tree rooted at ordinal 0 whose jump pointers
/// are the skew-binary ones.
pub open spec fn wf(m: Map<u64, LinePosition>) -> bool {
    &&& m.contains_key(0)
    &&& m[0] == root_spec()
    &&& forall|k: u64| #[trigger] m.contains_key(k) && k != 0 ==> node_ok(m, k)
    &&& forall|k: u64| #[trigger] m.contains_key(k) ==> jump_ok(m, k)
}

proof fn lemma_depth_zero_is_root(m: Map<u64, LinePosition>, k: u64)
    requires
        wf(m),
        m.contains_key(k),
        m[k].depth == 0,
    ensures
        k == 0,
{
    if k != 0 {
        assert(node_ok(m, k));
    }
}

/// Walking i <= depth(k) parents from k stays in the tree, one level per step.
proof fn lemma_up_depth(m: Map<u64, LinePosition>, k: u64, i: nat)
    requires
        wf(m),
        m.contains_key(k),
        i <= m[k].depth,
    ensures
        up(m, k, i).is_some(),
        m.contains_key(up(m, k, i).unwrap()),
        m[up(m, k, i).unwrap()].depth == m[k].depth - i,
    decreases i,
{
    if i > 0 {
        lemma_up_depth(m, k, (i - 1) as nat);
        let y = up(m, k, (i - 1) as nat).unwrap();
        assert(m[y].depth >= 1);
        if y == 0 {
            assert(m[0].depth == 0);
        }
        assert(node_ok(m, y));
    }
}

/// Walking past the root leaves the tree.
proof fn lemma_up_past(m: Map<u64, LinePosition>, k: u64, i: nat)
    requires
        wf(m),
        m.contains_key(k),
        i > m[k].depth,
    ensures
        up(m, k, i).is_none(),
    decreases i,
{
    let d = m[k].depth as nat;
    if i == d + 1 {
        lemma_up_depth(m, k, d);
        let r = up(m, k, d).unwrap();
        lemma_depth_zero_is_root(m, r);
    } else {
        lemma_up_past(m, k, (i - 1) as nat);
    }
}

proof fn lemma_up_compose(m: Map<u64, LinePosition>, n: u64, i: nat, j: nat)
    ensures
        up(m, n, i + j) == (match up(m, n, i) {
            Some(x) => up(m, x, j),
            None => None,
        }),
    decreases j,
{
    if j > 0 {
        lemma_up_compose(m, n, i, (j - 1) as nat);
        assert((i + j - 1) as nat == i + (j - 1) as nat);
        match up(m, n, i) {
            Some(x) => {},
            None => {},
        }
    }
}

/// Ancestry is the unique ancestor at the target depth.
proof fn lemma_ancestor_char(m: Map<u64, LinePosition>, a: u64, n: u64)
    requires
        wf(m),
        m.contains_key(n),
    ensures
        ancestor_or_self(m, a, n) <==> (m.contains_key(a) && m[a].depth <= m[n].depth && up(
            m,
            n,
            (m[n].depth - m[a].depth) as nat,
        ) == Some(a)),
{
    if ancestor_or_self(m, a, n) {
        let k = choose|k: nat| up(m, n, k) == Some(a);
        if k > m[n].depth {
            lemma_up_past(m, n, k);
        }
        lemma_up_depth(m, n, k);
        assert((m[n].depth - m[a].depth) as nat == k);
    }
    if m.contains_key(a) && m[a].depth <= m[n].depth && up(m, n, (m[n].depth - m[a].depth) as nat)
        == Some(a) {
        assert(up(m, n, (m[n].depth - m[a].depth) as nat) == Some(a));
    }
}

/// Inserting a fresh ordinal leaves every existing parent chain unchanged.
proof fn lemma_up_insert(m: Map<u64, LinePosition>, new: u64, v: LinePosition, k: u64, i: nat)
    requires
        wf(m),
        m.contains_key(k),
        !m.contains_key(new),
    ensures
        up(m.insert(new, v), k, i) == up(m, k, i),
    decreases i,
{
    if i > 0 {
        lemma_up_insert(m, new, v, k, (i - 1) as nat);
        if i <= m[k].depth {
            lemma_up_depth(m, k, (i - 1) as nat);
        } else if i - 1 <= m[k].depth {
            lemma_up_depth(m, k, (i - 1) as nat);
        } else {
            lemma_up_past(m, k, (i - 1) as nat);
        }
    }
}

/// The journal at creation (journal.rs:203-221) is one root node.
pub proof fn root_wf()
    ensures
        wf(map![0u64 => root_spec()]),
{
    let m = map![0u64 => root_spec()];
    lemma_jd_bounds(0);
    assert(up(m, 0, 0) == Some(0u64));
    assert(jump_ok(m, 0));
}

/// The skew-binary choice `child_position` makes, from the parent's two
/// retained jumps.
pub open spec fn child_jump(m: Map<u64, LinePosition>, p: u64) -> u64 {
    let a = m[p].jump;
    let b = m[a].jump;
    if m[p].depth - m[a].depth == m[a].depth - m[b].depth {
        b
    } else {
        p
    }
}

proof fn lemma_child_new_jump(m: Map<u64, LinePosition>, new: u64, p: u64, r: LinePosition)
    requires
        wf(m),
        m.contains_key(p),
        p < new,
        !m.contains_key(new),
        r.parent == Some(p),
        r.depth == m[p].depth + 1,
        r.jump == child_jump(m, p),
    ensures
        jump_ok(m.insert(new, r), new),
{
    let m2 = m.insert(new, r);
    let a = m[p].jump;
    let b = m[a].jump;
    let jump = r.jump;
    assert(jump_ok(m, p));
    assert(jump_ok(m, a));
    lemma_jd_rec(m[p].depth as nat);
    assert(up(m2, new, 0) == Some(new));
    assert(up(m2, new, 1) == Some(p));
    lemma_up_compose(m2, new, 1, (m[p].depth - m[jump].depth) as nat);
    if jump != p {
        lemma_up_insert(m, new, r, p, (m[p].depth - m[a].depth) as nat);
        lemma_up_insert(m, new, r, a, (m[a].depth - m[b].depth) as nat);
        lemma_up_compose(m2, p, (m[p].depth - m[a].depth) as nat, (m[a].depth - m[b].depth) as nat);
        assert((m[p].depth - m[a].depth) as nat + (m[a].depth - m[b].depth) as nat == (m[p].depth
            - m[b].depth) as nat);
        assert(up(m2, p, (m[p].depth - m[b].depth) as nat) == Some(b));
    }
    assert(1 + (m[p].depth - m[jump].depth) as nat == (m2[new].depth - m2[jump].depth) as nat);
}

proof fn lemma_child_wf(m: Map<u64, LinePosition>, new: u64, p: u64, r: LinePosition)
    requires
        wf(m),
        m.contains_key(p),
        p < new,
        !m.contains_key(new),
        r.parent == Some(p),
        r.depth == m[p].depth + 1,
        r.jump == child_jump(m, p),
    ensures
        wf(m.insert(new, r)),
{
    let m2 = m.insert(new, r);
    assert(node_ok(m2, new)) by {
        if p != 0 {
            assert(node_ok(m, p));
        }
    }
    lemma_child_new_jump(m, new, p, r);
    assert forall|k: u64| #[trigger] m2.contains_key(k) && k != 0 implies node_ok(m2, k) by {
        if k != new {
            assert(node_ok(m, k));
        }
    }
    assert forall|k: u64| #[trigger] m2.contains_key(k) implies jump_ok(m2, k) by {
        if k != new {
            assert(jump_ok(m, k));
            lemma_up_insert(m, new, r, k, (m[k].depth - m[m[k].jump].depth) as nat);
        }
    }
}

// ---------------------------------------------------------------------------
// The journal's node lookup (`position`, sources.rs:157-160).
// ---------------------------------------------------------------------------

pub trait Nodes {
    spec fn nodes(&self) -> Map<u64, LinePosition>;

    /// `Some(self.nodes.get(&ordinal)?.line)`.
    fn position(&self, ordinal: u64) -> (r: Option<LinePosition>)
        ensures
            r == (if self.nodes().contains_key(ordinal) {
                Some(self.nodes()[ordinal])
            } else {
                None
            }),
    ;
}

// ---------------------------------------------------------------------------
// child_position (sources.rs:29-49), called by reserve (recording.rs:69) with
// `parent = current` before `nodes.insert(next, ..line..)`.
// ---------------------------------------------------------------------------

pub fn child_position<N: Nodes>(journal: &N, parent: u64, Ghost(new): Ghost<u64>) -> (r: LinePosition)
    requires
        wf(journal.nodes()),
        journal.nodes().contains_key(parent),
        // reserve's `next` is fresh and above every retained ordinal, current included
        parent < new,
        !journal.nodes().contains_key(new),
    ensures
        r.parent == Some(parent),
        r.depth == journal.nodes()[parent].depth + 1,
        wf(journal.nodes().insert(new, r)),
{
    let ghost m = journal.nodes();
    let above = journal.position(parent).expect("a parent and its jumps are retained nodes");
    proof {
        assert(jump_ok(m, parent));
    }
    let jumped = journal.position(above.jump).expect("a parent and its jumps are retained nodes");
    proof {
        assert(jump_ok(m, above.jump));
        if parent != 0 {
            assert(node_ok(m, parent));
        }
    }
    let jump = if above.depth - jumped.depth == jumped.depth - journal.position(jumped.jump).expect(
        "a parent and its jumps are retained nodes",
    ).depth {
        jumped.jump
    } else {
        parent
    };
    let r = LinePosition { parent: Some(parent), depth: above.depth + 1, jump };
    proof {
        lemma_jd_rec(m[parent].depth as nat);
        lemma_child_wf(m, new, parent, r);
    }
    r
}

// ---------------------------------------------------------------------------
// The depth-only query the lift performs.
// ---------------------------------------------------------------------------

pub open spec fn qnext(x: nat, t: nat) -> nat {
    if jd(x) >= t {
        jd(x)
    } else {
        (x - 1) as nat
    }
}

/// Number of lift iterations from depth x to depth t (Lean: `JumpLift.steps`).
pub open spec fn qsteps(x: nat, t: nat) -> nat
    decreases x,
{
    if x <= t {
        0
    } else {
        if qnext(x, t) < x {
            1 + qsteps(qnext(x, t), t)
        } else {
            0  // unreachable: jd(x) < x for x > 0

        }
    }
}

proof fn lemma_qnext_lt(x: nat, t: nat)
    requires
        x > t,
    ensures
        qnext(x, t) < x,
{
    lemma_jd_bounds(x);
}

/// Where one lift iteration moves from `at` (sources.rs:147-153).
pub open spec fn lift_next(m: Map<u64, LinePosition>, at: u64, t: nat) -> u64 {
    if m.contains_key(m[at].jump) && m[m[at].jump].depth >= t {
        m[at].jump
    } else {
        m[at].parent.unwrap()
    }
}

/// One lift iteration keeps `at` on node's parent chain, at or above the
/// target depth, and follows the depth-only query.
proof fn lemma_lift_step(m: Map<u64, LinePosition>, node: u64, at: u64, t: nat)
    requires
        wf(m),
        m.contains_key(node),
        m.contains_key(at),
        m[at].depth > t,
        m[at].depth <= m[node].depth,
        up(m, node, (m[node].depth - m[at].depth) as nat) == Some(at),
    ensures
        m[at].parent.is_some(),
        m.contains_key(lift_next(m, at, t)),
        m[lift_next(m, at, t)].depth >= t,
        m[lift_next(m, at, t)].depth < m[at].depth,
        up(m, node, (m[node].depth - m[lift_next(m, at, t)].depth) as nat) == Some(lift_next(m, at, t)),
        m[lift_next(m, at, t)].depth as nat == qnext(m[at].depth as nat, t),
        qsteps(m[at].depth as nat, t) == 1 + qsteps(m[lift_next(m, at, t)].depth as nat, t),
{
    assert(jump_ok(m, at));
    if at == 0 {
        assert(m[0].depth == 0);
    }
    assert(node_ok(m, at));
    lemma_jd_bounds(m[at].depth as nat);
    lemma_qnext_lt(m[at].depth as nat, t);
    let next = lift_next(m, at, t);
    if next == m[at].jump {
        lemma_up_compose(m, node, (m[node].depth - m[at].depth) as nat, (m[at].depth - m[next].depth) as nat);
        assert((m[node].depth - m[at].depth) as nat + (m[at].depth - m[next].depth) as nat == (
        m[node].depth - m[next].depth) as nat);
    } else {
        lemma_up_compose(m, node, (m[node].depth - m[at].depth) as nat, 1);
        assert(up(m, at, 0) == Some(at));
        assert(up(m, at, 1) == Some(next));
        assert((m[node].depth - m[at].depth) as nat + 1 == (m[node].depth - m[next].depth) as nat);
    }
}

// ---------------------------------------------------------------------------
// is_ancestor_or_self (sources.rs:132-155).
// ---------------------------------------------------------------------------

pub fn is_ancestor_or_self<N: Nodes>(journal: &N, ancestor: u64, node: u64) -> (r: (
    bool,
    Ghost<nat>,
))
    requires
        wf(journal.nodes()),
    ensures
        r.0 == ancestor_or_self(journal.nodes(), ancestor, node),
        // r.1 counts node probes (`self.access(Access::Node)` in `position`)
        journal.nodes().contains_key(node) && journal.nodes().contains_key(ancestor) ==> r.1@
            <= 2 * qsteps(
            journal.nodes()[node].depth as nat,
            journal.nodes()[ancestor].depth as nat,
        ) + 2,
{
    let ghost m = journal.nodes();
    let ghost mut probes: nat = 1;
    let depth = match journal.position(ancestor) {
        Some(position) => position.depth,
        None => {
            proof {
                if ancestor_or_self(m, ancestor, node) {
                    lemma_ancestor_char(m, ancestor, node);
                }
            }
            return (false, Ghost(probes));
        },
    };
    let ghost t = depth as nat;
    let ghost steps: nat = 0;
    let mut at = node;
    proof {
        if m.contains_key(node) {
            lemma_ancestor_char(m, ancestor, node);
        }
    }
    loop
        invariant
            wf(m),
            m == journal.nodes(),
            m.contains_key(ancestor),
            depth == m[ancestor].depth,
            t == depth as nat,
            !m.contains_key(at) ==> at == node,
            m.contains_key(at) ==> m.contains_key(node) && m[at].depth <= m[node].depth && up(
                m,
                node,
                (m[node].depth - m[at].depth) as nat,
            ) == Some(at),
            at != node ==> m[at].depth >= depth,
            m.contains_key(at) && m[at].depth >= depth ==> steps + qsteps(m[at].depth as nat, t)
                == qsteps(m[node].depth as nat, t),
            m.contains_key(node) && m[node].depth >= depth ==> probes <= 2 * steps + 1,
            m.contains_key(node) && m[node].depth < depth ==> probes == 1,
        decreases
                (if m.contains_key(at) {
                    m[at].depth as int + 1
                } else {
                    0
                }),
    {
        proof {
            probes = probes + 1;
        }
        let position = match journal.position(at) {
            Some(position) => position,
            None => {
                proof {
                    assert(!m.contains_key(node));
                }
                return (false, Ghost(probes));
            },
        };
        if position.depth == depth {
            proof {
                lemma_ancestor_char(m, ancestor, node);
                if at == ancestor {
                    assert(up(m, node, (m[node].depth - m[ancestor].depth) as nat) == Some(
                        ancestor,
                    ));
                }
                assert(qsteps(m[at].depth as nat, t) == 0);
            }
            return (at == ancestor, Ghost(probes));
        }
        if position.depth < depth {
            proof {
                lemma_ancestor_char(m, ancestor, node);
                assert(at == node);
            }
            return (false, Ghost(probes));
        }
        // depth(at) > depth >= 0, so `at` is not the root and has a parent.
        proof {
            lemma_lift_step(m, node, at, t);
            probes = probes + 1;
            steps = steps + 1;
        }
        let ghost old_at = at;
        at = match journal.position(position.jump) {
            Some(jumped) if jumped.depth >= depth => position.jump,
            _ => match position.parent {
                Some(parent) => parent,
                None => return (false, Ghost(probes)),
            },
        };
        proof {
            assert(at == lift_next(m, old_at, t));
        }
    }
}

// ---------------------------------------------------------------------------
// The two callers' decisions (sources.rs:99-128), over the indexes' answers.
// ---------------------------------------------------------------------------

/// `restores_descendant_of` (sources.rs:99-115) once the two index lookups
/// (`endpoints[selected]`, `sources[source]`) have produced their ordinals.
pub fn restores_descendant_of<N: Nodes>(
    journal: &N,
    basis: Option<u64>,
    step: Option<u64>,
    current: u64,
) -> (r: bool)
    requires
        wf(journal.nodes()),
    ensures
        r == (match (basis, step) {
            (Some(basis), Some(step)) => step != 0 && ancestor_or_self(journal.nodes(), step, basis)
                && ancestor_or_self(journal.nodes(), basis, current),
            _ => false,
        }),
{
    let (basis, step) = match (basis, step) {
        (Some(basis), Some(step)) => (basis, step),
        _ => return false,
    };
    step != 0 && is_ancestor_or_self(journal, step, basis).0 && is_ancestor_or_self(
        journal,
        basis,
        current,
    ).0
}

fn main() {
}

} // verus!
