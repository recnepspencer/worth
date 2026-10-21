use super::{
    WorthQueryLiveGraphReadAccessDenial, WorthQueryLiveGraphReadAccessPosture,
    WorthQueryLiveGraphReadMaintenanceBudget, WorthQueryLiveGraphReadMutationDeltaScope,
};
use crate::identity::hash_parts;
use crate::runtime::WorthQueryRuntimeLiveSubscriptionInstallation;
#[cfg(test)]
use crate::runtime::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessAdmissionPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveGraphReadAccessPlan {
    digest: String,
    one_shot_access_plan_digest: String,
    one_shot_access_shape_digest: String,
    required_index_digest: String,
    posture: WorthQueryLiveGraphReadAccessPosture,
    maintenance_budget: WorthQueryLiveGraphReadMaintenanceBudget,
    mutation_delta_scope: WorthQueryLiveGraphReadMutationDeltaScope,
    maintenance_equivalence_digest: String,
}

impl WorthQueryLiveGraphReadAccessPlan {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn one_shot_access_plan_digest(&self) -> &str {
        &self.one_shot_access_plan_digest
    }

    pub fn one_shot_access_shape_digest(&self) -> &str {
        &self.one_shot_access_shape_digest
    }

    pub fn required_index_digest(&self) -> &str {
        &self.required_index_digest
    }

    pub fn posture(&self) -> &WorthQueryLiveGraphReadAccessPosture {
        &self.posture
    }

    pub fn maintenance_budget(&self) -> &WorthQueryLiveGraphReadMaintenanceBudget {
        &self.maintenance_budget
    }

    pub fn mutation_delta_scope(&self) -> &WorthQueryLiveGraphReadMutationDeltaScope {
        &self.mutation_delta_scope
    }

    pub fn maintenance_equivalence_digest(&self) -> &str {
        &self.maintenance_equivalence_digest
    }

    #[cfg(test)]
    pub(crate) fn from_one_shot_access_plan(
        one_shot: &WorthQueryAdmittedGraphReadAccessPlan,
        maintenance_budget: WorthQueryLiveGraphReadMaintenanceBudget,
    ) -> Result<Self, WorthQueryLiveGraphReadAccessDenial> {
        let mutation_delta_scope =
            WorthQueryLiveGraphReadMutationDeltaScope::from_one_shot_access_plan(one_shot);
        let posture = live_posture_for_one_shot(
            one_shot.posture(),
            &maintenance_budget,
            mutation_delta_scope.affected_requirement_row_count(),
        );
        if !posture.is_admitted() {
            return Err(WorthQueryLiveGraphReadAccessDenial::new(
                posture,
                one_shot.digest(),
                &maintenance_budget,
                "one-shot graph read access plan is not safely maintainable as a live read",
            ));
        }
        Ok(Self::new(
            one_shot.digest(),
            one_shot
                .admission()
                .requirement_set()
                .access_shape_digest()
                .render_hex(),
            one_shot
                .graph_index_support()
                .requirement_set_digest()
                .render_hex(),
            posture,
            maintenance_budget,
            mutation_delta_scope,
        ))
    }

    pub(crate) fn from_live_installation(
        installation: &WorthQueryRuntimeLiveSubscriptionInstallation,
        maintenance_budget: WorthQueryLiveGraphReadMaintenanceBudget,
    ) -> Result<Self, WorthQueryLiveGraphReadAccessDenial> {
        let one_shot_digest = installation.query_projection().label().to_string();
        let access_shape_digest = installation
            .canonical_result_shape_digest()
            .as_str()
            .to_string();
        let required_index_digest = installation
            .subscription_family_projection()
            .label()
            .to_string();
        let mutation_delta_scope =
            WorthQueryLiveGraphReadMutationDeltaScope::from_subscription_family(
                installation.subscription_family_kind(),
            );
        let posture = live_posture_for_subscription(
            installation.subscription_family_kind(),
            &maintenance_budget,
            mutation_delta_scope.affected_requirement_row_count(),
        );
        if !posture.is_admitted() {
            return Err(WorthQueryLiveGraphReadAccessDenial::new(
                posture,
                one_shot_digest,
                &maintenance_budget,
                "live subscription family requires stronger maintenance support than the admitted budget provides",
            ));
        }
        Ok(Self::new(
            installation.query_projection().label(),
            &access_shape_digest,
            &required_index_digest,
            posture,
            maintenance_budget,
            mutation_delta_scope,
        ))
    }

    fn new(
        one_shot_access_plan_digest: impl Into<String>,
        one_shot_access_shape_digest: impl Into<String>,
        required_index_digest: impl Into<String>,
        posture: WorthQueryLiveGraphReadAccessPosture,
        maintenance_budget: WorthQueryLiveGraphReadMaintenanceBudget,
        mutation_delta_scope: WorthQueryLiveGraphReadMutationDeltaScope,
    ) -> Self {
        let one_shot_access_plan_digest = one_shot_access_plan_digest.into();
        let one_shot_access_shape_digest = one_shot_access_shape_digest.into();
        let required_index_digest = required_index_digest.into();
        let maintenance_equivalence_digest = hash_parts(&[
            "worth_query_live_graph_read_access_equivalence_v1".to_string(),
            format!("shape:{one_shot_access_shape_digest}"),
            format!("indexes:{required_index_digest}"),
            format!("posture:{}", posture.as_str()),
            format!("delta_scope:{}", mutation_delta_scope.delta_scope_digest()),
        ]);
        let digest = hash_parts(&[
            "worth_query_live_graph_read_access_plan_v1".to_string(),
            format!("one_shot_plan:{one_shot_access_plan_digest}"),
            format!("shape:{one_shot_access_shape_digest}"),
            format!("indexes:{required_index_digest}"),
            format!("posture:{}", posture.as_str()),
            format!("budget:{}", maintenance_budget.digest()),
            format!("delta_scope:{}", mutation_delta_scope.delta_scope_digest()),
            format!("equivalence:{maintenance_equivalence_digest}"),
        ]);
        Self {
            digest,
            one_shot_access_plan_digest,
            one_shot_access_shape_digest,
            required_index_digest,
            posture,
            maintenance_budget,
            mutation_delta_scope,
            maintenance_equivalence_digest,
        }
    }
}

#[cfg(test)]
fn live_posture_for_one_shot(
    posture: &WorthQueryGraphReadAccessAdmissionPosture,
    budget: &WorthQueryLiveGraphReadMaintenanceBudget,
    affected_requirement_rows: usize,
) -> WorthQueryLiveGraphReadAccessPosture {
    if affected_requirement_rows > budget.max_requirement_rows() {
        return WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceBudget;
    }
    match posture {
        WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
        | WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex => {
            WorthQueryLiveGraphReadAccessPosture::AdmittedLiveIncrementalMaintenance
        }
        WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired => {
            WorthQueryLiveGraphReadAccessPosture::LivePersistentIndexRequired
        }
        WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired => {
            WorthQueryLiveGraphReadAccessPosture::LiveAsyncMaterializationRequired
        }
        WorthQueryGraphReadAccessAdmissionPosture::StoreBackedCapabilityRequired => {
            WorthQueryLiveGraphReadAccessPosture::LiveStoreBackedCapabilityRequired
        }
        WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired => {
            WorthQueryLiveGraphReadAccessPosture::LiveAccessCapabilityRegistrationRequired
        }
        WorthQueryGraphReadAccessAdmissionPosture::Denied => {
            WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceSupport
        }
    }
}

fn live_posture_for_subscription(
    family: &crate::subscription::QuerySubscriptionFamily,
    budget: &WorthQueryLiveGraphReadMaintenanceBudget,
    affected_requirement_rows: usize,
) -> WorthQueryLiveGraphReadAccessPosture {
    if affected_requirement_rows > budget.max_requirement_rows() {
        return WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceBudget;
    }
    match family {
        crate::subscription::QuerySubscriptionFamily::DetailExact
        | crate::subscription::QuerySubscriptionFamily::CollectionMembership
        | crate::subscription::QuerySubscriptionFamily::InspectorDetailExact => {
            WorthQueryLiveGraphReadAccessPosture::AdmittedLiveIncrementalMaintenance
        }
        crate::subscription::QuerySubscriptionFamily::GroupedCollectionMembership
        | crate::subscription::QuerySubscriptionFamily::BoundedMaterialization
            if budget.admits_snapshot_refresh() =>
        {
            WorthQueryLiveGraphReadAccessPosture::AdmittedLiveSnapshotRefresh
        }
        crate::subscription::QuerySubscriptionFamily::GroupedCollectionMembership
        | crate::subscription::QuerySubscriptionFamily::BoundedMaterialization => {
            WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceBudget
        }
    }
}
