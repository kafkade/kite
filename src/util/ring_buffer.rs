use std::collections::VecDeque;

/// A fixed-capacity ring buffer for storing time-series history.
///
/// When the buffer is full, pushing a new element evicts the oldest.
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

#[allow(dead_code)]
impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_full(&self) -> bool {
        self.data.len() == self.capacity
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn as_slice_pair(&self) -> (&[T], &[T]) {
        self.data.as_slices()
    }
}

#[allow(dead_code)]
impl<T: Clone> RingBuffer<T> {
    /// Returns all elements as a contiguous Vec (front to back).
    pub fn to_vec(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_iterate() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn evicts_oldest_when_full() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);
        assert_eq!(buf.to_vec(), vec![2, 3, 4]);
    }

    #[test]
    fn len_and_capacity() {
        let mut buf = RingBuffer::<i32>::new(5);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 5);
        assert!(buf.is_empty());
        assert!(!buf.is_full());

        for i in 0..5 {
            buf.push(i);
        }
        assert_eq!(buf.len(), 5);
        assert!(buf.is_full());
    }

    #[test]
    fn clear() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.clear();
        assert!(buf.is_empty());
    }
}
