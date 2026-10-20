//! Admission rejects contradictory declaration contracts before any adapter work.
use super::super::{
    WorthServerProductAdapterCertificationCode, WorthServerProductAdapterCertificationError,
    WorthServerProductOperationAuthorityRequirement, WorthServerProductOperationBasisKind,
    WorthServerProductRetryOwner,
};
use super::WorthServerProductOperationDeclaration;

impl WorthServerProductOperationDeclaration {
    pub(crate) fn validate(&self) -> Result<(), WorthServerProductAdapterCertificationError> {
        if self.retry_owner == WorthServerProductRetryOwner::ApplicationCustody
            && !matches!(
                self.authority_requirement,
                WorthServerProductOperationAuthorityRequirement::DraftMutation { .. }
            )
        {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::IncompatibleRetryOwner,
                "application retry custody requires a runtime-backed product mutation; reads, session coordination and durable executors retain their distinct contracts",
            ));
        }
        if self.operation_name.trim().is_empty() {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::BlankOperationName,
                "product operation declarations require a non-blank operation name",
            ));
        }
        if self.payload_schema_identity.trim().is_empty() {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::BlankPayloadSchemaIdentity,
                "product operation declarations require a non-blank payload schema identity",
            ));
        }
        if self.support_snapshot.support_row().trim().is_empty() {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::BlankSupportSnapshotRow,
                "product operation declarations require a non-blank support snapshot row",
            ));
        }
        if self.error_map.is_none() {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::MissingErrorMap,
                "product operation declarations require an explicit denial or failure error map",
            ));
        }
        if self.basis_kind == WorthServerProductOperationBasisKind::PrimaryGraphApplication
            && self.query_application_readiness.is_none()
        {
            return Err(WorthServerProductAdapterCertificationError::new(
                WorthServerProductAdapterCertificationCode::MissingQueryApplicationReadinessProvider,
                "primary-graph application operations require their owning Query readiness provider",
            ));
        }
        match &self.authority_requirement {
            WorthServerProductOperationAuthorityRequirement::SharedRead => {}
            WorthServerProductOperationAuthorityRequirement::DraftMutation { draft_scope } => {
                if draft_scope.trim().is_empty() {
                    return Err(WorthServerProductAdapterCertificationError::new(
                        WorthServerProductAdapterCertificationCode::BlankDraftScope,
                        "product mutation declarations require a non-blank draft scope",
                    ));
                }
            }
            WorthServerProductOperationAuthorityRequirement::DurableMutation { .. } => {}
            WorthServerProductOperationAuthorityRequirement::SessionCoordination {
                coordination_lane,
            } => {
                if coordination_lane.trim().is_empty() {
                    return Err(WorthServerProductAdapterCertificationError::new(
                        WorthServerProductAdapterCertificationCode::BlankCoordinationLane,
                        "product session declarations require a non-blank coordination lane",
                    ));
                }
            }
        }
        Ok(())
    }
}
