use cuda_memory_fabric::*;
use cuda_memory_fabric::operations::*;
use cuda_memory_fabric::synchronization::*;
use cuda_memory_fabric::conflict::*;

// 1. Basic read/write
#[test]
fn test_basic_read_write() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Private);
    fabric_write(&mut fabric, base, 42, 1).unwrap();
    assert_eq!(fabric_read(&fabric, base, 1).unwrap(), 42);
}

// 2. Permission enforcement
#[test]
fn test_permission_enforcement() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Private);
    fabric_write(&mut fabric, base, 99, 1).unwrap();
    // Vessel 2 should not be able to read
    assert!(matches!(
        fabric_read(&fabric, base, 2),
        Err(FabricError::PermissionDenied { .. })
    ));
    // Vessel 2 should not be able to write
    assert!(matches!(
        fabric_write(&mut fabric, base, 10, 2),
        Err(FabricError::PermissionDenied { .. })
    ));
}

// 3. Region allocation
#[test]
fn test_region_allocation() {
    let mut fabric = MemoryFabric::new();
    let r1 = fabric_alloc_region(&mut fabric, 8, 1, RegionType::Private);
    let r2 = fabric_alloc_region(&mut fabric, 4, 2, RegionType::Shared);
    assert_ne!(r1, r2);
    assert_eq!(r2, r1 + 8);
}

// 4. Broadcast write
#[test]
fn test_broadcast_write() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Broadcast);
    fabric_subscribe(&mut fabric, 2, base, 4, CallbackType::OnWrite);
    let notifications = fabric_broadcast_write(&mut fabric, base, 77, 1);
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].new_value, 77);
}

// 5. Fence: AllArrived
#[test]
fn test_fence_all_arrived() {
    let mut fabric = MemoryFabric::new();
    let fid = fence_create(&mut fabric, FenceCondition::AllArrived(vec![1, 2]));
    assert!(!fence_wait(&mut fabric, 1, fid));
    fence_signal(&mut fabric, 1, fid);
    assert!(!fence_wait(&mut fabric, 2, fid));
    fence_signal(&mut fabric, 2, fid);
    assert!(fence_wait(&mut fabric, 2, fid));
}

// 6. Fence: ValueEquals
#[test]
fn test_fence_value_equals() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Private);
    let fid = fence_create(&mut fabric, FenceCondition::ValueEquals(base, 42));
    assert!(!fence_wait(&mut fabric, 1, fid));
    fabric_write(&mut fabric, base, 42, 1).unwrap();
    assert!(fence_wait(&mut fabric, 1, fid));
}

// 7. Memory barrier (no-op but API works)
#[test]
fn test_memory_barrier() {
    let mut fabric = MemoryFabric::new();
    memory_barrier(&mut fabric, 1);
}

// 8. Conflict detection
#[test]
fn test_conflict_detection() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Broadcast);
    fabric_write(&mut fabric, base, 10, 1).unwrap();
    fabric_write(&mut fabric, base, 20, 2).unwrap();
    let conflicts = detect_conflicts(&fabric);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].addr, base);
    assert_eq!(conflicts[0].candidates.len(), 2);
}

// 9. Subscription notifications
#[test]
fn test_subscription_on_change() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Shared);
    fabric_subscribe(&mut fabric, 2, base, 4, CallbackType::OnChange);
    // Same value — no notification
    let n = fabric_broadcast_write(&mut fabric, base, 0, 1);
    assert_eq!(n.len(), 0);
    // Different value — notification
    let n = fabric_broadcast_write(&mut fabric, base, 55, 1);
    assert_eq!(n.len(), 1);
}

// 10. Version tracking
#[test]
fn test_version_tracking() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Private);
    assert_eq!(fabric.cells.get(&base).unwrap().version, 0);
    fabric_write(&mut fabric, base, 1, 1).unwrap();
    assert_eq!(fabric.cells.get(&base).unwrap().version, 1);
    fabric_write(&mut fabric, base, 2, 1).unwrap();
    assert_eq!(fabric.cells.get(&base).unwrap().version, 2);
}

// 11. Concurrent writes (simulated)
#[test]
fn test_concurrent_writes() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Broadcast);
    fabric_write(&mut fabric, base, 10, 1).unwrap();
    fabric_write(&mut fabric, base, 20, 2).unwrap();
    fabric_write(&mut fabric, base, 30, 3).unwrap();
    assert_eq!(fabric_read(&fabric, base, 1).unwrap(), 30);
    assert_eq!(fabric.cells.get(&base).unwrap().version, 3);
}

// 12. Region free
#[test]
fn test_region_free() {
    let mut fabric = MemoryFabric::new();
    let base = fabric_alloc_region(&mut fabric, 4, 1, RegionType::Private);
    fabric_free_region(&mut fabric, base).unwrap();
    assert!(matches!(
        fabric_read(&fabric, base, 1),
        Err(FabricError::AddressNotMapped(_))
    ));
}
