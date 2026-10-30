//! Rich source/producer intent delegates to the same connected request and terminal.
use super::super::super::{
    result::WorthQueryMaterializedApplicationInspection,
    rich::WorthQueryApplicationMaterializationRequest,
};
use super::*;
pub struct WorthQueryMaterializedConnectedInspectionRequest<
    'a,
    'p,
    'r,
    'o,
    's,
    Schema,
    Binding,
    Disclosure,
>(WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>);
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
{
    pub fn materialize(
        mut self,
        detail: WorthQueryApplicationMaterializationRequest,
    ) -> WorthQueryMaterializedConnectedInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    > {
        self.rich = Some(detail);
        WorthQueryMaterializedConnectedInspectionRequest(self)
    }
}
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryEvaluatedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
{
    pub fn materialize(
        self,
        detail: WorthQueryApplicationMaterializationRequest,
    ) -> WorthQueryMaterializedConnectedInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    > {
        self.0.materialize(detail)
    }
}
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryMaterializedConnectedInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
{
    pub fn with_allocation_policy<'funding>(
        self,
        policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryMaterializedConnectedInspectionRequest<
        'funding,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
    where
        'a: 'funding,
    {
        WorthQueryMaterializedConnectedInspectionRequest(
            WorthQueryConnectedReferenceOnlyInspectionRequest(self.0)
                .with_allocation_policy(policy)
                .0,
        )
    }
}
impl<'o, Schema, Binding, Disclosure>
    WorthQueryMaterializedConnectedInspectionRequest<
        '_,
        '_,
        '_,
        'o,
        '_,
        Schema,
        Binding,
        Disclosure,
    >
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    Disclosure: ApplicationSourceDisclosureBinding<Schema>,
{
    pub fn execute_with_diagnostics<QDC, QDO, QDI, SDC, SDO, SDI>(
        self,
        qd: Access<Schema, QDC, QDO, QDI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<
        WorthQueryMaterializedApplicationInspection<'o>,
        WorthQueryApplicationInspectionDenial,
    >
    where
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        self.0
            .execute_with_diagnostics_core(qd, sd)
            .map(SourceInspectionResult::rich)
    }
    pub fn execute_governed<QMC, QMO, QMI, QDC, QDO, QDI, SMC, SMO, SMI, SDC, SDO, SDI>(
        self,
        qm: Access<Schema, QMC, QMO, QMI>,
        qd: Access<Schema, QDC, QDO, QDI>,
        sm: Access<Schema, SMC, SMO, SMI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<
        WorthQueryMaterializedApplicationInspection<'o>,
        WorthQueryApplicationInspectionDenial,
    >
    where
        QMI: ApplicationCapabilityRequest<Schema, QMC, Scope = QueryScope<Schema, Binding>>,
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SMI: ApplicationCapabilityRequest<Schema, SMC, Scope = SourceScope<Schema, Disclosure>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        self.0
            .execute_governed_core(qm, qd, sm, sd)
            .map(SourceInspectionResult::rich)
    }
}
