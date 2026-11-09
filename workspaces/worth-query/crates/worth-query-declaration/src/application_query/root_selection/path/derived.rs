//! Named derived steps retain endpoint namespaces until completed-schema admission.
use super::*;
use crate::application_schema::ApplicationDerivedRelationRef;
impl<S, Start, Current> ApplicationQueryRootPath<S, Start, Current> {
    pub fn forward_derived<R, Next>(
        mut self,
        relation: ApplicationDerivedRelationRef<S, R, Current, Next>,
    ) -> ApplicationQueryRootPath<S, Start, Next> {
        self.steps
            .push(ApplicationQueryRootPathStep::from_untrusted_source_fields(
                relation.name().to_owned(),
                relation.from_member().into_meaning(),
                relation.to_member().into_meaning(),
                ApplicationQueryRootPathSourceKind::Derived,
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::OneHop,
            ));
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.to(),
            steps: self.steps,
            guards: self.guards,
            field_expectations: self.field_expectations,
            _marker: PhantomData,
        }
    }
    pub fn reverse_derived<R, Previous>(
        mut self,
        relation: ApplicationDerivedRelationRef<S, R, Previous, Current>,
    ) -> ApplicationQueryRootPath<S, Start, Previous> {
        self.steps
            .push(ApplicationQueryRootPathStep::from_untrusted_source_fields(
                relation.name().to_owned(),
                relation.from_member().into_meaning(),
                relation.to_member().into_meaning(),
                ApplicationQueryRootPathSourceKind::Derived,
                RelationDirection::Reverse,
                ApplicationQueryRootPathTraversal::OneHop,
            ));
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.from(),
            steps: self.steps,
            guards: self.guards,
            field_expectations: self.field_expectations,
            _marker: PhantomData,
        }
    }
}
