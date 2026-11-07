//! The shared Query compiler prepares B against a genuine nonempty A Home.
use super::*;
use worth_query_host::facade::{
    application_installation::*,
    declaration::{application_program::*, application_schema::*},
    primary_graph::SignalConditionalEvaluationBudget,
};

mod successor_schema;
use successor_schema::{SuccessorProgram, SuccessorSchema};
fn successor() -> PreparedHomeApplication<SuccessorSchema, SuccessorProgram> {
    let declaration = SuccessorSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<SuccessorSchema, SuccessorProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    prepare_home_program(
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build(),
        WorthQueryInMemoryApplicationLimits::new(
            identity::world_resources(),
            worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        None,
        |_, _| panic!("Relational-only B has no authored Signal/Bridge seed"),
    )
    .unwrap()
}

#[test]
fn successor_query_compilation_prepares_changed_meaning_without_installing_or_publishing() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(directory.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, mut home) = unshared(directory.path(), &phase);
        let query = finalize_native(query, &mut home, &phase)
            .unwrap_or_else(|refused| panic!("real A readmission: {:?}", refused.reason()));
        let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
        let original = native.config().schema.registry.authority_snapshot();
        let (_, basis) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        let head = native.history().historical_latest_commit();
        let rows = administration::rows(native, &execution, "Account");
        assert_eq!(rows.len(), 1);
        let target = successor()
            .prepare_home_program_transition_target(&phase, native, &query, &basis)
            .unwrap_or_else(|refused| panic!("real B compilation: {:?}", refused.reason()));
        assert_ne!(
            target.native_contracts().schema_digest().bytes(),
            native.config().schema.registry.authority_digest_bytes()
        );
        assert_eq!(
            native.config().schema.registry.authority_snapshot(),
            original
        );
        assert_eq!(native.history().historical_latest_commit(), head);
        assert_eq!(administration::rows(native, &execution, "Account"), rows);
        let (_, after) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        assert_eq!(after.descriptor(), basis.descriptor());
        drop((target, query, basis, after));
        assert!(matches!(
            home.close(&execution).unwrap(),
            ApplicationHomeCloseOutcome::Closed { .. }
        ));
    })
    .unwrap();
}

#[test]
fn successor_query_compilation_refuses_a_real_foreign_predecessor_before_installing() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(first.path(), execution);
    fresh_closed(second.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query_a, mut home_a) = unshared(first.path(), &phase);
        let query_a = finalize_native(query_a, &mut home_a, &phase).unwrap_or_else(|refused| panic!("real A: {:?}", refused.reason()));
        let (query_b, mut home_b) = unshared(second.path(), &phase);
        let (native, _, _) = home_b.recovered_query_finalization_parts().unwrap();
        let original = native.config().schema.registry.authority_snapshot();
        let (_, basis) = native.observe_branch(&native.main_branch_identity()).unwrap();
        let refused = successor().prepare_home_program_transition_target(&phase, native, &query_a, &basis)
            .err().expect("foreign real Home cannot supply compiler roles");
        assert!(matches!(refused.reason(), QueryHomeTransitionPreparationDenial::Native(
            worth_relational::facade::runtime::RelationalHomeProgramContractDenial::ForeignRuntime)));
        assert_eq!(native.config().schema.registry.authority_snapshot(), original);
        drop((refused, query_a, query_b, basis));
        assert!(matches!(home_b.close(&execution).unwrap(), ApplicationHomeCloseOutcome::Closed { .. }));
        assert!(matches!(home_a.close(&execution).unwrap(), ApplicationHomeCloseOutcome::Closed { .. }));
    }).unwrap();
}
