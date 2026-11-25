import CoupledSuccession.Model

/-!
# The live run (design §2.1, §2.3, §2.7, §3.1–3.2)

`liveRun` threads the candidate as production does:

* a global, ordered segment log (one entry per appended member, in append order) with one
  mask bit per entry (`SegmentMask`);
* for each chain `c`: the head pre-pin `B = chain.head_input` (`base`), the touched set
  `w_c = touched_c` (only grows), the head position `h`, and the live segment `Σ_c` (`live`).

A successor round of chain `c` retracts `Σ_c` back to `B` (§2.4), masks the chain's earlier
entries, applies round `k` as authored and appends it as the chain's new live segment.
-/
namespace CoupledSuccession

/-- Who appended a log entry: a chain (by part index) or a foreign member (by log position). -/
inductive Owner
  | chain (c : Nat)
  | foreign (n : Nat)
  deriving DecidableEq

/-- One log entry: a member's segment of the candidate accumulators. -/
structure Seg where
  owner : Owner
  eff : Eff
  /-- The member's first position (a chain's head, or a foreign member's only entry). -/
  head : Bool
  /-- The `SegmentMask` bit: set once a later round of the same chain supersedes it. -/
  masked : Bool

/-- Per-chain succession state. -/
structure ChainSt where
  /-- Head pre-pin `B = chain.head_input`. -/
  base : Store
  /-- `touched_c`: union of every round's changed records. -/
  touched : Dom
  /-- Head position `h` in the log. -/
  head : Nat
  /-- The live segment `Σ_c` (the latest round's effect). -/
  live : Eff

/-- The live candidate. -/
structure Live where
  store : Store
  log : List Seg
  chains : Nat → Option ChainSt

def Live.init (b : Store) : Live := ⟨b, [], fun _ => none⟩

def updateChain (m : Nat → Option ChainSt) (c : Nat) (v : Option ChainSt) : Nat → Option ChainSt :=
  fun c' => if c' = c then v else m c'

/-- The unmasked segment of owner `o` (its live segment), if any. -/
def liveOf (log : List Seg) (o : Owner) : Option Seg :=
  log.find? (fun s => decide (s.owner = o) && !s.masked)

/-- Set the mask bit of every entry of owner `o`. -/
def maskOwner (o : Owner) (log : List Seg) : List Seg :=
  log.map (fun s => if s.owner = o then { s with masked := true } else s)

/-- Replace owner `o`'s entry by `x`. -/
def replaceOwner (o : Owner) (x : Seg) (s : Seg) : Seg := if s.owner = o then x else s

/-- Compaction (§2.7): drop masked entries and move each member's live segment to the
member's first (head) position. -/
def compactAll (log : List Seg) : List Seg :=
  log.filterMap (fun s => if s.head then liveOf log s.owner else none)

/-- One appended member. -/
def stepLive (l : Live) : Step → Live
  | .foreign f =>
    { store := apply f l.store
      log := l.log ++ [⟨.foreign l.log.length, f, true, false⟩]
      chains := l.chains }
  | .chain c e =>
    match l.chains c with
    | none =>
      { store := apply e l.store
        log := l.log ++ [⟨.chain c, e, true, false⟩]
        chains := updateChain l.chains c (some ⟨l.store, e.dom, l.log.length, e⟩) }
    | some cs =>
      { store := apply e (retract cs.base cs.live.dom l.store)
        log := maskOwner (.chain c) l.log ++ [⟨.chain c, e, false, false⟩]
        chains := updateChain l.chains c
          (some { cs with touched := unionDom cs.touched e.dom, live := e }) }

def runFrom : Live → List Step → Live
  | l, [] => l
  | l, s :: ss => runFrom (stepLive l s) ss

def liveRun (b : Store) (steps : List Step) : Live := runFrom (Live.init b) steps

/-! ## Guards (the runtime checks, written as predicates) -/

/-- Backward `IntervenedOutput` (§2.3 step 3, §3.1): no unmasked segment of another owner after
the chain's head intersects `w_c`. -/
def BackwardOk (l : Live) (c : Nat) (cs : ChainSt) : Prop :=
  ∀ i s, l.log[i]? = some s → cs.head < i → s.owner ≠ .chain c → s.masked = false →
    Disj s.eff.dom cs.touched

/-- Forward `IntervenedOutput`, Option A (§2.3 step 8, §3.2): no unmasked segment of another
owner after the chain's head intersects round `k`'s changed records `e.dom`. -/
def ForwardOk (l : Live) (c : Nat) (cs : ChainSt) (e : Eff) : Prop :=
  ∀ i s, l.log[i]? = some s → cs.head < i → s.owner ≠ .chain c → s.masked = false →
    Disj s.eff.dom e.dom

/-- Stale-base guard (NOT in design §2.3/§5.1; added because T4m is false without it, see
`MultiChainGap.two_guards_insufficient`). An earlier-headed chain `X` that has re-rounded since
`c`'s head (it owns an unmasked segment after `h`) may have retracted records whose image in
`c`'s head pre-pin `B` is now superseded. Round `k` of `c` must not change any record `X` has
touched, or a later retraction of `c` would restore that superseded image. -/
def StaleBaseOk (l : Live) (c : Nat) (cs : ChainSt) (e : Eff) : Prop :=
  ∀ i s X csX, l.log[i]? = some s → cs.head < i → s.owner = .chain X → X ≠ c →
    s.masked = false → l.chains X = some csX → csX.head < cs.head →
    Disj csX.touched e.dom

/-- The design's two guards (§5.1) at one appended member. -/
def StepGuard2 (l : Live) : Step → Prop
  | .foreign _ => True
  | .chain c e =>
    match l.chains c with
    | none => True
    | some cs => BackwardOk l c cs ∧ ForwardOk l c cs e

/-- All guards at one appended member. -/
def StepGuard (l : Live) : Step → Prop
  | .foreign _ => True
  | .chain c e =>
    match l.chains c with
    | none => True
    | some cs => BackwardOk l c cs ∧ ForwardOk l c cs e ∧ StaleBaseOk l c cs e

def guardsFrom (g : Live → Step → Prop) : Live → List Step → Prop
  | _, [] => True
  | l, s :: ss => g l s ∧ guardsFrom g (stepLive l s) ss

/-- Every successor round passed the runtime guards. -/
def WellGuarded (b : Store) (steps : List Step) : Prop :=
  guardsFrom StepGuard (Live.init b) steps

/-- The design's §5.1 predicate (backward + forward only). -/
def WellGuarded2 (b : Store) (steps : List Step) : Prop :=
  guardsFrom StepGuard2 (Live.init b) steps

/-! ## Reference semantics and readouts -/

/-- Fold a segment list over a starting store, in list order. -/
def foldSegments (segs : List Seg) (b : Store) : Store := foldBy Seg.eff segs b

/-- The live segments, each at its member's head position. -/
def liveSegments (l : Live) : List Seg := compactAll l.log

/-- The live segment of owner `o`. -/
def LiveSeg (o : Owner) (l : Live) : Option Eff := (liveOf l.log o).map Seg.eff

/-- `R(E)` built directly from the appended members: each chain's head round replaced by its
final round at the head's position; superseded rounds removed. -/
def refFrom : List Seg → Nat → (Nat → Bool) → List Step → List Seg
  | acc, _, _, [] => acc
  | acc, n, seen, .foreign f :: ss => refFrom (acc ++ [⟨.foreign n, f, true, false⟩]) (n + 1) seen ss
  | acc, n, seen, .chain c e :: ss =>
    if seen c then
      refFrom (acc.map (replaceOwner (.chain c) ⟨.chain c, e, false, false⟩)) (n + 1) seen ss
    else
      refFrom (acc ++ [⟨.chain c, e, true, false⟩]) (n + 1) (fun c' => decide (c' = c) || seen c') ss

def refLog (steps : List Step) : List Seg := refFrom [] 0 (fun _ => false) steps

end CoupledSuccession
