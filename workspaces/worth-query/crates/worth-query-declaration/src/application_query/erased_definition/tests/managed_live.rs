//! Managed whole-Parent causes do not weaken ordinary business Live validation.
use super::*;
use crate::application_query::{
    ApplicationQueryManagedIndexMeaning, WorthQueryPortableQueryManagedIndexParts,
};

#[test]
fn managed_live_without_business_cause_still_requires_a_valid_probe_selection() {
    let mut parts = dynamically_owned_parts();
    parts.lanes = ApplicationQueryLaneEligibility::one_shot().with_live();
    assert_eq!(
        validate_portable_application_query_freshly(&parts),
        Err(ApplicationQueryDefinitionDenial::LiveLaneWithoutCauseContract)
    );
    parts.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "actual_installed_index".into(),
            probe_parameter: "probe".into(),
        },
    ));
    assert_eq!(
        validate_portable_application_query_freshly(&parts),
        Err(ApplicationQueryDefinitionDenial::InvalidManagedIndexShape)
    );
    parts.parameters = vec![parameter("probe")];
    assert_eq!(validate_portable_application_query_freshly(&parts), Ok(()));
    // Declaration permits the lane. Installation and the actual Native-backed
    // opening still decide the supported matching profile and READ admission.
    parts.managed_index = None;
    assert_eq!(
        validate_portable_application_query_freshly(&parts),
        Err(ApplicationQueryDefinitionDenial::LiveLaneWithoutCauseContract)
    );
}
