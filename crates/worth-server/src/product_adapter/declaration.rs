use std::sync::Arc;

use crate::WorthServerOperationFamily;

use super::{
    WorthServerProductAdapterCertificationCode, WorthServerProductAdapterCertificationError,
    WorthServerProductOperationAuthorityRequirement, WorthServerProductOperationBasisKind,
    WorthServerProductOperationErrorMap, WorthServerProductOperationSupportSnapshot,
    WorthServerProductPayloadSchemaValidator, WorthServerProductReadTransport,
};

#[derive(Clone)]
pub struct WorthServerProductOperationDeclaration {
    operation_name: String,
    operation_family: WorthServerOperationFamily,
    payload_schema_identity: String,
    result_contract: crate::WorthServerProductResultContract,
    basis_kind: WorthServerProductOperationBasisKind,
    support_snapshot: WorthServerProductOperationSupportSnapshot,
    authority_requirement: WorthServerProductOperationAuthorityRequirement,
    read_transport: Option<WorthServerProductReadTransport>,
    payload_validator: Option<Arc<dyn WorthServerProductPayloadSchemaValidator>>,
    error_map: Option<Arc<dyn WorthServerProductOperationErrorMap>>,
    query_application_readiness:
        Option<Arc<dyn super::WorthServerQueryApplicationReadinessProvider>>,
}

impl std::fmt::Debug for WorthServerProductOperationDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthServerProductOperationDeclaration")
            .field("operation_name", &self.operation_name)
            .field("operation_family", &self.operation_family)
            .field("payload_schema_identity", &self.payload_schema_identity)
            .field("result_contract", &self.result_contract)
            .field("basis_kind", &self.basis_kind)
            .field("support_snapshot", &self.support_snapshot)
            .field("authority_requirement", &self.authority_requirement)
            .field("read_transport", &self.read_transport)
            .field(
                "query_application_readiness",
                &self
                    .query_application_readiness
                    .as_ref()
                    .map(|provider| provider.provider_name()),
            )
            .finish()
    }
}

impl WorthServerProductOperationDeclaration {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        let authority_bytes = match &self.authority_requirement {
            WorthServerProductOperationAuthorityRequirement::SharedRead => 0,
            _ => u64::MAX,
        };
        string(&self.operation_name)
            .saturating_add(string(&self.payload_schema_identity))
            .saturating_add(self.result_contract.owned_allocation_capacity_bytes())
            .saturating_add(self.support_snapshot.owned_allocation_capacity_bytes())
            .saturating_add(authority_bytes)
    }

    pub fn product_read(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        basis_kind: WorthServerProductOperationBasisKind,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
    ) -> Self {
        Self::product_read_with_transport(
            operation_name,
            payload_schema_identity,
            result_contract,
            basis_kind,
            support_snapshot,
            WorthServerProductReadTransport::FlatQuery,
        )
    }

    pub fn product_structured_read(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        basis_kind: WorthServerProductOperationBasisKind,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
    ) -> Self {
        Self::product_read_with_transport(
            operation_name,
            payload_schema_identity,
            result_contract,
            basis_kind,
            support_snapshot,
            WorthServerProductReadTransport::StructuredQuery,
        )
    }

    fn product_read_with_transport(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        basis_kind: WorthServerProductOperationBasisKind,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
        read_transport: WorthServerProductReadTransport,
    ) -> Self {
        Self {
            operation_name: operation_name.into(),
            operation_family: WorthServerOperationFamily::ProductApplicationRead,
            payload_schema_identity: payload_schema_identity.into(),
            result_contract,
            basis_kind,
            support_snapshot,
            authority_requirement: WorthServerProductOperationAuthorityRequirement::SharedRead,
            read_transport: Some(read_transport),
            payload_validator: None,
            error_map: None,
            query_application_readiness: None,
        }
    }

    pub fn product_mutation(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        basis_kind: WorthServerProductOperationBasisKind,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
        draft_scope: impl Into<String>,
    ) -> Self {
        Self {
            operation_name: operation_name.into(),
            operation_family: WorthServerOperationFamily::ProductApplicationMutation,
            payload_schema_identity: payload_schema_identity.into(),
            result_contract,
            basis_kind,
            support_snapshot,
            authority_requirement: WorthServerProductOperationAuthorityRequirement::DraftMutation {
                draft_scope: draft_scope.into(),
            },
            read_transport: None,
            payload_validator: None,
            error_map: None,
            query_application_readiness: None,
        }
    }

    pub fn product_session_coordination(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        basis_kind: WorthServerProductOperationBasisKind,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
        coordination_lane: impl Into<String>,
    ) -> Self {
        Self {
            operation_name: operation_name.into(),
            operation_family: WorthServerOperationFamily::ProductSessionCoordination,
            payload_schema_identity: payload_schema_identity.into(),
            result_contract,
            basis_kind,
            support_snapshot,
            authority_requirement:
                WorthServerProductOperationAuthorityRequirement::SessionCoordination {
                    coordination_lane: coordination_lane.into(),
                },
            read_transport: None,
            payload_validator: None,
            error_map: None,
            query_application_readiness: None,
        }
    }

    pub fn with_payload_validator(
        mut self,
        payload_validator: Arc<dyn WorthServerProductPayloadSchemaValidator>,
    ) -> Self {
        self.payload_validator = Some(payload_validator);
        self
    }

    pub fn with_error_map(
        mut self,
        error_map: Arc<dyn WorthServerProductOperationErrorMap>,
    ) -> Self {
        self.error_map = Some(error_map);
        self
    }

    /// Binds this operation to the installed Query application that owns its
    /// readiness snapshot. The snapshot is descriptive only; execution must
    /// still enter Query through the operation's typed adapter.
    pub fn with_primary_graph_application<Schema>(
        mut self,
        application: Arc<
            worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
                Schema,
            >,
        >,
    ) -> Self
    where
        Schema: worth_query_host::facade::declaration::application_schema::ApplicationSchema
            + Send
            + Sync
            + 'static,
    {
        self.query_application_readiness = Some(
            super::primary_graph_application_readiness_provider(application),
        );
        self
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn operation_family(&self) -> WorthServerOperationFamily {
        self.operation_family
    }

    pub fn payload_schema_identity(&self) -> &str {
        &self.payload_schema_identity
    }

    pub fn durable_product_mutation(
        operation_name: impl Into<String>,
        payload_schema_identity: impl Into<String>,
        result_contract: crate::WorthServerProductResultContract,
        support_snapshot: WorthServerProductOperationSupportSnapshot,
        durable_contract: crate::WorthServerDurableProductMutationContract,
    ) -> Self {
        Self {
            operation_name: operation_name.into(),
            operation_family: WorthServerOperationFamily::ProductApplicationMutation,
            payload_schema_identity: payload_schema_identity.into(),
            result_contract,
            basis_kind: WorthServerProductOperationBasisKind::DurableProductDerived,
            support_snapshot,
            authority_requirement:
                WorthServerProductOperationAuthorityRequirement::DurableMutation {
                    contract: durable_contract,
                },
            read_transport: None,
            payload_validator: None,
            error_map: None,
            query_application_readiness: None,
        }
    }

    pub fn result_contract(&self) -> &crate::WorthServerProductResultContract {
        &self.result_contract
    }

    pub fn basis_kind(&self) -> WorthServerProductOperationBasisKind {
        self.basis_kind
    }

    pub fn support_snapshot(&self) -> &WorthServerProductOperationSupportSnapshot {
        &self.support_snapshot
    }

    pub fn authority_requirement(&self) -> &WorthServerProductOperationAuthorityRequirement {
        &self.authority_requirement
    }

    pub fn read_transport(&self) -> Option<WorthServerProductReadTransport> {
        self.read_transport
    }

    pub fn durable_mutation_contract(
        &self,
    ) -> Option<&crate::WorthServerDurableProductMutationContract> {
        match &self.authority_requirement {
            WorthServerProductOperationAuthorityRequirement::DurableMutation { contract } => {
                Some(contract)
            }
            _ => None,
        }
    }

    pub(crate) fn canonical_digest(&self) -> String {
        let authority = match &self.authority_requirement {
            WorthServerProductOperationAuthorityRequirement::SharedRead => {
                "shared-read".to_string()
            }
            WorthServerProductOperationAuthorityRequirement::DraftMutation { draft_scope } => {
                format!("draft:{draft_scope}")
            }
            WorthServerProductOperationAuthorityRequirement::DurableMutation { contract } => {
                format!("durable:{}", contract.canonical_digest())
            }
            WorthServerProductOperationAuthorityRequirement::SessionCoordination {
                coordination_lane,
            } => format!("session-coordination:{coordination_lane}"),
        };
        crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
            "worth-server-product-operation-declaration-v3",
        )
        .field("operation", &self.operation_name)
        .field("family", self.operation_family.as_str())
        .field("payload", &self.payload_schema_identity)
        .field("result", self.result_contract.canonical_digest())
        .field("basis", self.basis_kind.as_str())
        .field("support_row", self.support_snapshot.support_row())
        .field("support_posture", self.support_snapshot.canonical_label())
        .field("authority", &authority)
        .field(
            "read_transport",
            match self.read_transport {
                Some(WorthServerProductReadTransport::FlatQuery) => "flat-query",
                Some(WorthServerProductReadTransport::StructuredQuery) => "structured-query",
                None => "not-applicable",
            },
        )
        .field(
            "query_application",
            &self
                .query_application_readiness
                .as_ref()
                .map(|provider| provider.binding_digest())
                .unwrap_or_else(|| "none".to_string()),
        )
        .finish()
    }

    pub(crate) fn payload_validator(
        &self,
    ) -> Option<&Arc<dyn WorthServerProductPayloadSchemaValidator>> {
        self.payload_validator.as_ref()
    }

    pub(crate) fn error_map(&self) -> &Arc<dyn WorthServerProductOperationErrorMap> {
        self.error_map
            .as_ref()
            .expect("validated product declarations must retain an explicit error map")
    }

    pub(crate) fn query_application_readiness_provider(
        &self,
    ) -> Option<&Arc<dyn super::WorthServerQueryApplicationReadinessProvider>> {
        self.query_application_readiness.as_ref()
    }

    pub(crate) fn validate(&self) -> Result<(), WorthServerProductAdapterCertificationError> {
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
