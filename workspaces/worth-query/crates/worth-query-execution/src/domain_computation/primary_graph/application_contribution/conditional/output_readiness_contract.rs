use worth_query_declaration::facade::{
    canonicalization::CanonicalQueryBundle,
    domain_computation::{WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode},
};
use worth_query_installation::facade as domain;

/// Domain facts needed to declare Query's standard derived-output readiness operation.
pub struct WorthQueryOutputReadinessContractBuilder {
    identity: domain::WorthQueryDomainOperationIdentity,
    node_identity: String,
    native_projection: domain::WorthQueryOperationNativeProjectionContract,
    canonical_query: CanonicalQueryBundle,
    semantic_reads: Vec<domain::WorthQueryOperationNativeProjectionContract>,
    dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
    readiness_dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
    projection_role: domain::WorthQueryOperationProjectionRole,
    strategy: domain::WorthQueryExecutionStrategyName,
    work_limit: u64,
    retained_byte_limit: u64,
    lowering_family: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputReadinessContractDenial {
    detail: String,
}

impl WorthQueryOutputReadinessContractDenial {
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for WorthQueryOutputReadinessContractDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for WorthQueryOutputReadinessContractDenial {}

impl WorthQueryOutputReadinessContractBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: domain::WorthQueryDomainOperationIdentity,
        node_identity: impl Into<String>,
        native_projection: domain::WorthQueryOperationNativeProjectionContract,
        canonical_query: CanonicalQueryBundle,
        projection_role: domain::WorthQueryOperationProjectionRole,
        strategy: domain::WorthQueryExecutionStrategyName,
        work_limit: u64,
        retained_byte_limit: u64,
        lowering_family: impl Into<String>,
    ) -> Self {
        Self {
            identity,
            node_identity: node_identity.into(),
            native_projection,
            canonical_query,
            semantic_reads: Vec::new(),
            dependencies: Vec::new(),
            readiness_dependencies: Vec::new(),
            projection_role,
            strategy,
            work_limit,
            retained_byte_limit,
            lowering_family: lowering_family.into(),
        }
    }

    pub fn semantic_reads(
        mut self,
        reads: impl IntoIterator<Item = domain::WorthQueryOperationNativeProjectionContract>,
    ) -> Self {
        self.semantic_reads.extend(reads);
        self
    }

    pub fn dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = domain::WorthQuerySemanticTruthDependency>,
    ) -> Self {
        self.dependencies.extend(dependencies);
        self
    }

    pub fn readiness_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = domain::WorthQuerySemanticTruthDependency>,
    ) -> Self {
        self.readiness_dependencies.extend(dependencies);
        self
    }

    pub fn build<D, O, F>(
        self,
    ) -> Result<
        domain::WorthQueryDomainOperationDefinition<D, O, F>,
        WorthQueryOutputReadinessContractDenial,
    > {
        if self.dependencies.is_empty() || self.readiness_dependencies.is_empty() {
            return Err(denial(
                "output readiness requires delivery and readiness dependencies",
            ));
        }
        let node = readiness_node(
            &self.node_identity,
            self.dependencies,
            self.readiness_dependencies,
            self.projection_role.clone(),
        )?;
        let resources =
            resource_contract(self.strategy, self.work_limit, self.retained_byte_limit)?;
        Ok(domain::WorthQueryDomainOperationDefinition::new(
            self.identity,
            domain::WorthQueryDomainOperationSemanticClosure {
                parameters: domain::WorthQueryOperationParameterContract::NotRequired,
                native_projection: self.native_projection,
                canonical_query: self.canonical_query,
                collection: domain::WorthQueryOperationCollectionContract::NotCollection,
                required_capabilities: Vec::new(),
                required_domains: Vec::new(),
                workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
                evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
                conditional_nodes: vec![node],
                graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                    roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                        role: "primary".into(),
                        participation:
                            domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                        access: domain::WorthQueryOperationGraphAccess::Project,
                        semantic_reads: self.semantic_reads,
                    }],
                },
                decision_facts: domain::WorthQueryOperationDecisionFactContract::NotRequired,
                touches: domain::WorthQueryOperationTouchContract::NotRequired,
                effects: domain::WorthQueryOperationEffectContract::NotRequired,
                invariants: domain::WorthQueryOperationInvariantContract::NotRequired,
                invariant_execution: domain::WorthQueryInvariantExecutionContract::NotRequired,
                replay: domain::WorthQueryOperationReplayContract::ReExecutable,
                aftermath: None,
                lineage: domain::WorthQueryOperationLineageContract::NotRequired,
                promotion: domain::WorthQueryOperationPromotionContract::NotRequired,
                publication: domain::WorthQueryOperationPublicationContract::DerivedProjection {
                    projection_role: self.projection_role,
                },
                projection_consumption:
                    domain::WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority,
                terminal: domain::WorthQueryOperationTerminalContract {
                    result_states: vec![domain::WorthQueryOperationResultState::Ready],
                    failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
                },
                cost: domain::WorthQueryOperationCostContract {
                    lookup: domain::WorthQueryOperationCostClass::Constant,
                    execution: domain::WorthQueryOperationCostClass::Constant,
                    result_width: domain::WorthQueryOperationCostClass::Constant,
                },
                resources,
                support: support_contract(),
                lowering: domain::WorthQueryOperationLoweringContract {
                    family: self.lowering_family,
                    deterministic: true,
                },
            },
        ))
    }
}

fn readiness_node(
    node_identity: &str,
    dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
    readiness_dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
    projection_role: domain::WorthQueryOperationProjectionRole,
) -> Result<
    domain::WorthQueryPortableConditionalNodeDeclaration,
    WorthQueryOutputReadinessContractDenial,
> {
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        node_identity,
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies(dependencies)
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput { projection_role }])
    .required_context([domain::WorthQueryConditionalNodeContext::Snapshot])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered(readiness_dependencies)
            .map_err(|error| denial(error.to_string()))?,
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::EagerOnEligibleInvalidation,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsOperationOutput)
    .finish()
    .map_err(|error| denial(error.to_string()))
}

fn resource_contract(
    strategy: domain::WorthQueryExecutionStrategyName,
    work_limit: u64,
    retained_byte_limit: u64,
) -> Result<domain::WorthQueryExecutionResourceContract, WorthQueryOutputReadinessContractDenial> {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        work_limit,
        retained_byte_limit,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new(domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .map_err(|error| denial(error.to_string()))?,
    );
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            strategy,
            envelope,
            domain::WorthQueryExecutionProviderRequirements::new(
                domain::WorthQueryExecutionProviderFamily::new(
                    domain::APPLICATION_EXECUTION_PROVIDER_FAMILY,
                )
                .map_err(|error| denial(error.to_string()))?,
                domain::WorthQueryExecutionAccessProductFamily::new(
                    domain::APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY,
                )
                .map_err(|error| denial(error.to_string()))?,
                domain::WorthQueryExecutionAllocatorFamily::new(
                    domain::APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
                )
                .map_err(|error| denial(error.to_string()))?,
            ),
        ),
    ])
    .map_err(|error| denial(error.to_string()))
}

fn support_contract() -> domain::WorthQueryOperationSupportRequirements {
    let no = domain::WorthQuerySupportRequirement::NotRequired;
    let required = domain::WorthQuerySupportRequirement::Required;
    domain::WorthQueryOperationSupportRequirements {
        live: required,
        continuation: no,
        async_result_state: no,
        recovery: required,
        inspection: required,
        projection_consumption: required,
        dependency_impact: required,
        sharing: required,
        invalidation: required,
        collection_delivery: no,
        conditional_evaluation: required,
        conditional_comparator: required,
        conditional_trigger: required,
        conditional_temporal_or_on_demand: no,
    }
}

fn denial(detail: impl Into<String>) -> WorthQueryOutputReadinessContractDenial {
    WorthQueryOutputReadinessContractDenial {
        detail: detail.into(),
    }
}
