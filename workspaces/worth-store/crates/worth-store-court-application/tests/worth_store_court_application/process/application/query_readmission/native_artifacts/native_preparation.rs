//! One genuine unshared Native preparation route for ordinary and transitioning Homes.
use super::*;
pub(super) fn unshared(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> (
    PreparedHomeNativeMeaning<HomeIdentitySchema, HomeProgram>,
    UnsharedReadmittedApplicationHome,
) {
    prepare(path, phase, false)
}

pub(super) fn transition_unshared(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> (NativePreparation, UnsharedReadmittedApplicationHome) {
    prepare(path, phase, true)
}

fn prepare(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    transition: bool,
) -> (NativePreparation, UnsharedReadmittedApplicationHome) {
    let query = meaning();
    let execution = phase.execution_request();
    let predecessor = query.admitted_home_binding().unwrap();
    let request = if transition {
        ApplicationHomeRequest::new(path.to_path_buf(), query.home_relational_definition(),
            worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), build())
            .with_program_transition_predecessor(query.home_relational_definition(), predecessor)
    } else {
        ApplicationHomeRequest::new(
            path.to_path_buf(),
            query.home_relational_definition(),
            predecessor,
            build(),
        )
    };
    let home = admit_home(recover_home(request, &execution).unwrap(), &execution).unwrap();
    let mut home =
        attach_relational(home, &execution).unwrap_or_else(|_| panic!("actual Native attaches"));
    let preparation = home.prepare_native_schema_for_home(&execution).unwrap();
    let query = match preparation {
        ApplicationHomeNativeSchemaPreparation::Existing {
            native,
            source,
            maximum_kind,
            maximum_aspect,
        } => query.prepare_home_native_receiver_schema(
            phase,
            native,
            source,
            maximum_kind,
            maximum_aspect,
        ),
        ApplicationHomeNativeSchemaPreparation::Fresh { .. } => {
            panic!("existing Directory fixture selected fresh initialization")
        }
    }
    .unwrap_or_else(|refused| panic!("actual existing schema: {:?}", refused.reason()));
    let home = home
        .readmit(query.home_initial_schema_receipt(), &execution)
        .unwrap_or_else(|refused| panic!("actual Native recovery: {refused:?}"))
        .consume(|home| home);
    (query, home)
}
