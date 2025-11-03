/// 循环队列实现，使用 Vec 动态分配
///
/// # 特性
/// - 使用 Vec 动态分配内存，避免栈溢出
/// - 提供 push/pop 操作
/// - 提供非破坏性的迭代器
/// - 线程安全（需要外部同步）
#[derive(Debug)]
pub struct CircularQueue<T, const N: usize> {
    buffer: Vec<Option<T>>, // 使用 Vec 存储元素
    capacity: usize,        // 队列容量
    head: usize,            // 队头位置（出队）
    tail: usize,            // 队尾位置（入队）
}

#[allow(unused)]
impl<T, const N: usize> CircularQueue<T, N> {
    /// 创建一个新的空循环队列
    ///
    /// 使用 Vec 预分配容量，避免栈溢出
    pub fn new() -> Self {
        let mut buffer = Vec::with_capacity(N);
        buffer.resize_with(N, || None);
        Self {
            buffer,
            capacity: N,
            head: 0,
            tail: 0,
        }
    }

    /// 返回队列的容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 返回队列当前的元素数量
    #[inline]
    pub fn len(&self) -> usize {
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.capacity - self.head + self.tail
        }
    }

    /// 检查队列是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail && self.buffer[self.head].is_none()
    }

    /// 检查队列是否已满
    #[inline]
    pub fn is_full(&self) -> bool {
        let next_tail = (self.tail + 1) % self.capacity;
        next_tail == self.head && self.buffer[self.head].is_some()
    }

    /// 向队尾添加元素
    ///
    /// # 返回值
    /// - `Ok(())` - 成功添加元素
    /// - `Err(value)` - 队列已满，返回原值
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        self.buffer[self.tail] = Some(value);
        self.tail = (self.tail + 1) % self.capacity;
        Ok(())
    }

    /// 强制向队尾添加元素，如果队列已满则覆盖最旧的元素
    ///
    /// # 返回值
    /// - `None` - 队列未满，直接添加
    /// - `Some(old_value)` - 队列已满，返回被覆盖的旧值
    pub fn push_overwrite(&mut self, value: T) -> Option<T> {
        if self.is_full() {
            let old = self.buffer[self.head].take();
            self.head = (self.head + 1) % self.capacity;
            self.buffer[self.tail] = Some(value);
            self.tail = (self.tail + 1) % self.capacity;
            old
        } else {
            self.buffer[self.tail] = Some(value);
            self.tail = (self.tail + 1) % self.capacity;
            None
        }
    }

    /// 从队头移除并返回元素
    ///
    /// # 返回值
    /// - `Some(value)` - 成功移除并返回元素
    /// - `None` - 队列为空
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        value
    }

    /// 查看队头元素但不移除
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.head].as_ref()
        }
    }

    /// 查看队头元素的可变引用但不移除
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.head].as_mut()
        }
    }

    /// 清空队列
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// 返回一个从队头到队尾的迭代器（不改变队列）
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            queue: self,
            index: 0,
        }
    }
}

impl<T: Clone, const N: usize> CircularQueue<T, N> {
    /// 返回指定索引位置的元素引用（0 表示队头）
    pub fn get(&self, index: usize) -> Option<&T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        let actual_index = (self.head + index) % self.capacity;
        self.buffer[actual_index].as_ref()
    }
}

impl<T, const N: usize> Default for CircularQueue<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 循环队列的不可变迭代器
///
/// 从队头到队尾顺序迭代，不会修改队列
pub struct Iter<'a, T, const N: usize> {
    queue: &'a CircularQueue<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let queue_len = self.queue.len();
        if self.index >= queue_len {
            return None;
        }

        let actual_index = (self.queue.head + self.index) % self.queue.capacity;
        self.index += 1;
        self.queue.buffer[actual_index].as_ref()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let queue_len = self.queue.len();
        let remaining = queue_len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for Iter<'a, T, N> {
    fn len(&self) -> usize {
        let queue_len = self.queue.len();
        queue_len.saturating_sub(self.index)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic_operations() {
        let mut queue: CircularQueue<i32, 4> = CircularQueue::new();

        assert_eq!(queue.capacity(), 4);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert!(!queue.is_full());

        // 测试 push
        assert!(queue.push(1).is_ok());
        assert!(queue.push(2).is_ok());
        assert!(queue.push(3).is_ok());
        assert_eq!(queue.len(), 3);

        // 测试 peek
        assert_eq!(queue.peek(), Some(&1));
        assert_eq!(queue.len(), 3); // peek 不改变长度

        // 测试 pop
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.len(), 1);

        assert_eq!(queue.pop(), Some(3));
        assert!(queue.is_empty());
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_full_queue() {
        let mut queue: CircularQueue<i32, 3> = CircularQueue::new();

        assert!(queue.push(1).is_ok());
        assert!(queue.push(2).is_ok());
        assert!(queue.push(3).is_ok());
        assert!(queue.is_full());

        // 队列已满，push 失败
        assert_eq!(queue.push(4), Err(4));
    }

    #[test]
    fn test_push_overwrite() {
        let mut queue: CircularQueue<i32, 3> = CircularQueue::new();

        assert_eq!(queue.push_overwrite(1), None);
        assert_eq!(queue.push_overwrite(2), None);
        assert_eq!(queue.push_overwrite(3), None);

        // 队列已满，覆盖最旧的元素
        assert_eq!(queue.push_overwrite(4), Some(1));
        assert_eq!(queue.push_overwrite(5), Some(2));

        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(4));
        assert_eq!(queue.pop(), Some(5));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_circular_behavior() {
        let mut queue: CircularQueue<i32, 3> = CircularQueue::new();

        // 填满队列
        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        // 移除一些元素
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));

        // 再添加元素（测试循环）
        queue.push(4).unwrap();
        queue.push(5).unwrap();

        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(4));
        assert_eq!(queue.pop(), Some(5));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_iterator() {
        let mut queue: CircularQueue<i32, 5> = CircularQueue::new();

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        let values: Vec<&i32> = queue.iter().collect();
        assert_eq!(values, vec![&1, &2, &3]);

        // 迭代后队列不变
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(1));
    }

    #[test]
    fn test_iterator_after_wrap() {
        let mut queue: CircularQueue<i32, 3> = CircularQueue::new();

        // 填满并部分移除
        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();
        queue.pop();
        queue.pop();

        // 再添加（发生循环）
        queue.push(4).unwrap();
        queue.push(5).unwrap();

        let values: Vec<&i32> = queue.iter().collect();
        assert_eq!(values, vec![&3, &4, &5]);
    }

    #[test]
    fn test_clear() {
        let mut queue: CircularQueue<i32, 4> = CircularQueue::new();

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_get() {
        let mut queue: CircularQueue<i32, 4> = CircularQueue::new();

        queue.push(10).unwrap();
        queue.push(20).unwrap();
        queue.push(30).unwrap();

        assert_eq!(queue.get(0), Some(&10));
        assert_eq!(queue.get(1), Some(&20));
        assert_eq!(queue.get(2), Some(&30));
        assert_eq!(queue.get(3), None);
    }

    #[test]
    fn test_exact_size_iterator() {
        let mut queue: CircularQueue<i32, 5> = CircularQueue::new();

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        let mut iter = queue.iter();
        assert_eq!(iter.len(), 3);

        iter.next();
        assert_eq!(iter.len(), 2);

        iter.next();
        assert_eq!(iter.len(), 1);

        iter.next();
        assert_eq!(iter.len(), 0);
    }
}
