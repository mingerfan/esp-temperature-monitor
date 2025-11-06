use super::info_def;
use anyhow::Result;
use flashdb_rs::{tsdb::TSDB, StdStorage};

pub struct TimeDB {
    db: Box<TSDB<StdStorage>>,
}

impl TimeDB {
    pub fn new(path: &str, max_len: u32) -> Result<Self> {
        // 获取path最后的文件名作为数据库名称
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let mut slots_size = size_of::<info_def::InfoSlot>();
        // slots_size向4的整数倍取整，如果是整数则+4
        if slots_size & 0b11 != 0 {
            slots_size = (slots_size & !0b11) + 4;
        } else {
            slots_size += 4;
        }
        let max_size = (max_len * slots_size as u32 * 6 / 5).div_ceil(4096) * 4096;
        log::info!(
            "创建时间序列数据库: name={name}, path={path}, slot_size={slots_size}, max_size={max_size}"
        );
        // let db = TSDB::new_file(name, path, 4096, max_size, slots_size)?;
        let storage = StdStorage::new(
            path,
            name,
            4096,
            max_size,
            flashdb_rs::storage::FileStrategy::Single,
        )?;
        log::info!("TimeDB storage created at path: {path}");
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

    pub fn get_by_time(&mut self, left: i64, right: i64) -> Vec<info_def::InfoSlot> {
        let mut result = Vec::new();
        self.db.tsdb_iter_by_time(left, right, |db, tsl| {
            if let Ok(Some(data)) = db.get_value(tsl) {
                let slot = info_def::InfoSlot::from_bytes(data.as_slice());
                result.push(slot);
                return true;
            }
            false
        });
        result
    }

    pub fn latest(&mut self) -> Option<info_def::InfoSlot> {
        let mut tmp: Option<info_def::InfoSlot> = None;
        self.db.tsdb_iter(|db, tsl| {
            if let Ok(Some(data)) = db.get_value(tsl) {
                tmp = Some(info_def::InfoSlot::from_bytes(data.as_slice()));
            }
            false
        }, true);
        tmp
    }

    pub fn delete_range(&mut self, left: i64, right: i64) -> Result<()> {
        self.db.tsdb_iter_by_time(left, right, |db, tsl| {
            if let Err(e) = db.set_status(tsl, flashdb_rs::TSLStatus::Deleted) {
                log::error!("删除时间槽失败: {e:?}");
            }
            true
        });
        Ok(())
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
