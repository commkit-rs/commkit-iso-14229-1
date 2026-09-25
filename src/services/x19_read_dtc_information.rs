use commkit::TryTo;

use core::marker::PhantomData;
use core::mem::discriminant;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::services::x22_ReadDataByIdentifierRecords;
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 11.3

    The ReadDTCInformation service is used to read the status of server resident Diagnostic Trouble Code (DTC) information

    Supported NRC:
        - SFNS
        - IMLOIF
        - ROOR
*/

pub const DTC_MAX: u32 = 0xFF_FFFF;

const DTC_LEN: usize = 3;
const DID_LEN: usize = 2;

fn read_dtc(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])
}

fn write_dtc(dtc: u32, out: &mut [u8]) {
    out[..DTC_LEN].copy_from_slice(&dtc.to_be_bytes()[1..]);
}

fn check_dtc(dtc: u32) -> Result<(), UdsNrc> {
    if dtc > DTC_MAX {
        return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    NumberOfDtcByStatusMask,
    DtcByStatusMask,
    DtcSnapshotIdentification,
    DtcSnapshotRecordByDtcNumber,
    DtcStoredDataByRecordNumber,
    DtcExtDataRecordByDtcNumber,
    NumberOfDtcBySeverityMaskRecord,
    DtcBySeverityMaskRecord,
    SeverityInformationOfDtc,
    SupportedDtc,
    FirstTestFailedDtc,
    FirstConfirmedDtc,
    MostRecentTestFailedDtc,
    MostRecentConfirmedDtc,
    MirrorMemoryDtcByStatusMask,
    MirrorMemoryDtcExtDataRecordByDtcNumber,
    NumberOfMirrorMemoryDtcByStatusMask,
    NumberOfEmissionsObdDtcByStatusMask,
    EmissionsObdDtcByStatusMask,
    DtcFaultDetectionCounter,
    DtcWithPermanentStatus,
    DtcExtDataRecordByRecordNumber,
    UserDefMemoryDtcByStatusMask,
    UserDefMemoryDtcSnapshotRecordByDtcNumber,
    UserDefMemoryDtcExtDataRecordByDtcNumber,
    WwhObdDtcByMaskRecord,
    WwhObdDtcWithPermanentStatus,
}

impl ReportType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::NumberOfDtcByStatusMask),
            0x02 => Some(Self::DtcByStatusMask),
            0x03 => Some(Self::DtcSnapshotIdentification),
            0x04 => Some(Self::DtcSnapshotRecordByDtcNumber),
            0x05 => Some(Self::DtcStoredDataByRecordNumber),
            0x06 => Some(Self::DtcExtDataRecordByDtcNumber),
            0x07 => Some(Self::NumberOfDtcBySeverityMaskRecord),
            0x08 => Some(Self::DtcBySeverityMaskRecord),
            0x09 => Some(Self::SeverityInformationOfDtc),
            0x0A => Some(Self::SupportedDtc),
            0x0B => Some(Self::FirstTestFailedDtc),
            0x0C => Some(Self::FirstConfirmedDtc),
            0x0D => Some(Self::MostRecentTestFailedDtc),
            0x0E => Some(Self::MostRecentConfirmedDtc),
            0x0F => Some(Self::MirrorMemoryDtcByStatusMask),
            0x10 => Some(Self::MirrorMemoryDtcExtDataRecordByDtcNumber),
            0x11 => Some(Self::NumberOfMirrorMemoryDtcByStatusMask),
            0x12 => Some(Self::NumberOfEmissionsObdDtcByStatusMask),
            0x13 => Some(Self::EmissionsObdDtcByStatusMask),
            0x14 => Some(Self::DtcFaultDetectionCounter),
            0x15 => Some(Self::DtcWithPermanentStatus),
            0x16 => Some(Self::DtcExtDataRecordByRecordNumber),
            0x17 => Some(Self::UserDefMemoryDtcByStatusMask),
            0x18 => Some(Self::UserDefMemoryDtcSnapshotRecordByDtcNumber),
            0x19 => Some(Self::UserDefMemoryDtcExtDataRecordByDtcNumber),
            0x42 => Some(Self::WwhObdDtcByMaskRecord),
            0x55 => Some(Self::WwhObdDtcWithPermanentStatus),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::NumberOfDtcByStatusMask => 0x01,
            Self::DtcByStatusMask => 0x02,
            Self::DtcSnapshotIdentification => 0x03,
            Self::DtcSnapshotRecordByDtcNumber => 0x04,
            Self::DtcStoredDataByRecordNumber => 0x05,
            Self::DtcExtDataRecordByDtcNumber => 0x06,
            Self::NumberOfDtcBySeverityMaskRecord => 0x07,
            Self::DtcBySeverityMaskRecord => 0x08,
            Self::SeverityInformationOfDtc => 0x09,
            Self::SupportedDtc => 0x0A,
            Self::FirstTestFailedDtc => 0x0B,
            Self::FirstConfirmedDtc => 0x0C,
            Self::MostRecentTestFailedDtc => 0x0D,
            Self::MostRecentConfirmedDtc => 0x0E,
            Self::MirrorMemoryDtcByStatusMask => 0x0F,
            Self::MirrorMemoryDtcExtDataRecordByDtcNumber => 0x10,
            Self::NumberOfMirrorMemoryDtcByStatusMask => 0x11,
            Self::NumberOfEmissionsObdDtcByStatusMask => 0x12,
            Self::EmissionsObdDtcByStatusMask => 0x13,
            Self::DtcFaultDetectionCounter => 0x14,
            Self::DtcWithPermanentStatus => 0x15,
            Self::DtcExtDataRecordByRecordNumber => 0x16,
            Self::UserDefMemoryDtcByStatusMask => 0x17,
            Self::UserDefMemoryDtcSnapshotRecordByDtcNumber => 0x18,
            Self::UserDefMemoryDtcExtDataRecordByDtcNumber => 0x19,
            Self::WwhObdDtcByMaskRecord => 0x42,
            Self::WwhObdDtcWithPermanentStatus => 0x55,
        }
    }

    const fn shape(self) -> Shape {
        match self {
            Self::NumberOfDtcByStatusMask
            | Self::NumberOfDtcBySeverityMaskRecord
            | Self::NumberOfMirrorMemoryDtcByStatusMask
            | Self::NumberOfEmissionsObdDtcByStatusMask => Shape::Count,
            Self::DtcByStatusMask
            | Self::SupportedDtc
            | Self::FirstTestFailedDtc
            | Self::FirstConfirmedDtc
            | Self::MostRecentTestFailedDtc
            | Self::MostRecentConfirmedDtc
            | Self::MirrorMemoryDtcByStatusMask
            | Self::EmissionsObdDtcByStatusMask
            | Self::DtcWithPermanentStatus => Shape::DtcAndStatusList,
            Self::DtcSnapshotIdentification => Shape::SnapshotIdentification,
            Self::DtcSnapshotRecordByDtcNumber => Shape::Snapshot,
            Self::DtcStoredDataByRecordNumber => Shape::StoredData,
            Self::DtcExtDataRecordByDtcNumber | Self::MirrorMemoryDtcExtDataRecordByDtcNumber => Shape::ExtData,
            Self::DtcBySeverityMaskRecord | Self::SeverityInformationOfDtc => Shape::Severity,
            Self::DtcFaultDetectionCounter => Shape::FaultDetectionCounter,
            Self::DtcExtDataRecordByRecordNumber => Shape::ExtDataByRecordNumber,
            Self::UserDefMemoryDtcByStatusMask => Shape::UserDefMemoryList,
            Self::UserDefMemoryDtcSnapshotRecordByDtcNumber => Shape::UserDefMemorySnapshot,
            Self::UserDefMemoryDtcExtDataRecordByDtcNumber => Shape::UserDefMemoryExtData,
            Self::WwhObdDtcByMaskRecord => Shape::WwhObdSeverity,
            Self::WwhObdDtcWithPermanentStatus => Shape::WwhObdList,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcFormatIdentifier {
    SaeJ2012DaFormat00,
    Iso14229Format,
    SaeJ1939Format,
    Iso11992Format,
    SaeJ2012DaFormat04,
}

impl DtcFormatIdentifier {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::SaeJ2012DaFormat00),
            0x01 => Some(Self::Iso14229Format),
            0x02 => Some(Self::SaeJ1939Format),
            0x03 => Some(Self::Iso11992Format),
            0x04 => Some(Self::SaeJ2012DaFormat04),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::SaeJ2012DaFormat00 => 0x00,
            Self::Iso14229Format => 0x01,
            Self::SaeJ1939Format => 0x02,
            Self::Iso11992Format => 0x03,
            Self::SaeJ2012DaFormat04 => 0x04,
        }
    }
}

pub trait DtcRecord: Copy {
    const LEN: usize;

    fn dtc(&self) -> u32;
    fn decode(bytes: &[u8]) -> Self;
    fn encode(&self, out: &mut [u8]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DtcAndStatus {
    pub dtc: u32,
    pub status_of_dtc: u8,
}

impl DtcRecord for DtcAndStatus {
    const LEN: usize = 4;

    fn dtc(&self) -> u32 {
        self.dtc
    }

    fn decode(bytes: &[u8]) -> Self {
        Self { dtc: read_dtc(bytes), status_of_dtc: bytes[3] }
    }

    fn encode(&self, out: &mut [u8]) {
        write_dtc(self.dtc, out);
        out[3] = self.status_of_dtc;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DtcSnapshotIdentification {
    pub dtc: u32,
    pub dtc_snapshot_record_number: u8,
}

impl DtcRecord for DtcSnapshotIdentification {
    const LEN: usize = 4;

    fn dtc(&self) -> u32 {
        self.dtc
    }

    fn decode(bytes: &[u8]) -> Self {
        Self { dtc: read_dtc(bytes), dtc_snapshot_record_number: bytes[3] }
    }

    fn encode(&self, out: &mut [u8]) {
        write_dtc(self.dtc, out);
        out[3] = self.dtc_snapshot_record_number;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DtcAndSeverity {
    pub dtc_severity: u8,
    pub dtc_functional_unit: u8,
    pub dtc: u32,
    pub status_of_dtc: u8,
}

impl DtcRecord for DtcAndSeverity {
    const LEN: usize = 6;

    fn dtc(&self) -> u32 {
        self.dtc
    }

    fn decode(bytes: &[u8]) -> Self {
        Self {
            dtc_severity: bytes[0],
            dtc_functional_unit: bytes[1],
            dtc: read_dtc(&bytes[2..]),
            status_of_dtc: bytes[5],
        }
    }

    fn encode(&self, out: &mut [u8]) {
        out[0] = self.dtc_severity;
        out[1] = self.dtc_functional_unit;
        write_dtc(self.dtc, &mut out[2..]);
        out[5] = self.status_of_dtc;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WwhObdDtcAndSeverity {
    pub dtc_severity: u8,
    pub dtc: u32,
    pub status_of_dtc: u8,
}

impl DtcRecord for WwhObdDtcAndSeverity {
    const LEN: usize = 5;

    fn dtc(&self) -> u32 {
        self.dtc
    }

    fn decode(bytes: &[u8]) -> Self {
        Self { dtc_severity: bytes[0], dtc: read_dtc(&bytes[1..]), status_of_dtc: bytes[4] }
    }

    fn encode(&self, out: &mut [u8]) {
        out[0] = self.dtc_severity;
        write_dtc(self.dtc, &mut out[1..]);
        out[4] = self.status_of_dtc;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DtcFaultDetectionCounter {
    pub dtc: u32,
    pub dtc_fault_detection_counter: u8,
}

impl DtcRecord for DtcFaultDetectionCounter {
    const LEN: usize = 4;

    fn dtc(&self) -> u32 {
        self.dtc
    }

    fn decode(bytes: &[u8]) -> Self {
        Self { dtc: read_dtc(bytes), dtc_fault_detection_counter: bytes[3] }
    }

    fn encode(&self, out: &mut [u8]) {
        write_dtc(self.dtc, out);
        out[3] = self.dtc_fault_detection_counter;
    }
}

pub struct DtcRecords<'a, T> {
    remaining: &'a [u8],
    _record: PhantomData<T>,
}

impl<'a, T: DtcRecord> DtcRecords<'a, T> {
    const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining, _record: PhantomData }
    }
}

impl<T: DtcRecord> Iterator for DtcRecords<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.remaining.len() < T::LEN {
            return None;
        }
        let (record, rest) = self.remaining.split_at(T::LEN);
        self.remaining = rest;
        Some(T::decode(record))
    }
}

fn split_identifiers<'a, F: Fn(u16) -> Option<usize>>(
    data: &'a [u8],
    number_of_identifiers: u8,
    len_of: &F,
) -> Result<(&'a [u8], &'a [u8]), UdsNrc> {
    if number_of_identifiers == 0 {
        return Ok((data, &[]));
    }
    let mut offset = 0;
    for _ in 0..number_of_identifiers {
        let did = data.get(offset..offset + DID_LEN).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let len = len_of(u16::from_be_bytes([did[0], did[1]])).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        offset += DID_LEN + len;
        if offset > data.len() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
    }
    Ok(data.split_at(offset))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotRecord<'a> {
    pub dtc_snapshot_record_number: u8,
    pub number_of_identifiers: u8,
    pub data: &'a [u8],
}

impl<'a> SnapshotRecord<'a> {
    pub fn identifiers<F: Fn(u16) -> Option<usize>>(&self, len_of: F) -> x22_ReadDataByIdentifierRecords<'a, F> {
        x22_ReadDataByIdentifierRecords::new(self.data, len_of)
    }
}

pub struct SnapshotRecords<'a, F> {
    remaining: &'a [u8],
    len_of: F,
}

impl<'a, F: Fn(u16) -> Option<usize>> SnapshotRecords<'a, F> {
    fn next_record(&mut self) -> Result<SnapshotRecord<'a>, UdsNrc> {
        let remaining = self.remaining;
        let [number, count, rest @ ..] = remaining else {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        };
        let (data, rest) = split_identifiers(rest, *count, &self.len_of)?;
        self.remaining = rest;
        Ok(SnapshotRecord { dtc_snapshot_record_number: *number, number_of_identifiers: *count, data })
    }
}

impl<'a, F: Fn(u16) -> Option<usize>> Iterator for SnapshotRecords<'a, F> {
    type Item = Result<SnapshotRecord<'a>, UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let record = self.next_record();
        if record.is_err() {
            self.remaining = &[];
        }
        Some(record)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredDataRecord<'a> {
    pub dtc_stored_data_record_number: u8,
    pub dtc_and_status: DtcAndStatus,
    pub number_of_identifiers: u8,
    pub data: &'a [u8],
}

impl<'a> StoredDataRecord<'a> {
    pub fn identifiers<F: Fn(u16) -> Option<usize>>(&self, len_of: F) -> x22_ReadDataByIdentifierRecords<'a, F> {
        x22_ReadDataByIdentifierRecords::new(self.data, len_of)
    }
}

pub struct StoredDataRecords<'a, F> {
    remaining: &'a [u8],
    len_of: F,
    first: bool,
}

impl<'a, F: Fn(u16) -> Option<usize>> StoredDataRecords<'a, F> {
    const HEADER_LEN: usize = 1 + DtcAndStatus::LEN + 1;

    fn next_record(&mut self) -> Result<StoredDataRecord<'a>, UdsNrc> {
        let remaining = self.remaining;
        if remaining.len() < Self::HEADER_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let number_of_identifiers = remaining[Self::HEADER_LEN - 1];
        let (data, rest) = split_identifiers(&remaining[Self::HEADER_LEN..], number_of_identifiers, &self.len_of)?;
        self.remaining = rest;
        Ok(StoredDataRecord {
            dtc_stored_data_record_number: remaining[0],
            dtc_and_status: DtcAndStatus::decode(&remaining[1..]),
            number_of_identifiers,
            data,
        })
    }
}

impl<'a, F: Fn(u16) -> Option<usize>> Iterator for StoredDataRecords<'a, F> {
    type Item = Result<StoredDataRecord<'a>, UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = core::mem::replace(&mut self.first, false);
        if self.remaining.is_empty() || (first && self.remaining.len() == 1) {
            self.remaining = &[];
            return None;
        }
        let record = self.next_record();
        if record.is_err() {
            self.remaining = &[];
        }
        Some(record)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtDataRecord<'a> {
    pub dtc_ext_data_record_number: u8,
    pub data: &'a [u8],
}

pub struct ExtDataRecords<'a, F> {
    remaining: &'a [u8],
    len_of: F,
}

impl<'a, F: Fn(u8) -> Option<usize>> ExtDataRecords<'a, F> {
    fn next_record(&mut self) -> Result<ExtDataRecord<'a>, UdsNrc> {
        let remaining = self.remaining;
        let [number, rest @ ..] = remaining else {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        };
        let len = (self.len_of)(*number).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        if rest.len() < len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (data, rest) = rest.split_at(len);
        self.remaining = rest;
        Ok(ExtDataRecord { dtc_ext_data_record_number: *number, data })
    }
}

impl<'a, F: Fn(u8) -> Option<usize>> Iterator for ExtDataRecords<'a, F> {
    type Item = Result<ExtDataRecord<'a>, UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let record = self.next_record();
        if record.is_err() {
            self.remaining = &[];
        }
        Some(record)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DtcExtDataRecord<'a> {
    pub dtc_and_status: DtcAndStatus,
    pub data: &'a [u8],
}

pub struct DtcExtDataRecords<'a> {
    remaining: &'a [u8],
    record_len: usize,
}

impl<'a> Iterator for DtcExtDataRecords<'a> {
    type Item = Result<DtcExtDataRecord<'a>, UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let stride = DtcAndStatus::LEN + self.record_len;
        if self.remaining.len() < stride {
            self.remaining = &[];
            return Some(Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT));
        }
        let (record, rest) = self.remaining.split_at(stride);
        self.remaining = rest;
        Some(Ok(DtcExtDataRecord {
            dtc_and_status: DtcAndStatus::decode(record),
            data: &record[DtcAndStatus::LEN..],
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportParameters {
    None,
    StatusMask { dtc_status_mask: u8 },
    DtcMaskRecord { dtc: u32 },
    DtcAndRecordNumber { dtc: u32, record_number: u8 },
    RecordNumber { record_number: u8 },
    SeverityMaskRecord { dtc_severity_mask: u8, dtc_status_mask: u8 },
    StatusMaskAndMemorySelection { dtc_status_mask: u8, memory_selection: u8 },
    DtcAndRecordNumberAndMemorySelection { dtc: u32, record_number: u8, memory_selection: u8 },
    FunctionalGroupAndMaskRecord { functional_group_identifier: u8, dtc_status_mask: u8, dtc_severity_mask: u8 },
    FunctionalGroup { functional_group_identifier: u8 },
}

impl ReportParameters {
    const fn template(report_type: ReportType) -> Self {
        match report_type {
            ReportType::DtcSnapshotIdentification
            | ReportType::SupportedDtc
            | ReportType::FirstTestFailedDtc
            | ReportType::FirstConfirmedDtc
            | ReportType::MostRecentTestFailedDtc
            | ReportType::MostRecentConfirmedDtc
            | ReportType::DtcFaultDetectionCounter
            | ReportType::DtcWithPermanentStatus => Self::None,
            ReportType::NumberOfDtcByStatusMask
            | ReportType::DtcByStatusMask
            | ReportType::MirrorMemoryDtcByStatusMask
            | ReportType::NumberOfMirrorMemoryDtcByStatusMask
            | ReportType::NumberOfEmissionsObdDtcByStatusMask
            | ReportType::EmissionsObdDtcByStatusMask => Self::StatusMask { dtc_status_mask: 0 },
            ReportType::SeverityInformationOfDtc => Self::DtcMaskRecord { dtc: 0 },
            ReportType::DtcSnapshotRecordByDtcNumber
            | ReportType::DtcExtDataRecordByDtcNumber
            | ReportType::MirrorMemoryDtcExtDataRecordByDtcNumber => Self::DtcAndRecordNumber { dtc: 0, record_number: 0 },
            ReportType::DtcStoredDataByRecordNumber | ReportType::DtcExtDataRecordByRecordNumber => {
                Self::RecordNumber { record_number: 0 }
            }
            ReportType::NumberOfDtcBySeverityMaskRecord | ReportType::DtcBySeverityMaskRecord => {
                Self::SeverityMaskRecord { dtc_severity_mask: 0, dtc_status_mask: 0 }
            }
            ReportType::UserDefMemoryDtcByStatusMask => {
                Self::StatusMaskAndMemorySelection { dtc_status_mask: 0, memory_selection: 0 }
            }
            ReportType::UserDefMemoryDtcSnapshotRecordByDtcNumber
            | ReportType::UserDefMemoryDtcExtDataRecordByDtcNumber => {
                Self::DtcAndRecordNumberAndMemorySelection { dtc: 0, record_number: 0, memory_selection: 0 }
            }
            ReportType::WwhObdDtcByMaskRecord => Self::FunctionalGroupAndMaskRecord {
                functional_group_identifier: 0,
                dtc_status_mask: 0,
                dtc_severity_mask: 0,
            },
            ReportType::WwhObdDtcWithPermanentStatus => Self::FunctionalGroup { functional_group_identifier: 0 },
        }
    }

    pub const fn encoded_len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::StatusMask { .. } | Self::RecordNumber { .. } | Self::FunctionalGroup { .. } => 1,
            Self::SeverityMaskRecord { .. } | Self::StatusMaskAndMemorySelection { .. } => 2,
            Self::DtcMaskRecord { .. } | Self::FunctionalGroupAndMaskRecord { .. } => 3,
            Self::DtcAndRecordNumber { .. } => 4,
            Self::DtcAndRecordNumberAndMemorySelection { .. } => 5,
        }
    }

    const fn dtc(&self) -> Option<u32> {
        match *self {
            Self::DtcMaskRecord { dtc }
            | Self::DtcAndRecordNumber { dtc, .. }
            | Self::DtcAndRecordNumberAndMemorySelection { dtc, .. } => Some(dtc),
            _ => None,
        }
    }

    fn decode(report_type: ReportType, data: &[u8]) -> Result<Self, UdsNrc> {
        let template = Self::template(report_type);
        if data.len() != template.encoded_len() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(match template {
            Self::None => Self::None,
            Self::StatusMask { .. } => Self::StatusMask { dtc_status_mask: data[0] },
            Self::DtcMaskRecord { .. } => Self::DtcMaskRecord { dtc: read_dtc(data) },
            Self::DtcAndRecordNumber { .. } => Self::DtcAndRecordNumber { dtc: read_dtc(data), record_number: data[3] },
            Self::RecordNumber { .. } => Self::RecordNumber { record_number: data[0] },
            Self::SeverityMaskRecord { .. } => {
                Self::SeverityMaskRecord { dtc_severity_mask: data[0], dtc_status_mask: data[1] }
            }
            Self::StatusMaskAndMemorySelection { .. } => {
                Self::StatusMaskAndMemorySelection { dtc_status_mask: data[0], memory_selection: data[1] }
            }
            Self::DtcAndRecordNumberAndMemorySelection { .. } => Self::DtcAndRecordNumberAndMemorySelection {
                dtc: read_dtc(data),
                record_number: data[3],
                memory_selection: data[4],
            },
            Self::FunctionalGroupAndMaskRecord { .. } => Self::FunctionalGroupAndMaskRecord {
                functional_group_identifier: data[0],
                dtc_status_mask: data[1],
                dtc_severity_mask: data[2],
            },
            Self::FunctionalGroup { .. } => Self::FunctionalGroup { functional_group_identifier: data[0] },
        })
    }

    fn encode(&self, out: &mut [u8]) {
        match *self {
            Self::None => {}
            Self::StatusMask { dtc_status_mask } => out[0] = dtc_status_mask,
            Self::DtcMaskRecord { dtc } => write_dtc(dtc, out),
            Self::DtcAndRecordNumber { dtc, record_number } => {
                write_dtc(dtc, out);
                out[3] = record_number;
            }
            Self::RecordNumber { record_number } => out[0] = record_number,
            Self::SeverityMaskRecord { dtc_severity_mask, dtc_status_mask } => {
                out[0] = dtc_severity_mask;
                out[1] = dtc_status_mask;
            }
            Self::StatusMaskAndMemorySelection { dtc_status_mask, memory_selection } => {
                out[0] = dtc_status_mask;
                out[1] = memory_selection;
            }
            Self::DtcAndRecordNumberAndMemorySelection { dtc, record_number, memory_selection } => {
                write_dtc(dtc, out);
                out[3] = record_number;
                out[4] = memory_selection;
            }
            Self::FunctionalGroupAndMaskRecord { functional_group_identifier, dtc_status_mask, dtc_severity_mask } => {
                out[0] = functional_group_identifier;
                out[1] = dtc_status_mask;
                out[2] = dtc_severity_mask;
            }
            Self::FunctionalGroup { functional_group_identifier } => out[0] = functional_group_identifier,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct x19_ReadDtcInformation;

impl UdsService for x19_ReadDtcInformation {
    const SID: u8 = 0x19;

    type Request<'a> = x19_ReadDtcInformationRequest;
    type Response<'a> = x19_ReadDtcInformationResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x19_ReadDtcInformationRequest {
    pub subfunction: UdsSubfunction,
    pub parameters: ReportParameters,
}

impl x19_ReadDtcInformationRequest {
    pub const MIN_LEN: usize = 1;

    pub const fn new(report_type: ReportType, suppress_positive_response: bool, parameters: ReportParameters) -> Self {
        Self { subfunction: UdsSubfunction::from_parts(report_type.as_u8(), suppress_positive_response), parameters }
    }

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.parameters.encoded_len()
    }

    pub const fn report_type(&self) -> Option<ReportType> {
        ReportType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x19_ReadDtcInformationRequest {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&raw, parameters) = data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let subfunction = UdsSubfunction::new(raw);
        let report_type =
            ReportType::from_u8(subfunction.parameter_value()).ok_or(UdsNrc::SUB_FUNCTION_NOT_SUPPORTED)?;
        Ok(Self { subfunction, parameters: ReportParameters::decode(report_type, parameters)? })
    }
}

impl TryTo for x19_ReadDtcInformationRequest {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let report_type = self.report_type().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        if discriminant(&self.parameters) != discriminant(&ReportParameters::template(report_type)) {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if let Some(dtc) = self.parameters.dtc() {
            check_dtc(dtc)?;
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        self.parameters.encode(&mut buf[1..len]);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x19_ReadDtcInformationRequest {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Count,
    DtcAndStatusList,
    SnapshotIdentification,
    Snapshot,
    StoredData,
    ExtData,
    Severity,
    FaultDetectionCounter,
    ExtDataByRecordNumber,
    UserDefMemoryList,
    UserDefMemorySnapshot,
    UserDefMemoryExtData,
    WwhObdSeverity,
    WwhObdList,
}

impl Shape {
    const fn header_len(self) -> usize {
        match self {
            Self::SnapshotIdentification | Self::StoredData | Self::FaultDetectionCounter => 0,
            Self::DtcAndStatusList | Self::Severity | Self::ExtDataByRecordNumber => 1,
            Self::UserDefMemoryList => 2,
            Self::WwhObdList => 3,
            Self::Count | Self::Snapshot | Self::ExtData | Self::WwhObdSeverity => 4,
            Self::UserDefMemorySnapshot | Self::UserDefMemoryExtData => 5,
        }
    }

    const fn accepts(self, records: &[u8]) -> bool {
        let stride = match self {
            Self::Count => return records.is_empty(),
            Self::StoredData => return !records.is_empty(),
            Self::DtcAndStatusList | Self::UserDefMemoryList | Self::WwhObdList => DtcAndStatus::LEN,
            Self::SnapshotIdentification => DtcSnapshotIdentification::LEN,
            Self::FaultDetectionCounter => DtcFaultDetectionCounter::LEN,
            Self::Severity => DtcAndSeverity::LEN,
            Self::WwhObdSeverity => WwhObdDtcAndSeverity::LEN,
            Self::Snapshot
            | Self::ExtData
            | Self::ExtDataByRecordNumber
            | Self::UserDefMemorySnapshot
            | Self::UserDefMemoryExtData => return true,
        };
        records.len().is_multiple_of(stride)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportBody<'a> {
    NumberOfDtc {
        dtc_status_availability_mask: u8,
        dtc_format_identifier: u8,
        dtc_count: u16,
    },
    DtcAndStatusList {
        dtc_status_availability_mask: u8,
        records: &'a [u8],
    },
    SnapshotIdentification {
        records: &'a [u8],
    },
    SnapshotRecords {
        dtc_and_status: DtcAndStatus,
        records: &'a [u8],
    },
    StoredDataRecords {
        records: &'a [u8],
    },
    ExtDataRecords {
        dtc_and_status: DtcAndStatus,
        records: &'a [u8],
    },
    DtcAndSeverityList {
        dtc_status_availability_mask: u8,
        records: &'a [u8],
    },
    FaultDetectionCounters {
        records: &'a [u8],
    },
    ExtDataByRecordNumber {
        dtc_ext_data_record_number: u8,
        records: &'a [u8],
    },
    UserDefMemoryDtcAndStatusList {
        memory_selection: u8,
        dtc_status_availability_mask: u8,
        records: &'a [u8],
    },
    UserDefMemorySnapshotRecords {
        memory_selection: u8,
        dtc_and_status: DtcAndStatus,
        records: &'a [u8],
    },
    UserDefMemoryExtDataRecords {
        memory_selection: u8,
        dtc_and_status: DtcAndStatus,
        records: &'a [u8],
    },
    WwhObdDtcAndSeverityList {
        functional_group_identifier: u8,
        dtc_status_availability_mask: u8,
        dtc_severity_availability_mask: u8,
        dtc_format_identifier: u8,
        records: &'a [u8],
    },
    WwhObdDtcAndStatusList {
        functional_group_identifier: u8,
        dtc_status_availability_mask: u8,
        dtc_format_identifier: u8,
        records: &'a [u8],
    },
}

impl<'a> ReportBody<'a> {
    const fn shape(&self) -> Shape {
        match self {
            Self::NumberOfDtc { .. } => Shape::Count,
            Self::DtcAndStatusList { .. } => Shape::DtcAndStatusList,
            Self::SnapshotIdentification { .. } => Shape::SnapshotIdentification,
            Self::SnapshotRecords { .. } => Shape::Snapshot,
            Self::StoredDataRecords { .. } => Shape::StoredData,
            Self::ExtDataRecords { .. } => Shape::ExtData,
            Self::DtcAndSeverityList { .. } => Shape::Severity,
            Self::FaultDetectionCounters { .. } => Shape::FaultDetectionCounter,
            Self::ExtDataByRecordNumber { .. } => Shape::ExtDataByRecordNumber,
            Self::UserDefMemoryDtcAndStatusList { .. } => Shape::UserDefMemoryList,
            Self::UserDefMemorySnapshotRecords { .. } => Shape::UserDefMemorySnapshot,
            Self::UserDefMemoryExtDataRecords { .. } => Shape::UserDefMemoryExtData,
            Self::WwhObdDtcAndSeverityList { .. } => Shape::WwhObdSeverity,
            Self::WwhObdDtcAndStatusList { .. } => Shape::WwhObdList,
        }
    }

    const fn records(&self) -> &'a [u8] {
        match *self {
            Self::NumberOfDtc { .. } => &[],
            Self::DtcAndStatusList { records, .. }
            | Self::SnapshotIdentification { records }
            | Self::SnapshotRecords { records, .. }
            | Self::StoredDataRecords { records }
            | Self::ExtDataRecords { records, .. }
            | Self::DtcAndSeverityList { records, .. }
            | Self::FaultDetectionCounters { records }
            | Self::ExtDataByRecordNumber { records, .. }
            | Self::UserDefMemoryDtcAndStatusList { records, .. }
            | Self::UserDefMemorySnapshotRecords { records, .. }
            | Self::UserDefMemoryExtDataRecords { records, .. }
            | Self::WwhObdDtcAndSeverityList { records, .. }
            | Self::WwhObdDtcAndStatusList { records, .. } => records,
        }
    }

    fn decode(report_type: ReportType, data: &'a [u8]) -> Result<Self, UdsNrc> {
        let shape = report_type.shape();
        if data.len() < shape.header_len() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let (h, records) = data.split_at(shape.header_len());
        if !shape.accepts(records) {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(match shape {
            Shape::Count => Self::NumberOfDtc {
                dtc_status_availability_mask: h[0],
                dtc_format_identifier: h[1],
                dtc_count: u16::from_be_bytes([h[2], h[3]]),
            },
            Shape::DtcAndStatusList => Self::DtcAndStatusList { dtc_status_availability_mask: h[0], records },
            Shape::SnapshotIdentification => Self::SnapshotIdentification { records },
            Shape::Snapshot => Self::SnapshotRecords { dtc_and_status: DtcAndStatus::decode(h), records },
            Shape::StoredData => Self::StoredDataRecords { records },
            Shape::ExtData => Self::ExtDataRecords { dtc_and_status: DtcAndStatus::decode(h), records },
            Shape::Severity => Self::DtcAndSeverityList { dtc_status_availability_mask: h[0], records },
            Shape::FaultDetectionCounter => Self::FaultDetectionCounters { records },
            Shape::ExtDataByRecordNumber => Self::ExtDataByRecordNumber { dtc_ext_data_record_number: h[0], records },
            Shape::UserDefMemoryList => Self::UserDefMemoryDtcAndStatusList {
                memory_selection: h[0],
                dtc_status_availability_mask: h[1],
                records,
            },
            Shape::UserDefMemorySnapshot => Self::UserDefMemorySnapshotRecords {
                memory_selection: h[0],
                dtc_and_status: DtcAndStatus::decode(&h[1..]),
                records,
            },
            Shape::UserDefMemoryExtData => Self::UserDefMemoryExtDataRecords {
                memory_selection: h[0],
                dtc_and_status: DtcAndStatus::decode(&h[1..]),
                records,
            },
            Shape::WwhObdSeverity => Self::WwhObdDtcAndSeverityList {
                functional_group_identifier: h[0],
                dtc_status_availability_mask: h[1],
                dtc_severity_availability_mask: h[2],
                dtc_format_identifier: h[3],
                records,
            },
            Shape::WwhObdList => Self::WwhObdDtcAndStatusList {
                functional_group_identifier: h[0],
                dtc_status_availability_mask: h[1],
                dtc_format_identifier: h[2],
                records,
            },
        })
    }

    fn encode_header(&self, h: &mut [u8]) -> Result<(), UdsNrc> {
        match *self {
            Self::NumberOfDtc { dtc_status_availability_mask, dtc_format_identifier, dtc_count } => {
                h[0] = dtc_status_availability_mask;
                h[1] = dtc_format_identifier;
                h[2..4].copy_from_slice(&dtc_count.to_be_bytes());
            }
            Self::DtcAndStatusList { dtc_status_availability_mask, .. }
            | Self::DtcAndSeverityList { dtc_status_availability_mask, .. } => h[0] = dtc_status_availability_mask,
            Self::SnapshotIdentification { .. } | Self::StoredDataRecords { .. } | Self::FaultDetectionCounters { .. } => {}
            Self::SnapshotRecords { dtc_and_status, .. } | Self::ExtDataRecords { dtc_and_status, .. } => {
                check_dtc(dtc_and_status.dtc)?;
                dtc_and_status.encode(h);
            }
            Self::ExtDataByRecordNumber { dtc_ext_data_record_number, .. } => h[0] = dtc_ext_data_record_number,
            Self::UserDefMemoryDtcAndStatusList { memory_selection, dtc_status_availability_mask, .. } => {
                h[0] = memory_selection;
                h[1] = dtc_status_availability_mask;
            }
            Self::UserDefMemorySnapshotRecords { memory_selection, dtc_and_status, .. }
            | Self::UserDefMemoryExtDataRecords { memory_selection, dtc_and_status, .. } => {
                check_dtc(dtc_and_status.dtc)?;
                h[0] = memory_selection;
                dtc_and_status.encode(&mut h[1..]);
            }
            Self::WwhObdDtcAndSeverityList {
                functional_group_identifier,
                dtc_status_availability_mask,
                dtc_severity_availability_mask,
                dtc_format_identifier,
                ..
            } => {
                h[0] = functional_group_identifier;
                h[1] = dtc_status_availability_mask;
                h[2] = dtc_severity_availability_mask;
                h[3] = dtc_format_identifier;
            }
            Self::WwhObdDtcAndStatusList {
                functional_group_identifier, dtc_status_availability_mask, dtc_format_identifier, ..
            } => {
                h[0] = functional_group_identifier;
                h[1] = dtc_status_availability_mask;
                h[2] = dtc_format_identifier;
            }
        }
        Ok(())
    }

    pub fn dtc_and_status_records(&self) -> Option<DtcRecords<'a, DtcAndStatus>> {
        match *self {
            Self::DtcAndStatusList { records, .. }
            | Self::UserDefMemoryDtcAndStatusList { records, .. }
            | Self::WwhObdDtcAndStatusList { records, .. } => Some(DtcRecords::new(records)),
            _ => None,
        }
    }

    pub fn snapshot_identifications(&self) -> Option<DtcRecords<'a, DtcSnapshotIdentification>> {
        match *self {
            Self::SnapshotIdentification { records } => Some(DtcRecords::new(records)),
            _ => None,
        }
    }

    pub fn dtc_and_severity_records(&self) -> Option<DtcRecords<'a, DtcAndSeverity>> {
        match *self {
            Self::DtcAndSeverityList { records, .. } => Some(DtcRecords::new(records)),
            _ => None,
        }
    }

    pub fn wwh_obd_dtc_and_severity_records(&self) -> Option<DtcRecords<'a, WwhObdDtcAndSeverity>> {
        match *self {
            Self::WwhObdDtcAndSeverityList { records, .. } => Some(DtcRecords::new(records)),
            _ => None,
        }
    }

    pub fn fault_detection_counters(&self) -> Option<DtcRecords<'a, DtcFaultDetectionCounter>> {
        match *self {
            Self::FaultDetectionCounters { records } => Some(DtcRecords::new(records)),
            _ => None,
        }
    }

    pub fn snapshot_records<F: Fn(u16) -> Option<usize>>(&self, len_of: F) -> Option<SnapshotRecords<'a, F>> {
        match *self {
            Self::SnapshotRecords { records, .. } | Self::UserDefMemorySnapshotRecords { records, .. } => {
                Some(SnapshotRecords { remaining: records, len_of })
            }
            _ => None,
        }
    }

    pub fn stored_data_records<F: Fn(u16) -> Option<usize>>(&self, len_of: F) -> Option<StoredDataRecords<'a, F>> {
        match *self {
            Self::StoredDataRecords { records } => Some(StoredDataRecords { remaining: records, len_of, first: true }),
            _ => None,
        }
    }

    pub fn ext_data_records<F: Fn(u8) -> Option<usize>>(&self, len_of: F) -> Option<ExtDataRecords<'a, F>> {
        match *self {
            Self::ExtDataRecords { records, .. } | Self::UserDefMemoryExtDataRecords { records, .. } => {
                Some(ExtDataRecords { remaining: records, len_of })
            }
            _ => None,
        }
    }

    pub fn dtc_ext_data_records(&self, record_len: usize) -> Option<DtcExtDataRecords<'a>> {
        match *self {
            Self::ExtDataByRecordNumber { records, .. } => Some(DtcExtDataRecords { remaining: records, record_len }),
            _ => None,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x19_ReadDtcInformationResponse<'a> {
    pub subfunction: UdsSubfunction,
    pub body: ReportBody<'a>,
}

impl<'a> x19_ReadDtcInformationResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn new(report_type: ReportType, body: ReportBody<'a>) -> Self {
        Self { subfunction: UdsSubfunction::from_parts(report_type.as_u8(), false), body }
    }

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.body.shape().header_len() + self.body.records().len()
    }

    pub const fn report_type(&self) -> Option<ReportType> {
        ReportType::from_u8(self.subfunction.parameter_value())
    }
}

impl<'a> TryFrom<&'a [u8]> for x19_ReadDtcInformationResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        let (&raw, body) = data.split_first().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let subfunction = UdsSubfunction::new(raw);
        let report_type =
            ReportType::from_u8(subfunction.parameter_value()).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        Ok(Self { subfunction, body: ReportBody::decode(report_type, body)? })
    }
}

impl TryTo for x19_ReadDtcInformationResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let report_type = self.report_type().ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let shape = report_type.shape();
        let records = self.body.records();
        if self.body.shape() != shape || !shape.accepts(records) {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        let header_end = Self::MIN_LEN + shape.header_len();
        buf[0] = self.subfunction.raw();
        self.body.encode_header(&mut buf[Self::MIN_LEN..header_end])?;
        buf[header_end..len].copy_from_slice(records);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x19_ReadDtcInformationResponse<'a> {}

#[allow(non_camel_case_types)]
pub struct x19_RecordWriter<'b> {
    buf: &'b mut [u8],
    len: usize,
    identifier_count_at: Option<usize>,
    identifier_count: usize,
}

impl<'b> x19_RecordWriter<'b> {
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, len: 0, identifier_count_at: None, identifier_count: 0 }
    }

    pub fn push<T: DtcRecord>(&mut self, record: T) -> Result<(), UdsNrc> {
        check_dtc(record.dtc())?;
        record.encode(self.reserve(T::LEN)?);
        self.identifier_count_at = None;
        Ok(())
    }

    pub fn push_ext_data(&mut self, dtc_ext_data_record_number: u8, data: &[u8]) -> Result<(), UdsNrc> {
        let out = self.reserve(1 + data.len())?;
        out[0] = dtc_ext_data_record_number;
        out[1..].copy_from_slice(data);
        self.identifier_count_at = None;
        Ok(())
    }

    pub fn push_dtc_ext_data(&mut self, dtc_and_status: DtcAndStatus, data: &[u8]) -> Result<(), UdsNrc> {
        check_dtc(dtc_and_status.dtc)?;
        let out = self.reserve(DtcAndStatus::LEN + data.len())?;
        dtc_and_status.encode(out);
        out[DtcAndStatus::LEN..].copy_from_slice(data);
        self.identifier_count_at = None;
        Ok(())
    }

    pub fn begin_snapshot_record(&mut self, dtc_snapshot_record_number: u8) -> Result<(), UdsNrc> {
        let out = self.reserve(2)?;
        out[0] = dtc_snapshot_record_number;
        out[1] = 0;
        self.begin_identifiers();
        Ok(())
    }

    pub fn begin_stored_data_record(
        &mut self,
        dtc_stored_data_record_number: u8,
        dtc_and_status: DtcAndStatus,
    ) -> Result<(), UdsNrc> {
        check_dtc(dtc_and_status.dtc)?;
        let out = self.reserve(1 + DtcAndStatus::LEN + 1)?;
        out[0] = dtc_stored_data_record_number;
        dtc_and_status.encode(&mut out[1..]);
        out[1 + DtcAndStatus::LEN] = 0;
        self.begin_identifiers();
        Ok(())
    }

    pub fn push_identifier(&mut self, data_identifier: u16, data: &[u8]) -> Result<(), UdsNrc> {
        let count_at = self.identifier_count_at.ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let out = self.reserve(DID_LEN + data.len())?;
        out[..DID_LEN].copy_from_slice(&data_identifier.to_be_bytes());
        out[DID_LEN..].copy_from_slice(data);
        self.identifier_count += 1;
        self.buf[count_at] = u8::try_from(self.identifier_count).unwrap_or(0);
        Ok(())
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn finish(self) -> &'b [u8] {
        let buf: &'b [u8] = self.buf;
        &buf[..self.len]
    }

    fn begin_identifiers(&mut self) {
        self.identifier_count_at = Some(self.len - 1);
        self.identifier_count = 0;
    }

    fn reserve(&mut self, n: usize) -> Result<&mut [u8], UdsNrc> {
        let start = self.len;
        let end = start + n;
        if self.buf.len() < end {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        self.len = end;
        Ok(&mut self.buf[start..end])
    }
}
