use super::MixedCarrierFixtureProfile;

pub(super) fn seeded_workspace(
    profile: MixedCarrierFixtureProfile,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    let rows = (0..profile.collection_rows)
        .map(|index| (format!("mixed.{index:04}"), initial_value(profile, index)))
        .collect();
    worth_ui_query_binding::certification::seeded_mixed_projection_workspace(rows)
}

pub(crate) fn initial_value(profile: MixedCarrierFixtureProfile, index: usize) -> String {
    if index == 0 {
        return "Ready".to_owned();
    }
    let len = if index + 1 == profile.collection_rows {
        profile.final_text_bytes
    } else {
        profile.ordinary_text_bytes
    };
    format!("A{index:04}{}", "x".repeat(len - 5))
}

pub(crate) fn replacement_value(profile: MixedCarrierFixtureProfile, index: usize) -> String {
    let mut value = initial_value(profile, index).into_bytes();
    value[0] = b'B';
    String::from_utf8(value).unwrap()
}
