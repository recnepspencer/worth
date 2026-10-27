# Complete paths in declared queries

A declared query can select every live node reachable over one endorelation.
Compose the complete step with ordinary hops and guards:

```rust
ApplicationQueryRootPath::from(Node::reference())
    .forward(Next::reference())
    .where_equal(Blocked::reference(), ApplicationEncodedScalarValue::try_new(false)?)
    .forward_closure(Next::reference())
    .where_equal(Blocked::reference(), ApplicationEncodedScalarValue::try_new(false)?)
```

Pass the path to the query definition builder's `root_path`. Use
`reverse_closure` for complete incoming reachability. The closure is reflexive:
it includes its live starting nodes, including a root with no edges. Cycles and
diamonds return each entity once; the physical relation evidence still includes
cycle and nondiscovery edges. Typed closure methods accept only a relation whose
source and target are the current entity type. Portable validation enforces the
same condition independently.

A guard before a closure filters its starting frontier. A guard after it filters
returned entities. Every returned row retains its actual discovery predecessor's
path evidence plus the shared complete Native walk. Query does not run a second
traversal or attach every predecessor's guards to every row.

`result.receipt().root_path_steps()` describes executed authored steps using
zero-based path and step indices, direction, traversal mode, and actual acquired
reads. The same evidence follows continuation and live execution. It contains
no graph IDs or read authority. `reads()` remains the whole query's observation;
authorization work is separate. Empty frontiers record zero reads for the step.

Complete traversal uses the original request's work, cancellation and memory
custody. Result limits constrain the returned selection; an authored step's
ordinal is not a depth limit. Admission requests a complete traversal workset and
visited set. Cost estimates remain conservative unknowns rather than promises
about frontier size. Refusal produces no partial query result.

Original complete walk facts participate in row and result-set source identity
and currentness. An unrelated change can preserve them; a changed empty deep
adjacency or nondiscovery edge invalidates them even when the public rows remain
identical. Temporary verification shares comparison work within the current
call and selected Native image. Structural walk facts remain unsupported for
cold checkpoint reuse and selective membership cutoff witnesses.

Closure steps are query selection capability. The operation-body path profile
continues to require one distinct identity at each ordinary hop; it refuses
closure steps rather than interpreting them as one hop or a multi-target effect.
RecordFrame protocol 15 carries the exact traversal discriminator and refuses
older record grammar.

The executable examples and source/currentness/resource courts are in
[the installed walk fixture](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_walks/declared_queries.rs).
