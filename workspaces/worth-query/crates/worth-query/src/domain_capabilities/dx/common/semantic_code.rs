pub(super) fn qualify_semantic_code(
    authority: &crate::domain_installation::WorthQueryInstalledDomainAuthority,
    semantic_code: &str,
) -> String {
    format!("{}.{}", authority.domain_owner(), semantic_code)
}
