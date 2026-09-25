use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.7

    The AccessTimingParameter service is used to read and change the default timing parameters of a communication link

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - ROOR
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingParameterAccessType {
    ReadExtendedTimingParameterSet,
    SetTimingParametersToDefaultValues,
    ReadCurrentlyActiveTimingParameters,
    SetTimingParametersToGivenValues,
}

impl TimingParameterAccessType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::ReadExtendedTimingParameterSet),
            0x02 => Some(Self::SetTimingParametersToDefaultValues),
            0x03 => Some(Self::ReadCurrentlyActiveTimingParameters),
            0x04 => Some(Self::SetTimingParametersToGivenValues),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::ReadExtendedTimingParameterSet => 0x01,
            Self::SetTimingParametersToDefaultValues => 0x02,
            Self::ReadCurrentlyActiveTimingParameters => 0x03,
            Self::SetTimingParametersToGivenValues => 0x04,
        }
    }

    pub const fn has_request_record(self) -> bool {
        matches!(self, Self::SetTimingParametersToGivenValues)
    }

    pub const fn has_response_record(self) -> bool {
        matches!(self, Self::ReadExtendedTimingParameterSet | Self::ReadCurrentlyActiveTimingParameters)
    }
}

#[allow(non_camel_case_types)]
pub struct x83_AccessTimingParameter;

impl UdsService for x83_AccessTimingParameter {
    const SID: u8 = 0x83;

    type Request<'a> = x83_AccessTimingParameterRequest<'a>;
    type Response<'a> = x83_AccessTimingParameterResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x83_AccessTimingParameterRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub timing_parameter_request_record: &'a [u8],
}

impl<'a> x83_AccessTimingParameterRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.timing_parameter_request_record.len()
    }

    pub const fn timing_parameter_access_type(&self) -> Option<TimingParameterAccessType> {
        TimingParameterAccessType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x83_AccessTimingParameterRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), timing_parameter_request_record: &data[1..] })
    }
}

impl TryTo for x83_AccessTimingParameterRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.timing_parameter_request_record);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x83_AccessTimingParameterRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x83_AccessTimingParameterResponse<'a> {
    pub subfunction: UdsSubfunction,
    pub timing_parameter_response_record: &'a [u8],
}

impl<'a> x83_AccessTimingParameterResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.timing_parameter_response_record.len()
    }

    pub const fn timing_parameter_access_type(&self) -> Option<TimingParameterAccessType> {
        TimingParameterAccessType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x83_AccessTimingParameterResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), timing_parameter_response_record: &data[1..] })
    }
}

impl TryTo for x83_AccessTimingParameterResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.timing_parameter_response_record);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x83_AccessTimingParameterResponse<'a> {}
