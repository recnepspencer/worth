pub(crate) use {
    worth_store_recovery_runtime::PlannedPhysicalRecovery,
    worth_store_recovery_runtime::RecoveryPublicationAction,
};

pub(crate) fn assert_exact_publication_plan(planned: &PlannedPhysicalRecovery) {
    let publication = planned.publication_plan();
    assert!(!publication.candidates().is_empty());
    assert_eq!(
        publication.actions().len(),
        publication.candidates().len() * 2 + 2
    );
    for (candidate, actions) in publication
        .candidates()
        .iter()
        .zip(publication.actions().chunks_exact(2))
    {
        assert_eq!(
            actions,
            [
                worth_store_recovery_runtime::RecoveryPublicationAction::MaterializeRootCandidate {
                    artifact: candidate.artifact(),
                },
                worth_store_recovery_runtime::RecoveryPublicationAction::SynchronizeRootCandidate {
                    artifact: candidate.artifact(),
                },
            ]
        );
    }
    assert_eq!(
        &publication.actions()[publication.actions().len() - 2..],
        [
            worth_store_recovery_runtime::RecoveryPublicationAction::ReplaceRootProtocol,
            worth_store_recovery_runtime::RecoveryPublicationAction::SynchronizeStoreNamespace,
        ]
    );
}
