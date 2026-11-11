//! Compensation cannot publish an original whose material cannot yet continue.
use super::*;
#[test]
fn directory_compensation_refuses_before_original_publication_and_survives_reopen() {
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationRequestMutationDenialKind as Kind,
        product::WorthQueryDurableStateAdmissionDenial as Durable,
    };
    let location = std::env::temp_dir().join(format!(
        "worth-compensation-refusal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = lease();
    let execution = ExecutionRequest::leased(&lease);
    let calls = Arc::new(Calls::default());
    let home = open(&location, &execution, calls.clone(), true);
    let before = home.runtime().runtime().current_world();
    let updates_before = calls.updates.load(Ordering::SeqCst);
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let denied = home
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(created_content::Issue {
                key: "new-sample".into(),
                samples: vec![9],
            })
            .without_source()
            .idempotency(&"unsupported-compensation".to_owned())
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .err()
            .expect("compensation material has no graceful continuation");
        assert!(matches!(
            denied.kind(),
            Kind::DurableStateAdmission(Durable::UnsupportedCorrectionCompensation)
        ));
        assert_eq!(home.runtime().runtime().current_world(), before);
        assert_eq!(calls.updates.load(Ordering::SeqCst), updates_before);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("refusal close: {cause:?}"));
    let home = open(&location, &execution, Arc::new(Calls::default()), false);
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let rows = home
            .runtime()
            .request(&principal, &scope)
            .query(query::ReadSource {
                root: "record".into(),
            })
            .execute()
            .unwrap();
        assert!(
            rows.rows().is_empty(),
            "refused compensation must not create its sample entity"
        );
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("refusal reopen: {cause:?}"));
    std::fs::remove_dir_all(location).unwrap();
}
