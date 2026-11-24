//! checkpoint reopens ready output without producer contact and recomputes after source change.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;

#[cfg(test)]
mod serial {
    use super::*;
    #[test]
    fn checkpoint_reopens_ready_output_without_producer_contact_and_recomputes_after_source_change()
    {
        let _guard = checkpoint_recovery_test_guard();
        let provider_observation = crate::producer::ProviderContacts::default();
        provider_observation.reset();
        let application = support::observed::open_directory(None, &provider_observation);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let settled = settle(&request, &application);
        assert!(settled.work().producer_contact_count() > 0);
        assert!(provider_observation.count() > 0);
        drop(settled);
        drop(principal);
        drop(scope);

        let directory = application.checkpoint_and_close_for_reopen();

        provider_observation.reset();
        let restored = support::observed::open_directory(Some(directory), &provider_observation);
        let (scope, principal) = authenticate(&restored);
        let request = restored.request(&principal, &scope);
        let restored_settlement = settle(&request, &restored);
        assert_eq!(restored_settlement.work().producer_contact_count(), 0);
        assert_eq!(restored_settlement.work().delivery_contact_count(), 0);
        assert_eq!(restored_settlement.work().derived_publication_count(), 0);
        assert_eq!(provider_observation.count(), 0);
        let retained = crate::checkpoint_recovery::support::retain_read(&restored, &request)
            .expect("the recovered root is observable");
        request
            .at(&retained)
            .require_current_program_output(&restored_settlement)
            .expect("the restored settlement is current at its recovered source");

        let repeated_restored_settlement = settle(&request, &restored);
        assert_eq!(
            repeated_restored_settlement.work().producer_contact_count(),
            0
        );
        assert_eq!(provider_observation.count(), 0);
        drop(repeated_restored_settlement);

        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the recovered source is readable");
        provider_observation.reset();
        let before_contacts = restored.producer_contacts_on_this_thread_for_test();
        let (edited, writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(3),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&77_u64)
            .execute_performed::<CheckpointProgram, CheckpointRoot>(
                &restored,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the recovered source accepts a fresh edit")
            });
        let WorthQueryApplicationPerformedMutationOutcome::Performed(edited) = edited else {
            panic!("the recovered edit co-commits its required outputs");
        };
        // Cold-symbol preparation may abandon a candidate and enter handlers
        // again. The contract is one final publication, current outputs and
        // contact-free reuse, not an aggregate preparation-entry count.
        let edit_entries = restored.producer_contacts_on_this_thread_for_test() - before_contacts;
        assert!(
            edit_entries > 0,
            "the changed source executes real producers"
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        assert!(provider_observation.count() > 0);
        let value = request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            value.rows()[0].value,
            length(4),
            "authored y=3 yields y+1 before delivery"
        );
        drop(value);
        for (key, expected) in [("final:anchor-a", 5), ("final:anchor-b", 3)] {
            let output = request
                .query(PlanarOutputRead {
                    body_key: key.into(),
                })
                .execute()
                .unwrap();
            assert_eq!(output.rows()[0].body_key, key);
            assert_eq!(
                output.rows()[0].value,
                length(expected),
                "each source y + 2 is published before delivery"
            );
        }
        let current = crate::checkpoint_recovery::support::retain_read(&restored, &request)
            .expect("the edited root is observable");
        assert_eq!(
            edited
                .receipt()
                .committed_product_publication()
                .composite_commit(),
            current.selected_commit()
        );
        assert!(matches!(
            request.at(&current).require_current_program_output(
                &restored_settlement),
            Err(WorthQueryOutputCurrentnessDenial::Output(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
        ));
        drop(restored_settlement);

        let (recomputed, delivery_writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                settle(&request, &restored)
            });
        // The edit co-committed the root and both dependents. Delivery hands
        // back those three original receipts and their own account of that
        // production; it contacts no producer and writes nothing.
        assert_eq!(recomputed.root_producer_contacts_in_this_demand(), 0);
        assert_eq!(
            (
                recomputed.work().producer_contact_count(),
                recomputed.work().derived_publication_count()
            ),
            (3, 3)
        );
        assert_eq!(
            recomputed
                .root_receipt()
                .expect("the co-committed root keeps its original producer receipt")
                .committed_product_publication()
                .composite_commit(),
            current.selected_commit()
        );
        assert_eq!(
            delivery_writes.published_rows.len() + delivery_writes.omitted_publications,
            0
        );
        request
            .at(&current)
            .require_current_program_output(&recomputed)
            .unwrap();
        let (reused, reuse_writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                settle(&request, &restored)
            });
        assert_eq!(reused.root_producer_contacts_in_this_demand(), 0);
        assert_eq!(
            (
                reused.work().producer_contact_count(),
                reused.work().derived_publication_count()
            ),
            (3, 3),
            "a later current demand rejoins the same original receipts"
        );
        assert_eq!(
            reuse_writes.published_rows.len() + reuse_writes.omitted_publications,
            0
        );
        assert_eq!(
            restored.producer_contacts_on_this_thread_for_test() - before_contacts,
            edit_entries,
            "revalidation may call the provider but enters no producer handler"
        );
        assert_eq!(
            support::retain_read(&restored, &request)
                .unwrap()
                .selected_commit(),
            current.selected_commit()
        );
        let value = request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(value.rows()[0].value, length(4));
    }
}
