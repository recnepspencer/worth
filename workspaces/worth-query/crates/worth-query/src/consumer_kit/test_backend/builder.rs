use crate::domain_capabilities::WorthQueryInvariantCatalogRegistrationArtifact;
use worth_relational::facade::runtime::{
    CustomInvariantRegistration, CustomInvariantRule, InvariantCatalog,
};

use super::domain_package_installation::{domain_package_installer, TestDomainInstaller};
use super::error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind};
use super::schema::WorthQueryTestBackendSchema;

use super::runtime_installation::TestRuntimeInstaller;

#[derive(Default)]
pub struct WorthQueryInMemoryTestRuntimeBuilder {
    pub(super) schema: Option<WorthQueryTestBackendSchema>,
    pub(super) invariant_catalog: InvariantCatalog,
    pub(super) custom_invariants: Vec<CustomInvariantRegistration>,
    pub(super) domain_installers: Vec<TestDomainInstaller>,
    pub(super) runtime_installers: Vec<TestRuntimeInstaller>,
    pub(super) support_profile: Option<crate::runtime::WorthQueryRuntimeSupportProfile>,
    pub(super) remask_projection: Option<crate::runtime::WorthQueryRuntimeRemaskProjection>,
    pub(super) live_close_failures: usize,
    pub(super) collection_entity_lookup_disabled: bool,
    pub(super) initial_seed: Option<super::seed::WorthQueryTestSeedSpecification>,
}

pub fn in_memory_test_runtime() -> WorthQueryInMemoryTestRuntimeBuilder {
    WorthQueryInMemoryTestRuntimeBuilder::default()
}

impl WorthQueryInMemoryTestRuntimeBuilder {
    /// Removes exact collection entity lookup to exercise honest reset paths.
    pub fn without_collection_entity_lookup(mut self) -> Self {
        self.collection_entity_lookup_disabled = true;
        self
    }

    /// Injects exact backend close failures for lifecycle ownership tests.
    pub fn fail_next_live_closes(mut self, count: usize) -> Self {
        self.live_close_failures = count;
        self
    }

    pub fn remask_projection(
        mut self,
        projection: crate::runtime::WorthQueryRuntimeRemaskProjection,
    ) -> Self {
        self.remask_projection = Some(projection);
        self
    }

    pub fn controlled_workspace(
        self,
        name: impl Into<String>,
    ) -> Result<super::WorthQueryControlledTestWorkspace, WorthQueryTestBackendError> {
        self.workspace(name)
            .map(super::WorthQueryControlledTestWorkspace::new)
    }

    pub fn with_schema(mut self, schema: WorthQueryTestBackendSchema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn invariant_catalog(mut self, invariant_catalog: InvariantCatalog) -> Self {
        self.merge_invariant_catalog(invariant_catalog);
        self
    }

    pub fn invariant_registration_artifact(
        mut self,
        artifact: WorthQueryInvariantCatalogRegistrationArtifact,
    ) -> Self {
        self.merge_invariant_catalog(artifact.invariant_catalog().clone());
        self
    }

    pub fn custom_invariant(mut self, custom_invariant: CustomInvariantRegistration) -> Self {
        self.custom_invariants.push(custom_invariant);
        self
    }

    pub fn support_profile(
        mut self,
        profile: crate::runtime::WorthQueryRuntimeSupportProfile,
    ) -> Self {
        self.support_profile = Some(profile);
        self
    }

    pub fn consumer_support_posture(
        mut self,
        dimension: crate::domain_installation::WorthQueryConsumerSupportDimension,
        posture: crate::domain_installation::WorthQueryConsumerSupportPosture,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.consumer_support_posture(dimension, posture)
            })));
        self
    }

    pub fn domain_package<D: crate::application::WorthQueryDomainEntryMarker + 'static>(
        self,
        package: crate::domain_installation::WorthQueryDomainPackage<D>,
    ) -> Self {
        self.domain_package_with_artifact_support(
            package,
            crate::domain_installation::WorthQueryArtifactInstallationSupport::default(),
        )
    }

    pub fn domain_package_with_artifact_support<
        D: crate::application::WorthQueryDomainEntryMarker + 'static,
    >(
        mut self,
        package: crate::domain_installation::WorthQueryDomainPackage<D>,
        artifact_support: crate::domain_installation::WorthQueryArtifactInstallationSupport,
    ) -> Self {
        self.domain_installers
            .push(domain_package_installer(package, artifact_support));
        self
    }

    pub fn graph_participation<G: 'static>(
        mut self,
        definition: crate::domain_installation::WorthQueryGraphParticipationDefinition<G>,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.graph_participation(definition)
            })));
        self
    }

    pub fn domain_operation_executor<D: 'static, O, F: 'static, E>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        executor: E,
    ) -> Self
    where
        O: crate::domain_installation::WorthQueryExecutableDomainOperation<
            D,
            F,
            Execution = crate::domain_installation::WorthQueryDirectOperation,
        >,
        E: crate::domain_installation::WorthQueryDomainOperationExecutor<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.domain_operation_executor(domain, operation, family, executor)
            })));
        self
    }

    pub fn workflow_stage_executor<D: 'static, O, F: 'static, E>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        executor: E,
    ) -> Self
    where
        O: 'static
            + crate::domain_installation::WorthQueryExecutableDomainOperation<
                D,
                F,
                Execution = crate::domain_installation::WorthQueryWorkflowOperation,
            >,
        E: crate::domain_installation::WorthQueryDomainWorkflowStageExecutor<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.workflow_stage_executor(domain, operation, family, executor)
            })));
        self
    }

    pub fn replayable_workflow_stage_executor<D: 'static, O, F: 'static, E>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        executor: E,
    ) -> Self
    where
        O: 'static
            + crate::domain_installation::WorthQueryExecutableDomainOperation<
                D,
                F,
                Execution = crate::domain_installation::WorthQueryWorkflowOperation,
            >,
        E: crate::domain_installation::WorthQueryDomainWorkflowStageExecutor<D, O, F>
            + crate::domain_installation::WorthQueryDomainReplaySemanticComparator<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.replayable_workflow_stage_executor(domain, operation, family, executor)
            })));
        self
    }

    pub fn workflow_parallel_admission_provider<D: 'static, O: 'static, F: 'static, P>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        provider: P,
    ) -> Self
    where
        P: crate::domain_installation::WorthQueryWorkflowParallelAdmissionProvider<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.workflow_parallel_admission_provider(domain, operation, family, provider)
            })));
        self
    }

    pub fn graph_participation_provider<
        G: 'static,
        P: crate::domain_installation::WorthQueryGraphParticipationProvider<G>,
    >(
        mut self,
        marker: G,
        provider: P,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.graph_participation_provider(marker, provider)
            })));
        self
    }

    pub fn atomic_graph_participation_provider<G: 'static, C: 'static, P>(
        mut self,
        marker: G,
        provider: P,
        commit: C,
    ) -> Self
    where
        P: crate::domain_installation::WorthQueryGraphParticipationProvider<G>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.atomic_graph_participation_provider(marker, provider, commit)
            })));
        self
    }

    pub fn graph_commit_provider<C: 'static, P>(mut self, commit: C, provider: P) -> Self
    where
        P: crate::domain_installation::WorthQueryGraphCommitProvider<C>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.graph_commit_provider(commit, provider)
            })));
        self
    }

    pub fn register_invariant<R>(mut self, rule: R) -> Result<Self, WorthQueryTestBackendError>
    where
        R: CustomInvariantRule + std::panic::UnwindSafe + 'static,
    {
        let registration = CustomInvariantRegistration::new(rule).map_err(|error| {
            WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::InvariantRegistrationFailed,
                format!("failed to register in-memory test backend invariant: {error:?}"),
            )
        })?;
        self.custom_invariants.push(registration);
        Ok(self)
    }

    fn merge_invariant_catalog(&mut self, invariant_catalog: InvariantCatalog) {
        self.invariant_catalog
            .registrations
            .extend(invariant_catalog.registrations);
        self.invariant_catalog = self.invariant_catalog.clone().canonicalized();
    }
}
