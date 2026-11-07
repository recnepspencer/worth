use super::eligibility::{AdmittedProjectionConsumption, ProjectionConsumptionWarningKind};
use super::facts::ProjectionMaterializedFactPosture;
use super::facts::{ProjectionFactFieldPath, ProjectionFactKind, ProjectionFactRequest};
use super::identity::compose_materialized_projection_contract_digest;
use super::source::{
    ProjectionSourceBasisAuthority, ProjectionSourceFamily, ProjectionSourceIdentity,
    ProjectionSourceReferenceIdentity,
};
use super::DeclaredNativeFactContract;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionContractSourcePosture {
    QueryOwnedReceiptSource,
    RelationalAuthoritySource,
    BridgeAuthoritySource,
    RetainedArtifactBindingSource,
    LiveArtifactBindingSource,
}

impl ProjectionContractSourcePosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryOwnedReceiptSource => "query_owned_receipt_source",
            Self::RelationalAuthoritySource => "relational_authority_source",
            Self::BridgeAuthoritySource => "bridge_authority_source",
            Self::RetainedArtifactBindingSource => "retained_artifact_binding_source",
            Self::LiveArtifactBindingSource => "live_artifact_binding_source",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionContractSupportPosture {
    Admitted,
    AdmittedWithWarnings(Vec<ProjectionConsumptionWarningKind>),
}

impl ProjectionContractSupportPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::AdmittedWithWarnings(_) => "admitted_with_warnings",
        }
    }

    pub fn warning_kinds(&self) -> &[ProjectionConsumptionWarningKind] {
        match self {
            Self::Admitted => &[],
            Self::AdmittedWithWarnings(warnings) => warnings,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundProjectionFactFamily {
    request: ProjectionFactRequest,
    native_contract: Option<DeclaredNativeFactContract>,
    support_posture: ProjectionContractSupportPosture,
}

impl BoundProjectionFactFamily {
    pub fn request(&self) -> &ProjectionFactRequest {
        &self.request
    }

    pub fn kind(&self) -> ProjectionFactKind {
        self.request.kind()
    }

    pub fn field_path(&self) -> Option<&ProjectionFactFieldPath> {
        self.request.field_path()
    }

    pub fn support_posture(&self) -> &ProjectionContractSupportPosture {
        &self.support_posture
    }

    pub(crate) fn native_contract(&self) -> Option<&DeclaredNativeFactContract> {
        self.native_contract.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializedProjectionContract {
    declaration_digest: String,
    eligibility_digest: String,
    query_digest: Option<String>,
    basis_digest: Option<String>,
    basis_authority: ProjectionSourceBasisAuthority,
    result_digest: Option<String>,
    canonical_result_shape_digest: String,
    narrowed_result_shape_digest: String,
    authorized_projection_identity: String,
    policy_digest: String,
    tenant_schema_basis_digest: String,
    source_family: ProjectionSourceFamily,
    source_posture: ProjectionContractSourcePosture,
    source_identity: ProjectionSourceIdentity,
    source_reference_identities: Vec<ProjectionSourceReferenceIdentity>,
    materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
    fact_families: Vec<BoundProjectionFactFamily>,
    support_posture: ProjectionContractSupportPosture,
    contract_digest: String,
}

impl MaterializedProjectionContract {
    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.eligibility_digest
    }

    pub fn query_digest(&self) -> Option<&str> {
        self.query_digest.as_deref()
    }

    pub fn basis_digest(&self) -> Option<&str> {
        self.basis_digest.as_deref()
    }

    pub fn basis_authority(&self) -> &ProjectionSourceBasisAuthority {
        &self.basis_authority
    }

    pub fn result_digest(&self) -> Option<&str> {
        self.result_digest.as_deref()
    }

    pub fn canonical_result_shape_digest(&self) -> &str {
        &self.canonical_result_shape_digest
    }

    pub fn narrowed_result_shape_digest(&self) -> &str {
        &self.narrowed_result_shape_digest
    }

    pub fn authorized_projection_identity(&self) -> &str {
        &self.authorized_projection_identity
    }

    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    pub fn tenant_schema_basis_digest(&self) -> &str {
        &self.tenant_schema_basis_digest
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn source_posture(&self) -> ProjectionContractSourcePosture {
        self.source_posture
    }

    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    pub fn source_identity_handle(&self) -> &ProjectionSourceIdentity {
        &self.source_identity
    }

    pub fn source_reference_identities(&self) -> &[ProjectionSourceReferenceIdentity] {
        &self.source_reference_identities
    }

    pub fn fact_families(&self) -> &[BoundProjectionFactFamily] {
        &self.fact_families
    }

    pub fn materialized_fact_posture(&self) -> Option<&ProjectionMaterializedFactPosture> {
        self.materialized_fact_posture.as_ref()
    }

    pub fn support_posture(&self) -> &ProjectionContractSupportPosture {
        &self.support_posture
    }

    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }
}

pub(crate) fn bind_materialized_projection_contract(
    admitted: &AdmittedProjectionConsumption,
) -> Result<MaterializedProjectionContract, IdentityDerivationDenial> {
    let declaration = admitted.declaration();
    let source = declaration.source();
    let support_posture = if admitted.warning_kinds().is_empty() {
        ProjectionContractSupportPosture::Admitted
    } else {
        ProjectionContractSupportPosture::AdmittedWithWarnings(admitted.warning_kinds().to_vec())
    };
    let fact_families = declaration
        .requested()
        .requested()
        .map(|request| BoundProjectionFactFamily {
            native_contract: declaration
                .requested()
                .native_contract_for(request)
                .cloned(),
            request: request.clone(),
            support_posture: support_posture.clone(),
        })
        .collect::<Vec<_>>();
    let contract_digest = compose_materialized_projection_contract_digest(
        admitted,
        source,
        declaration.binding(),
        &fact_families,
        &support_posture,
        contract_source_posture(source.family()),
    )?;
    Ok(MaterializedProjectionContract {
        declaration_digest: declaration.declaration_digest().to_string(),
        eligibility_digest: admitted.eligibility_digest().to_string(),
        query_digest: source.query_digest().map(str::to_string),
        basis_digest: source.basis_digest().map(str::to_string),
        basis_authority: source.basis_authority().clone(),
        result_digest: source.result_digest().map(str::to_string),
        canonical_result_shape_digest: declaration.binding().result_shape_digest().to_string(),
        narrowed_result_shape_digest: declaration
            .binding()
            .narrowed_result_shape_digest()
            .to_string(),
        authorized_projection_identity: declaration
            .binding()
            .authorized_projection_identity()
            .to_string(),
        policy_digest: declaration.binding().policy_digest().to_string(),
        tenant_schema_basis_digest: declaration
            .binding()
            .tenant_schema_basis_digest()
            .to_string(),
        source_family: source.family(),
        source_posture: contract_source_posture(source.family()),
        source_identity: source.source_identity_handle().clone(),
        source_reference_identities: source.source_reference_identities().to_vec(),
        materialized_fact_posture: source.materialized_fact_posture().cloned(),
        fact_families,
        support_posture,
        contract_digest,
    })
}

fn contract_source_posture(family: ProjectionSourceFamily) -> ProjectionContractSourcePosture {
    match family {
        ProjectionSourceFamily::QueryReadReceipt
        | ProjectionSourceFamily::QueryLiveReadReceipt
        | ProjectionSourceFamily::QueryWriteReceipt
        | ProjectionSourceFamily::QueryContextExecution => {
            ProjectionContractSourcePosture::QueryOwnedReceiptSource
        }
        ProjectionSourceFamily::RetainedDerivedArtifactBinding => {
            ProjectionContractSourcePosture::RetainedArtifactBindingSource
        }
        ProjectionSourceFamily::LiveArtifactBinding => {
            ProjectionContractSourcePosture::LiveArtifactBindingSource
        }
        ProjectionSourceFamily::RelationalRowSet
        | ProjectionSourceFamily::RelationalGroupedProjection => {
            ProjectionContractSourcePosture::RelationalAuthoritySource
        }
        ProjectionSourceFamily::BridgeTruthViewRowSet
        | ProjectionSourceFamily::BridgeGroupedTruthView => {
            ProjectionContractSourcePosture::BridgeAuthoritySource
        }
    }
}
