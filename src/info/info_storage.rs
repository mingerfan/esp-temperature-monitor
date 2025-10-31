use crate::info::info_def::InfoSlot;
use crate::utils::circular_queue::CircularQueue;
use esp_idf_svc::{nvs, sys::EspError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfoStorageError {
    #[error("Storage capacity exceeded")]
    CapacityExceeded,
    #[error("Storage read error")]
    ReadError,
    #[error("Storage write error")]
    WriteError,
    #[error("Storage initialization error")]
    InitializationError,
    #[error("NVS error: {0}")]
    NvsError(#[from] EspError),
    #[error("Unknown error")]
    Unknown,
}

trait InfoRW {
    fn capacity(&self) -> usize;

    fn enqueue_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError>;

    fn dequeue_info(&mut self) -> Result<InfoSlot, InfoStorageError>;

    fn find_range(
        &self,
        start_time: u32,
        end_time: u32,
    ) -> Result<impl Iterator<Item = InfoSlot>, InfoStorageError>;

    fn clear_storage(&mut self) -> Result<(), InfoStorageError>;

    fn clear_range(&mut self, start_time: u32, end_time: u32) -> Result<(), InfoStorageError>;
}

trait InforPersistence {
    fn persist_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError>;

    fn load_info(&self, timestamp: u32) -> Result<InfoSlot, InfoStorageError>;

    fn erease_info(&mut self, timestamp: u32) -> Result<(), InfoStorageError>;

    fn persist_all(&mut self, infos: impl Iterator<Item = InfoSlot>) -> Result<(), InfoStorageError>;

    fn load_all(&self) -> Result<impl Iterator<Item = InfoSlot>, InfoStorageError>;

    fn clear_storage(&mut self) -> Result<(), InfoStorageError>;
}

/// 基于内存的信息存储（使用循环队列）
/// 适用于需要快速访问最近数据的场景
pub struct MemoryInfoStorage<const N: usize> {
    queue: CircularQueue<InfoSlot, N>,
}

impl<const N: usize> MemoryInfoStorage<N> {
    /// 创建新的内存存储
    pub const fn new() -> Self {
        Self {
            queue: CircularQueue::new(),
        }
    }
}

impl<const N: usize> InfoRW for MemoryInfoStorage<N> {
    fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    fn enqueue_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError> {
        // 使用覆盖模式，自动丢弃最旧的数据
        self.queue.push_overwrite(*info);
        Ok(())
    }

    fn dequeue_info(&mut self) -> Result<InfoSlot, InfoStorageError> {
        self.queue.pop().ok_or(InfoStorageError::ReadError)
    }

    fn find_range(
        &self,
        start_time: u32,
        end_time: u32,
    ) -> Result<impl Iterator<Item = InfoSlot>, InfoStorageError> {
        // 过滤出时间范围内的数据
        let filtered: Vec<InfoSlot> = self.queue
            .iter()
            .filter(|info| {
                let time = info.get_unix_time();
                time >= start_time && time <= end_time
            })
            .copied()
            .collect();
        
        Ok(filtered.into_iter())
    }

    fn clear_storage(&mut self) -> Result<(), InfoStorageError> {
        self.queue.clear();
        Ok(())
    }

    fn clear_range(&mut self, start_time: u32, end_time: u32) -> Result<(), InfoStorageError> {
        // 由于循环队列的特性，我们需要重建队列
        let mut new_queue = CircularQueue::new();
        
        for info in self.queue.iter() {
            let time = info.get_unix_time();
            // 保留不在删除范围内的数据
            if time < start_time || time > end_time {
                new_queue.push(*info).ok();
            }
        }
        
        self.queue = new_queue;
        Ok(())
    }
}

/// NVS 持久化存储（待实现）
pub struct NvsInfoStorage {

}
