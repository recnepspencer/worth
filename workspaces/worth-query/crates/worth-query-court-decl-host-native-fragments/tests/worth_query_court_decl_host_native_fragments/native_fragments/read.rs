use super::*;

#[test]
fn two_native_cases_are_corruption_before_a_case_constructor_can_refuse() {
    let world = World::new(Mode::BothBroken);
    let error = world.read().unwrap_err();
    assert!(matches!(error, FragmentDenial::CorruptOneOf { present, .. } if present.len() == 2));
}

#[test]
fn required_one_of_refuses_no_case_and_optional_one_of_returns_none() {
    let world = World::new(Mode::None);
    assert!(
        matches!(world.read(), Err(FragmentDenial::CorruptOneOf { present, .. }) if present.is_empty())
    );
    assert_eq!(world.optional().unwrap(), None);
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
