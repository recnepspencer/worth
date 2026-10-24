//! Original move-only descriptor drives complete ordinary discovered delivery.
use super::*;
use crate::application_program::discovered_delivery::{Action, Program, QueryDelivery};
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent,
    application_program::ApplicationProgramAuthoring,
    application_schema::{ApplicationEncodedScalarValue, TypedMutationPreconditions},
};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationAdvanceDenial as Denial,
    WorthQueryOperationProgress as Progress,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, BodyKey,
    PlanarMutationScope, PlanarSourceAdjustmentBinding,
};
macro_rules! declared_world {
    ($world:ident,$request:ident,$delivery:ty) => {
        let program=ApplicationProgramAuthoring::<ConsumerSchema,Program<$delivery>>::begin().validated_program().unwrap();
        let grants=[crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<$delivery>>("model-owner","model-owner")];
        let $world=installation::install_selected_program(&crate::installed_schema(),worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(),&grants,program).unwrap();
        let adapter=authentication::admit($world.application.installed_schema());
        let scope=authentication::request_scope();
        let principal=authentication::block_on(adapter.authenticate(authentication::LocalCredential::issued_for_model_owner(),&scope)).unwrap();
        let $request=$world.application.request(&principal,&scope);
    };
}
mod affinity;
mod delivery;
mod membership;
mod replay;
mod retention;
mod unrelated;

// The initiating intent cannot be cloned or reconstructed by the ordinary owner.
struct Original(PlanarSourceAdjustment);
impl ApplicationMutationIntent<ConsumerSchema> for Original {
    type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
    fn input(&self) -> &PlanarSourceAdjustment {
        &self.0
    }
    fn scope_binding(&self) -> PlanarMutationScope<ConsumerSchema> {
        <PlanarSourceAdjustment as ApplicationMutationIntent<ConsumerSchema>>::scope_binding(
            &self.0,
        )
    }
}

#[test]
fn discovered_ordinary_executable_entry_examples_use_installed_delivery_contracts() {
    let installed = crate::installed_schema();
    super::discovered::performed_source_discovers_required_root(&installed);
    super::discovered::isolated_source_settles_without_roots(&installed);
}
