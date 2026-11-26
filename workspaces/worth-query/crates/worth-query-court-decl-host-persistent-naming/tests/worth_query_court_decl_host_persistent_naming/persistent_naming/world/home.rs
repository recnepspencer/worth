//! The installed naming world on a real Directory Home: Store persistence,
//! manual checkpoints, a crash (the Home dropped while still owned, so only
//! what reached the Journal or a checkpoint survives) and a reopen of the
//! same directory by a brand-new Home, runtime and installation.
use super::*;
use std::path::{Path, PathBuf};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::WorthQueryApplicationProgramRoster, WorthQueryApplicationHome,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

pub type Home = WorthQueryApplicationHome<Schema, Program>;

/// The build identity every installation of this schema lineage runs under.
pub const BUILD: &str = "91820000-0000-0000-0000-00000000a71e";

/// A fresh, unique Home directory.
pub fn location() -> PathBuf {
    worth_query_testkit_host::facade::host::application_home::location()
}

fn request(location: &Path) -> WorthQueryApplicationHomeRequest<'static, Schema, Program> {
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(BUILD).unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| seed(graph, installed, true),
    )
}

/// Opens the Home at `location`: a new directory is seeded once; an
/// existing one is readmitted from its checkpoint and Journal tail.
pub fn open(location: &Path) -> Home {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    with_bootstrap_advancement_from_request(execution, |phase| {
        request(location).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("Directory Home at {location:?} must open: {refused:?}"))
}

/// Captures a checkpoint now, so a later reopen restores it plus the tail.
pub fn checkpoint(home: &Home) {
    let lease = checkpoint_capture::lease(256 << 20);
    home.checkpoint(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap_or_else(|refused| panic!("checkpoint must settle: {refused:?}"));
}

/// The installed runtime a test drives: in memory, or on a Directory Home.
pub enum Runtime {
    Memory(Host),
    Directory {
        home: Option<Home>,
        location: PathBuf,
    },
}

impl Runtime {
    pub fn directory(location: PathBuf) -> Self {
        Self::Directory {
            home: Some(open(&location)),
            location,
        }
    }

    /// Crashes the Home (drops it still owned, no close or checkpoint) and
    /// reopens the directory as a fresh installation sharing nothing live.
    pub fn crash_and_reopen(&mut self) {
        let Self::Directory { home, location } = self else {
            panic!("only a Directory Home reopens");
        };
        drop(home.take());
        *home = Some(open(location));
    }

    pub fn checkpoint(&self) {
        let Self::Directory {
            home: Some(home), ..
        } = self
        else {
            panic!("only an open Directory Home checkpoints");
        };
        checkpoint(home);
    }
}

impl std::ops::Deref for Runtime {
    type Target = Host;
    fn deref(&self) -> &Host {
        match self {
            Self::Memory(host) => host,
            Self::Directory { home, .. } => home.as_ref().expect("the Home is open").runtime(),
        }
    }
}
