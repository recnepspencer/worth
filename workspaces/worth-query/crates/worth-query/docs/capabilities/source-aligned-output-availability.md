# Source-aligned output availability

## What This Feature Is

`output_availability` observes an installed exactly-one producer family inside
an existing decision or checked-computation reader. It returns a native current
output or native source-aligned Pending without starting that producer.
This surface is being qualified; the native consumer proof is not yet complete.

## Why You Use It

A population may have a complete count while one member awaits regenerated
measurement. Observe that member in the same computation attempt that publishes
the population's per-measure result. Keep Pending distinct from a complete zero,
known empty, unavailable measurement and numerical indeterminacy.

## Stable Entry Points

The candidate public methods are `DecisionReader::output_availability` and
`WorthQueryComputationReader::output_availability`, through
`worth_query_host::facade`. The lower operation reader owns their implementation.
They accept an actual native producer identity and a typed output demand. The
demand authors selection; it grants no source, role or publication authority.

## Core Mental Model

Query freshly admits the demand's installed source Query under this reader's
original request, principal, selected occurrence, schema and program. Native
source qualification and complete installed publication coverage issue the
first-output classification. A missing head alone issues nothing.

`WorthQueryObservedOutputAvailability` has private construction. Its `state()` is
`WorthQueryOutputAvailabilityState::Current(native_output)` or `Pending`.
Missing, retired, foreign, ambiguous or unqualified sources are refusals.
The family retains its existing exactly-one output role and lifecycle contract.

## How It Executes

The completed source Query's session, authorization and dependency facts remain
in the original reader. Query records its native source-publication dependency,
validates it before publication, and binds the observed native companion image
to the real prepared commit. A racing completion refuses that attempt. Separate
upstream completion wakes previously published Pending, including an accepted
equal-input completion with no new business commit.

The method never progresses the upstream demand. Normal demand advancement and
ordinary `current_output` retain their existing behavior.

An explicit upstream demand prepares current canonical input, resources and its
idempotency key under fresh authorization before input cutoff. Accepted stable
reuse skips handler preparation, gather and kernels, with zero native execution
contacts and no new producer commit receipt. Provider admission callbacks and
their work remain separate from those execution contacts.

A normal advance may also progress other live required consumers after settling
its caller. For an isolated no-business-commit witness, `close_for_delivery`
releases the original complete tree's work interests while retaining its native
readonly currentness and historical delivery. A refused owner kept for retry
still owns live work; its upstream settlement alone does not describe every
commit in that call.

## Small Example

Inside a checked computation's gather, with declared Body and Length reads:

```rust
let observed = reader.output_availability(&member.entity, PlanarOutputDemand::new(key))?;
```

Here `member.entity` is the actual native member read from collection membership;
`key` is selection input read in this same reader. A key mismatch refuses.

## Real Example

The compiling counterpart is
`fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/output_availability/collection.rs`.
Its one installed owner serves Initial/Create and Preserve/Preserve bindings of
one operation. The gather retains native contributor identity and reads Length
only from the returned current output:

```rust
let (measure, output) = match observed.state() {
    WorthQueryOutputAvailabilityState::Pending => (Measure::Pending, None),
    WorthQueryOutputAvailabilityState::Current(identity) => {
        let value = reader.field(identity, Length::reference())?
            .ok_or(WorthQueryComputationInputDenial::Owner(3))?;
        (Measure::Complete(PositiveLength::get(&value)), Some(members::descriptor(identity)))
    }
};
```

Count includes every actual member. Any Pending member keeps the aggregate
measure Pending; a subtotal cannot become complete. Descriptive contributor
identities in the result never authorize later reads or publication.

## How It Relates To Other Features

Use the same installed checked collection and native output owner for renewal.
A held read stays historical. Ask the original owner for current delivery, and
retain original custody on refusal, retry and close. This observation adds no
watch registry, scheduler, retained-owner wire format or recovery authority.

## Inspection And Debugging

An observation error retains its original Query, current-output or native
verification refusal. `kind()` preserves request/resource stop posture; the
Query admission/execution getters expose their actual attached cause. Errors
are not availability results and must not be caught and published as Pending.

## Anti-Patterns

Do not infer Pending from `current_output` denial, absent publication rows,
copied receipts, keys or cached measurement. Do not run a separate validation
attempt and copy its conclusion into a tracked computation. Do not replace a
refusal or unknown contributor with a smaller complete group.

## Current Limits

Only the original producer execution supplies the source Query loan. Invoking
a handler as an ordinary mutation supplies no such context and refuses; a
caller cannot enable it with a flag or copied observation.

The observation performs its complete prerequisite source/authorization and
native currentness work on the original admission. It provides no sparse-cost
exemption. Unsupported retained-fact continuity requires funded full validation.

Live completion facts and native image proofs are not serializable. Restored
publication coverage is invalidated and must be reestablished by its native
restore owner before this route can classify headless sources. Fresh-on-restore
qualification remains separate. No old-data defaults or converters are provided.

## Related Docs

- [Build an application](../../../../../../docs/build-an-application.md)
- [Capability index](../../../../../../docs/capability-index.md)
