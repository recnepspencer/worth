use crate::authorized_projection::{
    derive_authorized_projection, AuthorizedProjectionArtifact, AuthorizedProjectionError,
    PolicyAspectMask, PolicyInfluenceSet,
};
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::projection_consumption::{
    ConsumedProjectionAuthorityDenial, DeferredProjectionConsumption, DeniedProjectionConsumption,
    ProjectionAuthorityContract, ProjectionAuthorityOutcome, ProjectionConsumptionDeclarationError,
    ProjectionConsumptionWarnings, ProjectionFactExtractionError, ProjectionFactFieldPath,
    SourceMismatchedProjectionConsumption, WorthQueryConsumedProjectionAuthority,
};
use crate::runtime::{WorthQueryReadGraph, WorthQueryReadResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProjectionDeclaration {
    contract: ProjectionAuthorityContract,
}

/// Begin a typed fact-extraction declaration for a completed ordinary read.
///
/// The declaration always requires settled consumption, source authority, and
/// matching basis generation. Consumers choose facts, not safety posture.
pub fn project_facts() -> WorthQueryProjectionDeclaration {
    WorthQueryProjectionDeclaration {
        contract: ProjectionAuthorityContract::declare()
            .require_settled_consumption()
            .require_source_authority()
            .require_basis_generation(),
    }
}

impl WorthQueryProjectionDeclaration {
    pub fn entity_identities(mut self) -> Self {
        self.contract = self.contract.require_entity_identities();
        self
    }

    pub fn view_local_identities(mut self) -> Self {
        self.contract = self.contract.require_view_local_identities();
        self
    }

    pub fn target_identity(mut self) -> Self {
        self.contract = self.contract.require_target_identity();
        self
    }

    pub fn source_references(mut self) -> Self {
        self.contract = self.contract.require_source_references();
        self
    }

    pub fn effect_continuity_facts(mut self) -> Self {
        self.contract = self.contract.require_effect_continuity_facts();
        self
    }

    pub fn memberships(mut self) -> Self {
        self.contract = self.contract.require_memberships();
        self
    }

    pub fn relation_endpoints(mut self) -> Self {
        self.contract = self.contract.require_relation_endpoints();
        self
    }

    pub fn display_field(mut self, field: ProjectionFactFieldPath) -> Self {
        self.contract = self.contract.require_display_field(field);
        self
    }

    pub fn derived_field(mut self, field: ProjectionFactFieldPath) -> Self {
        self.contract = self.contract.require_derived_field(field);
        self
    }

    pub(crate) fn display_native(
        mut self,
        contract: crate::projection_consumption::DeclaredNativeFactContract,
    ) -> Result<Self, crate::projection_consumption::NativeFactDeclarationConflict> {
        self.contract = self.contract.require_display_native(contract)?;
        Ok(self)
    }

    pub(crate) fn derived_native(
        mut self,
        contract: crate::projection_consumption::DeclaredNativeFactContract,
    ) -> Result<Self, crate::projection_consumption::NativeFactDeclarationConflict> {
        self.contract = self.contract.require_derived_native(contract)?;
        Ok(self)
    }
}

#[derive(Debug)]
pub struct WorthQueryProjectionAdvisory {
    authority: Box<WorthQueryConsumedProjectionAuthority>,
    warnings: ProjectionConsumptionWarnings,
}

impl WorthQueryProjectionAdvisory {
    pub fn authority(&self) -> &WorthQueryConsumedProjectionAuthority {
        &self.authority
    }

    pub fn warnings(&self) -> &ProjectionConsumptionWarnings {
        &self.warnings
    }
}

#[derive(Debug)]
pub enum WorthQueryProjectionViolation {
    Authority(ConsumedProjectionAuthorityDenial),
    Consumption(DeniedProjectionConsumption),
    SourceMismatch(SourceMismatchedProjectionConsumption),
    Declaration(ProjectionConsumptionDeclarationError),
    LiveInstallationMismatch {
        expected: WorthQueryEvidenceIdentity,
        actual: WorthQueryEvidenceIdentity,
    },
}

#[derive(Debug)]
pub enum WorthQueryProjectionUnavailable {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    AuthorityBinding(AuthorizedProjectionError),
    Extraction(ProjectionFactExtractionError),
}

#[derive(Debug)]
pub enum WorthQueryProjectionOutcome {
    Completed(Box<WorthQueryConsumedProjectionAuthority>),
    Advisory(WorthQueryProjectionAdvisory),
    Violation(WorthQueryProjectionViolation),
    Deferred(DeferredProjectionConsumption),
    Unavailable(WorthQueryProjectionUnavailable),
}

impl WorthQueryProjectionOutcome {
    pub(crate) fn from_foundation(outcome: ProjectionAuthorityOutcome) -> Self {
        match outcome {
            ProjectionAuthorityOutcome::Admitted(authority) => Self::Completed(authority),
            ProjectionAuthorityOutcome::AdmittedWithWarnings(authority, warnings) => {
                Self::Advisory(WorthQueryProjectionAdvisory {
                    authority,
                    warnings,
                })
            }
            ProjectionAuthorityOutcome::AuthorityDenied(denial) => {
                Self::Violation(WorthQueryProjectionViolation::Authority(denial))
            }
            ProjectionAuthorityOutcome::ConsumptionDenied(denial) => {
                Self::Violation(WorthQueryProjectionViolation::Consumption(denial))
            }
            ProjectionAuthorityOutcome::Deferred(deferred) => Self::Deferred(deferred),
            ProjectionAuthorityOutcome::SourceMismatch(mismatch) => {
                Self::Violation(WorthQueryProjectionViolation::SourceMismatch(mismatch))
            }
        }
    }

    pub(crate) fn with_query_context_advisory_for_certification(
        self,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(match self {
            Self::Completed(authority) => Self::Advisory(WorthQueryProjectionAdvisory {
                authority,
                warnings: ProjectionConsumptionWarnings::for_certification(
                    crate::projection_consumption::ProjectionConsumptionWarningKind::QueryContextRowBound,
                )?,
            }),
            other => other,
        })
    }

    pub fn authority(&self) -> Option<&WorthQueryConsumedProjectionAuthority> {
        match self {
            Self::Completed(authority) => Some(authority),
            Self::Advisory(advisory) => Some(advisory.authority()),
            Self::Violation(_) | Self::Deferred(_) | Self::Unavailable(_) => None,
        }
    }

    pub fn advisory(&self) -> Option<&WorthQueryProjectionAdvisory> {
        match self {
            Self::Advisory(advisory) => Some(advisory),
            _ => None,
        }
    }

    pub fn violation(&self) -> Option<&WorthQueryProjectionViolation> {
        match self {
            Self::Violation(violation) => Some(violation),
            _ => None,
        }
    }

    pub fn deferred(&self) -> Option<&DeferredProjectionConsumption> {
        match self {
            Self::Deferred(deferred) => Some(deferred),
            _ => None,
        }
    }

    pub fn unavailable(&self) -> Option<&WorthQueryProjectionUnavailable> {
        match self {
            Self::Unavailable(unavailable) => Some(unavailable),
            _ => None,
        }
    }

    /// Transfer the sealed projection authority into a downstream consumer.
    ///
    /// This preserves advisory warnings without exposing the decomposed
    /// projection-consumption lifecycle or allowing a receipt/digest to be
    /// promoted back into authority.
    pub fn into_admitted(
        self,
    ) -> Result<
        (
            Box<WorthQueryConsumedProjectionAuthority>,
            Option<ProjectionConsumptionWarnings>,
        ),
        Self,
    > {
        match self {
            Self::Completed(authority) => Ok((authority, None)),
            Self::Advisory(advisory) => Ok((advisory.authority, Some(advisory.warnings))),
            other => Err(other),
        }
    }
}

#[derive(Debug)]
pub(crate) struct WorthQueryReadProjectionBinding {
    result_shape: CanonicalResultShapeArtifact,
    authorized_projection: Result<AuthorizedProjectionArtifact, AuthorizedProjectionError>,
}

impl WorthQueryReadProjectionBinding {
    pub(crate) fn from_graph(graph: &WorthQueryReadGraph) -> Self {
        let result_shape = graph.canonical().result_shape().clone();
        let authorized_projection = graph
            .authorized_projection()
            .map(|authorized| Ok(authorized.clone()))
            .unwrap_or_else(|| unrestricted_projection(graph));
        Self {
            result_shape,
            authorized_projection,
        }
    }

    pub(crate) fn consume(
        &self,
        result: &WorthQueryReadResult,
        declaration: WorthQueryProjectionDeclaration,
    ) -> WorthQueryProjectionOutcome {
        self.consume_contract(result, declaration.contract)
    }

    pub(crate) fn consume_live(
        &self,
        result: &crate::runtime::WorthQueryLiveReadResult,
        declaration: WorthQueryProjectionDeclaration,
    ) -> WorthQueryProjectionOutcome {
        self.consume_live_contract(result, declaration.contract)
    }

    pub(crate) fn consume_live_contract(
        &self,
        result: &crate::runtime::WorthQueryLiveReadResult,
        contract: ProjectionAuthorityContract,
    ) -> WorthQueryProjectionOutcome {
        let authorized = match &self.authorized_projection {
            Ok(authorized) => authorized,
            Err(error) => {
                return WorthQueryProjectionOutcome::Unavailable(
                    WorthQueryProjectionUnavailable::AuthorityBinding(error.clone()),
                )
            }
        };
        let binding = crate::projection_consumption::ProjectionConsumptionBindingContext::from_authorized_projection(
            &self.result_shape,
            authorized,
        );
        match result.consume_projection_authority_with_binding(binding, contract) {
            Ok(outcome) => WorthQueryProjectionOutcome::from_foundation(outcome),
            Err(crate::projection_consumption::ProjectionFactConsumptionPathError::EvidenceIdentityDerivationDenied(cause)) => WorthQueryProjectionOutcome::Unavailable(WorthQueryProjectionUnavailable::EvidenceIdentityDerivationDenied(cause)),
            Err(
                crate::projection_consumption::ProjectionFactConsumptionPathError::Declaration(
                    error,
                ),
            ) => WorthQueryProjectionOutcome::Violation(
                WorthQueryProjectionViolation::Declaration(error),
            ),
            Err(crate::projection_consumption::ProjectionFactConsumptionPathError::Extraction(
                error,
            )) => WorthQueryProjectionOutcome::Unavailable(
                WorthQueryProjectionUnavailable::Extraction(error),
            ),
        }
    }

    pub(crate) fn consume_contract(
        &self,
        result: &WorthQueryReadResult,
        contract: ProjectionAuthorityContract,
    ) -> WorthQueryProjectionOutcome {
        let authorized = match &self.authorized_projection {
            Ok(authorized) => authorized,
            Err(error) => {
                return WorthQueryProjectionOutcome::Unavailable(
                    WorthQueryProjectionUnavailable::AuthorityBinding(error.clone()),
                )
            }
        };
        match result.consume_projection_authority(&self.result_shape, authorized, contract) {
            Ok(outcome) => WorthQueryProjectionOutcome::from_foundation(outcome),
            Err(crate::projection_consumption::ProjectionFactConsumptionPathError::EvidenceIdentityDerivationDenied(cause)) => WorthQueryProjectionOutcome::Unavailable(WorthQueryProjectionUnavailable::EvidenceIdentityDerivationDenied(cause)),
            Err(
                crate::projection_consumption::ProjectionFactConsumptionPathError::Declaration(
                    error,
                ),
            ) => WorthQueryProjectionOutcome::Violation(
                WorthQueryProjectionViolation::Declaration(error),
            ),
            Err(crate::projection_consumption::ProjectionFactConsumptionPathError::Extraction(
                error,
            )) => WorthQueryProjectionOutcome::Unavailable(
                WorthQueryProjectionUnavailable::Extraction(error),
            ),
        }
    }

    pub(crate) fn validates_installed_publication(
        &self,
        canonical: &crate::canonicalization::CanonicalQueryBundle,
    ) -> bool {
        if self.result_shape.digest() != canonical.result_shape().digest() {
            return false;
        }
        match &self.authorized_projection {
            Ok(authorized) => {
                authorized.query_digest() == canonical.query().digest().as_str()
                    && authorized.result_shape_digest()
                        == canonical.result_shape().digest().as_str()
            }
            Err(_) => false,
        }
    }
}

fn unrestricted_projection(
    graph: &WorthQueryReadGraph,
) -> Result<AuthorizedProjectionArtifact, AuthorizedProjectionError> {
    let policy_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "ordinary-unrestricted-read-projection",
            )
            .field_shape(WorthQueryEvidenceTag::new("read_graph"), graph.digest())
            .field_shape(
                WorthQueryEvidenceTag::new("schema_basis"),
                graph.schema_basis().render_support_hex(),
            )
            .seal()
            .expect(
                "projection policy uses a fixed role, sealed graph digest and fixed schema digest",
            );
    derive_authorized_projection(
        graph.canonical().query(),
        graph.canonical().result_shape(),
        policy_identity.as_str(),
        &graph.schema_basis().render_support_hex(),
        &PolicyAspectMask::allow_all(),
        &PolicyInfluenceSet::none(),
        usize::MAX,
        usize::MAX,
    )
}
