use worth_store::physical_runtime::{
    AdmittedPhysicalRuntime, InstalledCapabilityStatus, PhysicalRuntimeAdmission,
};

fn construct_admission() {
    let _admission = PhysicalRuntimeAdmission {
        declared_root: unavailable(),
    };
}

fn construct_runtime() {
    let _runtime = AdmittedPhysicalRuntime {
        core: unavailable(),
    };
}

fn construct_capability_status() {
    let _status = InstalledCapabilityStatus { serving: false };
}

fn unavailable<T>() -> T {
    panic!("compile-fail specimen")
}

fn main() {}
