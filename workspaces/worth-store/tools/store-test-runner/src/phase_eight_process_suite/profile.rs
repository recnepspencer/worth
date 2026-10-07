#[derive(Clone, Copy)]
pub(crate) enum ProcessSuiteProfile {
    Iteration,
    Ci,
}

impl ProcessSuiteProfile {
    pub(super) const fn cargo_profile(self) -> &'static str {
        match self {
            Self::Iteration => "dev",
            Self::Ci => "ci-test",
        }
    }

    pub(super) const fn directory(self) -> &'static str {
        match self {
            Self::Iteration => "debug",
            Self::Ci => "ci-test",
        }
    }

    pub(super) fn nextest_arguments(self, mut arguments: Vec<String>) -> Vec<String> {
        if matches!(self, Self::Ci) {
            arguments.splice(
                2..2,
                ["--profile", "ci", "--cargo-profile", "ci-test"].map(str::to_owned),
            );
        }
        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessSuiteProfile;
    use crate::plan::FocusSelection;
    use crate::product::FocusGroup;

    #[test]
    fn process_profiles_keep_support_binaries_and_nextest_in_the_same_build_lane() {
        let selection = FocusSelection::for_group(FocusGroup::PhaseCheckpoint);
        let ci = ProcessSuiteProfile::Ci;
        let arguments = ci.nextest_arguments(selection.nextest_arguments());
        assert_eq!(ci.cargo_profile(), "ci-test");
        assert_eq!(ci.directory(), "ci-test");
        assert!(arguments.windows(2).any(|pair| pair == ["--profile", "ci"]));
        assert!(arguments
            .windows(2)
            .any(|pair| pair == ["--cargo-profile", "ci-test"]));

        let iteration = ProcessSuiteProfile::Iteration;
        assert_eq!(iteration.cargo_profile(), "dev");
        assert_eq!(iteration.directory(), "debug");
        let arguments = iteration.nextest_arguments(selection.nextest_arguments());
        assert!(!arguments
            .iter()
            .any(|argument| argument == "--cargo-profile"));
    }
}
