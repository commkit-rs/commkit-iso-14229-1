use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.4

    The ReadScalingDataByIdentifier service is used to request scaling data record information from the server identified by a dataIdentifier

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
*/

const DID_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingDataType {
    UnsignedNumeric,
    SignedNumeric,
    BitMappedReportedWithoutMask,
    BitMappedReportedWithMask,
    BinaryCodedDecimal,
    StateEncodedVariable,
    Ascii,
    SignedFloatingPoint,
    Packet,
    Formula,
    UnitFormat,
    StateAndConnectionType,
}

impl ScalingDataType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Self::UnsignedNumeric),
            0x1 => Some(Self::SignedNumeric),
            0x2 => Some(Self::BitMappedReportedWithoutMask),
            0x3 => Some(Self::BitMappedReportedWithMask),
            0x4 => Some(Self::BinaryCodedDecimal),
            0x5 => Some(Self::StateEncodedVariable),
            0x6 => Some(Self::Ascii),
            0x7 => Some(Self::SignedFloatingPoint),
            0x8 => Some(Self::Packet),
            0x9 => Some(Self::Formula),
            0xA => Some(Self::UnitFormat),
            0xB => Some(Self::StateAndConnectionType),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::UnsignedNumeric => 0x0,
            Self::SignedNumeric => 0x1,
            Self::BitMappedReportedWithoutMask => 0x2,
            Self::BitMappedReportedWithMask => 0x3,
            Self::BinaryCodedDecimal => 0x4,
            Self::StateEncodedVariable => 0x5,
            Self::Ascii => 0x6,
            Self::SignedFloatingPoint => 0x7,
            Self::Packet => 0x8,
            Self::Formula => 0x9,
            Self::UnitFormat => 0xA,
            Self::StateAndConnectionType => 0xB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalingByte(u8);

impl ScalingByte {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(data_type: ScalingDataType, number_of_bytes: u8) -> Self {
        Self((data_type.as_u8() << 4) | (number_of_bytes & 0x0F))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn data_type(self) -> Option<ScalingDataType> {
        ScalingDataType::from_u8(self.0 >> 4)
    }

    pub const fn number_of_bytes(self) -> u8 {
        self.0 & 0x0F
    }

    pub const fn extension_len(self) -> usize {
        match self.data_type() {
            Some(ScalingDataType::UnitFormat) => 1,
            Some(ScalingDataType::BitMappedReportedWithoutMask | ScalingDataType::Formula) => {
                self.number_of_bytes() as usize
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalingEntry<'a> {
    pub scaling_byte: ScalingByte,
    pub scaling_byte_extension: &'a [u8],
}

#[allow(non_camel_case_types)]
pub struct x24_ReadScalingDataByIdentifier;

impl UdsService for x24_ReadScalingDataByIdentifier {
    const SID: u8 = 0x24;

    type Request<'a> = x24_ReadScalingDataByIdentifierRequest;
    type Response<'a> = x24_ReadScalingDataByIdentifierResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x24_ReadScalingDataByIdentifierRequest {
    pub data_identifier: u16,
}

impl x24_ReadScalingDataByIdentifierRequest {
    pub const LEN: usize = DID_LEN;
}

impl<'a> TryFrom<&'a [u8]> for x24_ReadScalingDataByIdentifierRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_identifier: u16::from_be_bytes([data[0], data[1]]) })
    }
}

impl TryTo for x24_ReadScalingDataByIdentifierRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..Self::LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceRequest<'a> for x24_ReadScalingDataByIdentifierRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x24_ReadScalingDataByIdentifierResponse<'a> {
    pub data_identifier: u16,
    pub scaling_record: &'a [u8],
}

impl<'a> x24_ReadScalingDataByIdentifierResponse<'a> {
    pub const MIN_LEN: usize = DID_LEN + 1;

    pub const fn encoded_len(&self) -> usize {
        DID_LEN + self.scaling_record.len()
    }

    pub const fn entries(&self) -> x24_ScalingEntries<'a> {
        x24_ScalingEntries { remaining: self.scaling_record }
    }

    pub fn data_record_len(&self) -> Result<usize, UdsNrc> {
        self.entries().try_fold(0, |total, entry| {
            let entry = entry?;
            Ok(match entry.scaling_byte.data_type() {
                Some(ScalingDataType::Formula | ScalingDataType::UnitFormat) => total,
                _ => total + entry.scaling_byte.number_of_bytes() as usize,
            })
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for x24_ReadScalingDataByIdentifierResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data_identifier: u16::from_be_bytes([data[0], data[1]]), scaling_record: &data[DID_LEN..] })
    }
}

impl TryTo for x24_ReadScalingDataByIdentifierResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if self.scaling_record.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        buf[DID_LEN..len].copy_from_slice(self.scaling_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x24_ReadScalingDataByIdentifierResponse<'a> {}

#[allow(non_camel_case_types)]
pub struct x24_ScalingEntries<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for x24_ScalingEntries<'a> {
    type Item = Result<ScalingEntry<'a>, UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        let (&raw, rest) = self.remaining.split_first()?;
        let scaling_byte = ScalingByte::new(raw);
        let extension_len = scaling_byte.extension_len();
        if rest.len() < extension_len {
            self.remaining = &[];
            return Some(Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT));
        }
        let (scaling_byte_extension, rest) = rest.split_at(extension_len);
        self.remaining = rest;
        Some(Ok(ScalingEntry { scaling_byte, scaling_byte_extension }))
    }
}

#[allow(non_camel_case_types)]
pub struct x24_ScalingRecordWriter<'b> {
    buf: &'b mut [u8],
    len: usize,
}

impl<'b> x24_ScalingRecordWriter<'b> {
    pub fn new(data_identifier: u16, buf: &'b mut [u8]) -> Result<Self, UdsNrc> {
        if buf.len() < DID_LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&data_identifier.to_be_bytes());
        Ok(Self { buf, len: DID_LEN })
    }

    pub fn push(&mut self, scaling_byte: ScalingByte, scaling_byte_extension: &[u8]) -> Result<(), UdsNrc> {
        if scaling_byte_extension.len() != scaling_byte.extension_len() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let end = self.len + 1 + scaling_byte_extension.len();
        if self.buf.len() < end {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        self.buf[self.len] = scaling_byte.raw();
        self.buf[self.len + 1..end].copy_from_slice(scaling_byte_extension);
        self.len = end;
        Ok(())
    }

    pub fn finish(self) -> Result<usize, UdsNrc> {
        if self.len == DID_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(self.len)
    }
}
