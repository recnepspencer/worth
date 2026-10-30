//! Close Source requirements against the actual rule and query, without admitting reads.
use super::super::{
    ApplicationInvariantExecutionPoint as Point, ApplicationInvariantScopeTarget as Target,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use crate::application_query::{
    ApplicationInvariantManagedIndexSource as Requirement, ApplicationQueryResultShape,
    ErasedApplicationQueryDefinition,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(members: &[Member]) -> Result<(), Denial> {
    let mut associations = BTreeSet::new();
    let mut scopes = BTreeMap::new();
    for member in members {
        let Member::InvariantManagedIndexSource {
            invariant,
            execution_point,
            requirement,
        } = member
        else {
            continue;
        };
        let denied = || Denial::UnsupportedInvariantSource {
            invariant: invariant.clone(),
            index: requirement.index().to_owned(),
        };
        if !associations.insert((invariant, execution_point, requirement.index())) {
            return Err(denied());
        }
        let (read, affected) =
            rule_audience(members, invariant, *execution_point).ok_or_else(denied)?;
        let source = source_query(members, requirement).ok_or_else(denied)?;
        let binding = requirement.definition().parts().source.binding.parts();
        let scope = (
            [
                binding.scope.as_str(),
                &binding.scope_entity,
                &binding.scope_aspect,
                &binding.scope_field,
                binding.scope_value.as_str(),
            ],
            binding.scope_resolution,
            binding.scope_writable,
        );
        if scopes
            .insert((invariant, execution_point), scope)
            .is_some_and(|previous| previous != scope)
        {
            return Err(denied());
        }
        if source_audience(source)
            .iter()
            .any(|target| !read.contains(target) || !affected.contains(target))
        {
            return Err(denied());
        }
    }
    Ok(())
}

fn rule_audience<'a>(
    members: &'a [Member],
    invariant: &str,
    point: Point,
) -> Option<(&'a [Target], &'a [Target])> {
    members.iter().find_map(|member| match member {
        Member::ApplicationInvariant {
            invariant: rule,
            execution_point,
            read_closure,
            applicability,
            ..
        } if rule == invariant && *execution_point == point => {
            Some((read_closure.as_slice(), applicability.as_slice()))
        }
        _ => None,
    })
}

fn source_query<'a>(
    members: &'a [Member],
    requirement: &Requirement,
) -> Option<&'a ErasedApplicationQueryDefinition> {
    let index = members.iter().find_map(|member| match member {
        Member::ManagedIndex { definition } if definition.parts().name == requirement.index() => {
            Some(definition)
        }
        _ => None,
    })?;
    if index != requirement.definition() {
        return None;
    }
    members.iter().find_map(|member| match member {
        Member::ApplicationQuery { definition }
            if definition.name() == index.parts().source.binding.parts().query_name =>
        {
            Some(definition)
        }
        _ => None,
    })
}

fn source_audience(source: &ErasedApplicationQueryDefinition) -> BTreeSet<Target> {
    let mut audience = BTreeSet::from([
        Target::Entity(source.scope_entity().to_owned()),
        Target::Entity(source.root_entity().to_owned()),
    ]);
    for path in source.root_paths() {
        for step in path.steps() {
            audience.insert(Target::Entity(step.from().to_owned()));
            audience.insert(Target::Entity(step.to().to_owned()));
            audience.insert(Target::Relation(step.relation().to_owned()));
        }
        audience.extend(
            path.guards()
                .iter()
                .map(|guard| Target::Entity(guard.entity().to_owned())),
        );
    }
    shape_audience(source.result_shape(), &mut audience);
    audience.extend(
        source
            .predicates()
            .iter()
            .map(|predicate| Target::Entity(predicate.field().0.to_owned())),
    );
    audience
}

fn shape_audience(shape: &ApplicationQueryResultShape, audience: &mut BTreeSet<Target>) {
    audience.insert(Target::Entity(shape.root_entity().to_owned()));
    for relation in shape.relations() {
        audience.insert(Target::Relation(relation.relation().to_owned()));
        audience.insert(Target::Entity(relation.from().to_owned()));
        audience.insert(Target::Entity(relation.to().to_owned()));
        shape_audience(relation.nested_shape(), audience);
    }
}
