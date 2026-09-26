# Milestone 9.17.6.1: Shared Typed Expression Language

> **Status:** Planned. Cross-runtime foundation after the workflow kernel in
> [9.17.6](./milestone-9.17.6.md); required substrate for
> [Worth UI 3.17](../worth-ui/worth_ui_roadmap.md#milestone-317-dsl-expressions-conditions-and-semantic-evaluation).
> This document governs language semantics, portable evaluation, Query adoption,
> and the shared-language portion of UI 3.17. It does not declare either milestone
> implemented. Existing completed phases remain historical.

## Goal And Placement

Ship one custom, total, typed expression language whose meaning is identical in
Query, Worth UI, and engineering consumers. Source, typed Rust construction, and
decoded artifacts converge on one bounded admission path, canonical typed program,
and pure evaluator in `worth-foundational`. UI must use it; an unused interpreter
library does not close this milestone.

An expression computes a value from explicitly admitted inputs. It cannot acquire
authority, discover a runtime, query a graph, mutate truth, schedule simulation,
perform geometry, or invoke arbitrary Rust. Rust plugins remain the ordinary home
for parametric-family algorithms, numerical solvers, and substantial domain work.
The language describes authored conditions and value transformations, not an HDL,
geometry kernel, workflow engine, or replacement for compiled domain computation.

CAD and chip workloads constrain the foundation now: dimensional correctness,
explicit bit widths and unknown logic, deterministic numerics, bulk input access,
bounded nested iteration, precise dependency consumption, and incremental reuse.
No per-face, per-net, or per-element runtime-node expansion is permitted merely to
evaluate an expression. Supporting these values is not certification of a medical
device, numerical solver, chip simulator, or safety-critical application.

The milestone consumes 9.17.6's installed vocabulary, fresh transition admission,
shared candidate inputs, exact output reuse, and managed views. It does not reopen
9.17.5, replace Relational/Signal/Bridge/World authority, or delay independent inbound
effect work in 9.17.7. Its condition cutover must land before downstream consumers
claim the shared language; numbering does not retroactively block completed work.
For this successor, 9.17.6's ban on a string expression evaluator becomes a ban on
direct string execution: text enters the shared bounded parser/admission path only.
Its fresh transition authority and prohibition on arbitrary code remain unchanged.

## Current Boundary And Required Change

- `crates/worth-foundational` already owns aspect values, contracts, canonicalization,
  identity, and portable descriptive vocabulary. It imports no Query or UI runtime.
  Its float carriers admit canonical NaN and preserve signed zero; their derived
  ordering is representation ordering, not expression numeric ordering. Decimal and
  rational carriers are not already a checked arithmetic implementation.
- `worth-ui-dsl/src/source/{lexical,parse,compile}` owns UI source and provenance;
  `semantic/` owns UI-authored meaning. Its manifest already imports foundational.
  UI semantic-package identity confirms an exact basis after fingerprint narrowing.
- `worth-ui-runtime/src/runtime/planning` receives semantic meaning and retained
  Query binding facts. `runtime/rebind` owns replacement, currentness, and lifecycle.
  Neither the renderer nor inspection may become an expression evaluator.
- Query declaration's `ApplicationWorkflowConditionRef` currently identifies an
  installed boolean query. Execution's
  `application_attempt/workflow_transition_program/condition.rs` validates its exact
  output source before selecting a transition. Preserve that authority checking;
  replace condition meaning with an admitted expression over declared query results.
- Query's existing query predicate/planner machinery and legacy `program/expressions`
  value-construction surface have different scopes. Inventory actual migrated
  consumers; do not relabel every Query AST as this language or delete unrelated
  query planning to force superficial uniformity.

Foundational gains **stateless, bounded interpretation of portable meaning**, not an
operational executor. Language limits and pure work accounting are values local to
one invocation. Live registries, resource reservations, clocks, continuations,
leases, cache admission/eviction, subscriptions, and currentness remain runtime-owned.
This narrow addition does not authorize a generic runtime in a substrate crate.

## Adversarial Courtroom

### Same meaning, hostile representations

Admit equivalent source, typed-builder, and decoded drafts through the public
foundational facade. Use two input representations: dense borrowed columns and
record-oriented immutable values. Compare canonical typed meaning, exact values,
denial category/location, and semantic work accounting. Reorder catalog insertion,
rename comprehension binders, vary whitespace, use colliding narrowing fingerprints,
and decode malformed/oversized artifacts. Only semantically irrelevant changes may
preserve identity. Unsupported input must deny before oversized allocation.

Exercise checked integer extrema, decimal rounding ties, float subnormals, signed
zero, forbidden NaN/infinity, incompatible dimensions, wrong bit widths, X/Z logic,
Unicode, option absence, empty collections, and short-circuit errors. A test oracle
must not call the production operator, canonicalizer, or type checker to compute its
expectation. Change the language/function version under identical source and verify
that retained meaning cannot silently acquire the replacement semantics.

### Real UI source to live consequence

Use the existing platform-pulse executable composition and real source-edit/hot-rebind
entry. An authored expression controls presence and operability; another shapes
options/payload; a dimensional scalar controls an existing typed presentation input.
Exercise all five 3.17 evaluation uses, including participation, through production
lowering. At least one expression edit must change the running product visibly;
product-issued evaluation/rebind evidence must join the same external adjudication.

While evaluation is pending, replace the selected source/program generation and
deliver the old result afterward. It must be stale and must not alter the replacement.
Switch worlds with identical displayed values, revoke an operand binding, exhaust
evaluation and explanation budgets independently, and feed an invalid new expression.
Keep the prior valid installation or deny activation according to the existing rebind
transaction; never install a partial expression package. Inspect true, false, stale,
denied, and cancelled postures without executing the expression a second time.

Change an unrelated fact: zero expression executions and zero renderer-side semantic
decisions. Change a guard so a formerly skipped branch becomes active: acquire its
declared inputs through the owner before accepting its result. Source-span diagnostics
must identify the authored clause, including after equivalent-source plan sharing.

### Query condition and engineering consumers

Through Query's ordinary installed application entry, publish a workflow condition
using at least two typed query operands and a nontrivial authored predicate. Exercise
true, false, absent, arithmetic denial, stale source, and revoked authority. A true
expression may select a successor; it cannot authorize that successor's operation.
Race evaluation with a consumed-source change before publication: fresh owner fences
must reject the stale transition. Retrying must preserve existing effect idempotency.

Use the real proprietary House composition, pinned to the candidate public revision,
for a unit-bearing clearance rule and member-selection expression over real projected
inputs. Edit a rejected member into eligibility, insert a matching member into an
initially empty selection, remove a selected member, and change an unrelated field.
Compare selected members and resulting application output with an independently
computed Rust reference. No public-repository CAD facsimile closes this product gate.

Use a production-valid Query application contract for a digital circuit snapshot:
width-typed buses, four-state signals, clock-domain-tagged ticks, and selected fan-in.
The reference computes fixed truth tables/bit arithmetic independently. This proves
expression consumption at the Query boundary, not a nonexistent simulator product.
If a real simulator consumer exists at implementation time, use its ordinary facade
for the same contract; do not claim its scheduling/solver correctness from these tests.

### Independent scale and exhaustion axes

Qualify 1k/100k/1M supplied members or nets, separately from 100/1k/10k installed
expressions, AST size, bit width, active readers, and source history. A full filter
honestly visits its full input once. After warm installation, a one-member field edit
in a map/filter view visits only affected element dependencies plus the declared
sequence/index granule; it cannot rescan the full collection to discover sameness.
Changing unrelated population/history adds zero unrelated field reads, evaluations,
or provider contacts. Broad operations pay their declared breadth.

Run nested 1,000-by-1,000 comprehensions with a budget below their required work;
exhaustion occurs at the exact charged boundary, with no successful partial list.
Exhaust source bytes, parse depth, type work, expanded function instructions, input
bytes, output bytes, scratch, dependency capture, registry entries, cache retention,
batch count, and queue capacity separately. Grow strings and bit vectors separately
from element count. A cheap opcode wrapping an expensive scan must fail the budget
court. Denying every valid large workload does not qualify scaling.

Destroy compiled plans, memoized outputs, dependency indexes, and UI derived plans;
rebuild from admitted source/artifacts plus current owner inputs under a cold budget.
Observe the same semantic result without replaying an effect. Close sessions and
replace registries while readers remain pinned: exact old support is retained only
under bounded owner custody, and new uses require current support admission.

The courts must convict missing negative dependencies, unmetered host calls, false
cache-currentness, process-local identities, eager evaluation of skipped arms,
per-element orchestration, global invalidation, hash-only aliasing, and changed
numeric semantics disguised by regenerated goldens. Use targeted negative twins or
temporary mutations where sensitivity is otherwise unclear; retain no mutation probe.

## Language V1: Product Decision Lock

### Grammar, typing, and totality

The language is expression-only. No assignment, mutation, arbitrary loops, recursion,
user-defined functions, first-class functions, dynamic evaluation, reflection,
imports with executable side effects, async, I/O, wall-clock access, or randomness.
Comprehension binders are lexical and nonescaping; they are not closure objects.
Finite acyclic installed functions are described below. Local immutable `let`
bindings may share values but cannot recurse or shadow an operand/function name.

The normative surface includes literals, declared operand/record-field references,
parentheses, `!`, unary `-`, `* / %`, `+ -`, comparisons, same-type equality,
`&&`, `||`, `??`, and `condition ? then : else`, in that descending precedence.
Binary operators associate left except `??` and the conditional, which associate
right. Chained comparisons are rejected. Calls resolve by qualified installed name
and exact signature, never runtime string lookup. `let name = expr; body` has the
widest scope and lowest precedence. Source offsets are UTF-8 byte ranges.

Boolean operators require `Bool`; there is no truthiness. Admission checks every
branch, including unreachable constant branches, against a closed operand schema.
Both conditional arms have the same type. No implicit numeric promotion, string
coercion, heterogeneous list, structural duck typing, or dynamic `Any` is admitted.
Unsuffixed integers are Int64 and decimal/exponent literals are finite Float64;
other scalar types require explicit typed constructors. Empty collections and `none`
require an expected type or explicit type annotation. Names are case-sensitive;
identifiers use ASCII letters/digits/underscore, with qualified namespace separators.
String literals are UTF-8 with defined escapes; invalid scalars/surrogates deny.

The expression grammar below is normative; `type` and `qualified_name` resolve only
against the admitted schema/catalog. Numeric suffixes are not an additional syntax.
Record fields use their schema order even if source fields are rearranged; diagnostics
retain the original source span. Map entry order remains authored evaluation order.

```text
expr        := "let" identifier "=" expr ";" expr | conditional
conditional := coalesce ["?" expr ":" expr]
coalesce    := disjunction ["??" coalesce]
disjunction := conjunction {"||" conjunction}
conjunction := equality {"&&" equality}
equality    := relation [("==" | "!=") relation]
relation    := additive [("<" | "<=" | ">" | ">=") additive]
additive    := product {("+" | "-") product}
product     := unary {("*" | "/" | "%") unary}
unary       := ("!" | "-") unary | postfix
postfix     := primary {"." identifier | "." comprehension "(" identifier "," expr ")"}
primary     := literal | qualified_name | "(" expr ")"
             | qualified_name "(" [expr {"," expr}] ")"
             | "none" "<" type ">" | "[" [expr {"," expr}] "]"
             | "{" [expr ":" expr {"," expr ":" expr}] "}"
             | qualified_name "{" [identifier ":" expr {"," identifier ":" expr}] "}"
comprehension := "map" | "filter" | "all" | "any"
qualified_name := identifier {"::" identifier}
```

`literal` covers true/false, signed-through-unary decimal integers, hexadecimal
integers, decimal/exponent floats, and double-quoted strings. Integer minimum values
are admitted by checking the signed literal as a unit, not overflowing its positive
intermediate. Escapes are quote, backslash, n/r/t, and `\u{hex_scalar}`. Comments use
`//` to line end. Collection/record trailing commas are accepted. Constructors
`int8` through `uint64`, `float32/64`, `decimal("...")`, `bytes("hex")`,
`bits<N>("binary")`, and `logic4("01XZ")` are reserved typed intrinsics; the latter
infers width from the literal. Generic constructor type arguments are compile-time
constants, not expressions. Nonliteral casts use the explicit conversion contracts.
Reserved type constructors extend `primary` directly; ordinary installed functions
cannot shadow them. Syntax not listed here is unsupported, not implementation-defined.

Foundational owns the bounded expression lexer/parser and AST. The existing UI parser
recognizes expression-bearing clauses and delegates the exact source slice; it does
not embed a second expression grammar. Rust builders bypass tokenization, not type
admission. Decoders produce only untrusted drafts. Parse/type diagnostics retain
source provenance without making source location part of mathematical meaning.

Total means every admitted bounded invocation terminates with a value or a typed
failure, not that every arithmetic operation succeeds. Admission and decoding are
also bounded. Use explicit bounded stacks/worklists so malicious nesting cannot
overflow the native stack before a language denial. Optimized execution must preserve
the admitted evaluation order, failure semantics, and dependency contract.

### Values and operators

Reuse existing foundational scalar/identity vocabulary through checked expression
refinements; do not create a competing generic document value or JSON evaluator.
Expression collections/records are a distinct typed evaluation product, not a new
authoritative aspect-state representation. V1 supports:

| Family | Contract |
| --- | --- |
| Bool | Exactly true/false; unknown logic and stale runtime state are different types. |
| Int8/16/32/64, UInt8/16/32/64 | Exact declared width, checked arithmetic, no wraparound operators. |
| Float32/64 | Finite IEEE binary values with the strict contract below. |
| Decimal | Checked base-10 coefficient, at most 38 significant digits and scale 0..18. |
| String, Bytes | Bounded immutable values; no interned-symbol number as content identity. |
| Nominal ID, enum, record | Versioned declared schema; IDs are opaque and never convertible to authority. |
| Option<T> | Present value or absent; neither substitutes for an evaluation error. |
| List<T>, Map<K,V> | Homogeneous, finite; keys restricted to String, integer, or nominal ID with canonical ordering. |
| Quantity<D> | Finite Float64 magnitude plus type-level dimension; authored/display unit is separate. |
| Bits<N>, Logic4<N> | Explicit fixed width, 1..4096 under the installed width ceiling; no numeric/Bool coercion. |

Native BigInt, Rational, temporal, and other aspect values do not gain unspecified
operators merely because their carriers exist. Unsupported operand types deny during
admission. A simulator represents ticks with a nominal record containing an exact
UInt64 tick and declared clock-domain/resolution identity; comparing different domains
requires an explicit installed conversion contract, not a hidden float conversion.
Domain vector/point/reference-frame values use nominal records and typed installed
functions. General matrix algebra and solver behavior remain Rust operations.

Integer overflow, zero division, minimum-signed negation, and minimum-signed/-1
division deny. Integer division truncates toward zero; remainder has the dividend's
sign. Casts check range; float-to-integer additionally requires an integral value.
Integer-to-float `exact_cast` rejects precision loss; `rounded_cast` explicitly uses
nearest-even. The spelling distinguishes them. No release/debug arithmetic drift.

Decimal literals and input carriers are numerically normalized at expression
admission (strip redundant zeros; one zero), without changing existing carrier
identity globally. `+`, `-`, `*` are exact within the declared digit/scale bounds and
deny otherwise. Decimal `/` is unavailable; `decimal_div(a,b,scale,rounding)` and
`quantize(a,scale,rounding)` require explicit rounding: nearest-even, toward-zero,
toward-positive, or toward-negative. Intermediate arithmetic is checked and metered;
binary floats never mediate decimal comparison or conversion. Currency remains a
nominal domain distinction, not a plain decimal alias.

### Floating point and engineering quantities

Reject NaN and infinities at literal, operand, decode, conversion, and result
boundaries. Normalize all zero results/inputs to positive zero. These are expression
refinements over `CanonicalF32/F64`; do not silently change their existing semantics
or use their bitwise `Ord` to implement numeric comparison.

V1 specifies nearest-ties-even arithmetic, gradual underflow, and one rounding at
each declared operation. No flush-to-zero, extended-precision intermediate leakage,
implicit FMA, reassociation, platform fast-math, or parallel reduction reordering.
Float division by zero denies, including 0/0; finite underflow to zero is permitted.
Overflow/nonfinite results deny. `%` is integer-only. `sqrt` is correctly rounded and
denies a negative operand. Domain-invalid operations never manufacture absence.

The same admitted expression and inputs produce identical canonical numeric bytes
on supported x86-64, AArch64, and wasm32 targets. Use a strict reference numeric
implementation where native behavior cannot satisfy the contract. Transcendentals
such as sin/exp/log and general pow are not V1 builtins: adding one requires a pinned
deterministic algorithm, error/rounding contract, bounded cost, and new semantic
version. Calling platform libm and testing only an epsilon is insufficient.

Equality on finite floats is exact numeric equality after zero normalization.
`near(a,b,abs_tol,rel_tol)` is explicit: nonnegative same-dimension absolute tolerance,
nonnegative dimensionless relative tolerance, and the inequality
`|a-b| <= max(abs_tol, rel_tol * max(|a|,|b|))`. Evaluate this comparison using exact
values of the finite binary operands with bounded widened arithmetic, so intermediate
overflow cannot turn a comparison into a passing infinity. Tolerance belongs to the
calling domain; never substitute `near` for canonical identity or cache equivalence.

Dimensions are an exponent vector over the seven SI base dimensions plus angle;
exponents are integers in -16..16. Addition/subtraction/order require identical
dimensions; multiplication/division add/subtract exponents and deny overflow.
Dimensionless quantities remain distinct from plain floats until explicitly converted.
No implicit length/angle/point/frame conversion. Affine temperatures are excluded;
temperature differences may use ordinary temperature quantities.

Unit literals use `quantity(magnitude, unit)` with a versioned installed unit catalog.
SI prefixes and inch/foot conversions use exact rational scales; convert to the
canonical SI magnitude with one nearest-even rounding, including decimal literal
parsing. Angle uses radians; degree conversion uses the correctly rounded binary64
value of pi/180 as a versioned constant. Compound units derive dimensional exponents.
Persist canonical magnitude/dimension and retain authored unit in provenance. Equal
canonical values share meaning; do not promise every rounded unit spelling is equal.

### Digital logic

`Bits<N>` is an unsigned N-bit pattern. Bitwise AND/OR/XOR/NOT, logical shifts,
concatenation, slicing, equality, and explicit checked/wrapping bit addition are
typed builtin functions. Binary bit operations require equal widths. Logical shifts
zero-fill; counts >= N yield zero. Slices use half-open [low,high), reject invalid
bounds, and have statically known result width. Bit 0 is the least significant bit;
serialization specifies byte order and zero padding, independent of host layout.
Integer conversions and width extension/truncation are explicit and checked unless
the function is explicitly named `truncate` or `wrapping_add`.

`Logic4<N>` represents 0, 1, X, and Z per bit. Case equality compares all symbols
exactly and returns Bool. Logical equality returns a one-bit Logic4: a known mismatch
returns 0, otherwise any X/Z returns X, otherwise 1. For bitwise operations treat Z
as unknown: 0 AND anything = 0, 1 OR anything = 1, known XOR uses binary truth, other
unknown combinations yield X; NOT 0/1 flips and NOT X/Z yields X. A one-bit mux with
known select chooses its arm; unknown select preserves identical arm symbols and
produces X where arms differ. It cannot be used as the language Bool conditional.
Resolution strength, drive conflicts, delay scheduling, and four-state arithmetic
are outside V1; reject unsupported operations rather than treating X/Z as false/zero.

### Absence, errors, order, strings, and collections

Evaluation is left-to-right. `&&`, `||`, `??`, and conditional selection are lazy.
`false && error` returns false; `error && false` returns the evaluated error.
`none ?? fallback` evaluates fallback; an error on the left is not caught by `??`.
`is_some`, `some`, typed `none`, and checked `unwrap` are explicit. `unwrap(none)`
denies. A missing required operand is not an optional absent value. Unknown logic,
not-yet-produced operands, stale observations, and denied disclosure remain distinct.

String equality is exact Unicode scalar-sequence equality; order is ordinal scalar
order, case-sensitive, with no implicit normalization or locale. `length` counts
Unicode scalars for String, bytes for Bytes, elements for collections, bits for buses.
String slices use scalar offsets, return Option for invalid bounds, and charge the
visited bytes. No grapheme/locale/case-fold/regex builtin in V1. Explicit future
normalization or formatting functions pin their Unicode/locale data version.

Record construction follows schema field order, requires every required field, and
rejects duplicates/unknowns; its field expressions evaluate in that order. Lists keep
authored order. Map entries evaluate in authored order; duplicate keys deny. Canonical
map *values* sort keys; canonical *programs* preserve evaluation order. Iteration over
maps is only through explicit `entries`, in canonical key order. `get` returns Option;
no missing-key default. Recursive equality is typed, structural, and metered by actual
visited bytes/elements. No heterogeneous numeric equality.
Ordering is defined only for numeric scalars, same-dimension quantities, String, and
Bytes (unsigned lexicographic byte order). Enums, IDs, records, collections, and buses
have no implicit language ordering; canonical storage order is a separate contract.

`map(x, body)`, `filter(x, predicate)`, `all(x,predicate)`, and `any(x,predicate)`
iterate in collection order. Map/filter preserve order; all/any short-circuit and have
empty identities true/false. Predicate errors propagate. `sum` is left-fold addition
from the typed zero and obeys checked numeric semantics; min/max over an empty list
return None. No arbitrary authored fold/state machine, generated unbounded range,
sorting comparator closure, or collection mutation. Nested comprehensions consume
one cumulative budget; suspension, function calls, and batches cannot reset it.

The remaining V1 builtins are typed min/max/abs/clamp, length, get, contains,
starts_with/ends_with, explicit casts, and the quantity/decimal/bit operations named
above. Clamp rejects reversed bounds. Contains is exact and metered; no hidden regex.
Builtin semantics, type signatures, errors, and cost rules are closed versioned data.

## Installed Functions, Identity, And Admission

An immutable `ExpressionFunctionCatalog` describes qualified function identity,
semantic version, exact parameter/result types, implementation semantic digest,
declared resource formula, and determinism profile. A runtime-owned
`InstalledExpressionRegistry` binds supported catalog entries once at installation.
Expression source selects only that admitted vocabulary; it cannot install code.

V1 installed function bodies are checked expression IR over parameters and previously
admitted functions, with an acyclic call graph and a bounded call depth/expanded work
contract. They can compose the closed foundational builtins. No native function
pointer, Rust closure, FFI, runtime handle, unchecked bytecode, or caller-provided
purity flag enters the expression registry. A new native primitive is an explicit
reviewed language implementation/version change, not runtime plugin registration.
This makes purity and termination enforceable; signature declarations alone cannot
make arbitrary native callbacks safe. General Rust plugins stay outside this lane.
Two functions with equal signatures but different meaning are different identities.
Catalog entries bind only their transitive referenced functions/types/units; installing
an unrelated function must not invalidate every compiled expression. Overloads are
finite exact signatures, with no user-extensible coercion or operator dispatch.

Functions receive only their typed arguments. Reading an argument subfield remains
tracked; opaque domain functions cannot silently fetch additional model state.
Installed bodies and calls consume the same meter, including nested collections,
byte work, scratch, and outputs. Admission detects cycles and expansion bombs before
allocating flattened instructions. Shared function bodies are not repeatedly copied
into every caller; bounded call frames or equivalent DAG lowering preserve semantics.

Canonical program identity uses existing foundational canonical-basis/digest APIs.
It includes language semantic version, resolved operand schema and slot mapping,
operator/type tags, exact literal values, unit catalog meaning, function semantic
closure, branch/order structure, and result type. No direct hashing library or Debug
text identity. Binder alpha-renaming and irrelevant formatting do not change meaning;
field binding, width, unit meaning, function version, and evaluation order do.

Source spans, labels, diagnostic richness, process handles, and registry addresses
are excluded from semantic identity. Preserve a separately bound source map from
canonical instruction occurrence to each authored source clause/call expansion.
Hash/fingerprint equality is only a narrowing hint; exact basis comparison or a
previously interned owner-confirmed identity establishes reuse. Never sort supposedly
commutative operations or erase a branch before typechecking to manufacture equality.

The shipped common path is parse/build -> admit -> compile once -> evaluate many.
Admission resolves overloads, all branch types, installed function closure, maximum
access contract, resource ceilings, and source bindings. Execution cannot repeat
name lookup, schema inference, parser work, canonical serialization, or registry scans.
Decoded compiled bytes never bypass fresh validation/support admission.

| Artifact | Constructor, meaning, and limitation |
| --- | --- |
| `ExpressionDraft` | Parser/builder/decoder; untrusted data, no execution entry. |
| `AdmittedExpression` | Foundational validator; sealed structural/type facts and canonical meaning, no world authority. |
| `CompiledExpression` | Foundational lowering from admitted meaning; immutable discardable plan, no registry/currentness grant. |
| `ExpressionReadManifest` | Compiler; maximum operand paths, collection dependencies, guards and conditional reads; no read permission. |
| `ExpressionConsumption` | Evaluator; actual accessed slots/paths/membership and work; descriptive evidence, not an owner receipt. |
| `ExpressionEvaluation` | Evaluator; Value or typed Denied with exact semantic cost and source occurrence; never an operation permit. |
| Runtime-bound evaluation input/result | Query/UI owner only; exact world/program/source/support/occurrence, reservations and freshness; consumed only by that owner's existing admission. |

Authority remains concrete `worth-proof`-based owner authority. No public constructor,
serde route, generic marker trait, or copied consumption record can mint the stronger
runtime-bound phase. Pure evaluation is callable without authority over caller-owned
values; its result cannot be substituted into an authority-sensitive runtime entry.

### Compatibility and persistence

V1 artifacts carry separate wire-schema and language-semantic versions plus exact
referenced function/type/unit identities. The initial compatibility window accepts
only V1 semantic meaning; unknown major/minor features deny before lowering. A
bug fix must preserve specified V1 results, while a semantic change requires a new
version with an explicit migration. Source may be retained for editing but cannot
silently be recompiled under new meaning on open or hot rebind.

Persist canonical drafts/meaning and provenance, never runtime handles, input
permissions, resumable machine state, or executable authority. Restored artifacts
undergo bounded structural/type/support readmission. Historical functions can coexist
under exact versioned identities while live registry support, security and retention
allow them; no `latest` name resolution at evaluation. Replacement requires an owner
disposition for pinned users. Missing old support is UnsupportedInstallation, not
automatic upgrade. Migration is explicit, provenance-preserving, and leaves old
records interpretable; there is no downgrade fallback or silent reinterpretation.

## Runtime Binding, Dependency Capture, And Reuse

Owners bind the manifest to disclosed, immutable input snapshots from their existing
admitted query/projection contracts. All potential reads are declared before execution;
actual tracking narrows consumption within those bounds, never widens it. Operand
access exposes borrowed typed fields/columns and stable element identity. No arbitrary
graph traversal callback is available to the evaluator. Bulk preparation shares
overlapping inputs through 9.17.6's existing exact-basis machinery.
The foundational input facade uses sealed immutable row/column/value views. It does
not accept a caller-implemented `read(path)` trait capable of arbitrary work or I/O.
Runtime-specific preparation may gather admitted inputs, but it is an explicit,
separately budgeted owner step. Borrowing/retention must keep the exact snapshot alive
across suspension without copying it wholesale or extending disclosure authority.

Record positive, absent, negative, and membership/order dependencies, including empty
selections and rejected filter members. A read of an optional field tracks presence
even when absent. A filter tracks every inspected predicate field and source
membership; downstream map fields are tracked for selected members only. Short-circuit
execution records guards and the examined prefix; unexamined inputs remain declared
but need not be read. A changed guard/membership invalidates the appropriate selection
before a previously skipped read may affect accepted output.

Expression compilation and runtime dependency integration must support bulk execution
and framework-managed element-granular map/filter maintenance. Retain stable element
keys, source membership/order basis, per-element predicate/result consumption, and
changed-entry output deltas under the owner's memory budget. Duplicate-looking values
with different element occurrences remain distinct. No user-managed sibling caches,
manual adjacency maps, one subscription per element, or scalar provider round trips.
An ordinary standalone full evaluation still honestly costs its full visited input.

For source insert/remove/reorder or guard changes, use the smallest correct sequence
or index granule and expose its cost. If a consumer requests a fully materialized
flat output list, charge its O(output bytes) delivery separately from incremental
evaluation. Sparse changes cannot be advertised as constant-time full-list delivery.
Canonical ordered floating sums may require suffix/full recomputation; report that
necessary scope rather than reorder arithmetic. All/any may rescan the necessary
prefix after its determining witness changes. Dense edits may choose an admitted
rebuild strategy with explicit cost, never an unbounded hidden fallback.

Reuse requires exact program/function/type meaning, compatible owner affinity,
complete tracked inputs, native dependency versions and the owner's declared
equivalence rules. Output content may remain reusable after reevaluation proves it
unchanged, but old currentness/approval evidence cannot survive native ABA by value
comparison alone. Unrelated versions do not invalidate source-local evidence.
Caches store derived results, never currentness. Signal/Bridge/Query retain their
existing eligibility and readmission roles; no expression-owned scheduler emerges.
Reuse must also satisfy the current result/input-size and semantic-work ceilings;
a prior result records enough logical cost to prevent a cache hit from laundering a
now-disallowed expression. Report reused logical cost separately from zero executed
work. Denied, incomplete, or cancelled evaluation cannot be reused as a successful
value. Incremental map/filter maintenance has its own admitted delta-work budget;
its success must equal full evaluation, while exhaustion remains an owner-visible
resource outcome rather than an invitation to fall back to an unbounded full scan.

Registry replacement, hot rebind, branch fork, program adoption, permission change,
and cold reconstruction invalidate or freshly bind owner state explicitly. A late
old-generation result is rejected even if its value equals the new expected value.
Unknown coverage denies reuse. Loss of a cache enters a bounded cold lane; it cannot
claim a warm hit, reconstruct the entire model inside a getter, or replay effects.

## Budgets, Outcomes, And Cost Contracts

Every source/admission/evaluation invocation receives a finite profile. Runtime owners
admit profiles and reserve resources; foundational only enforces the supplied pure
limits. Admission derives plan costs and rejects impossible resource contracts before
installation. Authoring may request narrower limits, never silently widen installed
ceilings. Structural work gates correctness; wall-clock numbers are diagnostics.

V1 ships named defaults, with explicit larger engineering profiles rather than an
unbounded switch:

| Ceiling | Interactive default | Engineering qualification |
| --- | ---: | ---: |
| Source bytes / AST nodes | 64 KiB / 8,192 | 1 MiB / 65,536 |
| Syntax depth / function call depth | 64 / 32 | 64 / 32 |
| Visited collection elements per invocation | 16,384 | 1,048,576 |
| Semantic work units | 1,000,000 | 100,000,000 |
| Logical input bytes / output bytes | 8 MiB / 1 MiB | 256 MiB / 64 MiB |
| Peak scratch / retained consumption bytes | 8 MiB / 2 MiB | 64 MiB / 64 MiB |
| Bit width | 4,096 | 4,096 |

These profiles are limits, not promised latency or permission to reserve all maxima
for every expression. Owners separately bound aggregate installed-plan/cache bytes,
active invocations, queued work, and pins; batch admission prevents per-expression
defaults multiplying without an aggregate ceiling. Large resident source populations
can exceed an invocation's admitted selection without requiring full materialization.
Admission uses a separately metered linear/sort-bounded work profile, not evaluator
fuel; parsed/typed nodes, symbol probes, type visits, call edges and canonical bytes
are charged and overflow-checked. Define exact finite admission ceilings at profile
installation from the listed input maxima and the chosen bounded algorithm.

The semantic meter is independent of optimization and host speed: charge one unit
per evaluated IR node and loop iteration, plus bytes inspected/emitted/copied and
64-bit limbs processed by variable-width operations. Checked fixed-width primitive
arithmetic has fixed cost. Decimal, widened tolerance, parsing, UTF-8 traversal, map
lookup, and collection equality expose their nonconstant work. Each builtin ships a
versioned normative cost formula with its implementation; optimized instructions
charge equivalent semantic work. Denial occurs before the next operation/allocation
would exceed its limit. Meter overflow itself is ResourceExceeded, never wraparound.

Expose logical work separately from physical counters: field/element reads, function
calls, comparisons, copied bytes, allocations, peak scratch, retained dependency and
cache bytes, cache hits/misses, cold compilation, provider contacts, and index work.
No warm parse/typecheck/canonicalization, global registry lock, or whole-model gather.
Cold admission is O(source + typed nodes + call edges + canonical bytes), allowing
named ordered-index logarithms; execution is O(visited instructions + charged input
and output work). No hidden quadratic subtree serialization or expanded-function copy.

Closed language denial families are Syntax, UnsupportedVersion, UnsupportedFeature,
UnknownBinding, AmbiguousBinding, TypeMismatch, InvalidValue, MissingOperand,
AbsentValue, Bounds, ArithmeticOverflow, ArithmeticDomain, DivisionByZero,
FunctionContractMismatch, and ResourceExceeded. Each carries a typed detail and
source occurrence where available. No catch-all string encodes machine behavior.
Internal invariant failure is a defect, not an authored false/absent result.

Owner outcomes separately distinguish Current(Value/Denied), Stale, WrongWorld,
UnsupportedInstallation, DisclosureDenied, Cancelled, and DeadlineExceeded. They
preserve underlying language denial; stale is never a language value. Cancellation
and deadlines are runtime concerns checked at bounded evaluator slice boundaries;
the pure core returns a budget-accounted suspended machine value if its quantum ends.
The owner retains that continuation and cumulative meter within bounded custody.
The stepped facade returns `ExpressionStep::Complete(ExpressionEvaluation)` or
`ExpressionStep::Suspended(ExpressionContinuation)`. Continuations are sealed pure
machine state, not live grants; owner binding is separate. The complete pure
`evaluate` convenience drives these steps within the supplied total budget, while
runtime owners use the stepped facade for cancellation and scheduling fairness.
No persisted executable continuation in V1; resume requires fresh owner currentness,
and losing continuation state restarts as a new admitted evaluation, not an effect.

Use a maximum slice quantum of 4,096 semantic units; a builtin exceeding remaining
quantum must checkpoint internal progress or yield before doing the work. Arithmetic
or allocation must not hide an unbounded noninterruptible step. A function call,
yield, retry, or batch cannot reset a logical evaluation's total limits. No successful
partial list on cancellation/exhaustion, no result publication until completion and
owner freshness checks. Partial diagnostic progress is explicitly non-authoritative.

## Query Cutover And UI 3.17 Adoption

Query declaration gains typed expression-backed conditions with declared query-result
operands. Installation compiles and binds their manifest to installed query contracts.
Execution gathers current disclosed inputs, evaluates, and retains the exact existing
source/currentness proof for transition admission. Output true/false chooses the
existing typed condition outcome; denial/stale/cancellation cannot select false or
fall through to an effect. The language cannot open a new query or authorize reads.

Migrate existing 9.17.6 workflow-condition consumers and their persisted declaration
format through a versioned readmission path. An old condition referring to an installed
Bool query maps to a new expression reading that explicit Bool operand, preserving
the original query's semantics and source proof. This is migration of old data, not
a permanent competing interpreter or ordinary legacy execution path. Unsupported
versions deny; old live instances require the existing program/definition disposition,
not silent in-place meaning replacement. Do not reinterpret general query filters
whose null/numeric semantics differ; those remain their existing versioned contracts.

Worth UI remains the owner of authored UI meaning. Its expression artifact wraps
shared admitted meaning with UI binding, intended result role, consumed aspects,
and source provenance. Planning evaluates over admitted UI/Query projection facts;
rebind classifies expression changes into affected lanes. Renderer receives already
resolved typed presentation, participation, options/payload, and scalar results.
Inspection renders retained explanation data without querying unauthorized sources
or rerunning semantics. Explanations have disclosure and richness budgets; redaction
cannot remove the fact of denial/staleness or turn missing evidence into success.

9.17.6.1 must ship source parsing/admission, one real source-edit/hot-rebind pulse,
and production integration tests for all five 3.17 uses. UI 3.17 retains ownership of
its complete UI product acceptance and any broader presentation-specific work; this
milestone's proof can be reused there, not replaced by a mocked evaluator. Neither
roadmap may defer the same shared-language integration to the other.

## Public Developer Experience

Target facade spelling below is a design contract, not an assertion that these APIs
exist. Implementation must publish compiling examples with consistent final names.
Ordinary callers select input schema, installed language support, and a finite profile;
they do not manage caches, meter increments, dependency ordinals, or per-element calls.

```rust,ignore
use worth_foundational::expression_api::{expressions, ExpressionProfile};

let admitted = expressions()
    .parse("opening.width - 2.0 * frame.thickness")?
    .admit(&operand_schema, &function_catalog, ExpressionProfile::interactive())?;
let compiled = admitted.compile()?;
let evaluation = compiled.evaluate(&immutable_inputs, &evaluation_limits);
// This pure value is NOT a Query transition or UI installation receipt.
```

Scalar-times-quantity and quantity-times/divided-by-scalar accept a finite Float64
scalar and preserve the quantity dimension. These are declared overloads, not general
numeric coercion; the example cannot add a plain Float64 to a length.

```text
// Expression slices embedded by UI's existing source parser:
approval.ready && !document.read_only
selection.name ?? "Nothing selected"
members.filter(m, m.material == Material::Steel).map(m, m.length)
quantity(900.0, mm) <= opening.clear_width
case_equal(signals.enable, logic4("1"))
```

Typed Rust builders produce the same draft and canonical identity without parsing
source strings. Unknown dynamic fields fail at admission; statically typed builder
connections enforce invalid operator/operand/result combinations where expressible.
UI and Query common paths hide pure compilation phases while still requiring their
own runtime-bound inputs and returning their exact typed outcomes. Advanced callers
can inspect canonical meaning, maximum reads, actual consumption, cost, and denial.

## Destination Topology And Enforcement

E = existing owner; N = new; R = extended/replaced responsibility; S = committed
successor destination, no empty placeholder. Paths start at repository root.

```text
crates/worth-foundational/src/
  expression_api/{mod,authoring,admission,evaluation}.rs       N public facade only
  expressions/
    syntax/{tokens,parser,ast,source_map}.rs                  N portable grammar/provenance
    types/{schema,operand,collection,quantity,digital}.rs     N shared type meaning
    admission/{typing,functions,access,resources}.rs          N pure structural validation
    canonical/{basis,encoding,identity}.rs                   N existing canonical API consumer
    program/{lowering,instructions,manifest}.rs              N discardable pure plan
    functions/{signature,catalog,closure}.rs                 N immutable installed meaning
    evaluation/{machine,operand_access,consumption,outcome}.rs N pure evaluator state per call
    evaluation/{meter,suspension}.rs                         N pure accounting/continuation data
    operators/{integer,float,decimal,quantity,digital}.rs    N separate numeric contracts
    operators/{boolean,option,string,collection}.rs          N separate semantic families
    compatibility/{decode,version}.rs                       N untrusted artifact boundary
  values/scalar_wrappers/ canonicalization/                  E reuse; no silent semantic rewrite
crates/worth-foundational/tests/
  expressions.rs                                           N grouped local suite
  expressions/{semantics,canonical,resources,conformance}.rs N distinct invariant families
  ui/expressions/                                          N existing grouped compile-fail lane
crates/worth-foundational/docs/expressions/
  {README,semantics,embedding,compatibility}.md              N durable public documentation
workspaces/worth-query/crates/
  worth-query-declaration/src/application_program/workflow/
    expression/{condition,operands,compatibility}.rs         N condition meaning/draft migration
  worth-query-installation/src/application_program/
    expression/{registry,binding,compilation}.rs             N runtime-installed support
  worth-query-execution/src/domain_computation/primary_graph/
    expression/{inputs,evaluation,currentness}.rs            N runtime-owned invocation
    expression/{reuse,resources,retirement}.rs               N existing owners' managed state
    application_attempt/workflow_transition_program/condition.rs R single new ordinary path
  worth-query-decl/src/facade.rs worth-query-host/src/facade.rs R audience reexports
  worth-query-certification/tests/application_graph/
    expressions/{conditions,engineering,resources}.rs        N grouped integration courts
workspaces/worth-ui/crates/
  worth-ui-dsl/src/source/parse/expression.rs                N embedding grammar delegation
  worth-ui-dsl/src/semantic/expression/
    {artifact,binding,role,provenance}.rs                     N UI-authored meaning
  worth-ui-runtime/src/runtime/planning/expression/
    {inputs,evaluation,consumption}.rs                       N owner-bound UI evaluation
  worth-ui-runtime/src/runtime/rebind/expression/
    {classification,currentness,retirement}.rs               N affected-lane lifecycle
  worth-ui-inspection/src/expression/{evaluation,source}.rs  N one-way disclosed explanation
  worth-ui-dsl/src/source/composition/                      S 3.18 preserved expansion source map
workspaces/worth-ui/apps/platform-pulse/tests/executable_world/
  source_delta/expression.rs                               N real source-rebind court
worth-proprietary/crates/worthy-house-certification/tests/
  journeys/expressions.rs                                  N real pinned consumer court
```

`expressions` is private; `expression_api` curates the shared facade and contains no
operator implementation. Syntax owns source meaning, types/admission own validity,
canonical owns identity, program owns derived instructions, operators own mathematical
semantics, and evaluation owns only local pure state. New operators enter their
semantic family without moving the facade or leaking runtime knowledge. The committed
UI composition successor adds expansion provenance, not a second AST/evaluator.

Catalog descriptors are portable immutable meaning. Live registry ownership stays in
installation/session owners; Query and UI may share immutable plans but not authority,
source maps, lifecycle state, or currentness handles. Runtime resource/reuse modules
extend existing managed owners rather than introducing their own cache service.
Inspection depends on disclosed projections, never the reverse. Product compositions
remain above public consumers; public WORTH has no proprietary import.

No `expressions.rs` god file, generic value bag, per-operator crate, runtime-bearing
substrate registry, UI-to-Query-internals import, or foundation-to-runtime dependency.
Enforce visibility/manifest direction through existing boundary-check rules and
compile-fail tests with valid counterparts. All production/test files remain within
the existing 400-line cap; this specification grants no exemption. New APIs enter
agent-context inventories through the generator, never hand-edited generated files.

## Ordered Phases

### Phase 1: Canonical language admission

Establish the complete V1 type/operator/function semantics, bounded shared parser,
typed builder, draft codec, source map, pure admission, and canonical program identity.
Consume existing scalar/canonical vocabulary and preserve its historical meaning.
Prove source/builder/decode parity, exact collision protection, width/unit/type denials,
bounded parser/type work, and forged admitted-plan compiler denial. A valid typed
program and independent semantic vectors, not a parser demo, are the next phase's input.

### Phase 2: Deterministic bounded evaluation

Implement the pure evaluator, all specified operators, acyclic installed bodies,
semantic/physical accounting, bounded suspension, and consumed-path recording.
Prove cross-target numeric bytes, independent digital truth tables, CEL intersection,
explicit divergence vectors, nested-budget exhaustion, and allocation/cancellation
safe points. No unmetered callback or hidden ambient input is callable. The next phase
trusts complete portable semantics; no runtime authority is claimed by this phase.

### Phase 3: Query binding and real condition cutover

Bind admitted manifests to current installed Query observations; migrate workflow
conditions and their ordinary consumers to the single shared evaluator. Retire the
replaced ordinary condition path after versioned draft migration and instance/program
disposition proof. Preserve fresh source/authority fencing, ABA, idempotency, effect
custody, and canonical definition identity evolution. Real Bank and proprietary House
conditions plus foreign-world/revocation races prove actual integration. Query source
providers remain compiled Rust and cannot acquire evaluation authority from a Bool.

### Phase 4: Worth UI source, planning, and rebind

Integrate the existing DSL parser and typed artifacts, all five 3.17 result uses,
planning evaluation, consumed-aspect capture, inspection, and expression-aware rebind.
Close the real live source-edit pulse, stale-generation race, invalid-source retention,
and zero-unrelated-lane work. UI source remains authoritative; compiled expressions
and mounted presentation are discardable. UI 3.17 receives implemented shared-language
contracts and reusable real-product evidence, not a future integration promise.

### Phase 5: Engineering scale, lifecycle, and closure

Complete managed element-granular reuse, bulk adapters, CAD/digital courts, profile
saturation, aggregate memory, retained-reader/registry replacement, and cold rebuild.
Qualify every independent scale axis, with no wall-clock acceptance threshold and no
denial-only large-scale proof. Verify complete lifecycle cleanup and exact canonical
parity after reconstruction. Finish docs, compatibility/deletion inventory, full
affected suites and boundary review. Numerical/domain kernels remain their existing
owners; the language does not expand into a new scheduler to meet its scale claims.

## Verification, Review, And Documentation

Pin official [CEL conformance cases](https://github.com/cel-expr/cel-spec) and a
[reference implementation](https://github.com/google/cel-go) by immutable revision
and preserve licensing. Use only the explicitly mapped semantic intersection as a
differential oracle. Every imported case is classified as supported-equivalent,
deliberately divergent with a WORTH expected result, or unsupported with typed
admission denial. No changing reference version to make a failure disappear.

WORTH deliberately differs in mandatory static typing, no dynamic coercions, finite
floats, option absence, left-to-right error propagation, dimensions, and digital
types. CEL parity cannot certify these. Use independent fixed vectors, bounded
property generation/shrinking, parser/codec fuzzing, exhaustive small-width bit/logic
tables, decimal/rational reference arithmetic, and an independent slow evaluator for
the WORTH subset. Do not derive expected outputs from production implementations.
Native transcendental precision is not a portable guarantee; see the
[Rust f64 contract](https://doc.rust-lang.org/std/primitive.f64.html).

Require x86-64, AArch64, and wasm32 parity on the declared portable core. Unsupported
targets deny support rather than silently weaken determinism. Debug/release,
interpreter/optimized path, cold/warm, and source/builder/decoded construction must
agree on values and typed denials. Physical work may differ; semantic resource
charging and observable evaluation order may not. Random tests record reproducible
seeds and minimized regressions without creating a new proof ledger.

QA review must cover numeric/absence/order semantics; source provenance and canonical
collision resistance; authority/currentness/disclosure; worst-case resource and native
stack behavior; incremental negative-space correctness; cross-target behavior; real
UI/Query/proprietary integration; registry/rebind/cancellation/reconstruction lifecycle;
and removal of replaced ordinary paths. Boundary review belongs to the integrator.
Reviewers judge evidence adequacy; passing conformance vectors alone cannot close an
authority or product-boundary claim. Preserve unaffected accepted predecessor evidence.

Use focused owner tests during work, grouped compile-pass/fail targets, existing
Query certification and platform-pulse harnesses, then all affected workspace suites
at closure. Scheduled large-scale/architecture lanes have finite resource profiles.
Record hardware/toolchain/workload, cold/warm posture, repetitions, p50/p95, and
structural/byte/memory counters; timing is diagnostic and timeout only a hang guard.
Required repository checks include formatting, dirty Rust line caps, boundary-check,
and agent-context. No warning suppression, skipped failing lane, invented review
clearance, temporary probe residue, or golden rewrite for unchanged meaning.

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
```

Documentation is part of implementation acceptance:

- Foundational authors/embedders: add `crates/worth-foundational/docs/expressions/`
  as shown above; link from existing README/docs index. Specify every operator,
  literal, conversion, cost formula, profile, outcome, source mapping, and unsupported
  family. Explain pure evaluation versus operational authority and the finite-value
  refinement of existing carriers. Public examples compile against the real facade.
- Query developers: revise
  `workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md`
  and `domain-capabilities/conditional-installed-operations.md` for declared expression
  operands, real condition authoring, typed failure/staleness, fresh admission,
  migration/disposition, and incremental cost. Correct superseded condition examples.
- UI authors/runtime integrators: revise `_docs/worth-ui/worth-ui-dsl-vision.md` and
  the DSL/runtime crate READMEs for supported syntax, five evaluation roles, disclosure,
  absence, source diagnostics, consumed aspects, and live rebind behavior. Keep the
  platform-pulse source example executable; renderer code is not an explanation oracle.
- CAD integrators/operators: revise `worth-proprietary/docs/house/query-platform.md`
  with actual unit-aware rule/member selection, public dependency pin, bounds,
  unchanged-output reuse, and Rust-plugin boundary. Describe the tested scale envelope
  without claiming geometry-kernel or simulator certification.

## Completion And Successor Handoff

Closure requires the shared foundational language, complete V1 semantics, bounded
pure evaluator, enforced installed-function model, actual Query cutover, real UI
source/rebind use, proprietary CAD proof, digital/scale evidence, lifecycle cleanup,
public docs, and no known material defect in the changed boundary. A line-count or
calendar estimate is not an acceptance criterion. Specification length is capped at
1,200 lines; implementation follows the repository's per-file cap.

UI 3.17 consumes the shared kernel and completes its own remaining product acceptance;
3.18 adds module/fragment provenance through the same source map and canonical IR.
Query 9.19/9.20 may supply richer admitted access products/set execution, never hidden
expression graph access. 9.22 may consume exact occurrence-safe reuse; this milestone
must already close the narrower expression-owned consumption/reuse promise.
Future deterministic math or domain type additions require explicit versioned meaning,
resource contracts, canonical identity, and independent conformance; they cannot alter
V1 in place. No old archive or cached plan mints current runtime authority.
