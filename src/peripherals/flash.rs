use crate::utils::calculate::quick_align;
use embedded_storage::nor_flash::{ErrorType, NorFlashError};
use esp_idf_sys::esp;
use std::ffi::CStr;
use thiserror;

const FLASH_MAGIC: &CStr = c"CUSTOM_FLASH";

const fn count_magic_bytes() -> usize {
    FLASH_MAGIC.to_bytes().len()
}

const fn magic_as_bytes() -> [u8; count_magic_bytes()] {
    let bytes = FLASH_MAGIC.to_bytes();
    let mut arr = [0u8; count_magic_bytes()];
    let mut i = 0;
    while i < count_magic_bytes() {
        arr[i] = bytes[i];
        i += 1;
    }
    arr
}

#[derive(thiserror::Error, Debug)]
pub enum FlashError {
    #[error("Not aligned! size: {0}, sector_size: {1}")]
    NotAligned(usize, usize),
    #[error("Out of bounds access at address {0}, size {1}, flash size {2}")]
    OutOfBounds(usize, usize, usize),
    #[error("Invalid flash header magic")]
    InvalidHeaderMagic,
    #[error("Raw pointer cast failed in 0x{0:x}")]
    PointerCastFailed(usize),
    #[error("Unknown error: {0}")]
    Unknown(i32),
}

impl NorFlashError for FlashError {
    fn kind(&self) -> embedded_storage::nor_flash::NorFlashErrorKind {
        match self {
            FlashError::NotAligned(_, _) => {
                embedded_storage::nor_flash::NorFlashErrorKind::NotAligned
            }
            FlashError::OutOfBounds(_, _, _) => {
                embedded_storage::nor_flash::NorFlashErrorKind::OutOfBounds
            }
            _ => embedded_storage::nor_flash::NorFlashErrorKind::Other,
        }
    }
}

impl ErrorType for Flash {
    type Error = FlashError;
}

#[repr(C, align(4))]
pub struct FlashHEADER {
    magic: [u8; count_magic_bytes()],
    size: usize,
    sector_size: usize,
}

impl FlashHEADER {
    pub fn new(size: usize, sector_size: usize) -> Self {
        FlashHEADER {
            magic: magic_as_bytes(),
            size,
            sector_size,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == magic_as_bytes()
    }

    unsafe fn from_raw(ptr: *const u8) -> Self {
        std::ptr::read_unaligned(ptr as *const FlashHEADER)
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_sector_size(&self) -> usize {
        self.sector_size
    }
}

const FLASH_HEADER_SIZE: usize = std::mem::size_of::<FlashHEADER>();
const FLASH_TYPE_CUSTOM: u32 = 0x40;

pub struct Flash {
    size: usize,
    sector_size: usize,
    partition: *const esp_idf_sys::esp_partition_t,
}

impl Flash {
    pub fn touch_header() -> Result<FlashHEADER, FlashError> {
        let partition = unsafe {
            esp_idf_sys::esp_partition_find_first(
                FLASH_TYPE_CUSTOM,
                esp_idf_sys::esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
                c"tsdb".as_ptr(),
            )
        };

        if partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let mut header_buf = [0u8; FLASH_HEADER_SIZE];

        let ret = unsafe {
            esp_idf_sys::esp_partition_read(
                partition,
                0,
                header_buf.as_mut_ptr() as *mut std::ffi::c_void,
                header_buf.len(),
            )
        };
        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        let header = unsafe { FlashHEADER::from_raw(header_buf.as_ptr()) };

        if !header.is_valid() {
            return Err(FlashError::InvalidHeaderMagic);
        }

        Ok(header)
    }

    pub fn new(size: usize, reset: bool) -> Result<Self, FlashError> {
        if size == 0 {
            return Err(FlashError::Unknown(-1));
        }
        log::info!("HEADER SIZE: {FLASH_HEADER_SIZE}");

        let partition = unsafe {
            esp_idf_sys::esp_partition_find_first(
                FLASH_TYPE_CUSTOM,
                esp_idf_sys::esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
                c"tsdb".as_ptr(),
            )
        };

        if partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let sector_size = unsafe {
            match partition.as_ref() {
                Some(p) => p.erase_size,
                None => return Err(FlashError::PointerCastFailed(partition as usize)),
            }
        } as usize;

        log::info!("partition sector size: {sector_size}");

        let mut header_buf = [0u8; FLASH_HEADER_SIZE];

        let ret = unsafe {
            esp_idf_sys::esp_partition_read(
                partition,
                0,
                header_buf.as_mut_ptr() as *mut std::ffi::c_void,
                header_buf.len(),
            )
        };
        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        let header = unsafe { FlashHEADER::from_raw(header_buf.as_ptr()) };

        if !header.is_valid() || reset {
            let last = size + sector_size;
            let size = quick_align(size + sector_size, sector_size);
            if last != size {
                log::warn!(
                    "Requested flash size {last} is not aligned to sector size {sector_size}, aligned to {size}"
                );
            }
            if !header.is_valid() {
                // 执行初始化操作
                log::warn!("Flash header is invalid, resetting partition");
            }
            return Flash::reset(size, partition);
        }

        // 如果是valid的，则直接读取size进行返回
        log::info!("Flash partition found with size: {}", header.size);
        let size = header.size;

        Ok(Flash { size, sector_size, partition })
    }

    pub fn reset(
        size: usize,
        partition: *const esp_idf_sys::esp_partition_t,
    ) -> Result<Self, FlashError> {
        if size == 0 {
            return Err(FlashError::Unknown(-1));
        }

        if partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let sector_size = unsafe {
            match partition.as_ref() {
                Some(p) => p.erase_size,
                None => return Err(FlashError::PointerCastFailed(partition as usize)),
            }
        } as usize;

        // reset不执行向上取整，直接抛出错误
        if size % sector_size != 0 {
            return Err(FlashError::NotAligned(size, sector_size));
        }

        // 初始化 Flash 分区，写入 HEADER 等
        let header = FlashHEADER::new(size, sector_size);
        log::info!("header: magic: {:?}, size: {}, header_size: {}, actual size: {size}", header.magic, header.size, std::mem::size_of::<FlashHEADER>());

        let ret = unsafe { esp_idf_sys::esp_partition_erase_range(partition, 0, size) };

        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        let ret = unsafe {
            esp_idf_sys::esp_partition_write(
                partition,
                0,
                &header as *const FlashHEADER as *const std::ffi::c_void,
                FLASH_HEADER_SIZE,
            )
        };

        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        Ok(Flash { size, sector_size, partition })
    }

    pub fn flash_read(&self, offset: usize, buf: &mut [u8]) -> Result<(), FlashError> {
        // 先检查是否越界
        if self.sector_size + offset + buf.len() > self.size {
            return Err(FlashError::OutOfBounds(offset, buf.len(), self.size));
        }

        if self.partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let ret = unsafe {
            esp_idf_sys::esp_partition_read(
                self.partition,
                offset + self.sector_size,
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                buf.len(),
            )
        };

        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        Ok(())
    }

    pub fn flash_write(&self, offset: usize, buf: &[u8]) -> Result<(), FlashError> {
        // 先检查是否越界
        if self.sector_size + offset + buf.len() > self.size {
            return Err(FlashError::OutOfBounds(offset, buf.len(), self.size));
        }

        if self.partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let ret = unsafe {
            esp_idf_sys::esp_partition_write(
                self.partition,
                offset + self.sector_size,
                buf.as_ptr() as *const std::ffi::c_void,
                buf.len(),
            )
        };

        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        Ok(())
    }

    pub fn flash_erase(&self, offset: usize, len: usize) -> Result<(), FlashError> {
        // 先检查是否越界
        if self.sector_size + offset + len > self.size {
            return Err(FlashError::OutOfBounds(offset, len, self.size));
        }

        if self.partition.is_null() {
            return Err(FlashError::PointerCastFailed(0));
        }

        let ret = unsafe {
            esp_idf_sys::esp_partition_erase_range(self.partition, offset + self.sector_size, len)
        };

        esp!(ret).map_err(|_| FlashError::Unknown(ret))?;

        Ok(())
    }

    pub fn flash_capacity(&self) -> usize {
        self.size - self.sector_size
    }
}

impl embedded_storage::nor_flash::ReadNorFlash for Flash {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> std::result::Result<(), Self::Error> {
        self.flash_read(offset as usize, bytes)
    }

    fn capacity(&self) -> usize {
        self.flash_capacity()
    }
}

impl embedded_storage::nor_flash::NorFlash for Flash {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = 4096;

    fn erase(&mut self, from: u32, to: u32) -> std::result::Result<(), Self::Error> {
        self.flash_erase(from as usize, (to - from) as usize)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> std::result::Result<(), Self::Error> {
        self.flash_write(offset as usize, bytes)
    }
}
