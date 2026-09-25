use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 10.5

    The ReadDataByPeriodicIdentifier service is used to request the periodic transmission of data record values from the server identified by one or more periodicDataIdentifiers

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmissionMode {
    SendAtSlowRate,
    SendAtMediumRate,
    SendAtFastRate,
    StopSending,
}

impl TransmissionMode {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::SendAtSlowRate),
            0x02 => Some(Self::SendAtMediumRate),
            0x03 => Some(Self::SendAtFastRate),
            0x04 => Some(Self::StopSending),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::SendAtSlowRate => 0x01,
            Self::SendAtMediumRate => 0x02,
            Self::SendAtFastRate => 0x03,
            Self::StopSending => 0x04,
        }
    }
}

pub const fn periodic_data_identifier_to_data_identifier(periodic_data_identifier: u8) -> u16 {
    0xF200 | periodic_data_identifier as u16
}

pub const fn data_identifier_to_periodic_data_identifier(data_identifier: u16) -> Option<u8> {
    match data_identifier {
        0xF200..=0xF2FF => Some(data_identifier as u8),
        _ => None,
    }
}

#[allow(non_camel_case_types)]
pub struct x2A_ReadDataByPeriodicIdentifier;

impl UdsService for x2A_ReadDataByPeriodicIdentifier {
    const SID: u8 = 0x2A;

    type Request<'a> = x2A_ReadDataByPeriodicIdentifierRequest<'a>;
    type Response<'a> = x2A_ReadDataByPeriodicIdentifierResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2A_ReadDataByPeriodicIdentifierRequest<'a> {
    pub transmission_mode: u8,
    pub periodic_data_identifiers: &'a [u8],
}

impl<'a> x2A_ReadDataByPeriodicIdentifierRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.periodic_data_identifiers.len()
    }

    pub const fn transmission_mode(&self) -> Option<TransmissionMode> {
        TransmissionMode::from_u8(self.transmission_mode)
    }

    pub fn data_identifiers(&self) -> impl Iterator<Item = u16> + 'a {
        self.periodic_data_identifiers.iter().map(|&pdid| periodic_data_identifier_to_data_identifier(pdid))
    }

    const fn has_valid_length(&self) -> bool {
        matches!(self.transmission_mode(), Some(TransmissionMode::StopSending))
            || !self.periodic_data_identifiers.is_empty()
    }
}

impl<'a> TryFrom<&'a [u8]> for x2A_ReadDataByPeriodicIdentifierRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&transmission_mode, periodic_data_identifiers) =
            data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let request = Self { transmission_mode, periodic_data_identifiers };
        if !request.has_valid_length() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(request)
    }
}

impl TryTo for x2A_ReadDataByPeriodicIdentifierRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if !self.has_valid_length() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.transmission_mode;
        buf[1..len].copy_from_slice(self.periodic_data_identifiers);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x2A_ReadDataByPeriodicIdentifierRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct x2A_ReadDataByPeriodicIdentifierResponse;

impl x2A_ReadDataByPeriodicIdentifierResponse {
    pub const LEN: usize = 0;
}

impl<'a> TryFrom<&'a [u8]> for x2A_ReadDataByPeriodicIdentifierResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self)
    }
}

impl TryTo for x2A_ReadDataByPeriodicIdentifierResponse {
    type Error = UdsNrc;

    fn try_to(&self, _buf: &mut [u8]) -> Result<usize, UdsNrc> {
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x2A_ReadDataByPeriodicIdentifierResponse {}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2A_PeriodicDataResponse<'a> {
    pub periodic_data_identifier: u8,
    pub data_record: &'a [u8],
}

impl<'a> x2A_PeriodicDataResponse<'a> {
    pub const MIN_LEN: usize = 2;

    pub const fn encoded_len(&self) -> usize {
        1 + self.data_record.len()
    }

    pub const fn data_identifier(&self) -> u16 {
        periodic_data_identifier_to_data_identifier(self.periodic_data_identifier)
    }
}

impl<'a> TryFrom<&'a [u8]> for x2A_PeriodicDataResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { periodic_data_identifier: data[0], data_record: &data[1..] })
    }
}

impl TryTo for x2A_PeriodicDataResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if self.data_record.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.periodic_data_identifier;
        buf[1..len].copy_from_slice(self.data_record);
        Ok(len)
    }
}
