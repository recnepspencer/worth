//! session entry.
use super::*;

impl WorthQueryWorkspace {
    pub fn preview<'a>(
        &'a mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.runtime.preview(request_execution, label)
    }

    pub fn preview_with_options<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
        options: WorthQueryPreviewOptions,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.runtime.preview_with_options(execution, label, options)
    }

    pub fn branch<'a>(
        &'a mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.runtime.branch(request_execution, label)
    }

    pub fn branch_with_options<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.runtime.branch_with_options(execution, label, options)
    }
}
