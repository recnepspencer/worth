//! Consumer admissions distinguish policy, original parameters and retained bodies.
use super::super::super::{authentication::*, host};
use super::super::{distinct_subjects, journey};
use super::*;
use worth_query_host::facade::{application_entry::*, causal_inspection::*, runtime::*};
macro_rules! read {
    ($application:expr, $principal:expr, $scope:expr, $read:ident, $enabled:expr, $retain:expr) => {{
        let request = $application.runtime().request($principal, $scope);
        let intent = request.query($read {
            key: "numeric".into(),
            enabled: $enabled,
        });
        let result = if $retain {
            intent.retain_projection_for_inspection().execute_governed(
                capability::Inspect::reference(),
                capability::InspectCausalEvidence::reference(),
                capability::Input::at("numeric"),
            )
        } else {
            intent.execute_governed(
                capability::Inspect::reference(),
                capability::InspectCausalEvidence::reference(),
                capability::Input::at("numeric"),
            )
        }
        .unwrap();
        (
            result.receipt().inspect().observation().unwrap().clone(),
            result.rows().to_vec(),
        )
    }};
}
mod changes;
mod empty;
mod policy;
pub use changes::optional_presence_and_nested_values_are_original_output_changes;
pub use empty::retained_empty_and_absent_bodies_keep_distinct_rich_outcomes;
pub use policy::current_policy_and_original_parameters_precede_body_availability;
