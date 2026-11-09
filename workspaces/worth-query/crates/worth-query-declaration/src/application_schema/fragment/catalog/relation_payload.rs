//! Relation records retain relation-owned codec loci in the same fragment catalog.
use super::*;
use crate::application_schema::ApplicationFieldBindingLocus;
impl FragmentCatalog {
    pub(crate) fn relation_aspect<Set, R, A>(
        &mut self,
        source_path: &str,
        columns: &[Column],
        optional: bool,
    ) -> Result<AspectDefinition, Denial>
    where
        Set: Record<A::Row>,
        R: super::super::Relation,
        A: Aspect,
    {
        super::super::super::identifier_validation::validate_simple_identifier(A::NAME)?;
        let path = format!("{source_path}.{}.{}", R::NAME, A::NAME);
        let identity = super::super::identity::native_relation_aspect_identity(R::NAME, A::NAME);
        if let Some(first) = self.native_aspects.get(&identity) {
            return Err(if first == &path {
                Denial::DuplicateFragmentMember { path }
            } else {
                Denial::FragmentIdentityCollision {
                    identity,
                    first: first.clone(),
                    second: path,
                }
            });
        }
        self.native_aspects.insert(identity, path.clone());
        let name = super::super::identity::native_relation_aspect_key::<R, A>();
        let fields: Vec<_> = columns
            .iter()
            .map(|column| RecordFieldDefinition {
                content: column.content.clone(),
                path: format!("{path}.{}", column.path),
                recipe: (column.recipe)(ApplicationFieldBindingLocus::for_relation(
                    R::NAME,
                    name,
                    &column.path,
                )),
                presence: column.presence,
                internal_presence: false,
                writable: column.writable,
                equality: column.equality,
                unique: column.unique,
            })
            .collect();
        let definition = AspectDefinition {
            path,
            entity: R::NAME.to_owned(),
            name: name.to_owned(),
            meaning: A::NAME.to_owned(),
            binding: RecordBinding::record::<Set, A::Row>(),
            fields,
            optional,
        };
        let owner = self.owner(source_path)?;
        if owner
            .aspects
            .insert((TypeId::of::<R>(), TypeId::of::<A>()), definition.clone())
            .is_some()
        {
            return Err(Denial::DuplicateFragmentMember {
                path: definition.path.clone(),
            });
        }
        Ok(definition)
    }
}
