use commkit::{Duration, Instant, TryTo};

use crate::nrc::UdsNrc;
use crate::server::positive_response_sid;
use crate::service::{UdsService, UdsServiceRequest};
use crate::services::{
    DiagnosticSessionType, x3E_TesterPresent, x3E_TesterPresentRequest, x7F_NegativeResponse, x10_DiagnosticSessionControl,
};

/*
    ISO 14229-1 Section 7.5 / ISO 14229-2 Section 7

    Polling UDS client. Holds no request/response buffers: requests are handed to a send callback as a request SID
    plus a `TryTo` payload, and responses are borrowed from the bytes passed to `handle_response`
*/

const DSC_SID: u8 = x10_DiagnosticSessionControl::SID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsClientConfig {
    pub p2_client: Duration,
    pub p2_star_client: Duration,
    pub s3_client: Duration,
}

impl Default for UdsClientConfig {
    fn default() -> Self {
        Self {
            p2_client: Duration::from_ticks(150_000),
            p2_star_client: Duration::from_ticks(5_500_000),
            s3_client: Duration::from_ticks(2_000_000),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsClientError {
    /// A request is already waiting for its response
    Busy,
    /// The request failed to encode or the send callback rejected it
    Send(UdsNrc),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsClientEvent<'a> {
    /// Not a response to the outstanding request
    Ignored,
    /// NRC 0x78 received, the response deadline was extended to P2*client
    ResponsePending { request_sid: u8 },
    /// Payload after the response SID. Decode with `S::Response::try_from(payload)`
    Positive { request_sid: u8, payload: &'a [u8] },
    Negative { request_sid: u8, nrc: UdsNrc },
    /// No final response arrived before the P2client / P2*client deadline
    Timeout { request_sid: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutstandingRequest {
    request_sid: u8,
    deadline: Instant,
}

pub struct UdsClient {
    pub config: UdsClientConfig,
    outstanding: Option<OutstandingRequest>,
    keep_alive: bool,
    s3_deadline: Option<Instant>,
}

impl UdsClient {
    pub const fn new(config: UdsClientConfig) -> Self {
        Self { config, outstanding: None, keep_alive: false, s3_deadline: None }
    }

    pub const fn outstanding_request_sid(&self) -> Option<u8> {
        match self.outstanding {
            Some(outstanding) => Some(outstanding.request_sid),
            None => None,
        }
    }

    pub const fn is_busy(&self) -> bool {
        self.outstanding.is_some()
    }

    pub fn cancel(&mut self) {
        self.outstanding = None;
    }

    pub const fn keep_alive(&self) -> bool {
        self.keep_alive
    }

    /// Enables TesterPresent keep-alive. Set automatically from DiagnosticSessionControl traffic
    pub fn set_keep_alive(&mut self, now: Instant, keep_alive: bool) {
        self.keep_alive = keep_alive;
        self.s3_deadline = keep_alive.then(|| now + self.config.s3_client);
    }

    /// Sends a request. A request with suppressPosRsp set completes as soon as it is sent
    pub fn send_request<'r, S, F>(&mut self, now: Instant, request: &S::Request<'r>, send: &mut F) -> Result<(), UdsClientError>
    where
        S: UdsService,
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        if self.outstanding.is_some() {
            return Err(UdsClientError::Busy);
        }
        send(S::SID, request).map_err(UdsClientError::Send)?;

        let subfunction = request.get_subfunction();
        if subfunction.is_some_and(|sf| sf.suppress_pos_rsp_msg_indication_bit()) {
            if S::SID == DSC_SID
                && let Some(sf) = subfunction
            {
                self.keep_alive = is_non_default_session(sf.parameter_value());
            }
            self.restart_s3(now);
        } else {
            self.s3_deadline = None;
            self.outstanding = Some(OutstandingRequest { request_sid: S::SID, deadline: now + self.config.p2_client });
        }
        Ok(())
    }

    /// Handles one complete response message (SID included)
    pub fn handle_response<'a>(&mut self, now: Instant, response: &'a [u8]) -> UdsClientEvent<'a> {
        let (Some(outstanding), Some((&sid, payload))) = (self.outstanding, response.split_first()) else {
            return UdsClientEvent::Ignored;
        };
        let request_sid = outstanding.request_sid;

        if sid == x7F_NegativeResponse::SID {
            let Ok(negative) = x7F_NegativeResponse::try_from(payload) else {
                return UdsClientEvent::Ignored;
            };
            if negative.request_sid != request_sid {
                return UdsClientEvent::Ignored;
            }
            if negative.is_response_pending() {
                self.outstanding = Some(OutstandingRequest { request_sid, deadline: now + self.config.p2_star_client });
                return UdsClientEvent::ResponsePending { request_sid };
            }
            self.conclude(now);
            return UdsClientEvent::Negative { request_sid, nrc: negative.nrc };
        }

        if sid != positive_response_sid(request_sid) {
            return UdsClientEvent::Ignored;
        }
        if request_sid == DSC_SID
            && let Some(&session) = payload.first()
        {
            self.keep_alive = is_non_default_session(session & 0x7F);
        }
        self.conclude(now);
        UdsClientEvent::Positive { request_sid, payload }
    }

    /// Reports a response timeout and sends TesterPresent (suppressPosRsp) when S3client expires in a non-default session
    pub fn poll<F>(&mut self, now: Instant, send: &mut F) -> Result<Option<UdsClientEvent<'static>>, UdsClientError>
    where
        F: FnMut(u8, &dyn TryTo<Error = UdsNrc>) -> Result<(), UdsNrc>,
    {
        if let Some(outstanding) = self.outstanding
            && now >= outstanding.deadline
        {
            self.conclude(now);
            return Ok(Some(UdsClientEvent::Timeout { request_sid: outstanding.request_sid }));
        }

        if self.outstanding.is_none()
            && self.keep_alive
            && let Some(deadline) = self.s3_deadline
            && now >= deadline
        {
            self.send_request::<x3E_TesterPresent, F>(now, &x3E_TesterPresentRequest::new(true), send)?;
        }
        Ok(None)
    }

    fn conclude(&mut self, now: Instant) {
        self.outstanding = None;
        self.restart_s3(now);
    }

    fn restart_s3(&mut self, now: Instant) {
        self.s3_deadline = self.keep_alive.then(|| now + self.config.s3_client);
    }
}

fn is_non_default_session(session_type: u8) -> bool {
    DiagnosticSessionType::from_u8(session_type) != Some(DiagnosticSessionType::DefaultSession)
}
