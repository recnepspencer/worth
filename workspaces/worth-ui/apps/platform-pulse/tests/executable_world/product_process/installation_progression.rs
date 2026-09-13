use crate::failure_teardown::{
    report_without_owned_resources, teardown_installed_world, PulseExecutableWorldFailure,
    PulseExecutableWorldFailureReport,
};
use crate::installation::{
    CanonicalPlatformPulse, IsolatedPulseInstallation, PulseInstallationPath,
};

use super::{AwaitingFirstFrame, CargoBuiltPlatformPulse, Installed, PulseExecutableWorld};

impl PulseExecutableWorld<Installed> {
    pub(crate) fn install(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseExecutableWorldFailureReport> {
        let installation = IsolatedPulseInstallation::install(canonical).map_err(|failure| {
            report_without_owned_resources(PulseExecutableWorldFailure::Installation(failure))
        })?;
        Ok(Self {
            state: Installed { installation },
        })
    }

    pub(crate) fn install_at(
        canonical: CanonicalPlatformPulse,
        path: &PulseInstallationPath,
    ) -> Result<Self, PulseExecutableWorldFailureReport> {
        let installation =
            IsolatedPulseInstallation::install_at(canonical, path).map_err(|failure| {
                report_without_owned_resources(PulseExecutableWorldFailure::Installation(failure))
            })?;
        Ok(Self {
            state: Installed { installation },
        })
    }

    pub(crate) fn launch(
        self,
        binary: CargoBuiltPlatformPulse,
    ) -> Result<PulseExecutableWorld<AwaitingFirstFrame>, PulseExecutableWorldFailureReport> {
        let installation = self.state.installation;
        let launch = match binary.launch(
            installation.source_root(),
            installation.source_root(),
            installation.source_root(),
        ) {
            Ok(launch) => launch,
            Err(failure) => {
                return Err(teardown_installed_world(
                    PulseExecutableWorldFailure::Launch(failure),
                    installation,
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: AwaitingFirstFrame {
                installation,
                process: launch.process,
                lifecycle: launch.lifecycle,
                launch_started: launch.launch_started,
            },
        })
    }
}
