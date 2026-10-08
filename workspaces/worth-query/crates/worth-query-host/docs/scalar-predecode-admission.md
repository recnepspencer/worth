# Tracked scalar predecode admission

`primary_graph::DecisionReader::field_with_predecode_admission` lets an Entry
owner admit its work and allocation using the actual prior carrier before Query
dispatches its declared decoder. It uses the same operation read declaration,
identity authority, installed field and retained snapshot as `field`.

The callback has this shape:

```text
for<'raw, 'checkpoint> FnOnce(
    &'raw ApplicationValue,
    &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
) -> Result<(), Denied>
```

The method returns
`Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>`. The inner error
retains the owner's concrete admission refusal. The outer error preserves Query
execution failure; a request interruption remains recoverable through
`error.downcast::<HandlerInterruption>()`, distinguishing Cancelled from
DeadlineExceeded. As with `field`, lawful absence or binding decode rejection
returns `Ok(Ok(None))`. Absence does not invoke carrier admission.

## Loan and decoder ownership

Query first admits the declared decision target and retains the exact field
dependency, including absence. Its snapshot projection borrows the original
exact-root scalar. Admission runs inside that original record loan, before the
ordinary observation's scalar clone or the binding macro's carrier clone. On
successful admission Query invokes the same binding decoder used by `field`;
the value-binding macro's mandatory validation remains intact. There is no
caller-supplied alternative decoder and no restored numerical proof or graph
identity/current-output grant.

The raw scalar and request checkpoint cannot escape the callback. Owned copies
remain descriptive and require the caller's own work/storage admission. The
checkpoint borrows the same admitted request; it does not introduce a clock,
request or cancellation context. Query checks the request before admission,
before decoder dispatch and after completion, including a callback refusal.
Owners poll the supplied checkpoint during bounded preflight scans rather than
trying to reborrow their DecisionReader through a RefCell.

Entry owns its domain-specific preflight, decoder CPU, canonical validation,
carrier-copy and peak/retained storage admission. Use the original cumulative
ledger and actual prior carrier, rather than the proposed replacement intent.
Any callback scratch and decoder allocation must have an explicit lifecycle;
retained credit is released only after its complete owner drops. Query's
projection metadata and field-read granule are separate framework costs. A
length is not an allocator capacity, and a field-read count is not decoder work.

## Complexity contract registry

These contracts cover predecode staging, not full decoder/domain latency or
publication cost. The existing exact entity/field lookup and decision-fact
insertion still incur their existing metadata/index costs; staging does not scan
or copy the scalar payload. Callback and binding costs depend on their actual
inputs and remain separately admitted by the Entry owner.

| Contract | Named counter or evidence | Status and scope |
| --- | --- | --- |
| One observed present or absent field consumes one existing projection field-read granule. | `WorthQueryInvariantProjectionWork::field_reads()`; `large_carrier_denial_allocates_only_projection_metadata_before_decoder` and tracked-fact tests. | Verified for the installed focused decisions. |
| Denied admission performs no scalar/carrier clone and no binding dispatch; Query metadata is independent of carrier byte width. | The real installed 128 KiB label denial allocates less than one carrier; `predecode_denial_prevents_actual_declared_binding_dispatch` uses the actual binding macro and a decoder trap. | Verified staging boundary; no general allocation/latency claim for accepted decoders. |
| Admitted dispatch preserves codec rejection and mandatory binding-macro validation. | `accepted_admission_preserves_decode_rejection_and_mandatory_macro_validation`. | Verified binding dispatch boundary. |
| Presence and absence remain publication-time decision facts. | `present_predecode_field_stales_after_a_competing_actual_change`; `absence_skips_admission_and_stales_after_a_competing_presence_change`. | Verified with genuine competing installed publications. |

The focused installed tests also cover foreign identities, undeclared fields,
actual scalar contents and distinct request cancellation/deadline. The public
compile-fail example rejects retention of the raw loan. Neither these contracts
nor decoding establish producer acceptance, current output or a supplier budget
profile for a consuming family.

## Destination and review boundary

Query Execution owns the handler entry, admitted decision target, borrowed
observation and decoder dispatch under
`domain_computation::primary_graph::{handler::invariant::predecode_admission,
invariant_projection, application_attempt::observation}`. Host remains a leaf
audience facade; Relational's existing exact-root loan and declaration codecs
remain their existing owners. Future field-read strategies enter as children of
these responsibilities, not alternate graph readers or codec bypasses.

Review must preserve admission before any payload copy, original request
checkpoint custody, unchanged decoder semantics and exact decision dependencies.
An owner still has to remove its own accounting amplification or obtain a
coordinated supplier profile if its real work exceeds its existing budget.
