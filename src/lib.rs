pub mod operations;
pub mod synchronization;
pub mod conflict;

use std::collections::HashMap;

pub type VesselId = u32;
pub type FenceId = u64;

#[derive(Debug, Clone)]
pub struct CellPermissions {
    pub readable_by: Vec<VesselId>,
    pub writable_by: Vec<VesselId>,
    pub shared: bool,
}

impl CellPermissions {
    pub fn owner_only(owner: VesselId) -> Self {
        Self {
            readable_by: vec![owner],
            writable_by: vec![owner],
            shared: false,
        }
    }

    pub fn shared_readable(owner: VesselId) -> Self {
        Self {
            readable_by: vec![],
            writable_by: vec![owner],
            shared: true,
        }
    }

    pub fn fully_shared(owner: VesselId) -> Self {
        Self {
            readable_by: vec![],
            writable_by: vec![],
            shared: true,
        }
    }

    pub fn can_read(&self, vessel: VesselId) -> bool {
        self.shared || self.readable_by.contains(&vessel)
    }

    pub fn can_write(&self, vessel: VesselId) -> bool {
        self.shared || self.writable_by.contains(&vessel)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryCell {
    pub value: u8,
    pub owner: VesselId,
    pub permissions: CellPermissions,
    pub last_modified: u64,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Private,
    Shared,
    Broadcast,
    RingBuffer,
    Stack,
    Heap,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: u16,
    pub size: u16,
    pub owner: VesselId,
    pub fabric_type: RegionType,
}

impl MemoryRegion {
    pub fn contains(&self, addr: u16) -> bool {
        addr >= self.base && addr < self.base + self.size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallbackType {
    OnChange,
    OnWrite,
    OnThreshold(u8),
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub addr: u16,
    pub range: u16,
    pub callback_type: CallbackType,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub addr: u16,
    pub old_value: u8,
    pub new_value: u8,
    pub writer: VesselId,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct MemoryFabric {
    pub cells: HashMap<u16, MemoryCell>,
    pub regions: Vec<MemoryRegion>,
    pub subscriptions: HashMap<VesselId, Vec<Subscription>>,
    pub next_fence_id: FenceId,
    pub fences: HashMap<FenceId, synchronization::Fence>,
    pub conflict_log: Vec<conflict::MemoryConflict>,
    pub write_order: Vec<(u16, VesselId, u32)>,
}

impl MemoryFabric {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            regions: Vec::new(),
            subscriptions: HashMap::new(),
            next_fence_id: 0,
            fences: HashMap::new(),
            conflict_log: Vec::new(),
            write_order: Vec::new(),
        }
    }

    pub fn find_region(&self, addr: u16) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    pub fn find_region_mut(&mut self, addr: u16) -> Option<&mut MemoryRegion> {
        self.regions.iter_mut().find(|r| r.contains(addr))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FabricError {
    AddressNotMapped(u16),
    PermissionDenied { addr: u16, vessel: VesselId, operation: &'static str },
    RegionOverlap(u16, u16),
    RegionNotFound(u16),
    FenceNotFound(FenceId),
    InvalidAddress,
}
