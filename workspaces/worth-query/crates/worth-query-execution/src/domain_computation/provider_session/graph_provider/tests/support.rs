use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};
use worth_query_installation::facade::{
    WorthQueryInstallationGeneration, WorthQueryInstalledGraphParticipationAuthority,
};

use super::super::{
    WorthQueryExecutionGraphReadStreamEvidence, WorthQueryGraphProviderCall,
    WorthQueryGraphProviderCallKind, WorthQueryGraphProviderCallRequest,
    WorthQueryGraphProviderFailure, WorthQueryGraphProviderReceipt, WorthQueryGraphReadMaterial,
    WorthQueryGraphReadRow, WorthQueryGraphReadStreamAccumulator, WorthQueryProviderWorkReport,
};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::operation_binding::direct_authority_with_graph;
use crate::domain_computation::provider_session::tests::admitted_plan;
use crate::domain_computation::provider_session::WorthQueryDirectExecutionResourceAttempt;

pub(in crate::domain_computation::provider_session::graph_provider) struct GraphAttempt {
    pub(in crate::domain_computation::provider_session::graph_provider) attempt:
        WorthQueryDirectExecutionResourceAttempt,
    pub(in crate::domain_computation::provider_session::graph_provider) graph:
        WorthQueryInstalledGraphParticipationAuthority,
    pub(in crate::domain_computation::provider_session::graph_provider) foreign_graph:
        WorthQueryInstalledGraphParticipationAuthority,
}

pub(super) trait StreamedReceiptForTest {
    fn streamed_for_test(
        &self,
        provider_receipt: &str,
        material: WorthQueryGraphReadMaterial,
        work_report: WorthQueryProviderWorkReport,
    ) -> Result<WorthQueryGraphProviderReceipt, WorthQueryGraphProviderFailure>;
}

impl StreamedReceiptForTest for WorthQueryGraphProviderCall {
    fn streamed_for_test(
        &self,
        provider_receipt: &str,
        material: WorthQueryGraphReadMaterial,
        work_report: WorthQueryProviderWorkReport,
    ) -> Result<WorthQueryGraphProviderReceipt, WorthQueryGraphProviderFailure> {
        let mut stream = WorthQueryGraphReadStreamAccumulator::new(self);
        stream.admit_chunk(material);
        let stream: WorthQueryExecutionGraphReadStreamEvidence = stream.finish(self);
        self.streamed(provider_receipt, stream, work_report)
    }
}

pub(super) fn projected_work_report() -> WorthQueryProviderWorkReport {
    WorthQueryProviderWorkReport::new(1, 0, 64, 64)
}

pub(in crate::domain_computation::provider_session::graph_provider) fn call(
    attempt: &GraphAttempt,
    scope: &str,
) -> WorthQueryGraphProviderCall {
    call_with_kind(attempt, scope, WorthQueryGraphProviderCallKind::Project)
}

pub(super) fn call_with_kind(
    attempt: &GraphAttempt,
    scope: &str,
    kind: WorthQueryGraphProviderCallKind,
) -> WorthQueryGraphProviderCall {
    attempt
        .attempt
        .provider_session_for_test()
        .bind_graph_provider_call(
            &attempt.graph,
            call_spec(scope, kind),
            attempt.attempt.evidence(),
            attempt.attempt.resources().shared_envelope(),
        )
        .unwrap()
}

pub(super) fn call_spec(
    scope: &str,
    kind: WorthQueryGraphProviderCallKind,
) -> WorthQueryGraphProviderCallRequest {
    WorthQueryGraphProviderCallRequest::direct(kind, scope)
        .with_managed_execution_snapshot("snapshot")
}

pub(super) fn material(label: &str) -> WorthQueryGraphReadMaterial {
    material_with_rows([label])
}

pub(super) fn material_with_rows<const N: usize>(labels: [&str; N]) -> WorthQueryGraphReadMaterial {
    WorthQueryGraphReadMaterial::new(labels.into_iter().map(|label| {
        let field = CanonicalFieldPath::single(FieldKey::new("id").unwrap());
        let values = BTreeMap::from([(field, AspectValue::String(InternedString::from(label)))]);
        WorthQueryGraphReadRow::from_native_fields(label, values).unwrap()
    }))
}

pub(super) fn material_with_field_order(reverse: bool) -> WorthQueryGraphReadMaterial {
    let identity_path = CanonicalFieldPath::single(FieldKey::new("id").unwrap());
    let kind_path = CanonicalFieldPath::single(FieldKey::new("kind").unwrap());
    let mut values = BTreeMap::new();
    let identity = AspectValue::String(InternedString::from("vertex-a"));
    let kind = AspectValue::String(InternedString::from("vertex"));
    if reverse {
        values.insert(kind_path, kind);
        values.insert(identity_path, identity);
    } else {
        values.insert(identity_path, identity);
        values.insert(kind_path, kind);
    }
    WorthQueryGraphReadMaterial::new([WorthQueryGraphReadRow::from_native_fields(
        "vertex-a", values,
    )
    .unwrap()])
}

pub(super) fn material_with_identity_value(value: &str) -> WorthQueryGraphReadMaterial {
    let identity_path = CanonicalFieldPath::single(FieldKey::new("id").unwrap());
    let values = BTreeMap::from([(
        identity_path,
        AspectValue::String(InternedString::from(value)),
    )]);
    WorthQueryGraphReadMaterial::new([WorthQueryGraphReadRow::from_native_fields(
        "stable-entity",
        values,
    )
    .unwrap()])
}

pub(in crate::domain_computation::provider_session::graph_provider) fn attempt() -> GraphAttempt {
    attempt_with_access(worth_query_installation::facade::WorthQueryOperationGraphAccess::Project)
}

pub(super) fn attempt_with_access(
    access: worth_query_installation::facade::WorthQueryOperationGraphAccess,
) -> GraphAttempt {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "remote",
        "test-graph-provider",
        false,
        Option::<String>::None,
        std::sync::Arc::new(()),
    )
    .unwrap();
    let foreign_graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "foreign",
        "foreign-test-graph-provider",
        false,
        Option::<String>::None,
        std::sync::Arc::new(()),
    )
    .unwrap();
    let runtime = installer
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0;
    let resources = admitted_plan("binding", 8);
    let authority = direct_authority_with_graph(&runtime, &resources, &graph, access);
    let reserved =
        worth_query_admission::integration::reserve_execution_resource_plan(resources).unwrap();
    let attempt = WorthQueryDirectExecutionResourceAttempt::start(reserved, &authority);
    GraphAttempt {
        attempt,
        graph,
        foreign_graph,
    }
}
