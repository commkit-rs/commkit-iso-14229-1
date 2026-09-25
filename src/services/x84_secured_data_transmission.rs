use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.8

    The SecuredDataTransmission service is used to transmit data that is protected against attacks from third parties

    Supported NRC:
        - IMLOIF
        - RBEDLSD (0x38 - 0x4F)
*/

#[allow(non_camel_case_types)]
pub struct x84_SecuredDataTransmission;

impl UdsService for x84_SecuredDataTransmission {
    const SID: u8 = 0x84;

    type Request<'a> = x84_SecuredDataTransmissionRequest<'a>;
    type Response<'a> = x84_SecuredDataTransmissionResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x84_SecuredDataTransmissionRequest<'a> {
    pub security_data_request_record: &'a [u8],
}

impl<'a> x84_SecuredDataTransmissionRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        self.security_data_request_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x84_SecuredDataTransmissionRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { security_data_request_record: data })
    }
}

impl TryTo for x84_SecuredDataTransmissionRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if len < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..len].copy_from_slice(self.security_data_request_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x84_SecuredDataTransmissionRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x84_SecuredDataTransmissionResponse<'a> {
    pub security_data_response_record: &'a [u8],
}

impl<'a> x84_SecuredDataTransmissionResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        self.security_data_response_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x84_SecuredDataTransmissionResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { security_data_response_record: data })
    }
}

impl TryTo for x84_SecuredDataTransmissionResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if len < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..len].copy_from_slice(self.security_data_response_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x84_SecuredDataTransmissionResponse<'a> {}
