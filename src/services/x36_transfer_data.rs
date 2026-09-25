use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 14.4

    The TransferData service is used by the client to transfer data either from the client to the server (download) or from the server to the client (upload)

    Supported NRC:
        - IMLOIF
        - RSE
        - ROOR
        - TDS
        - GPF
        - WBSC
        - VTH
        - VTL
*/

pub const INITIAL_BLOCK_SEQUENCE_COUNTER: u8 = 0x01;

pub const fn next_block_sequence_counter(block_sequence_counter: u8) -> u8 {
    block_sequence_counter.wrapping_add(1)
}

#[allow(non_camel_case_types)]
pub struct x36_TransferData;

impl UdsService for x36_TransferData {
    const SID: u8 = 0x36;

    type Request<'a> = x36_TransferDataRequest<'a>;
    type Response<'a> = x36_TransferDataResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x36_TransferDataRequest<'a> {
    pub block_sequence_counter: u8,
    pub transfer_request_parameter_record: &'a [u8],
}

impl<'a> x36_TransferDataRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.transfer_request_parameter_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x36_TransferDataRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&block_sequence_counter, transfer_request_parameter_record) =
            data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        Ok(Self { block_sequence_counter, transfer_request_parameter_record })
    }
}

impl TryTo for x36_TransferDataRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.block_sequence_counter;
        buf[Self::MIN_LEN..len].copy_from_slice(self.transfer_request_parameter_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x36_TransferDataRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x36_TransferDataResponse<'a> {
    pub block_sequence_counter: u8,
    pub transfer_response_parameter_record: &'a [u8],
}

impl<'a> x36_TransferDataResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.transfer_response_parameter_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x36_TransferDataResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&block_sequence_counter, transfer_response_parameter_record) =
            data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        Ok(Self { block_sequence_counter, transfer_response_parameter_record })
    }
}

impl TryTo for x36_TransferDataResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.block_sequence_counter;
        buf[Self::MIN_LEN..len].copy_from_slice(self.transfer_response_parameter_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x36_TransferDataResponse<'a> {}
