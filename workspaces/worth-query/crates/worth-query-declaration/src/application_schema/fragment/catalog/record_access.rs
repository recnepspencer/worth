//! Whole-record authority derives its columns and case transitions from membership.
use super::{AspectDefinition, FragmentCatalog};
use crate::application_schema::fragment::{Aspect, Entity, Fragment, In, Record};
use crate::application_schema::{
    ApplicationOperationDecisionReadTarget as ReadTarget,
    ApplicationOperationProgramTarget as WriteTarget, ApplicationOperationRef,
    ApplicationSchemaDeclarationBuilder as Builder, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
use std::any::TypeId;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct RecordAccess {
    operation: String,
    fragment: TypeId,
    entity: TypeId,
    marker: TypeId,
    write: bool,
    path: String,
}

impl<S> Builder<S> {
    /// Reads a row or one-of, including the complete case-presence guard.
    pub fn operation_read_record<O, I, F, E, A, EP, AP>(
        mut self,
        operation: ApplicationOperationRef<S, O, I>,
        _: F,
        _: E,
        _: A,
    ) -> Self
    where
        F: Fragment,
        E: Entity + In<F::Members, EP, F>,
        A: 'static + In<E::Aspects, AP, E>,
    {
        self.member_provenance
            .fragments
            .access
            .push(RecordAccess::of::<F, E, A>(operation.name(), false));
        self
    }

    /// A case write includes clearing its siblings. Scalar field grants do not.
    pub fn operation_write_record<O, I, F, E, A, EP, AP>(
        mut self,
        operation: ApplicationOperationRef<S, O, I>,
        _: F,
        _: E,
        _: A,
    ) -> Self
    where
        F: Fragment,
        E: Entity + In<F::Members, EP, F>,
        A: Aspect + In<E::Aspects, AP, E>,
        F::Bindings: Record<A::Row>,
    {
        self.member_provenance
            .fragments
            .access
            .push(RecordAccess::of::<F, E, A>(operation.name(), true));
        self
    }
}

impl RecordAccess {
    fn of<F: Fragment, E: Entity, A: 'static>(operation: &str, write: bool) -> Self {
        Self {
            operation: operation.to_owned(),
            fragment: TypeId::of::<F>(),
            entity: TypeId::of::<E>(),
            marker: TypeId::of::<A>(),
            write,
            path: format!("{}.{}.{}", F::NAME, E::NAME, std::any::type_name::<A>()),
        }
    }
    fn refusal(&self) -> Denial {
        Denial::InvalidRecordMember {
            path: self.path.clone(),
        }
    }
}

pub(crate) fn expand_record_access(
    catalog: &FragmentCatalog,
    requests: &[RecordAccess],
    members: &mut Vec<Member>,
) -> Result<(), Denial> {
    let mut seen = BTreeSet::new();
    for request in requests {
        if !seen.insert(request) {
            return Err(Denial::DuplicateFragmentMember {
                path: request.path.clone(),
            });
        }
        let fragment = catalog
            .definitions
            .get(&request.fragment)
            .ok_or_else(|| request.refusal())?;
        let selected = fragment.aspects.get(&(request.entity, request.marker));
        let group = fragment
            .groups
            .get(&(request.entity, request.marker))
            .or_else(|| {
                fragment
                    .groups
                    .iter()
                    .filter(|((entity, _), _)| *entity == request.entity)
                    .map(|(_, group)| group)
                    .find(|group| {
                        group
                            .cases()
                            .iter()
                            .any(|case| case.marker() == request.marker)
                    })
            });
        if request.write {
            let aspect = selected.ok_or_else(|| request.refusal())?;
            columns(members, request, aspect);
            if let Some(group) = group {
                for case in group
                    .cases()
                    .iter()
                    .filter(|case| case.marker() != request.marker)
                {
                    let sibling = fragment
                        .aspects
                        .get(&(request.entity, case.marker()))
                        .ok_or_else(|| request.refusal())?;
                    insert(
                        members,
                        Member::OperationProgram {
                            operation: request.operation.clone(),
                            target: WriteTarget::RemoveAspect {
                                entity: sibling.entity.clone(),
                                aspect: sibling.name.clone(),
                            },
                        },
                    );
                }
            }
        } else if let Some(group) = group {
            for case in group.cases() {
                columns(
                    members,
                    request,
                    fragment
                        .aspects
                        .get(&(request.entity, case.marker()))
                        .ok_or_else(|| request.refusal())?,
                );
            }
        } else {
            columns(members, request, selected.ok_or_else(|| request.refusal())?);
        }
    }
    Ok(())
}

fn columns(members: &mut Vec<Member>, request: &RecordAccess, aspect: &AspectDefinition) {
    for field in &aspect.fields {
        let locus = field.recipe.locus();
        let member = if request.write {
            Member::OperationProgram {
                operation: request.operation.clone(),
                target: WriteTarget::Write {
                    entity: locus.entity().to_owned(),
                    aspect: locus.aspect().to_owned(),
                    field: locus.field().to_owned(),
                },
            }
        } else {
            Member::OperationDecisionRead {
                operation: request.operation.clone(),
                target: ReadTarget::Field {
                    entity: locus.entity().to_owned(),
                    aspect: locus.aspect().to_owned(),
                    field: locus.field().to_owned(),
                },
            }
        };
        insert(members, member);
    }
}

fn insert(members: &mut Vec<Member>, member: Member) {
    if !members.contains(&member) {
        members.push(member);
    }
}
