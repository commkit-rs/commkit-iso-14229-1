use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::UdsServiceResponse;

/*
    ISO 14229-1 Section 7.4

    The negative response message is sent by the server in place of a positive response when a request cannot be performed

    Negative response SID 0x7F is followed by a copy of the rejected request SID and the responseCode (NRC)
*/

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x7F_NegativeResponse {
    pub request_sid: u8,
    pub nrc: UdsNrc,
}

impl x7F_NegativeResponse {
    pub const SID: u8 = 0x7F;
    pub const LEN: usize = 2;

    pub const fn new(request_sid: u8, nrc: UdsNrc) -> Self {
        Self { request_sid, nrc }
    }

    pub const fn is_response_pending(&self) -> bool {
        self.nrc.is_response_pending()
    }
}

impl<'a> TryFrom<&'a [u8]> for x7F_NegativeResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { request_sid: data[0], nrc: UdsNrc::new(data[1]) })
    }
}

impl TryTo for x7F_NegativeResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.request_sid;
        buf[1] = self.nrc.raw();
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x7F_NegativeResponse {}
