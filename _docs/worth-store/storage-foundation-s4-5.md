# Storage Foundation S.4.5: Physical Simulation And Fault Harness

## Goal

Provide one deterministic, replayable harness for physical storage behavior.
The harness must drive real Store boundaries, preserve authority and security
scope, inject faults at named production yieldpoints, and let independent
oracles judge observed behavior.

This milestone is infrastructure for finding storage defects. It is not a
reporting system and it does not certify itself.

## Governing Decisions

- Scenario definitions are inert data. They cannot mutate runtime state or
  authorize Store operations.
- Lowering validates capabilities, budget, topology, and security scope before
  execution.
- Execution consumes a typed plan and deterministic schedule. Executors do not
  reinterpret scenario intent.
- Faults enter through named production boundaries. Private-state mutation,
  copied terminal fields, fixture labels, and loose logs are not valid fault
  delivery or oracle inputs.
- Observers record facts. Oracles judge those facts independently of the code
  that produced them.
- Replay identity binds the scenario, plan, schedule, fault events,
  observations, oracle verdicts, and counters used by that replay.
- Simulation results never become runtime authority.
- Expensive scenarios are profile-gated; the local lane remains deterministic
  and fast enough for ordinary iteration.

## Required Topology

- `worth-store-physical-certification` owns scenario vocabulary, planning,
  scheduling, fault delivery, observation, replay, and physical oracles.
- Production crates own the yieldpoints and typed observations at the
  boundaries they implement.
- `worth-store-test-support` owns reusable fixtures and process helpers, not
  verdicts.
- `worth-store-certification` may compose direct courtroom scenarios but must
  not wrap replay results in another certification protocol.
- Foundational vocabulary may describe identities and boundary facts.
  `worth-proof` may represent real phase progression. Neither substitutes for
  executed Store behavior.

## Core Contracts

### Scenario and plan

A scenario names its actors, intended operations, required capabilities,
faults, oracle families, counter expectations, security scope, and resource
profile. Lowering either returns a typed `PhysicalSimulationPlan` or a typed
denial. The plan owns the exact execution identity.

### Schedule and fault delivery

The deterministic scheduler records actor order and yieldpoint decisions.
Fault delivery is admitted only at a matching named boundary. A fault request
that does not match the plan, phase, actor, or target is denied before the
operation runs.

### Observation and replay

The replay transcript contains actual observations from the run. A replay can
be repeated from its bound inputs and compared directly with another run.
Terminal JSON and human-readable logs are diagnostics only.

### Oracles and counters

Oracles consume observations, not expected fixture labels. Counters are exact
where the behavior has an exact structural count and bounded where the
contract is an envelope. Tests assert those values directly.

## Phase Plan

1. Define scenario, actor, fault, oracle, counter, and profile vocabulary.
2. Lower inert scenarios into capability- and budget-checked plans.
3. Execute deterministic schedules through named production yieldpoints.
4. Record typed observations and replayable transcripts.
5. Implement independent physical oracles and adversarial denial cases.
6. Add direct S.4 recovery scenarios and fresh-process replay support.
7. Add direct S.5 physical-isolation scenarios and extension points.
8. Add bounded I/O-pressure and blob-scale profiles used by later milestones.

Each phase ends when its production code and direct tests pass. Phase history
is the reviewed Git revision; current validity is the current build and test
result.

## Required Direct Tests

- same inputs produce the same plan, schedule, transcript identity, verdicts,
  and counters;
- changed actor order, fault target, capability, security scope, or budget is
  either reflected in replay identity or rejected;
- real recovery, compaction, checkpoint, stable-read, security-scope, I/O
  pressure, and blob boundaries are exercised by their owning scenarios;
- missing observations prevent the relevant oracle from passing;
- private mutation, fixture-label verdicts, copied transcript fields, loose
  logs, and terminal-only projections cannot satisfy replay;
- counters come from execution and vary when executed work varies;
- local and CI profiles preserve semantic topology while changing only their
  admitted resource envelope;
- compile-fail tests protect authority and crate-direction boundaries that
  runtime tests cannot express.

## Prohibited Machinery

Do not add manually maintained requirement ledgers, per-source closure maps,
generated inventories of tests, coverage-row registries, maturity ladders,
adjudication protocols, reviewer receipts, shortcut-rejection reports, or
tests whose subject is another test. Do not hash individual source files to
recreate Git inside the harness.

If a behavioral guarantee matters, protect it with the narrowest direct test
at the owning boundary. If a structural rule matters, enforce it with the
compiler or workspace boundary checker.

## Completion Rule

The milestone is complete at a reviewed revision when focused owner tests,
affected integration tests, deterministic replay tests, boundary checks,
formatting, and line-cap checks pass. Later changes do not rewrite that
historical result; they must pass the current direct checks.
