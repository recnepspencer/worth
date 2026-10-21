//! Candidate-local creation sites; peer roots are not owned descendants.
use super::super::ApplicationOperationBodyCreatedField;
/// One typed, candidate-local creation site in the admitted scope's context.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreation {
    pub(in crate::application_schema::operation_definition::body) entity: String,
    pub(in crate::application_schema::operation_definition::body) key: String,
    pub(in crate::application_schema::operation_definition::body) fields:
        Vec<ApplicationOperationBodyCreatedField>,
    pub(in crate::application_schema::operation_definition::body) links:
        Vec<super::super::ApplicationOperationBodyCreationLink>,
    pub(in crate::application_schema::operation_definition::body) signing:
        Option<super::super::ApplicationOperationBodySigningSpec>,
    pub(in crate::application_schema::operation_definition::body) signing_lifecycle:
        Option<super::super::ApplicationOperationBodySigningLifecycleSpec>,
    pub(in crate::application_schema::operation_definition::body) subject_impact_bound:
        Option<std::num::NonZeroUsize>,
    pub(in crate::application_schema::operation_definition::body) enrollment:
        Option<super::super::ApplicationOperationBodyEnrollment>,
    pub(in crate::application_schema::operation_definition::body) reuse_current_if_unchanged: bool,
    pub(in crate::application_schema::operation_definition::body) peers:
        Vec<ApplicationOperationBodyCreation>,
    pub(in crate::application_schema::operation_definition::body) path_links:
        Vec<super::super::ApplicationOperationBodyCreationPathLink>,
    pub(in crate::application_schema::operation_definition::body) children:
        Vec<super::super::ApplicationOperationBodyChildCreation>,
}

impl ApplicationOperationBodyCreation {
    pub fn from_untrusted(
        entity: String,
        key: String,
        fields: Vec<ApplicationOperationBodyCreatedField>,
    ) -> Self {
        Self {
            entity,
            key,
            fields,
            links: Vec::new(),
            signing: None,
            signing_lifecycle: None,
            subject_impact_bound: None,
            enrollment: None,
            reuse_current_if_unchanged: false,
            children: Vec::new(),
            peers: Vec::new(),
            path_links: Vec::new(),
        }
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn children(&self) -> &[super::super::ApplicationOperationBodyChildCreation] {
        &self.children
    }
    /// Inert flat site inventory; fresh admission checks topology and complete initialization.
    pub fn with_untrusted_children(
        mut self,
        children: Vec<super::super::ApplicationOperationBodyChildCreation>,
    ) -> Self {
        self.children = children;
        self
    }
    /// Root first, followed by authored child sites. All native input/capacity owners use this order.
    pub fn initialization_sites(&self) -> impl Iterator<Item = &Self> {
        self.creation_roots().flat_map(|root| {
            std::iter::once(root).chain(root.children.iter().map(|child| &child.creation))
        })
    }
    /// Independent roots share admission, not ownership or reuse disposition.
    pub fn peers(&self) -> &[Self] {
        &self.peers
    }
    pub fn path_links(&self) -> &[super::super::ApplicationOperationBodyCreationPathLink] {
        &self.path_links
    }
    pub fn with_untrusted_peers(mut self, peers: Vec<Self>) -> Self {
        self.peers = peers;
        self
    }
    pub fn with_untrusted_path_links(
        mut self,
        links: Vec<super::super::ApplicationOperationBodyCreationPathLink>,
    ) -> Self {
        self.path_links = links;
        self
    }
    pub fn creation_roots(&self) -> impl Iterator<Item = &Self> {
        std::iter::once(self).chain(self.peers.iter())
    }
    pub fn reuses_current_if_unchanged(&self) -> bool {
        self.reuse_current_if_unchanged
    }
    /// Authority-free carriage; fresh admission checks the complete comparison profile.
    pub fn with_untrusted_reuse_current_if_unchanged(mut self, reuse: bool) -> Self {
        self.reuse_current_if_unchanged = reuse;
        self
    }
    pub fn subject_impact_bound(&self) -> Option<std::num::NonZeroUsize> {
        self.subject_impact_bound
    }
    pub fn with_untrusted_subject_impact_bound(
        mut self,
        bound: Option<std::num::NonZeroUsize>,
    ) -> Self {
        self.subject_impact_bound = bound;
        self
    }
    pub fn enrollment(&self) -> Option<&super::super::ApplicationOperationBodyEnrollment> {
        self.enrollment.as_ref()
    }
    pub fn with_untrusted_enrollment(
        mut self,
        enrollment: Option<super::super::ApplicationOperationBodyEnrollment>,
    ) -> Self {
        self.enrollment = enrollment;
        self
    }
    pub fn signing(&self) -> Option<&super::super::ApplicationOperationBodySigningSpec> {
        self.signing.as_ref()
    }
    pub fn signing_lifecycle(
        &self,
    ) -> Option<&super::super::ApplicationOperationBodySigningLifecycleSpec> {
        self.signing_lifecycle.as_ref()
    }
    pub fn with_untrusted_signing_lifecycle(
        mut self,
        spec: Option<super::super::ApplicationOperationBodySigningLifecycleSpec>,
    ) -> Self {
        self.signing_lifecycle = spec;
        self
    }
    pub fn with_untrusted_signing(
        mut self,
        signing: Option<super::super::ApplicationOperationBodySigningSpec>,
    ) -> Self {
        self.signing = signing;
        self
    }
    pub fn with_untrusted_links(
        mut self,
        links: Vec<super::super::ApplicationOperationBodyCreationLink>,
    ) -> Self {
        self.links = links;
        self
    }
    pub fn links(&self) -> &[super::super::ApplicationOperationBodyCreationLink] {
        &self.links
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn fields(&self) -> &[ApplicationOperationBodyCreatedField] {
        &self.fields
    }
}
