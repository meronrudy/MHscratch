use std::collections::BinaryHeap;
use std::vec::Vec;
use std::cmp::Reverse;

use crate::event::{Event, EventKey};
use core::ids::{EdgeIx, NodeIx};

#[derive(Clone, Copy, Debug)]
struct HeapEntry {
    key: Reverse<EventKey>, // min-heap behavior
    slot: u32,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
            .then(self.slot.cmp(&other.slot)) // deterministic if keys equal
    }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.slot == other.slot
    }
}
impl Eq for HeapEntry {}

#[derive(Debug)]
struct Slot {
    event: Event,
    live: bool,
}

#[derive(Debug)]
pub struct EventQueue {
    slots: Vec<Slot>,
    free: Vec<u32>,           // free slot stack
    heap: BinaryHeap<HeapEntry>,
    next_seq: u64,
}

impl EventQueue {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            slots: Vec::with_capacity(cap),
            free: Vec::with_capacity(cap / 4 + 1),
            heap: BinaryHeap::with_capacity(cap),
            next_seq: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn alloc_slot(&mut self, event: Event) -> u32 {
        if let Some(ix) = self.free.pop() {
            let s = &mut self.slots[ix as usize];
            s.event = event;
            s.live = true;
            ix
        } else {
            let ix = self.slots.len() as u32;
            self.slots.push(Slot { event, live: true });
            ix
        }
    }

    pub fn push(&mut self, time: u64, priority: u32, node: NodeIx, edge: Option<EdgeIx>) {
        let seq = self.next_seq;
        self.next_seq += 1;

        let key = EventKey { time, priority, node, edge, seq };
        let event = Event { key };
        let slot = self.alloc_slot(event);

        self.heap.push(HeapEntry { key: Reverse(key), slot });
    }

    pub fn pop(&mut self) -> Option<Event> {
        while let Some(entry) = self.heap.pop() {
            let ix = entry.slot as usize;
            let slot = &mut self.slots[ix];
            if !slot.live {
                continue;
            }
            slot.live = false;
            self.free.push(entry.slot);
            return Some(slot.event);
        }
        None
    }
}
