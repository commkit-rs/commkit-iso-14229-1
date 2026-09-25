use crate::nrc::UdsNrc;

pub(crate) const MAX_FIELD_LEN: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressAndLengthFormatIdentifier(u8);

impl AddressAndLengthFormatIdentifier {
    pub const LEN: usize = 1;

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(memory_address_len: u8, memory_size_len: u8) -> Self {
        Self(((memory_size_len & 0x0F) << 4) | (memory_address_len & 0x0F))
    }

    pub const fn minimal(memory_address: u64, memory_size: u64) -> Self {
        Self::from_parts(min_bytes(memory_address), min_bytes(memory_size))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn memory_address_len(self) -> u8 {
        self.0 & 0x0F
    }

    pub const fn memory_size_len(self) -> u8 {
        self.0 >> 4
    }

    pub const fn is_valid(self) -> bool {
        let address_len = self.memory_address_len();
        let size_len = self.memory_size_len();
        address_len != 0 && address_len <= MAX_FIELD_LEN && size_len != 0 && size_len <= MAX_FIELD_LEN
    }

    pub const fn fields_len(self) -> usize {
        self.memory_address_len() as usize + self.memory_size_len() as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAddressAndSize {
    pub address_and_length_format_identifier: AddressAndLengthFormatIdentifier,
    pub memory_address: u64,
    pub memory_size: u64,
}

impl MemoryAddressAndSize {
    pub const MIN_LEN: usize = AddressAndLengthFormatIdentifier::LEN + 2;

    pub const fn new(memory_address: u64, memory_size: u64) -> Self {
        Self {
            address_and_length_format_identifier: AddressAndLengthFormatIdentifier::minimal(memory_address, memory_size),
            memory_address,
            memory_size,
        }
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize), UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let format = AddressAndLengthFormatIdentifier::new(data[0]);
        if !format.is_valid() {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let len = AddressAndLengthFormatIdentifier::LEN + format.fields_len();
        if data.len() < len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (memory_address, memory_size) = decode_fields(format, &data[AddressAndLengthFormatIdentifier::LEN..len]);
        Ok((Self { address_and_length_format_identifier: format, memory_address, memory_size }, len))
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let format = self.address_and_length_format_identifier;
        let len = self.encoded_len();
        if !format.is_valid() {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = format.raw();
        encode_fields(format, self.memory_address, self.memory_size, &mut buf[AddressAndLengthFormatIdentifier::LEN..len])?;
        Ok(len)
    }

    pub const fn encoded_len(&self) -> usize {
        AddressAndLengthFormatIdentifier::LEN + self.address_and_length_format_identifier.fields_len()
    }
}

pub(crate) fn decode_fields(format: AddressAndLengthFormatIdentifier, fields: &[u8]) -> (u64, u64) {
    let (address, size) = fields.split_at(format.memory_address_len() as usize);
    (read_be(address), read_be(size))
}

pub(crate) fn encode_fields(
    format: AddressAndLengthFormatIdentifier,
    memory_address: u64,
    memory_size: u64,
    out: &mut [u8],
) -> Result<(), UdsNrc> {
    if min_bytes(memory_address) > format.memory_address_len() || min_bytes(memory_size) > format.memory_size_len() {
        return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
    }
    let (address, size) = out.split_at_mut(format.memory_address_len() as usize);
    write_be(memory_address, address);
    write_be(memory_size, size);
    Ok(())
}

pub(crate) const fn min_bytes(value: u64) -> u8 {
    let bytes = (u64::BITS - value.leading_zeros()).div_ceil(8) as u8;
    if bytes == 0 { 1 } else { bytes }
}

pub(crate) fn read_be(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |acc, &b| (acc << 8) | b as u64)
}

pub(crate) fn write_be(value: u64, out: &mut [u8]) {
    let be = value.to_be_bytes();
    out.copy_from_slice(&be[be.len() - out.len()..]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFormatIdentifier(u8);

impl DataFormatIdentifier {
    pub const LEN: usize = 1;
    pub const UNCOMPRESSED_UNENCRYPTED: Self = Self(0x00);

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(compression_method: u8, encrypting_method: u8) -> Self {
        Self(((compression_method & 0x0F) << 4) | (encrypting_method & 0x0F))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn compression_method(self) -> u8 {
        self.0 >> 4
    }

    pub const fn encrypting_method(self) -> u8 {
        self.0 & 0x0F
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthFormatIdentifier(u8);

impl LengthFormatIdentifier {
    pub const LEN: usize = 1;

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_len(max_number_of_block_length_len: u8) -> Self {
        Self((max_number_of_block_length_len & 0x0F) << 4)
    }

    pub const fn minimal(max_number_of_block_length: u64) -> Self {
        Self::from_len(min_bytes(max_number_of_block_length))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn max_number_of_block_length_len(self) -> u8 {
        self.0 >> 4
    }

    pub const fn is_valid(self) -> bool {
        let len = self.max_number_of_block_length_len();
        len != 0 && len <= MAX_FIELD_LEN
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxNumberOfBlockLength {
    pub length_format_identifier: LengthFormatIdentifier,
    pub max_number_of_block_length: u64,
}

impl MaxNumberOfBlockLength {
    pub const MIN_LEN: usize = LengthFormatIdentifier::LEN + 1;

    pub const fn new(max_number_of_block_length: u64) -> Self {
        Self {
            length_format_identifier: LengthFormatIdentifier::minimal(max_number_of_block_length),
            max_number_of_block_length,
        }
    }

    pub fn decode(data: &[u8]) -> Result<Self, UdsNrc> {
        let (&raw, value) = data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let length_format_identifier = LengthFormatIdentifier::new(raw);
        if !length_format_identifier.is_valid()
            || value.len() != length_format_identifier.max_number_of_block_length_len() as usize
        {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { length_format_identifier, max_number_of_block_length: read_be(value) })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let format = self.length_format_identifier;
        if !format.is_valid() || min_bytes(self.max_number_of_block_length) > format.max_number_of_block_length_len() {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = LengthFormatIdentifier::from_len(format.max_number_of_block_length_len()).raw();
        write_be(self.max_number_of_block_length, &mut buf[LengthFormatIdentifier::LEN..len]);
        Ok(len)
    }

    pub const fn encoded_len(&self) -> usize {
        LengthFormatIdentifier::LEN + self.length_format_identifier.max_number_of_block_length_len() as usize
    }

    pub const fn max_transfer_data_record_len(&self) -> u64 {
        self.max_number_of_block_length.saturating_sub(2)
    }
}
