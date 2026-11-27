//! Governed workflow reads: a subject read over one born request, reached
//! from a company grant through the request's birth edge, and the required
//! definition-wide read over the company. Neither grants a command right;
//! `subject_contract` is the same rooted shape the instance steps use.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context_entity_slot, worth_query_operation,
    worth_query_structured_value_binding, worth_query_workflow_read_binding,
};

use super::control::{CompanyControl, CompanyControlInput};
use super::grants::{self, field, CompanyWorkflowContext, Projection, ENUMERATE, READ};
use super::schema::*;

/// The admitted subject of a read or step: one request, named with the
/// company it was born in.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequestSubject {
    pub company: String,
    pub request: u64,
}

worth_query_structured_value_binding!(pub RequestSubjectBinding for RequestSubject {
    identity: "workflow-birth-time.dx.subject-read.input.v1"
});
worth_query_capability_context_entity_slot!(pub SubjectCompany in BirthSchema, CompanyWorkflowContext => Company);
worth_query_capability!(pub SubjectReadCapability in BirthSchema);
worth_query_capability!(pub DefinitionReadCapability in BirthSchema);
worth_query_operation!(pub SubjectReadOp for BirthSchema, input RequestSubjectBinding);
worth_query_operation!(pub DefinitionReadOp for BirthSchema, input CompanyControlInput);

/// A capability over one request, with the grant action it requires.
pub trait SubjectAction: ApplicationCapabilityMarkerIdentity<Schema = BirthSchema> {
    const ACTION: &'static str;
}
impl SubjectAction for SubjectReadCapability {
    const ACTION: &'static str = READ;
}

impl<C: SubjectAction> ApplicationCapabilityRequest<BirthSchema, C> for RequestSubject {
    type Scope = Request;
    type Context = CompanyWorkflowContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BirthSchema, Request, CompanyWorkflowContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let request = ApplicationEncodedScalarValue::try_new(self.request)
            .map_err(|_| ApplicationCapabilityRequestProjectionDenial::input_variant("request"))?;
        let company = ApplicationCapabilityEntitySelector::new(
            CompanyFacts::company_id(),
            grants::encoded(&self.company),
        )?;
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(RequestFacts::request_id(), request)?,
            grants::encoded(C::ACTION),
            grants::encoded("purchase-request"),
            ApplicationCapabilityRequestContext::new(CompanyWorkflowContext::reference())
                .entity(SubjectCompany::reference(), company),
        ))
    }
}

impl ApplicationCapabilityRequest<BirthSchema, DefinitionReadCapability> for CompanyControl {
    type Scope = Company;
    type Context = CompanyWorkflowContext;
    fn capability_request(
        &self,
    ) -> Result<Projection, ApplicationCapabilityRequestProjectionDenial> {
        grants::request(&self.company, ENUMERATE)
    }
}

// The native declaration: input and codec are each operation's own, and the
// principal types are `Desktop`'s, so no binding repeats them.
worth_query_workflow_read_binding!(pub SubjectRead in BirthSchema {
    identity: "workflow-birth-time.dx.subject-read.v1",
    capability: SubjectReadCapability,
    operation: SubjectReadOp,
    binding_context: Desktop,
});
worth_query_workflow_read_binding!(pub DefinitionRead in BirthSchema {
    identity: "workflow-birth-time.dx.definition-read.v1",
    capability: DefinitionReadCapability,
    operation: DefinitionReadOp,
    binding_context: Desktop,
});

/// Owner → grant → company ← request: the company grant reaches each request
/// born there, and the request's company must be the requested one.
pub fn subject_contract<Cap, Op>(
    capability: ApplicationCapabilityRef<BirthSchema, Cap>,
    operation: ApplicationOperationRef<BirthSchema, Op, RequestSubject>,
) -> ApplicationCapabilityContract<BirthSchema, Cap, Op, RequestSubject>
where
    Cap: SubjectAction + 'static,
    Op: ApplicationOperationMarkerIdentity<BirthSchema, InputBinding = RequestSubjectBinding>
        + 'static,
{
    let path = ApplicationAuthorizationPathBuilder::from_principal(Owner::reference())
        .forward(Grantee::reference())
        .forward(GrantResource::reference())
        .reverse(RequestCompany::reference())
        .allow(Request::reference());
    let allow = ApplicationCapabilityGraphClause::new(path).anchored([
        ApplicationCapabilityPathContextAnchor::after_forward(
            GrantResource::reference(),
            SubjectCompany::reference(),
        ),
    ]);
    let resource = ApplicationCapabilityResourcePath::root(
        GrantResource::reference(),
        SubjectCompany::reference(),
    )
    .reverse(RequestCompany::reference())
    .finish()
    .expect("the request resource path is finite");
    grants::scoped_contract(
        capability,
        operation,
        Cap::ACTION,
        allow,
        resource,
        field(RequestFacts::company_code()),
    )
}

/// Declares both read operations and their contracts.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
) -> ApplicationSchemaDeclarationBuilder<BirthSchema> {
    let (subject, definition) = (SubjectReadOp::reference(), DefinitionReadOp::reference());
    schema
        .capability_context_entity_slot(SubjectCompany::reference())
        .operation(
            subject
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(subject, 128)
        .operation_read_field(subject, CompanyFacts::company_id())
        .operation_read_field(subject, RequestFacts::request_id())
        .operation_read_field(subject, RequestFacts::company_code())
        .capability(subject_contract(
            SubjectReadCapability::reference(),
            subject,
        ))
        .operation(
            definition
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(definition, 64)
        .operation_read_field(definition, CompanyFacts::company_id())
        .capability(grants::contract::<_, _, CompanyControl>(
            DefinitionReadCapability::reference(),
            definition,
            ENUMERATE,
        ))
}
