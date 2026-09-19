use super::{
    UiNativeApplicationPreparation, UiNativeApplicationPreparationDenialCause,
    UiNativeApplicationPreparationOutcome,
};

struct OneOwnerRuntime;

impl crate::native_platform::UiNativeApplicationRuntime for OneOwnerRuntime {
    fn readiness_owner_count(
        &self,
    ) -> crate::native_platform::UiNativeApplicationReadinessOwnerCount {
        crate::native_platform::UiNativeApplicationReadinessOwnerCount::new(1)
            .expect("one application readiness owner is valid")
    }

    fn activate(
        &mut self,
        _application: crate::facade::WorthUiNativeApplicationShell,
        _readiness: Box<[crate::native_platform::UiNativeApplicationReadinessPort]>,
    ) -> Result<
        crate::facade::WorthUiNativeApplicationShell,
        crate::native_platform::UiNativeApplicationRuntimeActivationStopped,
    > {
        unreachable!("preparation must not activate the application runtime")
    }

    fn native_pointer_affordance_ready(
        &mut self,
        application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            crate::native_platform::UiNativeApplicationRuntimeDirective,
        ),
        super::super::UiNativeApplicationRuntimeProgressStopped,
    > {
        Ok((
            application,
            crate::native_platform::UiNativeApplicationRuntimeDirective::Continue,
        ))
    }

    fn readiness_ready(
        &mut self,
        _application: crate::facade::WorthUiNativeApplicationShell,
        _owner_ordinal: u8,
        _generation: u64,
    ) -> Result<
        (
            crate::facade::WorthUiNativeApplicationShell,
            crate::native_platform::UiNativeApplicationRuntimeDirective,
        ),
        crate::native_platform::UiNativeApplicationRuntimeProgressStopped,
    > {
        unreachable!("preparation must not progress the application runtime")
    }

    fn close(
        self: Box<Self>,
        _application: crate::facade::WorthUiNativeApplicationShell,
    ) -> Result<
        crate::native_platform::UiNativeApplicationRuntimeClosed,
        crate::native_platform::UiNativeApplicationRuntimeCloseIncomplete,
    > {
        unreachable!("preparation must not close the application runtime")
    }
}

fn preparation(identity: u64) -> UiNativeApplicationPreparation {
    let binding =
        crate::native_platform::native_platform_binding::UiNativePlatformBindingGrant::issue(
            identity,
        );
    UiNativeApplicationPreparation::new(identity, binding)
}

fn install_change_profile(preparation: &mut UiNativeApplicationPreparation) {
    preparation
        .builder()
        .with_change_profile(crate::facade::rebind::UiChangeProfile::platform_pulse())
        .expect("the first change profile must install");
}

#[test]
fn complete_preparation_freezes_only_a_host_neutral_application() {
    let mut preparation = preparation(41);
    install_change_profile(&mut preparation);
    assert!(matches!(
        preparation.complete(),
        UiNativeApplicationPreparationOutcome::Prepared(_)
    ));
}

#[test]
fn preparation_builder_rejects_replacement_and_preserves_affine_slot() {
    let mut preparation = preparation(42);
    let mut builder = preparation.builder();
    builder
        .with_change_profile(crate::facade::rebind::UiChangeProfile::platform_pulse())
        .expect("the first change profile must install");
    assert_eq!(
        builder.with_change_profile(crate::facade::rebind::UiChangeProfile::platform_pulse()),
        Err(UiNativeApplicationPreparationDenialCause::ChangeProfileAlreadyInstalled)
    );
    drop(builder);
    assert!(matches!(
        preparation.complete(),
        UiNativeApplicationPreparationOutcome::Prepared(_)
    ));
}

#[test]
fn application_runtime_is_affine_and_remains_inert_during_preparation() {
    let mut preparation = preparation(43);
    preparation
        .install_application_runtime(OneOwnerRuntime)
        .expect("the first application runtime must install");
    assert_eq!(
        preparation.install_application_runtime(OneOwnerRuntime),
        Err(UiNativeApplicationPreparationDenialCause::ApplicationRuntimeAlreadyInstalled)
    );
    install_change_profile(&mut preparation);

    let UiNativeApplicationPreparationOutcome::Prepared(prepared) = preparation.complete() else {
        panic!("the configured application should prepare")
    };
    assert_eq!(
        prepared
            .application_runtime
            .as_ref()
            .expect("prepared application retains the runtime")
            .readiness_owner_count()
            .get(),
        1
    );
}

#[test]
fn native_surface_declaration_is_explicit_affine_launch_input() {
    let mut preparation = preparation(44);
    preparation
        .install_native_surface_declaration("workspace.surface.main")
        .expect("the application selects its authored native surface");
    assert_eq!(
        preparation.install_native_surface_declaration("workspace.surface.secondary"),
        Err(UiNativeApplicationPreparationDenialCause::NativeSurfaceDeclarationAlreadyInstalled)
    );
    install_change_profile(&mut preparation);
    let UiNativeApplicationPreparationOutcome::Prepared(prepared) = preparation.complete() else {
        panic!("the configured application should prepare")
    };
    assert_eq!(
        prepared.native_surface_declaration.as_deref(),
        Some("workspace.surface.main")
    );
}
