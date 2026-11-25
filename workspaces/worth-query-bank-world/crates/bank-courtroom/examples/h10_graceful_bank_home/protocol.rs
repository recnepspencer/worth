//! Process orchestration transports only descriptive DATA and passive observations.
use std::{
    net::SocketAddr,
    num::NonZeroU64,
    path::{Path, PathBuf},
};

pub(super) enum Stage {
    OpenCloseProbe,
    Preclose,
    PrekillBeforeCheckpoint,
    PrekillAfterCheckpoint,
    Reopen,
}
impl Stage {
    pub fn name(&self) -> &str {
        match self {
            Self::OpenCloseProbe => "probe-open-close",
            Self::Preclose => "preclose",
            Self::PrekillBeforeCheckpoint => "prekill-precheckpoint",
            Self::PrekillAfterCheckpoint => "prekill-postcheckpoint",
            Self::Reopen => "reopen",
        }
    }
}
pub(super) struct Arguments {
    pub stage: Stage,
    pub directory: PathBuf,
    pub recipe: String,
    pub rail: SocketAddr,
    pub control: SocketAddr,
    pub handoff: PathBuf,
    pub step_cutoff: Option<u8>,
}
impl Arguments {
    pub fn step_completed(&self, step: u8, name: &str) -> bool {
        eprintln!("BANK_WORKLOAD_STEP {step} {name}");
        self.step_cutoff == Some(step)
    }
    pub fn read() -> Self {
        let mut values = std::env::args_os().skip(1);
        let stage = match values
            .next()
            .and_then(|value| value.into_string().ok())
            .as_deref()
        {
            Some("probe-open-close") => Stage::OpenCloseProbe,
            Some("preclose") => Stage::Preclose,
            Some("prekill-precheckpoint") => Stage::PrekillBeforeCheckpoint,
            Some("prekill-postcheckpoint") => Stage::PrekillAfterCheckpoint,
            Some("reopen") => Stage::Reopen,
            _ => panic!(
                "expected probe-open-close, preclose, prekill-precheckpoint, prekill-postcheckpoint or reopen stage"
            ),
        };
        let directory = values.next().expect("Home directory").into();
        let recipe = values
            .next()
            .expect("bootstrap recipe")
            .into_string()
            .expect("UTF-8 recipe");
        let rail = values
            .next()
            .expect("rail address")
            .into_string()
            .unwrap()
            .parse()
            .unwrap();
        let control = values
            .next()
            .expect("rail control address")
            .into_string()
            .unwrap()
            .parse()
            .unwrap();
        let handoff = values.next().expect("descriptive handoff file").into();
        let step_cutoff = values.next().map(|value| {
            let step = value
                .into_string()
                .unwrap()
                .parse::<u8>()
                .expect("numeric step cutoff");
            assert!((1..=8).contains(&step), "step cutoff is 1 through 8");
            assert!(
                matches!(
                    &stage,
                    Stage::Preclose
                        | Stage::PrekillBeforeCheckpoint
                        | Stage::PrekillAfterCheckpoint
                        | Stage::Reopen
                ),
                "cutoff is a workload probe"
            );
            step
        });
        assert!(values.next().is_none(), "unexpected stage argument");
        Self {
            stage,
            directory,
            recipe,
            rail,
            control,
            handoff,
            step_cutoff,
        }
    }
}

/// Platform descriptors use their owning codecs. Native obligation coordinates
/// use the existing public DATA identity and are checked by real Home readmission.
/// No installed contract, issued workflow reference, snapshot or principal crosses.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Handoff {
    pub preclose_pid: u32,
    pub killed_without_close: bool,
    pub prekill_checkpoint_coverage: Vec<(String, u64)>,
    pub workflow: Vec<u8>,
    pub required: Vec<u8>,
    pub assessment: Vec<u8>,
    pub rejection_home: worth_foundational::facade::HomeIdentity,
    pub rejection_position: NonZeroU64,
    pub pending: super::external::Pending,
    pub reversals: super::correction::Reversals,
}
impl Handoff {
    pub fn write(&self, path: &Path) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .expect("create one graceful DATA handoff after Home close");
        serde_json::to_writer(&file, self).expect("encode descriptive process DATA");
        file.sync_all().expect("finish descriptive process DATA");
    }
    pub fn read(path: &Path) -> Self {
        serde_json::from_reader(std::fs::File::open(path).expect("preclose DATA handoff"))
            .expect("decode descriptive process DATA; this does not readmit any owner")
    }
}
