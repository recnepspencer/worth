//! Original artifact attribution is installed once before positioned emission.
use super::CanonicalCommitEnvelope;
use crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs;

impl CanonicalCommitEnvelope {
    pub(crate) fn recorded_original_inputs(&self) -> Option<&RecordedCanonicalInputs> {
        self.original_inputs.get().and_then(Option::as_ref)
    }

    /// Explicit standalone None is sealed too: a prepared DATA commitment
    /// cannot subsequently acquire an input attribution through another Arc.
    pub(crate) fn seal_recorded_original_inputs(&self) {
        self.original_inputs.get_or_init(|| None);
    }

    pub(crate) fn retain_recorded_original_inputs(
        &self,
        inputs: RecordedCanonicalInputs,
    ) -> Result<(), &'static str> {
        if let Some(original) = self.original_inputs.get() {
            return (original.as_ref() == Some(&inputs))
                .then_some(())
                .ok_or("canonical original input attribution changed");
        }
        if let Err(inputs) = self.original_inputs.set(Some(inputs)) {
            return (self.original_inputs.get() == Some(&inputs))
                .then_some(())
                .ok_or("canonical original input attribution changed concurrently");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durability::canonical_commit_artifact::{
        PreparedOriginalCanonicalPayload, RelationalCanonicalCommitArtifact as Artifact,
    };
    use crate::history::data::PositionedCanonicalCommit;
    use crate::publication::patch::data::PatchStreamPosition;
    use crate::tests::support::*;
    use std::sync::Arc;
    use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};

    #[test]
    fn prepared_memory_frame_preserves_position_widths_and_seals_absent_input_attribution() {
        let host = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let committed = create_entity_outcome(&request, &runtime, "sealed-original-payload");
        let source = committed.publication().envelope.clone().into_arc();
        let inputs = source.recorded_original_inputs().unwrap().clone();
        let mut descriptive = source.copy_original_for_test(&request);
        // This is the explicit standalone None codec subject, not Home evidence.
        descriptive.original_inputs = Default::default();
        let descriptive = Arc::new(descriptive);
        let prepared =
            PreparedOriginalCanonicalPayload::capture(descriptive.clone(), &request).unwrap();
        assert!(Arc::ptr_eq(prepared.envelope(), &descriptive));
        assert!(descriptive.retain_recorded_original_inputs(inputs).is_err());
        assert_eq!(descriptive.recorded_original_inputs(), None);
        for position in [
            1,
            127,
            128,
            255,
            256,
            65_535,
            65_536,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
            u64::MAX,
        ] {
            let position = PatchStreamPosition(position);
            let actual = Artifact::from_memory_publication(position, &prepared, &request).unwrap();
            let reference = Artifact::from_positioned(
                &PositionedCanonicalCommit::for_test(position, descriptive.clone()),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
            assert_eq!(actual.as_bytes(), reference.as_bytes());
            Artifact::from_untrusted_bytes(actual.as_bytes()).unwrap();
        }
        release_test_commit_snapshot(&runtime, &committed);
    }
}
