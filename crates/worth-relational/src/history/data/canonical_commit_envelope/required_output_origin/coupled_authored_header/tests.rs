//! A recorded edit-envelope stage list shorter than two refuses at the header.
use super::*;
use crate::tests::support::*;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};

fn validate(
    execution: &ExecutionRequest<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Stop> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| envelope.validate_output_origins(context).map(|_| ()),
            )
        })
        .unwrap()
        .unwrap()
        .0
}

#[test]
fn a_recorded_stage_list_shorter_than_two_refuses_before_any_reading() {
    // Start from a genuine recorded envelope and give it an edit-envelope
    // event whose stage list is empty or names only its prefix.
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture::default().build_runtime();
    let result = create_entity_outcome(execution, &runtime, "stage-source");
    let source = changed_entities(&result)[0];
    let mut envelope = result.publication().envelope.as_ref().clone();
    for sources in [Vec::new(), vec![source]] {
        envelope.coupled_authored_publication =
            Some(std::sync::Arc::new(CanonicalCoupledAuthoredPublication {
                sources: CanonicalCoupledAuthoredSources::Stages(sources.clone()),
                participants: Vec::new(),
                live_inputs: Default::default(),
            }));
        let stop = validate(execution, &envelope).expect_err("a short stage list is corrupt");
        assert!(
            matches!(&stop, Stop::Payload(detail) if detail == "coupled authored stage list is shorter than two"),
            "{sources:?}: {stop:?}"
        );
    }
}
