use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
};

use super::super::capability_registry::WorthQueryInstalledCapabilityPlan;

pub(super) fn lifecycle_decision_reads(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationDecisionReadTarget> {
    let elevation = installed.contract().elevation().definition().unwrap();
    let review = elevation.review();
    let mut reads = [
        elevation.identity(),
        elevation.reason(),
        elevation.status(),
        elevation.validity().not_before(),
        elevation.validity().not_after(),
        review.identity(),
        review.kind().field(),
        review.status(),
    ]
    .into_iter()
    .map(|field| ApplicationOperationDecisionReadTarget::Field {
        entity: field.entity().to_string(),
        aspect: field.aspect().to_string(),
        field: field.field().to_string(),
    })
    .collect::<Vec<_>>();
    let mut relations = vec![
        elevation.requester(),
        elevation.approver(),
        elevation.grant(),
        review.relation(),
        review.scope(),
        review.reviewer(),
    ];
    relations.extend(elevation.resource_relation());
    reads.extend(relations.into_iter().map(|relation| {
        ApplicationOperationDecisionReadTarget::Relation {
            relation: relation.relation().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        }
    }));
    reads
}

pub(super) fn close_decision_reads(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationDecisionReadTarget> {
    let mut reads = lifecycle_decision_reads(installed);
    let elevation = installed.contract().elevation().definition().unwrap();
    reads.push(field_read_target(elevation.closed_at()));
    reads
}

pub(super) fn review_decision_reads(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationDecisionReadTarget> {
    let mut reads = lifecycle_decision_reads(installed);
    let elevation = installed.contract().elevation().definition().unwrap();
    reads.push(field_read_target(elevation.review().reviewed_at()));
    reads
}

pub(super) fn approval_program_targets(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationProgramTarget> {
    let elevation = installed.contract().elevation().definition().unwrap();
    let mut targets = vec![
        write_target(elevation.status()),
        link_target(elevation.approver()),
    ];
    append_lifecycle_effect(&mut targets, elevation.lifecycle().approve());
    targets
}

pub(super) fn close_program_targets(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationProgramTarget> {
    let elevation = installed.contract().elevation().definition().unwrap();
    let mut targets = vec![
        write_target(elevation.status()),
        write_target(elevation.closed_at()),
    ];
    append_lifecycle_effect(&mut targets, elevation.lifecycle().revoke());
    targets
}

pub(super) fn review_program_targets(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Vec<ApplicationOperationProgramTarget> {
    let review = installed
        .contract()
        .elevation()
        .definition()
        .unwrap()
        .review();
    let mut targets = vec![
        write_target(review.status()),
        write_target(review.reviewed_at()),
        link_target(review.reviewer()),
    ];
    append_lifecycle_effect(
        &mut targets,
        installed
            .contract()
            .elevation()
            .definition()
            .unwrap()
            .lifecycle()
            .complete_review(),
    );
    targets
}

fn append_lifecycle_effect(
    targets: &mut Vec<ApplicationOperationProgramTarget>,
    transition: &worth_query_declaration::facade::application_capability::ApplicationCapabilityTransitionBinding,
) {
    targets.extend(transition.lifecycle_effect().map(|effect| {
        ApplicationOperationProgramTarget::Emit {
            effect: effect.effect().to_string(),
        }
    }));
}

fn write_target(
    field: &worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding,
) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Write {
        entity: field.entity().to_string(),
        aspect: field.aspect().to_string(),
        field: field.field().to_string(),
    }
}

fn field_read_target(
    field: &worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding,
) -> ApplicationOperationDecisionReadTarget {
    ApplicationOperationDecisionReadTarget::Field {
        entity: field.entity().to_string(),
        aspect: field.aspect().to_string(),
        field: field.field().to_string(),
    }
}

fn link_target(
    relation: &worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding,
) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Link {
        relation: relation.relation().to_string(),
        from: relation.from().to_string(),
        to: relation.to().to_string(),
    }
}
