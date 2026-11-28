use super::*;

#[test]
fn native_bootstrap_refuses_zero_or_two_cases_for_a_required_group() {
    for mode in [Mode::None, Mode::Both] {
        let result = World::try_seeded(mode, |seed| seed);
        assert!(matches!(
            result,
            Err(worth_query_host::facade::application_installation::WorthQueryApplicationPreparationDenial::ConditionalPublication(_))
        ));
    }
}

#[test]
fn optional_one_of_reads_as_none_without_a_case() {
    assert_eq!(World::new(Mode::Named).optional().unwrap(), None);
}

#[test]
fn the_native_case_present_selects_the_owner_enum_conversion() {
    assert_eq!(
        World::new(Mode::Named).read().unwrap(),
        FormValue::Named(Caption {
            text: "initial".to_owned()
        })
    );
    assert_eq!(
        World::new(Mode::Measured).read().unwrap(),
        FormValue::Measured(measured())
    );
}
