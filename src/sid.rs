/// UDS SID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsSid(u8);

impl UdsSid {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    /* Diagnostic and Communication Management */

    pub const DIAGNOSTIC_SESSION_CONTROL: Self = Self(0x10);
    pub const ECU_RESET: Self = Self(0x11);
    pub const SECURITY_ACCESS: Self = Self(0x27);
    pub const COMMUNICATION_CONTROL: Self = Self(0x28);

    // TODO: Missing 0x29 Authorization, need ISO doc

    pub const TESTER_PRESENT: Self = Self(0x3E);
    pub const ACCESS_TIMING_PARAMETERS: Self = Self(0x83);
    pub const SECURED_DATA_TRANSMISSION: Self = Self(0x84);
    pub const CONTROL_DTC_SETTING: Self = Self(0x85);
    pub const RESPONSE_ON_EVENT: Self = Self(0x86);
    pub const LINK_CONTROL: Self = Self(0x87);

    /* Data Transmission */

    // TODO: I think??
    pub const READ_DATA_BY_LOCAL_IDENTIFIER: Self = Self(0x21);
    pub const READ_DATA_BY_IDENTIFIER: Self = Self(0x22);
    pub const READ_MEMORY_BY_ADDRESS: Self = Self(0x23);
    pub const READ_SCALING_DATA_BY_IDENTIFIER: Self = Self(0x24);
    pub const READ_DATA_BY_PERIODIC_IDENTIFIER: Self = Self(0x2A);
    pub const DYNAMICALLY_DEFINE_DATA_IDENTIFIER: Self = Self(0x2C);
    pub const WRITE_DATA_BY_IDENTIFIER: Self = Self(0x2E);
    pub const WRITE_MEMORY_BY_ADDRESS: Self = Self(0x3D);

    /* Stored Data Transmission */

    pub const CLEAR_DIAGNOSTIC_INFORMATION: Self = Self(0x14);
    pub const READ_DTC_INFORMATION: Self = Self(0x19);

    /* Input/Output Control */

    pub const INPUT_OUTPUT_CONTROL_BY_IDENTIFIER: Self = Self(0x2F);

    /* Remote Activation of Routine */

    pub const ROUTINE_CONTROL: Self = Self(0x31);

    /* Upload/Download */

    pub const REQUEST_DOWNLOAD: Self = Self(0x34);
    pub const REQUEST_UPLOAD: Self = Self(0x35);
    pub const TRANSFER_DATA: Self = Self(0x36);
    pub const REQUEST_TRANSFER_EXIT: Self = Self(0x37);
    pub const REQUEST_FILE_TRANSFER: Self = Self(0x38);

    pub const NEGATIVE_RESPONSE: Self = Self(0x7F);
}
