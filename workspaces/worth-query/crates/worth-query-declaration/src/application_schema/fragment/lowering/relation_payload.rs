//! Relation members lower their payload records through the existing flat contract.
use super::{Builder, Denial, Lowering};
use crate::application_schema::{
    fragment::{self, Aspect, Entity, Record, Relation, Shape},
    ApplicationRelationFieldDefinition, ApplicationSchemaMember, AspectIdentity,
    ErasedApplicationRelationAspectDefinition, WorthQueryPortableRelationAspectParts,
};
use worth_foundational::facade::AbsenceLaw;

pub trait RelationMember<S, Set, Witness> {
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial>;
}
impl<S, Set, Members, W> RelationMember<S, Set, super::NestedWitness<W>> for Members
where
    Members: super::RelationMembers<S, Set, W>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        Members::lower(builder, prefix, entity, state)
    }
}
fn relation<S, R: Relation>(
    mut builder: Builder<S>,
    prefix: &str,
    entity: &str,
    state: &mut Lowering,
) -> Result<Builder<S>, Denial> {
    state.names.insert(&format!("{prefix}.{}", R::NAME))?;
    builder
        .member_provenance
        .fragments
        .catalog
        .relation::<R>(prefix, entity)?;
    builder.push_member_in_place(ApplicationSchemaMember::Relation {
        relation: R::NAME.to_owned(),
        from: entity.to_owned(),
        to: <R::To as Entity>::NAME.to_owned(),
        integrity: fragment::declaration::relation_integrity::<R>().into(),
    });
    Ok(builder)
}
impl<S, Set, R: Relation> RelationMember<S, Set, ()> for R {
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        relation::<S, R>(builder, prefix, entity, state)
    }
}
impl<S, Set, R: Relation, Members, W> RelationMember<S, Set, fragment::PayloadWitness<W>>
    for fragment::WithAspects<R, Members>
where
    Members: PayloadMembers<S, Set, R, W>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let builder = relation::<S, R>(builder, prefix, entity, state)?;
        Members::lower(builder, prefix, entity, state)
    }
}
pub trait PayloadMembers<S, Set, R, W> {
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        from: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial>;
}
pub trait PayloadAspect<S, Set, R, W> {
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        from: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial>;
}
impl<S, Set, R: Relation, A: Aspect> PayloadAspect<S, Set, R, super::RequiredWitness> for A
where
    Set: Record<A::Row>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        from: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        emit::<S, Set, R, A>(builder, prefix, from, false, state)
    }
}
impl<S, Set, R: Relation, A: Aspect> PayloadAspect<S, Set, R, super::OptionalWitness> for Option<A>
where
    Set: Record<A::Row>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        from: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        emit::<S, Set, R, A>(builder, prefix, from, true, state)
    }
}
fn emit<S, Set: Record<A::Row>, R: Relation, A: Aspect>(
    mut builder: Builder<S>,
    prefix: &str,
    from: &str,
    optional: bool,
    state: &mut Lowering,
) -> Result<Builder<S>, Denial> {
    let shape = Set::record();
    let columns = shape.columns("").map_err(|cause| Denial::InvalidRecord {
        path: format!("{prefix}.{}.{}.{}", R::NAME, A::NAME, cause.path()),
    })?;
    builder
        .member_provenance
        .fragments
        .register_bindings(shape.bindings());
    let definition = builder
        .member_provenance
        .fragments
        .catalog
        .relation_aspect::<Set, R, A>(prefix, &columns, optional)?;
    let identity = AspectIdentity(state.names.insert(definition.path())?);
    let mut revision_fields = Vec::new();
    let mut fields = Vec::new();
    for field in definition.fields() {
        let recipe = field.recipe();
        let locus = recipe.locus();
        state.names.insert(field.path())?;
        if field.content_format().is_some() {
            return Err(Denial::InvalidRecord {
                path: field.path().to_owned(),
            });
        }
        if let Some(unit) = recipe.unit()
            && state.units.insert(unit.as_str().to_owned())
        {
            builder.push_member_in_place(ApplicationSchemaMember::Unit {
                unit: unit.as_str().to_owned(),
            });
        }
        builder
            .member_provenance
            .register_field_binding(recipe.clone());
        fields.push(ApplicationRelationFieldDefinition {
            field: locus.field().to_owned(),
            presence: field.presence(),
            scalar_family: recipe.scalar_family(),
            value_type: recipe.binding_identity().as_str().to_owned(),
            canonical: recipe.canonical_contract().cloned(),
            unit: recipe.unit().map(|u| u.as_str().to_owned()),
            frame: recipe.frame().map(|f| f.as_str().to_owned()),
            writable: field.writable(),
        });
        revision_fields.push(ApplicationSchemaMember::Field {
            entity: R::NAME.to_owned(),
            aspect: definition.name().to_owned(),
            field: locus.field().to_owned(),
            presence: field.presence(),
            scalar_family: recipe.scalar_family(),
            value_type: recipe.binding_identity().as_str().to_owned(),
            canonical: recipe.canonical_contract().cloned(),
            content_format: None,
            unit: recipe.unit().map(|u| u.as_str().to_owned()),
            frame: recipe.frame().map(|f| f.as_str().to_owned()),
            writable: field.writable(),
            equality_queryable: field.equality(),
            unique: field.unique(),
        });
    }
    fields.sort_by(|a, b| a.field.cmp(&b.field));
    let revision = super::aspect::revision(&definition, &mut revision_fields, None)?;
    builder.push_member_in_place(ApplicationSchemaMember::RelationAspect {
        definition: ErasedApplicationRelationAspectDefinition::from_untrusted_parts(
            WorthQueryPortableRelationAspectParts {
                relation: R::NAME.to_owned(),
                from: from.to_owned(),
                to: <R::To as Entity>::NAME.to_owned(),
                aspect: definition.name().to_owned(),
                identity,
                revision,
                absence: if optional {
                    AbsenceLaw::Optional
                } else {
                    AbsenceLaw::Required
                },
                fields,
            },
        ),
    });
    Ok(builder)
}
macro_rules! tuples {
    ($($a:ident $w:ident),+) => { impl<S, Set, R, $($a, $w),+> PayloadMembers<S, Set, R, ($($w,)+)> for ($($a,)+)
        where $($a: PayloadAspect<S, Set, R, $w>,)+ {
        fn lower(builder: Builder<S>, prefix: &str, from: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> { $(let builder = $a::lower(builder, prefix, from, state)?;)+ Ok(builder) }
    }};
}
tuples!(A WA);
tuples!(A WA, B WB);
tuples!(A WA, B WB, C WC);
tuples!(A WA, B WB, C WC, D WD);
tuples!(A WA, B WB, C WC, D WD, E WE);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
