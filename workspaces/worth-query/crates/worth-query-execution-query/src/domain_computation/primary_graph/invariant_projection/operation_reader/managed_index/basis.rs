//! The committed basis or live coupled assembly that issued this decision.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::AppliedManagedIndexRead;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::read_basis::NativeProjectionRead;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::runtime::RelationalRuntime;

pub(super) enum DecisionReadBasis<'r> {
    Committed(&'r RelationalRuntime),
    Applied(AppliedManagedIndexRead<'r>),
}

impl<'r> DecisionReadBasis<'r> {
    pub(super) fn admit<'runtime: 'r, Schema: ApplicationSchema, Operation>(
        reader: &WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            'runtime,
            Schema,
            Operation,
        >,
        access: &'r SourceQueryReaderAccess<'_, Schema>,
    ) -> Result<Self, Failure> {
        let selected = access.shared.selected();
        let product = selected.product().observation();
        if reader.reader().selected_product_occurrence() != Some(product.lifecycle_incarnation())
            || reader.reader().selected_product_generation()
                != Some(product.reference_generation().get())
        {
            return Err(Failure::ForeignProjection);
        }
        let native = reader.reader().native();
        let basis = match *native {
            NativeProjectionRead::Committed(runtime) => Self::Committed(runtime),
            NativeProjectionRead::Applied {
                before,
                candidate,
                own_outputs,
            } => {
                // The candidate must be built on exactly the borrowed basis.
                if candidate
                    .candidate_identity()
                    .proposal()
                    .branch_observation()
                    != selected.product().relational_basis_descriptor().reference()
                {
                    return Err(Failure::ForeignProjection);
                }
                Self::Applied(AppliedManagedIndexRead {
                    assembly: access.applied_managed_sources.ok_or_else(unavailable)?,
                    participants: access.applied_participants.ok_or_else(unavailable)?,
                    before,
                    candidate,
                    own: own_outputs,
                })
            }
        };
        let snapshot = reader.reader().snapshot();
        let borrowed = selected.application_basis().snapshot_handle();
        let same = |runtime: &RelationalRuntime| {
            let truth = runtime.read_truth();
            match (
                truth.positioned_snapshot(snapshot),
                truth.positioned_snapshot(borrowed),
            ) {
                (Ok(original), Ok(borrowed)) => Ok(original == borrowed),
                _ => Err(Failure::ForeignProjection),
            }
        };
        let aligned = match &basis {
            Self::Committed(runtime) => same(runtime)?,
            Self::Applied(_) => {
                let graph = access
                    .application
                    .runtime
                    .primary_graph()
                    .ok_or_else(unavailable)?;
                ApplicationQueryNativeAccess::acquire().with_runtime(
                    graph,
                    reader.reader().execution(),
                    || Failure::ForeignProjection,
                    same,
                )?
            }
        };
        if !aligned {
            return Err(Failure::ForeignProjection);
        }
        Ok(basis)
    }

    /// The Native transport the read runs on: the reader's own committed
    /// borrow, or a fresh acquisition (an applied reader holds no runtime).
    pub(super) fn native_access<Schema: ApplicationSchema>(
        &self,
        access: &SourceQueryReaderAccess<'_, Schema>,
    ) -> Result<ApplicationQueryNativeAccess<'r>, Failure> {
        match self {
            Self::Committed(runtime) => {
                let graph = access
                    .application
                    .runtime
                    .primary_graph()
                    .ok_or_else(unavailable)?;
                ApplicationQueryNativeAccess::borrowed(graph, runtime)
                    .map_err(Failure::NativeAccess)
            }
            Self::Applied(_) => Ok(ApplicationQueryNativeAccess::acquire()),
        }
    }

    pub(super) fn applied(&self) -> Option<AppliedManagedIndexRead<'r>> {
        match self {
            Self::Committed(_) => None,
            Self::Applied(read) => Some(*read),
        }
    }
}
