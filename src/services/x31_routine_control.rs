use crate::nrc::UdsNrc;
use crate::service::UdsService;
use crate::subfunction::UdsSubfunction;

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
    pub routine_control_option_data: &'a [u8],
}

impl<'a> x31_RoutineControlRequest<'a> {
    pub fn decode(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < 3 {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            subfunction: UdsSubfunction::new(data[0]),
            routine_identifier: u16::from_be_bytes([data[1], data[2]]),
            routine_control_option_data: &data[3..],
        })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = 3 + self.routine_control_option_data.len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }

        buf[0] = self.subfunction.raw();
        buf[1..3].copy_from_slice(&self.routine_identifier.to_be_bytes());
        buf[3..len].copy_from_slice(self.routine_control_option_data);

        Ok(len)
    }

    pub fn routine_control_type(&self) -> Option<RoutineControlType> {
        RoutineControlType::from_u8(self.subfunction.parameter_value())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x31_RoutineControlResponse<'a> {
    pub subfunction: UdsSubfunction,
    pub routine_identifier: u16,
    pub routine_info: u8,
    pub routine_status_record: &'a [u8],
}

impl<'a> x31_RoutineControlResponse<'a> {
    pub fn decode(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < 4 {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self {
            subfunction: UdsSubfunction::new(data[0]),
            routine_identifier: u16::from_be_bytes([data[1], data[2]]),
            routine_info: data[3],
            routine_status_record: &data[4..],
        })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = 4 + self.routine_status_record.len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }

        buf[0] = self.subfunction.raw();
        buf[1..3].copy_from_slice(&self.routine_identifier.to_be_bytes());
        buf[3] = self.routine_info;
        buf[4..len].copy_from_slice(self.routine_status_record);

        Ok(len)
    }

    pub fn routine_control_type(&self) -> Option<RoutineControlType> {
        RoutineControlType::from_u8(self.subfunction.parameter_value())
    }
}
