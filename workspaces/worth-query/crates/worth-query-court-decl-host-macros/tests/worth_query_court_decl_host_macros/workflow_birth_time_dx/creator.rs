//! The creator: one body-only birth that creates a request in a company,
//! stamps its `created_at` from trusted admission time and enrolls its fixed
//! exactly-one Create output in `RequestWorkflow`, all in one commit. It is a
//! program-required, non-control mutation with its own capability.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationSourceCompletion,
};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_mutation_binding, worth_query_operation, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;

use super::grants::{self, OpenRequestCapability, Projection};
use super::schema::*;
use super::workflow::RequestWorkflow;

/// The caller names the company and the request; the body stamps the time.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenRequest {
    pub company: String,
    pub request: u64,
}

worth_query_structured_value_binding!(pub OpenRequestInput for OpenRequest {
    identity: "workflow-birth-time.dx.open-request.input.v1",
    capture_input: true,
    fields: [required("company", company: String), required("request", request: u64)]
});

worth_query_operation!(
    pub OpenRequestOp for BirthSchema, input OpenRequestInput;
    access { read_fields: [CompanyFacts::company_id()]; };
    reads [Company];
    creates [Request];
    links [RequestCompany]
);

fn company_scope(input: &OpenRequest) -> String {
    input.company.clone()
}

worth_query_mutation_binding!(
    pub OpenRequestBinding for OpenRequest, schema BirthSchema, context Desktop,
    identity "workflow-birth-time.dx.open-request.binding.v1",
    input OpenRequestInput, operation OpenRequestOp,
    result UnitApplicationValueBinding,
    decision WorthQueryOperationBodyDecision<BirthSchema, Company>,
    handler identity "workflow-birth-time.dx.open-request.body.v1",
    admission ApplicationCapabilityAdmission<OpenRequestCapability>,
    program required,
    outputs RequestOutputs,
    scope CompanyId, field CompanyFacts::company_id(), value company_scope,
);

impl ApplicationCapabilityMutationBinding<BirthSchema> for OpenRequestBinding {
    type Capability = OpenRequestCapability;
}

impl ApplicationCapabilityRequest<BirthSchema, OpenRequestCapability> for OpenRequest {
    type Scope = Company;
    type Context = grants::CompanyWorkflowContext;
    fn capability_request(
        &self,
    ) -> Result<Projection, ApplicationCapabilityRequestProjectionDenial> {
        grants::request(&self.company, grants::OPEN)
    }
}

/// The source-completion delivery the program's action retains.
pub struct RequestOpened;
impl ApplicationActionDelivery<BirthSchema, OpenRequestBinding> for RequestOpened {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "workflow-birth-time.dx.open-request.completion.v1";
}

/// The creator's one fixed, exactly-one Create output.
pub struct RequestOutputs;
pub struct CreatedRequest;
impl ApplicationMutationOutputContract<BirthSchema> for RequestOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[CreatedRequest::DESCRIPTOR];
}
impl WorthQueryApplicationOutputRole for CreatedRequest {
    type Schema = BirthSchema;
    type Contract = RequestOutputs;
    type Entity = Request;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-request";
}

fn request_input() -> ApplicationInputField<OpenRequest, u64> {
    ApplicationInputField::for_binding::<OpenRequestInput>("request")
        .expect("the request member is declared")
}

/// Creates the request, stamps its birth instant and enrolls it.
pub fn open_request_operation(
) -> ApplicationOperationDefinition<BirthSchema, OpenRequestOp, OpenRequest> {
    let creation = OpenRequestOp::reference()
        .typed_body()
        .in_scope(Company::reference())
        .create_in_scope(Request::reference(), "request")
        .initialize_input(RequestFacts::request_id(), request_input())
        .initialize_from_admission_time(RequestFacts::created_at())
        .initialize_from_scope(CompanyFacts::company_id(), RequestFacts::company_code())
        .link_to_scope(RequestCompany::reference(), "company")
        .into_inner()
        .expect("the creation admits");
    let body = creation
        .enroll_in_workflow::<RequestWorkflow, OpenRequestBinding, CreatedRequest>()
        .expect("the birth enrollment admits")
        .finish()
        .expect("the birth body admits");
    OpenRequestOp::reference()
        .pure_body_definition(body)
        .expect("the birth definition admits")
}
