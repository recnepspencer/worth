#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RecordedSourceIdentity {
    Runtime(super::super::application_query::WorthQueryRuntimeSourceIdentity),
    Checkpoint(super::super::application_query::WorthQueryCheckpointSourceIdentity),
}

impl RecordedSourceIdentity {
    pub(super) fn matches_current(
        self,
        runtime: super::super::application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: super::super::application_query::WorthQueryCheckpointSourceIdentity,
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
    use crate::domain_computation::primary_graph::application_query::{
        WorthQueryCheckpointSourceIdentity, WorthQueryRuntimeSourceIdentity,
    };

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
