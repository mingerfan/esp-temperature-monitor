// use rand_pcg::Pcg64;  // 或 rand_chacha::ChaCha20Rng
// use rand_core::{RngCore, SeedableRng};
// use std::time::UNIX_EPOCH;

// pub struct RandomGenerator {
//     rng: Pcg64,
// }

// impl RandomGenerator {
//     pub fn new() -> Self {
//         let rng = init_rng();
//         RandomGenerator { rng }
//     }

//     pub fn next_u32(&mut self) -> u32 {
//         self.rng.next_u32()
//     }

//     // pub fn next_u64(&mut self) -> u64 {
//     //     self.rng.next_u64()
//     // }

//     // pub fn fill_bytes(&mut self, dest: &mut [u8]) {
//     //     self.rng.fill_bytes(dest);
//     // }
// }

// // 初始化 RNG（使用固定种子，或从硬件获取）
// pub fn init_rng() -> Pcg64 {
//     // 示例：使用固定种子（生产中替换为动态种子，如 RTC 时间）
//     let time = std::time::SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs();
//     let seed = [time as u8; 32];  // 32 字节种子
//     Pcg64::from_seed(seed)
// }

// // // 生成随机数
// // pub fn generate_random_u32(rng: &mut Pcg64) -> u32 {
// //     rng.next_u32()
// // }

// // pub fn generate_random_u64(rng: &mut Pcg64) -> u64 {
// //     rng.next_u64()
// // }

// // // 示例：生成随机字节数组
// // pub fn generate_random_bytes(rng: &mut Pcg64, dest: &mut [u8]) {
// //     rng.fill_bytes(dest);
// // }