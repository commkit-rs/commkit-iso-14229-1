use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.7

    The WriteDataByIdentifier service is used to write a data record into the server at an internal location specified by the provided dataIdentifier

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
        - GPF
*/

const DID_LEN: usize = 2;

#[allow(non_camel_case_types)]
pub struct x2E_WriteDataByIdentifier;

impl UdsService for x2E_WriteDataByIdentifier {
    const SID: u8 = 0x2E;

    type Request<'a> = x2E_WriteDataByIdentifierRequest<'a>;
    type Response<'a> = x2E_WriteDataByIdentifierResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2E_WriteDataByIdentifierRequest<'a> {
    pub data_identifier: u16,
    pub data_record: &'a [u8],
}

impl<'a> x2E_WriteDataByIdentifierRequest<'a> {
    pub const MIN_LEN: usize = DID_LEN + 1;

    pub const fn encoded_len(&self) -> usize {
        DID_LEN + self.data_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x2E_WriteDataByIdentifierRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_identifier: u16::from_be_bytes([data[0], data[1]]), data_record: &data[DID_LEN..] })
    }
}

impl TryTo for x2E_WriteDataByIdentifierRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if self.data_record.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        buf[DID_LEN..len].copy_from_slice(self.data_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x2E_WriteDataByIdentifierRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2E_WriteDataByIdentifierResponse {
    pub data_identifier: u16,
}

impl x2E_WriteDataByIdentifierResponse {
    pub const LEN: usize = DID_LEN;
}

impl<'a> TryFrom<&'a [u8]> for x2E_WriteDataByIdentifierResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_identifier: u16::from_be_bytes([data[0], data[1]]) })
    }
}

impl TryTo for x2E_WriteDataByIdentifierResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..Self::LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x2E_WriteDataByIdentifierResponse {}
