use std::marker::PhantomData;

use crate::ordinary::inspection::{
    declare, WorthQueryInspectionContext, WorthQueryInspectionDeclaration,
    WorthQueryInspectionOutcome, WorthQueryInspectionRequest,
};
use crate::runtime::WorthQueryWorkspace;

use super::super::{
    WorthQueryInstalledDomainCapabilityKind, WorthQueryInstalledDomainCapabilityStop,
    WorthQueryInstalledDomainExecutionDrift, WorthQueryInstalledDomainExecutionReceipt,
};
use super::WorthQueryInstalledDomainReadCompletion;

pub struct WorthQueryInstalledDomainInspectionDeclaration<D> {
    source: WorthQueryInstalledDomainReadCompletion<D>,
    declaration: WorthQueryInspectionDeclaration,
}

impl<D> WorthQueryInstalledDomainReadCompletion<D> {
    pub fn inspect(self) -> WorthQueryInstalledDomainInspectionDeclaration<D> {
        let declaration = declare(self.completion());
        WorthQueryInstalledDomainInspectionDeclaration {
            source: self,
            declaration,
        }
    }
}

impl<D> WorthQueryInstalledDomainInspectionDeclaration<D> {
    pub fn with_rich_inspection(mut self) -> Self {
        self.declaration = self.declaration.with_rich_inspection();
        self
    }

    pub fn using(
        self,
        context: WorthQueryInspectionContext,
    ) -> WorthQueryInstalledDomainInspectionRequest<D> {
        WorthQueryInstalledDomainInspectionRequest {
            source: self.source,
            request: self.declaration.using(context),
        }
    }
}

pub struct WorthQueryInstalledDomainInspectionRequest<D> {
    source: WorthQueryInstalledDomainReadCompletion<D>,
    request: WorthQueryInspectionRequest,
}

impl<D: 'static> WorthQueryInstalledDomainInspectionRequest<D> {
    pub fn run(
        self,
        workspace: &WorthQueryWorkspace,
    ) -> Result<
        WorthQueryInstalledDomainInspectionOutcome<D>,
        WorthQueryInstalledDomainCapabilityStop<WorthQueryInstalledDomainExecutionDrift>,
    > {
        let refusal_witness = self.source.witness().clone();
        let refusal_declaration = self.source.declaration_identity().clone();
        let identity_refused = |cause| {
            WorthQueryInstalledDomainCapabilityStop::identity_refused(
                refusal_witness.clone(),
                WorthQueryInstalledDomainCapabilityKind::Inspection,
                Some(refusal_declaration.clone()),
                cause,
            )
        };

        WorthQueryInstalledDomainExecutionDrift::validate::<D>(self.source.witness(), workspace)
            .map_err(|drift| {
                WorthQueryInstalledDomainCapabilityStop::new(
                    self.source.witness().clone(),
                    WorthQueryInstalledDomainCapabilityKind::Inspection,
                    self.source.declaration_identity().clone(),
                    drift,
                )
            })?;
        let outcome = self.request.run(workspace).map_err(identity_refused)?;
        let operational_identity = outcome
            .settled()
            .map(|completion| completion.receipt().identity().clone())
            .unwrap_or_else(|| self.source.receipt().operational_identity().clone());
        let receipt = WorthQueryInstalledDomainExecutionReceipt::new(
            self.source.witness().clone(),
            WorthQueryInstalledDomainCapabilityKind::Inspection,
            self.source.declaration_identity().clone(),
            self.source.receipt().basis_identity().clone(),
            operational_identity,
        );
        Ok(WorthQueryInstalledDomainInspectionOutcome {
            outcome,
            receipt,
            marker: PhantomData,
        })
    }
}

pub struct WorthQueryInstalledDomainInspectionOutcome<D> {
    outcome: WorthQueryInspectionOutcome,
    receipt: WorthQueryInstalledDomainExecutionReceipt,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryInstalledDomainInspectionOutcome<D> {
    pub fn outcome(&self) -> &WorthQueryInspectionOutcome {
        &self.outcome
    }

    pub fn receipt(&self) -> &WorthQueryInstalledDomainExecutionReceipt {
        &self.receipt
    }

    pub fn into_outcome(self) -> WorthQueryInspectionOutcome {
        self.outcome
    }
}
