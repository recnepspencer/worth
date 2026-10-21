//! Same endpoint preserves current evidence; retarget and clear supersede only that row.
use super::super::{
    operator_identity::{authenticate_operator, block_on, request_scope},
    path_signing::authentication,
    world::install,
};
use super::{
    declaration::Encoding,
    signature_query::{ReadCurrent, ReadHistory},
    signing::Sign,
    Assign,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
#[test]
fn native_reference_noop_preserves_signatures_and_retarget_or_clear_supersedes_only_dependencies() {
    for replacement in [Some("parent-c"), None] {
        let mut app = install();
        let owner = authentication::install(&mut app);
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        for (id, row) in [(830_u64, "row-a"), (831, "row-sibling")] {
            let result = app
                .request(&principal, &scope)
                .advanced()
                .mutate(Assign {
                    row: row.into(),
                    selected: Some("parent-a".into()),
                    encoding: Encoding::Normal,
                })
                .without_source()
                .idempotency(&id)
                .execute_in_program(&app, Allocation::SystemAllocation)
                .unwrap();
            assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
            let sign_key = id + 10;
            let mut request = app
                .request(&principal, &scope)
                .advanced()
                .mutate(Sign(row.into()))
                .without_source()
                .idempotency(&sign_key);
            let mut prepared = match request
                .prepare_in_program(&app, Allocation::SystemAllocation)
                .unwrap()
            {
                Preparation::Prepared(prepared) => prepared,
                _ => panic!("fresh signing intent"),
            };
            let event = block_on(owner.authenticate(
                "verify",
                &principal,
                prepared.authentication_intent().unwrap().clone(),
                &scope,
            ))
            .unwrap();
            prepared.sign(&event).unwrap();
            let result = prepared.commit(Allocation::SystemAllocation);
            assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
        }
        let current = |row: &str| {
            app.request(&principal, &scope)
                .query(ReadCurrent(row.into()))
                .execute()
                .unwrap()
                .rows()
                .to_vec()
        };
        let original = current("row-a");
        let sibling = current("row-sibling");
        assert_eq!(original.len(), 1);
        assert_eq!(original[0].0, "parent-a");
        let noop = app
            .request(&principal, &scope)
            .advanced()
            .mutate(Assign {
                row: "row-a".into(),
                selected: Some("parent-a".into()),
                encoding: Encoding::Normal,
            })
            .without_source()
            .idempotency(&850_u64)
            .execute_in_program(&app, Allocation::SystemAllocation)
            .unwrap();
        assert!(matches!(noop, Outcome::Committed { .. }), "{noop:?}");
        assert_eq!(
            current("row-a"),
            original,
            "same actual endpoint may not churn or supersede evidence"
        );
        let result = app
            .request(&principal, &scope)
            .advanced()
            .mutate(Assign {
                row: "row-a".into(),
                selected: replacement.map(str::to_owned),
                encoding: Encoding::Normal,
            })
            .without_source()
            .idempotency(&851_u64)
            .execute_in_program(&app, Allocation::SystemAllocation)
            .unwrap();
        assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
        assert!(current("row-a").is_empty());
        assert_eq!(current("row-sibling"), sibling);
        let history = app
            .request(&principal, &scope)
            .query(ReadHistory("row-a".into()))
            .execute()
            .unwrap();
        assert_eq!(history.rows(), original);
    }
}
