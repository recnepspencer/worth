mod denial;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::WorthQueryMutationTargetCollectionIdentity;
pub use denial::{WorthQueryExistingTruthProbeDenial, WorthQueryExistingTruthProbeDenialKind};
use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};

use super::denied_aspect_touch::WorthQueryDeniedAspectTouch;
use super::{
    WorthQueryAspectTouch, WorthQueryExistingTruthTargetBinding, WorthQueryParsedAspectTarget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthProbeMode {
    BackendVerifiedProbe,
}

impl WorthQueryExistingTruthProbeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendVerifiedProbe => "backend_verified_probe",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthProbeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthProbeRequest {
    binding: WorthQueryExistingTruthTargetBinding,
    aspect_touches: Vec<WorthQueryAspectTouch>,
    request_digest: String,
}

impl WorthQueryExistingTruthProbeRequest {
    pub fn new(
        binding: WorthQueryExistingTruthTargetBinding,
        aspect_touches: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let mut aspect_touches = aspect_touches.into_iter().collect::<Vec<_>>();
        aspect_touches.sort();
        aspect_touches.dedup();
        if aspect_touches.is_empty() {
            return Err(WorthQueryWorkspaceError::new(
                "existing-truth probe must declare at least one aspect path",
            ));
        }
        let request_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-probe-request",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("binding"),
                    binding.binding_evidence_identity(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("aspect"),
                    aspect_touches
                        .iter()
                        .map(WorthQueryAspectTouch::admitted_touch_digest_part),
                )
                .seal()?
                .as_str()
                .to_string();
        Ok(Self {
            binding,
            aspect_touches,
            request_digest,
        })
    }

    pub fn binding(&self) -> &WorthQueryExistingTruthTargetBinding {
        &self.binding
    }

    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspect_touches
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryExistingTruthProbeField {
    target: WorthQueryParsedAspectTarget,
    value: AspectValue,
    value_digest: String,
}

impl WorthQueryExistingTruthProbeField {
    pub fn from_admitted_aspect_touch(
        aspect_touch: WorthQueryAspectTouch,
        value: AspectValue,
    ) -> Result<Self, IdentityDerivationDenial> {
        let target = aspect_touch.into_parsed_target();
        let aspect_touch = WorthQueryAspectTouch::from_parsed_target(target.clone());
        let value_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-probe-field",
                )
                .field_value(
                    WorthQueryEvidenceTag::new("aspect_touch"),
                    aspect_touch.admitted_touch_digest_part(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("value"),
                    prepare_aspect_value_identity_basis(&value).as_str(),
                )
                .seal()?
                .as_str()
                .to_string();
        Ok(Self {
            target,
            value,
            value_digest,
        })
    }

    pub fn aspect_touch(&self) -> WorthQueryAspectTouch {
        WorthQueryAspectTouch::from_parsed_target(self.target.clone())
    }

    pub fn foundational_value(&self) -> &AspectValue {
        &self.value
    }

    pub fn value_digest(&self) -> &str {
        &self.value_digest
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryExistingTruthProbe {
    binding: WorthQueryExistingTruthTargetBinding,
    mode: WorthQueryExistingTruthProbeMode,
    fields: Vec<WorthQueryExistingTruthProbeField>,
    probe_digest: String,
}

impl WorthQueryExistingTruthProbe {
    pub(crate) fn backend_verified(
        request: &WorthQueryExistingTruthProbeRequest,
        fields: Vec<WorthQueryExistingTruthProbeField>,
    ) -> Result<Self, WorthQueryExistingTruthProbeDenial> {
        for requested in request.aspect_touches() {
            if !fields
                .iter()
                .any(|field| field.target == *requested.parsed_target())
            {
                return Err(WorthQueryExistingTruthProbeDenial::from_admitted_touch(
                    request.binding(),
                    WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
                    requested.clone(),
                    "backend probe response did not return the requested aspect",
                ));
            }
        }
        let field_identities = fields
            .iter()
            .map(|field| {
                worth_query_evidence_identity(
                    WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-probe-row",
                )
                .field_value(
                    WorthQueryEvidenceTag::new("aspect_touch"),
                    field.aspect_touch().admitted_touch_digest_part(),
                )
                .field_value(WorthQueryEvidenceTag::new("field"), field.value_digest())
                .seal()
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| {
                WorthQueryExistingTruthProbeDenial::new(
                    request.binding(),
                    WorthQueryExistingTruthProbeDenialKind::EvidenceIdentityDerivationDenied(cause),
                    None,
                    "canonical digest bounds refused probe field evidence",
                )
            })?;
        let probe_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(WorthQueryEvidenceTag::new("role"), "existing-truth-probe")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("binding"),
                    request.binding().binding_evidence_identity(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("request"),
                    request.request_digest(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("mode"),
                    WorthQueryExistingTruthProbeMode::BackendVerifiedProbe.as_str(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("field"),
                    field_identities.iter(),
                )
                .seal()
                .map_err(|cause| {
                    WorthQueryExistingTruthProbeDenial::new(
                        request.binding(),
                        WorthQueryExistingTruthProbeDenialKind::EvidenceIdentityDerivationDenied(
                            cause,
                        ),
                        None,
                        "canonical digest bounds refused probe evidence",
                    )
                })?
                .as_str()
                .to_string();
        Ok(Self {
            binding: request.binding().clone(),
            mode: WorthQueryExistingTruthProbeMode::BackendVerifiedProbe,
            fields,
            probe_digest,
        })
    }

    pub fn binding(&self) -> &WorthQueryExistingTruthTargetBinding {
        &self.binding
    }

    pub fn mode(&self) -> WorthQueryExistingTruthProbeMode {
        self.mode
    }

    pub fn fields(&self) -> &[WorthQueryExistingTruthProbeField] {
        &self.fields
    }

    pub fn field_for_touch(
        &self,
        aspect_touch: &WorthQueryAspectTouch,
    ) -> Option<&WorthQueryExistingTruthProbeField> {
        self.fields
            .iter()
            .find(|field| field.target == *aspect_touch.parsed_target())
    }

    pub fn probe_digest(&self) -> &str {
        &self.probe_digest
    }
}
