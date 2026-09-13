//! Product-local semantic output correspondence owned by Query publication.

use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    provider::WorthQueryPrimaryGraphCommittedApplication,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputProjectionDenial,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SemanticSource {
    runtime_authority: u64,
    schema: ApplicationSchemaBindingIdentity,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    output_binding: TypeId,
}

#[derive(Default)]
pub(super) struct WorthQueryApplicationOutputLineage {
    by_source: HashMap<
        SemanticSource,
        HashMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, WorthQueryApplicationOutputCorrespondence>,
        >,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPriorOutputDenialKind {
    Unavailable,
    MissingRole,
    ActionMismatch,
    EntityMismatch,
    OutputUnavailable,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPriorOutputDenial {
    kind: WorthQueryPriorOutputDenialKind,
    role: String,
}

impl WorthQueryApplicationOutputLineage {
    pub(super) fn record(&mut self, application: &WorthQueryPrimaryGraphCommittedApplication) {
        let evidence = application.commit_evidence();
        let correspondence = evidence.output_correspondence();
        let Some(output_binding) = correspondence.binding_type() else {
            return;
        };
        let scope = evidence.operation_scope();
        let head = application.product_publication().new_product_head();
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let replaced = self
            .by_source
            .entry(source)
            .or_default()
            .entry(head.lifecycle_incarnation())
            .or_default()
            .insert(head.reference_generation().get(), correspondence.clone());
        assert!(
            replaced.is_none(),
            "one product generation may publish one output binding once"
        );
    }

    pub(super) fn resolve_binding<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Option<WorthQueryApplicationOutputCorrespondence> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        self.by_source
            .get(&source)?
            .get(&occurrence)?
            .range(..=generation)
            .next_back()
            .map(|(_, correspondence)| correspondence.clone())
    }

    pub(super) fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.by_source.retain(|_, versions| {
            versions.remove(&occurrence);
            !versions.is_empty()
        });
    }
}

impl WorthQueryPriorOutputDenial {
    pub const fn kind(&self) -> WorthQueryPriorOutputDenialKind {
        self.kind
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub(super) fn new(kind: WorthQueryPriorOutputDenialKind, role: impl Into<String>) -> Self {
        Self {
            kind,
            role: role.into(),
        }
    }

    pub(super) fn projection(
        role: &str,
        denial: WorthQueryApplicationOutputProjectionDenial,
    ) -> Self {
        let kind = match denial {
            WorthQueryApplicationOutputProjectionDenial::MissingRole => {
                WorthQueryPriorOutputDenialKind::MissingRole
            }
            WorthQueryApplicationOutputProjectionDenial::ForeignBinding => {
                WorthQueryPriorOutputDenialKind::Unavailable
            }
            WorthQueryApplicationOutputProjectionDenial::ActionMismatch => {
                WorthQueryPriorOutputDenialKind::ActionMismatch
            }
            WorthQueryApplicationOutputProjectionDenial::EntityMismatch => {
                WorthQueryPriorOutputDenialKind::EntityMismatch
            }
        };
        Self::new(kind, role)
    }
}

impl std::fmt::Display for WorthQueryPriorOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "prior output denied: {:?} ({})",
            self.kind, self.role
        )
    }
}

impl std::error::Error for WorthQueryPriorOutputDenial {}
