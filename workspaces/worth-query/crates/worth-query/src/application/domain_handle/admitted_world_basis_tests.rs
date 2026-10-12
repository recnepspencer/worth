use super::{WorthQueryAdmittedWorldBasis, WorthQueryDomainOperatingContext};
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDomainEntryMarker,
};
use crate::runtime::{
    runtime_state_snapshot_basis_label_identity,
    runtime_state_snapshot_result_shape_label_identity, WorthQueryRuntimeStateKind,
};

const ENTRY_CAPABILITIES: &[WorthQueryCapabilityFamily] =
    &[WorthQueryCapabilityFamily::QueryComposition];
const WORLD_CAPABILITIES: &[WorthQueryCapabilityFamily] =
    &[WorthQueryCapabilityFamily::HistoricalEvaluation];
const WORLD_SECTIONS: &[WorthQueryConfigSectionFamily] = &[
    WorthQueryConfigSectionFamily::Query,
    WorthQueryConfigSectionFamily::Relational,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryDomainEntry;

impl WorthQueryDomainEntryMarker for GeometryDomainEntry {
    fn domain_key(&self) -> &'static str {
        "test.geometry.world-basis"
    }

    fn display_name(&self) -> &'static str {
        "GeometryWorldBasisDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        ENTRY_CAPABILITIES
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryWorld(&'static str);

impl WorthQueryDomainOperatingContext<GeometryDomainEntry> for GeometryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        WORLD_CAPABILITIES
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        WORLD_SECTIONS
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = format!("geometry.world-basis.{}", self.0);
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StructuredGeometryWorld {
    mode: &'static str,
    tenant: &'static str,
    reverse_declaration_order: bool,
}

impl WorthQueryDomainOperatingContext<GeometryDomainEntry> for StructuredGeometryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        WORLD_CAPABILITIES
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        WORLD_SECTIONS
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let fields = if self.reverse_declaration_order {
            [("tenant", self.tenant), ("mode", self.mode)]
        } else {
            [("mode", self.mode), ("tenant", self.tenant)]
        };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::from_fields(fields)
            .expect("static operating-context identity fields should be valid")
    }
}

fn admitted_world_basis(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> WorthQueryAdmittedWorldBasis {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomainEntry,
        GeometryWorld(regime),
        [],
    )
    .retained_world_basis()
}

#[test]
fn retained_world_basis_matches_admitted_handle_identity_and_support() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomainEntry,
        GeometryWorld("collaborative"),
        [],
    );
    let basis = handle.retained_world_basis();
    let support = crate::basis_lifecycle::basis_lifecycle_support_matrix();

    assert_eq!(basis.domain_key(), handle.domain_key());
    assert_eq!(basis.display_name(), handle.display_name());
    assert_eq!(
        basis.operating_context_identity_digest(),
        handle.operating_context_identity_digest()
    );
    assert_eq!(basis.handle_identity(), handle.handle_identity());
    assert_eq!(
        basis.support_snapshot_digest(),
        handle.support_snapshot().snapshot_digest()
    );
    assert_eq!(
        basis.basis_lifecycle_support_identity(),
        &super::compose_basis_lifecycle_support_identity(support.matrix_digest())
    );
}

#[test]
fn equivalent_packages_mint_distinct_retained_world_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_world_basis(request_execution, "collaborative");
    let right = admitted_world_basis(request_execution, "collaborative");

    assert_eq!(left.domain_key(), right.domain_key());
    assert_eq!(
        left.support_snapshot_digest(),
        right.support_snapshot_digest()
    );
    assert_ne!(left.handle_identity(), right.handle_identity());
}

#[test]
fn changing_operating_context_changes_world_identity_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let collaborative = admitted_world_basis(request_execution, "collaborative");
    let restricted = admitted_world_basis(request_execution, "restricted");

    assert_ne!(
        collaborative.operating_context_identity_digest(),
        restricted.operating_context_identity_digest()
    );
    assert_ne!(
        collaborative.handle_identity(),
        restricted.handle_identity()
    );
}

#[test]
fn query_seals_structured_context_identity_independently_of_field_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomainEntry,
        StructuredGeometryWorld {
            mode: "strict",
            tenant: "alpha",
            reverse_declaration_order: false,
        },
        [],
    );
    let reordered = crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomainEntry,
        StructuredGeometryWorld {
            mode: "strict",
            tenant: "alpha",
            reverse_declaration_order: true,
        },
        [],
    );
    let changed = crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomainEntry,
        StructuredGeometryWorld {
            mode: "relaxed",
            tenant: "alpha",
            reverse_declaration_order: true,
        },
        [],
    );

    assert_eq!(
        left.operating_context_identity_digest(),
        reordered.operating_context_identity_digest()
    );
    assert_ne!(
        left.operating_context_identity_digest(),
        changed.operating_context_identity_digest()
    );
}

#[test]
fn separate_installations_cannot_share_world_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_world_basis(request_execution, "collaborative");
    let right = admitted_world_basis(request_execution, "collaborative");

    assert_eq!(
        left.operating_context_identity_digest(),
        right.operating_context_identity_digest()
    );
    assert_eq!(
        left.support_snapshot_digest(),
        right.support_snapshot_digest()
    );
    assert_ne!(left.handle_identity(), right.handle_identity());
}

#[test]
fn retained_world_basis_projects_into_runtime_state_without_raw_world_reconstruction() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) =
        crate::application::domain_test_support::installed_declaration_workspace(
            request_execution,
            GeometryDomainEntry,
            GeometryWorld("collaborative"),
            [],
        );
    let basis = handle.retained_world_basis();
    let state = workspace
        .state(request_execution, &basis)
        .expect("retained world basis should project into its installed runtime state");

    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(
        state.basis_for_reporting(),
        runtime_state_snapshot_basis_label_identity(basis.basis_lifecycle_support_identity())
            .as_str()
    );
    assert_eq!(
        state.result_shape_for_reporting(),
        runtime_state_snapshot_result_shape_label_identity(basis.handle_identity()).as_str()
    );
}
