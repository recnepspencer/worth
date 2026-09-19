pub(in crate::mounting::session_state) fn mounted_host_authority<'host>(
    host: &'host crate::facade::WorthUiHostSessionAuthority,
    capability_report: &'host worth_ui_host_contract::WorthUiHostCapabilityReport,
) -> crate::mounting::UiMountedHostPresentationAuthority<'host> {
    crate::mounting::UiMountedHostPresentationAuthority::new(
        host.identity().as_u64(),
        host.protocol(),
        capability_report,
        host.mounted_presentation_lease(),
    )
}
