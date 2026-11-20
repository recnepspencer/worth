use worth_query_installation::facade::WorthQueryInstalledPackageIndexDenialKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionOperationBindingDenial {
    ForeignInstallationAuthority,
    InstalledOperation(WorthQueryInstalledPackageIndexDenialKind),
    ForeignGraphRuntime,
    InstalledGraphTopology,
    RequiredDomain(WorthQueryInstalledPackageIndexDenialKind),
    RequiredDomainTopology,
    InstalledSupportTopology,
}
