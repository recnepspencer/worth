//! Bootstrap only a principal/context; author all catalog Source facts through the program.
use super::super::super::super::authentication;
use super::super::{host, rule};
use super::*;
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};

pub fn populate(world: &host::Host, rules: &rule::Calls) {
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    for (key, edit) in [
        Edit::Directory {
            identity: "A".into(),
        },
        Edit::Directory {
            identity: "B".into(),
        },
        Edit::Item {
            directory: "A".into(),
            identity: "x".into(),
            name: "shared".into(),
        },
        Edit::Item {
            directory: "B".into(),
            identity: "y".into(),
            name: "taken".into(),
        },
        Edit::Share {
            directory: "B".into(),
            item: "x".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        rules.records.lock().unwrap().clear();
        let creates_scope = matches!(edit, Edit::Directory { .. });
        let result = request
            .advanced()
            .mutate(edit)
            .idempotency(&(0x919300 + key as u64))
            .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "ordinary catalog authoring failed: {result:?}"
        );
        let records = rules.records.lock().unwrap();
        assert!(
            !records.is_empty(),
            "each authored scope/membership must reach the actual installed rule"
        );
        if creates_scope {
            assert!(
                records
                    .iter()
                    .any(|record| record.before.is_none() && record.after.is_empty()),
                "a new empty Source scope is invoked despite having no committed occurrence"
            );
        }
    }
}
