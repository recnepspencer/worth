# Boundary denial reports

Implement `worth_execution::report::Reportable` on the error at the boundary
that owns the denial. Use `DenialClass` to say what failed, `Disposition` to
describe the recommended response, and `ReportCode` for a stable namespaced
machine identifier. Set a `RetryHint` when the caller has a specific retry
condition. The caller still obtains any required authority and checks the
underlying operation's retry contract.

Preserve causal errors with `Error::source()`. Call `to_operator_report` on a
`&dyn Reportable`, or import `ReportProjection` to call it on a concrete error.
The projection creates a read-only `OperatorReport` with frames ordered from
immediate cause to root cause. A cyclic source chain yields a typed omission.
Equality and `to_canonical_json()`
describe the disclosed snapshot. Keep correlation IDs, timestamps, and
sequence counters in the sink's surrounding event metadata.

Select `DiagnosticDetail` at the receiving audience boundary under
[Architecture 20](arch_laws.md#the-numbered-laws). `Redacted` includes a fixed
summary and preserves chain depth. `Standard` includes the owner's audience-safe
static summary. `Forensic` also includes deterministic text explicitly selected
by the owner's forensic hooks. The projection records withheld or unavailable
detail in each frame. Use allowlisted fields in those hooks and exclude
credentials and occurrence metadata. Foreign source text requires explicit
selection and review before disclosure.

Inject a `DiagnosticSink` at the emitting boundary and pass it the report after
applying disclosure policy. The sink's owner applies audience routing and
retention. Use `NoopDiagnosticSink` when emission is disabled and
`RecordingDiagnosticSink` for tests or finite captures, draining captured reports
with `take_reports()`.
