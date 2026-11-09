//! Resolve extensions after all entity declarations, independent of tuple order.
use super::{AspectMembers, Builder, Denial, FragmentMember, Lowering, RelationMembers};
use crate::application_schema::fragment::{Entity, Extend, OptionalExtensionMembers};
use std::marker::PhantomData;

#[doc(hidden)]
pub struct ExtensionWitness<A, R>(PhantomData<(A, R)>);
impl<S, Set, E, Members, Relations, A, R> FragmentMember<S, Set, ExtensionWitness<A, R>>
    for Extend<E, Members, Relations>
where
    E: Entity,
    Members: OptionalExtensionMembers<E> + AspectMembers<S, Set, A>,
    Relations: RelationMembers<S, Set, R>,
{
    fn lower(builder: Builder<S>, _: &str, _: &mut Lowering) -> Result<Builder<S>, Denial> {
        Ok(builder)
    }
    fn lower_extensions(
        mut builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let path = format!("{prefix}.{}", E::NAME);
        let owner = state
            .entity_paths
            .get(E::NAME)
            .filter(|owner| {
                owner
                    .rsplit_once('.')
                    .is_some_and(|(fragment, _)| fragment != prefix)
            })
            .ok_or_else(|| Denial::UndeclaredExtensionEntity { path: path.clone() })?;
        // The marker must be declared, rather than another marker with the same name.
        builder
            .member_provenance
            .fragments
            .catalog
            .extension::<E>(&path, owner)?;
        state
            .extension_identity_paths
            .insert(path.clone(), owner.clone());
        let builder = Members::lower(builder, &path, E::NAME, state)?;
        Relations::lower(builder, &path, E::NAME, state)
    }
}
