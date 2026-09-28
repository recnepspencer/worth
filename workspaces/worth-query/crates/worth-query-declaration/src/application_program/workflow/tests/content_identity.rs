use super::*;

#[test]
fn a_recorded_content_identity_reads_back_exactly_what_display_wrote() {
    let identity = ApplicationWorkflowDefinitionContentIdentity(std::array::from_fn(|index| {
        u8::try_from(index * 7 + 3).expect("small index")
    }));
    let recorded = identity.to_string();
    assert_eq!(
        ApplicationWorkflowDefinitionContentIdentity::from_recorded(&recorded),
        Some(identity),
    );
    for malformed in [
        String::new(),
        recorded[..62].to_owned(),
        format!("{recorded}00"),
        recorded.to_uppercase(),
        format!("+{}", &recorded[1..]),
        format!("{}g", &recorded[..63]),
    ] {
        assert_eq!(
            ApplicationWorkflowDefinitionContentIdentity::from_recorded(&malformed),
            None,
            "{malformed:?} is not a recorded identity",
        );
    }
}
