use wasm_bindgen::prelude::*;

use crate::boundary::restore_tokens::{
    discard_restore_token, load_runtime_envelope, store_runtime_envelope,
};
use crate::boundary::serde::{from_js, from_json_wire, to_js, to_js_structured, to_json_wire};
use crate::runtime::adapters::{PortableRuntimeEnvelopeArtifact, RuntimeEnvelope};

use super::types::{SignalAdapters, SignalSpecialist};

#[wasm_bindgen]
impl SignalSpecialist {
    pub fn graph_summary(&self) -> Result<JsValue, JsValue> {
        let summary = self.core.borrow().graph_summary().map_err(JsValue::from)?;
        to_js(&summary).map_err(JsValue::from)
    }

    pub fn evaluate_dirty(&self) -> Result<JsValue, JsValue> {
        let summary = self
            .core
            .borrow_mut()
            .evaluate_dirty()
            .map_err(JsValue::from)?;
        to_js(&summary).map_err(JsValue::from)
    }

    pub fn read_versions(&self, ids: JsValue) -> Result<JsValue, JsValue> {
        let ids: Vec<String> = from_js(ids)?;
        let versions = self
            .core
            .borrow_mut()
            .read_versions(ids)
            .map_err(JsValue::from)?;
        to_js(&versions).map_err(JsValue::from)
    }
}

#[wasm_bindgen]
impl SignalAdapters {
    pub fn export_definitions(&self) -> Result<JsValue, JsValue> {
        let definitions = self
            .core
            .borrow_mut()
            .export_definitions()
            .map_err(JsValue::from)?;
        to_js(&definitions).map_err(JsValue::from)
    }

    pub fn export_runtime_envelope(&self) -> Result<JsValue, JsValue> {
        let envelope = self
            .core
            .borrow_mut()
            .export_runtime_envelope()
            .map_err(JsValue::from)?;
        to_js_structured(&envelope).map_err(JsValue::from)
    }

    /// Releases one pending exact runtime envelope minted by this realm's
    /// `export_runtime_envelope_wire`. `false` when it is not pending.
    pub fn discard_restore_token(&self, token: String) -> bool {
        discard_restore_token(token)
    }

    pub fn export_runtime_envelope_wire(&self) -> Result<String, JsValue> {
        let envelope = self
            .core
            .borrow_mut()
            .export_exact_runtime_restore_artifact()
            .map_err(JsValue::from)?;
        store_runtime_envelope(envelope).map_err(JsValue::from)
    }

    pub fn export_runtime_envelope_portable_wire(&self) -> Result<String, JsValue> {
        self.core
            .borrow()
            .preflight_runtime_envelope_export()
            .map_err(JsValue::from)?;
        let definitions = self
            .core
            .borrow_mut()
            .export_definitions()
            .map_err(JsValue::from)?;
        let state = self
            .core
            .borrow_mut()
            .snapshot()
            .map(|snapshot| snapshot.state)
            .map_err(JsValue::from)?;
        let artifact = PortableRuntimeEnvelopeArtifact { definitions, state };
        to_json_wire(&artifact).map_err(JsValue::from)
    }

    pub fn runtime_proof_report(&self) -> Result<JsValue, JsValue> {
        let report = self.core.borrow().runtime_proof_report();
        to_js(&report).map_err(JsValue::from)
    }

    pub fn replace_runtime_envelope(&self, envelope: JsValue) -> Result<(), JsValue> {
        let envelope: RuntimeEnvelope = from_js(envelope)?;
        self.core
            .borrow_mut()
            .replace_runtime_envelope(envelope)
            .map_err(JsValue::from)
    }

    pub fn replace_runtime_envelope_wire(&self, envelope: String) -> Result<(), JsValue> {
        let envelope = load_runtime_envelope(&envelope).map_err(JsValue::from)?;
        self.core
            .borrow_mut()
            .replace_runtime_envelope_exact(envelope)
            .map_err(JsValue::from)
    }

    pub fn replace_runtime_envelope_portable_wire(&self, envelope: String) -> Result<(), JsValue> {
        let artifact: PortableRuntimeEnvelopeArtifact =
            from_json_wire(&envelope).map_err(JsValue::from)?;
        self.core
            .borrow_mut()
            .replace_runtime_envelope_portable_artifact(artifact.definitions, artifact.state)
            .map_err(JsValue::from)
    }
}

#[cfg(test)]
impl SignalAdapters {
    pub(super) fn export_runtime_envelope_for_test(
        &self,
    ) -> Result<RuntimeEnvelope, crate::boundary::errors::WorthSignalJsError> {
        self.core.borrow_mut().export_runtime_envelope()
    }

    pub(super) fn replace_runtime_envelope_for_test(
        &self,
        envelope: RuntimeEnvelope,
    ) -> Result<(), crate::boundary::errors::WorthSignalJsError> {
        self.core.borrow_mut().replace_runtime_envelope(envelope)
    }
}
