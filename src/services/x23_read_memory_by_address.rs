use crate::memory::MemoryAddressAndSize;
use crate::nrc::UdsNrc;
use crate::service::UdsService;

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

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        let (memory, len) = MemoryAddressAndSize::decode(data)?;
        if data.len() != len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { memory })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.memory.encode(buf)
    }

    pub const fn encoded_len(&self) -> usize {
        self.memory.encoded_len()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x23_ReadMemoryByAddressResponse<'a> {
    pub data_record: &'a [u8],
}

impl<'a> x23_ReadMemoryByAddressResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub fn decode(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_record: data })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
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

    pub const fn encoded_len(&self) -> usize {
        self.data_record.len()
    }
}
