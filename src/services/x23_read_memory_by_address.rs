use commkit::TryTo;

use crate::memory::MemoryAddressAndSize;
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.3

    The ReadMemoryByAddress service is used to request memory data from the server via a provided starting address and size of memory to be read

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
*/

#[allow(non_camel_case_types)]
pub struct x23_ReadMemoryByAddress;

impl UdsService for x23_ReadMemoryByAddress {
    const SID: u8 = 0x23;

    type Request<'a> = x23_ReadMemoryByAddressRequest;
    type Response<'a> = x23_ReadMemoryByAddressResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x23_ReadMemoryByAddressRequest {
    pub memory: MemoryAddressAndSize,
}

impl x23_ReadMemoryByAddressRequest {
    pub const MIN_LEN: usize = MemoryAddressAndSize::MIN_LEN;

    pub const fn encoded_len(&self) -> usize {
        self.memory.encoded_len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x23_ReadMemoryByAddressRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (memory, len) = MemoryAddressAndSize::decode(data)?;
        if data.len() != len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { memory })
    }
}

impl TryTo for x23_ReadMemoryByAddressRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.memory.encode(buf)
    }
}

impl<'a> UdsServiceRequest<'a> for x23_ReadMemoryByAddressRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x23_ReadMemoryByAddressResponse<'a> {
    pub data_record: &'a [u8],
}

impl<'a> x23_ReadMemoryByAddressResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        self.data_record.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x23_ReadMemoryByAddressResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_record: data })
    }
}

impl TryTo for x23_ReadMemoryByAddressResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if len < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..len].copy_from_slice(self.data_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x23_ReadMemoryByAddressResponse<'a> {}
