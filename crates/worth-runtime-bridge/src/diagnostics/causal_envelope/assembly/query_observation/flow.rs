//! Prepare the funded descriptor, then enter the common assembly orchestrator.
use super::super::super::BridgeCausalEvidenceReference;
use super::super::{BridgeCausalEnvelopeAssemblyRequest, BridgeCausalInspectionAdmissionSummary};
use super::identity::{compose, ReportIdentityKind as Kind, ReportPart as Part};
use super::*;
use std::sync::Arc;

pub(super) fn assemble<Stop, Admit>(
    input: BridgeQueryObservationReportingRequest<'_>,
    admit: &mut Admit,
) -> Result<BridgeCausalExplanationEnvelope, BridgeQueryObservationReportingStop<Stop>>
where
    Admit: FnMut(usize, usize) -> Result<(), Stop>,
{
    storage::admit_containers(admit)?;
    let summary_identity = compose(
        Kind::AdmissionSummary,
        &[
            Part::Shape("query-observation-reporting"),
            Part::Shape(input.kind.as_str()),
            Part::Evidence(input.admission),
            Part::Evidence(input.anchor),
            Part::Evidence(input.observation),
        ],
        admit,
    )?;
    let summary = BridgeCausalInspectionAdmissionSummary::query_observation_reporting(
        input.kind,
        input.admission,
        input.anchor,
        input.observation,
        summary_identity,
    );
    let reference_identity = compose(
        Kind::EvidenceReference,
        &[
            Part::Shape("query"),
            Part::Shape("query_observation"),
            Part::Evidence(input.observation),
        ],
        admit,
    )?;
    let reference = BridgeCausalEvidenceReference::query_observation_reporting(
        input.observation,
        reference_identity,
    );
    let request_identity = compose(
        Kind::AssemblyRequest,
        &[
            Part::Evidence(summary.summary_evidence_identity()),
            Part::Evidence(reference.reference_digest_evidence_identity()),
        ],
        admit,
    )?;
    let request = BridgeCausalEnvelopeAssemblyRequest::query_observation_reporting(
        summary,
        Arc::from(vec![reference]),
        request_identity,
    );
    super::super::assemble(
        request,
        source::QueryObservationReportingSource {
            observation: input.observation,
            admit,
        },
    )
}
