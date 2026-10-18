//! Capture original source/input/key before effects; no request authority escapes.
use super::super::application_authentication::authenticate;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
};

impl WorthUiStatusMutation {
    /// Effect-free preparation; a refusal retains every original already captured.
    pub fn prepare(mut self, scope: &WorthQueryRequestScope) -> Result<Self, Self> {
        let principal = match authenticate(self.application.installed_schema(), scope) {
            Ok(principal) => principal,
            Err(denial) => return Err(self.refused(WorthUiStatusMutationStop::Setup(denial))),
        };
        let request = self.application.request(&principal, scope);
        let result = self.state.prepare_in_request(&request, &self.application);
        match result {
            Ok(()) => {
                self.state.stop = None;
                Ok(self)
            }
            Err(denial) => Err(self.refused(denial)),
        }
    }
}
impl MutationState {
    pub(super) fn prepare_in_request(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        application: &WorthUiStatusApplication,
    ) -> Result<(), WorthUiStatusMutationStop> {
        if self.source.is_none() {
            let before = request
                .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
                .execute()
                .map_err(|denial| {
                    WorthUiStatusMutationStop::Setup(WorthUiStatusOwnerError::Query(format!(
                        "{denial:?}"
                    )))
                })?;
            let [source] = before.observed_sources() else {
                return Err(WorthUiStatusMutationStop::Setup(
                    WorthUiStatusOwnerError::MissingUniqueSource,
                ));
            };
            let [row] = before.rows() else {
                return Err(WorthUiStatusMutationStop::Setup(
                    WorthUiStatusOwnerError::MissingUniqueRecord,
                ));
            };
            self.source = Some(source.clone());
            self.active_revision = Some(row.revision);
        }
        let native = self.native.take().expect("one original native state");
        let (native, result) = native.prepare(
            request,
            self.source.as_ref().expect("original source retained"),
            application,
        );
        self.native = Some(native);
        result
    }
}
