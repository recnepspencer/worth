//! One frame/table/payload verification engine for all anchored reads.
use super::*;
pub(in crate::physical_runtime::anchored_records) struct AnchoredValueStream {
    session: RecordReadSession,
    pub(in crate::physical_runtime::anchored_records) header: AnchoredRecordHeader,
    members: Vec<AnchoredRecordMemberLocator>,
    remaining: u64,
    expected_record_digest: [u8; 32],
    record_digest: Sha256,
    payload_digest: Sha256,
    complete: bool,
    refused: bool,
    // Last: table storage cannot outlive its original memory admission.
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl AnchoredValueStream {
    pub(in crate::physical_runtime::anchored_records) fn open(
        reader: &PhysicalRecordReader,
        name: AnchoredRecordName,
        anchor: RecordAnchor,
        limit: Option<u64>,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<Self, Failure> {
        check_request(request)?;
        let mut session = super::open(reader, anchor.record())?;
        let mut header_bytes = [0; HEADER_BYTES];
        require_filled(&mut session, &mut header_bytes)?;
        let mut record_digest = Sha256::new_with_prefix(header_bytes);
        let header =
            AnchoredRecordHeader::decode(&header_bytes, name.anchor(), anchor.generation())
                .map_err(Failure::Damaged)?;
        let record_bytes = header
            .member_count
            .checked_mul(ANCHORED_RECORD_MEMBER_LOCATOR_BYTES as u64)
            .and_then(|bytes| bytes.checked_add(HEADER_BYTES as u64))
            .and_then(|bytes| bytes.checked_add(header.payload_bytes));
        if record_bytes != Some(session.observation().bytes_requested()) {
            return Err(Failure::Damaged(AnchoredRecordDamage::Truncated));
        }
        if limit.is_some_and(|limit| header.payload_bytes > limit)
            || usize::try_from(header.payload_bytes).is_err()
        {
            return Err(Failure::LimitExceeded {
                payload_bytes: header.payload_bytes,
            });
        }
        let count = usize::try_from(header.member_count)
            .map_err(|_| Failure::Damaged(AnchoredRecordDamage::MalformedMemberTable))?;
        let table = std::alloc::Layout::array::<AnchoredRecordMemberLocator>(count)
            .map_err(|_| Failure::Damaged(AnchoredRecordDamage::MalformedMemberTable))?;
        let memory = match request {
            Some(request) => request
                .reserve_retained_memory(table.size() as u64)
                .map_err(Failure::Execution)?,
            None => None,
        };
        let members = read_member_table(&mut session, &header, &mut record_digest, request)?;
        Ok(Self {
            session,
            header,
            members,
            remaining: header.payload_bytes,
            expected_record_digest: anchor.record_payload_sha256(),
            record_digest,
            payload_digest: Sha256::new(),
            complete: false,
            refused: false,
            memory,
        })
    }
    pub(in crate::physical_runtime::anchored_records) fn complete(&self) -> bool {
        self.complete
    }
    pub(in crate::physical_runtime::anchored_records) fn read_next(
        &mut self,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        output: &mut [u8],
    ) -> Result<usize, Failure> {
        if self.complete {
            return Ok(0);
        }
        if self.refused {
            return Err(Failure::PreviouslyRefused);
        }
        let result = self.advance(request, output);
        if result.is_err() {
            self.refused = true;
        }
        result
    }
    fn advance(
        &mut self,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        output: &mut [u8],
    ) -> Result<usize, Failure> {
        check_request(request)?;
        if self.remaining == 0 {
            if fill(&mut self.session, &mut [0; 1])? != 0 {
                return Err(Failure::Damaged(AnchoredRecordDamage::TrailingBytes));
            }
            if <[u8; 32]>::from(self.record_digest.clone().finalize())
                != self.expected_record_digest
            {
                return Err(Failure::Damaged(AnchoredRecordDamage::RecordDigestMismatch));
            }
            self.header
                .verify_payload_digest(self.payload_digest.clone().finalize().into())
                .map_err(Failure::Damaged)?;
            self.complete = true;
            return Ok(0);
        }
        if output.is_empty() {
            return Err(Failure::EmptyReadBuffer);
        }
        let length = self
            .remaining
            .min(PAYLOAD_CHUNK_BYTES as u64)
            .min(output.len() as u64) as usize;
        require_filled(&mut self.session, &mut output[..length])?;
        self.record_digest.update(&output[..length]);
        self.payload_digest.update(&output[..length]);
        self.remaining -= length as u64;
        Ok(length)
    }
    pub(super) fn into_verified(self, payload: Option<Vec<u8>>) -> VerifiedAnchoredValue {
        assert!(
            self.complete,
            "verified anchored value requires actual complete EOF"
        );
        VerifiedAnchoredValue {
            header: self.header,
            members: self.members,
            payload,
            memory: self.memory,
        }
    }
}
fn check_request(
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<(), Failure> {
    if let Some(request) = request {
        request.check_live().map_err(|cause| {
            Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
        })?;
    }
    Ok(())
}
