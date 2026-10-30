pub use worth_query::facade::domain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryDomain;

impl domain::WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.geometry"
    }

    fn display_name(&self) -> &'static str {
        "Geometry"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ReadVertex;

pub struct ReadExecutionInput {
    pub state: domain::WorthQueryOperationResultState,
    pub warning: Option<domain::WorthQueryOperationExecutionWarning>,
    pub failure: Option<domain::WorthQueryOperationFailureClass>,
}

impl Default for ReadExecutionInput {
    fn default() -> Self {
        Self {
            state: domain::WorthQueryOperationResultState::Ready,
            warning: None,
            failure: None,
        }
    }
}

impl domain::WorthQueryOperationInput for ReadExecutionInput {
    fn parameters(&self) -> Vec<domain::WorthQueryOperationParameter<'_>> {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ReadVertexLookalike;

#[derive(Clone, Copy, Debug)]
pub struct ReadFamily;

#[derive(Clone, Copy, Debug)]
pub struct FederatedRead;

#[derive(Clone, Copy, Debug)]
pub struct WorkflowRead;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily> for ReadVertex {
    type Input = ReadExecutionInput;
    type Output = worth_query::facade::read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily> for FederatedRead {
    type Input = ();
    type Output = worth_query::facade::read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily> for WorkflowRead {
    type Input = ();
    type Output = worth_query::facade::read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryWorkflowOperation;
}
