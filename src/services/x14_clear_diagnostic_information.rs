use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 11.2

    The ClearDiagnosticInformation service is used to clear diagnostic information in one or multiple servers' memory

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - GPF
*/

#[allow(non_camel_case_types)]
pub struct x14_ClearDiagnosticInformation;

impl UdsService for x14_ClearDiagnosticInformation {
    const SID: u8 = 0x14;

    type Request<'a> = x14_ClearDiagnosticInformationRequest;
    type Response<'a> = x14_ClearDiagnosticInformationResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x14_ClearDiagnosticInformationRequest {
    pub group_of_dtc: u32,
}

impl x14_ClearDiagnosticInformationRequest {
    pub const LEN: usize = 3;
    pub const EMISSIONS_SYSTEM_GROUP: u32 = 0xFFFF33;
    pub const SAFETY_SYSTEM_GROUP: u32 = 0xFFFFD0;
    pub const ALL_GROUPS: u32 = 0xFFFFFF;
}

impl<'a> TryFrom<&'a [u8]> for x14_ClearDiagnosticInformationRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { group_of_dtc: u32::from_be_bytes([0, data[0], data[1], data[2]]) })
    }
}

impl TryTo for x14_ClearDiagnosticInformationRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if self.group_of_dtc > Self::ALL_GROUPS {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..Self::LEN].copy_from_slice(&self.group_of_dtc.to_be_bytes()[1..]);
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceRequest<'a> for x14_ClearDiagnosticInformationRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct x14_ClearDiagnosticInformationResponse;

impl x14_ClearDiagnosticInformationResponse {
    pub const LEN: usize = 0;
}

impl<'a> TryFrom<&'a [u8]> for x14_ClearDiagnosticInformationResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self)
    }
}

impl TryTo for x14_ClearDiagnosticInformationResponse {
    type Error = UdsNrc;

    fn try_to(&self, _buf: &mut [u8]) -> Result<usize, UdsNrc> {
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x14_ClearDiagnosticInformationResponse {}
