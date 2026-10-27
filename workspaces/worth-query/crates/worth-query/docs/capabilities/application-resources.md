# Application resources

Applications declare a process-local request/response resource through
`worth-query-decl` and use it through `worth-query-host`. The installed provider
verifies its own issued attempt; Signal owns request, timeout, retry and
completion transitions. A transport acknowledgement or a copied completion
envelope cannot fulfill a resource.

Milestone 9.18.1 Phase 8 is accepted on 2026-10-27. Its
[ordinary application journeys](../../../worth-query-certification/tests/application_graph/source_only/capability/resources/journeys.rs)
and [owned-effect journey](../../../worth-query-certification/examples/observable_workflow/resource_monitor/journey.rs)
exercise the installed public API and its actual provider/cleanup owners.

The example's [source court](../../../worth-query-certification/examples/observable_workflow/resource_monitor/source_binding.rs)
forks a populated Product, drops an actual prepared program adoption, then
publishes on its sibling before starting the Resource. A closed State-admitted
instrument compares the retained Native source with both genuine reads; branch
versions and reserved occurrence counters cannot substitute for that source.

## Declare and install

Implement `ApplicationResourceBinding<Schema>` with an actual Access query
binding, readable scalar input/result/failure bindings and a portable provider
role. Access supplies principal and scope recipes; its ordinary query permission
and payload plan grant no resource permission. `scope` and `parameters` map the
input once at Start. Later actions reissue the funded original selector and
parameters without rerunning those mappers or retaining a cloned business input.

`ApplicationResourceDefinition::request_response` includes the retry/timeout
profile, independent Start/Inspect/Advance/Close requirements, disclosure
contract, maximum live requests, retained results, ingress slots, input bytes and
frame bytes. Installation compares the complete transported value profile with
the real typed inventory before any provider contact. The profile text cannot
construct a decoder or Native proof.

Configure the provider with the declaring contribution's
`setup.resource::<Binding, Provider, Clock, Source>(provider, clock_arc)`. All
resource pools in one runtime must share that exact clock Arc, clock/source
types, source identity and timeline. The complete resource pool batch is installed
last on the existing conditional graph and refuses a cell already containing raw
owned-async producers with unspecified tick units. Separate legacy runtimes keep
their existing units. Source coordinates are checked nanoseconds on the actual
resource cell, rather than a second managed-clock partition.

Provider heaps and external work remain the provider owner's responsibility.
The installed framework limits bound its rows, Native slots, retained outputs
and preadmitted ingress backing; they do not bound arbitrary memory a callback
retains in the provider.

## Act through a fresh request

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let resources = runtime.request(&actor, &scope).resources();
let mut resource = resources.start::<Assessment>(input)?;
let selector = resource.receipt();
resources.advance(&mut resource)?;
resources.close(&mut resource)?;
```

Start consumes typed input. A refused Start returns
`WorthQueryResourceStartStop::into_parts()` with the exact optional resource
handle and original cause. A row may already own accepted clock movement,
Native admission or a contacted provider. Retain that handle for a fresh Close;
the absence of a Native request does not erase an accepted clock outcome.

Every retained action freshly authenticates the actor and selects the original
branch's current Product and security. It resolves the complete original scope
identity, including generation, before protected row lookup, Busy, provider or
clock access. Birth principal is provenance, not a cached permission. A newly
authenticated actor needs the actual installed action and requested facets.

Availability, state, result, domain failure and cleanup detail have independent
disclosure slots. Governed methods consume a real admitted capability access and
refresh it on the same ResourceRead session as the requested action. Action
permission cannot expose a count or a payload. Plain Close requires Close alone;
it never inherits Inspect or result disclosure. `close_detailed` selects Close
plus CleanupDetail and retains those admitted booleans in the returned cleanup
evidence, including after a later control stop. Cleanup detail requires the
declared Count and Explanation influences.

`advance` uses the genuine Native timeout/retry policy. `revalidate` transfers the
same custodian through the actual World/source transition while preserving its
original business Product and input. Post-admission refusal leaves any actual
successor and pending wake cleanup in that same row. Before-due close uses the
existing exact retry-abort door; it does not advance time to manufacture cleanup.

## Provider ingress and completion

The provider receives an opaque issued attempt and an ingress writer backed by
finite storage admitted at installation. Neither retains a borrowed execution
request. Writes validate the exact issued attempt under the same synchronization
that closes/reuses the lane. Old writers cannot complete a reused Native slot.
Cancellation of the opening request does not silently cancel later provider
ingress.

Progression seals one immutable frame and copies its admitted length into
contiguous scratch funded by the fresh original request. It releases ingress
locks before the arbitrary verifier callback. The verifier authenticates its
own actual issued attempt and returns a descriptive frame kind and scalar.
The framework measures/admit-retains that returned scalar, checks installed
limits and invokes the sole real admitted typed decoder. This is not a promise
to preallocate arbitrary provider-owned callback heaps.

An unverified buffered frame is distinct from an admitted Native queue item.
Each admitted enqueue receives exactly one dequeue; rejected/unverified backing
is released separately. Verified values remain owned through retained-result
capacity refusal without rerunning the provider. Result capacity is admitted
before Native stage/commit. Only genuine completion commit makes an ordinary
result available. A committed result may still retain timeout/provider cleanup
after a later original-request control refusal.

The public receipt is a selector, not a Native handle, permission, snapshot pin
or cleanup capability. Its clone retains the original funded scope/parameter
association. The runtime row owns Native/provider/queue obligations. Public handle
Drop marks only its exact allocated generation abandoned. A retained receipt can
be used by `reclaim_governed` after fresh Close plus Availability admission;
there is no resource browse/index or replacement authority minted from fields.

Closing an old retained result cannot cancel a successor that reused its Native
slot. Close refusal preserves the same handle. If cleanup completes but a later
request handoff stops, `denial.into_closed()` returns the actual completed cleanup
evidence separately from the original stop.

## Recovery and shutdown

`start_for_owned_effect` consumes an actual `WorthQueryRecoveryHandle` and its
genuine handle-bound inspect authority. Entry refusal returns any still-external
handle; an admitted row keeps it until authorized
`close_for_effect_recovery` returns that exact handle after resource cleanup.
Ordinary Close and abandoned-row handling cannot discard it.

The actual current terminal/outcome owner is checked before the original
committed outbox is read. That retained outbox proves original correlation and
commit provenance; it is not current effect status. The current check runs again
after arbitrary verification immediately before indeterminate admission.
Authenticated effects-indeterminate uses its genuine issuer and remains separate
from ordinary result commit. A timeout never implies external rollback and never
authorizes automatic redispatch. The recovery example's declared preimage
contract admits ownership/inspection; it does not certify or execute a corrector.

The existing `close_conditional_runtime(&mut self)` refuses while any installed
resource row is allocated, including Busy, abandoned and recovery rows. Freshly
close each exact occurrence first. Successful explicit runtime close clears the
installed resource registry, so old adapters cannot restart the pool.

This contract is process-local. Dropping all selectors leaves the bounded row
occupied; it does not authorize discovery or provider cleanup. Runtime Drop or
process termination cannot return a refused owner and makes no Native/provider
cleanup claim. A new runtime refuses an old selector's original branch through
the real typed selection boundary before provider, clock or row contact. It does
not reconstruct custody from a receipt, slot ordinal or display digest. Missing
original scope similarly retains the old obligation rather than assigning it to
a same-name successor.
