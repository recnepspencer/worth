pub use crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleAncestry,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply_from_parent,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::reject_duplicate_relation,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::insert_vessel,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::next_vessel_key,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::vessel_call_signs,
    crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError,
};

impl OracleBranch {
    pub fn apply(
        &self,
        delta: crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId,
    ) -> Result<Self, OracleApplicationError> {
        crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply(self, delta)
    }
}
