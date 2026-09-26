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

/// Owner custody for one admitted guarded workflow mutation. Only `Committed`
/// carries authority to settle the workflow's operation transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGuardedWorkflowOperationCustody {
    Unseen,
    Committed(WorthQueryApplicationCommitReceipt),
    DispatchPending(WorthQueryApplicationCommitReceipt),
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    Indeterminate(WorthQueryApplicationIdempotencyResolutionDenial),
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

    pub(super) const fn foreign_admission() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission,
            authorization: None,
        }
    }

    pub(super) const fn provider_unavailable() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable,
            authorization: None,
        }
    }

    pub(super) fn from_provider(
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
    pub(in crate::domain_computation::primary_graph) fn resolve_admitted_guarded_workflow_operation_custody<
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
    ) -> Result<
        WorthQueryGuardedWorkflowOperationCustody,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if !binding.matches_guarded_workflow_effect(transition_identity) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        let bound = binding
            .bind_operation(admission.operation_authority_identity_bytes())
            .bind_operation_scope(admission.operation_scope_binding())
            .bind_preconditions(admission.mutation_preconditions().identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity());
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission)?
            .publication_binding();
        let commit_lane = self
            .primary_provider
            .application_branch_commit_lane(product.observation());
        let coordination = commit_lane.enter();
        let proof = self
            .authorize_idempotency_inspection(admission, &coordination)
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        let custody = proof
            .govern((), |()| {
                self.primary_provider
                    .resolve_guarded_workflow_operation_custody(bound, &product)
            })
            .map_err(|(_, denial)| {
                WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial)
            })?;
        use crate::domain_computation::primary_graph::provider::WorthQueryProviderGuardedWorkflowOperationCustody as Owner;
        Ok(match custody {
            Owner::Unseen => WorthQueryGuardedWorkflowOperationCustody::Unseen,
            Owner::IntentDrift => WorthQueryGuardedWorkflowOperationCustody::IntentDrift,
            Owner::PublicationPending => {
                WorthQueryGuardedWorkflowOperationCustody::PublicationPending
            }
            Owner::ProductUnpublished(handle) => {
                WorthQueryGuardedWorkflowOperationCustody::ProductUnpublished(handle)
            }
            Owner::Indeterminate(denial) => {
                WorthQueryGuardedWorkflowOperationCustody::Indeterminate(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_provider(denial),
                )
            }
            Owner::Committed(provider) => {
                let Ok(projection) = WorthQueryCommittedReceiptProjection::resolve(provider) else {
                    return Ok(WorthQueryGuardedWorkflowOperationCustody::Indeterminate(
                        WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable(),
                    ));
                };
                let receipt = WorthQueryApplicationCommitReceipt::from_idempotency_read(
                    WorthQueryIdempotencyReadCommitReceiptPermit::mint(),
                    projection,
                    recover_equivalent_commit_evidence(admission.mutation_preconditions()),
                    admission.canonical_work(),
                    WorthQueryApplicationCommitAuthorityBinding::from_admission(admission, bound),
                );
                if super::workflow_transition_program::operation_receipt_requires_recovery(&receipt)
                {
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt)
                } else {
                    WorthQueryGuardedWorkflowOperationCustody::Committed(receipt)
                }
            }
        })
    }

    pub fn resolve_admitted_application_idempotency<Operation, Input, Scope>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.resolve_admitted_application_idempotencies(admission, [binding])
            .map(|mut reads| {
                reads
                    .pop()
                    .expect("one requested idempotency binding yields one read")
            })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_admitted_application_idempotencies<
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        bindings: impl IntoIterator<Item = WorthQueryApplicationIdempotencyBinding>,
    ) -> Result<
        Vec<WorthQueryAdmittedIdempotencyRead>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
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
        let bindings = bindings
            .into_iter()
            .map(|read_for| {
                let binding = read_for
                    .bind_operation(admission.operation_authority_identity_bytes())
                    .bind_operation_scope(admission.operation_scope_binding())
                    .bind_preconditions(admission.mutation_preconditions().identity())
                    .bind_governed_input(admission.governed_input_identity())
                    .bind_governed_proposal(admission.governed_proposal_identity());
                (read_for, binding)
            })
            .collect::<Vec<_>>();
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission)?
            .publication_binding();
        let commit_lane = self
            .primary_provider
            .application_branch_commit_lane(product.observation());
        let coordination = commit_lane.enter();
        let proof = self
            .authorize_idempotency_inspection(admission, &coordination)
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        let resolutions = proof
            .govern((), |()| {
                bindings
                    .iter()
                    .map(|(_, binding)| {
                        self.primary_provider
                            .resolve_idempotency_binding_at_product(*binding, &product)
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(|(_, denial)| {
                WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial)
            })?;
        bindings
            .into_iter()
            .zip(resolutions)
            .map(|((read_for, binding), resolution)| match resolution {
                Ok(WorthQueryProviderIdempotencyResolution::Absent) => {
                    Ok(WorthQueryAdmittedIdempotencyRead::mint(
                        read_for,
                        WorthQueryApplicationIdempotencyResolution::Unseen,
                    ))
                }
                Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt)) => {
                    let projection = WorthQueryCommittedReceiptProjection::resolve(receipt)
                        .map_err(|_| {
                            WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable()
                        })?;
                    let receipt = WorthQueryApplicationCommitReceipt::from_idempotency_read(
                        WorthQueryIdempotencyReadCommitReceiptPermit::mint(),
                        projection,
                        recover_equivalent_commit_evidence(admission.mutation_preconditions()),
                        admission.canonical_work(),
                        WorthQueryApplicationCommitAuthorityBinding::from_admission(
                            admission, binding,
                        ),
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
            })
            .collect()
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
