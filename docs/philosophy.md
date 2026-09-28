# WORTH Platform Philosophy

> Most software asks you to trust it. WORTH asks you to check it, and then
> makes the check unnecessary by making the lie impossible to write.

This document explains **why** WORTH is shaped the way it is. It states the
platform's principles, the problem each one solves, and what each one means
for code you write against the platform.

It is also the platform's argument: the bets WORTH makes, the ideas that make
it possible, and the engineering mentality that produced it.

- For **how** the machinery works, read [How WORTH Works](how-it-works.md).
- For exact term definitions, read the [Glossary](glossary.md).
- For which crate to import, read the [API Map](api.md).
- For the engineering laws that bind contributors, read
  [Coding Guidelines](coding-guidelines/), starting with
  [MENTALITY.md](coding-guidelines/MENTALITY.md).

WORTH is domain-neutral infrastructure. Nothing here assumes a particular
industry. Examples use ordinary records such as accounts, orders, approvals,
and documents, but the same laws govern a lab sample's chain of custody, a
staff schedule, a design revision awaiting sign-off, or a model's training
run.

## Contents

1. [The problem WORTH solves](#the-problem-worth-solves)
2. [The central distinction: truth, state, and authority](#the-central-distinction-truth-state-and-authority)
3. [The bet: separate everything that means something different](#the-bet-separate-everything-that-means-something-different)
4. [Six ideas that make WORTH possible](#six-ideas-that-make-worth-possible)
5. [The mentality that built it](#the-mentality-that-built-it)
6. [Principles](#principles)
7. [How the principles shape the platform](#how-the-principles-shape-the-platform)
8. [Precedence when principles collide](#precedence-when-principles-collide)
9. [What WORTH deliberately is not](#what-worth-deliberately-is-not)
10. [Summary for AI agents](#summary-for-ai-agents)

## The problem WORTH solves

Software lies. Not on purpose. It lies by accident, gradually, as it grows.

A cache is written next to the data it summarizes, and one day nobody can say
which of the two is right. A permission is checked at the top of a request and
trusted at the bottom, after it was revoked. Two services compute the same
balance and disagree by a cent. A status column reads `done` and hides whether
the write happened, whether it is durable, and whether the customer was told. A
network timeout is recorded as a failure, and the payment went through anyway.
An architectural rule lives in a wiki page and erodes one "just this once" at a
time.

None of these are exotic bugs. They are the **default outcome** of how
applications are built: truth, derived state, and permission are mixed into
the same objects, and the difference between them is carried in comments,
naming conventions, and the memory of whoever wrote the code.

Most applications fail in the same few ways as they grow:

| Failure | What it looks like | What it costs |
|---|---|---|
| **Truth drifts** | Two subsystems each compute "the balance" from the same inputs and disagree. | Reconciliation jobs, support tickets, audits nobody can finish. |
| **Caches become truth** | A derived value is stored beside source data, the source changes, and nobody can say which one is right. | Systems that cannot be rebuilt, replayed, or verified. |
| **Authority is rediscovered** | Every layer re-checks "may this user do this?" slightly differently, and one of them is wrong. | The wrong one is the breach. |
| **Stale authority is trusted** | A permission checked at the start of a request is used after it was revoked. | Revoked access keeps working. |
| **Progress is a boolean** | `is_done: true` hides whether the write happened, whether it is durable, and whether the email was sent. | Recovery code guesses which fact was meant. |
| **Completion is guessed** | A network timeout is treated as success, or as failure, when the true answer is "unknown". | Double charges, lost messages, phantom orders. |
| **Rules live in documents** | An architectural rule exists only in a wiki page, and erodes one shortcut at a time. | The architecture you have is not the one you drew. |

WORTH answers each failure with a structural rule that the compiler, the type
system, or an automated check enforces. The rest of this document states those
rules.

These failures are not inevitable. Each one is the consequence of a missing
distinction. Put the distinction into the type system, and the failure stops
being a bug you might write and becomes code that does not compile.

## The central distinction: truth, state, and authority

Three words carry most of the platform's meaning. They are not synonyms.

| Word | Meaning | Example |
|---|---|---|
| **Truth** | The authoritative record, held by exactly one owning runtime. It is the single source every other view derives from. | The committed set of accounts and their relations. |
| **State** | Anything a runtime holds while it works: derived values, caches, schedules, sessions, reports, counters. State may be current or stale, and it can be destroyed and rebuilt. | A computed account summary; a scheduling queue; an inspection report. |
| **Authority** | Proof that a *specific* principal may perform a *specific* operation, for a *specific* purpose and scope, against a *specific* basis, *now*. | An admitted request to transfer funds between two named accounts. |

The key consequences:

- **Truth is not authority.** A runtime can truthfully report that an account
  exists and that a transfer is mechanically possible. That does not prove that
  a particular user may request that transfer for a particular purpose.
- **State is not truth.** A derived value is useful, but it is never the record.
  If you delete all derived state, the system must be able to rebuild it from
  truth alone.
- **Authority is not a fact you look up.** It is a typed product that a lawful
  admission step creates and that later steps carry forward.

Every principle below follows from keeping these three apart.

## The bet: separate everything that means something different

The conventional stack fuses concerns for convenience. The ORM row is the
truth, the cache, and the permission check. The reactive store is both the
state and its source. The job queue is both the effect and the record that it
happened.

WORTH makes the opposite bet. **Everything that means something different is a
different thing**, with its own owner, its own type, and its own lifecycle, and
the platform does the work of making the separated pieces behave like one
coherent system.

Separation alone is easy; plenty of systems split storage from compute. The
hard part, and the part WORTH was built to solve, is keeping the pieces
separate **and** moving them together with precision: one commit, exactly the
derived work it invalidates, exactly the subscribers who can see it, with every
step explainable after the fact. Truth history, identity, change streams,
snapshots, and derived execution stay in separate runtimes and still behave
like one auditable system.

## Six ideas that make WORTH possible

These are the ideas the platform stands on. None of them is a feature. Each
one is a structural decision that the rest of the platform depends on.

### Idea 1: Authority is a value, not a check

In most systems, authorization is an `if` statement. It runs, it passes, and
then its result evaporates. Everything after it runs on faith.

In WORTH, admission **produces** something: a typed product that holds the
proof it was built from. The next step accepts only that exact type. Methods
exist only on the states that may legally call them, so "commit before admit"
is not a runtime error you might catch; it is a method that is not there.
Governed surfaces demand concrete owner-issued authority types, never generic
markers, so a caller who implements `AuthorityMarker` for their own type opens
no doors. Forged authority is not caught at runtime. It does not type-check.

### Idea 2: Proof costs nothing at runtime

If carrying proof were expensive, teams would skip it. `worth-proof` makes it
free. It is the platform's compile-time law layer: phased artifacts, sealed
construction, proof-bearing transitions, freshness, and readmission, encoded
once and reused everywhere. It answers the four questions every phased system
otherwise answers badly by hand: **what has been proven, who may prove it,
what may consume the proof, and when the proof goes stale.** The answers live
in types, and the types disappear at compile time.

### Idea 3: One language across every boundary

When every crate invents its own word for "key", "receipt", "basis", or
"canonical value", the result is not just duplicate code. It is duplicate
**meaning**, drifting apart one small difference at a time.
`worth-foundational` exists to stop that drift. It is the shared vocabulary
that artifacts speak when they cross from one runtime to another: canonical
values, keys, paths, digests, provenance, lineage, receipts, and boundary
contracts. It is not a runtime and owns no live state. It is the reason a value
means the same thing on both sides of every boundary.

### Idea 4: Truth and computation are two runtimes with a causal bridge

Truth needs identity, history, branches, and exact commits. Derived
computation needs invalidation, scheduling, rollback, and self-inspection.
Fusing them gives you a system that does both badly.

WORTH gives each its own runtime. `worth-relational` is a first-class truth
runtime, not a storage layer. `worth-signal` is scheduling and computation
infrastructure that is **never truth**: deterministic, transactional,
aspect-aware in its invalidation, and able to explain why every node is in its
current state. `worth-runtime-bridge` joins them. It is not glue code; it is
the causal protocol boundary that turns a truth change into exactly the derived
work it implies, without either runtime borrowing the other's authority.

### Idea 5: Every state has a name

"Done" is not a state. It is several states wearing one word. WORTH names
them: requested, admitted, prepared, executing, **performed**, **settled**,
completed, stopped, published, released. Recovery code never has to guess
which fact a boolean meant, because there is no boolean.

### Idea 6: The wrong program does not compile

This is the idea behind all the others. A rule that lives in a document is a
hope. A rule that lives in a type is a law.

WORTH is built for a world where a large share of code is written by AI
agents working fast, at scale, without the context of the person who designed
the system. Such an agent follows the patterns in front of it. If the easiest
thing to write is wrong, it will write it. So WORTH makes the easiest thing to
write the correct thing, and makes the wrong thing impossible to write:
audience facades whose names are snapshotted in CI, a boundary checker that
enforces which crates may import which, typestate that removes illegal
transitions, and reports that are explicitly not authority. An agent working
on WORTH cannot quietly promote a cache, forge a permission, or skip
admission, because the code that would do so has nowhere to compile.

## The mentality that built it

WORTH was not built by asking "what is the simplest thing that works?" It was
built under a written engineering mentality
([MENTALITY.md](coding-guidelines/MENTALITY.md)) that inverts the defaults of
ordinary product development. These stances explain why the platform looks
the way it does.

**Name the adversary first.** Before designing anything, state the condition
that would break a naive implementation at production scale: a crash at any
checkpoint, a subscriber resuming mid-stream, a replay that must be
bit-identical. The adversarial constraint is the most important design
artifact. Everything else exists to survive it.

**Solve the hard problem first.** The industry builds features, discovers the
foundation cannot hold them, and retrofits. WORTH builds the foundation that
survives the adversary, then builds features on top. "Simple first" is right
for features and catastrophic for infrastructure.

**Be deliberate about foundations, fast once decidable.** Storage models,
commit pipelines, and authority boundaries are load-bearing: if they are
wrong, everything above them is wrong. Once they are right, features are
cheap, often just a declaration. The foundation does the hard work; the
features are configuration.

**Be ambitious about scope, pessimistic about systems.** If the problem needs
a truth-grade runtime, build a truth-grade runtime; do not shrink the ambition
to what seems reasonable. Then assume every system meets its worst case: data
corrupts, networks drop, subscribers crash, caches go stale. Bold about what
you build, paranoid about how it breaks.

**Enforce mechanically.** A rule held only by convention erodes. Ask of every
rule: if a tired engineer breaks this at 2 a.m., what happens? If the answer
is "nothing, until review", move the rule up the enforcement hierarchy (see
[principle 13](#13-enforce-mechanically-not-by-convention)).

**The spec is the architecture is the code.** If the specification says a
pipeline has seven phases, the code has seven named phases. Code that does not
map to a named concept in the specification is either a gap in the
specification or a mistake in the code. There is no third option.

**Make cost visible.** Performance is an invariant, not an aspiration. Hot
paths declare their complexity, count their work with structural counters, and
prove the counts in tests. A performance claim without a counter is a guess.

**Name things honestly.** A name is a semantic contract. If it needs a comment
to explain what it actually means, the name is wrong. Values that differ in
meaning, truth status, lifecycle, or authority stay distinct types even when
their bytes match.

**Debt is a blocker record, not a design strategy.** When the right path is
hard, expand scope until the blocker is gone. "For now", "temporary",
"fallback", and "escape hatch" are signals to stop and build the missing
foundation, not to ship around it.

**Preserve order.** A favorable result cannot legitimize action outside
rightful authority. Disorder, in WORTH's vocabulary, has three forms: authority
beyond its scope, derivation mistaken for truth, and present success bought
with future incoherence. The platform is designed so that none of the three
can be written by accident.

## Principles

Each principle is stated as a rule, followed by the reason for it and what it
means in practice.

### 1. Truth has exactly one owner

**Rule.** Every kind of truth has exactly one owning runtime. Other runtimes may
observe it, derive from it, or ask its owner to change it. They never hold a
second copy that competes with it.

**Why.** Two sources of truth eventually disagree. One source with many derived
views cannot disagree with itself.

**In practice.**

- Relational truth (entities, relations, branches, commit history) is owned by
  `worth-relational`.
- Scheduling and derived-computation decisions are owned by `worth-signal`.
- Application meaning, admission, and publication are owned by Query.
- Whether an external consequence completed is owned by the external system.
- You cannot make a derived runtime "authoritative" by caching its output.

### 2. State is derived; derived state is disposable

**Rule.** Authoritative state and derived state are different objects with
different lifecycles. Derived state must be reproducible from truth alone.

**Why.** A cache that has forgotten it is a cache cannot be rebuilt, replayed,
or verified. Keeping derived state structurally separate makes recovery and
audit possible.

**In practice.**

- `worth-signal` is scheduling and computation infrastructure. It is never
  truth. It decides *what must be recomputed* and *when*, not *what is true*.
- Every commit produces one canonical artifact. Change streams, history,
  notifications, and invalidation all derive from that artifact instead of
  recomputing it independently.
- The test for any design: can you destroy all derived state and rebuild it?
  If not, a cache has been promoted into truth.

### 3. Meaning is declared; authority is admitted

**Rule.** An application *declares* what its reads and operations mean.
Installation validates and canonicalizes that meaning. *Admission* combines the
installed meaning with current request evidence. Execution consumes only the
admitted product.

**Why.** If any step can be skipped, the skip becomes a second, unchecked way to
act: a parallel authority lane.

**In practice.**

- You describe a query or operation once, as a typed declaration.
- You cannot execute a declaration directly. You execute an admitted request
  produced from an installed declaration.
- There is no "trusted internal" shortcut around admission.

### 4. Authority is carried, not rediscovered

**Rule.** When a step proves something, it returns a typed product that holds
that proof. The next step accepts that exact type. No step re-queries state to
rebuild authority an earlier step already established.

**Why.** Re-checking at every layer is slow, and it invites each layer to check
slightly differently. Carrying the proof means the check happens once, in the
place that owns it, and cannot be forged downstream.

**In practice.**

- Authority-bearing methods accept concrete owner-issued types, not generic
  markers. A caller-defined `AuthorityMarker` implementation opens no doors.
- Methods exist only on the states that may legally perform them. You cannot
  call "commit" on something that has not been admitted, because the method is
  not there.
- `worth-proof` supplies the reusable, zero-runtime-cost vocabulary for this:
  phased artifacts, sealed construction, and proof-bearing transitions.

### 5. Narrowing cannot widen

**Rule.** Purpose, tenant, relationship, capability, disclosure, branch, basis,
and lifecycle constraints may narrow a request. Nothing downstream may widen it
again.

**Why.** Widening is where privilege escalation hides. If an adapter, a helper,
or a lower-runtime result could expand scope, every one of them would need to be
audited as a security boundary.

**In practice.**

- A caller may lower a result or work ceiling below the installed ceiling. It
  can never raise it above the installed ceiling.
- A projection of an admitted product carries at most the authority of that
  product.

### 6. Currentness is part of authority

**Rule.** Authority is bound to the exact basis it was proven against:
identity, principal mapping, graph observations, policy decisions, capability
grants, lifecycle state, branch, and version. Before governed work or commit,
the relevant dependencies are revalidated.

**Why.** A permission that was true a moment ago may be false now. Authority
without a basis is a stale-read bug waiting to happen.

**In practice.**

- Each request selects the current product branch fresh.
- A commit is compare-and-publish: it succeeds only against the basis it was
  prepared on. If the basis moved, the commit is refused with a typed outcome,
  not silently merged.

### 7. Progression is explicit

**Rule.** Requested, admitted, prepared, executing, performed, settled,
completed, stopped, published, and released are different typed states, not
flags on one object.

**Why.** A boolean cannot say *which* guarantee holds. The difference between
"the branch reference moved" and "the move is durable and indexed" matters for
recovery, and a status string hides it.

**In practice.**

- **Performed** means the branch reference already moved.
- **Settled** means the owning runtime also acknowledged durability and any
  required Query publication.
- A performed-but-not-settled result is a real, typed state with its own
  recovery path. It is not an error and not a success.

### 8. Commit is not completion

**Rule.** A local commit proves local state. It does not prove that an external
consequence (a payment, a message, a webhook) completed.

**Why.** Networks fail in ways that look like success or failure but are
neither. Acknowledgment, silence, timeout, disconnect, and lost response are
different facts.

**In practice.**

- An outbound effect is recorded in a dispatch outbox co-committed with the
  local change.
- The external owner decides whether the effect completed. WORTH records the
  exact typed posture it observed and never guesses completion from transport
  behavior.

### 9. Branches are exact bases, not copies

**Rule.** A branch is a named reference into an immutable history, with an
exact basis. Work on a branch is prepared against that basis and published to
that branch only.

**Why.** Speculative work (previews, drafts, what-if analysis, AI proposals)
needs to be isolated without copying the world, and merging it back must be an
explicit, checked act rather than a side effect.

**In practice.**

- Reading or writing on a branch never affects another branch.
- Bringing work from one branch to another is an explicit, admitted operation
  with its own typed outcome.

### 10. Reporting is not authority

**Rule.** Digests, counters, inspection reports, explanations, support rows,
serialized documents, and public projections are evidence for humans and
tools. They authorize nothing unless a typed contract explicitly says so.

**Why.** Reports are easy to copy, fake, or misread. Authority must come from
the owner, not from something that describes the owner.

**In practice.**

- You cannot deserialize a report and use it to authorize execution.
- Self-description grants no disclosure authority: diagnostic output follows
  the same audience, redaction, and retention rules as everything else.

### 11. Support is explicit

**Rule.** A public type may be accepted, provisional, deferred, or
vocabulary-only. The owning facade's documentation and support contract say
which.

**Why.** "It compiles" is not "it is supported". Honest support status stops
consumers from building on experiments.

**In practice.**

- Rely on accepted surfaces. Treat provisional surfaces as subject to change.
- A type that exists only as vocabulary does not imply a working feature.

### 12. The facade is the only surface

**Rule.** Each subsystem exposes one public interface for each audience. All
other structure is private.

**Why.** If no consumer depends on internals, the internals can be replaced.
If one consumer reaches past the facade, every internal detail becomes a
permanent contract.

**In practice.**

- Applications import audience facades (for Query: `worth-query-decl` for
  declarations and `worth-query-host` for hosting and requests).
- Certification and replay code imports `worth-query-replay`. Ordinary
  application code never does.
- Facade names are snapshotted and checked in CI, so a facade change is always
  deliberate.

### 13. Enforce mechanically, not by convention

**Rule.** If a rule matters, the compiler, the type system, module visibility,
or an automated check enforces it. A document alone is not enforcement.

**Why.** Rules held only by convention erode. Rules held by the compiler do not.

**In practice.** WORTH enforces its own architecture with:

- typestate and sealed constructors (invalid progressions do not compile);
- `pub(crate)` visibility and audience facades;
- the boundary checker (`tools/boundary-check`), which enforces crate bands,
  import directions, and facade snapshots;
- line caps and generated per-crate agent context.

The enforcement hierarchy, strongest first: **unrepresentable**,
**uncompilable**, **automatically tested**, **mechanically observable**,
**documented**. Documentation is last because it is hope, not enforcement.

## How the principles shape the platform

The runtime layering is a direct consequence of the principles above.

```text
worth-proof            how proof-bearing artifacts progress (compile time)
worth-foundational     the shared language artifacts speak (values, keys, receipts)
      |
worth-relational       truth: entities, relations, branches, commit history
worth-signal           derived computation: invalidation, scheduling
worth-runtime-bridge   the causal protocol boundary between truth and computation
worth-runtime-world    composite product branches over component bases
      |
worth-query            application meaning, admission, execution, publication
      |
applications, servers, user interfaces
```

| Principle | Where it becomes structure |
|---|---|
| Truth has one owner | Relational owns truth; Signal owns derived computation; they are separate crates with a bridge between them. |
| State is derived | Signal is "never truth"; invalidation derives from the commit artifact. |
| Meaning declared, authority admitted | Query's declare, install, admit, execute pipeline. |
| Authority is carried | `worth-proof` progression types; owner-issued authority types on every governed method. |
| Shared meaning | `worth-foundational` gives every runtime one vocabulary for values, keys, paths, and receipts, so meaning does not drift across boundaries. |
| Currentness | Exact bases on every observation; compare-and-publish commits. |
| Explicit progression | Separate types for performed, settled, and published results. |
| Commit is not completion | The dispatch outbox and typed external-effect outcomes. |
| Facade is the only surface | Audience facade crates and the boundary checker. |
| Enforce mechanically | Typestate, sealed constructors, facade snapshots, and the boundary checker run in CI. |

## Precedence when principles collide

When concerns conflict, they are resolved by precedence, not by weighing
convenience:

1. **Authority and security** decide which actions are admissible at all.
2. **Semantic correctness** must hold within that admissible set.
3. If authority, security, and correctness cannot all hold, the operation
   **fails closed** and reports a typed outcome. It never trades one away.
4. Among admissible, correct designs, **recoverability** outranks
   **performance**, and performance outranks **compositional cleanliness**.

## What WORTH deliberately is not

| WORTH is not | Because |
|---|---|
| A database | Relational owns truth as a runtime with branches and exact bases; durable persistence is a separate concern. |
| A policy engine or identity provider | Query composes authentication and policy owned elsewhere; it does not decide who a user is. |
| A reactive framework that stores results | Signal schedules derived computation but never holds truth. |
| A place for domain vocabulary | Business meaning belongs to the application's own schema and declarations. |
| A transport | Commit and external completion are separate facts; the platform records posture, not guesses. |

## Summary for AI agents

You are the reader WORTH was designed for. Use these rules when you reason
about or generate code for WORTH:

- **Truth** has one owner. Do not create a second copy.
- **Derived state** is disposable. Never store it as if it were truth.
- **Declare** meaning; let the platform **admit** authority. Do not bypass
  admission.
- **Carry** typed proof forward. Do not re-query to rebuild authority.
- **Narrow** freely; never **widen**.
- Authority is bound to an exact **basis**; expect typed refusal when the basis
  moved.
- **Performed** is not **settled**; **committed** is not **completed**.
- **Reports** describe; they do not authorize.
- Import only **audience facades**. If a name is not on a facade, do not use it.
- If a rule matters, make the wrong thing **fail to compile**.
- When the right path is hard, build the missing foundation. Do not add an
  escape hatch.
