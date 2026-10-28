//! Fresh same-proposal observation across the committed/proposed posture boundary.
use super::{
    denial, map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantReadView, WorthQueryInvariantAccessDenial,
    WorthQueryInvariantAccessDenialKind as Kind,
};
use crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantEntityBinding;
use worth_relational::facade::runtime::StructuralReadError;

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// Observe the same physical entity in this view, using an owner-issued
    /// selector from either view of this exact proposal. The target view's
    /// declared kind, original controls and prepared scope remain authority.
    /// A missing target returns None; this never copies the selector's posture
    /// or accepts an identity retained from another proposal or installation.
    pub fn reobserve_entity<Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantEntityBinding<Schema, Entity>,
        selector: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<
        Option<WorthQueryApplicationInvariantEntity<Schema, Entity>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        self.check_binding(&selector.binding_identity)?;
        if self.proposal_affinity.is_none() || self.proposal_affinity != selector.proposal_affinity
        {
            return Err(denial(Kind::ForeignView, "entity observation proposal"));
        }
        if selector.kind != binding.entity_kind {
            return Err(denial(Kind::WrongEntityKind, "entity observation selector"));
        }
        self.relations
            .require_entity_kind(binding.entity_kind)
            .map_err(|error| map_structural_error(error, "entity observation kind"))?;
        // Absence is also a read. Evaluation cannot probe an unprepared ID.
        self.admission
            .entity(selector.entity_id, self.relations.read_budget())?;
        let actual = match self.relations.readable_entity_kind(selector.entity_id) {
            Ok(kind) => kind,
            Err(StructuralReadError::RecordUnavailable) => return Ok(None),
            Err(error) => return Err(map_structural_error(error, "entity observation")),
        };
        self.reads.endpoint();
        match actual {
            Some(kind) if kind == binding.entity_kind => {
                Ok(Some(self.entity(selector.entity_id, kind)))
            }
            Some(_) => Err(denial(Kind::WrongEntityKind, "observed entity kind")),
            None => Err(denial(Kind::OutsideDeclaredAccess, "observed entity kind")),
        }
    }
}
