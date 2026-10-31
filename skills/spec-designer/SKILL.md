---
name: spec-designer
description: Design or revise phase-based WORTH specifications through focused design passes. Read relevant laws when doing the corresponding work, then give each phase target APIs and DX, appropriate authority/proof transitions, and a proportionate acceptance matrix.
---

# Spec Designer

Treat specification design as multi-pass refinement, using diffusion as an
analogy: each pass increases resolution. Begin with the overall behavior,
boundaries and phase dependencies. Each pass reads the relevant guidance and
sharpens one dimension: APIs, authority and proof transitions, operational
behavior, then tests. Preserve settled decisions, but revise them when later
passes expose a conflict. Increase detail where uncertainty or consequences
warrant it; remove unnecessary machinery and repetition. Greater resolution
means greater precision. A successful pass can simplify the design.

Write the governing design as complete, implementable phases. Each phase must
settle its architecture, target interfaces, behavior and acceptance evidence.
A paragraph describing a phase followed by a long global design essay is not
a phase-based specification.

Do not implement production code unless requested. A spec defines the target;
it must not claim that proposed APIs exist or that unrun tests have passed.

## Work in focused design passes

Do the specification work in passes, completing each kind of design across the
milestone phases before moving to the next. Read the guidance for that pass
immediately before using it, then write or revise the corresponding phase
sections. Do not front-load the entire law library and try to design everything
from memory at once. Reuse guidance already understood in the current task.

1. **Ground and divide the work.** Read repository instructions, the roadmap,
   existing spec and relevant adjacent contracts. Inspect the real production
   path and existing tests. Establish scope, phase outcomes and dependencies.
   Current code establishes the starting point, not the destination.
2. **Design contracts and DX.** Read the architectural and DX laws relevant to
   the work. Write each phase's target API, caller journey, authoritative owners,
   legal transitions and operational behavior. Resolve the important authority
   and effect decisions before choosing test machinery.
3. **Place responsibilities, when placement changes.** Read the composition and
   domain-structure laws for the affected boundaries. Assign concrete owner
   modules, cutovers and meaningful new boundaries. A phase extending a suitable
   existing owner may need only a short placement statement.
4. **Design the tests together.** Read the testing laws and relevant court/lane
   guidance now. Write the concrete test cases and acceptance matrices for all
   phases against the settled contracts. This is test design, not test-code
   implementation unless requested. Reuse fixtures and distinguish the evidence
   contributed by different phases; correct earlier design decisions if this
   pass exposes a gap.
5. **Resolve material cost or specialist risks.** Read performance, CAD or other
   domain guidance when the actual claims need it. Specify the affected cost,
   numerical or topology contracts and add their evidence to the owning phase
   matrices. Do not invent a benchmark or specialist analysis for every phase.
6. **Close the design.** Reconcile dependencies, documentation, deletion and
   integration gates. Review phase completeness and update the roadmap. Revisit
   an earlier pass only where a new finding changes its decisions.

Use that order by default, but move a pass earlier when its rules determine the
design: numerical validity or a decisive cost constraint may need resolution
before API design. This ordering is an aid to focused thought, not a reason to
design against an unknown prerequisite.

Complete the applicable passes as one task. The finished spec is organized by
milestone phase, not by these drafting passes. Keep working decisions in the
spec or current context; this workflow needs no separate design journal.

## Allocate depth by consequence

Use judgment about which parts deserve thought and space. Spend detail where
implementation could plausibly get a consequential decision wrong: authority,
effects, identity, numerical validity, recovery, lifecycle, compatibility or a
real cost claim. Keep routine wiring and established contract reuse brief.
Do not give every phase or subsection equal treatment merely for symmetry.

The phase structure is a completeness aid, not a demand for a new mechanism in
every box. Carry forward adequate designs and evidence. Combine or omit
inapplicable subsections instead of filling them with boilerplate, repeated
law citations or justifications for imaginary risks. If uncertainty could
change the architecture or acceptance decision, investigate it; otherwise
leave product-equivalent private mechanics to implementation planning.

## Ground the phases

Identify the central consumer or owner journey, its real input and observable
result, the guarantees it must preserve, and the plausible defective
implementation the spec must expose. Include necessary foundations across
owners; exclude unrelated work with an independent purpose.

Derive phases from behavior and authority dependencies. Split where a meaningful
capability, proof, effect boundary, recovery guarantee or cutover becomes usable
and testable. Keep changes together when separating them would leave competing
authority or an unprovable intermediate state. Name the concrete later action
that a prerequisite enables. Reach a real working journey early, then expand
that journey's coverage; a collection of disconnected type declarations or
empty modules is not an MVP.

Milestone phases schedule deliverables. Runtime phases encode legal operations.
Do not invent a runtime typestate merely because a milestone phase has a number.

## Make phases the document's main structure

Keep the opening to the goal, scope, current boundary, genuinely shared
definitions and a compact overview:

| Phase | Usable result | Requires | Establishes for later phases | Parallel constraints |
|---|---|---|---|---|

Then write the full phase sections. Put architectural decisions, API examples,
failure semantics, documentation obligations and acceptance matrices in the
phase that delivers them. Shared definitions may be specified once and linked
precisely; do not repeat them or make the reader reconstruct a phase from
scattered essays. Keep any final milestone-wide integration gate short and
limited to obligations that actually cross phases.

When revising a spec, preserve accepted guarantees and phase identifiers unless
the user requests a sequencing change. Move scattered requirements into their
owning phases instead of appending another essay. Update the roadmap when
dependencies, ownership or committed successor obligations change.

## Required design within every phase

Use the following concerns to design each phase; scale their presentation to
the actual work. Applicable decisions must be settled, but separate headings,
tables and new abstractions are not mandatory for each concern. Internal and
purely local phases need interfaces and evidence appropriate to their owner.

### Outcome, scope and dependencies

State what becomes operationally true at this phase's end, what remains outside
it, and what exact predecessor contract or artifact it consumes. Describe the
first working journey through the actual production owners. Name the result
and guarantees a successor may rely on, and any reason this phase cannot run
in parallel with another.

### Target API and caller DX

Show the intended interface as an actual language code block: relevant imports,
declarations or signatures, concrete input, ordinary call sequence and typed
result handling. Identify existing versus proposed surfaces and their owning
facades. Proposed spelling must still express a settled contract; do not leave
the authority model or outcome shape for implementation to invent.

Give full caller code where a phase introduces or changes the interface or
workflow. A later phase using that unchanged contract may link the exact earlier
example and show only its new scenario; do not duplicate the whole API example
in every phase.

Show the caller-visible failure, retry, cancellation, recovery or disposal path
when it changes the workflow. State who owns the input, returned handles and
cleanup; expose expensive or external crossings honestly. Keep the common path
usable without exposing internal assembly work. For an internal phase, show
the real owner-to-owner interface and its consumer instead of fictional app DX.

An API name in prose, an ellipsis hiding the disputed admission, or a directory
tree is not a target API. A friendly wrapper around an unresolved raw boundary
does not settle DX.

### Authority, typed progression and architectural laws

Apply the actual repository laws to this phase's decisions. Cite the relevant
number and document, explain the invariant it requires, and identify the
signature, construction boundary or enforcement that makes it hold. A list of
law numbers or "follow the architecture laws" supplies no design.

For phased execution or authority-bearing work, make the progression explicit:

| Transition | Exact input/proof | Granting owner and admission | Output/proof | Next permitted consumer | Refusal and effect posture |
|---|---|---|---|---|---|

Use concrete target type and method names in the spec. Explain what each proof
establishes, who alone may mint it, which scope, generation or version it binds,
whether it is borrowed, consumed or reusable, and what it cannot authorize.
Reuse existing proofs where their meaning matches. Distinguish durable intent,
descriptive data, observations, admitted authority and committed facts.

Architecture 4 and 16 require semantic decisions to lower before execution,
cheap rejection before expensive construction, and successors to consume the
exact sealed predecessor proof. Specify how skipped or reordered calls and
forged witnesses become uncallable. Later stages accept the strongest proven
type and carry established facts rather than rebuilding them from identifiers,
booleans or strings or repeating the same check in every stage. Identify the
real trust, ownership, currentness or version change that requires readmission.
Facts genuinely unavailable earlier need an explicit plan-controlled runtime
admission, not pretend compile-time certainty.

Tie other relevant obligations to their owners: compiler-total construction and
lifecycle propagation (9), distinct semantic identity and authority (10), typed
outcomes (5), effect and recovery ownership (6, 14), declared equivalence (17),
authoritative versus derived state (18), managed disposal (19), disclosure (20),
compatibility (21) and interruption/partial-effect posture (22). Verify numbers
against the current laws. Apply only those relevant to the phase. Use ordinary
types and a concise contract for pure/local work that establishes no new
authority or phased-effect boundary; do not manufacture proof objects or an
essay explaining their absence.

### Operational behavior and cutover

Describe what the system actually does, in order: admission, reads, decisions,
construction, effects, publication and release, as applicable. Identify the
linearization or commit point and what happens when relevant failure boundaries
are crossed. State which effects may occur, what survives a stop, what a retry
may reuse and who performs recovery. A status label such as "cancelled" does
not establish rollback or disposal semantics.

Name the old path that is removed, narrowed or made insufficient, the consumers
that switch, and the closure point at which competing authority must be gone.
Specify compatibility only for the product's real support commitment. Keep
ordinary, diagnostic, reconstruction and migration costs distinct. Derive cost
obligations from the actual behavior and performance laws; do not introduce
mandatory counters, budgets or pre-accounting without a demonstrated need.

### Ownership and meaningful placement

Name the existing owner modules and facades being extended, and the specific
responsibility of any new, moved or removed module. Explain a new boundary by
its meaning, authority, truth source, lifecycle, failure behavior or replacement
fate. Locate effect boundaries and dependency direction concretely.

Use a small directory tree only when physical topology itself needs a design
decision. Every proposed node must have a phase-owned responsibility and real
content. Do not require a tree for every spec, preallocate speculative files,
or substitute a performative skeleton for interfaces and behavior. A one-file
directory is valid when an actual semantic boundary or committed successor
justifies it; file count alone argues for neither flattening nor splitting.
Identify where a committed successor enters without reopening today's ownership.

### Phase test matrix

Include a concrete matrix in every phase. Each row binds a phase requirement
or relevant law to a distinguishing case and a credible expected observation.
Keep simple phases' matrices small; spend rows on distinct failure mechanisms.

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|

Populate the matrix with actual scenarios, owners and outcomes; "unit tests,"
"handle edge cases," "verify recovery" and "test under load" are not rows.
Select the possible failures from this phase's real failure model, including
interactions that can invalidate the claim. Use relevant lawful positive twins,
wrong/foreign/stale authority, invalid progression, empty or absent data,
interruption, partial publication, duplicate delivery/retry, replacement/reuse,
recovery and compatibility boundaries. Do not demand this entire list in every
phase or enumerate a Cartesian product without a distinct defect to expose.

When order matters, name the exact stop or injection point and observe both
permitted progress and forbidden effects. A denial alone is insufficient when
fixture failure or an earlier unrelated gate could produce the same result.
Specify how a lawful counterpart reaches the boundary under dispute.

Use real issuance and composition when identity, authority, persistence or
validity is material. Use the simplest credible way to observe the result.
A literal expected value, a typed denial, a public readback or an existing
trusted reference can be sufficient. Do not require a separately implemented
oracle, alternate algorithm, extra authority model or new fixture framework
merely to call the test independent.

Require a stronger oracle only when the actual defect could contaminate both
the result and its check. For example, generating expected canonical bytes with
the serializer under test cannot protect the wire format; a fixed contractual
example may suffice without building another serializer. A round trip proves
inverse consistency, not automatically external compatibility. Choose the
observation that distinguishes the claim actually being made.

Name the plausible bypass, stale reuse, wrong routing or weakened check the case
would expose when that risk is material. Negative twins, fault injection and
mutation probes are tools for resolving real ambiguity, not requirements for
every row. Do not build recursive certification machinery.

Use the narrowest honest test boundary. Public capability claims need decisive
public entry-point evidence; private owner tests supplement that evidence.
Valuable public compile-time guarantees need a valid counterpart and a denial
at the intended boundary. Private struct layout does not justify a compiler
court. Reuse existing court targets, fixtures and lanes; do not create one
binary or a new harness per milestone phase.

For a performance claim, add scale-sensitive rows for the independent input
axes, workload and warm/cold lane. State the measured operation and growth class
or comparative equality, not a generous timeout or incidental exact work count.
Keep setup, ordinary work, retained state, peak memory and reconstruction
observations separate. Introduce an observer only when necessary for the claim.

### Completion gate and documentation

Make phase completion require its matrix, the operational journey, integrated
consumers, required cutover/deletions and applicable repository gates. Name the
existing test targets and focused verification entry points; distinguish
proposed cases from tests that already exist. Missing or undiscovered cases,
stubs, empty directories and a passing unrelated suite cannot close the phase.
The first working journey is a feedback checkpoint, not permission to omit the
remaining acceptance rows.

Name the continuing audience and existing document/example that must teach any
changed API, behavior, failure handling or lifecycle. Public docs teach current
consumer behavior, not milestone history, handoffs or future work. Assign the
doc change to the phase that makes it true and require it to agree with the
qualified implementation. Keep planning/status material in the spec/roadmap;
do not create implementation journals, proof ledgers or duplicate closeout docs.

## Review before finishing

Read each phase with only its explicitly named prerequisites. Could an
implementer identify the target calls, granting owners, legal transitions,
effects and placement without inventing architecture? Could QA derive the
expected outcome of every matrix row and distinguish a wrong-reason green?
Could the phase be declared done with its API unimplemented, a weaker/raw path
still open, or its decisive evidence deferred to an unspecified later phase?

Correct those gaps in the owning phase. Also challenge excess machinery: does
each proposed proof type, separate oracle, fault schedule, benchmark or new
module protect a real contract worth its construction and maintenance cost?
Remove generic essays, repetition and ceremony that do not settle a phase
decision. Finish when the requested phases provide concrete design and
proportionate acceptance contracts, with only necessary shared context.
