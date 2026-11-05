//! First-use eligibility compares semantic source meaning on the exact target.
use super::disclosure::WorthQueryCorrectionDerivationDenial as Denial;
use super::*;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation;
use worth_query_declaration::facade::application_aftermath::ApplicationInverseBinding;

/// The compatibility owner's exact handoff into correction preparation.
///
/// This moves existing ancestry and interpretation custody; it retains no Native
/// reader and does not promise that the target will still be current at use.
pub(super) struct CheckedCorrectionSource<'check, 'target, 'source, Schema, C, Binding> {
    target: &'check WorthQuerySelectedProductOperation<'target, Schema>,
    source: &'check WorthQuerySelectedCorrectionSource<'source, Schema>,
    ancestry: ProductBranchAncestorSelection,
    interpretation: WorthQueryProgramSupportInterpretation,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
    binding: std::marker::PhantomData<fn() -> (C, Binding)>,
}

impl<'check, 'target, 'source, Schema, C, Binding>
    CheckedCorrectionSource<'check, 'target, 'source, Schema, C, Binding>
{
    pub(super) fn into_parts(
        self,
    ) -> (
        &'check WorthQuerySelectedProductOperation<'target, Schema>,
        &'check WorthQuerySelectedCorrectionSource<'source, Schema>,
        ProductBranchAncestorSelection,
        WorthQueryProgramSupportInterpretation,
        Option<crate::equality_type_id::EqualityTypeId>,
    ) {
        (
            self.target,
            self.source,
            self.ancestry,
            self.interpretation,
            self.required_root,
        )
    }
}

fn require_selected_ancestry<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    ancestry: &ProductBranchAncestorSelection,
) -> Result<(), Denial> {
    if !std::ptr::eq(target.application(), source.application())
        || ancestry.source().identity() != source.ancestry().source().identity()
        || ancestry.target() != target.product().observation()
    {
        return Err(Denial::ForeignSelection);
    }
    Ok(())
}

pub(super) fn require_compatible_source<'check, 'target, 'source, Schema, C>(
    target: &'check WorthQuerySelectedProductOperation<'target, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &'check WorthQuerySelectedCorrectionSource<'source, Schema>,
    ancestry: ProductBranchAncestorSelection,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<
    CheckedCorrectionSource<
        'check,
        'target,
        'source,
        Schema,
        C,
        ApplicationInverseBinding<Schema, C>,
    >,
    Denial,
>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    require_selected_ancestry(target, source, &ancestry)?;
    let interpretation = require_target_program::<Schema, C>(target, source, required_root)?;
    if !matches!(
        super::material::mechanism(source)?,
        worth_query_installation::facade::InstalledCorrectionMechanism::Compensation(_)
    ) {
        let facts = source
            .occurrence()
            .commit_evidence()
            .rebased_source_facts()
            .map_err(|_| Denial::SemanticMaterialUnavailable)?;
        super::restored_support::require(target, phase, source, &facts)?;
    }
    // Compensation uses the historical effect/input and fresh ordinary reads;
    // it deliberately does not claim reverse-restoration eligibility.
    Ok(CheckedCorrectionSource {
        target,
        source,
        ancestry,
        interpretation,
        required_root,
        binding: std::marker::PhantomData,
    })
}

fn require_target_program<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<WorthQueryProgramSupportInterpretation, Denial>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    let interpretation = require_action_program::<Schema, ApplicationInverseBinding<Schema, C>>(
        target,
        source,
        required_root,
    )?;
    let inspection = target
        .inspect_selected_program()
        .map_err(Denial::TargetProgram)?;
    let support = target
        .application()
        .installed_program_support()
        .ok_or(Denial::UnsupportedTargetProgram)?;
    let record = support
        .native_program(inspection.revision())
        .ok_or(Denial::UnsupportedTargetProgram)?;
    if required_root.is_none()
        && record.requires_output_source(crate::equality_type_id::EqualityTypeId::of::<C::Source>())
    {
        return Err(Denial::RequiredOutputReconciliation);
    }
    Ok(interpretation)
}

pub(super) fn require_reapplication_program<'check, 'target, 'source, Schema, C>(
    target: &'check WorthQuerySelectedProductOperation<'target, Schema>,
    source: &'check WorthQuerySelectedCorrectionSource<'source, Schema>,
    ancestry: ProductBranchAncestorSelection,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<CheckedCorrectionSource<'check, 'target, 'source, Schema, C, C::Source>, Denial>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
{
    require_selected_ancestry(target, source, &ancestry)?;
    let retained = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .retained_governed_input_carrier()
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    let association = retained
        .correction()
        .ok_or(Denial::UnsupportedSourceMeaning)?
        .association();
    if association.declaration().reapplication_source()
        != Some(C::Source::descriptor().description().binding_identity())
    {
        return Err(Denial::ReapplicationUnavailable);
    }
    let interpretation =
        require_action_program::<Schema, C::Source>(target, source, required_root)?;
    Ok(CheckedCorrectionSource {
        target,
        source,
        ancestry,
        interpretation,
        required_root,
        binding: std::marker::PhantomData,
    })
}

fn require_action_program<Schema, Binding>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<WorthQueryProgramSupportInterpretation, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let original = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .original_program()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let inspection = target
        .inspect_selected_program()
        .map_err(Denial::TargetProgram)?;
    let support = target
        .application()
        .installed_program_support()
        .ok_or(Denial::UnsupportedTargetProgram)?;
    let record = support
        .native_program(inspection.revision())
        .ok_or(Denial::UnsupportedTargetProgram)?;
    let declared = original
        .action_inventory()
        .resolve_unique_binding::<Schema, Binding>()
        .map_err(Denial::Action)?;
    let action = record
        .installed
        .action_inventory()
        .resolve_declared_action::<Schema, Binding>(declared)
        .map_err(Denial::Action)?;
    let requirements = support
        .adoption_requirements(
            target.application().installed_schema(),
            original.revision(),
            inspection.revision(),
        )
        .map_err(|denial| Denial::ProgramComparison(Box::new(denial)))?;
    if !requirements.semantically_equivalent() {
        return Err(Denial::ProgramMeaningChanged(Box::new(requirements)));
    }
    let action_binding = crate::equality_type_id::EqualityTypeId::of::<Binding>();
    if required_root.is_none() && record.requires_output_source(action_binding) {
        return Err(Denial::RequiredOutputReconciliation);
    }
    if let Some(root) = required_root {
        let delivery = action.delivery().and_then(|delivery| delivery.output());
        if !delivery.is_some_and(|delivery| {
            !delivery.is_discovered() && delivery.root_type() == root.into_external()
        }) {
            return Err(Denial::Action(worth_query_installation::facade::WorthQueryActionResolutionDenial::DeliveryMismatch));
        }
        if !record.root_graph_types.contains(&root)
            || !record.requires_output_source(action_binding)
        {
            return Err(Denial::UnsupportedTargetProgram);
        }
    }
    if !record.declares_action(action_binding) {
        return Err(Denial::UnsupportedTargetProgram);
    }
    support
        .retain_interpretation(inspection.revision())
        .ok_or(Denial::UnsupportedTargetProgram)
}
