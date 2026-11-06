use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenialKind;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionRefusalKind;
use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;

pub(super) fn principal_binding_resolution_denial(
    kind: WorthQueryPrincipalBindingInstallationDenialKind,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let resolution_kind = match kind {
        WorthQueryPrincipalBindingInstallationDenialKind::ForeignRuntime => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime)
        }
        WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StaleInstalledSchema)
        }
        WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled
        | WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::SchemaMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::BindingNotInstalled)
        }
    };
    resolution_denial(resolution_kind, binding)
}

pub(super) fn entity_lookup_resolution_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let resolution_kind = match kind {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => {
            return relational_read_denial(cause, binding)
        }
        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof)
        }
        BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
        | BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit => {
            WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::IdentityIndexUnavailable,
            )
        }
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::CorruptIdentityIndex)
        }
    };
    resolution_denial(resolution_kind, binding)
}

pub(super) fn principal_index_currency_denial(
    denial: crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let kind = match denial {
        crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => {
            let stop = crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(stop, None);
            let kind = match stop {
                Ok(kind) => WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(kind)),
                Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
                ) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
                )),
                Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
                ) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
                )),
            };
            return resolution_denial(kind, binding);
        }

        crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::BranchMaterializationSuspended,
        ),
        crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(_)
        | crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable
            | crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted
            | crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted
            | crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::ForeignRuntime { .. }
            | crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted { .. }
            | crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::IdentityIndexUnavailable,
        ),
        crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::BranchIdentityUnavailable,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::IdentityIndexUnavailable,
        ),
    };
    resolution_denial(kind, binding)
}

pub(super) fn resolution_denial(
    kind: WorthQueryPrincipalResolutionDenialKind,
    binding: impl Into<String>,
) -> WorthQueryPrincipalResolutionDenial {
    WorthQueryPrincipalResolutionDenial::new(kind, binding)
}

pub fn query_read_resolution_denial(
    stop: worth_relational::facade::runtime::QueryReadExecutionStop,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
    use WorthQueryPrincipalResolutionDenialKind as Kind;
    let kind = match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::query_read_execution_kind(&stop)
    {
        Ok(Some(kind)) => Kind::Stopped(Ok(kind)),
        Ok(None) => Kind::Refused(PrincipalRefusal::StalePrincipalProof),
        Err(Control::Cancelled) => Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Err(Control::TimedOut) => Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
    };
    WorthQueryPrincipalResolutionDenial {
        kind,
        binding: binding.to_owned(),
        native_query_read_stop: Some(std::sync::Arc::new(stop)),
    }
}

pub(super) fn relational_read_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(
        cause, None,
    ) {
        Ok(kind) => WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(kind)),
        Err(Control::Cancelled) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Err(Control::TimedOut) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
    };
    resolution_denial(kind, binding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspended_materialization_is_not_reported_as_an_index_outage() {
        let denial = principal_index_currency_denial(
            crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
                crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
            ),
            "principal",
        );

        assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::BranchMaterializationSuspended
            )
        );
    }
}
