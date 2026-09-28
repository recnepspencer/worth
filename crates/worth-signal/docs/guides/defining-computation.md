# Defining Computation

This guide covers computed nodes, dependencies, conditions, recipes, and keyed
families.

Read these first if you need them:

- [../core-concepts/graph-and-nodes.md](../core-concepts/graph-and-nodes.md)
- [../core-concepts/aspects-and-dependencies.md](../core-concepts/aspects-and-dependencies.md)
- [../reference/conditions-and-comparators.md](../reference/conditions-and-comparators.md)

The examples use:

- product price
- inventory
- shipping quote
- checkout summary

Other common cases:

- one file changes and you want one rebuild target to rerun
- one section of a long document changes and you do not want to reindex the whole document
- one source file changes and you want symbol indexing, diagnostics, and one build target to update without touching the rest

## What You Use

- `graph.node()`
- `NodeBuilder`
- `DependencyEdge`
- `Recipe`
- `runtime.define(...)`
- `RecipeFamily`
- `KeyedRecipe`

## One Compact Example

This example has three pieces:

- `product_price`
- `inventory`
- `checkout_summary`

`checkout_summary` depends on price and inventory. It only runs when asked for,
and it uses output identity so downstream work does not churn when the summary
is still the same thing.

```rust
use worth_signal::facade::*;

const PRICE: Aspect = Aspect::new(0);
const INVENTORY: Aspect = Aspect::new(1);

let mut graph = SignalGraph::new();

let product_price = graph.node().build();
let inventory = graph.node().build();
let checkout_summary = graph
    .node()
    .on_demand()
    .output_identity()
    .build();

graph.set_dependencies(
    checkout_summary,
    [
        DependencyEdge::new(product_price, PRICE),
        DependencyEdge::new(inventory, INVENTORY),
    ],
)?;
# Ok::<(), SignalError>(())
```

That is already enough to say something useful:

- two source nodes
- one computed node
- explicit dependency edges
- lazy execution
- stable output identity behavior

## 1. Start With Clear Nodes

Nodes should stand for real things:

- source facts like `product_price`
- computed results like `checkout_summary`
- expensive pull-based results like `shipping_quote`

Useful builder helpers:

- `build()`
- `on_demand()`
- `partitioned_output()`
- `output_identity()`

If you cannot name the node clearly, the node is probably doing too much.

## 2. Use Conditions When The Question Is "Should This Run?"

Conditions answer whether a node is allowed to run.

Common cases:

- `on_demand()` for lazy work
- `debounce(...)` for bursty updates
- `aspect_filter(...)` when only some change kinds matter
- `custom_condition(...)` when the host decides

Use conditions when the question is about timing.

Do not use conditions to hide bad dependency setup. Conditions are about when a
node runs, not who it depends on.

## 3. Use Comparator Policy When The Question Is "Did That Change Matter?"

Comparator policy answers a different question:

- should a changed upstream version count for this node?

Use comparator policy for:

- tolerance-based change suppression
- output identity aware suppression
- custom comparator behavior

Do not use comparator policy as a substitute for clear business rules. If the
system needs a real condition, use a condition.

## 4. Use Recipes For Repeated Computation Shapes

When the same computed thing should exist as a stable runtime concept, use a
recipe instead of rebuilding the same shape by hand.

That gets you:

- one declared computation shape
- stable runtime-owned identity
- clearer reuse and diagnostics

This is one of the places where the system gets cleaner.
You stop hand-wiring the same pattern over and over and let the runtime own it.

## 5. Use Keyed Computation For "Same Work, Different Stable Key"

Keyed computation is for cases like:

- one computed node per file path
- one computed node per account id
- one computed node per product id

Use it when the system naturally says:

- same kind of work
- many instances
- each instance identified by a real stable key

That is cleaner than wiring dynamic nodes by hand.

Typical cases:

- one file path
- one account
- one product
- one entity id

## Practical Rule

If your question is:

- "What is this result?"
  - define a node or recipe

- "When is it allowed to run?"
  - use a condition

- "Did this upstream change matter enough to count?"
  - use comparator policy

- "Is this the same computation shape with many stable identities?"
  - use keyed computation
