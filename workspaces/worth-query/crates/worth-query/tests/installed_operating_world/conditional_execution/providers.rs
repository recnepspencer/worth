use super::*;

pub(super) type CapturedContext = (String, String, String, Option<String>, String, u64);

pub(super) struct CapturingCompute(pub(super) Arc<Mutex<Option<CapturedContext>>>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for CapturingCompute
{
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        *self.0.lock().unwrap() = Some((
            context.operation_identity().to_string(),
            context.binding_identity().to_string(),
            context.basis_identity().to_string(),
            context.workflow_run_identity().map(str::to_string),
            context.snapshot_identity().to_string(),
            context.attempt(),
        ));
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}

pub(super) struct StaticCondition(
    pub(super) worth_signal::facade::InstalledSignalConditionDecision,
);

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for StaticCondition {
    type SemanticContract = worth_signal::facade::InstalledSignalConditionDecision;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalConditionProvider for StaticCondition {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(self.0)
    }
}

pub(super) struct CountedCompute {
    contacts: Arc<AtomicUsize>,
    version: u64,
}

impl CountedCompute {
    pub(super) fn new(contacts: Arc<AtomicUsize>, version: u64) -> Self {
        Self { contacts, version }
    }
}

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for CountedCompute
{
    type SemanticContract = u64;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.version
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.version,
            )]),
        ))
    }
}

pub(super) fn domain_condition_node(
    identity: &str,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node_result(
        identity,
        dependency(domain::WorthQuerySemanticLocality::SourceRecord),
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap()
}
