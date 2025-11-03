use crate::info;
use crate::info::info_def::InfoSlot;
use log::{debug, error, info, warn};
use std::convert::TryInto;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use thiserror::Error;

const DATA_FILE_PATH: &str = "/spiffs/info_storage.dat";
const META_FILE_PATH: &str = "/spiffs/info_storage.meta";
const RECORD_MAGIC: u16 = 0x4952; // "IR"
const META_MAGIC: u32 = 0x4D455441; // "META"
const META_VERSION: u16 = 1;
const RECORD_SIZE: usize = 16;
const META_RECORD_SIZE: usize = 24;
const META_COPIES: usize = 2;
const STORAGE_CAPACITY: u16 = 300;

#[derive(Debug, Error)]
pub enum InfoStorageError {
    #[error("storage read error")]
    ReadError,
    #[error("storage write error")]
    WriteError,
    #[error("storage initialization error")]
    InitializationError,
    #[error("metadata corrupted")]
    MetadataCorrupted,
    #[error("record corrupted")]
    RecordCorrupted,
    #[error("persistence error: {0}")]
    PersistenceError(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unknown error")]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default)]
struct StorageState {
    head: u16,
    tail: u16,
    count: u16,
    next_seq: u32,
    generation: u32,
}

#[derive(Clone, Copy, Debug)]
struct StoredRecord {
    seq: u32,
    slot: InfoSlot,
}

pub trait RecoverableStorage {
    fn recover(&mut self) -> Result<(), InfoStorageError>;
}

pub struct InfoStorage {
    data_file: File,
    meta_file: File,
    state: StorageState,
}

impl InfoStorage {
    pub fn new() -> Result<Self, InfoStorageError> {
        info!("InfoStorage: 打开数据文件 {DATA_FILE_PATH}");
        let data_file = Self::open_rw(DATA_FILE_PATH)?;
        info!("InfoStorage: 打开元数据文件 {META_FILE_PATH}");
        let meta_file = Self::open_rw(META_FILE_PATH)?;
        info!("InfoStorage: 文件打开成功，开始加载状态");

        let mut storage = Self {
            data_file,
            meta_file,
            state: StorageState::default(),
        };
        info!("InfoStorage: 确保数据文件长度正确");
        storage.ensure_data_file_len()?;
        info!("InfoStorage: 加载元数据");
        storage.state = storage.load_meta()?.unwrap_or_else(|| {
            info!("InfoStorage: 未找到有效元数据，采用默认状态");
            StorageState::default()
        });
        storage.recover_internal()?;
        Ok(storage)
    }

    fn open_rw(path: &str) -> Result<File, InfoStorageError> {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
        {
            Ok(file) => {
                debug!("InfoStorage: 成功以读写模式打开 {path}");
                Ok(file)
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                warn!("InfoStorage: 文件 {path} 不存在，尝试创建");
                File::create(path)?;
                debug!("InfoStorage(open_rw): 文件 {path} 创建完成，重新打开");
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)
                    .map(|file| {
                        debug!("InfoStorage: 成功在重试后打开 {path}");
                        file
                    })
                    .map_err(Into::into)
            }
            Err(err) => {
                error!("InfoStorage: 打开文件 {path} 失败: {err:?}");
                Err(InfoStorageError::from(err))
            }
        }
    }

    fn recreate_file(path: &str) -> Result<File, InfoStorageError> {
        warn!("InfoStorage: 重新创建文件 {path}");
        match remove_file(path) {
            Ok(()) => debug!("InfoStorage: 已删除旧文件 {path}"),
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => warn!("InfoStorage: 删除旧文件 {path} 失败: {err:?}"),
        }
        info!("InfoStorage: 移除旧文件成功 {path}");
        File::create(path).map_err(|err| {
            error!("InfoStorage: 创建文件 {path} 失败: {err:?}");
            InfoStorageError::from(err)
        })?;
        info!("InfoStorage(recreate_file): 文件 {path} 创建完成，重新打开");
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| {
                error!("InfoStorage: 重新打开文件 {path} 失败: {err:?}");
                InfoStorageError::from(err)
            })
    }

    pub fn capacity(&self) -> usize {
        STORAGE_CAPACITY as usize
    }

    pub fn len(&self) -> usize {
        self.state.count as usize
    }

    pub fn enqueue(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError> {
        let seq = self.state.next_seq;
        let index = self.state.tail;
        self.write_record(index, seq, info)?;

        if self.state.count == STORAGE_CAPACITY {
            self.state.head = self.advance(self.state.head, 1);
        } else {
            self.state.count = self.state.count.saturating_add(1);
        }

        self.state.tail = self.advance(self.state.tail, 1);
        self.state.next_seq = self.state.next_seq.wrapping_add(1);
        self.state.generation = self.state.generation.wrapping_add(1);
        self.write_meta()
    }

    pub fn dequeue(&mut self) -> Result<InfoSlot, InfoStorageError> {
        if self.state.count == 0 {
            return Err(InfoStorageError::ReadError);
        }

        let index = self.state.head;
        let record = self
            .read_record(index)?
            .ok_or(InfoStorageError::RecordCorrupted)?;

        self.state.head = self.advance(self.state.head, 1);
        self.state.count = self.state.count.saturating_sub(1);
        if self.state.count == 0 {
            self.state.tail = self.state.head;
        }
        self.state.generation = self.state.generation.wrapping_add(1);
        self.write_meta()?;
        Ok(record.slot)
    }

    pub fn find_range(
        &mut self,
        start_time: u32,
        end_time: u32,
    ) -> Result<Vec<InfoSlot>, InfoStorageError> {
        let mut result = Vec::new();
        self.scan_ring(|record| {
            let ts = record.slot.get_unix_time();
            if ts >= start_time && ts <= end_time {
                result.push(record.slot);
            }
            Ok(())
        })?;
        Ok(result)
    }

    pub fn load_all(&mut self) -> Result<Vec<InfoSlot>, InfoStorageError> {
        let mut result = Vec::with_capacity(self.state.count as usize);
        self.scan_ring(|record| {
            result.push(record.slot);
            Ok(())
        })?;
        Ok(result)
    }

    pub fn load_info(&mut self, timestamp: u32) -> Result<Option<InfoSlot>, InfoStorageError> {
        let mut found = None;
        self.scan_ring(|record| {
            if record.slot.get_unix_time() == timestamp {
                found = Some(record.slot);
            }
            Ok(())
        })?;
        Ok(found)
    }

    pub fn erase_info(&mut self, timestamp: u32) -> Result<(), InfoStorageError> {
        let mut records = self.collect_records()?;
        let original_len = records.len();
        records.retain(|record| record.slot.get_unix_time() != timestamp);
        if records.len() == original_len {
            return Ok(());
        }
        self.rewrite_records(&records)
    }

    pub fn persist_all(
        &mut self,
        infos: impl IntoIterator<Item = InfoSlot>,
    ) -> Result<(), InfoStorageError> {
        for info in infos {
            self.enqueue(&info)?;
        }
        Ok(())
    }

    pub fn clear_storage(&mut self) -> Result<(), InfoStorageError> {
        self.zero_data_file()?;
        self.state = StorageState::default();
        self.write_meta()
    }

    pub fn clear_range(&mut self, start_time: u32, end_time: u32) -> Result<(), InfoStorageError> {
        let records = self.collect_records()?;
        let mut filtered = Vec::with_capacity(records.len());
        for record in records {
            let ts = record.slot.get_unix_time();
            if ts < start_time || ts > end_time {
                filtered.push(record);
            }
        }
        self.rewrite_records(&filtered)
    }

    pub fn recover(&mut self) -> Result<(), InfoStorageError> {
        self.recover_internal()
    }

    fn recover_internal(&mut self) -> Result<(), InfoStorageError> {
        if self.state.count == 0 {
            return Ok(());
        }

        if self.validate_ring()? {
            return Ok(());
        }

        self.full_scan_recovery()
    }

    fn ensure_data_file_len(&mut self) -> Result<(), InfoStorageError> {
        let expected_len = (RECORD_SIZE as u64) * (STORAGE_CAPACITY as u64);
        let actual_len = self.data_file.metadata()?.len();
        if actual_len != expected_len {
            warn!(
                "InfoStorage: 数据文件长度异常 (实际 {actual_len}, 期望 {expected_len})，重新初始化"
            );
            self.data_file = Self::recreate_file(DATA_FILE_PATH)?;
            info!("InfoStorage: 文件创建并打开成功");
            self.zero_data_file()?;
        } else {
            debug!("InfoStorage: 数据文件长度正常 {actual_len}");
        }
        Ok(())
    }

    fn advance(&self, index: u16, steps: u16) -> u16 {
        ((index as u32 + steps as u32) % STORAGE_CAPACITY as u32) as u16
    }

    fn write_record(
        &mut self,
        index: u16,
        seq: u32,
        slot: &InfoSlot,
    ) -> Result<(), InfoStorageError> {
        let mut buf = [0u8; RECORD_SIZE];
        buf[..2].copy_from_slice(&RECORD_MAGIC.to_le_bytes());
        buf[2..6].copy_from_slice(&seq.to_le_bytes());
        let raw = slot.as_bytes();
        buf[6..10].copy_from_slice(&raw[..4]);
        buf[10] = raw[4];
        buf[11] = raw[5];
        buf[12] = 0;
        buf[13] = 0;
        let crc = crc16_ccitt(&buf[..RECORD_SIZE - 2]);
        buf[RECORD_SIZE - 2..].copy_from_slice(&crc.to_le_bytes());

        self.seek_record(index)?;
        self.data_file.write_all(&buf).map_err(|err| {
            error!("InfoStorage: 写入记录失败 index={index} err={err:?}");
            InfoStorageError::from(err)
        })?;
        self.data_file.sync_data().map_err(|err| {
            error!("InfoStorage: 刷新数据文件失败 err={err:?}");
            InfoStorageError::from(err)
        })?;
        Ok(())
    }

    fn read_record(&mut self, index: u16) -> Result<Option<StoredRecord>, InfoStorageError> {
        let mut buf = [0u8; RECORD_SIZE];
        self.seek_record(index)?;
        self.data_file.read_exact(&mut buf)?;

        let magic = u16::from_le_bytes([buf[0], buf[1]]);
        if magic != RECORD_MAGIC {
            return Ok(None);
        }

        let crc_expected = u16::from_le_bytes([buf[RECORD_SIZE - 2], buf[RECORD_SIZE - 1]]);
        let crc_actual = crc16_ccitt(&buf[..RECORD_SIZE - 2]);
        if crc_expected != crc_actual {
            return Ok(None);
        }

        let seq = u32::from_le_bytes(buf[2..6].try_into().unwrap());
        let mut slot_bytes = [0u8; InfoSlot::SERIALIZED_SIZE];
        slot_bytes[..4].copy_from_slice(&buf[6..10]);
        slot_bytes[4] = buf[10];
        slot_bytes[5] = buf[11];
        let slot = InfoSlot::from_bytes(slot_bytes);

        Ok(Some(StoredRecord { seq, slot }))
    }

    fn seek_record(&mut self, index: u16) -> Result<(), InfoStorageError> {
        let offset = (index as u64) * (RECORD_SIZE as u64);
        self.data_file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }

    fn load_meta(&mut self) -> Result<Option<StorageState>, InfoStorageError> {
        let total_size = (META_RECORD_SIZE * META_COPIES) as u64;
        let current_len = self.meta_file.metadata()?.len();
        if current_len != total_size {
            warn!(
                "InfoStorage: 元数据文件长度异常 (实际 {current_len}, 期望 {total_size})，重新初始化"
            );
            self.meta_file = Self::recreate_file(META_FILE_PATH)?;
            info!("InfoStorage: 元数据文件创建并打开成功");
            self.write_empty_meta(total_size as usize)?;
            info!("InfoStorage: 元数据文件初始化完成");
            return Ok(None);
        }

        let mut buf = vec![0u8; META_RECORD_SIZE * META_COPIES];
        self.meta_file.seek(SeekFrom::Start(0))?;
        self.meta_file.read_exact(&mut buf)?;

        let mut best: Option<StorageState> = None;
        for chunk in buf.chunks_exact(META_RECORD_SIZE) {
            if let Some(state) = StorageState::from_bytes(chunk) {
                if best
                    .as_ref()
                    .map(|current| state.generation > current.generation)
                    .unwrap_or(true)
                {
                    best = Some(state);
                }
            }
        }

        Ok(best)
    }

    fn write_meta(&mut self) -> Result<(), InfoStorageError> {
        let state = self.state;
        let encoded = state.to_bytes();
        let mut buf = vec![0u8; META_RECORD_SIZE * META_COPIES];
        for chunk in buf.chunks_exact_mut(META_RECORD_SIZE) {
            chunk.copy_from_slice(&encoded);
        }
        self.meta_file.seek(SeekFrom::Start(0))?;
        self.meta_file.write_all(&buf).map_err(|err| {
            error!("InfoStorage: 写入元数据失败 err={err:?}");
            InfoStorageError::from(err)
        })?;
        self.meta_file.flush().map_err(|err| {
            error!("InfoStorage: 刷新元数据失败 err={err:?}");
            InfoStorageError::from(err)
        })?;
        Ok(())
    }

    fn scan_ring<F>(&mut self, mut visitor: F) -> Result<(), InfoStorageError>
    where
        F: FnMut(StoredRecord) -> Result<(), InfoStorageError>,
    {
        let mut index = self.state.head;
        for _ in 0..self.state.count {
            let record = self
                .read_record(index)?
                .ok_or(InfoStorageError::RecordCorrupted)?;
            visitor(record)?;
            index = self.advance(index, 1);
        }
        Ok(())
    }

    fn collect_records(&mut self) -> Result<Vec<StoredRecord>, InfoStorageError> {
        let mut records = Vec::with_capacity(self.state.count as usize);
        self.scan_ring(|record| {
            records.push(record);
            Ok(())
        })?;
        Ok(records)
    }

    fn rewrite_records(&mut self, records: &[StoredRecord]) -> Result<(), InfoStorageError> {
        if records.len() > STORAGE_CAPACITY as usize {
            return Err(InfoStorageError::WriteError);
        }

        self.zero_data_file()?;
        for (i, record) in records.iter().enumerate() {
            self.write_record(i as u16, record.seq, &record.slot)?;
        }

        self.state.head = 0;
        self.state.count = records.len() as u16;
        self.state.tail = self.advance(0, self.state.count);
        self.state.next_seq = records
            .last()
            .map(|record| record.seq.wrapping_add(1))
            .unwrap_or(self.state.next_seq);
        self.state.generation = self.state.generation.wrapping_add(1);
        self.write_meta()
    }

    fn zero_data_file(&mut self) -> Result<(), InfoStorageError> {
        info!("InfoStorage: 清零数据文件");
        self.data_file.seek(SeekFrom::Start(0))?;
        let zero_block = [0u8; RECORD_SIZE];
        for _ in 0..STORAGE_CAPACITY {
            self.data_file.write_all(&zero_block)?;
        }
        self.data_file.flush()?;
        self.data_file.seek(SeekFrom::Start(0))?;
        info!("InfoStorage: 数据文件清零完成");
        Ok(())
    }

    fn write_empty_meta(&mut self, total_size: usize) -> Result<(), InfoStorageError> {
        let zeros = vec![0u8; total_size];
        self.meta_file.write_all(&zeros)?;
        self.meta_file.flush()?;
        info!("InfoStorage: 空元数据写入完成");
        Ok(())
    }

    fn validate_ring(&mut self) -> Result<bool, InfoStorageError> {
        if self.state.count == 0 {
            return Ok(true);
        }

        let mut index = self.state.head;
        let mut last_seq = None;
        let mut last_ts = None;

        for _ in 0..self.state.count {
            let record = match self.read_record(index)? {
                Some(rec) => rec,
                None => return Ok(false),
            };

            if let Some(prev_seq) = last_seq {
                if record.seq <= prev_seq {
                    return Ok(false);
                }
            }

            if let Some(prev_ts) = last_ts {
                if record.slot.get_unix_time() < prev_ts {
                    return Ok(false);
                }
            }

            last_seq = Some(record.seq);
            last_ts = Some(record.slot.get_unix_time());
            index = self.advance(index, 1);
        }

        Ok(true)
    }

    fn full_scan_recovery(&mut self) -> Result<(), InfoStorageError> {
        let mut all_records = Vec::new();
        for idx in 0..STORAGE_CAPACITY {
            if let Some(record) = self.read_record(idx)? {
                all_records.push(record);
            }
        }

        if all_records.is_empty() {
            self.state = StorageState::default();
            return self.write_meta();
        }

        all_records.sort_by_key(|record| record.seq);
        if all_records.len() > STORAGE_CAPACITY as usize {
            all_records.drain(..all_records.len() - STORAGE_CAPACITY as usize);
        }

        self.zero_data_file()?;
        for (i, record) in all_records.iter().enumerate() {
            self.write_record(i as u16, record.seq, &record.slot)?;
        }

        self.state.head = 0;
        self.state.count = all_records.len() as u16;
        self.state.tail = self.advance(0, self.state.count);
        self.state.next_seq = all_records
            .last()
            .map(|record| record.seq.wrapping_add(1))
            .unwrap_or(0);
        self.state.generation = self.state.generation.wrapping_add(1);
        self.write_meta()
    }
}

impl RecoverableStorage for InfoStorage {
    fn recover(&mut self) -> Result<(), InfoStorageError> {
        self.recover_internal()
    }
}

impl StorageState {
    fn to_bytes(self) -> [u8; META_RECORD_SIZE] {
        let mut buf = [0u8; META_RECORD_SIZE];
        buf[..4].copy_from_slice(&META_MAGIC.to_le_bytes());
        buf[4..6].copy_from_slice(&META_VERSION.to_le_bytes());
        buf[6..8].copy_from_slice(&0u16.to_le_bytes());
        buf[8..12].copy_from_slice(&self.generation.to_le_bytes());
        buf[12..14].copy_from_slice(&self.head.to_le_bytes());
        buf[14..16].copy_from_slice(&self.tail.to_le_bytes());
        buf[16..18].copy_from_slice(&self.count.to_le_bytes());
        buf[18..22].copy_from_slice(&self.next_seq.to_le_bytes());
        let crc = crc16_ccitt(&buf[..META_RECORD_SIZE - 2]);
        buf[META_RECORD_SIZE - 2..].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != META_RECORD_SIZE {
            return None;
        }

        let magic = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let version = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
        if magic != META_MAGIC || version != META_VERSION {
            return None;
        }

        let crc_expected = u16::from_le_bytes(
            bytes[META_RECORD_SIZE - 2..META_RECORD_SIZE]
                .try_into()
                .ok()?,
        );
        let crc_actual = crc16_ccitt(&bytes[..META_RECORD_SIZE - 2]);
        if crc_expected != crc_actual {
            return None;
        }

        let generation = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let head = u16::from_le_bytes(bytes[12..14].try_into().ok()?);
        let tail = u16::from_le_bytes(bytes[14..16].try_into().ok()?);
        let count = u16::from_le_bytes(bytes[16..18].try_into().ok()?);
        let next_seq = u32::from_le_bytes(bytes[18..22].try_into().ok()?);

        if head >= STORAGE_CAPACITY || tail >= STORAGE_CAPACITY || count > STORAGE_CAPACITY {
            return None;
        }

        Some(Self {
            head,
            tail,
            count,
            next_seq,
            generation,
        })
    }
}

fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}
