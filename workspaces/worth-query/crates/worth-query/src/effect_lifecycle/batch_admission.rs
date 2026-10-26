use crate::basis_lifecycle::BasisFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::admission::admit_effect_intent;
use super::authoring_basis::EffectAuthoringBasis;
use super::counters::EffectLifecycleCounters;
use super::eligibility::EffectEligibilityOutcome;
use super::normalized::NormalizedEffectIntent;
use super::support_contract::EffectDeferredSupportContract;
use super::support_matrix::EffectSupportCause;
use super::taxonomy::{DeniedEffectEligibilityKind, EffectAuthorityLane, EffectFamily};
use super::{
    evaluate_effect_eligibility, normalize_raw_effect_intent, AdmittedEffectIntent, RawEffectIntent,
};

pub fn effect_batch() -> EffectBatchIntentDraft {
    EffectBatchIntentDraft {
        intents: Vec::new(),
    }
}

pub fn admit_effect_batch_components(
    admitted: Vec<AdmittedEffectIntent>,
) -> Result<AdmittedEffectBatch, EffectBatchAdmissionDenial> {
    if admitted.is_empty() {
        return Err(EffectBatchAdmissionDenial::new(
            EffectBatchAdmissionDenialKind::EmptyBatch,
            None,
            "effect batch must declare at least one effect",
            EffectLifecycleCounters::batch_admission_denied(0),
            None,
        ));
    }

    validate_batch_component_lane_coherence(&admitted, admitted.len(), |item| item.normalized())?;
    if let Some((index, component)) = admitted
        .iter()
        .enumerate()
        .find(|(_, component)| component.normalized().family() != EffectFamily::Mutation)
    {
        return Err(EffectBatchAdmissionDenial::new(
            EffectBatchAdmissionDenialKind::UnsupportedBatchFamily(component.normalized().family()),
            Some(index),
            "phase 4 batch-native execution currently admits only ordered relational mutation effects",
            EffectLifecycleCounters::batch_admission_denied(admitted.len()),
            None,
        ));
    }

    Ok(AdmittedEffectBatch::new(admitted))
}

#[derive(Clone, Debug, Default)]
pub struct EffectBatchIntentDraft {
    intents: Vec<RawEffectIntent>,
}

impl EffectBatchIntentDraft {
    pub fn push(mut self, intent: RawEffectIntent) -> Self {
        self.intents.push(intent);
        self
    }

    pub fn using_basis(self, basis: EffectAuthoringBasis) -> EffectBatchIntentDraftWithBasis {
        EffectBatchIntentDraftWithBasis {
            basis,
            intents: self.intents,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EffectBatchIntentDraftWithBasis {
    basis: EffectAuthoringBasis,
    intents: Vec<RawEffectIntent>,
}

impl EffectBatchIntentDraftWithBasis {
    pub fn push(mut self, intent: RawEffectIntent) -> Self {
        self.intents.push(intent);
        self
    }

    pub fn admit(self) -> Result<AdmittedEffectBatch, EffectBatchAdmissionDenial> {
        if self.intents.is_empty() {
            return Err(EffectBatchAdmissionDenial::new(
                EffectBatchAdmissionDenialKind::EmptyBatch,
                None,
                "effect batch must declare at least one effect",
                EffectLifecycleCounters::batch_admission_denied(0),
                None,
            ));
        }

        let mut normalized = Vec::with_capacity(self.intents.len());
        for (index, intent) in self.intents.into_iter().enumerate() {
            let normalized_intent =
                normalize_raw_effect_intent(&self.basis, intent).map_err(|denial| {
                    EffectBatchAdmissionDenial::new(
                        EffectBatchAdmissionDenialKind::ComponentNormalizationDenied,
                        Some(index),
                        denial.message(),
                        EffectLifecycleCounters::batch_admission_denied(index + 1),
                        None,
                    )
                })?;
            normalized.push(normalized_intent);
        }

        validate_batch_component_lane_coherence(&normalized, normalized.len(), |item| item)?;

        let mut admitted = Vec::with_capacity(normalized.len());
        for (index, normalized_intent) in normalized.into_iter().enumerate() {
            let admitted_intent = match evaluate_effect_eligibility(normalized_intent) {
                EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
                EffectEligibilityOutcome::Advisory(advisory) => {
                    return Err(EffectBatchAdmissionDenial::new(
                        EffectBatchAdmissionDenialKind::ComponentAdvisory(
                            advisory.advisory_cause(),
                        ),
                        Some(index),
                        advisory.decision_trace().message(),
                        EffectLifecycleCounters::batch_admission_denied(index + 1),
                        None,
                    ))
                }
                EffectEligibilityOutcome::Denied(denial) => {
                    return Err(EffectBatchAdmissionDenial::new(
                        EffectBatchAdmissionDenialKind::ComponentEligibilityDenied(
                            denial.denial_kind(),
                        ),
                        Some(index),
                        denial.decision_trace().message(),
                        EffectLifecycleCounters::batch_admission_denied(index + 1),
                        None,
                    ))
                }
                EffectEligibilityOutcome::RebindRequired(rebind) => {
                    return Err(EffectBatchAdmissionDenial::new(
                        EffectBatchAdmissionDenialKind::ComponentRebindRequired(
                            rebind.denial_kind(),
                        ),
                        Some(index),
                        rebind.decision_trace().message(),
                        EffectLifecycleCounters::batch_admission_denied(index + 1),
                        None,
                    ))
                }
                EffectEligibilityOutcome::Deferred(deferred) => {
                    return Err(EffectBatchAdmissionDenial::new(
                        EffectBatchAdmissionDenialKind::ComponentDeferred(deferred.denial_kind()),
                        Some(index),
                        deferred.decision_trace().message(),
                        EffectLifecycleCounters::batch_admission_denied(index + 1),
                        Some(deferred.deferred_contract().clone()),
                    ))
                }
            };
            admitted.push(admitted_intent);
        }

        admit_effect_batch_components(admitted)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectBatchAdmissionDenialKind {
    EmptyBatch,
    MixedAuthorityLane,
    MixedBasisLane,
    MixedBasisIdentity,
    UnsupportedBatchFamily(EffectFamily),
    ComponentNormalizationDenied,
    ComponentAdvisory(EffectSupportCause),
    ComponentEligibilityDenied(DeniedEffectEligibilityKind),
    ComponentRebindRequired(DeniedEffectEligibilityKind),
    ComponentDeferred(DeniedEffectEligibilityKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBatchAdmissionDenial {
    denial_kind: EffectBatchAdmissionDenialKind,
    component_index: Option<usize>,
    deferred_contract: Option<EffectDeferredSupportContract>,
    message: String,
    counters: EffectLifecycleCounters,
}

impl EffectBatchAdmissionDenial {
    fn new(
        denial_kind: EffectBatchAdmissionDenialKind,
        component_index: Option<usize>,
        message: impl Into<String>,
        counters: EffectLifecycleCounters,
        deferred_contract: Option<EffectDeferredSupportContract>,
    ) -> Self {
        Self {
            denial_kind,
            component_index,
            deferred_contract,
            message: message.into(),
            counters,
        }
    }

    pub fn denial_kind(&self) -> &EffectBatchAdmissionDenialKind {
        &self.denial_kind
    }
    pub fn component_index(&self) -> Option<usize> {
        self.component_index
    }
    pub fn deferred_contract(&self) -> Option<&EffectDeferredSupportContract> {
        self.deferred_contract.as_ref()
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedEffectBatch {
    admitted: Vec<AdmittedEffectIntent>,
    authority_lane: EffectAuthorityLane,
    basis_family: BasisFamily,
    batch_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl AdmittedEffectBatch {
    fn new(admitted: Vec<AdmittedEffectIntent>) -> Self {
        let authority_lane = admitted[0].normalized().authority_lane();
        let basis_family = admitted[0].normalized().basis_family();
        let batch_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "admitted_effect_batch_v1",
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("admitted"),
                    admitted.iter().map(AdmittedEffectIntent::admitted_identity),
                )
                .seal();
        let counters = EffectLifecycleCounters::admitted_batch(admitted.len());
        Self {
            admitted,
            authority_lane,
            basis_family,
            batch_identity,
            counters,
        }
    }

    pub fn admitted(&self) -> &[AdmittedEffectIntent] {
        &self.admitted
    }
    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_lane
    }
    pub fn basis_family(&self) -> BasisFamily {
        self.basis_family
    }
    pub fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_identity
    }

    pub fn batch_for_reporting(&self) -> &str {
        self.batch_identity.as_str()
    }
    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

fn validate_batch_component_lane_coherence<T>(
    components: &[T],
    component_count: usize,
    normalized: impl Fn(&T) -> &NormalizedEffectIntent,
) -> Result<(), EffectBatchAdmissionDenial> {
    let first = normalized(&components[0]);
    if components
        .iter()
        .any(|item| normalized(item).authority_lane() != first.authority_lane())
    {
        return Err(EffectBatchAdmissionDenial::new(
            EffectBatchAdmissionDenialKind::MixedAuthorityLane,
            None,
            "effect batch may not mix authority lanes",
            EffectLifecycleCounters::batch_admission_denied(component_count),
            None,
        ));
    }
    if components
        .iter()
        .any(|item| normalized(item).basis_family() != first.basis_family())
    {
        return Err(EffectBatchAdmissionDenial::new(
            EffectBatchAdmissionDenialKind::MixedBasisLane,
            None,
            "effect batch may not mix basis families",
            EffectLifecycleCounters::batch_admission_denied(component_count),
            None,
        ));
    }
    if components.iter().any(|item| {
        normalized(item).scoped_basis_identity() != first.scoped_basis_identity()
            || normalized(item).expected_lower_runtime_binding_identity()
                != first.expected_lower_runtime_binding_identity()
    }) {
        return Err(EffectBatchAdmissionDenial::new(
            EffectBatchAdmissionDenialKind::MixedBasisIdentity,
            None,
            "effect batch may not mix distinct scoped basis identities within one basis lane",
            EffectLifecycleCounters::batch_admission_denied(component_count),
            None,
        ));
    }
    Ok(())
}
