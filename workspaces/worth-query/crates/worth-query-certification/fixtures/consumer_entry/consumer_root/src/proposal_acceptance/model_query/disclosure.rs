//! Model and target projections require the installed domain disclosure capability.
use super::super::access;
use super::*;

pub(super) fn model() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        "proposal-model",
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .disclose_field_by(model_name(), access::encoded(1), influence)
}
pub(super) fn target() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        "proposal-target",
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .use_field_by(
        TargetName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .use_field_by(
        Retention::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .disclose_field_by(target_name(), access::encoded(1), influence.clone())
    .disclose_field_by(days(), access::encoded(1), influence)
}
