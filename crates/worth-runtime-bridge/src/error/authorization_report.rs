use crate::authorization::BridgeAuthorizationDenial;
use crate::authorization::BridgeAuthorizationDenialKind;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl Reportable for BridgeAuthorizationDenial {
    fn denial_class(&self) -> DenialClass {
        report(self).0
    }

    fn disposition(&self) -> Disposition {
        report(self).1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(report(self).2).expect("valid static Bridge report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Bridge authorization was denied."
    }
}

fn report(denial: &BridgeAuthorizationDenial) -> (DenialClass, Disposition, &'static str) {
    match denial.kind() {
        BridgeAuthorizationDenialKind::EmptyPolicy => (
            InvalidInput,
            CorrectInput,
            "bridge.authorization.empty_policy",
        ),
        BridgeAuthorizationDenialKind::MissingRequiredRule => (
            Authorization,
            Reauthorize,
            "bridge.authorization.missing_required_rule",
        ),
        BridgeAuthorizationDenialKind::EmptyRule => (
            InvalidInput,
            CorrectInput,
            "bridge.authorization.empty_rule",
        ),
        BridgeAuthorizationDenialKind::EmptyRequirement => (
            InvalidInput,
            CorrectInput,
            "bridge.authorization.empty_requirement",
        ),
        BridgeAuthorizationDenialKind::DuplicateCorrespondence => (
            Conflict,
            Final,
            "bridge.authorization.duplicate_correspondence",
        ),
        BridgeAuthorizationDenialKind::UnknownCorrespondence => (
            NotFound,
            CorrectInput,
            "bridge.authorization.unknown_correspondence",
        ),
        BridgeAuthorizationDenialKind::ObservationShapeMismatch => (
            InvalidInput,
            CorrectInput,
            "bridge.authorization.observation_shape_mismatch",
        ),
        BridgeAuthorizationDenialKind::SignalInstallationRejected => (
            Authorization,
            Reauthorize,
            "bridge.authorization.signal_installation_rejected",
        ),
        BridgeAuthorizationDenialKind::SignalEvaluationRejected => (
            Authorization,
            Reauthorize,
            "bridge.authorization.signal_evaluation_rejected",
        ),
    }
}
