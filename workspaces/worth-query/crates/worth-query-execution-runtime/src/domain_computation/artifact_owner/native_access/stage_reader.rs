use worth_foundational::facade::{AspectKey, AspectShape};
use worth_query_installation::facade::{
    WorthQueryArtifactFieldSlicePosture, WorthQueryArtifactRowBatchPosture,
    WorthQueryArtifactScalarFallbackPosture,
};

use super::super::WorthQueryTransferredArtifactHandle;
use super::admission::WorthQueryArtifactReaderAuthorityAdmission;
use super::field_slice::with_borrowed_field;
use super::row_batch::with_borrowed_rows;
use super::row_batch::WorthQueryArtifactRowProgress;
use super::{
    WorthQueryArtifactAccessAuthority, WorthQueryArtifactChunkCursor,
    WorthQueryArtifactChunkRequest, WorthQueryArtifactFieldSliceRequest,
    WorthQueryArtifactNativeAccessAdmission, WorthQueryArtifactNativeAccessBound,
    WorthQueryArtifactNativeAccessDenial, WorthQueryArtifactNativeAccessDenialKind as Kind,
    WorthQueryArtifactNativeAccessOutcome, WorthQueryArtifactNativeValueView,
    WorthQueryArtifactProjectedChunkCursor, WorthQueryArtifactProjectedChunkRequest,
    WorthQueryArtifactRowBatchRequest, WorthQueryArtifactScalarFallbackRequest,
};

pub struct WorthQueryStageArtifactReader<'a> {
    handle: &'a WorthQueryTransferredArtifactHandle,
    authority: WorthQueryArtifactReaderAuthorityAdmission<'a>,
}

impl<'a> WorthQueryStageArtifactReader<'a> {
    pub fn admit(
        handle: &'a WorthQueryTransferredArtifactHandle,
        authority: &'a WorthQueryArtifactAccessAuthority,
    ) -> Result<Self, WorthQueryArtifactNativeAccessDenial> {
        Ok(Self {
            handle,
            authority: WorthQueryArtifactReaderAuthorityAdmission::admit(handle, authority)?,
        })
    }

    pub fn with_rows<T>(
        self,
        request: WorthQueryArtifactRowBatchRequest,
        consume: impl for<'view> FnOnce(super::WorthQueryArtifactBorrowedRowBatch<'view>) -> T,
    ) -> Result<WorthQueryArtifactNativeAccessOutcome<T>, WorthQueryArtifactNativeAccessDenial>
    {
        let contract = self.authority.native_contract();
        if contract.row_batch() != WorthQueryArtifactRowBatchPosture::Borrowed {
            return Err(self.authority.denial(Kind::RowBatchDenied));
        }
        validate_direct_fields(contract, request.fields())
            .map_err(|kind| self.authority.denial(kind))?;
        validate_chunk_bound(contract, request.max_rows())
            .map_err(|kind| self.authority.denial(kind))?;
        let mut admission = self.authority.admit_access(
            self.handle,
            request.layout(),
            request.fields(),
            WorthQueryArtifactNativeAccessBound::RowBatch {
                start_row: request.start_row(),
                max_rows: request.max_rows(),
            },
        )?;
        let outcome = with_borrowed_rows(
            &mut admission,
            request.start_row(),
            request.max_rows(),
            request.fields(),
            WorthQueryArtifactRowProgress::MayBeEmpty,
            consume,
        )?;
        Ok(WorthQueryArtifactNativeAccessOutcome::new(
            outcome.value,
            admission.evidence(),
        ))
    }

    pub fn with_field_slice<T>(
        self,
        request: WorthQueryArtifactFieldSliceRequest,
        consume: impl for<'view> FnOnce(super::WorthQueryArtifactNativeFieldSlice<'view>) -> T,
    ) -> Result<WorthQueryArtifactNativeAccessOutcome<T>, WorthQueryArtifactNativeAccessDenial>
    {
        let contract = self.authority.native_contract();
        let Some(field) = contract
            .layout()
            .fields()
            .iter()
            .find(|field| field.aspect().key() == request.field())
        else {
            return Err(self.authority.denial(Kind::FieldNotDeclared));
        };
        if field.field_slice() != WorthQueryArtifactFieldSlicePosture::Borrowed {
            return Err(self.authority.denial(Kind::FieldSliceDenied));
        }
        validate_chunk_bound(contract, request.max_rows())
            .map_err(|kind| self.authority.denial(kind))?;
        let requested = [request.field().clone()];
        let mut admission = self.authority.admit_access(
            self.handle,
            request.layout(),
            &requested,
            WorthQueryArtifactNativeAccessBound::FieldSlice {
                start_row: request.start_row(),
                max_rows: request.max_rows(),
            },
        )?;
        let value = with_borrowed_field(
            &mut admission,
            request.start_row(),
            request.max_rows(),
            request.field(),
            consume,
        )?;
        Ok(WorthQueryArtifactNativeAccessOutcome::new(
            value,
            admission.evidence(),
        ))
    }

    pub fn chunks(
        self,
        request: WorthQueryArtifactChunkRequest,
    ) -> Result<WorthQueryArtifactChunkCursor<'a>, WorthQueryArtifactNativeAccessDenial> {
        let contract = self.authority.native_contract();
        if contract.row_batch() != WorthQueryArtifactRowBatchPosture::Borrowed {
            return Err(self.authority.denial(Kind::RowBatchDenied));
        }
        validate_direct_fields(contract, request.fields())
            .map_err(|kind| self.authority.denial(kind))?;
        validate_chunk_bound(contract, request.chunk_rows())
            .map_err(|kind| self.authority.denial(kind))?;
        let admission = self.authority.admit_access(
            self.handle,
            request.layout(),
            request.fields(),
            WorthQueryArtifactNativeAccessBound::Chunk {
                chunk_rows: request.chunk_rows(),
            },
        )?;
        WorthQueryArtifactChunkCursor::new(admission, request)
    }

    pub fn projected_chunks(
        self,
        request: WorthQueryArtifactProjectedChunkRequest,
    ) -> Result<WorthQueryArtifactProjectedChunkCursor<'a>, WorthQueryArtifactNativeAccessDenial>
    {
        let contract = self.authority.native_contract();
        validate_chunk_bound(contract, request.chunk_rows())
            .map_err(|kind| self.authority.denial(kind))?;
        let Some(projection) = contract
            .bulk_projections()
            .iter()
            .find(|projection| projection.identity() == request.projection_identity())
            .cloned()
        else {
            return Err(self.authority.denial(Kind::ProjectionDenied));
        };
        let fields = projection.source_fields().to_vec();
        let admission = self.authority.admit_access(
            self.handle,
            request.layout(),
            &fields,
            WorthQueryArtifactNativeAccessBound::Projection {
                projection_identity: request.projection_identity().to_owned(),
                chunk_rows: request.chunk_rows(),
            },
        )?;
        WorthQueryArtifactProjectedChunkCursor::new(admission, request, projection)
    }

    pub fn scalar_fallback(
        self,
        request: WorthQueryArtifactScalarFallbackRequest,
    ) -> Result<WorthQueryArtifactScalarFallbackSession<'a>, WorthQueryArtifactNativeAccessDenial>
    {
        let WorthQueryArtifactScalarFallbackPosture::Admitted {
            max_calls_per_admission,
            max_call_amplification,
        } = self.authority.native_contract().scalar_fallback()
        else {
            return Err(self.authority.denial(Kind::ScalarFallbackDenied));
        };
        validate_direct_fields(self.authority.native_contract(), request.fields())
            .map_err(|kind| self.authority.denial(kind))?;
        let admission = self.authority.admit_access(
            self.handle,
            request.layout(),
            request.fields(),
            WorthQueryArtifactNativeAccessBound::ScalarFallback {
                max_calls_per_admission,
                max_call_amplification,
            },
        )?;
        Ok(WorthQueryArtifactScalarFallbackSession {
            admission,
            max_calls_per_admission,
            max_call_amplification,
            calls: 0,
        })
    }
}

pub struct WorthQueryArtifactScalarFallbackSession<'a> {
    admission: WorthQueryArtifactNativeAccessAdmission<'a>,
    max_calls_per_admission: usize,
    max_call_amplification: usize,
    calls: usize,
}

impl WorthQueryArtifactScalarFallbackSession<'_> {
    pub fn with_value<T>(
        &mut self,
        row: usize,
        field: &AspectKey,
        consume: impl for<'view> FnOnce(WorthQueryArtifactNativeValueView<'view>) -> T,
    ) -> Result<T, WorthQueryArtifactNativeAccessDenial> {
        if !self.admission.requested_fields().contains(field) {
            return Err(self.denial(Kind::FieldNotDeclared));
        }
        let amplified_limit = self
            .admission
            .requested_fields()
            .len()
            .saturating_mul(self.max_call_amplification);
        if self.calls >= self.max_calls_per_admission || self.calls >= amplified_limit {
            return Err(self.denial(Kind::BoundsExceeded));
        }
        let layout = self.admission.native_contract().layout().clone();
        let field = field.clone();
        let value = self.admission.with_provider(|provider, session| {
            let value = provider.scalar(session, row, &field)?;
            let source_bytes = value.physical_bytes();
            let Some(contract) = layout
                .fields()
                .iter()
                .find(|contract| contract.aspect().key() == &field)
            else {
                return Err(super::WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            };
            if !value.matches_shape(contract.aspect().shape()) {
                return Err(super::WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            }
            Ok((
                consume(WorthQueryArtifactNativeValueView::from_provider(value)),
                source_bytes,
            ))
        })?;
        self.calls += 1;
        self.admission.counters_mut().scalar_calls += 1;
        self.admission.counters_mut().values_exposed += 1;
        self.admission.counters_mut().source_bytes += value.1;
        Ok(value.0)
    }

    pub fn evidence(&self) -> super::WorthQueryArtifactNativeAccessEvidence {
        self.admission.evidence()
    }

    fn denial(&self, kind: Kind) -> WorthQueryArtifactNativeAccessDenial {
        self.admission.denial(
            kind,
            "scalar fallback request exceeds its installed contract",
        )
    }
}

fn validate_chunk_bound(
    contract: &worth_query_installation::facade::WorthQueryArtifactNativeAccessContract,
    rows: usize,
) -> Result<(), Kind> {
    let Some(chunks) = contract.chunks() else {
        return Err(Kind::ChunkingDenied);
    };
    if rows == 0 || rows > chunks.max_rows() {
        return Err(Kind::BoundsExceeded);
    }
    Ok(())
}

fn validate_direct_fields(
    contract: &worth_query_installation::facade::WorthQueryArtifactNativeAccessContract,
    fields: &[AspectKey],
) -> Result<(), Kind> {
    for requested in fields {
        let Some(field) = contract
            .layout()
            .fields()
            .iter()
            .find(|field| field.aspect().key() == requested)
        else {
            return Err(Kind::FieldNotDeclared);
        };
        if !matches!(
            field.aspect().shape(),
            AspectShape::Scalar(_) | AspectShape::Struct(_)
        ) {
            return Err(Kind::ProviderNativeProjectionRequired);
        }
    }
    Ok(())
}
