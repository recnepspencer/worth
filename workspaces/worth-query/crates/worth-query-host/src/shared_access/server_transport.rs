//! Strictly nonblocking loopback TCP connection pool with absolute deadlines and bounded ticks.

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use super::denial::{BudgetRefusalReason, SharedAccessDenial};
use super::framing::{
    encode_frame, encode_refusal_frame, parse_header, FrameKind, FRAME_HEADER_LEN,
};
use super::types::RemoteServerLimits;

const MAX_CONCURRENT_CONNECTIONS: usize = 64;
const MAX_ACCEPTS_PER_TICK: usize = 16;
const MAX_FRAMES_PER_TICK: usize = 32;

pub(super) struct RemoteTcpPool {
    listener: TcpListener,
    local_addr: SocketAddr,
    connections: BTreeMap<u64, NonblockingConnection>,
    next_connection_id: u64,
}

impl RemoteTcpPool {
    pub(super) fn bind(bind_addr: SocketAddr) -> Result<Self, SharedAccessDenial> {
        if !bind_addr.ip().is_loopback() {
            return Err(SharedAccessDenial::NonLoopbackAddressRefused);
        }
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

    pub(super) fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub(super) fn clear(&mut self) {
        self.connections.clear();
    }

    /// Accepts up to `MAX_ACCEPTS_PER_TICK` connections and advances nonblocking I/O
    /// without ever switching sockets to blocking mode or exceeding per-tick quotas.
    pub(super) fn serve_pending(
        &mut self,
        limits: RemoteServerLimits,
        mut dispatch: impl FnMut(
            u64,
            FrameKind,
            &[u8],
        ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    ) -> Result<(usize, Vec<u64>), SharedAccessDenial> {
        let now = worth_execution::MonotonicClock::system().now();
        self.accept_bounded(now, limits)?;
        let ids: Vec<u64> = self.connections.keys().copied().collect();
        let mut dispatched = 0_usize;
        let mut served = 0_usize;
        let mut dropped = Vec::new();
        for conn_id in ids {
            let budget = StepBudget {
                now,
                limits,
                allow_dispatch: dispatched < MAX_FRAMES_PER_TICK,
            };
            let step = self.step_connection(conn_id, budget, &mut dispatch);
            if step.dispatched {
                dispatched += 1;
            }
            if step.served {
                served += 1;
            }
            if step.drop {
                self.connections.remove(&conn_id);
                dropped.push(conn_id);
            }
        }
        Ok((served, dropped))
    }

    fn accept_bounded(
        &mut self,
        now: Instant,
        limits: RemoteServerLimits,
    ) -> Result<(), SharedAccessDenial> {
        for _ in 0..MAX_ACCEPTS_PER_TICK {
            match self.listener.accept() {
                Ok((stream, _peer)) => {
                    if self.connections.len() >= MAX_CONCURRENT_CONNECTIONS {
                        drop(stream);
                        continue;
                    }
                    let _ = stream.set_nodelay(true);
                    if stream.set_nonblocking(true).is_err() {
                        drop(stream);
                        continue;
                    }
                    let id = self.next_connection_id;
                    self.next_connection_id = self.next_connection_id.saturating_add(1);
                    self.connections
                        .insert(id, NonblockingConnection::new(stream, now, limits));
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(err) => return Err(SharedAccessDenial::TransportIo(err.to_string())),
            }
        }
        Ok(())
    }

    fn step_connection(
        &mut self,
        conn_id: u64,
        budget: StepBudget,
        dispatch: &mut impl FnMut(
            u64,
            FrameKind,
            &[u8],
        ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    ) -> ConnectionStep {
        let Some(conn) = self.connections.get_mut(&conn_id) else {
            return ConnectionStep::DROP;
        };
        if conn.is_expired(budget.now) {
            return ConnectionStep::DROP;
        }
        match conn.flush_pending_write(budget.now, budget.limits) {
            FlushOutcome::InProgress => return ConnectionStep::IDLE,
            FlushOutcome::CompletedAndDrop => return ConnectionStep::of(false, true, true),
            FlushOutcome::Completed => return ConnectionStep::of(false, true, false),
            FlushOutcome::Failed => return ConnectionStep::DROP,
            FlushOutcome::Empty => {}
        }
        if !budget.allow_dispatch {
            return ConnectionStep::IDLE;
        }
        conn.read_and_dispatch(conn_id, budget.now, budget.limits, dispatch)
    }
}

#[derive(Clone, Copy)]
struct StepBudget {
    now: Instant,
    limits: RemoteServerLimits,
    allow_dispatch: bool,
}

#[derive(Clone, Copy)]
struct ConnectionStep {
    dispatched: bool,
    served: bool,
    drop: bool,
}

impl ConnectionStep {
    const IDLE: Self = Self::of(false, false, false);
    const DROP: Self = Self::of(false, false, true);

    const fn of(dispatched: bool, served: bool, drop: bool) -> Self {
        Self {
            dispatched,
            served,
            drop,
        }
    }
}

enum FlushOutcome {
    Empty,
    InProgress,
    Completed,
    CompletedAndDrop,
    Failed,
}

struct NonblockingConnection {
    stream: TcpStream,
    header_buf: [u8; FRAME_HEADER_LEN],
    header_read: usize,
    parsed_header: Option<(FrameKind, u32)>,
    payload_buf: Vec<u8>,
    payload_read: usize,
    write_buf: Vec<u8>,
    write_pos: usize,
    drop_after_write: bool,
    idle_deadline: Instant,
    frame_deadline: Option<Instant>,
}

impl NonblockingConnection {
    fn new(stream: TcpStream, now: Instant, limits: RemoteServerLimits) -> Self {
        Self {
            stream,
            header_buf: [0_u8; FRAME_HEADER_LEN],
            header_read: 0,
            parsed_header: None,
            payload_buf: Vec::new(),
            payload_read: 0,
            write_buf: Vec::new(),
            write_pos: 0,
            drop_after_write: false,
            idle_deadline: now + limits.idle_timeout(),
            frame_deadline: None,
        }
    }

    fn is_expired(&self, now: Instant) -> bool {
        if let Some(deadline) = self.frame_deadline
            && now >= deadline
        {
            return true;
        }
        self.write_buf.is_empty() && self.frame_deadline.is_none() && now >= self.idle_deadline
    }

    fn flush_pending_write(&mut self, now: Instant, limits: RemoteServerLimits) -> FlushOutcome {
        if self.write_buf.is_empty() {
            return FlushOutcome::Empty;
        }
        while self.write_pos < self.write_buf.len() {
            match self.stream.write(&self.write_buf[self.write_pos..]) {
                Ok(0) => return FlushOutcome::Failed,
                Ok(n) => self.write_pos += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    return FlushOutcome::InProgress;
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return FlushOutcome::Failed,
            }
        }
        let drop_now = self.drop_after_write;
        self.write_buf.clear();
        self.write_pos = 0;
        self.drop_after_write = false;
        self.frame_deadline = None;
        self.idle_deadline = now + limits.idle_timeout();
        if drop_now {
            FlushOutcome::CompletedAndDrop
        } else {
            FlushOutcome::Completed
        }
    }

    fn read_and_dispatch(
        &mut self,
        conn_id: u64,
        now: Instant,
        limits: RemoteServerLimits,
        dispatch: &mut impl FnMut(
            u64,
            FrameKind,
            &[u8],
        ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    ) -> ConnectionStep {
        if self.parsed_header.is_none() {
            match self.read_header_bytes(now, limits) {
                ReadStatus::Ready => {
                    let max_bytes = limits.max_frame_bytes();
                    match parse_header(&self.header_buf, max_bytes) {
                        Ok((kind, payload_len)) => {
                            self.parsed_header = Some((kind, payload_len));
                            self.payload_buf = vec![0_u8; payload_len as usize];
                            self.payload_read = 0;
                        }
                        Err(denial) => {
                            let frame = encode_refusal_frame(&denial);
                            return self.queue_and_flush_dispatched(
                                frame.to_vec(),
                                true,
                                now,
                                limits,
                            );
                        }
                    }
                }
                ReadStatus::WouldBlock => return ConnectionStep::IDLE,
                ReadStatus::ClosedOrError => return ConnectionStep::DROP,
            }
        }
        match self.read_payload_bytes() {
            ReadStatus::Ready => {}
            ReadStatus::WouldBlock => return ConnectionStep::IDLE,
            ReadStatus::ClosedOrError => return ConnectionStep::DROP,
        }
        let (kind, _) = self.parsed_header.take().expect("header parsed");
        let payload = std::mem::take(&mut self.payload_buf);
        self.header_read = 0;
        self.payload_read = 0;
        let response_frame =
            encode_dispatch_outcome(dispatch(conn_id, kind, &payload), limits.max_frame_bytes());
        self.queue_and_flush_dispatched(response_frame, false, now, limits)
    }

    fn read_header_bytes(&mut self, now: Instant, limits: RemoteServerLimits) -> ReadStatus {
        while self.header_read < FRAME_HEADER_LEN {
            match self.stream.read(&mut self.header_buf[self.header_read..]) {
                Ok(0) => return ReadStatus::ClosedOrError,
                Ok(n) => {
                    if self.frame_deadline.is_none() {
                        self.frame_deadline = Some(now + limits.request_timeout());
                    }
                    self.header_read += n;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => return ReadStatus::WouldBlock,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return ReadStatus::ClosedOrError,
            }
        }
        ReadStatus::Ready
    }

    fn read_payload_bytes(&mut self) -> ReadStatus {
        while self.payload_read < self.payload_buf.len() {
            match self.stream.read(&mut self.payload_buf[self.payload_read..]) {
                Ok(0) => return ReadStatus::ClosedOrError,
                Ok(n) => self.payload_read += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => return ReadStatus::WouldBlock,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return ReadStatus::ClosedOrError,
            }
        }
        ReadStatus::Ready
    }

    /// Charges the per-tick dispatch quota immediately regardless of whether the
    /// response flushes synchronously, hits `WouldBlock`, or errors on write.
    fn queue_and_flush_dispatched(
        &mut self,
        frame: Vec<u8>,
        drop_after: bool,
        now: Instant,
        limits: RemoteServerLimits,
    ) -> ConnectionStep {
        self.write_buf = frame;
        self.write_pos = 0;
        self.drop_after_write = drop_after;
        if self.frame_deadline.is_none() {
            self.frame_deadline = Some(now + limits.request_timeout());
        }
        match self.flush_pending_write(now, limits) {
            FlushOutcome::Completed => ConnectionStep::of(true, true, false),
            FlushOutcome::CompletedAndDrop => ConnectionStep::of(true, true, true),
            FlushOutcome::InProgress => ConnectionStep::of(true, false, false),
            FlushOutcome::Failed | FlushOutcome::Empty => ConnectionStep::of(true, false, true),
        }
    }
}

enum ReadStatus {
    Ready,
    WouldBlock,
    ClosedOrError,
}

pub(super) fn encode_dispatch_outcome(
    outcome: Result<(FrameKind, Vec<u8>), SharedAccessDenial>,
    max_bytes: u32,
) -> Vec<u8> {
    match outcome {
        Ok((kind, payload)) => match encode_frame(kind, &payload, max_bytes) {
            Ok(frame) => frame,
            Err(_) => {
                let refusal =
                    SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
                        bytes: u32::try_from(payload.len()).unwrap_or(u32::MAX),
                        maximum: max_bytes,
                    });
                encode_refusal_frame(&refusal).to_vec()
            }
        },
        Err(denial) => encode_refusal_frame(&denial).to_vec(),
    }
}
