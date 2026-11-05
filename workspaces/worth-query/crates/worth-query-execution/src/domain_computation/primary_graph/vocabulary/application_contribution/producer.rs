//! Moved whole from `domain_computation::primary_graph::application_contribution::producer`: the items below and every impl they own.
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;

#[derive(Clone, Eq, PartialEq)]
pub enum WorthQueryProducerCommitAuthority {
    Ordinary,
    ProgramOutput,
    SelectedProgram {
        identity: worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
        revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryProducerLifecyclePosture {
    Initial,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerApplicability {
    profile_kind: &'static str,
    lifecycle: WorthQueryProducerLifecyclePosture,
}

impl WorthQueryProducerApplicability {
    pub const fn new(
        profile_kind: &'static str,
        lifecycle: WorthQueryProducerLifecyclePosture,
    ) -> Self {
        Self {
            profile_kind,
            lifecycle,
        }
    }

    pub const fn profile_kind(self) -> &'static str {
        self.profile_kind
    }

    pub const fn lifecycle(self) -> WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAssessmentPosture {
    Passing,
    Failing,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerInvariantRequirement {
    identifier: &'static str,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
}

impl WorthQueryProducerInvariantRequirement {
    pub const fn new(
        identifier: &'static str,
        major: u16,
        minor: u16,
        execution_point: ApplicationInvariantExecutionPoint,
    ) -> Self {
        Self {
            identifier,
            major,
            minor,
            execution_point,
        }
    }

    pub const fn identifier(self) -> &'static str {
        self.identifier
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub const fn minor(self) -> u16 {
        self.minor
    }

    pub const fn execution_point(self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
}
