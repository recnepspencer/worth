pub use std::sync::Arc;

pub use worth_query::facade::{domain, read};
pub use worth_query_host::facade::domain::{
    APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY, APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
    APPLICATION_EXECUTION_PROVIDER_FAMILY, APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
};

pub use worth_query_testkit_host::facade::host::contract::{
    TemporalDomain, TemporalDomainFamily, TemporalDomainOperation,
};

#[path = "domain/query_shapes.rs"]
pub mod query_shapes;
#[path = "domain/unrelated.rs"]
pub mod unrelated;
pub use query_shapes::{
    detail_patch_query, detail_read_declaration, ordered_portfolio_query, ordered_read_declaration,
};
pub use unrelated::{
    unrelated_package, UnrelatedDomain, UnrelatedExecutor, UnrelatedFamily, UnrelatedOperation,
};

#[derive(Clone, Copy)]
pub struct PrimaryGraph;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumerProfile {
    ValuePatch,
    SharedValuePatch,
    OrderedPortfolio,
}

pub fn package(profile: ConsumerProfile) -> domain::WorthQueryDomainPackage<TemporalDomain> {
    domain::WorthQueryDomainPackage::declare(
        TemporalDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.certification").unwrap(),
            domain::WorthQueryDomainIdentityName::new("temporal-consumer").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(consumer_operation_definition(profile))
    .operation_graph_participation::<TemporalDomainOperation, TemporalDomainFamily, PrimaryGraph>(
        "primary",
    )
}

pub fn consumer_operation_definition(
    profile: ConsumerProfile,
) -> domain::WorthQueryDomainOperationDefinition<
    TemporalDomain,
    TemporalDomainOperation,
    TemporalDomainFamily,
> {
    let declared = worth_query_testkit_host::facade::host::contract::operation_definition();
    let mut semantics = declared.semantics().clone();
    let dependency = semantics.conditional_nodes[0].dependencies()[0].clone();
    semantics.native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
        dependency.contract().clone(),
        worth_foundational::facade::AspectMask::whole_aspect(),
    )
    .unwrap();
    if matches!(
        profile,
        ConsumerProfile::ValuePatch | ConsumerProfile::SharedValuePatch
    ) {
        semantics.canonical_query = detail_patch_query();
    }
    if profile == ConsumerProfile::SharedValuePatch {
        let node = &semantics.conditional_nodes[0];
        semantics.conditional_nodes[0] =
            domain::WorthQueryPortableConditionalNodeDeclaration::declare(
                node.identity(),
                node.role(),
            )
            .dependencies(node.dependencies().to_vec())
            .outputs(node.outputs().to_vec())
            .required_context(node.required_context().to_vec())
            .evaluation(node.condition().clone(), node.trigger().clone())
            .comparison(
                node.dependency_comparator().clone(),
                node.output_equivalence().clone(),
            )
            .artifact_policy(
                domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
                node.maintenance(),
                domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
            )
            .output_relationship(node.output_relationship())
            .finish()
            .unwrap();
    }
    if profile == ConsumerProfile::OrderedPortfolio {
        let field =
            domain::WorthQueryOperationCollectionField::from_dotted("IntentFacts.IntentGateField")
                .unwrap();
        semantics.canonical_query = ordered_portfolio_query();
        semantics.collection = domain::WorthQueryOperationCollectionContract::Collection {
            row_identity_field: field.clone(),
            ordering_fields: vec![field.clone()],
            grouping: domain::WorthQueryOperationGroupingContract::Grouped {
                grouping_fields: vec![field],
            },
            window: domain::WorthQueryOperationWindowPolicy::ContinuationBounded,
            continuation: domain::WorthQueryOperationContinuationPosture::SnapshotCursor,
        };
        semantics.support.continuation = domain::WorthQuerySupportRequirement::Required;
    }
    domain::WorthQueryDomainOperationDefinition::new(declared.identity().clone(), semantics)
}

pub fn graph_definition() -> domain::WorthQueryGraphParticipationDefinition<PrimaryGraph> {
    domain::WorthQueryGraphParticipationDefinition::new(
        "primary",
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::InProcess,
            budget: domain::WorthQueryGraphBudgetPosture::ConstantAdmission,
            commit: domain::WorthQueryGraphCommitPosture::ReadOnly,
            failure: domain::WorthQueryGraphFailureTopology::Local,
        },
    )
}

pub struct PrimaryGraphProvider;

impl domain::WorthQueryGraphParticipationProvider<PrimaryGraph> for PrimaryGraphProvider {
    type Execution = OneStepGraphRead;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        resource_support()
    }

    fn begin(
        &self,
        _call: &domain::WorthQueryGraphProviderCall,
        start: &mut domain::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        domain::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        domain::WorthQueryGraphProviderFailure,
    > {
        start
            .admit_cooperative_execution(|| OneStepGraphRead(false))
            .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}

pub struct OneStepGraphRead(pub bool);

impl domain::WorthQueryGraphProviderExecution for OneStepGraphRead {
    fn advance(
        &mut self,
        step: &mut domain::WorthQueryGraphProviderStep,
    ) -> Result<
        domain::WorthQueryGraphProviderStepDisposition,
        domain::WorthQueryGraphProviderFailure,
    > {
        if self.0 {
            return Err(domain::WorthQueryGraphProviderFailure::new(
                "primary graph read advanced after completion",
            ));
        }
        self.0 = true;
        step.perform_work_unit(|| Ok(()))?;
        domain::WorthQueryGraphProviderStepDisposition::complete(Arc::from("primary-graph-read"))
            .map_err(domain::WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), domain::WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct OperationExecutor(pub ConsumerProfile);

impl
    domain::WorthQueryDomainOperationExecutor<
        TemporalDomain,
        TemporalDomainOperation,
        TemporalDomainFamily,
    > for OperationExecutor
{
    const LOWERING_FAMILY: &'static str = "temporal-host-courtroom-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(match self.0 {
            ConsumerProfile::ValuePatch | ConsumerProfile::SharedValuePatch => {
                detail_read_declaration()
            }
            ConsumerProfile::OrderedPortfolio => ordered_read_declaration(),
        })
    }

    fn execute(
        &self,
        _: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            context.execute_installed_read(workspace)?,
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

pub fn resource_support() -> domain::WorthQueryExecutionResourceSupport {
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new(APPLICATION_EXECUTION_PROVIDER_FAMILY)
            .unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new(
            APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY,
        )
        .unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new(APPLICATION_EXECUTION_ALLOCATOR_FAMILY)
            .unwrap(),
        domain::WorthQueryExecutionResourceEnvelope::bounded(
            1_000,
            1_000,
            domain::WorthQueryExecutionMode::Synchronous,
            domain::WorthQueryCancellationSafePointFamily::new(
                APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
            )
            .unwrap(),
        ),
        Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint(
                APPLICATION_EXECUTION_PROVIDER_FAMILY,
                1_000,
            )
            .unwrap(),
        ),
    )
}
