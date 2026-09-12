mod effect_accumulator;
mod effect_lowering;
mod registration;

pub(in crate::domain_computation::primary_graph) use registration::WorthQueryPrimaryGraphApplicationAttempt;
pub(in crate::domain_computation::primary_graph::application_attempt) use registration::WorthQueryProviderRegistrationInspectionPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use registration::WorthQueryRegisteredProviderAttemptSeal;

#[cfg(test)]
mod semantic_model_tests;

use worth_query_installation::facade::{
    InstalledCorrectionMechanism, InstalledPreImageDemand, WorthQueryInstalledAftermathContract,
};

use self::effect_accumulator::WorthQueryProviderEffectAccumulator;
use super::effect_program::WorthQueryApplicationRealizedEffect;
use super::fact::WorthQueryApplicationObservedFact;
use super::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEmission,
};

pub(in crate::domain_computation) struct WorthQueryPreparedApplicationProviderAttempt {
    installed_read_scopes: Vec<worth_query_installation::facade::WorthQueryOperationGraphReadScope>,
    facts: Vec<WorthQueryApplicationObservedFact>,
    effects: effect_accumulator::WorthQueryRegisteredProviderEffects,
    preimage_demand: Option<InstalledPreImageDemand>,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    validator_work_admission: super::effect_program::WorthQueryCandidateValidatorWorkAdmission,
}

impl WorthQueryPreparedApplicationProviderAttempt {
    pub(in crate::domain_computation) fn register<'run, Schema, Operation, Input, Scope>(
        self,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        authorization: crate::domain_computation::authorization::WorthQueryProviderAuthorizationDecisionFacts,
        attempt_basis: super::provider_execution::WorthQueryApplicationAttemptBasis,
        context: super::provider_execution::WorthQueryProviderAttemptRegistrationContext<
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Result<
        super::provider_execution::WorthQueryRegisteredProviderAttempt<'run>,
        super::provider_execution::WorthQueryProviderProgressionOutcome,
    > {
        registration::register_provider_attempt(self, staged, authorization, attempt_basis, context)
    }
}

/// Derive the retention demand from the admitted operation's compiled aftermath
/// (Q8.26-C1).
///
/// The demand is a property of the installed contract. Nothing about retention
/// is a caller's to supply.
pub(super) fn installed_preimage_demand(
    aftermath: Option<&WorthQueryInstalledAftermathContract>,
) -> Option<InstalledPreImageDemand> {
    match aftermath?.mechanism()? {
        InstalledCorrectionMechanism::RecordedInverse(inverse) => {
            Some(inverse.preimage_demand().clone())
        }
        InstalledCorrectionMechanism::Compensation(_) => None,
    }
}

pub(super) fn prepare_provider_attempt(
    installed_read_scopes: Vec<worth_query_installation::facade::WorthQueryOperationGraphReadScope>,
    facts: Vec<WorthQueryApplicationObservedFact>,
    effects: Vec<WorthQueryApplicationRealizedEffect>,
    expected_emission_retained_bytes: u64,
    emission_retained_bytes_ceiling: u64,
    preimage_demand: Option<InstalledPreImageDemand>,
    conditional_definition: Option<
        crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition,
    >,
    validator_work_admission: super::effect_program::WorthQueryCandidateValidatorWorkAdmission,
    output_correspondence: super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
) -> Result<WorthQueryPreparedApplicationProviderAttempt, WorthQueryApplicationAttemptDenial> {
    let mut accumulator = WorthQueryProviderEffectAccumulator::new(&facts, &effects);
    for effect in effects {
        accumulator.add_effect(effect)?;
    }
    let completed = accumulator.finish(
        expected_emission_retained_bytes,
        emission_retained_bytes_ceiling,
        output_correspondence,
    )?;
    Ok(WorthQueryPreparedApplicationProviderAttempt {
        installed_read_scopes,
        facts,
        effects: completed,
        preimage_demand,
        conditional_definition,
        validator_work_admission,
    })
}

pub(super) fn progression_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::IncompleteEffectBasis,
        "provider progression",
    )
}

pub(super) fn retained_bytes_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded,
        "application emission batch",
    )
}
