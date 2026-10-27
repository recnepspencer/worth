use super::*;
impl ApplicationQueryDisclosureContract {
    /// Govern identity, complete membership and count as one component result slot.
    pub fn components_by<
        Schema,
        Query,
        Slot,
        Set,
        Entity,
        DisclosureBinding: ApplicationScalarValueBinding,
    >(
        mut self,
        selection: super::super::ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>,
        disclosure_value: ApplicationEncodedScalarValue<DisclosureBinding>,
        influence: ApplicationQueryInfluenceContract,
    ) -> Self {
        self.rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Components(selection.into_meaning()),
            disclosure_value: disclosure_value.into_foundational_value(),
            influence,
        });
        self.rules.sort();
        self
    }
}
