use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 12.2

    The InputOutputControlByIdentifier service is used to substitute a value for an input signal, internal server function and/or force control to a value for an output of an electronic system

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
*/

const DID_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputOutputControlParameter {
    ReturnControlToEcu,
    ResetToDefault,
    FreezeCurrentState,
    ShortTermAdjustment,
}

impl InputOutputControlParameter {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::ReturnControlToEcu),
            0x01 => Some(Self::ResetToDefault),
            0x02 => Some(Self::FreezeCurrentState),
            0x03 => Some(Self::ShortTermAdjustment),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::ReturnControlToEcu => 0x00,
            Self::ResetToDefault => 0x01,
            Self::FreezeCurrentState => 0x02,
            Self::ShortTermAdjustment => 0x03,
        }
    }

    pub const fn has_control_state(self) -> bool {
        matches!(self, Self::ShortTermAdjustment)
    }
}

#[allow(non_camel_case_types)]
pub struct x2F_InputOutputControlByIdentifier;

impl UdsService for x2F_InputOutputControlByIdentifier {
    const SID: u8 = 0x2F;

    type Request<'a> = x2F_InputOutputControlByIdentifierRequest<'a>;
    type Response<'a> = x2F_InputOutputControlByIdentifierResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2F_InputOutputControlByIdentifierRequest<'a> {
    pub data_identifier: u16,
    pub input_output_control_parameter: u8,
    pub control_option_record: &'a [u8],
}

impl<'a> x2F_InputOutputControlByIdentifierRequest<'a> {
    pub const MIN_LEN: usize = DID_LEN + 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.control_option_record.len()
    }

    pub const fn input_output_control_parameter(&self) -> Option<InputOutputControlParameter> {
        InputOutputControlParameter::from_u8(self.input_output_control_parameter)
    }

    pub fn control_state_and_enable_mask(&self, control_state_len: usize) -> Result<(&'a [u8], &'a [u8]), UdsNrc> {
        let control_state_len = match self.input_output_control_parameter() {
            Some(parameter) if parameter.has_control_state() => control_state_len,
            _ => 0,
        };
        if self.control_option_record.len() < control_state_len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(self.control_option_record.split_at(control_state_len))
    }

    pub fn encode_parts(
        data_identifier: u16,
        input_output_control_parameter: u8,
        control_state: &[u8],
        control_enable_mask: &[u8],
        buf: &mut [u8],
    ) -> Result<usize, UdsNrc> {
        let len = Self::MIN_LEN + control_state.len() + control_enable_mask.len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        let state_end = Self::MIN_LEN + control_state.len();
        buf[..DID_LEN].copy_from_slice(&data_identifier.to_be_bytes());
        buf[DID_LEN] = input_output_control_parameter;
        buf[Self::MIN_LEN..state_end].copy_from_slice(control_state);
        buf[state_end..len].copy_from_slice(control_enable_mask);
        Ok(len)
    }
}

impl<'a> TryFrom<&'a [u8]> for x2F_InputOutputControlByIdentifierRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            data_identifier: u16::from_be_bytes([data[0], data[1]]),
            input_output_control_parameter: data[2],
            control_option_record: &data[Self::MIN_LEN..],
        })
    }
}

impl TryTo for x2F_InputOutputControlByIdentifierRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        buf[DID_LEN] = self.input_output_control_parameter;
        buf[Self::MIN_LEN..len].copy_from_slice(self.control_option_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x2F_InputOutputControlByIdentifierRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x2F_InputOutputControlByIdentifierResponse<'a> {
    pub data_identifier: u16,
    pub input_output_control_parameter: u8,
    pub control_state: &'a [u8],
}

impl<'a> x2F_InputOutputControlByIdentifierResponse<'a> {
    pub const MIN_LEN: usize = DID_LEN + 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.control_state.len()
    }

    pub const fn input_output_control_parameter(&self) -> Option<InputOutputControlParameter> {
        InputOutputControlParameter::from_u8(self.input_output_control_parameter)
    }
}

impl<'a> TryFrom<&'a [u8]> for x2F_InputOutputControlByIdentifierResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            data_identifier: u16::from_be_bytes([data[0], data[1]]),
            input_output_control_parameter: data[2],
            control_state: &data[Self::MIN_LEN..],
        })
    }
}

impl TryTo for x2F_InputOutputControlByIdentifierResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[..DID_LEN].copy_from_slice(&self.data_identifier.to_be_bytes());
        buf[DID_LEN] = self.input_output_control_parameter;
        buf[Self::MIN_LEN..len].copy_from_slice(self.control_state);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x2F_InputOutputControlByIdentifierResponse<'a> {}
