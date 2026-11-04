//! Mutation truth and its authorization source have separate owner custody.
use super::*;
use crate::basis::WorthQueryProductObservationLease;

pub(super) struct MutationProductBasis {
    pub(super) lease: Option<WorthQueryApplicationSnapshotLease>,
    control: Option<WorthQueryProductObservationLease>,
    coupled_handler: Option<
        crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation,
    >,
    authentication: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl MutationProductBasis {
    pub(super) fn authorization_product(&self) -> Option<&WorthQueryProductObservationLease> {
        match &self.control {
            Some(control) => Some(control),
            None => self
                .lease
                .as_ref()
                .map(|lease| lease.product().read_lease_ref()),
        }
    }

    pub(super) fn admits_replacement_authorization(
        &self,
        authorization: &WorthQueryGraphWorkBranchAffinity,
        replacement: &WorthQueryApplicationSnapshotLease,
    ) -> bool {
        match &self.control {
            Some(control) => {
                *authorization == WorthQueryGraphWorkBranchAffinity::from_product(control)
            }
            None => authorization.admits_snapshot(replacement.snapshot()),
        }
    }
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn arm_coupled_handler(&mut self) {
        let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis else {
            unreachable!("typed mutation admission")
        };
        assert!(
            basis.coupled_handler.is_none(),
            "one producer handler preparation"
        );
        basis.coupled_handler = Some(crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation::new());
    }
    pub(in crate::domain_computation) fn coupled_handler(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation>
    {
        let WorthQueryGraphWorkBasis::Mutation(basis) = &self.basis else {
            return None;
        };
        basis.coupled_handler.as_ref()
    }
    pub(in crate::domain_computation) fn coupled_handler_mut(
        &mut self,
    ) -> Option<
        &mut crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation,
    > {
        let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis else {
            return None;
        };
        basis.coupled_handler.as_mut()
    }
    pub(in crate::domain_computation) fn take_coupled_handler(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation>
    {
        let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis else {
            return None;
        };
        basis.coupled_handler.take()
    }

    pub(in crate::domain_computation) fn with_actor_authentication(
        mut self,
        proof: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        if let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis {
            basis.authentication = Some(proof);
        }
        self
    }
    pub(in crate::domain_computation) fn authenticating_principal<Schema: 'static>(&self) -> Option<&worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>>{
        let WorthQueryGraphWorkBasis::Mutation(basis) = &self.basis else {
            return None;
        };
        basis.authentication.as_ref()?.downcast_ref()
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        Self::start_mutation_with_control(
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            lease,
            None,
            provider,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation_with_control(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        control: Option<WorthQueryProductObservationLease>,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let branch = WorthQueryGraphWorkBranchAffinity::from_snapshot(lease.snapshot());
        if !branch.admits_snapshot(lease.snapshot()) {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
        }
        let authorization = control.as_ref().map_or_else(
            || branch.clone(),
            WorthQueryGraphWorkBranchAffinity::from_product,
        );
        Self::start(
            WorthQueryGraphWorkSessionIdentity::mint()
                .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted)?,
            None,
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            branch,
            authorization,
            WorthQueryGraphWorkBasis::Mutation(MutationProductBasis {
                lease: Some(lease),
                control,
                authentication: None,
                coupled_handler: None,
            }),
            provider,
        )
    }
}
