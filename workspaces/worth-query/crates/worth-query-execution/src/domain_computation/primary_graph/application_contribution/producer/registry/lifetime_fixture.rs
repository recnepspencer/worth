//! Fixture declaration carried by a typed demand whose only action is Drop.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::tests::fixture::{
    retained_output_capacity::RetainedFamily, IdentityExecutionSchema as Schema,
};
impl InstalledProducerProvider<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn lifetime_fixture(
    ) -> Arc<Self> {
        let declaration = DeclaredProducerBinding {
            owner: "lifetime".into(),
            identity: "lifetime".into(),
            source_selector: "source".into(),
            output_family: RetainedFamily::IDENTITY.into(),
            output_family_type: TypeId::of::<RetainedFamily>(),
            output_role_descriptors: Vec::new(),
            output_role_families: Vec::new(),
            output_role: "output".into(),
            operation: "unused".into(),
            handler_identity: "unused".into(),
            provider_identity: "unused".into(),
            applicability: Vec::new(),
            supported: Vec::new(),
            required_invariants: Vec::new(),

            reuse_policy: "exact".into(),
            input_reuse: None,
            binding_type: TypeId::of::<RetainedFamily>(),
            source_type: TypeId::of::<()>(),
            operation_binding_type: TypeId::of::<()>(),
            operation_type: TypeId::of::<()>(),
            provider_type: TypeId::of::<()>(),
            output_contract_type: TypeId::of::<()>(),
        };
        let edition = InstalledProducerEdition::from_declaration(
            &declaration,
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .unwrap(),
        )
        .unwrap();
        Arc::new(Self {
            declaration,
            edition,
            value: Arc::new(()),
            executor: super::super::execution::lifetime_fixture::executor(),
        })
    }
}
