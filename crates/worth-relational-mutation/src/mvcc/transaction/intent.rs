use crate::contracts::schema::data::continuity::transition_classification::SchemaReconciliationPolicy;
use crate::contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition;
use worth_relational_branch::internal_port::branch::materialization::RelationalMaterializationCustodyOwnerPort as _;

/// Caller-authored mutation policy before the Relational owner binds it to an
/// exact branch basis. It carries no branch selector or operational authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalTransactionIntent {
    pub(crate) allow_nested_savepoints: bool,
    pub(crate) proposed_schema_transition: Option<ProposedSchemaTransition>,
    pub(crate) schema_reconciliation_policy: Option<SchemaReconciliationPolicy>,
    pub(crate) materialization_mode: Option<RelationalMaterializationTransactionMode>,
    restoration: Option<crate::validation::candidate_source::NativeSourceRestorationOrigin>,
    materialization_source: Option<(
        worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>,
    )>,
}

pub use crate::contracts::mvcc::transaction::intent::materialization_mode;
pub(crate) use crate::contracts::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode;

impl RelationalTransactionIntent {
    /// Exhaustively expose only portable authored policy to the Home commitment.
    pub(crate) fn home_authored_policy(
        &self,
    ) -> Option<(
        bool,
        Option<&ProposedSchemaTransition>,
        Option<&SchemaReconciliationPolicy>,
    )> {
        let Self {
            allow_nested_savepoints,
            proposed_schema_transition,
            schema_reconciliation_policy,
            materialization_mode,
            restoration,
            materialization_source,
        } = self;
        if materialization_mode.is_some()
            || restoration.is_some()
            || materialization_source.is_some()
        {
            return None;
        }
        Some((
            *allow_nested_savepoints,
            proposed_schema_transition.as_ref(),
            schema_reconciliation_policy.as_ref(),
        ))
    }

    pub fn ordinary() -> Self {
        Self {
            allow_nested_savepoints: true,
            proposed_schema_transition: None,
            schema_reconciliation_policy: None,
            materialization_mode: None,
            materialization_source: None,
            restoration: None,
        }
    }

    pub(crate) fn materialization(mode: RelationalMaterializationTransactionMode) -> Self {
        Self {
            allow_nested_savepoints: false,
            proposed_schema_transition: None,
            schema_reconciliation_policy: None,
            materialization_mode: Some(mode),
            materialization_source: None,
            restoration: None,
        }
    }

    pub(crate) fn with_materialization_source(
        mut self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        source: std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>,
    ) -> Self {
        self.materialization_source = Some((basis.clone(), source));
        self
    }

    pub(crate) fn with_materialization_custody(
        self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        custody: &worth_relational_branch::internal_port::branch::materialization::RelationalMaterializationCustody,
    ) -> Self {
        let mut intent =
            self.with_materialization_source(basis, std::sync::Arc::clone(&custody.data().source));
        intent.restoration = Some(
            crate::validation::candidate_source::NativeSourceRestorationOrigin::from_custody(
                basis, custody,
            ),
        );
        intent
    }

    pub(crate) fn restoration_origin(
        &self,
    ) -> Option<&crate::validation::candidate_source::NativeSourceRestorationOrigin> {
        self.restoration.as_ref()
    }

    pub(crate) fn materialization_source(
        &self,
    ) -> Option<&std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>>
    {
        self.materialization_source
            .as_ref()
            .map(|(_, source)| source)
    }

    pub(crate) fn admits_materialization_source(
        &self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> bool {
        self.materialization_source
            .as_ref()
            .is_none_or(|(selected, source)| {
                selected == basis
                    && match self.materialization_mode {
                        Some(RelationalMaterializationTransactionMode::Suspend) => {
                            source.basis() == basis
                        }
                        Some(RelationalMaterializationTransactionMode::Rematerialize) => {
                            source.basis().identity() == basis.identity()
                        }
                        None => false,
                    }
            })
    }

    pub(crate) const fn materialization_mode(
        &self,
    ) -> Option<RelationalMaterializationTransactionMode> {
        self.materialization_mode
    }

    pub(crate) const fn allow_nested_savepoints(&self) -> bool {
        self.allow_nested_savepoints
    }

    pub(crate) fn proposed_schema_transition(&self) -> Option<&ProposedSchemaTransition> {
        self.proposed_schema_transition.as_ref()
    }

    pub(crate) fn schema_reconciliation_policy(&self) -> Option<&SchemaReconciliationPolicy> {
        self.schema_reconciliation_policy.as_ref()
    }

    pub fn with_schema_transition(
        mut self,
        proposed_schema_transition: ProposedSchemaTransition,
        schema_reconciliation_policy: Option<SchemaReconciliationPolicy>,
    ) -> Self {
        self.proposed_schema_transition = Some(proposed_schema_transition);
        self.schema_reconciliation_policy = schema_reconciliation_policy;
        self
    }
}

#[doc(hidden)]
pub trait RelationalTransactionIntentOwnerPort: Sized {
    fn home_authored_policy(
        &self,
    ) -> Option<(
        bool,
        Option<&ProposedSchemaTransition>,
        Option<&SchemaReconciliationPolicy>,
    )>;

    fn materialization(mode: RelationalMaterializationTransactionMode) -> Self;

    fn with_materialization_source(
        self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        source: std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>,
    ) -> Self;

    fn with_materialization_custody(
        self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        custody: &worth_relational_branch::internal_port::branch::materialization::RelationalMaterializationCustody,
    ) -> Self;

    fn restoration_origin(
        &self,
    ) -> Option<&crate::validation::candidate_source::NativeSourceRestorationOrigin>;

    fn materialization_source(
        &self,
    ) -> Option<&std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>>;

    fn admits_materialization_source(
        &self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> bool;

    fn materialization_mode(&self) -> Option<RelationalMaterializationTransactionMode>;

    fn allow_nested_savepoints(&self) -> bool;

    fn proposed_schema_transition(&self) -> Option<&ProposedSchemaTransition>;

    fn schema_reconciliation_policy(&self) -> Option<&SchemaReconciliationPolicy>;
}

impl RelationalTransactionIntentOwnerPort for RelationalTransactionIntent {
    fn home_authored_policy(
        &self,
    ) -> Option<(
        bool,
        Option<&ProposedSchemaTransition>,
        Option<&SchemaReconciliationPolicy>,
    )> {
        self.home_authored_policy()
    }

    fn materialization(mode: RelationalMaterializationTransactionMode) -> Self {
        Self::materialization(mode)
    }

    fn with_materialization_source(
        self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        source: std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>,
    ) -> Self {
        self.with_materialization_source(basis, source)
    }

    fn with_materialization_custody(
        self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
        custody: &worth_relational_branch::internal_port::branch::materialization::RelationalMaterializationCustody,
    ) -> Self {
        self.with_materialization_custody(basis, custody)
    }

    fn restoration_origin(
        &self,
    ) -> Option<&crate::validation::candidate_source::NativeSourceRestorationOrigin> {
        self.restoration_origin()
    }

    fn materialization_source(
        &self,
    ) -> Option<&std::sync::Arc<crate::validation::candidate_source::NativeMaterializationSource>>
    {
        self.materialization_source()
    }

    fn admits_materialization_source(
        &self,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> bool {
        self.admits_materialization_source(basis)
    }

    fn materialization_mode(&self) -> Option<RelationalMaterializationTransactionMode> {
        self.materialization_mode()
    }

    fn allow_nested_savepoints(&self) -> bool {
        self.allow_nested_savepoints()
    }

    fn proposed_schema_transition(&self) -> Option<&ProposedSchemaTransition> {
        self.proposed_schema_transition()
    }

    fn schema_reconciliation_policy(&self) -> Option<&SchemaReconciliationPolicy> {
        self.schema_reconciliation_policy()
    }
}

#[doc(hidden)]
pub trait TransactionSchemaChangeOwnerPort {
    fn into_schema_change(
        self,
    ) -> (
        Option<ProposedSchemaTransition>,
        Option<SchemaReconciliationPolicy>,
    );
}

impl TransactionSchemaChangeOwnerPort for RelationalTransactionIntent {
    fn into_schema_change(
        self,
    ) -> (
        Option<ProposedSchemaTransition>,
        Option<SchemaReconciliationPolicy>,
    ) {
        (
            self.proposed_schema_transition,
            self.schema_reconciliation_policy,
        )
    }
}
