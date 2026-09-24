use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeWindowSpec {
    title: Arc<str>,
    initial_logical_size: [u32; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativePlatformProfile {
    window: UiNativeWindowSpec,
    #[cfg(feature = "certification-support")]
    qualification: Option<worth_ui_host_native::UiNativeQualificationPlan>,
    driver_qualification: UiNativeDriverQualification,
}

/// The derived-state loss a certification profile asks the driver to inject.
///
/// Ordinary builds carry no plan, so the driver and its progression keep one
/// shape whatever features are enabled; only the progress owner reads it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeDriverQualification {
    #[cfg(feature = "certification-support")]
    plan: Option<super::runtime_qualification::UiNativeRuntimeQualificationPlan>,
}

impl UiNativeDriverQualification {
    pub(crate) const fn ordinary() -> Self {
        Self {
            #[cfg(feature = "certification-support")]
            plan: None,
        }
    }

    #[cfg(feature = "certification-support")]
    pub(crate) const fn plan(
        self,
    ) -> Option<super::runtime_qualification::UiNativeRuntimeQualificationPlan> {
        self.plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePlatformPreparationDenial {
    EmptyWindowTitle,
    WindowTitleCapacityExceeded,
    EmptyWindowExtent,
    WindowExtentCapacityExceeded,
    QualifiedProfileMismatch,
    PreparationIdentityExhausted,
    UnsupportedPlatform,
    UnsupportedArchitecture,
}

impl UiNativeWindowSpec {
    pub fn new(title: impl Into<Arc<str>>, initial_logical_size: [u32; 2]) -> Self {
        Self {
            title: title.into(),
            initial_logical_size,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn initial_logical_size(&self) -> [u32; 2] {
        self.initial_logical_size
    }
}

impl UiNativePlatformProfile {
    pub fn single_window(window: UiNativeWindowSpec) -> Self {
        Self {
            window,
            #[cfg(feature = "certification-support")]
            qualification: None,
            driver_qualification: UiNativeDriverQualification::ordinary(),
        }
    }

    #[cfg(feature = "certification-support")]
    pub fn with_native_qualification_plan(
        mut self,
        plan: worth_ui_host_native::UiNativeQualificationPlan,
    ) -> Self {
        self.qualification = Some(plan);
        self
    }

    #[cfg(feature = "certification-support")]
    pub fn with_runtime_qualification_plan(
        mut self,
        plan: super::runtime_qualification::UiNativeRuntimeQualificationPlan,
    ) -> Self {
        self.driver_qualification = UiNativeDriverQualification { plan: Some(plan) };
        self
    }

    pub fn window(&self) -> &UiNativeWindowSpec {
        &self.window
    }

    pub(crate) fn prepare_native_host(&self) -> worth_ui_host_native::WorthUiPreparedNativeHost {
        #[cfg(feature = "certification-support")]
        if let Some(plan) = self.qualification {
            return worth_ui_host_native::WorthUiPreparedNativeHost::prepare_qualified_for_certification(
                plan,
            );
        }
        worth_ui_host_native::WorthUiPreparedNativeHost::prepare_qualified()
    }

    pub(crate) const fn driver_qualification(&self) -> UiNativeDriverQualification {
        self.driver_qualification
    }

    pub(crate) fn validate(&self) -> Result<(), UiNativePlatformPreparationDenial> {
        validate_environment(worth_ui_host_native::UiNativeQualifiedTarget::RESOLVED)?;
        if self.window.title.is_empty() {
            return Err(UiNativePlatformPreparationDenial::EmptyWindowTitle);
        }
        if self.window.title.len() > 256 {
            return Err(UiNativePlatformPreparationDenial::WindowTitleCapacityExceeded);
        }
        let [width, height] = self.window.initial_logical_size;
        if width == 0 || height == 0 {
            return Err(UiNativePlatformPreparationDenial::EmptyWindowExtent);
        }
        if width > 16_384 || height > 16_384 {
            return Err(UiNativePlatformPreparationDenial::WindowExtentCapacityExceeded);
        }
        Ok(())
    }
}

/// Classifies an already-resolved qualification verdict into a preparation
/// denial.
///
/// The verdict arrives resolved so this stays a closed classifier with no
/// lookups of its own. A qualified target naming a profile other than the one
/// this build compiled is a mismatch, not a pass: the runtime binds exactly the
/// profile the host-native crate selected.
fn validate_environment(
    target: worth_ui_host_native::UiNativeQualifiedTarget,
) -> Result<(), UiNativePlatformPreparationDenial> {
    match target {
        worth_ui_host_native::UiNativeQualifiedTarget::Qualified(identity)
            if identity == worth_ui_host_native::WORTH_UI_NATIVE_PROFILE_IDENTITY =>
        {
            Ok(())
        }
        worth_ui_host_native::UiNativeQualifiedTarget::Qualified(_) => {
            Err(UiNativePlatformPreparationDenial::QualifiedProfileMismatch)
        }
        worth_ui_host_native::UiNativeQualifiedTarget::UnqualifiedOperatingSystem => {
            Err(UiNativePlatformPreparationDenial::UnsupportedPlatform)
        }
        worth_ui_host_native::UiNativeQualifiedTarget::UnqualifiedArchitecture => {
            Err(UiNativePlatformPreparationDenial::UnsupportedArchitecture)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_environment, UiNativePlatformPreparationDenial};

    #[test]
    fn closed_environment_classifier_rejects_each_platform_substitution() {
        use worth_ui_host_native::{
            UiNativeQualifiedTarget, WORTH_UI_NATIVE_PROFILE_IDENTITY,
            WORTH_UI_QUALIFIED_PROFILE_IDENTITIES,
        };
        assert_eq!(
            validate_environment(UiNativeQualifiedTarget::UnqualifiedOperatingSystem),
            Err(UiNativePlatformPreparationDenial::UnsupportedPlatform)
        );
        assert_eq!(
            validate_environment(UiNativeQualifiedTarget::UnqualifiedArchitecture),
            Err(UiNativePlatformPreparationDenial::UnsupportedArchitecture)
        );
        // Every qualified identity is admitted on exactly one build: its own.
        // The list is the host crate's own, derived from its qualified-profile
        // array, so a profile added there is asserted here without this test
        // changing; and exactly one iteration must take the admitted arm, or
        // the positive branch of the classifier is unexercised on this build.
        let mut admitted = 0;
        for identity in WORTH_UI_QUALIFIED_PROFILE_IDENTITIES {
            let expected = if identity == WORTH_UI_NATIVE_PROFILE_IDENTITY {
                admitted += 1;
                Ok(())
            } else {
                Err(UiNativePlatformPreparationDenial::QualifiedProfileMismatch)
            };
            assert_eq!(
                validate_environment(UiNativeQualifiedTarget::Qualified(identity)),
                expected,
                "{}",
                identity.as_str()
            );
        }
        assert_eq!(
            admitted, 1,
            "the active identity must be one of the qualified identities, exactly once"
        );
    }
}
