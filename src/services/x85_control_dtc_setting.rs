use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.9

    The ControlDTCSetting service is used to stop or resume the updating of DTC status bits in the server(s)

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - ROOR
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcSettingType {
    On,
    Off,
    VehicleManufacturerSpecific(u8),
    SystemSupplierSpecific(u8),
}

impl DtcSettingType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::On),
            0x02 => Some(Self::Off),
            0x40..=0x5F => Some(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::On => 0x01,
            Self::Off => 0x02,
            Self::VehicleManufacturerSpecific(value) => value,
            Self::SystemSupplierSpecific(value) => value,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x85_ControlDtcSetting;

impl UdsService for x85_ControlDtcSetting {
    const SID: u8 = 0x85;

    type Request<'a> = x85_ControlDtcSettingRequest<'a>;
    type Response<'a> = x85_ControlDtcSettingResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x85_ControlDtcSettingRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub dtc_setting_control_option_record: &'a [u8],
}

impl<'a> x85_ControlDtcSettingRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.dtc_setting_control_option_record.len()
    }

    pub const fn dtc_setting_type(&self) -> Option<DtcSettingType> {
        DtcSettingType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x85_ControlDtcSettingRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), dtc_setting_control_option_record: &data[1..] })
    }
}

impl TryTo for x85_ControlDtcSettingRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.dtc_setting_control_option_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x85_ControlDtcSettingRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x85_ControlDtcSettingResponse {
    pub subfunction: UdsSubfunction,
}

impl x85_ControlDtcSettingResponse {
    pub const LEN: usize = 1;

    pub const fn dtc_setting_type(&self) -> Option<DtcSettingType> {
        DtcSettingType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x85_ControlDtcSettingResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() != Self::LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]) })
    }
}

impl TryTo for x85_ControlDtcSettingResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        if buf.len() < Self::LEN {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        Ok(Self::LEN)
    }
}

impl<'a> UdsServiceResponse<'a> for x85_ControlDtcSettingResponse {}
