//! Tuple composition and the existing temporal operation contracts.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct TemporalHostSchema;
impl Fragment for TemporalHostSchema {
    const NAME: &'static str = "TemporalHostSchema";
    type Bindings = NoBindings;
    type Members = (TemporalHostContribution,);
}
impl ApplicationSchema for TemporalHostSchema {
    const OWNER: &'static str = "temporal_host_courtroom";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl ApplicationSchemaComposition for TemporalHostSchema {
    type Contributions = <Self as Fragment>::Members;
}
impl TemporalHostSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct TemporalHostContribution;
impl ApplicationSchemaContribution<TemporalHostSchema> for TemporalHostContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.host.courtroom.temporal.v1");
    type Members = (
        TemporalModel,
        TemporalPrincipalBinding,
        TemporalAmendmentEffect,
        TemporalExecutionEffect,
        (
            ExecuteTemporal,
            AmendTemporal,
            AmendTemporalAndPublishDefinition,
            RevokeTemporalPrincipal,
        ),
        (
            TemporalIntentQuery,
            TemporalIntentCurrentReadBinding,
            TemporalIntentLiveQuery,
        ),
    );
}
impl TemporalHostContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<TemporalHostSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct TemporalPrincipalBinding;
impl TemporalPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        TemporalHostSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "TemporalPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}
impl Registration<TemporalHostSchema> for TemporalPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
impl Registration<TemporalHostSchema> for TemporalAmendmentEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<TemporalHostSchema> for TemporalExecutionEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<TemporalHostSchema> for TemporalIntentQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(temporal_intent_query_definition()))
    }
}
impl Registration<TemporalHostSchema> for TemporalIntentCurrentReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
impl Registration<TemporalHostSchema> for TemporalIntentLiveQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(temporal_intent_live_query_definition()))
    }
}
impl Registration<TemporalHostSchema> for ExecuteTemporal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_field(operation, IntentIdentityField::at("IntentIdentityField"))
            .operation_read_field(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_read_field(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_read_field(operation, IntentEffectField::at("IntentEffectField"))
            .operation_write(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_write(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_write(operation, IntentEffectField::at("IntentEffectField"))
            .operation_emit(operation, TemporalExecutionEffect::reference()))
    }
}
impl Registration<TemporalHostSchema> for AmendTemporal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, SCALED_AMENDMENT_PROJECTION_WORK_BUDGET)
            .operation_read_field(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_read_field(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_read_field(operation, IntentGateField::at("IntentGateField"))
            .operation_read_field(operation, IntentDueField::at("IntentDueField"))
            .operation_read_field(operation, IntentInputField::at("IntentInputField"))
            .operation_write(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_write(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_write(operation, IntentGateField::at("IntentGateField"))
            .operation_write(operation, IntentDueField::at("IntentDueField"))
            .operation_write(operation, IntentInputField::at("IntentInputField")))
    }
}
impl Registration<TemporalHostSchema> for AmendTemporalAndPublishDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect(
                        TemporalAmendmentEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new("temporal-amendment")
                            .unwrap(),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 12)
            .operation_read_field(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_read_field(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_read_field(operation, IntentGateField::at("IntentGateField"))
            .operation_read_field(operation, IntentDueField::at("IntentDueField"))
            .operation_read_field(operation, IntentInputField::at("IntentInputField"))
            .operation_write(operation, IntentRevisionField::at("IntentRevisionField"))
            .operation_write(operation, IntentLifecycleField::at("IntentLifecycleField"))
            .operation_write(operation, IntentGateField::at("IntentGateField"))
            .operation_write(operation, IntentDueField::at("IntentDueField"))
            .operation_write(operation, IntentInputField::at("IntentInputField"))
            .operation_emit(operation, TemporalAmendmentEffect::reference()))
    }
}
impl Registration<TemporalHostSchema> for RevokeTemporalPrincipal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 3)
            .operation_read_field(operation, MappingStatusField::at("MappingStatusField"))
            .operation_write(operation, MappingStatusField::at("MappingStatusField")))
    }
}
