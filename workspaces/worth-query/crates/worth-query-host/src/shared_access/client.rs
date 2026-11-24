//! Authenticated remote collaborator client and governed live watch handle.

use std::io::Write;
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};

use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::auth::RemoteAuthToken;
use super::binding::{RemoteBinaryCodec, WorthQuerySharedAccessProgram};
use super::denial::{ProtocolRefusalReason, SharedAccessDenial};
use super::framing::{
    decode_refusal_payload, decode_rows_payload, encode_scope_request, encode_watch_id_request,
    read_bounded_frame, write_bounded_frame, BinaryReader, FrameKind,
};
use super::types::{RemoteClientLimits, RemoteWatchId, RemoteWatchPollOutcome};

/// Authenticated collaborator client connected to a single-owner remote Directory Home.
pub struct WorthQueryRemoteClient {
    addr: SocketAddr,
    stream: TcpStream,
    auth_token: RemoteAuthToken,
    limits: RemoteClientLimits,
}

impl WorthQueryRemoteClient {
    pub fn connect(
        addr: SocketAddr,
        auth_token: RemoteAuthToken,
    ) -> Result<Self, SharedAccessDenial> {
        Self::connect_with_limits(addr, auth_token, RemoteClientLimits::default())
    }

    pub fn connect_with_limits(
        addr: SocketAddr,
        auth_token: RemoteAuthToken,
        limits: RemoteClientLimits,
    ) -> Result<Self, SharedAccessDenial> {
        let stream = TcpStream::connect_timeout(&addr, limits.io_timeout())
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
        configure_stream(&stream, limits)?;
        Ok(Self {
            addr,
            stream,
            auth_token,
            limits,
        })
    }

    pub const fn peer_addr(&self) -> SocketAddr {
        self.addr
    }

    pub const fn auth_token(&self) -> &RemoteAuthToken {
        &self.auth_token
    }

    pub fn set_auth_token(&mut self, auth_token: RemoteAuthToken) {
        self.auth_token = auth_token;
    }

    /// Executes a governed remote query synchronously against the single-owner Home.
    pub fn query<Schema, Program>(
        &mut self,
        scope: &Program::ScopeId,
        query: &Program::QueryPayload,
    ) -> Result<Vec<Program::Row>, SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        self.send_query::<Schema, Program>(scope, query)?;
        self.recv_query::<Schema, Program>()
    }

    /// Sends a framed remote query without waiting for the response yet.
    pub fn send_query<Schema, Program>(
        &mut self,
        scope: &Program::ScopeId,
        query: &Program::QueryPayload,
    ) -> Result<(), SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        let scope_key = Program::scope_key(scope);
        let payload = encode_scope_request(&self.auth_token, &scope_key, &query.encode_remote());
        write_bounded_frame(
            &mut self.stream,
            FrameKind::QueryRequest,
            &payload,
            self.limits.max_frame_bytes(),
        )
    }

    /// Receives and decodes the response for a previously sent remote query.
    pub fn recv_query<Schema, Program>(&mut self) -> Result<Vec<Program::Row>, SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        let (kind, payload) = read_bounded_frame(&mut self.stream, self.limits.max_frame_bytes())?;
        match kind {
            FrameKind::QueryResponse => {
                let mut reader = BinaryReader::new(&payload);
                let raw_rows = decode_rows_payload(&mut reader, self.limits.max_frame_bytes())?;
                reader.finish()?;
                decode_typed_rows::<Program::Row>(&raw_rows)
            }
            FrameKind::RefusalResponse => Err(decode_refusal_payload(&payload)?),
            _ => Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::UnknownMessageKind,
            )),
        }
    }

    /// Opens an authenticated, WORTH-governed live watch on `scope`.
    pub fn watch<Schema, Program>(
        &mut self,
        scope: &Program::ScopeId,
        query: &Program::QueryPayload,
    ) -> Result<WorthQueryRemoteWatch<Schema, Program>, SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        self.send_watch_open::<Schema, Program>(scope, query)?;
        self.recv_watch_open::<Schema, Program>()
    }

    /// Sends a watch-open request over this client's stream without waiting yet.
    pub fn send_watch_open<Schema, Program>(
        &mut self,
        scope: &Program::ScopeId,
        query: &Program::QueryPayload,
    ) -> Result<(), SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        let scope_key = Program::scope_key(scope);
        let payload = encode_scope_request(&self.auth_token, &scope_key, &query.encode_remote());
        write_bounded_frame(
            &mut self.stream,
            FrameKind::WatchOpenRequest,
            &payload,
            self.limits.max_frame_bytes(),
        )
    }

    /// Receives the watch-open response and returns a self-contained remote watch handle.
    pub fn recv_watch_open<Schema, Program>(
        &mut self,
    ) -> Result<WorthQueryRemoteWatch<Schema, Program>, SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        let (kind, payload) = read_bounded_frame(&mut self.stream, self.limits.max_frame_bytes())?;
        match kind {
            FrameKind::WatchOpenResponse => {
                let mut reader = BinaryReader::new(&payload);
                let watch_id = RemoteWatchId::new(reader.read_u64()?);
                let raw_rows = decode_rows_payload(&mut reader, self.limits.max_frame_bytes())?;
                reader.finish()?;
                let initial_rows = decode_typed_rows::<Program::Row>(&raw_rows)?;
                let watch_stream = TcpStream::connect_timeout(&self.addr, self.limits.io_timeout())
                    .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
                configure_stream(&watch_stream, self.limits)?;
                Ok(WorthQueryRemoteWatch {
                    id: watch_id,
                    stream: watch_stream,
                    auth_token: self.auth_token.clone(),
                    limits: self.limits,
                    initial_rows,
                    _marker: PhantomData,
                })
            }
            FrameKind::RefusalResponse => Err(decode_refusal_payload(&payload)?),
            _ => Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::UnknownMessageKind,
            )),
        }
    }

    /// Sends a watch-poll request for an arbitrary `RemoteWatchId` on this client stream.
    pub fn send_watch_poll_id(
        &mut self,
        watch_id: RemoteWatchId,
    ) -> Result<(), SharedAccessDenial> {
        let payload = encode_watch_id_request(&self.auth_token, watch_id);
        write_bounded_frame(
            &mut self.stream,
            FrameKind::WatchPollRequest,
            &payload,
            self.limits.max_frame_bytes(),
        )
    }

    /// Receives the response for a `send_watch_poll_id` request.
    pub fn recv_watch_poll_id<Schema, Program>(
        &mut self,
    ) -> Result<RemoteWatchPollOutcome<Program::Row>, SharedAccessDenial>
    where
        Schema: ApplicationSchema + 'static,
        Program: WorthQuerySharedAccessProgram<Schema>,
    {
        decode_watch_poll_response::<Schema, Program>(&mut self.stream, self.limits)
    }

    /// Sends arbitrary raw bytes on the TCP stream (for framing boundary verification).
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) -> Result<(), SharedAccessDenial> {
        self.stream
            .write_all(bytes)
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
        self.stream
            .flush()
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))
    }

    /// Reads one refusal frame back from the server.
    pub fn read_refusal_frame(&mut self) -> Result<SharedAccessDenial, SharedAccessDenial> {
        let (kind, payload) = read_bounded_frame(&mut self.stream, self.limits.max_frame_bytes())?;
        match kind {
            FrameKind::RefusalResponse => decode_refusal_payload(&payload),
            _ => Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::UnknownMessageKind,
            )),
        }
    }
}

/// Self-contained handle for an open remote governed live watch.
///
/// Fails closed immediately on the very next `poll` if the owner revokes the
/// backing capability grant or closes the Directory Home.
pub struct WorthQueryRemoteWatch<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    id: RemoteWatchId,
    stream: TcpStream,
    auth_token: RemoteAuthToken,
    limits: RemoteClientLimits,
    initial_rows: Vec<Program::Row>,
    _marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> WorthQueryRemoteWatch<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    pub const fn id(&self) -> RemoteWatchId {
        self.id
    }

    pub fn initial_rows(&self) -> &[Program::Row] {
        &self.initial_rows
    }

    pub fn set_auth_token(&mut self, auth_token: RemoteAuthToken) {
        self.auth_token = auth_token;
    }

    pub fn poll(&mut self) -> Result<RemoteWatchPollOutcome<Program::Row>, SharedAccessDenial> {
        self.send_poll()?;
        self.recv_poll()
    }

    pub fn send_poll(&mut self) -> Result<(), SharedAccessDenial> {
        let payload = encode_watch_id_request(&self.auth_token, self.id);
        write_bounded_frame(
            &mut self.stream,
            FrameKind::WatchPollRequest,
            &payload,
            self.limits.max_frame_bytes(),
        )
    }

    pub fn recv_poll(
        &mut self,
    ) -> Result<RemoteWatchPollOutcome<Program::Row>, SharedAccessDenial> {
        decode_watch_poll_response::<Schema, Program>(&mut self.stream, self.limits)
    }

    pub fn close(mut self) -> Result<(), SharedAccessDenial> {
        self.send_close()?;
        self.recv_close()
    }

    pub fn send_close(&mut self) -> Result<(), SharedAccessDenial> {
        let payload = encode_watch_id_request(&self.auth_token, self.id);
        write_bounded_frame(
            &mut self.stream,
            FrameKind::WatchCloseRequest,
            &payload,
            self.limits.max_frame_bytes(),
        )
    }

    pub fn recv_close(mut self) -> Result<(), SharedAccessDenial> {
        let (kind, payload) = read_bounded_frame(&mut self.stream, self.limits.max_frame_bytes())?;
        match kind {
            FrameKind::WatchCloseResponse => {
                if !payload.is_empty() {
                    return Err(SharedAccessDenial::ProtocolRefused(
                        ProtocolRefusalReason::TrailingBytes,
                    ));
                }
                Ok(())
            }
            FrameKind::RefusalResponse => Err(decode_refusal_payload(&payload)?),
            _ => Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::UnknownMessageKind,
            )),
        }
    }
}

fn decode_watch_poll_response<Schema, Program>(
    stream: &mut TcpStream,
    limits: RemoteClientLimits,
) -> Result<RemoteWatchPollOutcome<Program::Row>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    let (kind, payload) = read_bounded_frame(stream, limits.max_frame_bytes())?;
    match kind {
        FrameKind::WatchPollResponse => {
            let mut reader = BinaryReader::new(&payload);
            let tag = reader.read_u8()?;
            match tag {
                0 => {
                    reader.finish()?;
                    Ok(RemoteWatchPollOutcome::Pending)
                }
                1 => {
                    let raw_rows = decode_rows_payload(&mut reader, limits.max_frame_bytes())?;
                    reader.finish()?;
                    let rows = decode_typed_rows::<Program::Row>(&raw_rows)?;
                    Ok(RemoteWatchPollOutcome::Delivered(rows))
                }
                _ => Err(SharedAccessDenial::ProtocolRefused(
                    ProtocolRefusalReason::MalformedPayload,
                )),
            }
        }
        FrameKind::RefusalResponse => Err(decode_refusal_payload(&payload)?),
        _ => Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::UnknownMessageKind,
        )),
    }
}

fn configure_stream(
    stream: &TcpStream,
    limits: RemoteClientLimits,
) -> Result<(), SharedAccessDenial> {
    stream
        .set_nodelay(true)
        .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
    stream
        .set_read_timeout(Some(limits.io_timeout()))
        .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
    stream
        .set_write_timeout(Some(limits.io_timeout()))
        .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))
}

fn decode_typed_rows<Row: RemoteBinaryCodec>(
    raw_rows: &[Vec<u8>],
) -> Result<Vec<Row>, SharedAccessDenial> {
    let mut rows = Vec::with_capacity(raw_rows.len());
    for raw in raw_rows {
        rows.push(Row::decode_remote(raw)?);
    }
    Ok(rows)
}
