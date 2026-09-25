use commkit::TryTo;

use crate::memory::{self, DataFormatIdentifier, MAX_FIELD_LEN};
use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 14.6

    The RequestFileTransfer service is used by the client to initiate a file data transfer from either the client to the server or from the server to the client, or to retrieve information about the file system

    Supported NRC:
        - IMLOIF
        - CNC
        - ROOR
        - SAD
        - UDNA
*/

const PATH_LEN_LEN: usize = 2;
const DIR_INFO_PARAMETER_LEN_LEN: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeOfOperation {
    AddFile,
    DeleteFile,
    ReplaceFile,
    ReadFile,
    ReadDir,
}

impl ModeOfOperation {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::AddFile),
            0x02 => Some(Self::DeleteFile),
            0x03 => Some(Self::ReplaceFile),
            0x04 => Some(Self::ReadFile),
            0x05 => Some(Self::ReadDir),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::AddFile => 0x01,
            Self::DeleteFile => 0x02,
            Self::ReplaceFile => 0x03,
            Self::ReadFile => 0x04,
            Self::ReadDir => 0x05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSizes {
    pub parameter_length: u8,
    pub file_size_uncompressed: u64,
    pub file_size_compressed: u64,
}

impl FileSizes {
    pub const fn new(file_size_uncompressed: u64, file_size_compressed: u64) -> Self {
        let uncompressed_len = memory::min_bytes(file_size_uncompressed);
        let compressed_len = memory::min_bytes(file_size_compressed);
        let parameter_length = if uncompressed_len > compressed_len { uncompressed_len } else { compressed_len };
        Self { parameter_length, file_size_uncompressed, file_size_compressed }
    }

    pub const fn uncompressed(file_size: u64) -> Self {
        Self::new(file_size, file_size)
    }

    const fn values_len(&self) -> usize {
        2 * self.parameter_length as usize
    }

    fn decode(parameter_length: usize, data: &[u8], invalid_length: UdsNrc) -> Result<Self, UdsNrc> {
        if parameter_length == 0 || parameter_length > MAX_FIELD_LEN as usize {
            return Err(invalid_length);
        }
        if data.len() != 2 * parameter_length {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (uncompressed, compressed) = data.split_at(parameter_length);
        Ok(Self {
            parameter_length: parameter_length as u8,
            file_size_uncompressed: memory::read_be(uncompressed),
            file_size_compressed: memory::read_be(compressed),
        })
    }

    fn encode_values(&self, out: &mut [u8]) -> Result<(), UdsNrc> {
        let len = self.parameter_length;
        if len == 0
            || len > MAX_FIELD_LEN
            || memory::min_bytes(self.file_size_uncompressed) > len
            || memory::min_bytes(self.file_size_compressed) > len
        {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let (uncompressed, compressed) = out.split_at_mut(len as usize);
        memory::write_be(self.file_size_uncompressed, uncompressed);
        memory::write_be(self.file_size_compressed, compressed);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileTransferBlockLength {
    pub length_format_identifier: u8,
    pub max_number_of_block_length: u64,
}

impl FileTransferBlockLength {
    pub const fn new(max_number_of_block_length: u64) -> Self {
        Self { length_format_identifier: memory::min_bytes(max_number_of_block_length), max_number_of_block_length }
    }

    pub const fn max_transfer_data_record_len(&self) -> u64 {
        self.max_number_of_block_length.saturating_sub(2)
    }

    const fn encoded_len(&self) -> usize {
        1 + self.length_format_identifier as usize
    }

    fn decode(data: &[u8]) -> Result<(Self, &[u8]), UdsNrc> {
        let (&length_format_identifier, rest) =
            data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let len = length_format_identifier as usize;
        if len == 0 || len > MAX_FIELD_LEN as usize || rest.len() < len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (value, rest) = rest.split_at(len);
        Ok((Self { length_format_identifier, max_number_of_block_length: memory::read_be(value) }, rest))
    }

    fn encode(&self, out: &mut [u8]) -> Result<(), UdsNrc> {
        let len = self.length_format_identifier;
        if len == 0 || len > MAX_FIELD_LEN || memory::min_bytes(self.max_number_of_block_length) > len {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        out[0] = len;
        memory::write_be(self.max_number_of_block_length, &mut out[1..self.encoded_len()]);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    AddFile { data_format_identifier: DataFormatIdentifier, file_sizes: FileSizes },
    DeleteFile,
    ReplaceFile { data_format_identifier: DataFormatIdentifier, file_sizes: FileSizes },
    ReadFile { data_format_identifier: DataFormatIdentifier },
    ReadDir,
}

impl FileOperation {
    pub const fn mode_of_operation(&self) -> ModeOfOperation {
        match self {
            Self::AddFile { .. } => ModeOfOperation::AddFile,
            Self::DeleteFile => ModeOfOperation::DeleteFile,
            Self::ReplaceFile { .. } => ModeOfOperation::ReplaceFile,
            Self::ReadFile { .. } => ModeOfOperation::ReadFile,
            Self::ReadDir => ModeOfOperation::ReadDir,
        }
    }

    const fn encoded_len(&self) -> usize {
        match self {
            Self::AddFile { file_sizes, .. } | Self::ReplaceFile { file_sizes, .. } => {
                DataFormatIdentifier::LEN + 1 + file_sizes.values_len()
            }
            Self::ReadFile { .. } => DataFormatIdentifier::LEN,
            Self::DeleteFile | Self::ReadDir => 0,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x38_RequestFileTransfer;

impl UdsService for x38_RequestFileTransfer {
    const SID: u8 = 0x38;

    type Request<'a> = x38_RequestFileTransferRequest<'a>;
    type Response<'a> = x38_RequestFileTransferResponse;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x38_RequestFileTransferRequest<'a> {
    pub file_path_and_name: &'a [u8],
    pub operation: FileOperation,
}

impl<'a> x38_RequestFileTransferRequest<'a> {
    pub const MIN_LEN: usize = 1 + PATH_LEN_LEN + 1;

    pub const fn mode_of_operation(&self) -> ModeOfOperation {
        self.operation.mode_of_operation()
    }

    pub const fn encoded_len(&self) -> usize {
        1 + PATH_LEN_LEN + self.file_path_and_name.len() + self.operation.encoded_len()
    }
}

impl<'a> TryFrom<&'a [u8]> for x38_RequestFileTransferRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let mode_of_operation = ModeOfOperation::from_u8(data[0]).ok_or(UdsNrc::REQUEST_OUT_OF_RANGE)?;
        let path_len = u16::from_be_bytes([data[1], data[2]]) as usize;
        if path_len == 0 {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let path_start = 1 + PATH_LEN_LEN;
        let path_end = path_start + path_len;
        if data.len() < path_end {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let file_path_and_name = &data[path_start..path_end];
        let rest = &data[path_end..];

        let operation = match mode_of_operation {
            ModeOfOperation::AddFile | ModeOfOperation::ReplaceFile => {
                let [data_format_identifier, parameter_length, sizes @ ..] = rest else {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                };
                let data_format_identifier = DataFormatIdentifier::new(*data_format_identifier);
                let file_sizes = FileSizes::decode(*parameter_length as usize, sizes, UdsNrc::REQUEST_OUT_OF_RANGE)?;
                if mode_of_operation == ModeOfOperation::AddFile {
                    FileOperation::AddFile { data_format_identifier, file_sizes }
                } else {
                    FileOperation::ReplaceFile { data_format_identifier, file_sizes }
                }
            }
            ModeOfOperation::ReadFile => match rest {
                [data_format_identifier] => {
                    FileOperation::ReadFile { data_format_identifier: DataFormatIdentifier::new(*data_format_identifier) }
                }
                _ => return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT),
            },
            ModeOfOperation::DeleteFile | ModeOfOperation::ReadDir => {
                if !rest.is_empty() {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
                if mode_of_operation == ModeOfOperation::DeleteFile {
                    FileOperation::DeleteFile
                } else {
                    FileOperation::ReadDir
                }
            }
        };

        Ok(Self { file_path_and_name, operation })
    }
}

impl TryTo for x38_RequestFileTransferRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let path_len = u16::try_from(self.file_path_and_name.len()).map_err(|_| UdsNrc::REQUEST_OUT_OF_RANGE)?;
        if path_len == 0 {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.mode_of_operation().as_u8();
        buf[1..1 + PATH_LEN_LEN].copy_from_slice(&path_len.to_be_bytes());
        let mut offset = 1 + PATH_LEN_LEN;
        buf[offset..offset + self.file_path_and_name.len()].copy_from_slice(self.file_path_and_name);
        offset += self.file_path_and_name.len();

        match self.operation {
            FileOperation::AddFile { data_format_identifier, file_sizes }
            | FileOperation::ReplaceFile { data_format_identifier, file_sizes } => {
                buf[offset] = data_format_identifier.raw();
                buf[offset + 1] = file_sizes.parameter_length;
                file_sizes.encode_values(&mut buf[offset + 2..len])?;
            }
            FileOperation::ReadFile { data_format_identifier } => buf[offset] = data_format_identifier.raw(),
            FileOperation::DeleteFile | FileOperation::ReadDir => {}
        }
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x38_RequestFileTransferRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        None
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum x38_RequestFileTransferResponse {
    AddFile { block_length: FileTransferBlockLength, data_format_identifier: DataFormatIdentifier },
    DeleteFile,
    ReplaceFile { block_length: FileTransferBlockLength, data_format_identifier: DataFormatIdentifier },
    ReadFile { block_length: FileTransferBlockLength, data_format_identifier: DataFormatIdentifier, file_sizes: FileSizes },
    ReadDir { block_length: FileTransferBlockLength, dir_info_parameter_length: u16, dir_info_length: u64 },
}

impl x38_RequestFileTransferResponse {
    pub const MIN_LEN: usize = 1;

    pub const fn mode_of_operation(&self) -> ModeOfOperation {
        match self {
            Self::AddFile { .. } => ModeOfOperation::AddFile,
            Self::DeleteFile => ModeOfOperation::DeleteFile,
            Self::ReplaceFile { .. } => ModeOfOperation::ReplaceFile,
            Self::ReadFile { .. } => ModeOfOperation::ReadFile,
            Self::ReadDir { .. } => ModeOfOperation::ReadDir,
        }
    }

    pub const fn block_length(&self) -> Option<FileTransferBlockLength> {
        match *self {
            Self::AddFile { block_length, .. }
            | Self::ReplaceFile { block_length, .. }
            | Self::ReadFile { block_length, .. }
            | Self::ReadDir { block_length, .. } => Some(block_length),
            Self::DeleteFile => None,
        }
    }

    pub const fn read_dir(block_length: FileTransferBlockLength, dir_info_length: u64) -> Self {
        Self::ReadDir { block_length, dir_info_parameter_length: memory::min_bytes(dir_info_length) as u16, dir_info_length }
    }

    pub const fn encoded_len(&self) -> usize {
        let body = match self {
            Self::AddFile { block_length, .. } | Self::ReplaceFile { block_length, .. } => {
                block_length.encoded_len() + DataFormatIdentifier::LEN
            }
            Self::ReadFile { block_length, file_sizes, .. } => {
                block_length.encoded_len() + DataFormatIdentifier::LEN + DIR_INFO_PARAMETER_LEN_LEN + file_sizes.values_len()
            }
            Self::ReadDir { block_length, dir_info_parameter_length, .. } => {
                block_length.encoded_len()
                    + DataFormatIdentifier::LEN
                    + DIR_INFO_PARAMETER_LEN_LEN
                    + *dir_info_parameter_length as usize
            }
            Self::DeleteFile => 0,
        };
        Self::MIN_LEN + body
    }
}

impl<'a> TryFrom<&'a [u8]> for x38_RequestFileTransferResponse {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&mode, rest) = data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let mode_of_operation =
            ModeOfOperation::from_u8(mode).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        if mode_of_operation == ModeOfOperation::DeleteFile {
            if !rest.is_empty() {
                return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
            }
            return Ok(Self::DeleteFile);
        }

        let (block_length, rest) = FileTransferBlockLength::decode(rest)?;
        let (&data_format_identifier, rest) =
            rest.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let data_format_identifier = DataFormatIdentifier::new(data_format_identifier);

        match mode_of_operation {
            ModeOfOperation::AddFile | ModeOfOperation::ReplaceFile => {
                if !rest.is_empty() {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
                Ok(if mode_of_operation == ModeOfOperation::AddFile {
                    Self::AddFile { block_length, data_format_identifier }
                } else {
                    Self::ReplaceFile { block_length, data_format_identifier }
                })
            }
            ModeOfOperation::ReadFile | ModeOfOperation::ReadDir => {
                let [high, low, values @ ..] = rest else {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                };
                let parameter_length = u16::from_be_bytes([*high, *low]);
                if mode_of_operation == ModeOfOperation::ReadFile {
                    let file_sizes = FileSizes::decode(
                        parameter_length as usize,
                        values,
                        UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT,
                    )?;
                    return Ok(Self::ReadFile { block_length, data_format_identifier, file_sizes });
                }
                if parameter_length == 0
                    || parameter_length > MAX_FIELD_LEN as u16
                    || values.len() != parameter_length as usize
                {
                    return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
                }
                Ok(Self::ReadDir {
                    block_length,
                    dir_info_parameter_length: parameter_length,
                    dir_info_length: memory::read_be(values),
                })
            }
            ModeOfOperation::DeleteFile => unreachable!(),
        }
    }
}

impl TryTo for x38_RequestFileTransferResponse {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.mode_of_operation().as_u8();
        let Some(block_length) = self.block_length() else {
            return Ok(len);
        };
        block_length.encode(&mut buf[1..])?;
        let mut offset = 1 + block_length.encoded_len();

        match *self {
            Self::AddFile { data_format_identifier, .. } | Self::ReplaceFile { data_format_identifier, .. } => {
                buf[offset] = data_format_identifier.raw();
            }
            Self::ReadFile { data_format_identifier, file_sizes, .. } => {
                buf[offset] = data_format_identifier.raw();
                offset += DataFormatIdentifier::LEN;
                buf[offset..offset + DIR_INFO_PARAMETER_LEN_LEN]
                    .copy_from_slice(&(file_sizes.parameter_length as u16).to_be_bytes());
                file_sizes.encode_values(&mut buf[offset + DIR_INFO_PARAMETER_LEN_LEN..len])?;
            }
            Self::ReadDir { dir_info_parameter_length, dir_info_length, .. } => {
                if dir_info_parameter_length == 0
                    || dir_info_parameter_length > MAX_FIELD_LEN as u16
                    || memory::min_bytes(dir_info_length) as u16 > dir_info_parameter_length
                {
                    return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
                }
                buf[offset] = DataFormatIdentifier::UNCOMPRESSED_UNENCRYPTED.raw();
                offset += DataFormatIdentifier::LEN;
                buf[offset..offset + DIR_INFO_PARAMETER_LEN_LEN].copy_from_slice(&dir_info_parameter_length.to_be_bytes());
                memory::write_be(dir_info_length, &mut buf[offset + DIR_INFO_PARAMETER_LEN_LEN..len]);
            }
            Self::DeleteFile => {}
        }
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x38_RequestFileTransferResponse {}
