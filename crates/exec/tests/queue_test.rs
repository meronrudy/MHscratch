use exec::event::Event;
use exec::queue::{EventQueue, FifoEventQueue, BinaryHeapEventQueue};
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
