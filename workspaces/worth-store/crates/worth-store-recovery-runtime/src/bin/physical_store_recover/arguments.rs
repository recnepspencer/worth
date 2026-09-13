use std::path::PathBuf;

#[path = "arguments/options.rs"]
mod options;

const PHASE_TWO_PROFILE: &str = "c8-phase2-admission-v1";
const FATE_COVERAGE_PROFILE: &str = "c8-phase8-fate-coverage-v1";
const REFUSED_PROFILE: &str = "c8-phase8-refused-v1";
const PUBLICATION_INDETERMINATE_PROFILE: &str = "c8-phase8-publication-indeterminate-v1";

#[derive(Clone, Copy)]
pub(super) enum BoundedProfile {
    PhaseTwoAdmission,
    FateCoverage,
    Refused,
    PublicationIndeterminate,
}

pub(super) struct Invocation {
    pub(super) root: PathBuf,
    pub(super) profile: BoundedProfile,
    pub(super) report_path: Option<PathBuf>,
    pub(super) yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
}

pub(super) fn parse(arguments: Vec<std::ffi::OsString>) -> Result<Invocation, String> {
    let [root, profile, remaining @ ..] = arguments.as_slice() else {
        return Err(format!(
            "usage: physical_store_recover <store-root> --bounded-profile={PHASE_TWO_PROFILE}|{FATE_COVERAGE_PROFILE}|{REFUSED_PROFILE}|{PUBLICATION_INDETERMINATE_PROFILE} \
             [--report=<path>] \
             [--yieldpoint-stage=<stage> --yieldpoint-reached=<path> --yieldpoint-release=<path> --yieldpoint-cancel=<path> --yieldpoint-deadline-ms=<milliseconds>]"
        ));
    };
    let profile = match profile.to_string_lossy().as_ref() {
        value if value == format!("--bounded-profile={PHASE_TWO_PROFILE}") => {
            BoundedProfile::PhaseTwoAdmission
        }
        value if value == format!("--bounded-profile={FATE_COVERAGE_PROFILE}") => {
            BoundedProfile::FateCoverage
        }
        value if value == format!("--bounded-profile={REFUSED_PROFILE}") => {
            BoundedProfile::Refused
        }
        value if value == format!("--bounded-profile={PUBLICATION_INDETERMINATE_PROFILE}") => {
            BoundedProfile::PublicationIndeterminate
        }
        _ => {
            return Err(format!(
            "unsupported bounded profile; expected {PHASE_TWO_PROFILE}, {FATE_COVERAGE_PROFILE}, {REFUSED_PROFILE}, or {PUBLICATION_INDETERMINATE_PROFILE}"
        ))
        }
    };
    let root = PathBuf::from(root);
    let (report_path, yieldpoint) = options::parse(&root, remaining)?;
    Ok(Invocation {
        root,
        profile,
        report_path,
        yieldpoint,
    })
}
