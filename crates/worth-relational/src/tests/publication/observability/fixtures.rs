pub(super) use crate::diagnostics::data::RelationalDiagnosticValue;
pub(super) use crate::facade::diagnostics::{
    DiagnosticsDeliveryClass, RelationalDiagnosticsProfile,
};
pub(super) use crate::facade::runtime::HarnessAuditMode;
pub(super) use crate::schema::data::{
    ContractId, EndpointKindContractDeclaration, RelationIntegrityDeclarations,
    SymmetryContractDeclaration, SymmetryMode,
};
pub(super) use crate::tests::support::*;

pub(super) use crate::tests::publication::harness_summary_projection::{
    harness_diagnostic_entries, harness_diagnostic_field_matches, harness_summary_counter,
    harness_summary_field,
};

#[derive(Clone)]
pub(super) struct InvariantHarnessAdapter {
    invariant_catalog: InvariantCatalog,
}

impl InvariantHarnessAdapter {
    pub(super) fn new(invariant_catalog: InvariantCatalog) -> Self {
        Self { invariant_catalog }
    }
}

impl worth_harness::facade::HarnessAdapter for InvariantHarnessAdapter {
    type Runtime = crate::facade::runtime::RelationalRuntime;
    type Fixture = crate::presentation::harness::RelationalFixture;
    type Mutation = crate::facade::transactions::WorkerIntentBatch;
    type TargetId = String;
    type Error = crate::facade::harness::RelationalHarnessError;

    fn adapter_name(&self) -> &'static str {
        RelationalHarnessAdapter.adapter_name()
    }

    fn capabilities(&self) -> worth_harness::facade::HarnessCapabilities {
        RelationalHarnessAdapter.capabilities()
    }

    fn create_runtime(&self) -> Result<Self::Runtime, Self::Error> {
        let builder = RelationalRuntimeApi::builder()
            .schema_registry(test_schema_registry())
            .invariant_catalog(self.invariant_catalog.clone());
        Ok(builder
            .build()
            .expect("valid invariant harness configuration"))
    }

    fn prepare_runtime(
        &self,
        runtime: &mut Self::Runtime,
        profile: &worth_harness::facade::ExecutionProfile,
    ) -> Result<(), Self::Error> {
        RelationalHarnessAdapter.prepare_runtime(runtime, profile)
    }

    fn load_fixture(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
    ) -> Result<(), Self::Error> {
        RelationalHarnessAdapter.load_fixture(runtime, fixture)
    }

    fn apply_mutation_batch(
        &self,
        runtime: &mut Self::Runtime,
        batch: &worth_harness::facade::MutationBatch<Self::Mutation>,
    ) -> Result<(), Self::Error> {
        RelationalHarnessAdapter.apply_mutation_batch(runtime, batch)
    }

    fn execute(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        request: &worth_harness::facade::ExecutionRequest<Self::TargetId>,
        profile: &worth_harness::facade::ExecutionProfile,
    ) -> Result<worth_harness::facade::RunRecord<Self::TargetId>, Self::Error> {
        RelationalHarnessAdapter.execute(runtime, fixture, request, profile)
    }

    fn capture_snapshot(
        &self,
        runtime: &Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        request: &worth_harness::facade::ExecutionRequest<Self::TargetId>,
        profile: &worth_harness::facade::ExecutionProfile,
    ) -> Result<worth_harness::facade::SnapshotRecord<Self::TargetId>, Self::Error> {
        RelationalHarnessAdapter.capture_snapshot(runtime, fixture, request, profile)
    }
}

impl worth_harness::facade::DiagnosticsHarnessAdapter for InvariantHarnessAdapter {
    fn capture_diagnostics(
        &self,
        runtime: &Self::Runtime,
        fixture: &worth_harness::facade::ScenarioFixture<Self::Fixture>,
        profile: &worth_harness::facade::ExecutionProfile,
    ) -> Result<worth_harness::facade::DiagnosticsRecord, Self::Error> {
        RelationalHarnessAdapter.capture_diagnostics(runtime, fixture, profile)
    }
}

pub(super) fn diagnostic_object_field<'a>(
    value: &'a RelationalDiagnosticValue,
    field: &str,
) -> &'a RelationalDiagnosticValue {
    let RelationalDiagnosticValue::Object(fields) = value else {
        panic!("diagnostic value is not an object: {value:?}");
    };
    fields
        .get(field)
        .unwrap_or_else(|| panic!("diagnostic object field '{field}' missing from {value:?}"))
}

pub(super) fn existing_entity_reference_diagnostic_value(
    entity_id: crate::identity::data::EntityId,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "reference_kind",
            RelationalDiagnosticValue::string("existing"),
        ),
        ("entity_id", RelationalDiagnosticValue::EntityId(entity_id)),
    ])
}
pub(super) use worth_harness::facade::{
    ExecutionProfile, ExecutionRequest, MutationBatch, ReplayRequest, ScenarioPlan,
};

pub(super) fn harness_phase8_fixture_batch_request() -> (
    worth_harness::facade::ScenarioFixture<crate::presentation::harness::RelationalFixture>,
    MutationBatch<crate::facade::transactions::WorkerIntentBatch>,
    worth_harness::facade::ExecutionRequest<String>,
) {
    let fixture = ScenarioPlan::new(
        "fixture",
        crate::presentation::harness::RelationalFixture {
            entities: Vec::new(),
            relations: Vec::new(),
        },
    )
    .compile();
    let batch = MutationBatch::new("mutate")
        .push(batch_create("alpha"))
        .push(batch_create("beta"));
    let request = ExecutionRequest::target("inspect", "entity:0:1".to_string());
    (fixture, batch, request)
}

pub(super) fn certification_case<'a>(
    report: &'a worth_harness::facade::CertificationMatrixReport,
    candidate_profile: &str,
) -> &'a worth_harness::facade::CertificationMatrixCase {
    report
        .cases
        .iter()
        .find(|case| case.candidate_profile == candidate_profile)
        .unwrap_or_else(|| panic!("missing certification case for profile {candidate_profile}"))
}
