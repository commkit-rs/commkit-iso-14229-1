use commkit::{Duration, Instant, TryTo};

use crate::UDS_PROTOCOL_POSITIVE_RESPONSE_SID_OFFSET;
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest};
use crate::services::{DiagnosticSessionType, x7F_NegativeResponse};

/*
    ISO 14229-1 Section 7.5 / ISO 14229-2 Section 7

    Polling UDS server. Holds no request/response buffers: every outgoing message is handed to a send callback
    as a response SID plus a `TryTo` payload, which the caller encodes straight into its transport
*/

/// Receives every outgoing message: the response SID and the payload to encode after it
pub type UdsSendFn<'s> = dyn FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc> + 's;

/// A server's service table. Handlers must be `Sync` so the table can be a `static`
pub type UdsServerTable<'t, C> = [&'t (dyn UdsServerEntry<C> + Sync)];

/// Security level reported while no level is unlocked
pub const SECURITY_LEVEL_LOCKED: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsServerConfig {
    pub p2_server_max: Duration,
    pub p2_star_server_max: Duration,
    pub s3_server: Duration,
    pub response_pending_interval: Duration,
}

impl Default for UdsServerConfig {
    fn default() -> Self {
        Self {
            p2_server_max: Duration::from_ticks(50_000),
            p2_star_server_max: Duration::from_ticks(5_000_000),
            s3_server: Duration::from_ticks(5_000_000),
            response_pending_interval: Duration::from_ticks(2_500_000),
        }
    }
}

/// Spec-mandated server values, handed to every handler
pub struct UdsServerState<C> {
    pub config: UdsServerConfig,
    pub on_session_change: Option<fn(&mut C, DiagnosticSessionType, DiagnosticSessionType)>,
    pub on_security_level_change: Option<fn(&mut C, u8, u8)>,
    session: DiagnosticSessionType,
    security_level: u8,
}

impl<C> UdsServerState<C> {
    pub const fn new(config: UdsServerConfig) -> Self {
        Self {
            config,
            on_session_change: None,
            on_security_level_change: None,
            session: DiagnosticSessionType::DefaultSession,
            security_level: SECURITY_LEVEL_LOCKED,
        }
    }

    pub const fn session(&self) -> DiagnosticSessionType {
        self.session
    }

    pub fn set_session(&mut self, ctx: &mut C, session: DiagnosticSessionType) {
        let old = self.session;
        self.session = session;
        if old != session
            && let Some(cb) = self.on_session_change
        {
            cb(ctx, old, session);
        }
    }

    pub const fn security_level(&self) -> u8 {
        self.security_level
    }

    pub const fn is_unlocked(&self) -> bool {
        self.security_level != SECURITY_LEVEL_LOCKED
    }

    pub fn set_security_level(&mut self, ctx: &mut C, security_level: u8) {
        let old = self.security_level;
        self.security_level = security_level;
        if old != security_level
            && let Some(cb) = self.on_security_level_change
        {
            cb(ctx, old, security_level);
        }
    }

    pub fn is_default_session(&self) -> bool {
        self.session == DiagnosticSessionType::DefaultSession
    }
}

/// Outcome of a typed request handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerResult<R> {
    Positive(R),
    Negative(UdsNrc),
    Pending,
}

/// Implemented by the user, once per supported service
pub trait UdsRequestHandler<C> {
    type Service: UdsService;

    /// Runs before the request is decoded. Return NRC 0x7F here for sessions the service is not supported in
    fn request_precheck(&self, _ctx: &C, _server: &UdsServerState<C>) -> Result<(), UdsNrc> {
        Ok(())
    }

    fn request_handle<'c>(
        &self,
        ctx: &'c mut C,
        server: &mut UdsServerState<C>,
        request: <Self::Service as UdsService>::Request<'_>,
    ) -> HandlerResult<<Self::Service as UdsService>::Response<'c>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    PositiveSent,
    PositiveSuppressed,
    Negative(UdsNrc),
    Pending,
}

/// Type-erased form of `UdsRequestHandler`, implemented automatically. Server tables hold `&dyn UdsServerEntry<C>`
pub trait UdsServerEntry<C> {
    fn sid(&self) -> u8;

    fn precheck(&self, ctx: &C, server: &UdsServerState<C>) -> Result<(), UdsNrc>;

    fn dispatch(
        &self,
        ctx: &mut C,
        server: &mut UdsServerState<C>,
        payload: &[u8],
        send: &mut UdsSendFn<'_>,
    ) -> Result<DispatchResult, UdsNrc>;
}

impl<C, H: UdsRequestHandler<C>> UdsServerEntry<C> for H {
    fn sid(&self) -> u8 {
        H::Service::SID
    }

    fn precheck(&self, ctx: &C, server: &UdsServerState<C>) -> Result<(), UdsNrc> {
        self.request_precheck(ctx, server)
    }

    fn dispatch(
        &self,
        ctx: &mut C,
        server: &mut UdsServerState<C>,
        payload: &[u8],
        send: &mut UdsSendFn<'_>,
    ) -> Result<DispatchResult, UdsNrc> {
        let request = match <H::Service as UdsService>::Request::try_from(payload) {
            Ok(request) => request,
            Err(nrc) => return Ok(DispatchResult::Negative(nrc)),
        };
        let suppress = request.get_subfunction().is_some_and(|sf| sf.suppress_pos_rsp_msg_indication_bit());

        match self.request_handle(ctx, server, request) {
            HandlerResult::Positive(response) => {
                if suppress {
                    return Ok(DispatchResult::PositiveSuppressed);
                }
                send(positive_response_sid(H::Service::SID), &response)?;
                Ok(DispatchResult::PositiveSent)
            }
            HandlerResult::Negative(nrc) if nrc.is_response_pending() => Ok(DispatchResult::Pending),
            HandlerResult::Negative(nrc) => Ok(DispatchResult::Negative(nrc)),
            HandlerResult::Pending => Ok(DispatchResult::Pending),
        }
    }
}

/// What the server did with an incoming request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsServerOutcome {
    /// Nothing to respond to (empty message)
    Ignored,
    PositiveSent,
    PositiveSuppressed,
    NegativeSent(UdsNrc),
    NegativeSuppressed(UdsNrc),
    /// NRC 0x78 sent. Complete the request later with `force_response` / `force_negative_response`
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingRequest {
    request_sid: u8,
    next_response_pending_at: Instant,
}

pub struct UdsServer<'t, C> {
    pub state: UdsServerState<C>,
    table: &'t UdsServerTable<'t, C>,
    pending: Option<PendingRequest>,
    s3_deadline: Option<Instant>,
}

impl<'t, C> UdsServer<'t, C> {
    pub const fn new(table: &'t UdsServerTable<'t, C>, config: UdsServerConfig) -> Self {
        Self { state: UdsServerState::new(config), table, pending: None, s3_deadline: None }
    }

    pub const fn pending_request_sid(&self) -> Option<u8> {
        match self.pending {
            Some(pending) => Some(pending.request_sid),
            None => None,
        }
    }

    /// Handles one complete request message (SID included). `functional` is whether it arrived functionally addressed
    pub fn handle_request<F>(
        &mut self,
        ctx: &mut C,
        now: Instant,
        request: &[u8],
        functional: bool,
        send: &mut F,
    ) -> Result<UdsServerOutcome, UdsNrc>
    where
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        let Some((&sid, payload)) = request.split_first() else {
            return Ok(UdsServerOutcome::Ignored);
        };
        self.s3_deadline = None;

        if self.pending.is_some() {
            return self.finish_negative(now, sid, UdsNrc::BUSY_REPEAT_REQUEST, functional, send);
        }

        let Some(entry) = self.table.iter().find(|entry| entry.sid() == sid) else {
            return self.finish_negative(now, sid, UdsNrc::SERVICE_NOT_SUPPORTED, functional, send);
        };

        if let Err(nrc) = entry.precheck(ctx, &self.state) {
            return self.finish_negative(now, sid, nrc, functional, send);
        }

        match entry.dispatch(ctx, &mut self.state, payload, send)? {
            DispatchResult::PositiveSent => {
                self.conclude(now);
                Ok(UdsServerOutcome::PositiveSent)
            }
            DispatchResult::PositiveSuppressed => {
                self.conclude(now);
                Ok(UdsServerOutcome::PositiveSuppressed)
            }
            DispatchResult::Negative(nrc) => self.finish_negative(now, sid, nrc, functional, send),
            DispatchResult::Pending => {
                send_negative(sid, UdsNrc::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING, send)?;
                self.pending = Some(PendingRequest {
                    request_sid: sid,
                    next_response_pending_at: now + self.state.config.response_pending_interval,
                });
                Ok(UdsServerOutcome::Pending)
            }
        }
    }

    /// Repeats NRC 0x78 for a pending request and reverts to the default session when S3server expires
    pub fn poll<F>(&mut self, ctx: &mut C, now: Instant, send: &mut F) -> Result<(), UdsNrc>
    where
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        if let Some(pending) = self.pending.as_mut()
            && now >= pending.next_response_pending_at
        {
            send_negative(pending.request_sid, UdsNrc::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING, send)?;
            pending.next_response_pending_at = now + self.state.config.response_pending_interval;
        }

        if let Some(deadline) = self.s3_deadline
            && now >= deadline
        {
            self.s3_deadline = None;
            self.state.set_session(ctx, DiagnosticSessionType::DefaultSession);
            self.state.set_security_level(ctx, SECURITY_LEVEL_LOCKED);
        }

        Ok(())
    }

    /// Sends a final positive response, clearing any pending request. Always sent, regardless of suppressPosRsp
    pub fn force_response<S, F>(&mut self, now: Instant, response: &S::Response<'_>, send: &mut F) -> Result<(), UdsNrc>
    where
        S: UdsService,
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        self.pending = None;
        send(positive_response_sid(S::SID), response)?;
        self.conclude(now);
        Ok(())
    }

    /// Sends a final negative response, clearing any pending request. Always sent, regardless of addressing
    pub fn force_negative_response<F>(&mut self, now: Instant, request_sid: u8, nrc: UdsNrc, send: &mut F) -> Result<(), UdsNrc>
    where
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        self.pending = None;
        send_negative(request_sid, nrc, send)?;
        self.conclude(now);
        Ok(())
    }

    fn finish_negative<F>(
        &mut self,
        now: Instant,
        sid: u8,
        nrc: UdsNrc,
        functional: bool,
        send: &mut F,
    ) -> Result<UdsServerOutcome, UdsNrc>
    where
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        let outcome = if functional && is_suppressed_on_functional(nrc) {
            UdsServerOutcome::NegativeSuppressed(nrc)
        } else {
            send_negative(sid, nrc, send)?;
            UdsServerOutcome::NegativeSent(nrc)
        };
        if self.pending.is_none() {
            self.conclude(now);
        }
        Ok(outcome)
    }

    fn conclude(&mut self, now: Instant) {
        self.s3_deadline = if self.state.is_default_session() { None } else { Some(now + self.state.config.s3_server) };
    }
}

pub const fn positive_response_sid(request_sid: u8) -> u8 {
    request_sid.wrapping_add(UDS_PROTOCOL_POSITIVE_RESPONSE_SID_OFFSET)
}

fn send_negative<F>(request_sid: u8, nrc: UdsNrc, send: &mut F) -> Result<(), UdsNrc>
where
    F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
{
    send(x7F_NegativeResponse::SID, &x7F_NegativeResponse::new(request_sid, nrc))
}

const fn is_suppressed_on_functional(nrc: UdsNrc) -> bool {
    matches!(nrc.raw(), 0x11 | 0x12 | 0x31 | 0x7E | 0x7F)
}
