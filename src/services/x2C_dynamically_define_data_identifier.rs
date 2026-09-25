use commkit::TryTo;

use crate::memory::{self, AddressAndLengthFormatIdentifier, MemoryAddressAndSize};
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.6

    The DynamicallyDefineDataIdentifier service is used to dynamically define a data identifier in the server that can be read via the ReadDataByIdentifier service at a later time

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - ROOR
        - SAD
*/

const DID_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionType {
    DefineByIdentifier,
    DefineByMemoryAddress,
    ClearDynamicallyDefinedDataIdentifier,
}

impl DefinitionType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::DefineByIdentifier),
            0x02 => Some(Self::DefineByMemoryAddress),
            0x03 => Some(Self::ClearDynamicallyDefinedDataIdentifier),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::DefineByIdentifier => 0x01,
            Self::DefineByMemoryAddress => 0x02,
            Self::ClearDynamicallyDefinedDataIdentifier => 0x03,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceDataDefinition {
    pub source_data_identifier: u16,
    pub position_in_source_data_record: u8,
    pub memory_size: u8,
}

impl SourceDataDefinition {
    pub const LEN: usize = 4;

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            source_data_identifier: u16::from_be_bytes([data[0], data[1]]),
            position_in_source_data_record: data[2],
            memory_size: data[3],
        })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&self.source_data_identifier.to_be_bytes());
        buf[2] = self.position_in_source_data_record;
        buf[3] = self.memory_size;
        Ok(Self::LEN)
    }
}

#[allow(non_camel_case_types)]
pub struct x2C_DynamicallyDefineDataIdentifier;

impl UdsService for x2C_DynamicallyDefineDataIdentifier {
    const SID: u8 = 0x2C;

    type Request<'a> = x2C_DynamicallyDefineDataIdentifierRequest<'a>;
    type Response<'a> = x2C_DynamicallyDefineDataIdentifierResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2C_DynamicallyDefineDataIdentifierRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub dynamically_defined_data_identifier: Option<u16>,
    pub definition_record: &'a [u8],
}

impl<'a> x2C_DynamicallyDefineDataIdentifierRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        let did_len = if self.dynamically_defined_data_identifier.is_some() { DID_LEN } else { 0 };
        Self::MIN_LEN + did_len + self.definition_record.len()
    }

    pub const fn definition_type(&self) -> Option<DefinitionType> {
        DefinitionType::from_u8(self.subfunction.parameter_value())
    }

    pub fn source_data_definitions(&self) -> Option<impl Iterator<Item = SourceDataDefinition> + 'a> {
        match self.definition_type() {
            Some(DefinitionType::DefineByIdentifier) => Some(
                self.definition_record
                    .chunks_exact(SourceDataDefinition::LEN)
                    .map(|chunk| SourceDataDefinition {
                        source_data_identifier: u16::from_be_bytes([chunk[0], chunk[1]]),
                        position_in_source_data_record: chunk[2],
                        memory_size: chunk[3],
                    }),
            ),
            _ => None,
        }
    }

    pub fn memory_definitions(&self) -> Option<impl Iterator<Item = MemoryAddressAndSize> + 'a> {
        match self.definition_type() {
            Some(DefinitionType::DefineByMemoryAddress) => {
                let (&raw, fields) = self.definition_record.split_first()?;
                let format = AddressAndLengthFormatIdentifier::new(raw);
                if !format.is_valid() {
                    return None;
                }
                Some(fields.chunks_exact(format.fields_len()).map(move |chunk| {
                    let (memory_address, memory_size) = memory::decode_fields(format, chunk);
                    MemoryAddressAndSize { address_and_length_format_identifier: format, memory_address, memory_size }
                }))
            }
            _ => None,
        }
    }

    pub fn encode_define_by_identifier(
        suppress_positive_response: bool,
        dynamically_defined_data_identifier: u16,
        definitions: &[SourceDataDefinition],
        buf: &mut [u8],
    ) -> Result<usize, UdsNrc> {
        if definitions.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = Self::MIN_LEN + DID_LEN + definitions.len() * SourceDataDefinition::LEN;
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        Self::encode_header(DefinitionType::DefineByIdentifier, suppress_positive_response, Some(dynamically_defined_data_identifier), buf);
        for (chunk, definition) in buf[Self::MIN_LEN + DID_LEN..len].chunks_exact_mut(SourceDataDefinition::LEN).zip(definitions) {
            definition.encode(chunk)?;
        }
        Ok(len)
    }

    pub fn encode_define_by_memory_address(
        suppress_positive_response: bool,
        dynamically_defined_data_identifier: u16,
        address_and_length_format_identifier: AddressAndLengthFormatIdentifier,
        memory_ranges: &[(u64, u64)],
        buf: &mut [u8],
    ) -> Result<usize, UdsNrc> {
        let format = address_and_length_format_identifier;
        if memory_ranges.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if !format.is_valid() {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let header_len = Self::MIN_LEN + DID_LEN + AddressAndLengthFormatIdentifier::LEN;
        let len = header_len + memory_ranges.len() * format.fields_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        Self::encode_header(DefinitionType::DefineByMemoryAddress, suppress_positive_response, Some(dynamically_defined_data_identifier), buf);
        buf[header_len - 1] = format.raw();
        for (chunk, &(memory_address, memory_size)) in buf[header_len..len].chunks_exact_mut(format.fields_len()).zip(memory_ranges) {
            memory::encode_fields(format, memory_address, memory_size, chunk)?;
        }
        Ok(len)
    }

    pub fn encode_clear(
        suppress_positive_response: bool,
        dynamically_defined_data_identifier: Option<u16>,
        buf: &mut [u8],
    ) -> Result<usize, UdsNrc> {
        let len = Self::MIN_LEN + if dynamically_defined_data_identifier.is_some() { DID_LEN } else { 0 };
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        Self::encode_header(
            DefinitionType::ClearDynamicallyDefinedDataIdentifier,
            suppress_positive_response,
            dynamically_defined_data_identifier,
            buf,
        );
        Ok(len)
    }

    fn encode_header(definition_type: DefinitionType, suppress_positive_response: bool, did: Option<u16>, buf: &mut [u8]) {
        buf[0] = UdsSubfunction::from_parts(definition_type.as_u8(), suppress_positive_response).raw();
        if let Some(did) = did {
            buf[1..1 + DID_LEN].copy_from_slice(&did.to_be_bytes());
        }
    }

    fn validate(&self) -> Result<(), UdsNrc> {
        let record = self.definition_record;
        let has_did = self.dynamically_defined_data_identifier.is_some();
        match self.definition_type() {
            Some(DefinitionType::DefineByIdentifier) => {
                if !has_did || record.is_empty() || !record.len().is_multiple_of(SourceDataDefinition::LEN) {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
            }
            Some(DefinitionType::DefineByMemoryAddress) => {
                let (&raw, fields) =
                    record.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
                if !has_did {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
                let format = AddressAndLengthFormatIdentifier::new(raw);
                if !format.is_valid() {
                    return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
                }
                if fields.is_empty() || !fields.len().is_multiple_of(format.fields_len()) {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
            }
            Some(DefinitionType::ClearDynamicallyDefinedDataIdentifier) => {
                if !record.is_empty() {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a [u8]> for x2C_DynamicallyDefineDataIdentifierRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&subfunction, rest) =
            data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let subfunction = UdsSubfunction::new(subfunction);

        let (dynamically_defined_data_identifier, definition_record) = match rest.split_first_chunk::<DID_LEN>() {
            Some((did, record)) => (Some(u16::from_be_bytes(*did)), record),
            None if rest.is_empty() => (None, rest),
            None => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
        };

        let request = Self { subfunction, dynamically_defined_data_identifier, definition_record };
        request.validate()?;
        Ok(request)
    }
}

impl TryTo for x2C_DynamicallyDefineDataIdentifierRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        self.validate()?;
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        let mut offset = 1;
        if let Some(did) = self.dynamically_defined_data_identifier {
            buf[offset..offset + DID_LEN].copy_from_slice(&did.to_be_bytes());
            offset += DID_LEN;
        }
        buf[offset..len].copy_from_slice(self.definition_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x2C_DynamicallyDefineDataIdentifierRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2C_DynamicallyDefineDataIdentifierResponse {
    pub subfunction: UdsSubfunction,
    pub dynamically_defined_data_identifier: Option<u16>,
}

impl x2C_DynamicallyDefineDataIdentifierResponse {
    pub const MIN_LEN: usize = 1;
    pub const MAX_LEN: usize = Self::MIN_LEN + DID_LEN;

    pub const fn encoded_len(&self) -> usize {
        if self.dynamically_defined_data_identifier.is_some() { Self::MAX_LEN } else { Self::MIN_LEN }
    }

    pub const fn definition_type(&self) -> Option<DefinitionType> {
        DefinitionType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x2C_DynamicallyDefineDataIdentifierResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let dynamically_defined_data_identifier = match data.len() {
            Self::MIN_LEN => None,
            Self::MAX_LEN => Some(u16::from_be_bytes([data[1], data[2]])),
            _ => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
        };
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), dynamically_defined_data_identifier })
    }
}

impl TryTo for x2C_DynamicallyDefineDataIdentifierResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        if let Some(did) = self.dynamically_defined_data_identifier {
            buf[1..Self::MAX_LEN].copy_from_slice(&did.to_be_bytes());
        }
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x2C_DynamicallyDefineDataIdentifierResponse {}
