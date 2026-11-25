//! Check the actual registered relation recipe against the selected read audience.
use crate::contracts::derived_relation::definition::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle,
    DerivedRelationHandleFieldsPort as _,
};
use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use crate::contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use crate::contracts::validation::data::structural_read::StructuralReadError;
use crate::validation::custom_rule::structural_views::StructuralRelationView;

impl StructuralRelationView<'_> {
    /// Admit only the actual registered source and this view's full audience.
    /// This does not create a root, observed member or prepared read proof.
    pub(crate) fn require_derived_relation(&self, definition: &Handle) -> Result<(), Denial> {
        self.read_budget()
            .check_live()
            .map_err(Denial::Structural)?;
        self.read_budget()
            .checkpoint(self.derived_admissions.probe_work())
            .map_err(Denial::Structural)?;
        if self.derived_admissions.contains(definition) {
            return Ok(());
        }
        let (registry, _) = self
            .structural_source
            .as_ref()
            .ok_or(DefinitionDenial::ForeignObservation)?;
        let names = definition
            .parts()
            .identity
            .schema
            .0
            .len()
            .checked_add(definition.parts().identity.name.len())
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        let levels = u64::from(
            usize::BITS
                - (*registry.registry().owner_derived_relations())
                    .len()
                    .max(1)
                    .leading_zeros(),
        );
        self.read_budget()
            .checkpoint(
                (names as u64)
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )
            .map_err(Denial::Structural)?;
        if !registry.registry().admits_derived_relation(definition) {
            return Err(DefinitionDenial::ForeignDefinition.into());
        }
        // An observation can be shared by rules with different read rosters.
        // A genuine prepared proof must never widen its consumer's audience.
        for (relation, from, to) in definition.physical_sources() {
            self.require_relation_kind(*relation)
                .map_err(|error| self.derived_refusal(error))?;
            self.require_entity_kind(*from)
                .map_err(|error| self.derived_refusal(error))?;
            self.require_entity_kind(*to)
                .map_err(|error| self.derived_refusal(error))?;
        }
        // A carrier-only recipe may have no physical relation leaves. Its
        // existence and empty-result reads still require the actual carrier.
        for &carrier in &definition.owner_entry().carriers {
            self.require_entity_kind(carrier)
                .map_err(|error| self.derived_refusal(error))?;
        }
        // Only the actual registry and this consumer's successful audience
        // checks can populate this immutable view's inline admission cache.
        self.read_budget()
            .checkpoint(self.derived_admissions.probe_work())
            .map_err(Denial::Structural)?;
        self.derived_admissions.remember(definition);
        Ok(())
    }

    pub(super) fn derived_refusal(&self, error: StructuralReadError) -> Denial {
        Denial::Structural(self.read_budget().refusal(error))
    }
}
