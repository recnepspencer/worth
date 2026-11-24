//! Minimal external and inbound schema fixture for omitted capability admission tests.
use std::num::NonZeroU64;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::{
    application_operation::*,
    application_schema::{
        fragment::{self, FieldSelection, Fragment, NoBindings, Registration},
        *,
    },
    authentication::{WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus},
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryInvariantMutationTarget,
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    },
};
use worth_query_testkit_host::facade::host::application_home::{
    native_mutation::{BalanceDenialBinding, BalanceValueBinding},
    schema::{
        Account, AccountIdentity, ExternalIdentity, ExternalMapping, HomeRecords, MappingTarget,
        Principal, PrincipalIdentity,
    },
};

pub(super) type Number = FieldSelection<HomeRecords, Account, AccountIdentity, String>;
pub(super) type Balance = FieldSelection<HomeRecords, Account, AccountIdentity, u64>;
pub(super) type Scope = ApplicationMutationFieldScope<Number>;

pub(super) struct OmittedSchema;
impl Fragment for OmittedSchema {
    const NAME: &'static str = "OmittedExternalCapabilitySchema";
    type Bindings = NoBindings;
    type Members = (super::OmittedSurface,);
}
impl ApplicationSchema for OmittedSchema {
    const OWNER: &'static str = "host_omitted_external_capability_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for OmittedSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub(super) struct OmittedIdentityBinding;
impl OmittedIdentityBinding {
    pub(super) fn reference() -> ApplicationPrincipalBindingRef<
        OmittedSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "OmittedIdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    FieldSelection::<
                        HomeRecords,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryExternalPrincipalIdentity,
                    >::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    FieldSelection::<
                        HomeRecords,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryPrincipalMappingStatus,
                    >::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(fragment::relation(
                    HomeRecords,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    FieldSelection::<HomeRecords, Principal, PrincipalIdentity, u64>::at(
                        "PrincipalIdentityField",
                    ),
                ),
            },
        )
    }
}
impl Registration<OmittedSchema> for OmittedIdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

worth_query_structured_value_binding!(pub(super) OmittedNoticePayload for u64 {
    identity: "worth.query.host.omitted-capability.notice-payload.v1"
});
impl ApplicationExternalEffectBinding for OmittedNoticePayload {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("worth.query.host.omitted-capability.notice"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 8;
    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }
}
worth_query_effect!(pub(super) OmittedNoticeEffect for OmittedSchema, payload OmittedNoticePayload);
impl Registration<OmittedSchema> for OmittedNoticeEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}

worth_query_operation!(pub(super) OmittedNotifyOp for OmittedSchema, input BalanceValueBinding);
worth_query_operation_reads!(OmittedNotifyOp => [Number, Balance]);
worth_query_operation_writes!(OmittedNotifyOp => [Balance]);
worth_query_operation_emits!(OmittedNotifyOp => [OmittedNoticeEffect]);
impl Registration<OmittedSchema> for OmittedNotifyOp {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let op = Self::reference();
        Ok(builder
            .operation(
                op.definition()
                    .external_effect(
                        OmittedNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new("omitted-notify-family")
                            .unwrap(),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(op, 4)
            .operation_read_field(op, Number::at("AccountNumber"))
            .operation_read_field(op, Balance::at("Balance"))
            .operation_write(op, Balance::at("Balance"))
            .operation_emit(op, OmittedNoticeEffect::reference()))
    }
}

worth_query_operation!(pub(super) PartialInboundOp for OmittedSchema, input BalanceValueBinding);
worth_query_operation_emits!(PartialInboundOp => [OmittedNoticeEffect]);
impl Registration<OmittedSchema> for PartialInboundOp {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let op = Self::reference();
        let n = |v| NonZeroU64::new(v).unwrap();
        let inbound = ApplicationInboundOccurrenceBinding::new(
            ApplicationInboundOccurrenceProtocol::new(
                OmittedNoticePayload::PROTOCOL.identity().clone(),
                OmittedNoticePayload::PROTOCOL.version(),
            ),
            "omitted-inbound-source",
            ApplicationInboundOccurrenceLimits {
                maximum_envelope_bytes: n(1024),
                maximum_verifier_work: n(4096),
                maximum_payload_bytes: n(8),
                maximum_outstanding_dispatch_provenance: n(8),
                maximum_accepted_occurrences: n(8),
                maximum_accepted_bytes: n(8192),
                maximum_concurrent_publications: n(2),
                maximum_discovery_work: n(16),
                replay_window_milliseconds: n(60_000),
                maximum_cleanup_work: n(16),
            },
        )
        .unwrap();
        Ok(builder
            .operation(
                op.definition()
                    .external_effect_with_inbound(
                        OmittedNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new("omitted-inbound-family")
                            .unwrap(),
                        inbound,
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(op, 1)
            .operation_emit(op, OmittedNoticeEffect::reference()))
    }
}

pub(super) struct OmittedNotifyBinding;
impl ApplicationMutationAdmissionBinding<OmittedSchema> for OmittedNotifyBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<OmittedSchema> for OmittedNotifyBinding {
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = OmittedNotifyOp;
    type Decision = WorthQueryInvariantMutationTarget<OmittedSchema, Account>;
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = OmittedIdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.host.omitted.notify-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.host.omitted.notify-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.host.omitted.notify-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        OmittedSchema,
        OmittedIdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        OmittedIdentityBinding::reference()
    }
}
impl Registration<OmittedSchema> for OmittedNotifyBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}

pub(super) struct PartialInboundBinding;
impl ApplicationMutationAdmissionBinding<OmittedSchema> for PartialInboundBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<OmittedSchema> for PartialInboundBinding {
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = PartialInboundOp;
    type Decision = ();
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = OmittedIdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.host.omitted.inbound-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.host.omitted.inbound-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.host.omitted.inbound-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        OmittedSchema,
        OmittedIdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        OmittedIdentityBinding::reference()
    }
}
impl Registration<OmittedSchema> for PartialInboundBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<OmittedSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OmittedSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}

#[derive(Clone, Debug)]
pub(super) struct OmittedNotifyIntent(pub(super) u64);
impl ApplicationMutationIntent<OmittedSchema> for OmittedNotifyIntent {
    type Binding = OmittedNotifyBinding;
    fn input(&self) -> &u64 {
        &self.0
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Number::at("AccountNumber"), "account-001".to_owned())
    }
}

#[derive(Clone, Debug)]
pub(super) struct PartialInboundIntent(pub(super) u64);
impl ApplicationMutationIntent<OmittedSchema> for PartialInboundIntent {
    type Binding = PartialInboundBinding;
    fn input(&self) -> &u64 {
        &self.0
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Number::at("AccountNumber"), "account-001".to_owned())
    }
}

pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<OmittedSchema>,
    schema: &WorthQueryInstalledApplicationSchema<OmittedSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(OmittedIdentityBinding::reference())
        .unwrap();
    graph.bind_principal(
        &binding,
        WorthQueryApplicationPrincipalKey::new("host-principal").unwrap(),
        1_u64,
        WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject").unwrap(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            fragment::entity(HomeRecords, Account),
            WorthQueryApplicationEntityKey::new("account-row").unwrap(),
        )
        .field(Number::at("AccountNumber"), "account-001".to_owned())
        .field(Balance::at("Balance"), 7_u64),
    )?;
    Ok(())
}
