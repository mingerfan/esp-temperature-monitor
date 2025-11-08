use super::info_def;
use anyhow::Result;
use flashdb_rs::{tsdb::TSDB};
use crate::peripherals::flash;
use crate::utils::calculate;
use embedded_io::Read;

pub struct TimeDB {
    db: Box<TSDB<flash::Flash>>,
}

impl TimeDB {
    pub fn new(name: &str, max_len: u32, reset_if_size_incompatible: bool) -> Result<Self> {
        let mut slots_size = size_of::<info_def::InfoSlot>();
        // slots_size向4的整数倍取整，如果是整数则+4
        if slots_size & 0b11 != 0 {
            slots_size = (slots_size & !0b11) + 4;
        } else {
            slots_size += 4;
        }
        let max_size = calculate::quick_align((max_len * slots_size as u32 * 6 / 5) as usize, 4096);
        log::info!(
            "创建时间序列数据库: slot_size={slots_size}, max_size={max_size}"
        );

        let should_reset =if let Ok(header) = flash::Flash::touch_header() {
            let cur = header.get_size() - header.get_sector_size(); // 减去一个扇区的大小
            if cur != max_size {
                log::warn!(
                    "Flash 分区大小不匹配 (当前: {cur}, 期望: {max_size})"
                );
                true
            } else {
                false
            }
        } else {
            log::warn!("无法读取 Flash 分区大小");
            true
        };
        let storage = flash::Flash::new(max_size, reset_if_size_incompatible && should_reset)?;
        
        let mut db = Box::new(TSDB::new(storage));
        db.set_name(name)?;
        db.init(slots_size)?;
        Ok(TimeDB { db })
    }

    pub fn insert(&mut self, timestamp: i64, value: &info_def::InfoSlot) -> Result<()> {
        let data = value.as_bytes();
        self.db.append_with_timestamp(timestamp, data)?;
        Ok(())
    }

    // pub fn get_by_time(&mut self, left: i64, right: i64) -> Vec<info_def::InfoSlot> {
    //     let mut result = Vec::new();
    //     self.db.tsdb_iter_by_time(left, right, |db, tsl| {
    //         let mut cur = db.open_read(tsl.clone());
    //         let mut buf = vec![0u8; size_of::<info_def::InfoSlot>()];
    //         if cur.read(buf.as_mut_slice()).is_ok() {
    //             let slot = info_def::InfoSlot::from_bytes(buf.as_slice());
    //             result.push(slot);
    //         } else {
    //             log::error!("迭代过程中读取时间槽数据失败: tsl={tsl:?}");
    //         }
    //         true
    //     });
    //     result
    // }

    pub fn latest(&mut self) -> Option<info_def::InfoSlot> {
        let mut tmp: Option<info_def::InfoSlot> = None;
        self.db.tsdb_iter(|db, tsl| {
            let mut cur = db.open_read(tsl.clone());
            let mut buf = vec![0u8; size_of::<info_def::InfoSlot>()];
            if cur.read(buf.as_mut_slice()).is_ok() {
                tmp = Some(info_def::InfoSlot::from_bytes(buf.as_slice()));
                return false;
            }
            false
        }, true);
        tmp
    }

    // pub fn delete_range(&mut self, left: i64, right: i64) -> Result<()> {
    //     self.db.tsdb_iter_by_time(left, right, |db, tsl| {
    //         if let Err(e) = db.set_status(tsl, flashdb_rs::TSLStatus::Deleted) {
    //             log::error!("删除时间槽失败: {e:?}");
    //         }
    //         true
    //     });
    //     Ok(())
    // }

    // pub fn clear(&mut self) -> Result<()> {
    //     self.db.tsdb_iter(|db, tsl| {
    //         if let Err(e) = db.set_status(tsl, flashdb_rs::TSLStatus::Deleted) {
    //             log::error!("清空时间槽失败: {e:?}");
    //         }
    //         true
    //     }, false);
    //     Ok(())
    // }

    pub fn get_all_data(&mut self) -> Vec<info_def::InfoSlot> {
        let mut result = Vec::new();
        self.db.tsdb_iter(|db, tsl| {
            let mut cur = db.open_read(tsl.clone());
            let mut buf = vec![0u8; size_of::<info_def::InfoSlot>()];
            if cur.read(buf.as_mut_slice()).is_ok() {
                let slot = info_def::InfoSlot::from_bytes(buf.as_slice());
                result.push(slot);
            } else {
                log::error!("迭代过程中读取时间槽数据失败: tsl={tsl:?}");
            }
            true
        }, false);
        result
    }

    // pub fn earliest(&mut self) -> Option<info_def::InfoSlot> {
    //     let mut tmp: Option<info_def::InfoSlot> = None;
    //     self.db.tsdb_iter(|db, tsl| {
    //         if let Ok(Some(data)) = db.get_value(tsl) {
    //             tmp = Some(info_def::InfoSlot::from_bytes(data.as_slice()));
    //         }
    //         false
    //     }, false);
    //     tmp
    // }

}


