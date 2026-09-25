//! The bounded-dimension host's installed workflow and authentication owner.

use worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime;

use super::super::{
    programs::DimensionProgramP0,
    schema::BoundedDimensionSchema,
    workflow::{CertificationAuthenticationOwner, ReviewedGeometryWorkflow},
};

pub struct BoundedDimensionWorkflowRuntime {
    pub(in super::super) workflow: WorthQueryWorkflowApplicationRuntime<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >,
    pub(in super::super) authentication: CertificationAuthenticationOwner,
}

impl BoundedDimensionWorkflowRuntime {
    pub fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }
}

impl std::ops::Deref for BoundedDimensionWorkflowRuntime {
    type Target = WorthQueryWorkflowApplicationRuntime<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >;

    fn deref(&self) -> &Self::Target {
        &self.workflow
    }
}
