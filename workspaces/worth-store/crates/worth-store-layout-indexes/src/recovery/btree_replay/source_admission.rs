use super::{denial::BTreeReplayDenied, request::BTreeReplayRequest};

pub(super) fn admit(
    request: &BTreeReplayRequest<'_>,
) -> Result<super::AdmittedBTreeReplayPhysicalSource, BTreeReplayDenied> {
    crate::btree_replay_runtime()
        .admit_physical_source(
            request.physical_source.readiness.clone(),
            request.physical_source.root_reference,
            request.physical_source.replay_artifact.clone(),
            request.physical_source.expected_store_identity.clone(),
            request.physical_source.durable_source.clone(),
        )
        .map_err(|denial| BTreeReplayDenied::Execution(Box::new(denial)))
}
