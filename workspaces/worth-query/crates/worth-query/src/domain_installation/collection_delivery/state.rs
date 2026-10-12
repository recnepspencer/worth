use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryAdmittedConsumerInvalidation, WorthQueryBoundCollection,
    WorthQueryBoundCollectionWindow, WorthQueryCollectionDeliveryCounters,
    WorthQueryCollectionDeliveryDenial, WorthQueryCollectionDeliveryDenialKind,
    WorthQueryCollectionDeliveryOutcome, WorthQueryCollectionPatch,
    WorthQueryCollectionPatchApplicationReceipt, WorthQueryCollectionRowHandle,
    WorthQueryConsumerInvalidationAuthority, WorthQueryOperationResultState,
};

#[path = "state/evidence.rs"]
mod evidence;
pub(super) use evidence::denial;
use evidence::{carry_planning_counters, collection_evidence, maintenance_ordinal};
#[path = "state/granular_maintenance.rs"]
mod granular_maintenance;

pub struct WorthQueryCollectionConsumerWindow {
    pub(super) window: WorthQueryBoundCollectionWindow,
    pub(super) index: super::index::WorthQueryCollectionMaintenanceIndex,
    pub(super) authority: Option<WorthQueryConsumerInvalidationAuthority>,
    pub(super) last_maintenance_ordinal: Option<u64>,
    pub(super) pending_maintenance_ordinal: Option<u64>,
    pub(super) reset_pending: bool,
}

impl WorthQueryCollectionConsumerWindow {
    pub(crate) fn from_prepared(
        index: super::WorthQueryCollectionMaintenanceIndex,
        window: WorthQueryBoundCollectionWindow,
    ) -> Self {
        Self {
            index,
            window,
            authority: None,
            last_maintenance_ordinal: None,
            pending_maintenance_ordinal: None,
            reset_pending: false,
        }
    }

    pub fn from_bound<D, O, F, L: BasisOperationLane>(
        collection: WorthQueryBoundCollection<D, O, F, L>,
        window: WorthQueryBoundCollectionWindow,
    ) -> Result<Self, WorthQueryCollectionDeliveryDenial> {
        let mut counters = WorthQueryCollectionDeliveryCounters::default();
        counters.generation_checks += 1;
        if collection.capability_identity() != window.capability_identity
            || collection.capability_generation() != window.capability_generation
        {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::ForeignCollectionCapability,
                counters,
            ));
        }
        counters.semantic_contract_checks += 1;
        let mismatch = if collection.source_identity() != window.source_identity {
            Some(WorthQueryCollectionDeliveryDenialKind::SourceMismatch)
        } else if collection.binding_identity() != window.binding_identity {
            Some(WorthQueryCollectionDeliveryDenialKind::BindingMismatch)
        } else if collection.result_shape_identity() != window.result_shape_identity {
            Some(WorthQueryCollectionDeliveryDenialKind::ResultShapeMismatch)
        } else if collection.collection_delivery_contract_identity()
            != window.collection_delivery_contract_identity
        {
            Some(WorthQueryCollectionDeliveryDenialKind::CollectionContractMismatch)
        } else if collection.ordering_identity() != window.ordering_identity {
            Some(WorthQueryCollectionDeliveryDenialKind::OrderingMismatch)
        } else if collection.basis_identity() != window.basis_identity {
            Some(WorthQueryCollectionDeliveryDenialKind::BasisMismatch)
        } else {
            None
        };
        if let Some(kind) = mismatch {
            return Err(denial(kind, counters));
        }
        Ok(Self {
            index: collection.into_maintenance_index(),
            window,
            authority: None,
            last_maintenance_ordinal: None,
            pending_maintenance_ordinal: None,
            reset_pending: false,
        })
    }

    pub fn bind_shared_target(
        &mut self,
        admitted: &WorthQueryAdmittedConsumerInvalidation<'_>,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<(), WorthQueryCollectionDeliveryDenial> {
        let mut counters = WorthQueryCollectionDeliveryCounters::default();
        counters.invalidation_authority_checks += 1;
        if !admitted.remains_current(workspace) {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::ForeignOrStaleInvalidation,
                counters,
            ));
        }
        let authority = admitted.delta().authority();
        counters.semantic_contract_checks += 1;
        if self.window.binding_identity != authority.binding_identity() {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::BindingMismatch,
                counters,
            ));
        }
        if authority.collection_delivery_contract_identity()
            != Some(self.window.collection_delivery_contract_identity.as_str())
        {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::CollectionContractMismatch,
                counters,
            ));
        }
        if self
            .authority
            .as_ref()
            .is_some_and(|current| !current.is_same_current_authority_as(authority))
        {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::WrongLease,
                counters,
            ));
        }
        self.window = self.window.targetized(
            authority.capability_identity(),
            authority.capability_generation(),
        );
        self.window.source_identity = authority.source_identity().to_string();
        self.authority = Some(authority.clone());
        Ok(())
    }

    pub fn plan_patch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admitted: &WorthQueryAdmittedConsumerInvalidation<'_>,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryCollectionDeliveryOutcome {
        super::planning::plan(execution, self, admitted, workspace)
    }

    pub fn apply_patch(
        &mut self,
        patch: WorthQueryCollectionPatch,
    ) -> Result<WorthQueryCollectionPatchApplicationReceipt, WorthQueryCollectionDeliveryDenial>
    {
        let mut counters = WorthQueryCollectionDeliveryCounters::default();
        self.validate_patch(&patch, &mut counters)?;
        Ok(self.commit_patch(patch, counters))
    }

    pub(crate) fn apply_granular_maintenance(
        &mut self,
        pending: super::WorthQueryPendingCollectionStateMutation,
    ) {
        self.index.apply(pending.delta);
        self.window = pending.next;
        self.last_maintenance_ordinal = Some(pending.next_maintenance_ordinal);
    }

    pub fn current_rows(&self) -> &[WorthQueryCollectionRowHandle] {
        self.window.rows()
    }

    fn validate_patch(
        &self,
        patch: &WorthQueryCollectionPatch,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<(), WorthQueryCollectionDeliveryDenial> {
        self.validate_patch_lease(patch, counters)?;
        self.validate_patch_window(patch, counters)?;
        self.validate_patch_order(patch, counters)
    }

    fn validate_patch_lease(
        &self,
        patch: &WorthQueryCollectionPatch,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<(), WorthQueryCollectionDeliveryDenial> {
        counters.lease_checks += 1;
        let Some(authority) = &self.authority else {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::WrongLease,
                *counters,
            ));
        };
        if !authority.is_same_current_authority_as(&patch.authority) {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::WrongLease,
                *counters,
            ));
        }
        if self.reset_pending {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::ResetPending,
                *counters,
            ));
        }
        Ok(())
    }

    fn validate_patch_window(
        &self,
        patch: &WorthQueryCollectionPatch,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<(), WorthQueryCollectionDeliveryDenial> {
        counters.generation_checks += 1;
        if self.window.capability_identity != patch.next.capability_identity
            || self.window.capability_generation != patch.next.capability_generation
        {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::CapabilityGenerationMismatch,
                *counters,
            ));
        }
        counters.cursor_checks += 1;
        if self.window.cursor() != &patch.prior_cursor {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::CursorMismatch,
                *counters,
            ));
        }
        counters.semantic_contract_checks += 1;
        if self.window.window_contract_identity != patch.next.window_contract_identity {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::WindowContractMismatch,
                *counters,
            ));
        }
        Ok(())
    }

    fn validate_patch_order(
        &self,
        patch: &WorthQueryCollectionPatch,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<(), WorthQueryCollectionDeliveryDenial> {
        if self
            .last_maintenance_ordinal
            .is_some_and(|last| patch.maintenance_ordinal <= last)
        {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::DuplicateOrReorderedDelivery,
                *counters,
            ));
        }
        counters.pending_patch_checks += 1;
        if self.pending_maintenance_ordinal != Some(patch.maintenance_ordinal) {
            return Err(denial(
                WorthQueryCollectionDeliveryDenialKind::SupersededPatch,
                *counters,
            ));
        }
        Ok(())
    }

    fn commit_patch(
        &mut self,
        patch: WorthQueryCollectionPatch,
        mut counters: WorthQueryCollectionDeliveryCounters,
    ) -> WorthQueryCollectionPatchApplicationReceipt {
        carry_planning_counters(&mut counters, patch.counters);
        let operations = patch.operations;
        let reset_required = operations.iter().any(|operation| {
            matches!(
                operation,
                crate::domain_installation::WorthQueryCollectionPatchOperation::ResetRequired { .. }
            )
        });
        let facts = patch.facts;
        let foundational_invalidation = patch.foundational_invalidation;
        let index_delta = patch.index_delta;
        let maintenance_ordinal = patch.maintenance_ordinal;
        if !reset_required {
            self.window = patch.next;
            if let Some(delta) = index_delta {
                self.index.apply(delta);
            }
        } else {
            self.reset_pending = true;
        }
        self.last_maintenance_ordinal = Some(maintenance_ordinal);
        self.pending_maintenance_ordinal = None;
        WorthQueryCollectionPatchApplicationReceipt::new(
            super::model::WorthQueryCollectionPatchApplicationParts {
                operations,
                facts,
                foundational_invalidation,
                maintenance_ordinal,
                counters,
                reset_required,
            },
        )
    }

    pub fn rows(&self) -> &[WorthQueryCollectionRowHandle] {
        self.window.rows()
    }

    pub const fn result_state(&self) -> WorthQueryOperationResultState {
        self.window.result_state()
    }

    pub fn continuation(&self) -> &crate::domain_installation::WorthQueryCollectionContinuation {
        self.window.continuation()
    }

    pub fn warnings(&self) -> &[crate::domain_installation::WorthQueryCollectionWindowWarning] {
        self.window.warnings()
    }

    pub const fn reset_pending(&self) -> bool {
        self.reset_pending
    }

    pub fn binding_identity_evidence(&self) -> crate::WorthQueryEvidenceIdentity {
        collection_evidence("collection-binding")
            .field_value(
                crate::WorthQueryEvidenceTag::new("binding"),
                &self.window.binding_identity,
            )
            .field_value(
                crate::WorthQueryEvidenceTag::new("basis"),
                &self.window.basis_identity,
            )
            .seal()
    }

    pub fn source_generation_identity_evidence(&self) -> crate::WorthQueryEvidenceIdentity {
        collection_evidence("collection-source-generation")
            .field_value(
                crate::WorthQueryEvidenceTag::new("source"),
                &self.window.source_identity,
            )
            .field_value(
                crate::WorthQueryEvidenceTag::new("generation"),
                self.window.capability_generation.ordinal().to_string(),
            )
            .field_value(
                crate::WorthQueryEvidenceTag::new("maintenance"),
                maintenance_ordinal(self.last_maintenance_ordinal),
            )
            .seal()
    }

    pub fn result_generation_identity_evidence(&self) -> crate::WorthQueryEvidenceIdentity {
        collection_evidence("collection-result-generation")
            .field_value(
                crate::WorthQueryEvidenceTag::new("result-shape"),
                &self.window.result_shape_identity,
            )
            .field_evidence_identity(
                crate::WorthQueryEvidenceTag::new("source-generation"),
                &self.source_generation_identity_evidence(),
            )
            .field_value(
                crate::WorthQueryEvidenceTag::new("maintenance"),
                maintenance_ordinal(self.last_maintenance_ordinal),
            )
            .seal()
    }
}
