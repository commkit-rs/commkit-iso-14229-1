use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 14.5

    The RequestTransferExit service is used by the client to terminate a data transfer between client and server (upload or download)

    Supported NRC:
        - IMLOIF
        - RSE
        - ROOR
        - GPF
*/

#[allow(non_camel_case_types)]
pub struct x37_RequestTransferExit;

impl UdsService for x37_RequestTransferExit {
    const SID: u8 = 0x37;

    type Request<'a> = x37_RequestTransferExitRequest<'a>;
    type Response<'a> = x37_RequestTransferExitResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x37_RequestTransferExitRequest<'a> {
    pub transfer_request_parameter_record: &'a [u8],
}

impl<'a> x37_RequestTransferExitRequest<'a> {
    pub const fn encoded_len(&self) -> usize {
        self.transfer_request_parameter_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x37_RequestTransferExitRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        Ok(Self { transfer_request_parameter_record: data })
    }
}

impl TryTo for x37_RequestTransferExitRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..len].copy_from_slice(self.transfer_request_parameter_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x37_RequestTransferExitRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x37_RequestTransferExitResponse<'a> {
    pub transfer_response_parameter_record: &'a [u8],
}

impl<'a> x37_RequestTransferExitResponse<'a> {
    pub const fn encoded_len(&self) -> usize {
        self.transfer_response_parameter_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x37_RequestTransferExitResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        Ok(Self { transfer_response_parameter_record: data })
    }
}

impl TryTo for x37_RequestTransferExitResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..len].copy_from_slice(self.transfer_response_parameter_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x37_RequestTransferExitResponse<'a> {}
