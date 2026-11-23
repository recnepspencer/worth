//! Borrowed Parent membership from inline declarations and retained shared leaves.
use crate::domain_operation::WorthQueryOperationGraphReadRole;
use worth_query_declaration::facade::application_query::ApplicationQueryBindingReference;

/// One declared peer-scope Parent read: the Parent, and the relation whose
/// shared `owner` admits a peer `member` scope beside the operation's own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedOwnerManagedIndexRead {
    parent: ApplicationQueryBindingReference,
    relation: String,
    owner: String,
    member: String,
}

impl WorthQuerySharedOwnerManagedIndexRead {
    pub(crate) fn new(
        parent: ApplicationQueryBindingReference,
        relation: String,
        owner: String,
        member: String,
    ) -> Self {
        Self {
            parent,
            relation,
            owner,
            member,
        }
    }
    pub const fn parent(&self) -> &ApplicationQueryBindingReference {
        &self.parent
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn member(&self) -> &str {
        &self.member
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryOperationManagedIndexReadView<'a> {
    inline: &'a [ApplicationQueryBindingReference],
    roles: &'a [WorthQueryOperationGraphReadRole],
}

impl<'a> WorthQueryOperationManagedIndexReadView<'a> {
    pub(super) fn new(
        inline: &'a [ApplicationQueryBindingReference],
        roles: &'a [WorthQueryOperationGraphReadRole],
    ) -> Self {
        Self { inline, roles }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a ApplicationQueryBindingReference> {
        self.inline.iter().chain(
            self.roles
                .iter()
                .flat_map(|role| role.managed_index_reads()),
        )
    }

    pub fn contains(&self, parent: &ApplicationQueryBindingReference) -> bool {
        self.iter().any(|candidate| candidate == parent)
    }

    pub fn len(&self) -> usize {
        self.inline.len()
            + self
                .roles
                .iter()
                .map(|role| role.managed_index_reads().len())
                .sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        self.inline.is_empty()
            && self
                .roles
                .iter()
                .all(|role| role.managed_index_reads().is_empty())
    }
}
