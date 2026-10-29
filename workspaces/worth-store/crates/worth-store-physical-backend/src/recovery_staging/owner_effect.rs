use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct NonCurrentStagingMutationScope<'a> {
    root: &'a Path,
    staging_plan_fingerprint: [u8; 32],
}

impl<'a> NonCurrentStagingMutationScope<'a> {
    pub(crate) const fn new(root: &'a Path, staging_plan_fingerprint: [u8; 32]) -> Self {
        Self {
            root,
            staging_plan_fingerprint,
        }
    }

    pub const fn root(self) -> &'a Path {
        self.root
    }

    pub const fn staging_plan_fingerprint(self) -> [u8; 32] {
        self.staging_plan_fingerprint
    }
}

pub trait NonCurrentStagingOwnerEffect {
    fn effect_fingerprint(&self) -> [u8; 32];
}

#[derive(Debug)]
pub enum NonCurrentStagingOwnerExecutionDenial<OwnerDenial> {
    Backend(super::NonCurrentStagingExecutionDenial),
    Owner(OwnerDenial),
}

impl<OwnerDenial> From<super::NonCurrentStagingExecutionDenial>
    for NonCurrentStagingOwnerExecutionDenial<OwnerDenial>
{
    fn from(value: super::NonCurrentStagingExecutionDenial) -> Self {
        Self::Backend(value)
    }
}

impl super::PhysicalRecoveryStagingOwner {
    pub fn execute_lowered_guarded_with_owner_effect<Effect, OwnerDenial>(
        plan: super::LoweredNonCurrentStagingPlan,
        mut continuation: impl FnMut(super::NonCurrentStagingBoundary) -> bool,
        owner_effect: impl FnOnce(NonCurrentStagingMutationScope<'_>) -> Result<Effect, OwnerDenial>,
    ) -> Result<
        (super::NonCurrentStagingExecutionReceipt, Effect),
        NonCurrentStagingOwnerExecutionDenial<OwnerDenial>,
    >
    where
        Effect: NonCurrentStagingOwnerEffect,
    {
        let mut effect_slot = None;
        let mut owner_denial = None;
        let mut owner_effect = Some(owner_effect);
        match drive_owner_effect(plan, &mut continuation, &mut |scope| match owner_effect
            .take()
            .expect("staging owner effect runs once")(
            scope
        ) {
            Ok(effect) => {
                let fingerprint = effect.effect_fingerprint();
                effect_slot = Some(effect);
                Ok(fingerprint)
            }
            Err(denial) => {
                owner_denial = Some(denial);
                Err(())
            }
        }) {
            Ok(receipt) => Ok((
                receipt,
                effect_slot.expect("staging owner effect stores its value"),
            )),
            Err(OwnerEffectDriveFailure::Backend(denial)) => {
                Err(NonCurrentStagingOwnerExecutionDenial::Backend(denial))
            }
            Err(OwnerEffectDriveFailure::Owner) => {
                Err(NonCurrentStagingOwnerExecutionDenial::Owner(
                    owner_denial.expect("staging owner denial is stored"),
                ))
            }
        }
    }
}

enum OwnerEffectDriveFailure {
    Backend(super::NonCurrentStagingExecutionDenial),
    Owner,
}

/// One compiled staging copy for every owner effect.
#[inline(never)]
fn drive_owner_effect(
    plan: super::LoweredNonCurrentStagingPlan,
    continuation: &mut dyn FnMut(super::NonCurrentStagingBoundary) -> bool,
    owner_effect: &mut dyn FnMut(NonCurrentStagingMutationScope<'_>) -> Result<[u8; 32], ()>,
) -> Result<super::NonCurrentStagingExecutionReceipt, OwnerEffectDriveFailure> {
    let copied = super::execution::copy_lowered(&plan, continuation)
        .map_err(OwnerEffectDriveFailure::Backend)?;
    super::execution::admit_continuation(
        continuation,
        super::NonCurrentStagingBoundary::OwnerEffect,
    )
    .map_err(OwnerEffectDriveFailure::Backend)?;
    let fingerprint = owner_effect(NonCurrentStagingMutationScope::new(
        plan.binding().staging_root(),
        plan.binding().fingerprint(),
    ))
    .map_err(|()| OwnerEffectDriveFailure::Owner)?;
    super::execution::admit_continuation(
        continuation,
        super::NonCurrentStagingBoundary::OwnerEffectApplied,
    )
    .map_err(OwnerEffectDriveFailure::Backend)?;
    super::execution::finalize_lowered(plan, copied, fingerprint, continuation)
        .map_err(OwnerEffectDriveFailure::Backend)
}
