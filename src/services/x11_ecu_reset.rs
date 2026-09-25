use commkit::TryTo;

use commkit::Duration;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.3

    The ECUReset service is used by the client to request a server reset

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - SAD
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetType {
    HardReset,
    KeyOffOnReset,
    SoftReset,
    EnableRapidPowerShutDown,
    DisableRapidPowerShutDown,
    VehicleManufacturerSpecific(u8),
    SystemSupplierSpecific(u8),
}

impl ResetType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::HardReset),
            0x02 => Some(Self::KeyOffOnReset),
            0x03 => Some(Self::SoftReset),
            0x04 => Some(Self::EnableRapidPowerShutDown),
            0x05 => Some(Self::DisableRapidPowerShutDown),
            0x40..=0x5F => Some(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::HardReset => 0x01,
            Self::KeyOffOnReset => 0x02,
            Self::SoftReset => 0x03,
            Self::EnableRapidPowerShutDown => 0x04,
            Self::DisableRapidPowerShutDown => 0x05,
            Self::VehicleManufacturerSpecific(value) => value,
            Self::SystemSupplierSpecific(value) => value,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x11_EcuReset;

impl UdsService for x11_EcuReset {
    const SID: u8 = 0x11;

    type Request<'a> = x11_EcuResetRequest;
    type Response<'a> = x11_EcuResetResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x11_EcuResetRequest {
    pub subfunction: UdsSubfunction,
}

impl x11_EcuResetRequest {
    pub const LEN: usize = 1;

    pub const fn reset_type(&self) -> Option<ResetType> {
        ResetType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x11_EcuResetRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }
}

impl TryTo for x11_EcuResetRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceRequest<'a> for x11_EcuResetRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerDownTime {
    Time(Duration),
    NotAvailable,
}

impl PowerDownTime {
    const RESOLUTION_US: u64 = 1_000_000;
    const NOT_AVAILABLE: u8 = 0xFF;
    const MAX_SECONDS: u8 = 0xFE;

    pub const fn from_u8(value: u8) -> Self {
        match value {
            Self::NOT_AVAILABLE => Self::NotAvailable,
            seconds => Self::Time(Duration::from_ticks(seconds as u64 * Self::RESOLUTION_US)),
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Self::Time(duration) => (duration.as_ticks() / Self::RESOLUTION_US).min(Self::MAX_SECONDS as u64) as u8,
            Self::NotAvailable => Self::NOT_AVAILABLE,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x11_EcuResetResponse {
    pub subfunction: UdsSubfunction,
    pub power_down_time: Option<PowerDownTime>,
}

impl x11_EcuResetResponse {
    pub const MIN_LEN: usize = 1;
    pub const MAX_LEN: usize = Self::MIN_LEN + 1;

    pub const fn encoded_len(&self) -> usize {
        if self.power_down_time.is_some() { Self::MAX_LEN } else { Self::MIN_LEN }
    }

    pub const fn reset_type(&self) -> Option<ResetType> {
        ResetType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x11_EcuResetResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let power_down_time = match data.len() {
            Self::MIN_LEN => None,
            Self::MAX_LEN => Some(PowerDownTime::from_u8(data[1])),
            _ => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
        };
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), power_down_time })
    }
}

impl TryTo for x11_EcuResetResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        if let Some(power_down_time) = self.power_down_time {
            buf[1] = power_down_time.as_u8();
        }
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x11_EcuResetResponse {}
