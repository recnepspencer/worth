//! A real field-valued grant controls each resource facet independently of Close.
use super::super::{grants, host::Host, schema::*};
use worth_query_decl::facade::{
    application_capability::*, application_schema::*, worth_query_capability, worth_query_field,
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::{admission::authenticated_principal::*, primary_graph::*};

worth_query_field!(pub ResourceDisclosureField for CapabilitySchema, grants::Grant, grants::GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_capability!(pub ResourceDisclosure in CapabilitySchema);
#[derive(serde::Serialize)]
pub struct MetadataInput {
    identity: String,
}
worth_query_structured_value_binding!(pub MetadataInputBinding for MetadataInput { identity: "capability-retention.resource-metadata-input.v1" });
worth_query_operation!(pub InspectResource for CapabilitySchema, input MetadataInputBinding);
worth_query_operation_reads!(InspectResource => [DocumentIdentityField, ResourceDisclosureField]);

impl ApplicationCapabilityRequest<CapabilitySchema, ResourceDisclosure> for MetadataInput {
    type Scope = Document;
    type Context = grants::GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, grants::GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(grants::projection(1, &self.identity).field(
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap(),
        ))
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    schema
        .field(
            grants::Grant::reference(),
            ResourceDisclosureField::reference(),
        )
        .operation(
            InspectResource::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(InspectResource::reference(), 32)
        .operation_read_field(
            InspectResource::reference(),
            DocumentIdentityField::reference(),
        )
        .capability(grants::grant_contract_for_field(
            ResourceDisclosure::reference(),
            InspectResource::reference(),
            1,
            1,
            ApplicationCapabilityRecordedActorRule::not_applicable(),
            ApplicationCapabilityFieldDimension::bound(ResourceDisclosureField::reference()),
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        ResourceDisclosureField::reference(),
                        [
                            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
                                .unwrap(),
                        ],
                    ),
                ]),
            ]),
        ))
}

pub type Admitted = WorthQueryAdmittedApplicationCapabilityAccess<
    CapabilitySchema,
    ResourceDisclosure,
    InspectResource,
    MetadataInput,
>;

pub fn admit(
    host: &Host,
    actor: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
) -> Result<Admitted, WorthQueryOperationAuthorizationDenial> {
    admit_on(
        host,
        actor,
        scope,
        host.application.runtime().current_world(),
    )
}

pub(super) fn admit_on(
    host: &Host,
    actor: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) -> Result<Admitted, WorthQueryOperationAuthorizationDenial> {
    host.application
        .runtime()
        .with_application_advancement(scope, |phase| {
            let selected = host
                .application
                .runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap();
            let installed = host.application.installed_schema();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &installed
                        .principal_binding(DocumentPrincipalBinding::reference())
                        .unwrap(),
                    actor,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            selected.admit_capability_access(
                &phase,
                &principal,
                &installed
                    .capability(
                        ResourceDisclosure::reference(),
                        InspectResource::reference(),
                    )
                    .unwrap(),
                MetadataInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                },
                scope,
            )
        })
        .unwrap()
}
