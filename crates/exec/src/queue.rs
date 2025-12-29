use crate::event::Event;
use std::collections::{BinaryHeap, VecDeque};

// A deterministic event queue.
pub trait EventQueue {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn push(&mut self, event: Event);
    fn pop(&mut self) -> Option<Event>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

// A simple FIFO queue for events with equal times.
pub struct FifoEventQueue {
    queue: VecDeque<Event>,
}

impl EventQueue for FifoEventQueue {
    fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self { queue: VecDeque::with_capacity(capacity) }
    }

    fn push(&mut self, event: Event) {
        self.queue.push_back(event);
    }

    fn pop(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

// A binary heap for events with mixed times.
pub struct BinaryHeapEventQueue {
    heap: BinaryHeap<Event>,
}

impl EventQueue for BinaryHeapEventQueue {
    fn new() -> Self {
        Self { heap: BinaryHeap::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self { heap: BinaryHeap::with_capacity(capacity) }
    }

    fn push(&mut self, event: Event) {
        self.heap.push(event);
    }

    fn pop(&mut self) -> Option<Event> {
        self.heap.pop()
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use core::ids::{NodeIx, EdgeIx};

    #[test]
    fn fifo_queue_test() {
        let mut queue = FifoEventQueue::new();
        assert!(queue.is_empty());

        let event1 = Event { time: 0, priority: 0, node: 0, edge: None };
        let event2 = Event { time: 0, priority: 1, node: 1, edge: Some(0) };

        queue.push(event1.clone());
        queue.push(event2.clone());

        assert_eq!(queue.len(), 2);
        assert!(!queue.is_empty());

        assert_eq!(queue.pop(), Some(event1));
        assert_eq!(queue.pop(), Some(event2));
        assert_eq!(queue.pop(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn binary_heap_queue_test() {
        let mut queue = BinaryHeapEventQueue::new();
        assert!(queue.is_empty());

        let event1 = Event { time: 1, priority: 0, node: 0, edge: None };
        let event2 = Event { time: 0, priority: 1, node: 1, edge: Some(0) };
        let event3 = Event { time: 0, priority: 0, node: 2, edge: None };

        queue.push(event1.clone());
        queue.push(event2.clone());
        queue.push(event3.clone());

        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());

        // Events should be popped in order of time, then priority.
        assert_eq!(queue.pop(), Some(event3));
        assert_eq!(queue.pop(), Some(event2));
        assert_eq!(queue.pop(), Some(event1));
        assert_eq!(queue.pop(), None);
        assert!(queue.is_empty());
    }
}

