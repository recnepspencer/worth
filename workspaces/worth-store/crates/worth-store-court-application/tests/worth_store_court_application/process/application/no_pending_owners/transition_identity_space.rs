//! A resumed transition's identity bound is its recorded planning predecessor's.
use super::*;
use worth_foundational::facade::AspectIdentity;
use worth_relational::facade::identity::KindId;
use worth_store_application::{
    ApplicationHomeNativeIdentityBound, ApplicationHomeNativeSchemaPreparation,
};

#[test]
fn a_resumed_transition_bounds_identities_at_its_planning_predecessor_not_its_durable_prefix() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    // The durable target step introduced KindId(2000) and AspectIdentity(MAX).
    let initial = super::transition_source::interrupted::home(directory.path(), &execution).initial;
    let input = super::transition_opening::successor_request(directory.path());
    let admitted = worth_store_application::admit_home(
        worth_store_application::recover_home(input, &execution).unwrap(),
        &execution,
    )
    .unwrap();
    let mut home = worth_store_application::attach_relational(admitted, &execution)
        .unwrap_or_else(|_| panic!("actual same Home Native attachment"));
    {
        let ApplicationHomeNativeSchemaPreparation::Existing {
            native,
            source,
            identities,
            installation,
        } = home.prepare_native_schema_for_home(&execution).unwrap()
        else {
            panic!("same directory cannot initialize");
        };
        assert!(
            installation.is_none(),
            "a pending unbound transition grants no successor installation"
        );
        assert_eq!(source.authority_snapshot(), initial);
        // Store hands the resumed open the identities occupied at its planning
        // predecessor: the role only its durable target step introduced is not
        // occupied there, so the successor recompile reproduces its plan.
        let ApplicationHomeNativeIdentityBound::TransitionPlanning {
            maximum_kind,
            maximum_aspect,
        } = identities
        else {
            panic!("a pending transition resumes from its planning predecessor: {identities:?}");
        };
        let initial_kind = initial
            .entity_kinds
            .iter()
            .map(|kind| kind.kind_id)
            .chain(initial.relation_kinds.iter().map(|kind| kind.kind_id))
            .max();
        assert!(initial_kind.is_some());
        assert_eq!(maximum_kind, initial_kind);
        assert!(maximum_kind < Some(KindId(2000)));
        assert!(maximum_aspect < Some(AspectIdentity(u64::MAX)));
        assert!(
            native.history().historical_latest_commit().is_none(),
            "projection does not replay or expose a readmitted receiver"
        );
    }
    // No transition continuation or final binding is claimed. The original
    // durable owner custody remains in the actual Home, then is abandoned.
    drop(home);
}
