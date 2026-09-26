//! The bounded-dimension host's installed workflow and authentication owner.

use std::time::Duration;

use worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime;

use super::super::{
    programs::DimensionProgramP0,
    schema::BoundedDimensionSchema,
    workflow::{
        CertificationAuthenticationClockSource, CertificationAuthenticationOwner,
        ReviewedGeometryWorkflow,
    },
};

pub struct BoundedDimensionWorkflowRuntime {
    pub(in super::super) workflow: WorthQueryWorkflowApplicationRuntime<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >,
    pub(in super::super) authentication: CertificationAuthenticationOwner,
    pub(in super::super) authentication_clock: CertificationAuthenticationClockSource,
}

impl BoundedDimensionWorkflowRuntime {
    pub fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }

    pub fn advance_authentication_clock_for_test(&self, duration: Duration) {
        self.authentication_clock.advance_for_test(duration);
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
