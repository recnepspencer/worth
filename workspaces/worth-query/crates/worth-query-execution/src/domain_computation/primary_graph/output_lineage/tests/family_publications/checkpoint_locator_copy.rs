//! Descriptive locator selection/copy controls; no producer freshness claim.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::merge_native_checkpoint_priors,
    application_output_demand::{
        WorthQueryAcceptedOutputCheckpointIdentity as Identity,
        WorthQueryAcceptedOutputCheckpointPosture as Posture,
    },
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        native_prior_checkpoint::NativePriorCheckpointLocator,
    },
};
use worth_relational::facade::mvcc::CompanionPreflightBudget;

fn admission(work: u64, bytes: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: bytes,
    })
}

fn court() -> Court {
    let mut court = Court::new();
    court.publish::<Initial>(
        9,
        Some(7),
        2,
        WorthQueryApplicationOutputPosture::Create,
        false,
    );
    let cell = court
        .lineage
        .by_source
        .get_mut(&court.source)
        .unwrap()
        .get_mut(&court.coordinate.occurrence)
        .unwrap()
        .get_mut(&9)
        .unwrap();
    let recorded = Arc::get_mut(&mut cell[0]).unwrap().get_mut().unwrap();
    recorded.native_prior_checkpoint = Some(NativePriorCheckpointLocator {
        producer: "fixture-producer".into(),
        source: [11; 32],
    });
    recorded.correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            TypeId::of::<Initial>(),
            TypeId::of::<()>(),
            BTreeSet::new(),
            expected_roles(),
            |_| Some(TypeId::of::<()>()),
        )
        .unwrap(),
    );
    court
}

fn expected_roles() -> Vec<WorthQueryCheckpointOutputRole> {
    vec![
        WorthQueryCheckpointOutputRole {
            role: "auxiliary".into(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "auxiliary-entity".into(),
            entity: entity(3),
        },
        WorthQueryCheckpointOutputRole {
            role: "output".into(),
            posture: WorthQueryApplicationOutputPosture::Create,
            entity_name: "fixture".into(),
            entity: entity(2),
        },
    ]
}

fn expected(court: &Court) -> Identity {
    Identity {
        producer: "fixture-producer".into(),
        posture: Posture::Performed,
        source: [11; 32],
        scope: court.source.scope,
        source_partition: [7; 32],
        producer_dependency: None,
        idempotency_key: [2; 32],
        resources: None,
        roles: expected_roles(),
        producer_facts: None,
        producer_fact_wire_version: 0,
    }
}

fn select<'a>(
    court: &'a Court,
    admission: &mut InvalidationEditAdmission,
) -> Vec<super::super::super::family_selection::NativePriorCheckpointOutput<'a>> {
    court
        .lineage
        .checkpoint_prior_outputs(
            court.source.runtime_authority,
            &court.source.schema,
            court.coordinate.occurrence,
            court.coordinate.generation,
            admission,
        )
        .unwrap()
}

#[test]
fn accepted_native_head_skips_only_the_discarded_full_locator_copy() {
    let court = court();
    let mut admitted = admission(u64::MAX, u64::MAX);
    let priors = select(&court, &mut admitted);
    let before = (admitted.charged_work(), admitted.charged_bytes());
    let mut accepted = expected(&court);
    accepted.producer_facts = Some(vec![7, 8, 9]);
    accepted.producer_fact_wire_version = 4;
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(accepted.clone(), None, Some(TypeId::of::<Initial>()))],
        std::iter::empty(),
        priors,
        &mut admitted,
    )
    .unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].0, accepted);
    assert!(merged[0].1.is_none());
    let matched = (admitted.charged_work(), admitted.charged_bytes());
    assert!(
        matched.0 > before.0,
        "actual merge/index work remains charged"
    );
    assert!(
        matched.1 > before.1,
        "borrowed indexes retain scratch admission"
    );
    let mut rejected = accepted;
    rejected.idempotency_key = [3; 32];
    let mut charged = admission(u64::MAX, u64::MAX);
    let priors = select(&court, &mut charged);
    let merged = merge_native_checkpoint_priors(
        &court.lineage,
        vec![(rejected, None, Some(TypeId::of::<Initial>()))],
        std::iter::empty(),
        priors,
        &mut charged,
    )
    .unwrap();
    assert_eq!(merged[0].0, expected(&court));
    assert!(
        charged.charged_work() > matched.0,
        "rejecting the accepted packet must restore full locator copy work"
    );
}

#[test]
fn absent_or_rejected_accepted_rows_preserve_the_complete_native_locator() {
    let court = court();
    let expected = expected(&court);
    for drift in 0..6 {
        let mut candidate = expected.clone();
        let mut binding = TypeId::of::<Initial>();
        match drift {
            0 => {}
            1 => binding = TypeId::of::<Preserve>(),
            2 => candidate.idempotency_key = [3; 32],
            3 => candidate.source = [12; 32],
            4 => candidate.producer = "other-producer".into(),
            5 => candidate.roles[1].entity = entity(4),
            _ => unreachable!(),
        }
        let rows = if drift == 0 {
            Vec::new()
        } else {
            vec![(candidate, None, Some(binding))]
        };
        let mut admitted = admission(u64::MAX, u64::MAX);
        let priors = select(&court, &mut admitted);
        let before = (admitted.charged_work(), admitted.charged_bytes());
        let merged = merge_native_checkpoint_priors(
            &court.lineage,
            rows,
            std::iter::empty(),
            priors,
            &mut admitted,
        )
        .unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0, expected);
        assert!(merged[0].1.is_none());
        assert!(admitted.charged_work() > before.0);
        assert!(admitted.charged_bytes() > before.1);
    }
}

#[test]
fn required_native_locator_still_refuses_its_copy_work_and_scratch() {
    let court = court();
    let mut quote = admission(u64::MAX, u64::MAX);
    let priors = select(&court, &mut quote);
    let before = (quote.charged_work(), quote.charged_bytes());
    let copied = priors[0].materialize_identity(&mut quote).unwrap().unwrap();
    assert_eq!(copied, expected(&court));
    let copy = (
        quote.charged_work() - before.0,
        quote.charged_bytes() - before.1,
    );
    let mut measured = admission(u64::MAX, u64::MAX);
    let priors = select(&court, &mut measured);
    merge_native_checkpoint_priors(
        &court.lineage,
        Vec::new(),
        std::iter::empty(),
        priors,
        &mut measured,
    )
    .unwrap();
    for (work, bytes, phase) in [
        (
            measured.charged_work() - copy.0,
            u64::MAX,
            "native locator copy work",
        ),
        (
            u64::MAX,
            measured.charged_bytes() - copy.1,
            "native locator scratch",
        ),
    ] {
        let mut admitted = admission(work, bytes);
        let priors = select(&court, &mut admitted);
        let denial = merge_native_checkpoint_priors(
            &court.lineage,
            Vec::new(),
            std::iter::empty(),
            priors,
            &mut admitted,
        )
        .err()
        .expect("the complete needed locator remains charged");
        let crate::domain_computation::primary_graph::application_checkpoint::WorthQueryCheckpointCaptureDenial::Durability(denial) = denial else {
            panic!("selector admission remains a durability capture denial");
        };
        assert!(denial.detail.contains(phase));
    }
}
