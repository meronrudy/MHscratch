// use bridge::cache::GateCache;
// use core::ids::Epoch;

// #[test]
// fn test_cache_invalidation() {
//     let mut cache = GateCache::default();
//     assert!(!cache.is_valid(0, 0));
// 
//     cache.update(vec![1], vec![0.1], 0, 0);
//     assert!(cache.is_valid(0, 0));
//     assert!(!cache.is_valid(1, 0));
//     assert!(!cache.is_valid(0, 1));
//     assert!(!cache.is_valid(1, 1));
// 
//     cache.update(vec![0], vec![0.2], 1, 1);
//     assert!(cache.is_valid(1, 1));
//     assert_eq!(cache.edge_active[0], 0);
//     assert_eq!(cache.edge_weight[0], 0.2);
// }
