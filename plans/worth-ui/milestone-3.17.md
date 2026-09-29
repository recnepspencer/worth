# Milestone 3.17: DSL Expressions, Conditions, and Semantic Evaluation

## Goal and placement

Authored conditions and derived values become part of the real Worth UI language.
A `.wui` file can state a typed, pure expression over declared facts. Compilation
admits it through the shared kernel in `worth_foundational::expression_api`. The
runtime evaluates it once per relevant fact change. The result drives presence,
participation, operability, payload, or a displayed scalar. Inspection explains
each outcome from retained evidence.

There is no UI-local interpreter. Renderer conditionals, Rust closures, and
ambient reads cannot stand in for an expression. The kernel owns grammar, typing,
canonical meaning, and bounded evaluation:

- `worth-ui-dsl` owns expression declarations, operand binding roles, and source
  provenance.
- The runtime owns evaluation over admitted facts.
- Rebind owns currentness and the expression lane.

This follows [3.16](./milestone-3.16.md). [3.18](./worth_ui_roadmap.md) extends
the source and expansion map. [Query 9.17.6.1](../WORTH-query/milestone-9.17.6.1.md)
phases 1-3 supply the kernel. Its phases 4-5 are
[deferred](../deferred-work.md), and 3.17 does not close them.

## Language

An expression is a declaration with a stable identity, a closed set of named
operands, and a result role:

```
condition platform.pulse.condition.action_allowed {
  operand status query-scalar platform.pulse.status;
  operand allowed application-boolean platform.pulse.action.policy-allowed;
  when (status == "ONLINE" && allowed)
}

derived platform.pulse.derived.status_label {
  operand status query-scalar platform.pulse.status;
  result text;
  value (status == "ONLINE" ? "Online" : "Syncing")
}
```

The two declaration kinds:

- `condition` is the only way to author a `bool`.
- `derived` names its `result` type: `text`, `integer`, `decimal`, or `token`.
  A `token` result selects a theme token, which gives data-driven appearance.

Operand sources are a closed vocabulary:

- `query-scalar <query_scalar>`: its type comes from the declaration's
  `require`, which gains `integer` and `decimal` beside `text` and `boolean`.
- `application-boolean <fact>`
- `application-unsigned64 <fact>`
- `application-text <fact>`
- `condition <id>` and `derived <id>`: another named expression's result.

Named expressions form an acyclic graph, so multi-step logic is a chain of
named steps. Each step is typed, evaluated, and explained on its own. Expression
bodies may not use kernel `let` bindings; an intermediate result is always a
named declaration. Row-scoped operands (`lot.status` inside a repetition) arrive
with 3.18's keyed repetition. The operand model reserves room for them.

Rules for the expression body:

- The parenthesized body is kernel source. The DSL finds its extent by
  balancing `()[]{}` while skipping kernel strings (`"…"` with escapes) and
  `//` comments.
- The DSL hands the inner slice to the kernel as it is. It never re-tokenizes
  expression text.
- Kernel spans are offset by the slice's start byte, so every diagnostic names
  its authored clause in the host file.

Use sites refer to an expression by identity. Each use has one typed role:

| Use | Site | Role type |
| --- | --- | --- |
| Conditional presence | region `mount … when <condition>` | `bool` |
| Conditional participation | slot participant `participates when <condition>` | `bool` |
| Operability projection | intent `policy-condition` / `readiness-condition` / `mutability-condition <condition>` | `bool` |
| Payload shaping | intent `payload <name> derived <derived>` | the payload's type |
| Derived scalar | scalar text presentation `value derived <derived>` | `text` |
| Data-driven appearance | appearance aspect `use derived <derived>` | `token` |

Compilation fails when a use site's role type differs from the expression's
result type. A use site only names an expression; there is no inline expression
at use sites.

## Laws

1. **One kernel.** Admission, canonical identity, and evaluation go through
   `expression_api`. The UI never interprets expression syntax or values.
2. **A denial is never false.** Unsupported, impure, ambiguous, ill-typed, and
   wrong-world expressions deny. So do missing, stale, or unavailable operands.
   A denied or stale outcome keeps its posture all the way to the consumer.
   Operability treats it as not operable, with the denial as the reason.
   Presence and participation keep the last current outcome, marked stale, or
   deny if no current outcome exists. No consumer turns it into `false`.
3. **Currentness before evaluation.** An evaluation first proves every operand
   current: query world, binding, result generation, and application fact
   revision. It then records those identities and the program identity. A
   result delivered after its generation was replaced is stale and cannot
   change the replacement.
4. **Identity excludes spans.** An expression enters the package's exact basis
   through its operands, result role, and the kernel's program identity.
   Spans live in a provenance side table. Equal meaning shares identity, and
   each source keeps its own spans.
5. **Bounded invalidation.** Evaluation keys on declared operand facts. When a
   fact no expression reads changes, zero expressions evaluate. A result change
   invalidates only its consumers. Op counters prove this; timing does not.
6. **All-or-nothing source.** Expression admission runs inside
   `compile_source`. An invalid expression in new source denies the whole
   candidate, and the prior generation stays installed.
7. **Retained explanation.** Every evaluation keeps a record. The record holds
   the outcome, the operand fact identities, the consumed aspects from
   `consumption()`, the program identity, the generation stamp, the cost, and
   the span. Inspection reads the record and never re-executes.
8. **One canonical form.** Each meaning has exactly one authored spelling.
   Booleans are conditions. Intermediates are named declarations, not `let` or
   inline use-site expressions. Every operand states its source and type.
   Constrained, named, typed structure makes source easier for people and
   agents to author correctly. See the evidence note below.

## Phases

Opus implementers do the work from self-contained briefs. A separate, fresh
Opus reviewer gates each phase with the qa-loop, qa-tests, and
code-quality-qa skills before the next phase starts. Every phase must pass BC,
AC, clippy, fmt, and the tests of the crates it touches.

### Phase 1: expression declarations (`worth-ui-dsl`)

Completed.

**Parsing and extent**
- Tokens and parsing for `condition` and `derived`.
- The operand grammar.
- The extent scanner.
- Handing the body to `expressions().reading_within(ExpressionProfile::interactive())`.

**Admission during compile**
- An operand schema built from the operand kinds.
- A builtin catalog.
- `admit_as` the declared result type.
- Kernel denials become compile diagnostics with offset spans. New diagnostic
  codes are added per denial family class.

**The artifact**
- `WorthUiExpressionArtifact` holds:
  - the identity;
  - the role;
  - the operands;
  - the encoded draft;
  - the program identity;
  - the admitted program;
  - the provenance and span table.
- It is sealed with the package.
- It enters the exact basis and fingerprint through law 4.
- Duplicate and unknown operand, identity, and source diagnostics.

**Rust-authored parity**
- Rust-authored packages declare expressions by kernel source text through the
  same admission path.

**Tests**
- Parse and admit for each operand kind and result type.
- Spans offset correctly across multi-line files.
- Identity is equal across whitespace and comment edits and differs across
  meaning edits.
- Each denial class yields its code.
- An invalid expression denies the package.

### Phase 2: evaluation owner (`worth-ui-runtime`)

Completed.

**Installation**
- `runtime/expression/` installs each artifact's compiled program once per
  prepared generation.

**Operand binding**
- Operands bind through their owners:
  - Query scalar text comes through its projection fact receipt and posture;
  - application facts come with their revision.
- Absent, unavailable, stale, and wrong-world operands settle the outcome
  without evaluating (law 2).

**Evaluation and dependencies**
- The owner evaluates and keeps the law 7 record.
- A dependency index maps operand facts to expressions.
- A result is republished, and its dependents re-evaluated, only when its
  value or posture changes.
- The produced fact family `ExpressionResult` arrives in 3a with its first
  consumer, and the `Expression` lane arrives in Phase 4 with its first plan
  nodes. Neither lands as an empty placeholder.

**Tests**
- Each outcome posture: true, false, value, denied, stale, unavailable.
- The consumed aspects.
- Zero evaluations when an unrelated fact changes.
- A late result against a replaced generation is stale.

### Phase 3: the five uses (`worth-ui-dsl` + `worth-ui-runtime`)

The phase lands in this order:

- **3a operability.** Condition sources for mutability, readiness, and policy
  feed `observe_operability_basis`. The existing decision and affinity code
  stays the consumer. The appearance Operability axis follows unchanged.
  Completed. A changed condition re-observes only the standing facts of the
  intents that read it. Application-fact and projection sources stay
  pull-only.
- **3b payload shaping.** A `derived` payload source.
- **3c derived scalar.** Scalar text presentation from a `derived` value.
- **3d conditional presence.** A region mount gated by a condition, through the
  existing identity lifecycle decisions (Create/Retire). It never uses a
  renderer skip.
- **3e conditional participation.** A slot participant gated by a condition.
  It stays mounted with its state preserved, but leaves layout, focus order,
  and hit testing while false.
- **3f data-driven appearance.** An appearance aspect takes its token from a
  `derived` token value. The state-axis decision table stays as it is. A
  derived token and state cells compose through the existing resolver.
- **3g typed Query scalars.** Scalar projections admit `require boolean`,
  `require integer`, and `require decimal` beside `text`.
  - Today the whole scalar binding path is text-only: one Query domain
    operation, its executor, native request, progression, and outcomes. Even
    `boolean`, which the DSL parses, is refused at binding.
  - The path becomes generic over the native value family. It is not copied
    per family.
  - Values keep their fact receipts. A value of the wrong type is a typed
    projection posture, never a coercion.
  - The five uses above prove themselves over text and application facts
    first, so this does not block them.

Before each brief, I read the owning code and fix the seam. Each use gets
tests for its denial and stale postures.

### Phase 4: rebind expression lane (`worth-ui-runtime`)

**Classification**
- Source classification gains an authored expression fact kind.
- An `Expression` lane joins `WorthUiPlanExecutionLane`, lowered from one
  plan node per installed expression.
- A meaning change (program identity or operands) invalidates that expression
  and its consumers through that lane.
- A span-only change is evidence only.

**Proofs**
- Invalid new expression source keeps the prior generation.
- Unrelated source edits cause no expression evaluations.
- A result in flight across a source replacement arrives stale.

### Phase 5: inspection (`worth-ui-inspection` + runtime producer)

**Evidence type**
- `UiExpressionInspectionExplanation` holds:
  - the result role;
  - the outcome: true, false, a value, denied (family, detail, and span),
    stale, or cancelled;
  - the operand facts with their Query identities;
  - the consumed aspects;
  - the cost.

**Budgets and producer**
- Evaluation and explanation budgets are independent.
- Disclosure and redaction never hide a denial or staleness.
- The producer reads law 7 records.

### Phase 6: Platform Pulse

**Pulse source**
- Pulse authors `platform.pulse.condition.action_allowed` over the status query
  and the policy fact. Its operability visibly changes the action control's
  appearance.
- A `derived` status label drives the status text.

**Product evidence**
- Pulse publishes expression evaluation and rebind observations in its
  lifecycle observation contract.

**Executable world**
- `source_delta/expression.rs` edits the expression clause in the real source,
  for example `"ONLINE"` to `"SYNCHRONIZED"`. The court sees:
  - the live process rebind;
  - the pixels change through the visual oracle;
  - product-issued evaluation evidence joined in adjudication.
- A malformed-expression delta keeps the prior pixels and reports a span
  diagnostic.
- A Query value flip through `QueryStatusV1`/`V2` flips the outcome with no
  source edit.

### Phase 7: closeout

**Docs**
- Update the DSL expression docs under `docs/`: the syntax, operand kinds,
  denial postures, and inspection.

**Status**
- Update the roadmap status.
- Log any deferrals in [deferred work](../deferred-work.md).

## Out of scope

- 9.17.6.1 phases 4-5 (the closure courtroom, House courts, and AArch64
  parity).
- Column or bulk operand inputs.
- Dimensional (quantity) Query operands.
- Keyed repetition over a Query collection and row-scoped operands. These
  belong to [3.18](./worth_ui_roadmap.md), which needs typed fragments to pass
  a row into a component.
- Expressions inside appearance `when` cells. They stay an exhaustive decision
  table over the six state axes. Data-driven appearance goes through a derived
  token (3f) instead.

## Evidence note

Khalfan and Al Mazrouei,
[Anka](https://arxiv.org/html/2512.23214v1) (2025), compare LLM-generated code
in a constrained DSL with generated Python. The DSL has:

- one canonical form per operation;
- mandatory named intermediates;
- explicit step structure;
- declared schemas.

On multi-step tasks, accuracy rose from 60% to 100% (Claude 3.5 Haiku) and by
26.7 points on GPT-4o-mini. It made no difference on one- or two-operation
tasks, and it lost 10 points on logic-heavy "hard" tasks. The study is small:
two small models, one benchmark written by the authors, and the
data-pipeline domain only. It supports law 8 and the named-step rule. It does
not justify replacing the shared kernel's operator syntax with keywords, since
single expressions stay short.
