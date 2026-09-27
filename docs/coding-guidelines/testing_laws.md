# Testing Evidence Laws

> Tests protect production behavior. Code review decides whether the selected
> evidence is adequate for the requirement and risk. Tests do not need a second
> evidence system that certifies the tests themselves.

Use these laws with [QA Review Guide](qa_review_guide.md).

## Evidence

1. Every test or coherent parameterized family protects a named production
   behavior or regression. Test count, coverage, runtime, and assertion count
   are not evidence by themselves.

2. Choose the narrowest boundary that can honestly protect the behavior:
   product journeys for end-to-end claims, real subsystem boundaries for
   integration claims, property or model tests for broad semantic spaces, and
   focused tests for genuinely local semantics.

3. Expected results must be independent enough to expose the disputed defect.
   A test must also distinguish the intended behavior from relevant setup,
   routing, rejection, partial-effect, cleanup, and observation failures.

4. Sensitivity checks are proportional to risk. Negative twins, fault
   injection, and mutation probes are useful for high-risk or ambiguous tests,
   but are not mandatory when sensitivity is already clear.

5. Each test or parameterized family should contribute distinct evidence.
   Consolidate or remove cases whose deletion loses no meaningful protection.

## Fixtures and worlds

6. Fixture provenance must be honest when identity, authority, persistence,
   revision, or validity is material to the behavior under test. Irrelevant
   local values do not require production issuance for ceremony.

7. A fixture or world compiler may bypass irrelevant workflow history when it
   preserves the production invariants relevant to the claim. Review its
   boundary and cover meaningful behavior with ordinary focused tests; do not
   recursively certify it.

8. Do not counterfeit authority or validity with arbitrary identifiers,
   tokens, revisions, or timestamps when those properties are under test.
   Literals remain appropriate when validity is irrelevant or the literal is
   itself the subject.

9. Reuse expensive setup without shared mutable fate. Avoid ordering
   dependencies, cross-test state, clock leakage, identifier collision, and
   cleanup dependence.

10. World construction, action, observation, and teardown must be separately
    diagnosable when confusion could create a wrong-reason green result. Fixtures
    obey production privacy, authority, retention, and secret-handling rules.

## Test forms

11. An end-to-end test begins at a real product entry surface and observes a
    real consequence through the production composition root. Replacing a
    boundary relevant to the claim narrows what the test can claim.

12. Integration tests cross real semantic, authority, persistence, protocol, or
    lifecycle boundaries. Calling several in-memory objects is not integration
    merely because several types participate.

13. Focused tests are appropriate for parsers, algorithms, state machines,
    canonicalizers, numerical invariants, and closed transition tables. Avoid
    tests that only mirror private branches, assignments, getters, or incidental
    call order.

14. Mocks, fakes, and stubs prove caller behavior against the substitute. They
    do not certify the real adapter, external system, persistence, timing, or
    failure topology.

15. Compile-pass and compile-fail tests are for valuable public compile-time
    guarantees. Include a valid counterpart, fail for the intended boundary,
    avoid incidental compiler prose, and consolidate compiler sessions.

16. Snapshot and golden tests are appropriate when the artifact itself is the
    contract. Normalize only declared nondeterminism and avoid snapshots of
    incidental or mostly irrelevant structures.

17. Stateful and asynchronous systems need adversarial coverage proportional
    to their real failure model. Reviewers select the relevant cancellation,
    exhaustion, ordering, concurrency, partial-effect, recovery, migration, and
    irreversible-commit cases; the full list is not automatically required.

## Cost and integrity

18. Test topology is performance architecture. In Rust, group scenario families
    into a small number of intentional integration targets and avoid duplicated
    compiler sessions and dependency graphs.

19. Assign tests to focused, CI, or scheduled lanes. Expensive suites need a
    runtime or resource budget. Run cheap discovery, configuration, fixture,
    exact-name, and harness checks before expensive worlds.

20. Test configuration must not weaken validation, change production semantics,
    create authority, or introduce a composition root unavailable to the real
    system. Narrow test support is acceptable when it preserves production
    behavior and cannot bypass the behavior under test.

21. Flakiness is a product or harness defect. Quarantine removes the test's
    protection and needs an owner, reason, and bounded follow-up. Retries and
    widened timeouts must not manufacture green status.

22. Performance tests name the boundary, workload, scale axes, environment,
    cold or warm posture, and useful measurements. A generous timeout alone is
    not a performance contract.

23. Test and fixture code obey production composition and domain structure.
    Remove obsolete scenarios, redundant assertions, stale snapshots, unused
    builders, and unnecessary harness capabilities.

## Review boundary

24. Code review is the authority for evidence adequacy. Reviewers compare the
    specification, implementation, tests, and residual risk.

25. Do not create proof ledgers, evidence-authentication layers, source
    fingerprints, mutation receipts, or tests for tests merely to guarantee
    accepted tests. Test infrastructure may have ordinary tests for meaningful
    behavior.

26. Source control contains specifications, tests, and configuration. CI
    records results against a commit. Generated run journals and execution
    envelopes are not normally written back into source.

27. Completed historical phases remain historical. A later regression blocks
    the current change through its tests; it does not reopen earlier portfolios.

28. Stop when the affected behavior has appropriate tests, required checks pass
    on the final change, review finds the evidence adequate, and no known
    material defect remains. Residual risk is an engineering judgment, not a
    reason to build recursive proof machinery.

## Named failure modes

- **Counterfeit world:** Relevant validity is asserted through arbitrary values.
- **Self-certifying oracle:** Expected results repeat the disputed semantics.
- **Wrong-reason green:** Setup, routing, or observation fails first.
- **Boundary cosplay:** A test replaces the boundary its label claims to prove.
- **Compile-contract inflation:** Compiler sessions police private shape.
- **Coverage theatre:** Counts or snapshots replace a credible regression.
- **Test-only architecture:** Test configuration changes production semantics.
- **Sedimentary testing:** Tests accumulate without distinct value or deletion.
- **Recursive certification:** Evidence machinery mainly certifies other
  evidence machinery.
