use crate::info::info_def::InfoSlot;
use crate::utils::circular_queue::CircularQueue;
use esp_idf_svc::{
    nvs::{self, NvsDefault},
    sys::EspError,
};
use thiserror::Error;
use esp_storage;

#[derive(Debug, Error)]
pub enum InfoStorageError {
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

    fn len(&self) -> usize;

    fn enqueue_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError>;

    fn dequeue_info(&mut self) -> Result<InfoSlot, InfoStorageError>;

    fn find_range(
        &self,
        start_time: u32,
        end_time: u32,
    ) -> Result<impl Iterator<Item = &InfoSlot>, InfoStorageError>;

    fn clear_storage(&mut self) -> Result<(), InfoStorageError>;

    fn clear_range(&mut self, start_time: u32, end_time: u32) -> Result<(), InfoStorageError>;
}

trait InforPersistence {
    fn persist_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError>;

    fn load_info(&self, timestamp: u32) -> Result<InfoSlot, InfoStorageError>;

    fn erease_info(&mut self, timestamp: u32) -> Result<(), InfoStorageError>;

    fn persist_all(
        &mut self,
        infos: impl Iterator<Item = InfoSlot>,
    ) -> Result<(), InfoStorageError>;

    fn load_all(&self) -> Result<impl Iterator<Item = InfoSlot>, InfoStorageError>;

    fn clear_storage(&mut self) -> Result<(), InfoStorageError>;
}

/// NVS 持久化存储（待实现）
pub struct NvsInfoStorage {
    nvs: nvs::EspNvs<NvsDefault>,
    queue: CircularQueue<InfoSlot, 4096>,
}

impl NvsInfoStorage {
    const NVS_NAMESPACE: &'static str = "info_storage";
    pub fn new() -> Result<Self, InfoStorageError> {
        let queue = CircularQueue::new();
        log::info!("Circular queue created successfully.");
        let partition = nvs::EspNvsPartition::<NvsDefault>::take()
            .map_err(|_| InfoStorageError::InitializationError)?;
        log::info!("NVS partition taken successfully.");
        let nvs = nvs::EspNvs::new(partition, Self::NVS_NAMESPACE, true)
            .map_err(|_| InfoStorageError::InitializationError)?;
        log::info!("NVS initialized successfully.");
        Ok(Self { nvs, queue })
    }

    pub fn nvs_write_u8(&mut self, key: &str, value: u8) -> Result<(), InfoStorageError> {
        self.nvs.set_u8(key, value).map_err(InfoStorageError::NvsError)
    }
    
    pub fn nvs_read_u8(&mut self, key: &str) -> Result<u8, InfoStorageError> {
        Ok(self.nvs.get_u8(key).map_err(InfoStorageError::NvsError)?.unwrap())
    }

    // fn nvs_get_all(&self) -> Result
}

impl InfoRW for NvsInfoStorage {

    fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn enqueue_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError> {
        if self.queue.is_full() {
            self.queue.pop();
        }
        self.queue.push(*info).map_err(|_| InfoStorageError::WriteError)
    }

    fn dequeue_info(&mut self) -> Result<InfoSlot, InfoStorageError> {
        self.queue.pop().ok_or(InfoStorageError::ReadError)
    }

    fn find_range(
            &self,
            start_time: u32,
            end_time: u32,
        ) -> Result<impl Iterator<Item = &InfoSlot>, InfoStorageError> {
        let infos = self.queue
            .iter()
            .filter(move |info| {
                let timestamp = info.get_unix_time();
                timestamp >= start_time && timestamp <= end_time
            });
        Ok(infos)
    }

    fn clear_storage(&mut self) -> Result<(), InfoStorageError> {
        self.queue.clear();
        Ok(())
    }

    fn clear_range(&mut self, start_time: u32, end_time: u32) -> Result<(), InfoStorageError> {
        let mut new_queue = CircularQueue::new();
        for info in self.queue.iter() {
            let timestamp = info.get_unix_time();
            if timestamp < start_time || timestamp > end_time {
                new_queue.push(*info).map_err(|_| InfoStorageError::WriteError)?;
            }
        }
        self.queue = new_queue;
        Ok(())
    }
}

// impl InforPersistence for NvsInfoStorage {
//     fn persist_info(&mut self, info: &InfoSlot) -> Result<(), InfoStorageError> {
//         let key = info.get_unix_time().to_string();
//     }
// }
