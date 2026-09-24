use commkit::Duration;

use crate::nrc::UdsNrc;
use crate::service::UdsService;
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.2

    The DiagnosticSessionControl service is used to enable different diagnostic sessions in the server(s)

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
*/

/// 9.2.2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSessionType {
    DefaultSession,
    ProgrammingSession,
    ExtendedDiagnosticSession,
    SafetySystemDiagnosticSession,
    VehicleManufacturerSpecific(u8),
    SystemSupplierSpecific(u8),
}

impl DiagnosticSessionType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::DefaultSession),
            0x02 => Some(Self::ProgrammingSession),
            0x03 => Some(Self::ExtendedDiagnosticSession),
            0x04 => Some(Self::SafetySystemDiagnosticSession),
            0x40..=0x5F => Some(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::DefaultSession => 0x01,
            Self::ProgrammingSession => 0x02,
            Self::ExtendedDiagnosticSession => 0x03,
            Self::SafetySystemDiagnosticSession => 0x04,
            Self::VehicleManufacturerSpecific(value) => value,
            Self::SystemSupplierSpecific(value) => value,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x10_DiagnosticSessionControl;

impl UdsService for x10_DiagnosticSessionControl {
    const SID: u8 = 0x10;

    type Request<'a> = x10_DiagnosticSessionControlRequest;
    type Response<'a> = x10_DiagnosticSessionControlResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x10_DiagnosticSessionControlRequest {
    pub subfunction: UdsSubfunction,
}

impl x10_DiagnosticSessionControlRequest {
    pub const LEN: usize = 1;

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }

    pub const fn diagnostic_session_type(&self) -> Option<DiagnosticSessionType> {
        DiagnosticSessionType::from_u8(self.subfunction.parameter_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionParameterRecord {
    pub p2_server_max: Duration,
    pub p2_star_server_max: Duration,
}

impl SessionParameterRecord {
    pub const LEN: usize = 4;

    const P2_RESOLUTION_US: u64 = 1_000;
    const P2_STAR_RESOLUTION_US: u64 = 10_000;

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let p2 = u16::from_be_bytes([data[0], data[1]]) as u64;
        let p2_star = u16::from_be_bytes([data[2], data[3]]) as u64;
        Ok(Self {
            p2_server_max: Duration::from_ticks(p2 * Self::P2_RESOLUTION_US),
            p2_star_server_max: Duration::from_ticks(p2_star * Self::P2_STAR_RESOLUTION_US),
        })
    }

    /// Durations are rounded up to the parameter resolution and clamped to the encodable maximum.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        let p2 = Self::to_counts(self.p2_server_max, Self::P2_RESOLUTION_US);
        let p2_star = Self::to_counts(self.p2_star_server_max, Self::P2_STAR_RESOLUTION_US);
        buf[0..2].copy_from_slice(&p2.to_be_bytes());
        buf[2..4].copy_from_slice(&p2_star.to_be_bytes());
        Ok(Self::LEN)
    }

    fn to_counts(duration: Duration, resolution_us: u64) -> u16 {
        duration.as_ticks().div_ceil(resolution_us).min(u16::MAX as u64) as u16
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x10_DiagnosticSessionControlResponse {
    //  Subfunction parameter value = diagnostic session type
    pub subfunction: UdsSubfunction,
    pub session_parameter_record: Option<SessionParameterRecord>,
}

impl x10_DiagnosticSessionControlResponse {
    pub const MIN_LEN: usize = 1;
    pub const MAX_LEN: usize = Self::MIN_LEN + SessionParameterRecord::LEN;

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        let session_parameter_record = match data.len() {
            Self::MIN_LEN => None,
            Self::MAX_LEN => Some(SessionParameterRecord::decode(&data[1..])?),
            _ => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
        };
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), session_parameter_record })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        if let Some(record) = &self.session_parameter_record {
            record.encode(&mut buf[1..])?;
        }
        Ok(len)
    }

    pub const fn encoded_len(&self) -> usize {
        if self.session_parameter_record.is_some() { Self::MAX_LEN } else { Self::MIN_LEN }
    }

    pub const fn diagnostic_session_type(&self) -> Option<DiagnosticSessionType> {
        DiagnosticSessionType::from_u8(self.subfunction.parameter_value())
    }
}
