use crate::facade::{
    admit_home, readmit_relational, recover_home, ApplicationHomeCloseOutcome,
    ApplicationHomeDescriptorDenial, ApplicationHomeOpenRefusal,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::ApplicationBuildIdentity;

#[test]
fn changed_implementation_build_preserves_authored_binding_and_exact_durable_history() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let admitted = admit_home(
        recover_home(super::program::request(directory.path()), &execution).unwrap(),
        &execution,
    )
    .unwrap();
    let home = readmit_relational(admitted, &execution)
        .unwrap()
        .consume(|home| home);
    let binding = home.binding().clone();
    let identity = home.home_identity();
    let build = ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    assert_ne!(home.build_identity(), build);
    super::native_schema::commit(home.relational().runtime(), &execution, "original-build");
    let before = super::native_schema::artifacts(home.relational().runtime());
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    let next = super::program::request_with_build(directory.path(), build);
    let admitted = admit_home(recover_home(next, &execution).unwrap(), &execution).unwrap();
    let home = readmit_relational(admitted, &execution)
        .unwrap()
        .consume(|home| home);
    assert_eq!(home.home_identity(), identity);
    assert_eq!(home.binding(), &binding);
    assert_eq!(home.build_identity(), build);
    assert_eq!(
        super::native_schema::artifacts(home.relational().runtime()),
        before
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn legacy_native_descriptor_refuses_application_adoption_without_advancing_its_incarnation() {
    use crate::open::physical_admission::{self, PhysicalOpening};
    use worth_store::physical_runtime::{
        AnchoredRecordName, AnchoredRecordPublication, PhysicalMutationDeadline,
    };
    let directory = tempfile::tempdir().unwrap();
    let serving =
        physical_admission::initialize(directory.path(), PhysicalOpening::production()).unwrap();
    let definition = super::native_schema::definition();
    let config = definition.configuration();
    let mut legacy = Vec::new();
    legacy.extend_from_slice(b"WSRHOME1");
    legacy.extend_from_slice(&2_u16.to_le_bytes());
    legacy.extend_from_slice(&serving.store_identity().bytes());
    legacy.extend_from_slice(&1_u64.to_le_bytes());
    for text in [
        &config.execution.runtime_name,
        &format!("{:?}", config.profile),
    ] {
        legacy.extend_from_slice(&(text.len() as u32).to_le_bytes());
        legacy.extend_from_slice(text.as_bytes());
    }
    legacy.extend_from_slice(&0_u16.to_le_bytes());
    let anchor = AnchoredRecordName::new("home").unwrap();
    serving
        .anchored_records()
        .unwrap()
        .publish(AnchoredRecordPublication::new(
            anchor,
            0,
            legacy.clone(),
            physical_admission::placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    let _closed = serving.close();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let recovered = recover_home(super::program::request(directory.path()), &execution).unwrap();
    assert!(matches!(
        admit_home(recovered, &execution),
        Err(ApplicationHomeOpenRefusal::Descriptor(
            ApplicationHomeDescriptorDenial::ApplicationBindingMissing
        ))
    ));
    let opened = physical_admission::open(directory.path(), PhysicalOpening::production()).unwrap();
    let records = opened.serving.anchored_records().unwrap();
    let record = records.read(anchor, 64 << 10).unwrap().unwrap();
    assert_eq!(record.version().generation(), 1);
    assert_eq!(record.bytes(), legacy);
    let _closed = opened.serving.close();
}
