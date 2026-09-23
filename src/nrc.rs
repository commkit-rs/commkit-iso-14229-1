/// UDS NRC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsNrc(u8);

impl UdsNrc {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn is_response_pending(self) -> bool {
        self.0 == Self::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING.0
    }

    pub const POSITIVE_RESPONSE: Self = Self(0x00);

    pub const GENERAL_REJECT: Self = Self(0x10);
    pub const SERVICE_NOT_SUPPORTED: Self = Self(0x11);
    pub const SUB_FUNCTION_NOT_SUPPORTED: Self = Self(0x12);
    pub const INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT: Self = Self(0x13);
    pub const RESPONSE_TOO_LONG: Self = Self(0x14);

    /* 0x15 - 0x20 (ISOSAERESRVD) */

    pub const BUSY_REPEAT_REQUEST: Self = Self(0x21);
    pub const CONDITIONS_NOT_CORRECT: Self = Self(0x22);

    /* 0x23 (ISOSAERESRVD) */

    pub const REQUEST_SEQUENCE_ERROR: Self = Self(0x24);
    pub const NO_RESPONSE_FROM_SUBNET_COMPONENT: Self = Self(0x25);
    pub const FAILURE_PREVENTS_EXECUTION_OF_REQUESTED_ACTION: Self = Self(0x26);

    /* 0x27 - 0x30 (ISOSAERESRVD) */

    pub const REQUEST_OUT_OF_RANGE: Self = Self(0x31);

    /* 0x32 (ISOSAERESRVD) */

    pub const SECURITY_ACCESS_DENIED: Self = Self(0x33);

    /* 0x34 (ISOSAERESRVD) */

    pub const INVALID_KEY: Self = Self(0x35);
    pub const EXCEED_NUMBER_OF_ATTEMPTS: Self = Self(0x36);
    pub const REQUIRED_TIME_DELAY_NOT_EXPIRED: Self = Self(0x37);

    /* 0x38 - 0x4F (RBEDLSD) Reserved By Extended Data Link Security Document */
    // pub const SECURE_DATA_TRANSMISSION_REQUIRED: Self = Self(0x38);
    // pub const SECURE_DATA_TRANSMISSION_NOT_ALLOWED: Self = Self(0x39);
    // pub const SECURE_DATA_TRANSMISSION_FAILED: Self = Self(0x3A);

    /* 0x50 - 0x6F (ISOSAERESRVD) */
    // pub const CERT_VALIDATION_FAILED_INVALID_TIME_PERIOD: Self = Self(0x50);
    // pub const CERT_VALIDATION_FAILED_INVALID_SIGNATURE: Self = Self(0x51);
    // pub const CERT_VALIDATION_FAILED_INVALID_CHAIN_OF_TRUST: Self = Self(0x52);
    // pub const CERT_VALIDATION_FAILED_INVALID_TYPE: Self = Self(0x53);
    // pub const CERT_VALIDATION_FAILED_INVALID_FORMAT: Self = Self(0x54);
    // pub const CERT_VALIDATION_FAILED_INVALID_CONTENT: Self = Self(0x55);
    // pub const CERT_VALIDATION_FAILED_INVALID_SCOPE: Self = Self(0x56);
    // pub const CERT_VALIDATION_FAILED_INVALID_CERTIFICATE: Self = Self(0x57);
    // pub const OWNERSHIP_VERIFICATION_FAILED: Self = Self(0x58);
    // pub const CHALLENGE_CALCULATION_FAILED: Self = Self(0x59);
    // pub const SETTING_ACCESS_RIGHT_FAILED: Self = Self(0x5A);
    // pub const SESSION_KEY_CREATION_FAILED: Self = Self(0x5B);
    // pub const CONFIGURATION_DATA_USAGE_FAILED: Self = Self(0x5C);
    // pub const DEAUTHENTICATION_FAILED: Self = Self(0x5D);

    pub const UPLOAD_DOWNLOAD_NOT_ACCEPTED: Self = Self(0x70);
    pub const TRANSFER_DATA_SUSPENDED: Self = Self(0x71);
    pub const GENERAL_PROGRAMMING_FAILURE: Self = Self(0x72);
    pub const WRONG_BLOCK_SEQUENCE_NUMBER: Self = Self(0x73);

    /* 0x74 - 0x77 (ISOSAERESRVD) */

    pub const REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING: Self = Self(0x78);

    /* 0x79 - 0x7D (ISOSAERESRVD) */

    pub const SUB_FUNCTION_NOT_SUPPORTED_IN_ACTIVE_SESSION: Self = Self(0x7E);
    pub const SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION: Self = Self(0x7F);

    /* 0x80 (ISOSAERESRVD) */

    pub const RPM_TOO_HIGH: Self = Self(0x81);
    pub const RPM_TOO_LOW: Self = Self(0x82);
    pub const ENGINE_IS_RUNNING: Self = Self(0x83);
    pub const ENGINE_IS_NOT_RUNNING: Self = Self(0x84);
    pub const ENGINE_RUN_TIME_TOO_LOW: Self = Self(0x85);
    pub const TEMPERATURE_TOO_HIGH: Self = Self(0x86);
    pub const TEMPERATURE_TOO_LOW: Self = Self(0x87);
    pub const VEHICLE_SPEED_TOO_HIGH: Self = Self(0x88);
    pub const VEHICLE_SPEED_TOO_LOW: Self = Self(0x89);
    pub const THROTTLE_PEDAL_TOO_HIGH: Self = Self(0x8A);
    pub const THROTTLE_PEDAL_TOO_LOW: Self = Self(0x8B);
    pub const TRANSMISSION_RANGE_NOT_IN_NEUTRAL: Self = Self(0x8C);
    pub const TRANSMISSION_RANGE_NOT_IN_GEAR: Self = Self(0x8D);

    /* 0x8E (ISOSAERESRVD) */

    pub const BRAKE_SWITCH_NOT_CLOSED: Self = Self(0x8F);
    pub const SHIFTER_LEVER_NOT_IN_PARK: Self = Self(0x90);
    pub const TORQUE_CONVERTER_CLUTCH_LOCKED: Self = Self(0x91);
    pub const VOLTAGE_TOO_HIGH: Self = Self(0x92);
    pub const VOLTAGE_TOO_LOW: Self = Self(0x93);

    /* 0x94 - 0xEF (RFSCNC) Reserved for Specific Conditions not Correct */

    /* 0xF0 - 0xFE (VMSCNC) Vehicle Manufacturer Specific Conditions Not Correct */

    /* 0xFF (ISOSAEReserved) */
}
