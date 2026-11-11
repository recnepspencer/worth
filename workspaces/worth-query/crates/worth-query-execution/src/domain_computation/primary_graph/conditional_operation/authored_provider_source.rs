//! Query's provider projection stores its node and the host-owned complete source.
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_runtime_bridge::facade::{
    BridgeAuthoredProviderDenial as D, BridgeAuthoredProviderDescription as Description,
    BridgeConditionalProviderHeapRetention as Heap,
};

pub(super) fn encode(
    protocol: &'static str,
    node: &str,
    host: Option<&Description>,
) -> Description {
    let mut bytes = Vec::new();
    blob(&mut bytes, node.as_bytes());
    bytes.push(u8::from(host.is_some()));
    if let Some(host) = host {
        blob(&mut bytes, host.protocol().as_str().as_bytes());
        bytes.extend_from_slice(&host.version().get().to_le_bytes());
        blob(&mut bytes, host.payload());
    }
    Description::new(
        BoundaryProtocolIdentity::new(protocol),
        BoundaryProtocolVersion::new(1),
        bytes,
    )
}
fn blob(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(&(value.len() as u64).to_le_bytes());
    out.extend_from_slice(value);
}

pub(super) struct Decoded<'a> {
    pub(super) node: &'a str,
    pub(super) host: Option<Description>,
    _backing: Option<ExecutionMemoryReservation>,
}
pub(super) fn decode<'a>(
    source: &'a Description,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Decoded<'a>, D> {
    let mut input = Cursor {
        bytes: source.payload(),
        offset: 0,
    };
    let node = input.text()?;
    let fields = match input.fixed(1)?[0] {
        0 => None,
        1 => {
            let protocol = input.text()?;
            BoundaryProtocolIdentity::validate(protocol).map_err(|_| D::InvalidPayload)?;
            let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
                input.fixed(4)?.try_into().unwrap(),
            ))
            .map_err(|_| D::InvalidPayload)?;
            Some((protocol, version, input.blob()?))
        }
        _ => return Err(D::InvalidPayload),
    };
    if input.offset != input.bytes.len() {
        return Err(D::InvalidPayload);
    }
    let bytes = match fields {
        None => 0,
        Some((protocol, _, payload)) => Heap::arc_allocation_bytes(source.protocol())
            .checked_add(protocol.len() as u64)
            .and_then(|n| n.checked_add(Heap::arc_allocation_bytes(payload)))
            .ok_or(D::InvalidPayload)?,
    };
    let backing = execution
        .reserve_retained_memory(bytes)
        .map_err(D::Request)?;
    let host = fields.map(|(protocol, version, payload)| {
        Description::new(
            BoundaryProtocolIdentity::parse(protocol.to_owned()).expect("validated protocol"),
            version,
            payload,
        )
    });
    Ok(Decoded {
        node,
        host,
        _backing: backing,
    })
}
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn fixed(&mut self, count: usize) -> Result<&'a [u8], D> {
        let end = self.offset.checked_add(count).ok_or(D::InvalidPayload)?;
        let bytes = self.bytes.get(self.offset..end).ok_or(D::InvalidPayload)?;
        self.offset = end;
        Ok(bytes)
    }
    fn blob(&mut self) -> Result<&'a [u8], D> {
        let count = usize::try_from(u64::from_le_bytes(self.fixed(8)?.try_into().unwrap()))
            .map_err(|_| D::InvalidPayload)?;
        self.fixed(count)
    }
    fn text(&mut self) -> Result<&'a str, D> {
        std::str::from_utf8(self.blob()?).map_err(|_| D::InvalidPayload)
    }
}
