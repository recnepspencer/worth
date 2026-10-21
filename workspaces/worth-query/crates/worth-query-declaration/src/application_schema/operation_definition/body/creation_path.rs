//! Links bind a retained existing path anchor to this exact created site.
mod authoring;
use super::{path_source, ApplicationOperationBodyCreationLink as Link};
use crate::{
    application_query::ApplicationQueryRootPathMeaning,
    application_schema::ApplicationOperationDecisionReadTarget as Read,
};
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreationPathLink {
    pub path: ApplicationQueryRootPathMeaning,
    pub link: Link,
}
impl ApplicationOperationBodyCreationPathLink {
    pub fn has_supported_shape(&self) -> bool {
        path_source::supported_path(&self.path)
            && matches!(
                self.link.source,
                super::ApplicationOperationBodyCreationLinkSource::Scope
            )
            && self.link.scope_entity() == self.path.terminal_entity()
            && !self.link.key.is_empty()
            && self.link.key.trim() == self.link.key
            && !self.link.posture.replaces_selection()
    }
    pub fn decision_reads(&self) -> Vec<Read> {
        path_source::path_reads(&self.path)
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("creation-path-link-v1;");
        let mut part = |s: &str| write!(&mut record, "{}:{s};", s.len()).expect("String write");
        part(self.path.start_entity());
        part(self.path.terminal_entity());
        part(&self.path.steps().len().to_string());
        for step in self.path.steps() {
            part(step.relation());
            part(step.from());
            part(step.to());
            part(match step.direction() {
                crate::application_query::ApplicationQueryRootPathDirection::Forward => "forward",
                crate::application_query::ApplicationQueryRootPathDirection::Reverse => "reverse",
            });
        }
        part(&self.link.canonical_record());
        record
    }
}
