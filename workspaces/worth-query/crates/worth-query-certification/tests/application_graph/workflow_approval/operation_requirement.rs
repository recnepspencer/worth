use worth_query_host::facade::application_entry::RequiredWorkflowOperation;

pub(super) fn assert_same(
    warm: &RequiredWorkflowOperation,
    reconstructed: &RequiredWorkflowOperation,
) {
    assert_eq!(
        warm.transition_identity(),
        reconstructed.transition_identity()
    );
    assert_eq!(warm.input_identity(), reconstructed.input_identity());
    assert_eq!(warm.binding(), reconstructed.binding());
    assert_eq!(warm.branch(), reconstructed.branch());
    assert_eq!(warm.occurrence(), reconstructed.occurrence());
}
