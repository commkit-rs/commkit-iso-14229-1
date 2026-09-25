use commkit::TryTo;

use crate::memory::MemoryAddressAndSize;
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.8

    The WriteMemoryByAddress service is used to write information into the server at one or more contiguous memory locations

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
        - GPF
*/

#[allow(non_camel_case_types)]
pub struct x3D_WriteMemoryByAddress;

impl UdsService for x3D_WriteMemoryByAddress {
    const SID: u8 = 0x3D;

    type Request<'a> = x3D_WriteMemoryByAddressRequest<'a>;
    type Response<'a> = x3D_WriteMemoryByAddressResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x3D_WriteMemoryByAddressRequest<'a> {
    pub memory: MemoryAddressAndSize,
    pub data_record: &'a [u8],
}

impl<'a> x3D_WriteMemoryByAddressRequest<'a> {
    pub const MIN_LEN: usize = MemoryAddressAndSize::MIN_LEN + 1;

    pub const fn new(memory_address: u64, data_record: &'a [u8]) -> Self {
        Self { memory: MemoryAddressAndSize::new(memory_address, data_record.len() as u64), data_record }
    }

    pub const fn encoded_len(&self) -> usize {
        self.memory.encoded_len() + self.data_record.len()
    }

    fn validate(&self) -> Result<(), UdsNrc> {
        if self.data_record.is_empty() || self.data_record.len() as u64 != self.memory.memory_size {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a [u8]> for x3D_WriteMemoryByAddressRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (memory, len) = MemoryAddressAndSize::decode(data)?;
        let request = Self { memory, data_record: &data[len..] };
        request.validate()?;
        Ok(request)
    }
}

impl TryTo for x3D_WriteMemoryByAddressRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.validate()?;
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        let memory_len = self.memory.encode(buf)?;
        buf[memory_len..len].copy_from_slice(self.data_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x3D_WriteMemoryByAddressRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x3D_WriteMemoryByAddressResponse {
    pub memory: MemoryAddressAndSize,
}

impl x3D_WriteMemoryByAddressResponse {
    pub const MIN_LEN: usize = MemoryAddressAndSize::MIN_LEN;

    pub const fn encoded_len(&self) -> usize {
        self.memory.encoded_len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x3D_WriteMemoryByAddressResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (memory, len) = MemoryAddressAndSize::decode(data)?;
        if data.len() != len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { memory })
    }
}

impl TryTo for x3D_WriteMemoryByAddressResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.memory.encode(buf)
    }
}

impl<'a> UdsServiceResponse<'a> for x3D_WriteMemoryByAddressResponse {}
