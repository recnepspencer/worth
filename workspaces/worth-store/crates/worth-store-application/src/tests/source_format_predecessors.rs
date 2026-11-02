//! Unsupported Native journal versions refuse before descriptor or owner effects.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_store::physical_runtime::{
    AnchoredRecordName, OrderedJournalName, OrderedJournalPublication, PhysicalMutationDeadline,
};

#[derive(Clone, Copy)]
enum Predecessor {
    Canonical,
    Transition,
}

#[test]
fn native_format_predecessors_refuse_filesystem_home_before_incarnation_or_owner_publication() {
    for predecessor in [Predecessor::Canonical, Predecessor::Transition] {
        let directory = tempfile::tempdir().unwrap();
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            CancellationSource::new().token(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let home = crate::readmit_relational(
            crate::admit_home(
                crate::recover_home(super::program::request(directory.path()), &request).unwrap(),
                &request,
            )
            .unwrap(),
            &request,
        )
        .unwrap()
        .consume(|home| home);
        let (name, magic, version) = match predecessor {
            Predecessor::Canonical => (
                worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
                b"WRTHRCCA",
                4u16,
            ),
            Predecessor::Transition => (
                worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
                b"WRTHRBTA",
                1u16,
            ),
        };
        // Deliberately untrusted predecessor framing. The current-only version
        // boundary must refuse before interpreting even an empty old payload.
        let mut bytes = magic.to_vec();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.resize(66, 0);
        home.context
            .serving
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap()
            .publish(OrderedJournalPublication::new(
                1,
                bytes,
                crate::open::physical_admission::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ))
            .unwrap();
        // The actual committed journal survives ordinary abandonment; this
        // test makes no successful close or discarded-source claim.
        drop(home);
        let recovered =
            crate::recover_home(super::program::request(directory.path()), &request).unwrap();
        let (recovered, serving) = recovered.consume(|home| {
            let serving = home.serving.clone();
            (worth_proof::StoreRecovered::from_owner(home), serving)
        });
        let records = serving.anchored_records().unwrap();
        let names = [
            crate::registry::HOME_ANCHOR,
            crate::registry::RELATIONAL_INITIAL_INPUTS_ANCHOR,
        ];
        let before = names.map(|name| {
            records
                .read(AnchoredRecordName::new(name).unwrap(), 1 << 30)
                .unwrap()
                .unwrap()
        });
        let selected = crate::checkpoint::SelectedHomeSource::capture(&serving, &request).unwrap();
        let journals = crate::registry::HOME_JOURNALS
            .map(|journal| OrderedJournalName::new(journal.name).unwrap());
        let heads = journals.map(|name| selected.journal_head(&serving, name, &request).unwrap());
        let refused = crate::admit_home(recovered, &request)
            .err()
            .expect("predecessor version cannot be adopted");
        use worth_relational::facade::durability::{
            RelationalBranchTransitionArtifactFrameRefusal as Transition,
            RelationalCanonicalCommitArtifactFrameRefusal as Canonical,
            RelationalDurableOwnedReadRefusal as Source, RelationalInitialInputsRefusal as Input,
        };
        let reason = match (predecessor, refused) {
            (
                Predecessor::Canonical,
                crate::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::InitialInputs(
                        Input::Source(Source::CanonicalFrame(Canonical::UnsupportedVersion(
                            reason,
                        ))),
                    ),
                ),
            ) => reason,
            (
                Predecessor::Transition,
                crate::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::InitialInputs(
                        Input::Source(Source::TransitionFrame(Transition::UnsupportedVersion(
                            reason,
                        ))),
                    ),
                ),
            ) => reason,
            (_, other) => panic!("typed Native unsupported-version refusal required: {other:?}"),
        };
        assert_eq!(reason.produced().get(), u32::from(version));
        for (name, original) in names.into_iter().zip(before) {
            let actual = records
                .read(AnchoredRecordName::new(name).unwrap(), 1 << 30)
                .unwrap()
                .unwrap();
            assert_eq!(
                actual.version().generation(),
                original.version().generation()
            );
            assert_eq!(actual.bytes(), original.bytes());
        }
        let after = crate::checkpoint::SelectedHomeSource::capture(&serving, &request).unwrap();
        assert_eq!(
            journals.map(|name| after.journal_head(&serving, name, &request).unwrap()),
            heads
        );
        assert!(records
            .read(
                AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR).unwrap(),
                1 << 30
            )
            .unwrap()
            .is_none());
        drop((selected, after, records, serving));
    }
}
