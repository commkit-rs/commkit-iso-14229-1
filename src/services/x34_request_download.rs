use commkit::TryTo;

use crate::memory::{DataFormatIdentifier, MaxNumberOfBlockLength, MemoryAddressAndSize};
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 14.2

    The RequestDownload service is used by the client to initiate a data transfer from the client to the server (download)

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
        - UDNA
*/

#[allow(non_camel_case_types)]
pub struct x34_RequestDownload;

impl UdsService for x34_RequestDownload {
    const SID: u8 = 0x34;

    type Request<'a> = x34_RequestDownloadRequest;
    type Response<'a> = x34_RequestDownloadResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x34_RequestDownloadRequest {
    pub data_format_identifier: DataFormatIdentifier,
    pub memory: MemoryAddressAndSize,
}

impl x34_RequestDownloadRequest {
    pub const MIN_LEN: usize = DataFormatIdentifier::LEN + MemoryAddressAndSize::MIN_LEN;

    pub const fn new(data_format_identifier: DataFormatIdentifier, memory_address: u64, memory_size: u64) -> Self {
        Self { data_format_identifier, memory: MemoryAddressAndSize::new(memory_address, memory_size) }
    }

    pub const fn encoded_len(&self) -> usize {
        DataFormatIdentifier::LEN + self.memory.encoded_len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x34_RequestDownloadRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (memory, len) = MemoryAddressAndSize::decode(&data[DataFormatIdentifier::LEN..])?;
        if data.len() != DataFormatIdentifier::LEN + len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_format_identifier: DataFormatIdentifier::new(data[0]), memory })
    }
}

impl TryTo for x34_RequestDownloadRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.data_format_identifier.raw();
        self.memory.encode(&mut buf[DataFormatIdentifier::LEN..len])?;
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x34_RequestDownloadRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x34_RequestDownloadResponse {
    pub max_number_of_block_length: MaxNumberOfBlockLength,
}

impl x34_RequestDownloadResponse {
    pub const MIN_LEN: usize = MaxNumberOfBlockLength::MIN_LEN;

    pub const fn new(max_number_of_block_length: u64) -> Self {
        Self { max_number_of_block_length: MaxNumberOfBlockLength::new(max_number_of_block_length) }
    }

    pub const fn encoded_len(&self) -> usize {
        self.max_number_of_block_length.encoded_len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x34_RequestDownloadResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        Ok(Self { max_number_of_block_length: MaxNumberOfBlockLength::decode(data)? })
    }
}

impl TryTo for x34_RequestDownloadResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.max_number_of_block_length.encode(buf)
    }
}

impl<'a> UdsServiceResponse<'a> for x34_RequestDownloadResponse {}
