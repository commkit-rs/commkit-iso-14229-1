use crate::nrc::UdsNrc;
use crate::service::UdsService;
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.6

    The TesterPresent service is used to indicate to the server(s) that a client is still connected to the vehicle

    Supported NRC:
        - SFNS
        - IMLOIF
*/

#[allow(non_camel_case_types)]
pub struct x3E_TesterPresent;

impl UdsService for x3E_TesterPresent {
    const SID: u8 = 0x3E;

    type Request<'a> = x3E_TesterPresentRequest;
    type Response<'a> = x3E_TesterPresentResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x3E_TesterPresentRequest {
    pub subfunction: UdsSubfunction,
}

impl x3E_TesterPresentRequest {
    pub const LEN: usize = 1;

    pub const fn new(suppress_positive_response: bool) -> Self {
        Self { subfunction: UdsSubfunction::from_parts(0x00, suppress_positive_response) }
    }

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }

    pub const fn is_zero_sub_function(&self) -> bool {
        self.subfunction.parameter_value() == 0x00
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x3E_TesterPresentResponse {
    pub subfunction: UdsSubfunction,
}

impl x3E_TesterPresentResponse {
    pub const LEN: usize = 1;

    pub const fn new() -> Self {
        Self { subfunction: UdsSubfunction::from_parts(0x00, false) }
    }

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }

    pub const fn is_zero_sub_function(&self) -> bool {
        self.subfunction.parameter_value() == 0x00
    }
}

impl Default for x3E_TesterPresentResponse {
    fn default() -> Self {
        Self::new()
    }
}
