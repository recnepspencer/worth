//! Tuple traversal preserves declaration, extension and correction order.
use super::*;
macro_rules! tuples {
    ($($member:ident $witness:ident),+) => {
        impl<S, Set, $($member, $witness),+> FragmentMembers<S, Set, ($($witness,)+)> for ($($member,)+)
        where $($member: FragmentMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower(builder, prefix, state)?;)+ Ok(builder)
            }
            fn lower_extensions(builder: Builder<S>, prefix: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower_extensions(builder, prefix, state)?;)+ Ok(builder)
            }
            fn visit_correction<Visitor>(source: std::any::TypeId, visitor: Visitor) -> Result<Visitor::Output, Visitor>
            where S: crate::application_schema::ApplicationSchema, Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<S> {
                $(let visitor = match <$member as FragmentMember<S, Set, $witness>>::visit_correction(source, visitor) {
                    Ok(output) => return Ok(output), Err(visitor) => visitor,
                };)+
                Err(visitor)
            }
        }
        impl<S, Set, $($member, $witness),+> AspectMembers<S, Set, ($($witness,)+)> for ($($member,)+)
        where $($member: AspectMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower(builder, prefix, entity, state)?;)+ Ok(builder)
            }
        }
    };
}

macro_rules! relations {
    ($($relation:ident $witness:ident),+) => {
        impl<S, Set, $($relation, $witness),+> RelationMembers<S, Set, ($($witness,)+)> for ($($relation,)+)
        where $($relation: relation_payload::RelationMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $relation::lower(builder, prefix, entity, state)?;)+ Ok(builder)
            }
        }
    };
}

tuples!(A WA);
tuples!(A WA, B WB);
tuples!(A WA, B WB, C WC);
tuples!(A WA, B WB, C WC, D WD);
tuples!(A WA, B WB, C WC, D WD, E WE);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
relations!(A WA);
relations!(A WA, B WB);
relations!(A WA, B WB, C WC);
relations!(A WA, B WB, C WC, D WD);
relations!(A WA, B WB, C WC, D WD, E WE);
relations!(A WA, B WB, C WC, D WD, E WE, F WF);
relations!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
relations!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
