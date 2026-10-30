use worth_store::physical_runtime::{
    AdmissionError, ObservationError, PhysicalRuntimeAdmission, PhysicalStore,
};

fn main() {
    assert_standard_error::<AdmissionError>();

    let admission = PhysicalRuntimeAdmission::new(
        std::env::temp_dir().join("worth-store-ui-supported-admission"),
    )
    .unwrap();
    let runtime = PhysicalStore::admit(admission).unwrap();

    assert_standard_error::<ObservationError>();
    let observer = runtime.observe();
    assert_eq!(observer.runtime_identity(), runtime.runtime_identity());
    let cloned_observer = observer.clone();
    assert_eq!(
        cloned_observer.runtime_identity(),
        runtime.runtime_identity()
    );
    let _snapshot = observer.snapshot().unwrap();
    let _closed = runtime.close();
}

fn assert_standard_error<ErrorType: std::error::Error>() {}
