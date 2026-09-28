# Developer Experience Laws

Developer experience is caller-visible architecture. These laws govern what a public surface must make obvious, possible, difficult, and recoverable; the other coding guidelines govern the internal structure that produces those properties.

1. Good DX is organized truth through progressive disclosure. The common path reads as semantic intent, the advanced path exposes the next responsible layer, and unsafe or weakened paths state their lost guarantees. Friendly syntax may compress ceremony but never cost, authority, consistency, failure, or lifecycle.

~~~ts
await users.rename(userId, { displayName: "Spencer" });

const plan = users.rename
  .intent({ userId, displayName: "Spencer" })
  .compile();

plan.explain();
await executor.run(plan);
~~~

2. The default path must carry the strongest ordinary guarantees, bounded resource behavior, and narrowest effect scope selectable without caller knowledge. Global, irreversible, weak-consistency, unbounded, or authority-widening behavior requires an explicit typed choice and must never be easier to invoke accidentally than the safe path.

3. Every boundary where caller responsibility changes must expose the controls only the caller can own: scope, deadline, cancellation, idempotency, retry tolerance, consistency, durability, artifact policy, and execution substrate. Global scope and guarantee weakening require explicit typed values; portable intent does not erase substrate-specific semantics.

~~~ts
await remote.execute(intent, {
  scope: Workspace(workspaceId),
  idempotencyKey,
  deadline: Duration.seconds(3),
  retry: RetryPolicy.exponential({ maxAttempts: 3 }),
  consistency: ReadYourWrites,
  signal: abortController.signal,
  artifactPolicy: MinimalArtifacts,
});
~~~

4. Long-running, streaming, subscribed, or potentially unbounded work returns a framework-owned handle exposing progress, partial materialization, checkpoints, cancellation, backpressure, overflow, recovery, finalization, and disposal. The framework must be able to enumerate and terminate every managed resource; fire-and-forget is not a lifecycle.

~~~ts
const job = await indexes.rebuild({ scope: Workspace(workspaceId) });

for await (const event of job.events()) {
  renderProgress(event);
}

await job.cancel();
await job.recover();
await app.lifecycle.dispose(job);
~~~

5. One declarative definition owns a semantic unit; explicit registration owns its membership in an application. The framework derives wiring, validation, scheduling, lifecycle, and discovery from the definition. Filesystem scanning, reflection, bundler behavior, and naming conventions may assist discovery but must remain inspectable and replaceable.

~~~ts
const UserModule = defineModule({
  model: User,
  routes: UserRoutes,
  permissions: UserPermissions,
  workflows: UserWorkflows,
});

app.register(UserModule);
~~~

6. Definition functions establish semantic identity, inference, validation, and inspectability. Object specifications encode simultaneously visible shape; builders encode ordered accumulation of constraints, proofs, or phases. A surface that neither reveals structure nor enforces progression is ceremony.

7. Canonical definitions carry stable namespace, identity, and version, derive ordinary projections without imprisoning specialized views, and remain interpretable across evolution. Overrides may specialize derived presentation without forking truth. Persisted plans, jobs, workflows, records, SDKs, and clients require explicit compatibility windows and deterministic migrations.

8. The type system is part of the interface. Capabilities must be discoverable from semantic roots; valid next actions appear in autocomplete, invalid transitions are unavailable, and declared names become typed references. Policy, authority, lifecycle, phase, mode, and semantic context use typed values rather than strings, positional booleans, flat option bags, globals, or ambient state.

9. Semantic intent is a first-class value that can be validated, authorized, planned, simulated, queued, retried, audited, migrated, replayed, and explained. Friendly authoring surfaces lower into an inspectable plan containing resolved strategy, policy, locality, consistency, artifact policy, concurrency footprint, effects, and required capabilities before execution.

10. Plans expose proof-carrying phase progression, resolved policy, and structural footprint. Each phase returns exactly the proof needed for the next; applicability is solved before execution; read, write, invalidation, conflict, and parallel-admission sets are inspectable. The executor consumes these decisions rather than recreating them.

11. Public outcomes distinguish success, advisory, violation, partial completion, conflict, cancellation, timeout, dependency failure, indeterminate external state, and internal defect through typed variants with machine-readable context. Indeterminate outcomes carry recovery handles. Exceptions are reserved for programmer defects or unrecoverable runtime failure.

~~~ts
type ExecutionResult<T> =
  | Succeeded<T>
  | Advised<T, AdvisoryContext>
  | Rejected<ViolationContext>
  | Partial<T, RecoveryHandle>
  | Cancelled<CancellationContext>
  | TimedOut<DeadlineContext>
  | Failed<FailureTopology>
  | Indeterminate<RecoveryHandle>;

if (result.kind === "indeterminate") {
  await result.recovery.inspect();
}
~~~

12. Approval, escalation, deferral, rejection, emergency override, and guarantee weakening are governed state transitions carrying authority, scope, reason, expiration, and audit policy—not booleans, comments, or side channels.

13. Reads are planned contracts, not casual getters. Queries and projections declare dependencies, consistency, invalidation, and observation semantics; authoritative, derived, cached, stale, speculative, pending, and committed values are distinct API objects. A caller must never infer truth status from convention.

~~~ts
const observation = await app.query(UserProfile, {
  userId,
  consistency: Committed,
});

observation.committed;
observation.dependencies;
observation.invalidation;
~~~

14. Every boundary returns a self-describing canonical result and an explanation surface at the caller's semantic level. Policy-controlled diagnostic sidecars may materialize traces, warnings, effects, cost, and boundary detail without polluting domain truth or the hot path. Explanation answers the caller's question; logging does not substitute for it.

15. Definitions, validation, policy, plans, effects, and failure topology must be testable before external execution. Simulation predicts governed effects without commitment; replay reproduces prior decisions from versioned definitions, context, authority, clock, and recorded external observations.

16. Physical, financial, geometric, temporal, and numerical APIs encode units, precision, tolerance, rounding, and reference frames wherever confusion could silently change meaning.

17. Invalid usage must fail at the earliest boundary capable of explaining it. Compiler and runtime errors identify the attempted action, current phase or state, violated contract, required authority or input, valid next actions, and a stable remediation path. An error that merely reports internal mechanism has leaked implementation instead of teaching the caller.

18. Public examples, snippets, generated clients, migration guides, and documented call sequences are executable compatibility contracts. They must compile or run in CI against the real public facade. Documentation that can drift independently from the API is not a DX surface.

19. Composing operations into batches, workflows, retries, queues, transactions, or higher-level intents must preserve scope, cancellation, idempotency, policy, proof, failure topology, effects, and recovery. Composition may aggregate these contracts but must not erase them.

20. Deprecation must be mechanically discoverable and carry its replacement, semantic difference, migration operation, compatibility deadline, and changed guarantees. A deprecated capability without a guided path to its successor exports maintenance work to every caller.

21. A caller must find capabilities from domain identity, intent, or current proof state without knowing internal modules, registry keys, transport names, generated paths, or string identifiers. The final DX test is whether authoring, reading, debugging, testing, migration, recovery, and operation remain possible without architectural archaeology while the runtime can still prove what happened.
