//! Installed empty Branch roots for existing cross-crate courts.
use super::basis_observation_inputs::BranchObservationInputs;
use super::reference::RelationalBranchReferenceCell;
use super::root::RelationalBranchRoot;
use crate::history::data::coordinates::BranchId;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{runtime_descriptor_semantics_policy, RelationalSchemaRegistry};

#[doc(hidden)]
pub fn installed_history_for_test(runtime_id: u64) -> HistorySubsystem {
    let branch = BranchId("main".into());
    let mut history = HistorySubsystem::from_main_cell(
        RelationalBranchReferenceCell::empty(0, branch.clone()).unwrap(),
    );
    history.set_runtime_instance_id(runtime_id).unwrap();
    history
        .install_branch_root(
            &branch,
            RelationalBranchRoot::empty_with_schema(
                &RelationalSchemaRegistry::new(),
                runtime_descriptor_semantics_policy().current_write_version(),
            ),
        )
        .unwrap();
    history
}

pub(super) fn observation_inputs<'a>(
    history: &'a HistorySubsystem,
    schema: &'a RelationalSchemaRegistry,
    instrumentation: &'a RuntimeInstrumentation,
) -> BranchObservationInputs<'a> {
    BranchObservationInputs {
        history,
        runtime_instance_id: history.runtime_instance_id,
        empty_schema: schema,
        instrumentation,
    }
}

#[doc(hidden)]
pub fn observed_empty_branch_for_test(
    runtime_id: u64,
) -> (
    HistorySubsystem,
    super::basis::AdmittedRelationalBranchBasis,
) {
    let history = installed_history_for_test(runtime_id);
    let schema = RelationalSchemaRegistry::new();
    let instrumentation = RuntimeInstrumentation::new();
    let inputs = observation_inputs(&history, &schema, &instrumentation);
    let identity = inputs.branch_identity(&history.main_branch).unwrap();
    let (_, basis) = inputs.observe(&identity, || None).unwrap();
    (history, basis)
}
