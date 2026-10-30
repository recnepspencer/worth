use worth_store::physical_runtime::{
    lifecycle::LifecycleCoordinator, root_admission::RootAdmission,
};

fn name_internal_owners(root: RootAdmission, lifecycle: LifecycleCoordinator) {
    drop((root, lifecycle));
}

fn main() {}
