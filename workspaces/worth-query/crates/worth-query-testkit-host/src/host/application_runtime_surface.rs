pub use worth_query_declaration::facade::application_schema::{
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
pub use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::declaration::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
pub use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey;
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryPrincipalResolutionMode,
};
pub use worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller;

worth_query_application_schema! {
    pub schema HostIdentitySchema {
        owner: host_identity_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .aspect(Account::reference(), AccountIdentity::reference())
                .field(Account::reference(), AccountNumber::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
        }
    }
}

worth_query_entity!(pub ExternalMapping for HostIdentitySchema);
worth_query_entity!(pub Principal for HostIdentitySchema);
worth_query_entity!(pub Account for HostIdentitySchema);
worth_query_aspect!(pub ExternalIdentity for HostIdentitySchema, ExternalMapping; identity = AspectIdentity(0x9161103e), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for HostIdentitySchema, Principal; identity = AspectIdentity(0x9161103f), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for HostIdentitySchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in HostIdentitySchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in HostIdentitySchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_aspect!(pub AccountIdentity for HostIdentitySchema, Account; identity = AspectIdentity(0x91611040), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountNumber for HostIdentitySchema, Account, AccountIdentity:
    String => StringApplicationValueBinding, read_only, equality
);

pub use std::time::{Duration, Instant};
