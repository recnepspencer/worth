//! Current Head data survives typed historical symbol refusal through real read owners.
use super::{record_field_values::name_value, symbol_view_source::SymbolViewRecipe};
use std::sync::atomic::Ordering;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial,
    history::BranchId,
    storage::{RelationalStorageSubject, RelationalStorageUnavailableCause as Cause},
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;
#[test]
fn current_symbols_preserve_full_history_refusal_and_original_head_values() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = SymbolViewRecipe::new(request);
    for damage in [false, true] {
        let fixture = recipe.reopen(request);
        let mut txn = begin_main_transaction(&fixture.runtime);
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        if damage {
            *fixture.source.failed_member.lock().unwrap() = Some(fixture.historical_page);
        }
        let current = txn
            .read_entity(request, fixture.entity)
            .unwrap()
            .effective()
            .cloned()
            .expect("a Head miss cannot escape as absence");
        assert_eq!(current.entity_id, fixture.entity);
        assert!(
            matches!(name_value(&current),Some(AspectValue::String(InternedString::Symbol(symbol))) if symbol.0==fixture.current.0)
        );
        let symbols = fixture.runtime.observe_resident_symbol_dictionary();
        assert_eq!(
            symbols.resolve(fixture.current),
            Some("paged-view-value-0600")
        );
        assert!(
            symbols.resolve(fixture.historical).is_none(),
            "Head must not admit the history-only page of a physically Full cell"
        );
        assert!(
            symbols.len() < 400,
            "one current closure has ample headroom below the 601 admitted texts"
        );
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        let read_full = || {
            fixture.runtime.entity_record_at_observation(
                request,
                &basis.observation(),
                fixture.entity,
            )
        };
        if damage {
            let failure = read_full().unwrap_err();
            let RelationalObservationReadDenial::Execution(
                RelationalExecutionDenialCause::StorageUnavailable(storage),
            ) = failure
            else {
                panic!("original Full failure remains typed")
            };
            assert!(
                matches!(storage.subject,RelationalStorageSubject::Symbols{first,last} if first<=fixture.historical&&fixture.historical<=last)
            );
            assert_eq!(storage.cause, Cause::Damaged);
            *fixture.source.failed_member.lock().unwrap() = None;
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            assert_eq!(read_full().unwrap_err(), failure);
            let again = txn
                .read_entity(request, fixture.entity)
                .unwrap()
                .effective()
                .cloned()
                .unwrap();
            assert_eq!(again, current);
            assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
        } else {
            assert!(read_full().unwrap().is_some());
            assert!(
                fixture
                    .runtime
                    .observe_resident_entity_metadata_history_len(
                        &BranchId("main".into()),
                        fixture.entity,
                    )
                    .unwrap()
                    > 1,
                "Full must retain both real committed field versions"
            );
            assert_eq!(
                fixture
                    .runtime
                    .observe_resident_symbol_dictionary()
                    .resolve(fixture.historical),
                Some("paged-view-value-0300")
            );
            let history = fixture
                .runtime
                .history()
                .entity_aspect_history(request, &BranchId("main".into()), fixture.entity, None)
                .unwrap();
            assert!(
                history.len() > 1,
                "the real committed field versions remain publicly readable"
            );
            let covered = fixture.runtime.fork().unwrap();
            let (_, covered_basis) = covered
                .observe_branch(&covered.main_branch_identity())
                .unwrap();
            assert!(covered
                .entity_record_at_observation(request, &covered_basis.observation(), fixture.entity)
                .unwrap()
                .is_some());
            assert_eq!(
                covered
                    .observe_branch_region_residency(
                        &BranchId("main".into()),
                        fixture.entity.partition_id,
                    )
                    .unwrap()
                    .head,
                worth_relational::facade::durability::RelationalRegionProbeStatus::Resident,
                "a Full symbol guard must cover Head without recollecting the view"
            );
            let mut read = begin_main_transaction(&covered);
            assert_eq!(
                read.read_entity(request, fixture.entity)
                    .unwrap()
                    .effective(),
                Some(&current)
            );
        }
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            frontier
        );
    }
}
