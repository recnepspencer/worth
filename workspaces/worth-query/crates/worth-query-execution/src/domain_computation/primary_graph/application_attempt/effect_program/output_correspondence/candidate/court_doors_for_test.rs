//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::role_use::OutputRoleUse;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::model::WorthQueryApplicationEffectEntity;

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    #[doc(hidden)]
    pub fn bind_for_test<Schema, Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::bind::<Schema, Entity>(self, role, target, program)
    }
}

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    #[doc(hidden)]
    pub fn prepare_test_contract_for_test<Schema, Contract>(&mut self)
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    Contract: worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract<Schema>,{
        Self::prepare_test_contract::<Schema, Contract>(self)
    }
}

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    #[doc(hidden)]
    pub fn seal_with_for_test(
        self,
        resolve_created: impl Fn(
            &worth_relational::facade::transactions::CreatedEntityRef,
        ) -> Option<EntityId>,
    ) -> WorthQueryApplicationOutputCorrespondence {
        Self::seal_with(self, resolve_created)
    }
}
