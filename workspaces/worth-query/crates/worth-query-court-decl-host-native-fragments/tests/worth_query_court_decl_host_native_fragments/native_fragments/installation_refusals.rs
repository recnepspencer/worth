//! Installation carries field codec and Native commit refusals through publication.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryApplicationPreparationDenial;
use worth_query_testkit_decl::fragment_capacity_fixture::{Limited, Text, TextBinding};

#[test]
fn over_capacity_field_installation_preserves_its_locus_and_typed_codec_cause() {
    let error = match World::try_seeded(Mode::Named, |seed| {
        seed.field(select_field(Limited, Item, Text, "text"), "four".to_owned())
    }) {
        Err(error) => error,
        Ok(_) => panic!("the over-capacity field must refuse installation"),
    };
    let WorthQueryApplicationPreparationDenial::InitialState(denial) = error else {
        panic!("the field codec must refuse the initial seed: {error:?}");
    };
    let (field, cause) = denial
        .field_encoding_denial()
        .expect("the original typed field refusal");
    let selected: FieldSelection<Limited, Item, Text, String> =
        select_field(Limited, Item, Text, "text");
    assert_eq!(field, selected.binding_recipe().locus());
    assert!(denial.subject().contains(field.field()));
    assert_eq!(
        cause,
        &ApplicationValueEncodeDenial::Canonical {
            binding_identity: TextBinding::IDENTITY,
            cause: ValueCodecDenial::StringBytes,
        }
    );
}

#[test]
fn a_field_within_the_same_declared_capacity_installs_and_reads_back() {
    let world = World::try_seeded(Mode::Named, |seed| {
        seed.field(select_field(Limited, Item, Text, "text"), "abc".to_owned())
    })
    .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_operation::<ChangeForm, _>(
                    &phase,
                    |reader| {
                        let item = reader
                            .fragment(Fixture)
                            .unwrap()
                            .resolve(
                                record_field(Fixture, Item, ItemIdentity, "id")
                                    .resolve()
                                    .unwrap(),
                                11u64,
                            )
                            .unwrap();
                        let row =
                            work(|work| reader.fragment(Limited).unwrap().read(&item, Text, work))
                                .unwrap();
                        assert_eq!(row.unwrap().text, "abc");
                    },
                    SystemAllocation,
                )
                .unwrap();
        })
        .unwrap();
}

#[test]
fn conditional_installation_retains_the_original_native_field_commit_refusal() {
    use worth_query_host::facade::primary_graph::{
        ConflictClass, ContractValidationDenial, PortableAspectReadmissionDenial,
        RecordAspectPatchDenial, TransactionCommitError,
    };
    let error = match World::try_seeded(Mode::Named, |_| {
        WorthQueryApplicationEntitySeed::new(
            entity(Fixture, Item),
            WorthQueryApplicationEntityKey::new("item").unwrap(),
        )
        .field(select_field(Fixture, Item, ItemIdentity, "id"), 11u64)
        .field(select_field(Fixture, Item, Metrics, "components[0]"), 2u64)
    }) {
        Err(error) => error,
        Ok(_) => panic!("the incomplete stored aspect must refuse publication"),
    };
    let WorthQueryApplicationPreparationDenial::ConditionalPublication(denial) = error else {
        panic!("expected the conditional publication refusal: {error:?}");
    };
    let WorthQueryConditionalRuntimeInstallationDenialKind::PrimaryGraphPublication(cause) =
        denial.kind()
    else {
        panic!("expected the primary publication cause: {denial:?}");
    };
    let original = cause
        .native_commit_denial()
        .expect("the original Native transaction refusal");
    let TransactionCommitError::Conflict { error, .. } = original else {
        panic!("expected a rejected field patch: {original:?}");
    };
    assert!(
        matches!(&error.class, ConflictClass::RecordAspectPatchDenied {
        denial: RecordAspectPatchDenial::ReadmissionDenied(PortableAspectReadmissionDenial::ValueValidation {
            denial: ContractValidationDenial::MissingRequiredField(field), ..
        }), ..
    } if field.as_str() == "components[1]"),
        "{original:?}"
    );
}
