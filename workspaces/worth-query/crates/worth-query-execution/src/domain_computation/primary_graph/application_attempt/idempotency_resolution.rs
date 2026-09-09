use worth_query_installation::facade::ApplicationSchema;

use super::{
    provider_recomparison::recover_equivalent_commit_evidence,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyBinding, WorthQueryCommittedReceiptProjection,
};

pub(super) struct WorthQueryIdempotencyReadCommitReceiptPermit {
    _owner_mint: (),
}

impl WorthQueryIdempotencyReadCommitReceiptPermit {
    fn mint() -> Self {
        Self { _owner_mint: () }
    }
}
use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryOperationAuthorizationDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationIdempotencyResolution {
    Unseen,
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    IntentDrift,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationIdempotencyResolutionDenialKind {
    Authorization,
    ForeignAdmission,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ProviderUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationIdempotencyResolutionDenial {
    kind: WorthQueryApplicationIdempotencyResolutionDenialKind,
    authorization: Option<WorthQueryOperationAuthorizationDenial>,
}

impl WorthQueryApplicationIdempotencyResolutionDenial {
    pub const fn kind(&self) -> WorthQueryApplicationIdempotencyResolutionDenialKind {
        self.kind
    }

    pub const fn authorization(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization.as_ref()
    }

    fn from_authorization(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization,
            authorization: Some(denial),
        }
    }

    const fn foreign_admission() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission,
            authorization: None,
        }
    }

    const fn provider_unavailable() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable,
            authorization: None,
        }
    }

    fn from_provider(
        denial: crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial,
    ) -> Self {
        let kind = match denial {
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted
            }
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionIdentityExhausted
            }
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::SnapshotIdentityExhausted
            }
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::Unavailable => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
            }
        };
        Self {
            kind,
            authorization: None,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn resolve_admitted_application_idempotency<Operation, Input, Scope>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        let read_for = binding;
        let binding = binding
            .bind_operation(admission.operation_authority_identity_bytes())
            .bind_operation_scope(admission.operation_scope_binding())
            .bind_preconditions(admission.mutation_preconditions().identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity());
        let serialization = self.primary_provider.serialize_application_commit();
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission)?
            .publication_binding();
        let proof = self
            .authorize_idempotency_inspection(admission, &serialization)
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        let resolution = proof
            .govern((), |()| {
                self.primary_provider
                    .resolve_idempotency_binding_at_product(binding, &product)
            })
            .map_err(|(_, denial)| {
                WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial)
            })?;
        match resolution {
            Ok(WorthQueryProviderIdempotencyResolution::Absent) => {
                Ok(WorthQueryAdmittedIdempotencyRead::mint(
                    read_for,
                    WorthQueryApplicationIdempotencyResolution::Unseen,
                ))
            }
            Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt)) => {
                let projection =
                    WorthQueryCommittedReceiptProjection::resolve(receipt).map_err(|_| {
                        WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable()
                    })?;
                let receipt = WorthQueryApplicationCommitReceipt::from_idempotency_read(
                    WorthQueryIdempotencyReadCommitReceiptPermit::mint(),
                    projection,
                    recover_equivalent_commit_evidence(admission.mutation_preconditions()),
                    admission.canonical_work(),
                    WorthQueryApplicationCommitAuthorityBinding::from_admission(admission, binding),
                );
                Ok(WorthQueryAdmittedIdempotencyRead::mint(
                    read_for,
                    WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt),
                ))
            }
            Ok(WorthQueryProviderIdempotencyResolution::Drift) => {
                Ok(WorthQueryAdmittedIdempotencyRead::mint(
                    read_for,
                    WorthQueryApplicationIdempotencyResolution::IntentDrift,
                ))
            }
            Ok(WorthQueryProviderIdempotencyResolution::Unpublished) => {
                Err(WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable())
            }
            Err(denial) => {
                Err(WorthQueryApplicationIdempotencyResolutionDenial::from_provider(denial))
            }
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationIdempotencyResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application idempotency resolution denied: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for WorthQueryApplicationIdempotencyResolutionDenial {}
