//! Later-phase custody is refused by the actual Directory receiver before effects.
use super::*;
use worth_foundational::facade::DurableStateFamily as Family;
use worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial as Durable;
use worth_query_testkit_document_retention::facade::document_retention_model as document;
mod program;

#[test]
fn directory_adoption_refuses_before_custody_and_reopens_its_original_activation() {
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationProgramAdoptionPreparationDenial as Entry,
        primary_graph::WorthQueryBranchAdoptionPreparationDenial as Preparation,
    };
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = program::open(path.clone(), execution);
    let original = home.runtime().runtime().current_world();
    {
        let runtime = home.runtime();
        let scope = document::operator_identity::request_scope();
        let principal =
            document::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        let target = *runtime
            .supported_program::<program::Target>()
            .unwrap()
            .installed_program()
            .revision();
        let requirements = runtime
            .request(&principal, &scope)
            .programs()
            .compare(&target)
            .unwrap();
        let denied = runtime
            .request(&principal, &scope)
            .programs()
            .adopt(&requirements)
            .unwrap()
            .prepare(100_000)
            .err()
            .expect("Directory adoption has no recoverable custody yet");
        assert!(matches!(
            denied,
            Entry::Adoption(Preparation::DurableStateAdmission(Durable::Unsupported {
                family: Family::BranchAdoptionRecovery,
                supported_from_phase: 6
            }))
        ));
        let branches = runtime.runtime().branches();
        let coverage = branches
            .program_adoption_coverage(&[original], std::num::NonZeroUsize::MIN)
            .unwrap();
        let ordered = branches
            .order_program_adoption_coverage(coverage, &[original])
            .unwrap();
        let denied = runtime
            .runtime()
            .prepare_branch_set_adoption(ordered, &target, 100_000, &principal, &scope)
            .err()
            .expect("set adoption shares the custody refusal");
        assert!(matches!(denied, worth_query_host::facade::primary_graph::WorthQueryBranchSetAdoptionPreparationDenial::DurableStateAdmission(Durable::Unsupported {family: Family::BranchAdoptionRecovery, supported_from_phase: 6})));
        assert_eq!(runtime.runtime().current_world(), original);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("refused adoption close: {cause:?}"))
        .reclaim();
    let home = program::open(path, execution);
    let scope = document::operator_identity::request_scope();
    let principal = document::operator_identity::authenticate_operator(
        home.runtime().installed_schema(),
        &scope,
    );
    assert_eq!(
        home.runtime()
            .request(&principal, &scope)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        home.runtime().installed_program().revision()
    );
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("unchanged activation close: {cause:?}"))
        .reclaim();
}
