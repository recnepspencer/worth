#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryOperationBindingCounters {
    pub authority_checks: usize,
    pub operation_lookups: usize,
    pub required_domain_lookups: usize,
    pub graph_binding_lookups: usize,
    pub graph_participation_lookups: usize,
    pub graph_provider_contacts: usize,
    pub conditional_lowering_lookups: usize,
    pub conditional_lowerings_retained: usize,
    pub conditional_declarations_inspected: usize,
    pub conditional_workflow_stages_inspected: usize,
    pub conditional_lowering_checks: usize,
    pub graph_contract_checks: usize,
    pub graph_read_role_checks: usize,
    pub touched_graph_role_checks: usize,
    pub commit_graph_checks: usize,
    pub commit_authority_checks: usize,
    pub planning_steps: usize,
    pub authority_shape_admissions: usize,
    pub commit_posture_classifications: usize,
    pub executor_route_lookups: usize,
    pub workflow_executor_route_lookups: usize,
    pub parallel_admission_route_lookups: usize,
    pub execution_authority_admissions: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationBindingDenialKind {
    DomainAuthority,
    OperationNotInstalled,
    RequiredDomainNotInstalled,
    GraphParticipationNotInstalled,
    GraphRoleMismatch,
    GraphAuthorityInsufficient,
    BasisLaneInsufficient,
    BasisExecutionUnsupported,
    ConditionalLoweringNotInstalled,
    ConditionalLoweringDrift,
    IncoherentAuthoritySet,
    CommitAuthorityMismatch,
    CompensationUndeclared,
    ExecutionAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationBindingDenial {
    kind: WorthQueryOperationBindingDenialKind,
    detail: String,
    counters: WorthQueryOperationBindingCounters,
}

impl WorthQueryOperationBindingDenial {
    pub(super) fn new(
        kind: WorthQueryOperationBindingDenialKind,
        detail: impl Into<String>,
        counters: WorthQueryOperationBindingCounters,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            counters,
        }
    }

    pub fn kind(&self) -> WorthQueryOperationBindingDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn counters(&self) -> WorthQueryOperationBindingCounters {
        self.counters
    }
}
