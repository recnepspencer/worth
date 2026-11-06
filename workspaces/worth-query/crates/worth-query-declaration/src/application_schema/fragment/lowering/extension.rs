//! Resolve extensions after all entity declarations, independent of tuple order.
use super::{AspectMembers, Builder, Denial, FragmentMember, Lowering};
use crate::application_schema::fragment::{Entity, Extend, OptionalExtensionMembers};
use std::marker::PhantomData;

#[doc(hidden)]
pub struct ExtensionWitness<W>(PhantomData<W>);
impl<S, Set, E, Members, W> FragmentMember<S, Set, ExtensionWitness<W>> for Extend<E, Members>
where
    E: Entity,
    Members: OptionalExtensionMembers<E> + AspectMembers<S, Set, W>,
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
        Members::lower(builder, &path, E::NAME, state)
    }
}
