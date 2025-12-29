#[cfg(test)]
mod tests {
    use exec::queue::EventQueue;

    // Deterministic shuffle without adding rand
    fn xorshift64(mut x: u64) -> impl FnMut() -> u64 {
        move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
    }

    #[test]
    fn event_order_is_total_and_stable() {
        let mut q = EventQueue::with_capacity(256);

        // Create a set of events
        let mut events = Vec::new();
        for i in 0..100u32 {
            let time = (i % 7) as u64;
            let prio = (i % 5) as u32;
            let node = (i % 11) as u32;
            let edge = if i % 3 == 0 { Some(i % 17) } else { None };
            events.push((time, prio, node, edge));
        }

        // Push in a scrambled order
        let mut rng = xorshift64(12345);
        for _ in 0..events.len() {
            let j = (rng() as usize) % events.len();
            let (t,p,n,e) = events[j];
            q.push(t,p,n,e);
        }

        // Pop all; ensure monotone by key
        let mut last = None;
        while let Some(ev) = q.pop() {
            if let Some(prev) = last {
                assert!(prev <= ev.key, "non-monotone order: {:?} then {:?}", prev, ev.key);
            }
            last = Some(ev.key);
        }
    }

    #[test]
    fn duplicates_preserve_enqueue_order() {
        let mut q = EventQueue::with_capacity(16);

        // Same (time,prio,node,edge) repeated; seq must enforce FIFO
        for _ in 0..10 {
            q.push(5, 1, 7, Some(3));
        }

        let mut seqs = Vec::new();
        while let Some(ev) = q.pop() {
            seqs.push(ev.key.seq);
        }

        // Must be strictly increasing
        for w in seqs.windows(2) {
            assert!(w[0] < w[1]);
        }
    }
}
