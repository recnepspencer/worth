# WORTH Query And Bridge Authoritative Mutation Evidence And Causality Plan
> **Status:** Proposed cross-runtime side-quest gate
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
> **Bridge parent:** [../worth-runtime-bridge/worth_runtime_bridge_roadmap.md](../worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
> **Primary predecessors:** [aspect-api-finalization-closeout.md](./aspect-api-finalization-closeout.md), [runtime-api-public-stabilization-closeout.md](./runtime-api-public-stabilization-closeout.md), and [../worth-runtime-bridge/milestone-12.md](../worth-runtime-bridge/milestone-12.md)
> **Primary downstream pressure:** [../worth/worth-query-runtime-rewrite-plan.md](../worth/worth-query-runtime-rewrite-plan.md)
> **Primary owners:** `worth-query` and `worth-runtime-bridge`
> **Purpose:** harden the public mutation and receipt contract together with the bridge carry-forward contract so serious domains can express authoritative writes against new and existing truth without shadow identity glue, semantic target loss, dropped causality/provenance, or domain-local writeback runtimes.
>
> **Required follow-on hardening:** [runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md)
>
> **Shipped follow-on closeout:** [runtime-generic-graph-authoring-closeout.md](./runtime-generic-graph-authoring-closeout.md)
## Goal
Freeze one cross-runtime authority-evidence contract so:
- `worth-query` exposes a public mutation/receipt/inspection surface that is
  semantically strong enough for downstream domains
- `worth-runtime-bridge` carries forward the lower-runtime causality,
  provenance, naming, continuity, and writeback evidence that Query promises to
  expose

The resulting end-to-end contract must make aspect-native `insert`, `update`,
`delete`, `batch`, preview, receipt, state, and inspection surfaces preserve enough
authored and resolved meaning for:

- direct authoritative writes against existing truth
- domain-authored edit lowering
- projected naming writeback
- lineage and continuity-aware authority crossings
- first-class causality and provenance carried by the runtime rather than
  reconstructed by domains
- future cross-domain writeback and certification work

This gate is not about inventing new domain semantics inside Query. It is about
making the generic mutation substrate honest enough that domains stop needing to
rebuild target classification, identity binding, and authority evidence above
the facade.

Once this gate closes its target evidence, causality, provenance, existing-
truth binding, naming, and continuity responsibilities, the next upstream
pressure is the remaining authoring substrate: identity-preserving existing-
target relation updates, first-class graph composition, and bridge-backed
backend-verified existing-truth support on production runtimes. Those follow-on
requirements are defined in
[runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md).

## Why This Plan Exists

The public runtime API is now stable enough that downstream domains are trying
to use Query as the real authority lane rather than a read helper. That is the
right pressure, and it exposes a real gap: aspect-native writes currently preserve touched
aspect meaning, but they are still too weak around target-class evidence,
existing-truth binding, authoritative naming attachment, causality/provenance,
and continuity-aware mutation evidence.

Without this gate:

- serious domains can author a write but still need local glue to explain what
  class of thing was actually targeted
- existing-truth edits depend on domain-local identity rebinding between domain
  authority ids and Query mutation targets
- naming writeback risks becoming a second runtime because attachment/rebind
  meaning is not explicit enough in the generic receipt surface
- causality and provenance are too easy to lose or flatten into ad hoc metadata
  instead of remaining first-class runtime evidence
- lineage and continuity-sensitive domains cannot ask Query to preserve enough
  identity-transition evidence for truthful inspection and replay
- batch receipts stay too scalar-shaped for authority-heavy workflows

This is exactly the kind of substrate gap that should be solved once in `worth-query`,
not rediscovered in each downstream runtime.

Additional governing summaries, artifact-boundary rules, compile-time
enforcement policy, scenario matrix, proof obligations, and failure taxonomy
live in [runtime-authoritative-mutation-evidence-plan-appendix.md](./runtime-authoritative-mutation-evidence-plan-appendix.md).

## Adversarial Constraint

Under direct writes, ordered batches, authoritative imports, preview-local
mutation, projected naming attachment, continuity-sensitive updates, and
domain-authored writeback lowering, the same canonical authored mutation must
produce the same target-class meaning, the same target identity evidence, the
same authority-lane explanation, the same causality/provenance bundle, and the
same typed denial behavior regardless of whether the target already existed,
was created earlier in the same batch, was addressed through a projected naming
attachment, or participated in lineage continuity.

If the runtime loses any of the following and asks a domain to recover it
locally, this gate has failed:

- what class of truth was targeted
- whether that target was declared, resolved, or rebound
- which authoritative identity the mutation was anchored to
- which upstream authority, source declaration, or writeback cause produced the
  mutation
- which provenance and lineage breadcrumbs must follow the outcome forward for
  later reads, naming, and certification
- whether naming was created, rebound, orphaned, or removed
- whether continuity/lineage meaning was preserved, denied, or ambiguous
- what batch-scoped authority evidence explains the whole mutation session

## Non-Negotiable Boundary

- `worth-query` owns mutation declaration vocabulary, target evidence
  vocabulary, receipt/inspection shaping, support gating, and typed denial.
- `worth-runtime-bridge` owns cross-runtime causality transfer, replay-safe
  provenance carry-forward, writeback-family protocol meaning, and the lowered
  bridge artifact surfaces that connect truth/runtime outcomes to Query-facing
  evidence.
- lower runtimes remain authoritative for truth mutation execution, continuity
  semantics, persistent naming semantics, writeback protocol semantics, and
  lineage truth.
- `worth-relational`, `worth-signal`, and the runtime bridge must be treated as
  the source of causality/provenance truth below the Query facade. If Query
  cannot carry that meaning forward without loss, the bridge/runtime seam is
  incomplete and must be hardened before downstream domains continue.
- domains may declare target meaning, naming meaning, and continuity meaning,
  but they must hand that meaning to Query through generic public contracts
  rather than preserving shadow runtimes.
- unsupported identity-binding, naming-writeback, or continuity families must
  fail typed and early instead of degrading into "best effort" target recovery.

This is one end-to-end contract, not a Query promise stapled onto a separate
bridge promise:

- Query may not overclaim evidence the bridge cannot preserve
- the bridge may not preserve evidence in a way Query cannot expose honestly
- downstream domains should not need to know where inside the cross-runtime
  path one breadcrumb was minted in order to trust the public receipt

## Phases

### Phase 1: Freeze Target Evidence Vocabulary

Define the shared Query-facing and bridge-facing vocabulary for mutation
targets, causality, provenance, and authority evidence.

Must ship:

- distinct declared-versus-resolved target evidence in public mutation receipts
- target collection or target class evidence for insert, update, delete, and
  batch components
- explicit target-entity identity evidence where the mutation family addresses
  one concrete target
- explicit causality and provenance sections in receipts and inspection so a
  caller gets lineage/provenance for free from the runtime once the mutation
  crosses the public authority lane
- bridge-side causality/provenance bundles that the Query receipt and
  inspection surfaces can consume without reclassification or host-local repair
- one closed naming rule set for target evidence so "declared target",
  "resolved target", and "authoritative target binding" do not blur together
- inspection accessors and batch inspection sections that expose the same
  evidence without domains reaching into raw deltas

Must preserve:

- touched aspects remain part of the contract; target evidence does not replace
  fallout meaning
- target evidence is generic runtime vocabulary, not domain-specific nouns
- preview receipts and authoritative receipts use the same conceptual model

This phase is complete only when one engineer can point at one concrete public
type for target evidence and another engineer cannot accidentally mint a weaker
"just use metadata" substitute in ordinary downstream code.

### Phase 2: Make Batch And Session Evidence Honest

Treat authoritative mutation sessions as bulk authority artifacts rather than
scalar last-write shadows.

Must ship:

- batch receipts and inspection that preserve per-component and aggregate
  target evidence
- batch/session causality bundles that preserve source declaration identity,
  authority-transition identity, and aggregate provenance rather than leaving
  downstream code to summarize the session itself
- explicit counters and summaries for target-class breadth, resolved-target
  breadth, and authored-metadata breadth
- public aggregation rules for mixed insert/update/delete sessions
- authoritative import/session helpers that preserve one inspectable batch
  artifact rather than teaching domains to inspect only the final component
- bridge-side aggregation rules so multi-write carry-forward evidence does not
  fragment into per-component-only provenance that Query must restitch

Must preserve:

- batch aggregation remains a summary of canonical component receipts rather
  than a second mutation truth source
- bulk import lanes stay explicit about target breadth and fallout breadth

This phase is complete only when bulk authoritative import, ordered write batch,
and single-write mutation each produce inspection artifacts that are different
only where the semantic boundary is actually different.

### Phase 3: Existing-Truth Identity Binding

Add a generic admitted path for mutations that target already-existing
authoritative truth.

Must ship:

- a public existing-truth target binding contract that can carry authoritative
  identity without collapsing it into ad hoc string reuse
- causality and provenance rules for existing-truth binding so the runtime can
  explain why a preexisting target was selected, denied, or rebound
- typed denial for unresolved, mismatched, or unsupported existing-truth target
  bindings
- batch-safe rules for symbolic same-batch references and existing-truth
  references living in one mutation session
- bridge-carried existing-truth binding artifacts that preserve how lower
  authority resolved, denied, or rebound the target
- inspection evidence that distinguishes:
  - newly created target
  - existing authoritative target
  - same-batch symbolic target
  - denied or unresolved target binding

Must preserve:

- Query does not become the owner of truth identity semantics
- lower runtimes still decide whether a supplied authoritative identity binding
  is valid
- domains do not need to mint a parallel target-id registry above Query

Adversarial constraint for the next hardening batch:

> Two existing-truth-targeted mutations that a downstream domain would
> consider semantically different must not collapse to the same public receipt,
> inspection, or batch/session meaning merely because they touched the same
> aspect paths or produced the same final row values.
>
> Differences in authoritative binding basis, target class, target resolution,
> or typed denial outcome must remain distinguishable all the way through the
> Bridge -> Query contract.

The practical hostile cases this phase must survive are:

- one existing-targeted mutation and one same-batch symbolic-targeted mutation
  touching the same collection and aspect set
- two existing-targeted mutations with different authoritative basis digests
- mixed ordered batches where one component is admitted and another is denied
  as unresolved, mismatched, or unsupported
- receipt and inspection paths that would appear equal if they only compared
  touched aspects or final row values

Public output for this phase:

- one typed public authoring surface for existing-truth-targeted update/delete
- one typed existing-truth target-binding artifact family
- one typed denial family for binding failure and support-gate failure
- receipt evidence, inspection evidence, and batch/session aggregate evidence
  that preserve binding family, declared target, resolved target, causality,
  provenance, and binding digest
- support-matrix and closeout/support-digest participation for the new binding
  family
- compile-fail privacy tests proving callers cannot WORTH proof-bearing
  binding/evidence artifacts
- hostile certification scenarios proving the binding semantics are sensitive
  to authoritative identity changes rather than only to touched-aspect drift

DX surface for this phase:

- the ordinary downstream path must converge on materially equivalent public
  surfaces to:
  - `workspace.bind_existing_entity(...)`
  - `workspace.bind_existing_relation(...)`
  - `workspace.update_existing(binding)...`
  - `workspace.delete_existing(binding)...`
- authoring an existing-targeted mutation must require a typed binding artifact
  rather than raw `String` identity reuse, bool flags, or mutation metadata
  bags
- mixed ordered batches must let callers combine:
  - new inserts
  - same-batch symbolic references
  - existing-targeted updates/deletes
  - admitted naming or continuity neighbors
  without forcing the caller to reconstruct target meaning after execution
- receipt and inspection accessors must expose materially equivalent typed
  fields for:
  - binding family
  - binding digest
  - declared target
  - resolved target
  - causality digest
  - provenance digest
  rather than teaching callers to decode those fields from generic metadata
  maps
- denial surfaces must converge on materially equivalent closed families such
  as:
  - unsupported binding family
  - unresolved target
  - target-class mismatch
  - collection mismatch
  - authoritative-lane required

Definition of done for the next hardening batch inside this phase:

- a downstream crate can author one existing-truth-targeted update and one
  existing-truth-targeted delete entirely through the public Query facade
- both surfaces require explicit typed target binding and reject raw identity
  shortcuts
- Bridge carries the admitted binding family forward without Query
  re-synthesizing target meaning after the fact
- receipts, inspection, and batch/session aggregate evidence preserve binding
  family, binding digest, declared target, resolved target, causality, and
  provenance with the same class and caliber as the already-admitted mutation
  families
- unsupported neighboring binding families fail typed and early
- support metadata, closeout/support digests, docs, and hostile certification
  suites update mechanically from the new admitted family
- the resulting public DX is clean enough that a downstream runtime would
  choose to call it directly instead of wrapping it in local target-recovery
  glue

This phase is complete only when existing-truth-targeted update/delete
authoring, typed target binding, typed denial, receipt/inspection evidence, and
batch/session aggregate evidence all exist as one coherent public DX surface,
with no downstream crate required to rebuild target meaning after execution.

### Phase 4: Naming-Aware Authority Evidence

Make projected naming writeback and authoritative naming attachment a first-
class generic contract neighbor.

Must ship:

- a public mutation evidence family that can preserve naming attachment intent
  and outcome class without requiring domains to inspect opaque side channels
- provenance fields that let later reads and certification know which naming
  attachment or rebinding path produced the current outcome
- enough target-binding structure to say whether naming was:
  - attached to a new target
  - attached to an existing target
  - rebound from one target to another
  - removed
  - denied as ambiguous or unsupported
- inspection and receipt evidence that preserves naming attachment outcome
  alongside ordinary target evidence
- typed denial for unsupported naming-writeback families
- bridge-carried naming/writeback provenance strong enough that Query does not
  have to synthesize naming outcome meaning after the fact

Must preserve:

- Query does not invent persistent naming semantics; it carries and exposes the
  authority evidence lower runtimes and domains already own
- ordinary CRUD remains domain-neutral and does not become naming-shaped by
  default

This phase is complete only when naming attachment, rebind, removal, and denial
can each be named concretely in receipts and inspection without reading
domain-local glue code.

### Phase 5: Continuity And Lineage-Aware Authority Evidence

Make room for full-caliber continuity-sensitive mutation without requiring a
second explanation runtime above Query.

Must ship:

- a public evidence extension for continuity-sensitive mutations where the
  authoritative outcome may preserve, deny, or ambiguously classify identity
  continuity
- first-class provenance and causality evidence for continuity-sensitive
  outcomes so lineage-aware domains do not need a second runtime just to follow
  identity transition breadcrumbs
- typed distinction between ordinary target mutation and continuity-aware
  mutation evidence
- inspection bundles that can expose continuity class, denial class, and basis
  identity without forcing domains to rediscover lineage meaning from raw lower
  artifacts
- support/admission rows that keep unimplemented continuity families fail-closed
- bridge-side continuity carry-forward rules that preserve authoritative
  lineage/continuity outcomes into one Query-facing evidence model

Must preserve:

- `worth-relational` remains authoritative for lineage and continuity truth
- Query carries continuity evidence; it does not decide lineage semantics
- unsupported continuity cases deny explicitly rather than widening into plain
  updates that lose identity meaning

This phase is complete only when continuity-sensitive mutation can either carry
one explicit runtime-owned provenance chain forward or stop with a typed denial
before semantic drift occurs.

### Phase 6: Certification And Dependency Contract Closeout

Close the gate with certification-grade proof and an explicit downstream
dependency contract.

Must ship:

- a closeout doc naming what mutation evidence is safe to build on now
- named certification suites in `test-requirements.md`
- named certification suites in `../worth-runtime-bridge/test-requirements.md`
- runtime API stabilization and support-matrix tests extended for the new
  mutation evidence surfaces
- bridge replay/causality tests extended for the same end-to-end evidence story
- migration guidance for downstream domains that currently carry local
  identity-binding or naming-writeback glue

Must preserve:

- the public mutation surface stays aspect-native
- deferred temporal/async/store/durable neighbors remain deferred
- domains do not learn expert lower-level seams as the ordinary path

Additional scenario rows, must-ship lists, proof obligations, roadmap
placement, failure taxonomy, and self-check live in the appendix:
[runtime-authoritative-mutation-evidence-plan-appendix.md](./runtime-authoritative-mutation-evidence-plan-appendix.md).
