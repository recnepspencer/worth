# Architectural Laws

1. Runtime state must be partitioned into autonomous subsystems behind contractual facades. Each operation receives only the state and authority it uses; read paths accept phase-scoped observation handles incapable of mutation. Subsystems must be independently borrowable, testable, and replaceable, making false field coupling impossible and snapshot isolation structural.

2. Write-path contracts and read-path contracts are duals of the same truth. Every mutation must structurally declare what it invalidates, every projection must structurally declare what it consumes, and the framework must compute their intersection. Implicit contracts force global coupling.

3. A contract declared on a type strictly dominates one discovered at runtime. Dependencies, invariant groups, and context requirements must be compile-time declarations from which the framework verifies and sequences the system graph; runtime discovery is architectural late binding and late failure.

4. Semantic intent, declared contracts, resolved policy, and current structural state must compile before execution into a lowered plan fixing phase sequence, invariant set, strategy, artifact policy, and parallel-admission proof. The executor accepts only that plan and may not rediscover or re-decide them; genuinely unavailable runtime facts must appear as explicit plan-controlled branches.

5. Public outcome topology must be typed, composable, and machine-readable: decisions distinguish success, advisory, violation, and indeterminate states; failures preserve kind, context, and recovery semantics. Booleans and strings may render outcomes but must never encode them.

6. Domain mutations emit typed declarative effects and never address consumers. The framework derives routing, observability, audit, patches, and events from effect shape; producer knowledge ends at the effect contract.

7. Every boundary emits a self-describing envelope with a canonical core—typed outcome, effect or delta, boundary and schema identity, integrity, contractual cost, recovery or commit posture, and diagnostic disposition—and policy-materialized sidecars for warnings, traces, explanations, and high-cardinality counters. Sidecar omission is explicit; policy may remove richness, never facts required to interpret correctness, authority, security, recovery, or declared cost.

8. Operational reconstruction uses two orthogonal artifacts. A span-aware, queryable decision log records authority, policy, inputs, outcomes, and causal reasoning with O(1) decision lookup and incremental summary; a checkpoint plus its bounded subsequent journal reconstructs authoritative state. Neither substitutes for the other, and checkpointing must not stop mutation authority.

9. Construction and lifecycle propagation must be compiler-total: required fields are enforced by typestate or exhaustive construction, and adding a subsystem must break every construction and fork site until it is initialized and propagated. If omission compiles, lifecycle completeness is conventional.

10. Semantic identity dominates representation. Every name, field, accessor, and public type has exactly one meaning; values differing in domain meaning, truth status, lifecycle, or authority remain distinct sealed types even when their layouts match. Private representation reuse or phantom tagging is legal only across shared invariants, failure behavior, and cost class, and representation must never reconstruct or promote authority; lost authority requires fresh admission by its owner.

11. Configuration and context must mirror subsystem and authority boundaries. Static configuration is nested by subsystem; semantic runtime context is typed, declared, explicitly injected at an entry or phase boundary, and thereafter framework-carried. Ambient state may transport plumbing only and must not affect outcomes or open authority.

12. Unify constructs only when they share semantic authority, lifecycle, failure topology, and cost class; encode mode, target, topology, strategy, and artifact policy as parameters of that shared lifecycle. Similar shape alone is irrelevant, and abstraction must stop before merging correctness distinctions, failure modes, or asymptotic costs.

13. Declarative resource definition strictly dominates scattered registration. A computation or handler that requires coordination across registries must be one declaration from which the framework derives wiring, scheduling, and lifecycle.

14. Rollback exists only inside a declared reversible transaction and must derive from recorded inverse data without external reread. Escaping effects are typed as reversible, compensatable, reconcilable, or irreversible; the latter three require governed recovery or an explicit irreversible commit point and must never be mislabeled rollback.

15. API signatures must explicitly declare boundary crossings. Distributed sagas, disk flushes, complex traversals, and other orchestration boundaries must not masquerade as synchronous property access; API shape must force caller acknowledgment.

16. Phase progression must be compiler-visible and cost-ordered: rejection and eligibility precede expensive construction; each phase consumes the exact sealed proof type produced by its predecessor and emits the next proof plus any immutable batch summary shared downstream. Invalid, skipped, or out-of-order transitions are uncallable; later phases accept the strongest proven type and never re-prove it. Binary authority preconditions use witnesses mintable only by the granting authority.

17. Every reuse surface—caching, memoization, output suppression, or incremental recomputation—requires an explicit equivalence contract declaring identity basis, canonical dependency order, comparator, and invalidation basis. Reuse without contractual sameness is heuristic and may not claim correctness.

18. Authoritative and derived state are different runtime objects with different authority and lifecycle. Derived state must be reproducible from authority alone; if destroying all derived state prevents complete reconstruction, the system has promoted a cache into authority.

19. The framework owns every managed resource lifecycle. Computations, subscriptions, observers, cache entries, and projections must be registered, tracked, and disposable through framework authority; consumer-created invisible resources are forbidden.

20. Self-description grants no disclosure authority. Envelopes, sidecars, logs, journals, and checkpoints obey explicit audience, scope, classification, redaction, retention, deletion, and legal-hold policy; collect only contract-required data and never credentials. Redaction or expiry leaves typed omissions, derived copies cannot defeat deletion, and diagnostic paths may never widen authority.

21. Every durable or boundary-crossing artifact carries stable schema or protocol identity and version. Producers and consumers declare negotiation and compatibility windows; incompatible evolution requires deterministic provenance-preserving migration, coexistence and retirement rules, and downgrade posture. Never reinterpret old bytes; reject unsupported versions before effects with a typed migration or recovery outcome.

22. Every unbounded, asynchronous, or externally blocked operation declares deadlines, cancellation safe points, bounded queues, buffers, and concurrency, backpressure and overflow policy, resource budgets, and partial-effect posture. Cancellation is an outcome, not rollback. Exhaustion rejects before effects or enters named degradation that may reduce freshness, richness, or throughput but never correctness, authority, security, or durability; results state progress, retained or weakened guarantees, and recovery.

23. Architectural tradeoffs are governed by precedence, not weighted convenience. Authority and security determine which actions are admissible; semantic correctness must be preserved within that permitted set. If authority, security, and correctness cannot all be satisfied, the operation must fail closed and escalate rather than trading one away. Among admissible correct designs, recoverability dominates performance, and performance dominates compositional cleanliness. A lower-priority concern may shape implementation only after every higher-priority concern is satisfied.
