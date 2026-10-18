use crate::application::WorthQueryDeclarationEntryContributionCategoryFamily;
use crate::orchestration_inventory::WorthQueryOrchestrationContributionCompatibility;

#[test]
fn declaration_scoped_contribution_compatibility_canonicalizes_family_order() {
    let left = WorthQueryOrchestrationContributionCompatibility::declaration_scoped(vec![
        WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
        WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
        WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
    ]);
    let right = WorthQueryOrchestrationContributionCompatibility::declaration_scoped(vec![
        WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
        WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
    ]);

    assert_eq!(left.supported_families(), right.supported_families());
    assert_eq!(left.as_digest_fragment(), right.as_digest_fragment());
}
