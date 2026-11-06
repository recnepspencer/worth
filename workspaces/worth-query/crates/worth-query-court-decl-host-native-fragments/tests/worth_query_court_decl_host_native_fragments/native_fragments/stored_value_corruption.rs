//! Damaged Native storage must fail through the installed SDK's own row decoder.
use super::*;
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_testkit_decl::fragment_codec_fixture::*;
use worth_query_testkit_decl::fragment_extension_fixture::Annotations;
mod storage;

#[derive(Clone, Copy, Debug)]
enum Location {
    Owned,
    Extension,
}
impl Location {
    fn field_path(self) -> &'static str {
        match self {
            Self::Owned => "fixture.item.owned_value.value",
            Self::Extension => "fixture.annotations.item.extended_value.value",
        }
    }
    fn read(self, world: &World) -> Result<Option<StoredValue>, FragmentDenial> {
        world
            .admitted_read(ChangeForm::reference(), |reader, item| {
                self.read_from(reader, item)
            })
            .unwrap()
    }
    fn read_from(
        self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Fixture,
            ChangeForm,
        >,
        item: &ObservedEntity<Item>,
    ) -> Result<Option<StoredValue>, FragmentDenial> {
        work(|work| match self {
            Self::Owned => reader
                .fragment(Fixture)
                .unwrap()
                .read(item, OwnedValue, work),
            Self::Extension => {
                reader
                    .fragment(Annotations)
                    .unwrap()
                    .read(item, ExtendedValue, work)
            }
        })
    }
}
fn lawful_world() -> World {
    World::seeded(Mode::Named, |seed| {
        seed.record_field(
            record_field(Fixture, Item, OwnedValue, "value").unwrap(),
            Number { number: 7 },
        )
        .unwrap()
        .record_field(
            record_field(Annotations, Item, ExtendedValue, "value").unwrap(),
            Number { number: 7 },
        )
        .unwrap()
    })
}
fn refused(replacement: Option<AspectValue>, assert_kind: impl Fn(&RecordRefusalKind)) {
    for location in [Location::Owned, Location::Extension] {
        let world = lawful_world();
        let denial = storage::read_damaged(&world, location, replacement.clone()).unwrap_err();
        let FragmentDenial::Record(refusal) = denial else {
            panic!("{location:?}: {denial:?}")
        };
        assert_eq!(refusal.path(), location.field_path());
        assert_kind(refusal.kind());
    }
}
#[test]
fn owned_and_extension_view_reads_return_lawful_stored_values() {
    let world = lawful_world();
    for location in [Location::Owned, Location::Extension] {
        assert_eq!(
            location.read(&world).unwrap(),
            Some(StoredValue {
                value: Number { number: 7 }
            })
        );
    }
}
#[test]
fn owned_and_extension_view_reads_refuse_a_missing_required_field() {
    refused(None, |kind| {
        assert!(matches!(kind, RecordRefusalKind::MissingField), "{kind:?}")
    });
}
#[test]
fn owned_and_extension_view_reads_refuse_a_non_canonical_value() {
    // Valid grammar and meaning, but whitespace violates this binding's canonical bytes.
    refused(
        Some(AspectValue::String("{\"number\": 7}".into())),
        |kind| {
            assert!(
                matches!(kind, RecordRefusalKind::Decode(ApplicationValueDecodeDenial::Canonical {
            binding_identity, cause: ValueCodecDenial::NonCanonical
        }) if binding_identity == &NumberBinding::IDENTITY),
                "{kind:?}"
            );
        },
    );
}
#[test]
fn owned_and_extension_view_reads_refuse_a_malformed_value() {
    refused(Some(AspectValue::String("{".into())), |kind| {
        assert!(
            matches!(kind, RecordRefusalKind::Decode(ApplicationValueDecodeDenial::Canonical {
            binding_identity, cause: ValueCodecDenial::Malformed
        }) if binding_identity == &NumberBinding::IDENTITY),
            "{kind:?}"
        );
    });
}
#[test]
fn owned_and_extension_view_reads_refuse_the_wrong_scalar_family() {
    refused(Some(AspectValue::Bool(true)), |kind| {
        assert!(
            matches!(kind, RecordRefusalKind::Decode(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            binding_identity, expected: ScalarAspectType::String, observed: ScalarAspectType::Bool
        }) if binding_identity == &NumberBinding::IDENTITY),
            "{kind:?}"
        );
    });
}
