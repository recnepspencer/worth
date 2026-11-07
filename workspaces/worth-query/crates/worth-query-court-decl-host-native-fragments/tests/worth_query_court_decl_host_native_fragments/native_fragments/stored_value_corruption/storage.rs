//! Damage the actual stored image after lawful bootstrap, leaving contracts and presence intact.
use super::*;

pub(super) fn read_damaged(
    world: &World,
    location: Location,
    replacement: Option<AspectValue>,
) -> Result<Option<StoredValue>, FragmentDenial> {
    let entity = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let item = reader
                            .resolve_entity(select_field(Fixture, Item, ItemIdentity, "id"), 11u64)
                            .unwrap();
                        *item.entity_id_for_test()
                    },
                    SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let locus = match location {
        Location::Owned => {
            record_field::<_, _, _, Number, _, _>(Fixture, Item, OwnedValue, "value")
                .unwrap()
                .locus()
                .clone()
        }
        Location::Extension => {
            record_field::<_, _, _, Number, _, _>(Annotations, Item, ExtendedValue, "value")
                .unwrap()
                .locus()
                .clone()
        }
    };
    world
        .admitted_read(ChangeForm::reference(), |reader, item| {
            let base = reader.reader_for_test();
            let locator = base
                .layout_for_test()
                .field_locator_for_test(locus.entity(), locus.aspect(), locus.field())
                .unwrap();
            let runtime = base.native_for_test().committed_runtime_for_test().unwrap();
            assert!(
                runtime.corrupt_snapshot_field_for_test(
                    base.execution_for_test(),
                    base.snapshot_for_test(),
                    entity,
                    locator,
                    replacement
                ),
                "the lawful stored field must exist before damage"
            );
            location.read_from(reader, item)
        })
        .unwrap()
}
