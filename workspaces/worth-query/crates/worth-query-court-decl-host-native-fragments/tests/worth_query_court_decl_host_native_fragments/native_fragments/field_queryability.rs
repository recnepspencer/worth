//! Equality posture belongs to each field even when its row and codec are shared.
use super::*;

#[test]
fn same_typed_record_fields_have_independent_installed_equality_postures() {
    let world = World::new(Mode::Named);
    let declaration = <Fixture as ApplicationSchema>::declaration().unwrap();
    let aspect = aspect(Fixture, Item, Metrics).name();
    let fields: Vec<_> = declaration
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Field {
                aspect: owner,
                field,
                value_type,
                equality_queryable,
                ..
            } if owner == aspect && matches!(field.as_str(), "interval.start" | "interval.end") => {
                Some((field.as_str(), value_type.as_str(), *equality_queryable))
            }
            _ => None,
        })
        .collect();
    assert_eq!(fields.len(), 2);
    assert_eq!(
        fields[0].1, fields[1].1,
        "the codec identity stays the same"
    );
    assert!(
        fields
            .iter()
            .find(|row| row.0 == "interval.start")
            .unwrap()
            .2
    );
    assert!(!fields.iter().find(|row| row.0 == "interval.end").unwrap().2);
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected(&phase);
            let request = super::authentication::live_scope();
            let item = selected
                .resolve_entity(
                    &phase,
                    select_field(Fixture, Item, Metrics, "interval.start"),
                    7u64,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            assert_eq!(item.examined_candidate_count(), 1);
            let denial = match selected.resolve_entity(
                &phase,
                select_field(Fixture, Item, Metrics, "interval.end"),
                13u64,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ) {
                Err(denial) => denial,
                Ok(_) => panic!("an unindexed field must refuse equality resolution"),
            };
            assert!(
                matches!(
                    denial.kind(),
                    WorthQueryEntityResolutionDenialKind::Refused(
                        WorthQueryEntityResolutionRefusalKind::FieldNotInstalled
                    )
                ),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "interval.end");
        })
        .unwrap();
}
