#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordedSourceIdentity {
    Runtime(crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity),
    Checkpoint(crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity),
}

impl RecordedSourceIdentity {
    pub(super) fn matches_current(
        self,
        runtime: crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity,
        checkpoint: crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity,
    ) -> bool {
        match self {
            Self::Runtime(identity) => identity == runtime,
            Self::Checkpoint(identity) => identity == checkpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RecordedSourceIdentity;
    use {crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity, crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity};

    #[test]
    fn matching_never_crosses_runtime_and_checkpoint_lanes() {
        let shared_bytes = [0x11; 32];
        let other_bytes = [0x22; 32];

        assert!(
            !RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new(
                shared_bytes
            ),)
            .matches_current(
                WorthQueryRuntimeSourceIdentity::new(shared_bytes),
                WorthQueryCheckpointSourceIdentity::new(other_bytes),
            )
        );
        assert!(
            !RecordedSourceIdentity::Runtime(WorthQueryRuntimeSourceIdentity::new(shared_bytes),)
                .matches_current(
                    WorthQueryRuntimeSourceIdentity::new(other_bytes),
                    WorthQueryCheckpointSourceIdentity::new(shared_bytes),
                )
        );
    }
}
