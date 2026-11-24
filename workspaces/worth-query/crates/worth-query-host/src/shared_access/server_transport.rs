//! Non-blocking loopback TCP connection table and bounded frame I/O for the remote server.

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpListener, TcpStream};

use super::denial::{BudgetRefusalReason, SharedAccessDenial};
use super::framing::{
    encode_refusal_payload, parse_header, write_bounded_frame, FrameKind, FRAME_HEADER_LEN,
};
use super::types::RemoteServerLimits;

const MAX_CONCURRENT_CONNECTIONS: usize = 64;

pub(super) struct RemoteTcpPool {
    listener: TcpListener,
    local_addr: SocketAddr,
    connections: BTreeMap<u64, TcpStream>,
    next_connection_id: u64,
}

impl RemoteTcpPool {
    pub(super) fn bind(bind_addr: SocketAddr) -> Result<Self, SharedAccessDenial> {
        let listener = TcpListener::bind(bind_addr)
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
        listener
            .set_nonblocking(true)
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
        let local_addr = listener
            .local_addr()
            .map_err(|err| SharedAccessDenial::TransportIo(err.to_string()))?;
        Ok(Self {
            listener,
            local_addr,
            connections: BTreeMap::new(),
            next_connection_id: 1,
        })
    }

    pub(super) const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub(super) fn clear(&mut self) {
        self.connections.clear();
    }

    pub(super) fn serve_pending(
        &mut self,
        limits: RemoteServerLimits,
        mut dispatch: impl FnMut(FrameKind, &[u8]) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    ) -> Result<usize, SharedAccessDenial> {
        self.accept_pending()?;
        let ids: Vec<u64> = self.connections.keys().copied().collect();
        let mut served = 0_usize;
        for conn_id in ids {
            match self.service_connection(conn_id, limits, &mut dispatch) {
                ConnectionStep::Served => served += 1,
                ConnectionStep::ServedAndDrop => {
                    served += 1;
                    self.connections.remove(&conn_id);
                }
                ConnectionStep::Idle => {}
                ConnectionStep::Drop => {
                    self.connections.remove(&conn_id);
                }
            }
        }
        Ok(served)
    }

    fn accept_pending(&mut self) -> Result<(), SharedAccessDenial> {
        loop {
            match self.listener.accept() {
                Ok((stream, _peer)) => {
                    if self.connections.len() >= MAX_CONCURRENT_CONNECTIONS {
                        drop(stream);
                        continue;
                    }
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_nonblocking(true);
                    let id = self.next_connection_id;
                    self.next_connection_id = self.next_connection_id.saturating_add(1);
                    self.connections.insert(id, stream);
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(err) => return Err(SharedAccessDenial::TransportIo(err.to_string())),
            }
        }
    }

    fn service_connection(
        &mut self,
        conn_id: u64,
        limits: RemoteServerLimits,
        dispatch: &mut impl FnMut(FrameKind, &[u8]) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    ) -> ConnectionStep {
        let Some(mut stream) = self.connections.remove(&conn_id) else {
            return ConnectionStep::Drop;
        };
        let mut first = [0_u8; 1];
        match stream.read(&mut first) {
            Ok(0) => return ConnectionStep::Drop,
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                self.connections.insert(conn_id, stream);
                return ConnectionStep::Idle;
            }
            Err(_) => return ConnectionStep::Drop,
        }
        let step = handle_stream_frame(&mut stream, first[0], limits, dispatch);
        if matches!(step, ConnectionStep::Served) {
            let _ = stream.set_nonblocking(true);
            self.connections.insert(conn_id, stream);
        }
        step
    }
}

enum ConnectionStep {
    Served,
    ServedAndDrop,
    Idle,
    Drop,
}

fn handle_stream_frame(
    stream: &mut TcpStream,
    first_byte: u8,
    limits: RemoteServerLimits,
    dispatch: &mut impl FnMut(FrameKind, &[u8]) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
) -> ConnectionStep {
    let max_bytes = limits.max_frame_bytes();
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(limits.request_timeout()));
    let _ = stream.set_write_timeout(Some(limits.request_timeout()));
    let mut header = [0_u8; FRAME_HEADER_LEN];
    header[0] = first_byte;
    if stream.read_exact(&mut header[1..]).is_err() {
        return ConnectionStep::Drop;
    }
    let (kind, payload_len) = match parse_header(&header, max_bytes) {
        Ok(parsed) => parsed,
        Err(denial) => {
            let _ = write_bounded_frame(
                stream,
                FrameKind::RefusalResponse,
                &encode_refusal_payload(&denial),
                max_bytes,
            );
            return ConnectionStep::ServedAndDrop;
        }
    };
    let mut payload = vec![0_u8; payload_len as usize];
    if stream.read_exact(&mut payload).is_err() {
        return ConnectionStep::Drop;
    }
    write_dispatch_response(stream, dispatch(kind, &payload), max_bytes)
}

fn write_dispatch_response(
    stream: &mut TcpStream,
    outcome: Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    max_bytes: u32,
) -> ConnectionStep {
    let (resp_kind, resp_payload) = match outcome {
        Ok(pair) => pair,
        Err(denial) => (FrameKind::RefusalResponse, encode_refusal_payload(&denial)),
    };
    let written = if resp_payload.len() > max_bytes as usize {
        let refusal = SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
            bytes: u32::try_from(resp_payload.len()).unwrap_or(u32::MAX),
            maximum: max_bytes,
        });
        write_bounded_frame(
            stream,
            FrameKind::RefusalResponse,
            &encode_refusal_payload(&refusal),
            max_bytes,
        )
    } else {
        write_bounded_frame(stream, resp_kind, &resp_payload, max_bytes)
    };
    if written.is_ok() {
        ConnectionStep::Served
    } else {
        ConnectionStep::Drop
    }
}
