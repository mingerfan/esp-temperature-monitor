# 循环队列 (Circular Queue)

一个适用于嵌入式 no_std 环境的循环队列实现。

## 特性

- ✅ **零堆分配**: 使用预分配的固定大小数组
- ✅ **no_std 兼容**: 不依赖标准库，适用于嵌入式环境
- ✅ **泛型实现**: 支持任意类型
- ✅ **编译时大小**: 使用 const 泛型参数指定容量
- ✅ **非破坏性迭代器**: 可以遍历队列而不改变其内容
- ✅ **覆盖模式**: 支持在队列满时自动覆盖最旧的元素
- ✅ **完整测试**: 包含全面的单元测试

## 基本用法

### 创建队列

```rust
use crate::utils::circular_queue::CircularQueue;

// 创建一个容量为 10 的循环队列
let mut queue: CircularQueue<i32, 10> = CircularQueue::new();
```

### 添加和移除元素

```rust
// 入队
queue.push(1).unwrap();
queue.push(2).unwrap();
queue.push(3).unwrap();

// 查看队头元素（不移除）
if let Some(value) = queue.peek() {
    println!("队头: {}", value);
}

// 出队
if let Some(value) = queue.pop() {
    println!("出队: {}", value);
}
```

### 覆盖模式

当队列满时，可以使用 `push_overwrite` 自动覆盖最旧的元素：

```rust
let mut queue: CircularQueue<i32, 3> = CircularQueue::new();

queue.push(1).unwrap();
queue.push(2).unwrap();
queue.push(3).unwrap();

// 队列已满，覆盖最旧的元素 1
let old = queue.push_overwrite(4);
assert_eq!(old, Some(1));
```

### 迭代器

循环队列提供非破坏性迭代器，可以遍历队列而不改变其内容：

```rust
let mut queue: CircularQueue<i32, 5> = CircularQueue::new();
queue.push(10).unwrap();
queue.push(20).unwrap();
queue.push(30).unwrap();

// 迭代器从队头到队尾遍历
for value in queue.iter() {
    println!("{}", value);  // 输出: 10, 20, 30
}

// 队列内容保持不变
assert_eq!(queue.len(), 3);
```

## API 参考

### 构造函数

- `new()`: 创建一个空的循环队列

### 查询方法

- `capacity()`: 返回队列容量
- `len()`: 返回当前元素数量
- `is_empty()`: 检查队列是否为空
- `is_full()`: 检查队列是否已满

### 修改方法

- `push(value)`: 向队尾添加元素，队列满时返回 `Err`
- `push_overwrite(value)`: 向队尾添加元素，队列满时覆盖最旧的元素
- `pop()`: 从队头移除并返回元素
- `clear()`: 清空队列

### 访问方法

- `peek()`: 查看队头元素但不移除
- `peek_mut()`: 获取队头元素的可变引用
- `get(index)`: 获取指定位置的元素（0 表示队头）
- `iter()`: 返回从队头到队尾的迭代器

## 嵌入式应用示例

### 温度数据缓冲

```rust
use crate::utils::circular_queue::CircularQueue;

#[derive(Clone, Copy)]
struct TempReading {
    timestamp: u32,
    temperature: f32,
}

// 存储最近 100 个温度读数
let mut temp_buffer: CircularQueue<TempReading, 100> = CircularQueue::new();

// 添加新读数（自动覆盖最旧的数据）
let reading = TempReading {
    timestamp: 1000,
    temperature: 25.3,
};
temp_buffer.push_overwrite(reading);

// 计算平均温度
let sum: f32 = temp_buffer.iter().map(|r| r.temperature).sum();
let avg = sum / temp_buffer.len() as f32;
```

### 传感器数据队列

```rust
use crate::info::info_def::InfoSlot;
use crate::utils::circular_queue::CircularQueue;

// 创建信息槽队列
let mut info_queue: CircularQueue<InfoSlot, 50> = CircularQueue::new();

// 添加传感器数据
let mut slot = InfoSlot::default();
slot.set_temperature(24.5);
slot.set_humidity(60.0);
slot.set_unix_time(1234567890);

info_queue.push(slot).ok();

// 批量处理
for info in info_queue.iter() {
    println!("温度: {:.1}°C, 湿度: {:.1}%", 
             info.get_temperature(), 
             info.get_humidity());
}
```

## 性能特点

- **时间复杂度**:
  - `push`: O(1)
  - `pop`: O(1)
  - `peek`: O(1)
  - `iter`: O(n)

- **空间复杂度**: O(N)，其中 N 是队列容量

## 线程安全

此实现本身不是线程安全的。在多线程环境中使用时，需要外部同步机制（如 `Mutex`）：

```rust
use std::sync::Mutex;

let queue = Mutex::new(CircularQueue::<i32, 10>::new());

// 在线程中使用
{
    let mut q = queue.lock().unwrap();
    q.push(42).ok();
}
```

## 测试

运行测试：

```bash
cargo test
```

查看测试覆盖：

```bash
cargo test -- --nocapture
```

## 与 heapless 的比较

如果你想使用现有的 crate，`heapless` 是一个很好的选择：

```toml
[dependencies]
heapless = "0.8"
```

使用 heapless:

```rust
use heapless::Deque;

let mut deque: Deque<i32, 10> = Deque::new();
deque.push_back(1).ok();
```

**本实现的优势**:
- 更简单，代码完全可控
- 提供了专门的 `push_overwrite` 覆盖模式
- 针对特定需求优化
- 无外部依赖

**heapless 的优势**:
- 成熟稳定，广泛使用
- 提供更多数据结构（Vec, String, HashMap 等）
- 经过充分测试和优化

## 许可证

与项目保持一致。
