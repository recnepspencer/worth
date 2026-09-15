use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use crate::boundary::restore_tokens::{
    discard_restore_token, load_runtime_envelope, store_runtime_envelope, store_snapshot_envelope,
};
use crate::boundary::serde::{from_js, from_json_wire, to_js, to_js_structured, to_json_wire};
use crate::recipe::model::TransactionOp;
use crate::runtime::adapters::{PortableRuntimeEnvelopeArtifact, RuntimeEnvelope};
use crate::runtime::policy::RuntimePolicySpec;
use crate::runtime::summaries::RuntimeSnapshotEnvelope;
use crate::runtime::worker_host::{
    WorkerBrowserHistoryIngress, WorkerHostCapabilityIngressBatch, WorkerHostEffectAcknowledgement,
    WorkerHostEffectRequest, WorkerMainThreadHostedCallbackRequestEnvelope,
    WorkerMainThreadHostedCallbackResult, WorkerOutputDeliveryRequest,
    WorkerPortableGraphPublication, WorkerRuntimeShell,
};

use super::types::SignalWorkerRuntime;

mod branch_history;
mod test_support;
mod test_support_branches;

#[wasm_bindgen]
impl SignalWorkerRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<SignalWorkerRuntime, JsValue> {
        Ok(Self {
            shell: RefCell::new(WorkerRuntimeShell::new(RuntimePolicySpec::default())?),
        })
    }

    #[wasm_bindgen(js_name = bootstrapRecord)]
    pub fn bootstrap_record(&self) -> Result<JsValue, JsValue> {
        to_js(&self.bootstrap_record_for_test()?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = workerRuntimeShellLock)]
    pub fn worker_runtime_shell_lock(&self) -> Result<JsValue, JsValue> {
        to_js(&self.worker_runtime_shell_lock_for_test()?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = publishPortableGraph)]
    pub fn publish_portable_graph(&self, publication: JsValue) -> Result<JsValue, JsValue> {
        let publication: WorkerPortableGraphPublication =
            from_js(publication).map_err(JsValue::from)?;
        let summary = self.publish_portable_graph_for_test(publication)?;
        to_js(&summary).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = applyTransaction)]
    pub fn apply_transaction(&self, transaction_ops: JsValue) -> Result<JsValue, JsValue> {
        let transaction_ops: Vec<TransactionOp> =
            from_js(transaction_ops).map_err(JsValue::from)?;
        let envelope = self.apply_transaction_for_test(transaction_ops)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitHostCapabilityIngress)]
    pub fn admit_host_capability_ingress(&self, batch: JsValue) -> Result<JsValue, JsValue> {
        let batch: WorkerHostCapabilityIngressBatch = from_js(batch).map_err(JsValue::from)?;
        let report = self.admit_host_capability_ingress_for_test(batch)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitBrowserHistoryIngress)]
    pub fn admit_browser_history_ingress(&self, ingress: JsValue) -> Result<JsValue, JsValue> {
        let ingress: WorkerBrowserHistoryIngress = from_js(ingress).map_err(JsValue::from)?;
        let report = self.admit_browser_history_ingress_for_test(ingress)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = issueHostEffectRequest)]
    pub fn issue_host_effect_request(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerHostEffectRequest = from_js(request).map_err(JsValue::from)?;
        let envelope = self.issue_host_effect_request_for_test(request)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitHostEffectAcknowledgement)]
    pub fn admit_host_effect_acknowledgement(
        &self,
        acknowledgement: JsValue,
    ) -> Result<JsValue, JsValue> {
        let acknowledgement: WorkerHostEffectAcknowledgement =
            from_js(acknowledgement).map_err(JsValue::from)?;
        let report = self.admit_host_effect_acknowledgement_for_test(acknowledgement)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyMainThreadHostBridge)]
    pub fn certify_main_thread_host_bridge(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_main_thread_host_bridge_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = issueMainThreadHostedCallbackRequest)]
    pub fn issue_main_thread_hosted_callback_request(
        &self,
        callback_id: String,
    ) -> Result<JsValue, JsValue> {
        let envelope = self.issue_main_thread_hosted_callback_request_for_test(callback_id)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitMainThreadHostedCallbackResult)]
    pub fn admit_main_thread_hosted_callback_result(
        &self,
        request: JsValue,
        result: JsValue,
    ) -> Result<JsValue, JsValue> {
        let request: WorkerMainThreadHostedCallbackRequestEnvelope =
            from_js(request).map_err(JsValue::from)?;
        let result: WorkerMainThreadHostedCallbackResult =
            from_js(result).map_err(JsValue::from)?;
        let report = self.admit_main_thread_hosted_callback_result_for_test(request, result)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyMainThreadHostedCallbackExecution)]
    pub fn certify_main_thread_hosted_callback_execution(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_main_thread_hosted_callback_execution_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = exportWorkerRuntimeEnvelope)]
    pub fn export_worker_runtime_envelope(&self) -> Result<JsValue, JsValue> {
        let envelope = self.export_worker_runtime_envelope_for_test()?;
        to_js_structured(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = exportWorkerRuntimeEnvelopePortableWire)]
    pub fn export_worker_runtime_envelope_portable_wire(&self) -> Result<String, JsValue> {
        self.shell
            .borrow()
            .preflight_worker_runtime_envelope_export()
            .map_err(JsValue::from)?;
        let definitions = self
            .shell
            .borrow_mut()
            .export_definitions()
            .map_err(JsValue::from)?;
        let state = self
            .export_worker_runtime_envelope_for_test()
            .map(|envelope| envelope.snapshot.state)
            .map_err(JsValue::from)?;
        let artifact = PortableRuntimeEnvelopeArtifact { definitions, state };
        to_json_wire(&artifact).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = exportWorkerSnapshotEnvelope)]
    pub fn export_worker_snapshot_envelope(&self) -> Result<JsValue, JsValue> {
        let snapshot: RuntimeSnapshotEnvelope = self.export_worker_snapshot_envelope_for_test()?;
        to_js_structured(&snapshot).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = exportWorkerSnapshotEnvelopeArtifact)]
    pub fn export_worker_snapshot_envelope_artifact(&self) -> Result<JsValue, JsValue> {
        let snapshot = self.export_worker_snapshot_envelope_for_test()?;
        branch_history::worker_snapshot_envelope_artifact(snapshot)
    }

    #[wasm_bindgen(js_name = exportWorkerSnapshotEnvelopeWire)]
    pub fn export_worker_snapshot_envelope_wire(&self) -> Result<String, JsValue> {
        store_snapshot_envelope(self.export_worker_snapshot_envelope_for_test()?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = exportWorkerSnapshotEnvelopePortableWire)]
    pub fn export_worker_snapshot_envelope_portable_wire(&self) -> Result<String, JsValue> {
        to_json_wire(&self.export_worker_snapshot_envelope_for_test()?).map_err(JsValue::from)
    }
    /// Releases one pending exact restore artifact minted in the worker
    /// realm (snapshot, snapshot envelope, or runtime envelope). `false`
    /// when it is not pending.
    #[wasm_bindgen(js_name = discardRestoreToken)]
    pub fn discard_restore_token(&self, token: String) -> bool {
        discard_restore_token(token)
    }

    #[wasm_bindgen(js_name = exportWorkerRuntimeEnvelopeWire)]
    pub fn export_worker_runtime_envelope_wire(&self) -> Result<String, JsValue> {
        let artifact = self.export_exact_worker_runtime_restore_artifact_for_test()?;
        store_runtime_envelope(artifact).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyWorkerCallbackCapabilityExport)]
    pub fn certify_worker_callback_capability_export(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_worker_callback_capability_export_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyWorkerCallbackPhase4Closeout)]
    pub fn certify_worker_callback_phase4_closeout(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_worker_callback_phase4_closeout_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyWorkerImportExportCallbackUnavailability)]
    pub fn certify_worker_import_export_callback_unavailability(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_worker_import_export_callback_unavailability_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitWorkerRuntimeEnvelopeImport)]
    pub fn admit_worker_runtime_envelope_import(
        &self,
        envelope: JsValue,
    ) -> Result<JsValue, JsValue> {
        let envelope: RuntimeEnvelope = from_js(envelope).map_err(JsValue::from)?;
        let report = self.admit_worker_runtime_envelope_import_for_test(envelope)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitWorkerRuntimeEnvelopeImportPortableWire)]
    pub fn admit_worker_runtime_envelope_import_portable_wire(
        &self,
        envelope: String,
    ) -> Result<JsValue, JsValue> {
        let artifact: PortableRuntimeEnvelopeArtifact =
            from_json_wire(&envelope).map_err(JsValue::from)?;
        let report = self
            .shell
            .borrow_mut()
            .admit_worker_runtime_envelope_import_portable_artifact(
                artifact.definitions,
                artifact.state,
            )
            .map_err(JsValue::from)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = admitWorkerRuntimeEnvelopeImportWire)]
    pub fn admit_worker_runtime_envelope_import_wire(
        &self,
        envelope: String,
    ) -> Result<JsValue, JsValue> {
        let envelope = load_runtime_envelope(&envelope).map_err(JsValue::from)?;
        let report = self.admit_exact_worker_runtime_restore_artifact_for_test(envelope)?;
        to_js(&report).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = deliverLatestObservation)]
    pub fn deliver_latest_observation(&self) -> Result<JsValue, JsValue> {
        let packet = self.deliver_latest_observation_for_test()?;
        to_js(&packet).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyWorkerObservationDelivery)]
    pub fn certify_worker_observation_delivery(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_worker_observation_delivery_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = deliverOutputs)]
    pub fn deliver_outputs(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerOutputDeliveryRequest = from_js(request).map_err(JsValue::from)?;
        let packet = self.deliver_outputs_for_test(request)?;
        to_js(&packet).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = certifyWorkerOutputDelivery)]
    pub fn certify_worker_output_delivery(&self) -> Result<JsValue, JsValue> {
        let package = self.certify_worker_output_delivery_for_test()?;
        to_js(&package).map_err(JsValue::from)
    }
}
