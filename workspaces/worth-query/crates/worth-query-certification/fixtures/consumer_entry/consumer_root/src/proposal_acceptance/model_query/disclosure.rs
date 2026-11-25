//! Model and target projections require the installed domain disclosure capability.
use super::super::access;
use super::*;

worth_query_decl::facade::worth_query_disclosure_classification!(ProposalModelDisclosure);
worth_query_decl::facade::worth_query_disclosure_classification!(ProposalTargetDisclosure);

pub(super) fn model() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        ProposalModelDisclosure,
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::at("ModelName"),
        access::encoded(1),
        influence.clone(),
    )
    .expect("owned record field matches its declaration")
    .disclose_field_by(model_name(), access::encoded(1), influence)
}
pub(super) fn target() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        ProposalTargetDisclosure,
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::at("ModelName"),
        access::encoded(1),
        influence.clone(),
    )
    .expect("owned record field matches its declaration")
    .use_field_by(
        TargetName::at("TargetName"),
        access::encoded(1),
        influence.clone(),
    )
    .expect("owned record field matches its declaration")
    .use_field_by(
        Retention::at("Retention"),
        access::encoded(1),
        influence.clone(),
    )
    .expect("owned record field matches its declaration")
    .disclose_field_by(target_name(), access::encoded(1), influence.clone())
    .disclose_field_by(days(), access::encoded(1), influence)
}
