use worth_query_testkit_decl::fragment_fixture::*;

#[test]
fn every_registered_binding_round_trips_owner_supplied_samples() {
    let schema = declare(Fixture).unwrap();
    for binding in schema.record_bindings() {
        let samples = samples(binding);
        assert!(
            !samples.is_empty(),
            "missing samples for {}",
            binding.value_name()
        );
        for sample in samples {
            let encoded = binding
                .encode(sample.value(), "fixture.item.metrics")
                .unwrap();
            let decoded = sample
                .decode(&encoded, "fixture.item.metrics", &mut |_, _| Ok(()))
                .unwrap();
            assert!(
                sample.equals(decoded.as_ref()),
                "round-trip mismatch for {}",
                binding.value_name()
            );
        }
    }
}

#[test]
fn records_lower_arrays_and_nested_records_to_existing_scalar_fields() {
    let schema = declare(Fixture).unwrap();
    let paths: Vec<_> = schema
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Field { aspect, field, .. }
                if aspect == "a58d034afa6a8a8df" =>
            {
                Some(field.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        paths,
        [
            "components[0]",
            "components[1]",
            "components[2]",
            "interval.end",
            "interval.start"
        ]
    );
    assert!(schema.erased().members().iter().any(|member| matches!(member, ApplicationSchemaMember::Aspect { aspect, absence, .. } if aspect == "a6c57178f458971a8" && format!("{absence:?}") == "Optional")));
    assert!(schema.erased().members().iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { target: ApplicationOperationProgramTarget::Write { field, .. }, .. } if field == "components[1]")));
}

#[test]
fn scalar_decode_refusals_retain_the_complete_indexed_path() {
    let shape = <Bindings as Record<Measurements>>::record();
    let value = Measurements {
        components: [2, 3, 5],
        interval: Interval::checked(7, 11).unwrap(),
    };
    let mut encoded = shape.encode(&value, "fixture.item.metrics").unwrap();
    encoded.remove("fixture.item.metrics.components[1]");
    let denied = shape
        .decode(&encoded, "fixture.item.metrics", &mut |_, _| Ok(()))
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.item.metrics.components[1]");
    assert!(matches!(denied.kind(), RecordRefusalKind::MissingField));
}

#[test]
fn nested_decode_uses_the_owners_checked_constructor() {
    let shape = <Bindings as Record<Interval>>::record();
    let mut encoded = shape
        .encode(
            &Interval::checked(3, 11).unwrap(),
            "fixture.item.metrics.interval",
        )
        .unwrap();
    let start = encoded
        .remove("fixture.item.metrics.interval.start")
        .unwrap();
    let end = encoded.remove("fixture.item.metrics.interval.end").unwrap();
    encoded.insert("fixture.item.metrics.interval.start".to_owned(), end);
    encoded.insert("fixture.item.metrics.interval.end".to_owned(), start);
    let denied = shape
        .decode(
            &encoded,
            "fixture.item.metrics.interval",
            &mut |_, _| Ok(()),
        )
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.item.metrics.interval");
    assert!(
        matches!(denied.kind(), RecordRefusalKind::Constructor(cause) if cause.downcast_ref::<ReversedInterval>().is_some())
    );
}

struct RepeatedEntity;
impl Fragment for RepeatedEntity {
    const NAME: &'static str = "repeated";
    type Bindings = Bindings;
    type Members = (Item, (Item,));
}

#[test]
fn duplicate_members_refuse_even_without_a_membership_inference() {
    assert!(
        matches!(declare(RepeatedEntity), Err(ApplicationSchemaDeclarationDenial::DuplicateFragmentMember { path }) if path == "repeated.item")
    );
}

struct RepeatedAspectEntity;
impl Entity for RepeatedAspectEntity {
    const NAME: &'static str = "repeated_aspect";
    type Aspects = (Metrics, (Metrics,));
    type Relations = ();
}
struct RepeatedAspect;
impl Fragment for RepeatedAspect {
    const NAME: &'static str = "repeated";
    type Bindings = Bindings;
    type Members = (RepeatedAspectEntity,);
}

#[test]
fn duplicate_aspects_are_a_typed_build_refusal() {
    assert!(
        matches!(declare(RepeatedAspect), Err(ApplicationSchemaDeclarationDenial::DuplicateFragmentMember { path }) if path == "repeated.repeated_aspect.metrics")
    );
}

#[test]
fn the_original_borrowed_context_controls_the_checked_constructor() {
    use std::num::NonZeroU64;
    use worth_query_testkit_decl::fragment_context_fixture::{
        ContextBindings, InvalidPositive, ValidationContext,
    };
    let shape = <ContextBindings as Record<NonZeroU64>>::record();
    let encoded = shape
        .encode(
            &NonZeroU64::new(7).unwrap(),
            "fixture.contextual.limit.positive",
        )
        .unwrap();
    let maximum = 3;
    let mut admissions = 0;
    let mut admission = |_, _| {
        admissions += 1;
        Ok(())
    };
    let mut work = ValidationContext {
        maximum: &maximum,
        admission: &mut admission,
    };
    let denied = shape
        .decode(&encoded, "fixture.contextual.limit.positive", &mut work)
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.contextual.limit.positive");
    assert!(
        matches!(denied.kind(), RecordRefusalKind::Constructor(cause) if cause.downcast_ref::<InvalidPositive>() == Some(&InvalidPositive::OutsideOwnerRange))
    );
    assert!(
        admissions > 0,
        "the supplied original admission was consulted"
    );
}

#[test]
fn one_of_cases_are_members_through_their_group() {
    fn aspect<E: Entity, A, W>(_: E, _: A)
    where
        A: In<E::Aspects, W, E>,
    {
    }
    aspect(Item, Measured);
    aspect(Item, Named);
    aspect(Item, Noted);
    aspect(Item, Form);
    let schema = declare(Fixture).unwrap();
    let catalog = schema.member_provenance().fragment_catalog();
    let fragment = catalog.fragment(Fixture).unwrap();
    let group = fragment.group::<Item, Form>().unwrap();
    assert!(!group.optional());
    assert_eq!(group.cases().len(), 3);
    assert!(group.contains::<Measured>());
    assert_eq!(
        catalog
            .fragment(worth_query_testkit_decl::fragment_context_fixture::Contextual)
            .unwrap()
            .path(),
        "fixture.contextual"
    );
    assert!(
        !fragment
            .relation::<Item, Next>()
            .unwrap()
            .integrity
            .endpoints
            .self_edges_allowed
    );
}

struct OptionalFormItem;
impl Entity for OptionalFormItem {
    const NAME: &'static str = "item";
    type Aspects = (Metrics, Option<Label>, Option<Exclusive<Form>>);
    type Relations = (Next,);
}
struct OptionalFormSchema;
impl Fragment for OptionalFormSchema {
    const NAME: &'static str = "fixture";
    type Bindings = Bindings;
    type Members = (OptionalFormItem,);
}

#[test]
fn group_optionality_is_part_of_its_derived_case_contract_revision() {
    fn revision(schema: &ErasedApplicationSchemaDeclaration) -> AspectContractRevision {
        schema
            .members()
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::Aspect {
                    aspect, revision, ..
                } if aspect == "ac9a7ff83b921347e" => Some(*revision),
                _ => None,
            })
            .unwrap()
    }
    let required = declare(Fixture).unwrap();
    let optional = declare(OptionalFormSchema).unwrap();
    assert_ne!(revision(required.erased()), revision(optional.erased()));
}

#[test]
fn native_decode_cannot_replace_original_admission_with_the_owners_domain_context() {
    use std::num::NonZeroU64;
    use worth_query_testkit_decl::fragment_context_fixture::{ContextBindings, ValidationContext};
    let shape = <ContextBindings as Record<NonZeroU64>>::record();
    let encoded = shape
        .encode(
            &NonZeroU64::new(7).unwrap(),
            "fixture.contextual.limit.positive",
        )
        .unwrap();
    let maximum = 11;
    let mut owner_calls = 0;
    let mut owner_admission = |_, _| {
        owner_calls += 1;
        Ok(())
    };
    let mut work = ValidationContext {
        maximum: &maximum,
        admission: &mut owner_admission,
    };
    let denied = shape
        .decode_admitted(
            &encoded,
            "fixture.contextual.limit.positive",
            &mut work,
            &mut |_, _| Err(ValueCodecDenial::Admission),
        )
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.contextual.limit.positive.value");
    assert!(matches!(
        denied.kind(),
        RecordRefusalKind::Decode(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::Admission,
            ..
        })
    ));
    assert_eq!(
        owner_calls, 0,
        "the checked constructor cannot bypass original refusal"
    );
}

#[test]
fn unknown_flat_columns_refuse_instead_of_disappearing() {
    let shape = <Bindings as Record<Interval>>::record();
    let mut encoded = shape
        .encode(
            &Interval::checked(3, 11).unwrap(),
            "fixture.item.metrics.interval",
        )
        .unwrap();
    let value = encoded["fixture.item.metrics.interval.start"].clone();
    encoded.insert("fixture.item.metrics.interval.other".to_owned(), value);
    let denied = shape
        .decode(
            &encoded,
            "fixture.item.metrics.interval",
            &mut |_, _| Ok(()),
        )
        .unwrap_err();
    assert_eq!(denied.path(), "fixture.item.metrics.interval.other");
    assert!(matches!(denied.kind(), RecordRefusalKind::UnexpectedField));
}

impl ApplicationSchema for RepeatedEntity {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}

impl ApplicationSchema for RepeatedAspect {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}

impl ApplicationSchema for OptionalFormSchema {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}

struct RepeatedRegistration;
impl Fragment for RepeatedRegistration {
    const NAME: &'static str = "repeated_registration";
    type Bindings = Bindings;
    type Members = (Item, SetMetric, (SetMetric,));
}
impl ApplicationSchema for RepeatedRegistration {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
#[test]
fn a_repeated_registration_is_a_typed_member_refusal() {
    assert!(matches!(
        declare(RepeatedRegistration),
        Err(ApplicationSchemaDeclarationDenial::DuplicateFragmentMember { .. })
    ));
}

#[test]
fn a_selected_handle_carries_its_record_requirement_into_generic_code() {
    fn resolve<F, E, A, T>(
        selection: FieldSelection<F, E, A, T>,
    ) -> Result<RecordField<E, A, T>, ApplicationSchemaDeclarationDenial> {
        selection.resolve()
    }
    let field = resolve(select_field::<_, _, _, u64, _, _>(
        Fixture,
        Item,
        Metrics,
        "components[1]",
    ))
    .unwrap();
    assert_eq!(field.locus().field(), "components[1]");
}
