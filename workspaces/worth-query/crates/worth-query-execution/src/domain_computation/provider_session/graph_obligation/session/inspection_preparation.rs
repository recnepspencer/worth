//! Inspection binds its original target separately from current authorization truth.
use super::*;
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget;
use std::sync::Arc;
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkIntentKind;

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn inspection_start_backing_bytes(
        basis: &WorthQueryApplicationBasisIdentity,
        product: &crate::basis::WorthQueryProductObservationLease,
        provider: &str,
    ) -> Option<u64> {
        let read_branch = basis.branch_id().0.len();
        let security_branch = product.relational_basis_descriptor().branch_id().0.len();
        let branch_names = read_branch.checked_add(security_branch)?;
        let strings = branch_names
            .checked_add(provider.len())?
            .checked_add("original-query-observation-metadata-v1".len())?;
        // Each actual branch affinity owns one String and one flat Arc<str>.
        let arc_string = |length| {
            std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
                .extend(std::alloc::Layout::array::<u8>(length).ok()?)
                .ok()
                .map(|(layout, _)| layout.pad_to_align().size())
        };
        let arcs = arc_string(read_branch)?.checked_add(arc_string(security_branch)?)?;
        basis
            .retained_clone_heap_bytes()?
            .checked_add(u64::try_from(strings.checked_add(arcs)?).ok()?)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_inspection(
        reserved: WorthQueryPreparedProductReadSessionIdentity,
        target: Arc<WorthQueryApplicationInspectionTarget>,
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        port: WorthQueryGraphReadOwnerPort,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let contract = target.contract();
        let obligations = contract.graph_obligations();
        if plan.intent().kind() != WorthQueryGraphWorkIntentKind::ApplicationInspectionRead
            || plan.binding_identity() != target.schema_binding()
            || plan.obligation_identity() != obligations.identity()
            || target.runtime_authority() != runtime.as_u64()
        {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        let binding = target.schema_binding().clone();
        let obligation = obligations.identity().clone();
        // Both leases are the freshly selected CURRENT Product. No original
        // historical Product custody is invented from the descriptive target.
        let authorization_product = product.retained_clone();
        Self::start_product_read(
            reserved,
            ProductReadIntent::Inspection(target),
            plan,
            runtime,
            &binding,
            &obligation,
            "original-query-observation-metadata-v1",
            principal,
            access,
            basis,
            product,
            &authorization_product,
            provider,
            port,
        )
    }

    pub(in crate::domain_computation) fn inspection_binding(
        &self,
    ) -> Option<(
        &Arc<WorthQueryApplicationInspectionTarget>,
        &WorthQueryApplicationBasisIdentity,
    )> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead {
                intent: ProductReadIntent::Inspection(target),
                read,
            } => Some((target, &read.identity)),
            _ => None,
        }
    }
}
