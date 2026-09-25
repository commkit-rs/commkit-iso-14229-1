use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 13.2

    The RoutineControl service is used by the client to execute a defined sequence of steps and obtain any relevant results

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - RSE
        - ROOR
        - SAD
        - GPF
*/

const RID_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutineControlType {
    StartRoutine,
    StopRoutine,
    RequestRoutineResults,
}

impl RoutineControlType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::StartRoutine),
            0x02 => Some(Self::StopRoutine),
            0x03 => Some(Self::RequestRoutineResults),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::StartRoutine => 0x01,
            Self::StopRoutine => 0x02,
            Self::RequestRoutineResults => 0x03,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x31_RoutineControl;

impl UdsService for x31_RoutineControl {
    const SID: u8 = 0x31;

    type Request<'a> = x31_RoutineControlRequest<'a>;
    type Response<'a> = x31_RoutineControlResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x31_RoutineControlRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub routine_identifier: u16,
    pub routine_control_option_record: &'a [u8],
}

impl<'a> x31_RoutineControlRequest<'a> {
    pub const MIN_LEN: usize = 1 + RID_LEN;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.routine_control_option_record.len()
    }

    pub const fn routine_control_type(&self) -> Option<RoutineControlType> {
        RoutineControlType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x31_RoutineControlRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            subfunction: UdsSubfunction::new(data[0]),
            routine_identifier: u16::from_be_bytes([data[1], data[2]]),
            routine_control_option_record: &data[Self::MIN_LEN..],
        })
    }
}

impl TryTo for x31_RoutineControlRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..Self::MIN_LEN].copy_from_slice(&self.routine_identifier.to_be_bytes());
        buf[Self::MIN_LEN..len].copy_from_slice(self.routine_control_option_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x31_RoutineControlRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x31_RoutineControlResponse<'a> {
    pub subfunction: UdsSubfunction,
    pub routine_identifier: u16,
    pub routine_status_record: &'a [u8],
}

impl<'a> x31_RoutineControlResponse<'a> {
    pub const MIN_LEN: usize = 1 + RID_LEN;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.routine_status_record.len()
    }

    pub const fn routine_control_type(&self) -> Option<RoutineControlType> {
        RoutineControlType::from_u8(self.subfunction.parameter_value())
    }

    pub const fn split_routine_info(&self) -> Option<(u8, &'a [u8])> {
        match self.routine_status_record {
            [routine_info, routine_status_record @ ..] => Some((*routine_info, routine_status_record)),
            [] => None,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for x31_RoutineControlResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            subfunction: UdsSubfunction::new(data[0]),
            routine_identifier: u16::from_be_bytes([data[1], data[2]]),
            routine_status_record: &data[Self::MIN_LEN..],
        })
    }
}

impl TryTo for x31_RoutineControlResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..Self::MIN_LEN].copy_from_slice(&self.routine_identifier.to_be_bytes());
        buf[Self::MIN_LEN..len].copy_from_slice(self.routine_status_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x31_RoutineControlResponse<'a> {}
