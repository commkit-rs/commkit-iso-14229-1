use crate::nrc::UdsNrc;
use crate::service::UdsService;
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.5

    The CommunicationControl service is used to switch on/off the transmission and/or the reception of certain messages of the server(s)

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - ROOR
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    EnableRxAndTx,
    EnableRxAndDisableTx,
    DisableRxAndEnableTx,
    DisableRxAndTx,
    EnableRxAndDisableTxWithEnhancedAddressInformation,
    EnableRxAndTxWithEnhancedAddressInformation,
    VehicleManufacturerSpecific(u8),
    SystemSupplierSpecific(u8),
}

impl ControlType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::EnableRxAndTx),
            0x01 => Some(Self::EnableRxAndDisableTx),
            0x02 => Some(Self::DisableRxAndEnableTx),
            0x03 => Some(Self::DisableRxAndTx),
            0x04 => Some(Self::EnableRxAndDisableTxWithEnhancedAddressInformation),
            0x05 => Some(Self::EnableRxAndTxWithEnhancedAddressInformation),
            0x40..=0x5F => Some(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::EnableRxAndTx => 0x00,
            Self::EnableRxAndDisableTx => 0x01,
            Self::DisableRxAndEnableTx => 0x02,
            Self::DisableRxAndTx => 0x03,
            Self::EnableRxAndDisableTxWithEnhancedAddressInformation => 0x04,
            Self::EnableRxAndTxWithEnhancedAddressInformation => 0x05,
            Self::VehicleManufacturerSpecific(value) => value,
            Self::SystemSupplierSpecific(value) => value,
        }
    }

    pub const fn requires_node_identification_number(self) -> bool {
        matches!(
            self,
            Self::EnableRxAndDisableTxWithEnhancedAddressInformation | Self::EnableRxAndTxWithEnhancedAddressInformation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationMessages {
    Normal,
    NetworkManagement,
    NetworkManagementAndNormal,
}

impl CommunicationMessages {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x1 => Some(Self::Normal),
            0x2 => Some(Self::NetworkManagement),
            0x3 => Some(Self::NetworkManagementAndNormal),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Normal => 0x1,
            Self::NetworkManagement => 0x2,
            Self::NetworkManagementAndNormal => 0x3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subnet {
    AllConnectedNetworks,
    SubnetNumber(u8),
    ReceivingNetwork,
}

impl Subnet {
    pub const fn from_u8(value: u8) -> Self {
        match value & 0x0F {
            0x0 => Self::AllConnectedNetworks,
            0xF => Self::ReceivingNetwork,
            number => Self::SubnetNumber(number),
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::AllConnectedNetworks => 0x0,
            Self::SubnetNumber(number) => number & 0x0F,
            Self::ReceivingNetwork => 0xF,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommunicationType(u8);

impl CommunicationType {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(messages: CommunicationMessages, subnet: Subnet) -> Self {
        Self((subnet.as_u8() << 4) | messages.as_u8())
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn messages(self) -> Option<CommunicationMessages> {
        CommunicationMessages::from_u8(self.0 & 0x03)
    }

    pub const fn subnet(self) -> Subnet {
        Subnet::from_u8(self.0 >> 4)
    }
}

#[allow(non_camel_case_types)]
pub struct x28_CommunicationControl;

impl UdsService for x28_CommunicationControl {
    const SID: u8 = 0x28;

    type Request<'a> = x28_CommunicationControlRequest;
    type Response<'a> = x28_CommunicationControlResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x28_CommunicationControlRequest {
    pub subfunction: UdsSubfunction,
    pub communication_type: CommunicationType,
    pub node_identification_number: Option<u16>,
}

impl x28_CommunicationControlRequest {
    pub const MIN_LEN: usize = 2;
    pub const MAX_LEN: usize = Self::MIN_LEN + 2;

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        let node_identification_number = match data.len() {
            Self::MIN_LEN => None,
            Self::MAX_LEN => Some(u16::from_be_bytes([data[2], data[3]])),
            _ => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
        };
        Ok(Self {
            subfunction: UdsSubfunction::new(data[0]),
            communication_type: CommunicationType::new(data[1]),
            node_identification_number,
        })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1] = self.communication_type.raw();
        if let Some(node_identification_number) = self.node_identification_number {
            buf[2..4].copy_from_slice(&node_identification_number.to_be_bytes());
        }
        Ok(len)
    }

    pub const fn encoded_len(&self) -> usize {
        if self.node_identification_number.is_some() { Self::MAX_LEN } else { Self::MIN_LEN }
    }

    pub const fn control_type(&self) -> Option<ControlType> {
        ControlType::from_u8(self.subfunction.parameter_value())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x28_CommunicationControlResponse {
    pub subfunction: UdsSubfunction,
}

impl x28_CommunicationControlResponse {
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

    pub const fn control_type(&self) -> Option<ControlType> {
        ControlType::from_u8(self.subfunction.parameter_value())
    }
}
