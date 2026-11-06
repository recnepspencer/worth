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
        E: Entity,
        A: 'static + In<F::Members, (EP, AP), (F, E)>,
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
        E: Entity,
        A: Aspect + In<F::Members, (EP, AP), (F, E)>,
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

pub(in crate::application_schema) fn expand_record_access(
    catalog: &FragmentCatalog,
    requests: &[RecordAccess],
    members: &mut Vec<Member>,
    contributions: &mut [crate::application_schema::contribution::AuthoredApplicationSchemaContribution],
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
        let mut expansion = Vec::new();
        if request.write {
            let aspect = selected.ok_or_else(|| request.refusal())?;
            columns(&mut expansion, request, aspect);
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
                        &mut expansion,
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
                    &mut expansion,
                    request,
                    fragment
                        .aspects
                        .get(&(request.entity, case.marker()))
                        .ok_or_else(|| request.refusal())?,
                );
            }
        } else {
            columns(
                &mut expansion,
                request,
                selected.ok_or_else(|| request.refusal())?,
            );
        }
        for contribution in contributions.iter_mut() {
            contribution.expand_record(request, &expansion);
        }
        for member in expansion {
            insert(members, member);
        }
    }
    // A scalar write also carries the SDK's structural parent column. The
    // declaration never asks its owner to name that column.
    let sources = members.clone();
    for source in sources {
        let derived: Vec<_> = match &source {
            Member::OperationProgram {
                operation,
                target:
                    WriteTarget::Write {
                        entity,
                        aspect,
                        field,
                    },
            } if !catalog.internal_field(
                &crate::application_schema::ApplicationFieldBindingLocus::new(
                    entity, aspect, field,
                ),
            ) =>
            {
                catalog
                    .presence_field(entity, aspect)
                    .into_iter()
                    .map(|presence| Member::OperationProgram {
                        operation: operation.clone(),
                        target: WriteTarget::Write {
                            entity: entity.clone(),
                            aspect: aspect.clone(),
                            field: presence.recipe.locus().field().to_owned(),
                        },
                    })
                    .collect()
            }
            Member::OperationDecisionRead {
                operation,
                target:
                    ReadTarget::Field {
                        entity,
                        aspect,
                        field,
                    },
            } if !catalog.internal_field(
                &crate::application_schema::ApplicationFieldBindingLocus::new(
                    entity, aspect, field,
                ),
            ) =>
            {
                catalog
                    .presence_field(entity, aspect)
                    .into_iter()
                    .map(|presence| Member::OperationDecisionRead {
                        operation: operation.clone(),
                        target: ReadTarget::Field {
                            entity: entity.clone(),
                            aspect: aspect.clone(),
                            field: presence.recipe.locus().field().to_owned(),
                        },
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        for member in derived {
            for contribution in contributions.iter_mut() {
                contribution.expand_member(&source, &member);
            }
            insert(members, member);
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
