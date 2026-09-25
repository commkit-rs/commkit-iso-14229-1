use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.11

    The LinkControl service is used to control the communication between the client and the server(s) in order to gain bus bandwidth for diagnostic purposes

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - RSE
        - ROOR
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkControlType {
    VerifyModeTransitionWithFixedParameter,
    VerifyModeTransitionWithSpecificParameter,
    TransitionMode,
    VehicleManufacturerSpecific(u8),
    SystemSupplierSpecific(u8),
}

impl LinkControlType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::VerifyModeTransitionWithFixedParameter),
            0x02 => Some(Self::VerifyModeTransitionWithSpecificParameter),
            0x03 => Some(Self::TransitionMode),
            0x40..=0x5F => Some(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::VerifyModeTransitionWithFixedParameter => 0x01,
            Self::VerifyModeTransitionWithSpecificParameter => 0x02,
            Self::TransitionMode => 0x03,
            Self::VehicleManufacturerSpecific(value) => value,
            Self::SystemSupplierSpecific(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkControlModeIdentifier {
    Pc9600Baud,
    Pc19200Baud,
    Pc38400Baud,
    Pc57600Baud,
    Pc115200Baud,
    Can125000Baud,
    Can250000Baud,
    Can500000Baud,
    Can1000000Baud,
    ProgrammingSetup,
}

impl LinkControlModeIdentifier {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Pc9600Baud),
            0x02 => Some(Self::Pc19200Baud),
            0x03 => Some(Self::Pc38400Baud),
            0x04 => Some(Self::Pc57600Baud),
            0x05 => Some(Self::Pc115200Baud),
            0x10 => Some(Self::Can125000Baud),
            0x11 => Some(Self::Can250000Baud),
            0x12 => Some(Self::Can500000Baud),
            0x13 => Some(Self::Can1000000Baud),
            0x20 => Some(Self::ProgrammingSetup),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Pc9600Baud => 0x01,
            Self::Pc19200Baud => 0x02,
            Self::Pc38400Baud => 0x03,
            Self::Pc57600Baud => 0x04,
            Self::Pc115200Baud => 0x05,
            Self::Can125000Baud => 0x10,
            Self::Can250000Baud => 0x11,
            Self::Can500000Baud => 0x12,
            Self::Can1000000Baud => 0x13,
            Self::ProgrammingSetup => 0x20,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x87_LinkControl;

impl UdsService for x87_LinkControl {
    const SID: u8 = 0x87;

    type Request<'a> = x87_LinkControlRequest<'a>;
    type Response<'a> = x87_LinkControlResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x87_LinkControlRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub data: &'a [u8],
}

impl<'a> x87_LinkControlRequest<'a> {
    pub const MIN_LEN: usize = 1;
    pub const LINK_RECORD_LEN: usize = 3;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.data.len()
    }

    pub const fn link_control_type(&self) -> Option<LinkControlType> {
        LinkControlType::from_u8(self.subfunction.parameter_value())
    }

    pub const fn link_control_mode_identifier(&self) -> Option<LinkControlModeIdentifier> {
        match (self.link_control_type(), self.data) {
            (Some(LinkControlType::VerifyModeTransitionWithFixedParameter), [identifier]) => {
                LinkControlModeIdentifier::from_u8(*identifier)
            }
            _ => None,
        }
    }

    pub const fn link_record(&self) -> Option<u32> {
        match (self.link_control_type(), self.data) {
            (Some(LinkControlType::VerifyModeTransitionWithSpecificParameter), [high, middle, low]) => {
                Some(u32::from_be_bytes([0, *high, *middle, *low]))
            }
            _ => None,
        }
    }

    pub const fn link_record_bytes(mode_parameter: u32) -> [u8; 3] {
        let [_, high, middle, low] = mode_parameter.to_be_bytes();
        [high, middle, low]
    }
}

impl<'a> TryFrom<&'a [u8]> for x87_LinkControlRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), data: &data[1..] })
    }
}

impl TryTo for x87_LinkControlRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.data);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x87_LinkControlRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x87_LinkControlResponse {
    pub subfunction: UdsSubfunction,
}

impl x87_LinkControlResponse {
    pub const LEN: usize = 1;

    pub const fn link_control_type(&self) -> Option<LinkControlType> {
        LinkControlType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x87_LinkControlResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }
}

impl TryTo for x87_LinkControlResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x87_LinkControlResponse {}
