/// A caller-funded read preserves its request failure and its caller admission.
pub enum ReadStop<E> {
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    Admission(E),
}
