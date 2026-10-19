//! V38 extends the root with path links and independent creation trees.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationPathLink as PathLink;
fn write_paths(output: &mut dyn BinaryEncodingSink, paths: &[PathLink]) -> Result<(), Denial> {
    write_sequence(output, paths, |output, path| {
        if !path.has_supported_shape() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        super::super::application_query::write_root_path(output, &path.path)?;
        super::creation_links::write(output, std::slice::from_ref(&path.link), 38)
    })
}
fn decode_paths(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<PathLink>, Denial> {
    decode_sequence(input, budget, 12, |input, budget| {
        let path = super::super::application_query::decode_root_path(input, budget)?;
        let links = super::creation_links::decode(input, budget, 38)?;
        let [link]: [_; 1] = links
            .try_into()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
        let path = PathLink { path, link };
        if !path.has_supported_shape() {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        Ok(path)
    })
}
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    body: &ApplicationOperationBodyProgram,
) -> Result<(), Denial> {
    if !body.has_supported_peer_creations() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let root = body.creation();
    write_paths(output, root.map_or(&[], |c| c.path_links()))?;
    write_sequence(output, root.map_or(&[], |c| c.peers()), |output, peer| {
        creation_fields::write(output, peer)?;
        creation_links::write(output, peer.links(), 38)?;
        write_paths(output, peer.path_links())?;
        children::write(output, peer.children())
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    root: Option<ApplicationOperationBodyCreation>,
) -> Result<Option<ApplicationOperationBodyCreation>, Denial> {
    let paths = decode_paths(input, budget)?;
    let peers = decode_sequence(input, budget, 32, |input, budget| {
        let peer = creation_fields::decode(input, budget, 38, true)?.expect("explicit peer site");
        let links = creation_links::decode(input, budget, 38)?;
        let paths = decode_paths(input, budget)?;
        let children = children::decode(input, budget, 38)?;
        Ok(peer
            .with_untrusted_links(links)
            .with_untrusted_path_links(paths)
            .with_untrusted_children(children))
    })?;
    match root {
        Some(root) => Ok(Some(
            root.with_untrusted_path_links(paths)
                .with_untrusted_peers(peers),
        )),
        None if paths.is_empty() && peers.is_empty() => Ok(None),
        None => Err(Denial::new(Kind::InvalidRecordShape)),
    }
}
