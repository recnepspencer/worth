//! A distinct report purpose binds the exact performed comparison and admitted field.
use super::super::connection::ProducerExplanation;
use super::*;
pub(in super::super) fn producer_explanation_identity<Stop>(
    query: &super::super::report::InspectionReportPayload,
    source: &Arc<WorthQueryApplicationInspectionTarget>,
    session: &WorthQueryManagedGraphWorkSession,
    explanation: &ProducerExplanation,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    let ProducerExplanation::Evaluation(evaluation) = explanation else {
        let ProducerExplanation::Creation(connection) = explanation else {
            unreachable!()
        };
        return super::connected_identity(query, source, session, connection, admit);
    };
    let source = identities(source, session, InspectionOriginalReference::Source, admit)?;
    let field = evaluation.output_field();
    compose(
        "query.application-inspection.producer-output-evaluation.v1",
        &[
            Part::Text(query.report_identity()),
            Part::Text(source.observation.terminal_projection_for_reporting()),
            Part::Text(source.admission.terminal_projection_for_reporting()),
            Part::Number(evaluation.source_commit().commit_id.0),
            Part::Number(evaluation.source_commit().version_id.0),
            Part::Number(evaluation.producer_commit().commit_id.0),
            Part::Number(evaluation.producer_commit().version_id.0),
            Part::Number(evaluation.performed_basis().runtime_instance_id()),
            Part::Number(evaluation.performed_basis().root_identity()),
            Part::Text(evaluation.output_role()),
            Part::Text(field.entity()),
            Part::Text(field.aspect()),
            Part::Text(field.field()),
            Part::Digest(CanonicalDigestId::new(evaluation.field_meaning())),
            Part::Number(evaluation.observed_output_revision().version().0),
            Part::Text(evaluation.baseline().lowering_projection().label()),
            Part::Text(evaluation.baseline().signal_projection().label()),
            Part::Number(evaluation.baseline().attempt()),
            Part::Text(evaluation.native().lowering_projection().label()),
            Part::Text(evaluation.native().signal_projection().label()),
            Part::Number(evaluation.native().attempt()),
        ],
        admit,
    )
}
