/// A caller-funded read preserves its request failure and its caller admission.
pub(crate) enum ReadStop<E> {
    Request(crate::execution::RelationalExecutionDenialCause),
    Admission(E),
}
