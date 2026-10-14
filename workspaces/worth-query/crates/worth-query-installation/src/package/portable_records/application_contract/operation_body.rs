//! Retained body meaning is part of the whole portable operation contract.
use super::super::WorthQueryPortableApplicationOperationContractRecord;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;

impl WorthQueryPortableApplicationOperationContractRecord {
    pub(crate) fn with_body(mut self, body: Option<ApplicationOperationBodyProgram>) -> Self {
        self.body = body;
        self
    }

    pub fn body(&self) -> Option<&ApplicationOperationBodyProgram> {
        self.body.as_ref()
    }
}
