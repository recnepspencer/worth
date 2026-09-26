//! The bounded-dimension host's installed workflow and authentication owner.

use std::time::Duration;

use worth_query_host::facade::application_installation::{
    WorthQueryWorkflowApplicationRuntime, WorthQueryWorkflowVocabulary,
};

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

    /// Lets a certification add vocabularies for other rostered programs.
    pub fn workflow_mut(
        &mut self,
    ) -> &mut WorthQueryWorkflowApplicationRuntime<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    > {
        &mut self.workflow
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

/// Every workflow entry names its vocabulary; this host's initial one is the
/// P0 vocabulary its workflow runtime retained at installation.
impl<'application> From<&'application BoundedDimensionWorkflowRuntime>
    for WorthQueryWorkflowVocabulary<
        'application,
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >
{
    fn from(application: &'application BoundedDimensionWorkflowRuntime) -> Self {
        application.workflow.vocabulary()
    }
}
