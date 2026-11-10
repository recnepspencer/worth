use super::lane_builders::{
    certified_lane, certified_lane_with_basis, certified_lane_with_context,
    certified_lane_with_scale,
};
use super::{MilestoneNineOneCertificationRow, MilestoneNineOnePerturbationClass};
use crate::harness::certification::{HostileExpectation, ParityAnchor};
use crate::live::LiveQueryFamily;
use crate::subscription::QuerySubscriptionConstructionSource;
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn canonical_rows() -> Vec<MilestoneNineOneCertificationRow> {
    vec![
        MilestoneNineOneCertificationRow {
            row_name: "detail-direct-scope-template-saved-facade-parity",
            perturbation_class: MilestoneNineOnePerturbationClass::ConstructionSourceParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::TemplateInstantiated,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::SavedExactReuse,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "direct-scope-template-saved-subscription-parity",
            perturbation_class:
                MilestoneNineOnePerturbationClass::RepresentativeConstructionSourceParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::ScopeExpanded,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::TemplateInstantiated,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "facade-helper-subscription-parity",
            perturbation_class: MilestoneNineOnePerturbationClass::FacadeHelperParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::SavedExactReuse,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "collection-table-bridge-lowering-parity",
            perturbation_class: MilestoneNineOnePerturbationClass::CollectionBridgeLoweringParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::ScopeExpanded,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "grouped-query-meaning-shares-collection-bridge-family",
            perturbation_class: MilestoneNineOnePerturbationClass::GroupedQueryMeaning,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::KanbanGrouped),
                QuerySubscriptionConstructionSource::TemplateInstantiated,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "inspector-query-meaning-shares-detail-bridge-family",
            perturbation_class: MilestoneNineOnePerturbationClass::InspectorQueryMeaning,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                Some(LiveViewShapeFamily::Detail),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::Detail,
                Some(LiveViewShapeFamily::InspectorDetailFocused),
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                Some(LiveViewShapeFamily::InspectorDetailFocused),
                QuerySubscriptionConstructionSource::SavedExactReuse,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "bounded-materialization-relation-scope-lowering",
            perturbation_class: MilestoneNineOnePerturbationClass::BoundedMaterializationLowering,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::BoundedMaterialization,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane(
                LiveQueryFamily::BoundedMaterialization,
                None,
                QuerySubscriptionConstructionSource::ScopeExpanded,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::BoundedMaterialization,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "activation-certification-source-binding",
            perturbation_class:
                MilestoneNineOnePerturbationClass::ActivationCertificationSourceBinding,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane_with_basis(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::Direct,
                crate::subscription::QuerySubscriptionBasisPosture::BranchHead,
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "basis-request-binds-policy-tenant-meaning",
            perturbation_class: MilestoneNineOnePerturbationClass::BasisRequestPolicyTenantBinding,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::SavedExactReuse,
                "policy-alpha",
                "tenant-alpha",
                "relationship-proof",
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::SavedExactReuse,
                "policy-alpha",
                "tenant-beta",
                "relationship-proof",
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
                "policy-alpha",
                "tenant-beta",
                "relationship-proof",
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "relationship-proof-binds-subscription-meaning",
            perturbation_class: MilestoneNineOnePerturbationClass::RelationshipProofBinding,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::SavedExactReuse,
                "policy-alpha",
                "tenant-alpha",
                "relationship-proof-alpha",
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::SavedExactReuse,
                "policy-alpha",
                "tenant-alpha",
                "relationship-proof-beta",
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane_with_context(
                LiveQueryFamily::Detail,
                None,
                QuerySubscriptionConstructionSource::FacadeLive,
                "policy-alpha",
                "tenant-alpha",
                "relationship-proof-beta",
            )
            .expect("fixture digest must fit"),
        },
        MilestoneNineOneCertificationRow {
            row_name: "scale-slope-row-count-only-honesty",
            perturbation_class: MilestoneNineOnePerturbationClass::ScaleSlopeHonesty,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: certified_lane(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
            )
            .expect("fixture digest must fit"),
            hostile_lane: certified_lane_with_scale(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::Direct,
                [2, 20, 200],
            )
            .expect("fixture digest must fit"),
            parity_lane: certified_lane_with_scale(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                QuerySubscriptionConstructionSource::FacadeLive,
                [3, 30, 300],
            )
            .expect("fixture digest must fit"),
        },
    ]
}
