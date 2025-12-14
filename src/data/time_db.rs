use super::info_def;
use anyhow::Result;
use flashdb_rs::{tsdb::TSDB};
use crate::peripherals::flash;
use crate::utils::calculate;
use embedded_io::Read;

pub struct TimeDB {
    db: Box<TSDB<flash::Flash>>,
    max_size: usize,
    slot_size: usize,
    /// 容量警戒线百分比 (0-100)，默认为 80%
    capacity_threshold: f32,
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
        Ok(TimeDB { 
            db,
            max_size,
            slot_size: slots_size,
            capacity_threshold: 80.0, // 默认 80% 触发清理
        })
    }

    pub fn insert(&mut self, timestamp: i64, value: &info_def::InfoSlot) -> Result<()> {
        // 检查容量，如果需要则清理最旧的数据
        self.cleanup_if_needed()?;
        
        let data = value.as_bytes();
        self.db.append_with_timestamp(timestamp, data)?;
        Ok(())
    }

    /// 计算当前数据库的使用大小（字节）
    fn get_current_size(&mut self) -> usize {
        let mut size = 0;
        self.db.tsdb_iter(|_db, _tsl| {
            size += self.slot_size;
            true
        }, false);
        size
    }

    /// 如果容量超过警戒线，标记最旧的数据块为删除
    /// 采用标记方式，实际删除由 flashdb_rs 异步处理
    fn cleanup_if_needed(&mut self) -> Result<()> {
        let current_size = self.get_current_size();
        let threshold_size = (self.max_size as f32 * self.capacity_threshold / 100.0) as usize;

        if current_size >= threshold_size {
            log::warn!(
                "数据库容量接近上限 (当前: {}B, 警戒线: {}B), 开始清理最旧的数据",
                current_size, threshold_size
            );
            
            // 标记最旧的 10% 的数据为删除
            let cleanup_size = (self.max_size as f32 * 0.1) as usize;
            let mut cleaned_size = 0;
            let mut cleanup_count = 0;

            self.db.tsdb_iter(|db, tsl| {
                if cleaned_size >= cleanup_size {
                    return false; // 停止迭代
                }
                
                match db.set_status(tsl, flashdb_rs::TSLStatus::Deleted) {
                    Ok(_) => {
                        cleaned_size += self.slot_size;
                        cleanup_count += 1;
                    }
                    Err(e) => {
                        log::error!("标记数据为删除失败: {e:?}");
                    }
                }
                true
            }, false); // false 表示从最旧的开始迭代

            log::info!(
                "已标记 {} 条记录为删除 (约 {}B)",
                cleanup_count, cleaned_size
            );
        }

        Ok(())
    }

    // 设置容量警戒线百分比
    // pub fn set_capacity_threshold(&mut self, threshold: f32) {
    //     self.capacity_threshold = threshold.max(1.0).min(100.0);
    // }

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

    // pub fn get_all_data(&mut self) -> Vec<info_def::InfoSlot> {
    //     let mut result = Vec::new();
    //     self.db.tsdb_iter(|db, tsl| {
    //         let mut cur = db.open_read(tsl.clone());
    //         let mut buf = vec![0u8; size_of::<info_def::InfoSlot>()];
    //         if cur.read(buf.as_mut_slice()).is_ok() {
    //             let slot = info_def::InfoSlot::from_bytes(buf.as_slice());
    //             result.push(slot);
    //         } else {
    //             log::error!("迭代过程中读取时间槽数据失败: tsl={tsl:?}");
    //         }
    //         true
    //     }, false);
    //     result
    // }

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


