//! Reporting strategy supplies no conditional observation, Native reader or Route.
use super::super::super::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeIdentity, BridgeCausalEnvelopeReceipt,
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceFamily as Family,
    BridgeCausalEvidenceOwner as Owner, BridgeCausalEvidenceReference,
};
use super::super::{
    progress::BridgeCausalEnvelopeAssemblyProgress, source::CausalEnvelopeAssemblySource,
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalExplanationEnvelope,
};
use super::identity::{
    compose, BridgeQueryObservationReportingStop as Failure, ReportIdentityKind as Kind,
    ReportPart as Part,
};
use crate::identity::BridgeIdentityEvidence;

pub(super) struct QueryObservationReportingSource<'a, Admit> {
    pub(super) observation: &'a BridgeIdentityEvidence,
    pub(super) admit: &'a mut Admit,
}

impl<Stop, Admit: FnMut(usize, usize) -> Result<(), Stop>> CausalEnvelopeAssemblySource
    for QueryObservationReportingSource<'_, Admit>
{
    type Stop = Failure<Stop>;

    fn prepare_progress(
        &mut self,
        request: &BridgeCausalEnvelopeAssemblyRequest,
    ) -> Result<BridgeCausalEnvelopeAssemblyProgress, Self::Stop> {
        if !request
            .admission_summary()
            .requires_application_inspection_admission()
            || request.references().len() != 1
        {
            return Err(Failure::InvalidReportingAssociation);
        }
        Ok(
            BridgeCausalEnvelopeAssemblyProgress::query_observation_reporting(
                request,
                Vec::with_capacity(1),
            ),
        )
    }

    fn bind_reference(
        &mut self,
        progress: &mut BridgeCausalEnvelopeAssemblyProgress,
        reference: &BridgeCausalEvidenceReference,
    ) -> Result<(), Self::Stop> {
        if reference.owner() != Owner::Query
            || reference.family() != Family::QueryObservation
            || reference.reference_evidence_identity() != self.observation
        {
            return Err(Failure::InvalidReportingAssociation);
        }
        let identity = compose(
            Kind::EvidenceBinding,
            &[
                Part::Shape("query-observation-reporting"),
                Part::Shape(Owner::Query.as_str()),
                Part::Shape(Family::QueryObservation.as_str()),
                Part::Evidence(self.observation),
            ],
            self.admit,
        )?;
        progress.record_query_report(BridgeCausalEvidenceBinding::query_observation_reporting(
            reference, identity,
        ));
        Ok(())
    }

    fn requires_bridge_route(&self) -> bool {
        false
    }

    fn missing_bridge_route(
        &mut self,
        _: &BridgeCausalEnvelopeAssemblyRequest,
        _: &BridgeCausalEnvelopeAssemblyProgress,
    ) -> Self::Stop {
        Failure::InvalidReportingAssociation
    }

    fn finish(
        self,
        request: BridgeCausalEnvelopeAssemblyRequest,
        progress: BridgeCausalEnvelopeAssemblyProgress,
    ) -> Result<BridgeCausalExplanationEnvelope, Self::Stop> {
        let counts = progress.query_reporting_counts();
        if counts != [1, 0, 0, 0, 1, 1, 0, 0] {
            return Err(Failure::InvalidReportingAssociation);
        }
        let counter_identity = compose(
            Kind::Counters,
            &[
                Part::Value("1"),
                Part::Value("0"),
                Part::Value("0"),
                Part::Value("0"),
                Part::Value("1"),
                Part::Value("1"),
                Part::Value("0"),
                Part::Value("0"),
            ],
            self.admit,
        )?;
        let counters =
            BridgeCausalEnvelopeCounters::query_observation_reporting(counts, counter_identity);
        let bindings = progress.into_bindings();
        let binding_identity = bindings[0].binding_evidence_identity();
        let binding_set = compose(
            Kind::BindingSet,
            &[Part::Evidence(&binding_identity)],
            self.admit,
        )?;
        let identity_value = compose(
            Kind::Identity,
            &[
                Part::Evidence(request.request_evidence_identity()),
                Part::Evidence(
                    request
                        .admission_summary()
                        .causal_observation_anchor_identity(),
                ),
                Part::Evidence(&binding_set),
                Part::Evidence(counters.counter_evidence_identity()),
            ],
            self.admit,
        )?;
        let identity = BridgeCausalEnvelopeIdentity::query_observation_reporting(
            request.request_evidence_identity().clone(),
            request
                .admission_summary()
                .causal_observation_anchor_identity()
                .clone(),
            binding_set,
            counters.counter_evidence_identity().clone(),
            identity_value,
        );
        let envelope = compose(
            Kind::ExplanationEnvelope,
            &[
                Part::Evidence(identity.envelope_evidence_identity()),
                Part::Evidence(request.request_evidence_identity()),
                Part::Evidence(
                    request
                        .admission_summary()
                        .causal_observation_anchor_identity(),
                ),
                Part::Evidence(&binding_identity),
                Part::Evidence(counters.counter_evidence_identity()),
            ],
            self.admit,
        )?;
        let receipt_identity = compose(
            Kind::Receipt,
            &[
                Part::Evidence(identity.envelope_evidence_identity()),
                Part::Evidence(&envelope),
                Part::Evidence(counters.counter_evidence_identity()),
            ],
            self.admit,
        )?;
        let receipt = BridgeCausalEnvelopeReceipt::query_observation_reporting(
            &identity,
            &envelope,
            &counters,
            receipt_identity,
        );
        Ok(
            BridgeCausalExplanationEnvelope::query_observation_reporting(
                request, bindings, counters, identity, receipt, envelope,
            ),
        )
    }
}
