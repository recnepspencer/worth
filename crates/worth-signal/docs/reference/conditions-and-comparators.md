# Conditions And Comparators

Conditions, comparators, triggers, output equivalence, and aspects answer
different questions. Keeping them separate lets Signal avoid unnecessary work
without misreporting what changed.

## Main Rule

- "Which part of this node changed?" is an aspect question.
- "Is this node eligible to evaluate?" is a condition question.
- "Was evaluation explicitly requested?" is a trigger question.
- "Does a committed dependency delta matter here?" is a dependency-comparator question.
- "Did this evaluation produce a meaningfully different output?" is an output-equivalence question.
- "May an earlier artifact be reused?" is an artifact-equivalence question.

## Standalone Signal Surfaces

- `Aspect`
- `EvaluationCondition`
- `VersionComparatorPolicy`
- `OutputEquivalencePolicy`
- `.on_demand()`
- `.debounce(...)`
- `.aspect_filter(...)`
- `.delta_threshold(...)`
- `.custom_condition(...)`
- `.dependency_comparator(...)`
- `.output_equivalence(...)`

`NodeBuilder::dependency_comparator(...)` and
`NodeBuilder::output_equivalence(...)` are deliberately separate. The former
belongs to the consumer; the latter belongs to the producer. The deprecated
`comparator(...)` spelling lowers only to the consumer role.

## Installed Conditional Contracts

Query-hosted Signal work uses an installed contract:

The host binds that contract through Runtime Bridge's sealed
`BridgeOwnedSignalRuntime` service. Signal admits and evaluates the exact
component branch basis and returns performed evidence; Query and Runtime World
carry it as part of the selected product occurrence. Do not reopen the raw
Signal graph or infer component authority from a product or branch identifier.

- `InstalledSignalConditionalContract`
- `InstalledSignalConditionIdentity`
- `InstalledSignalComparatorIdentity`
- `InstalledSignalConditionDecision`
- `SignalConditionalExecutionRequest`
- `SignalConditionalDecisionEvidence`

Query authors portable semantic dependencies, condition families, trigger
families, thresholds and units, comparison requirements, maintenance posture,
and output relationships. Runtime Bridge admits those declarations against the
actual Signal graph and lowers them into `InstalledSignalConditionalContract`.
Signal remains the only owner of the runtime decision.

## Decision Semantics

The installed execution path preserves the exact decision classes:

- `ComputedChanged`
- `ComputedRevertedClean`
- `DependencyUnchanged`
- `SuppressedBeforeCompute`
- `DeferredByCondition`
- `DeferredTemporal`
- `DeferredOnDemand`

Dependency-unchanged, suppressed, and deferred decisions do not invoke
compute. Reverted-clean decisions preserve compute cost but must not emit a
semantic change. Query and Runtime Bridge carry this evidence; they do not
infer or reclassify it.

Before Signal asks any ordinary, temporal, on-demand, custom, or installed
condition whether a node is eligible, it resolves that node's immediate
pending dependency causes. A deferred upstream is not stable-output evidence
and cannot release a dependent. A required structural recompute also cannot be
skipped by an ordinary condition.

## Delta Thresholds

Portable Query thresholds carry typed unit identity, value family, comparison
domain, and inclusive/exclusive boundary. Runtime Bridge lowers that meaning;
Signal resolves the threshold against installed dependency observations.

Do not reimplement a threshold in a Query executor or Bridge callback. A
provider is appropriate only for a declared typed comparator family.

## Custom Conditions And Triggers

Standalone Signal graphs may use host-local custom conditions. Query-installed
operations instead declare typed portable families. Runtime construction
registers the matching Bridge provider.

A string-dispatch callback is not a typed family. A provider cannot select a
different family or redefine the portable condition.

## Producer-Local Aspects

Signal `Aspect` values are runtime-local node slots. Their meaning is scoped to
the installed graph, producer node, partition, and lowering owner.

Foundational aspect contracts and Relational aspect bindings are portable
semantic meaning. Runtime Bridge installs the correspondence between the two.
Do not persist a Signal aspect number as domain identity.

Consider this graph:

```text
source --A--> translator --B--> consumer with aspect_filter(B)
             translator produces B
```

When source `A` changes, the translator first resolves that cause and may
publish a committed `B` delta. The consumer is gated from the translator's
`B` cause. Signal never propagates the root's `A` slot as if it also meant `A`
on every descendant. The executable test
`aspect_filter_uses_the_immediate_producers_translated_aspect` covers this
three-node translation.

For scoped outputs, author exact locality with
`NodeEvaluationResult::with_changed_aspect_region(aspect, region)`. Signal
preserves each `(aspect, region)` pair through cause admission, checkpoint
restore, conditions, and async admission. It does not form the cross product
of an aspect mask and a separate scope bag. The unpaired
`with_changed_region(...)` API remains a conservative legacy union when a
single evaluation changes multiple aspects.

## Source Intent Is Not An Output Delta

`mark_changed` and `mark_changed_with_regions` request root recomputation.
Their `ChangeBatchAdmission` result proves admission only. A producer delta is
created later, after evaluation and producer output-equivalence have selected
the committed output. Deprecated names containing `Commit` are compatibility
aliases for admission and do not grant output-commit authority.

## Anti-Patterns

- Treating a condition and comparator as interchangeable.
- Running compute before the condition decision.
- Treating any candidate output as a meaningful committed change.
- Treating `mark_changed` or `ChangeBatchAdmission` as a committed producer delta.
- Reusing one comparator policy for producer output equivalence and consumer dependency comparison.
- Copying a root aspect or root scope through transitive descendants.
- Re-evaluating a Signal decision in Query or Runtime Bridge.
- Using string dispatch for an installed condition or trigger family.
- Treating a numeric Signal aspect as portable semantic identity.

## Related Docs

- [Aspects And Dependencies](../core-concepts/aspects-and-dependencies.md)
- [Defining Computation](../guides/defining-computation.md)
- [Query Conditional Installed Operations](../../../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
