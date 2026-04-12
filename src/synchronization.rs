use crate::*;

#[derive(Debug, Clone)]
pub struct Fence {
    pub waiting_vessels: Vec<VesselId>,
    pub condition: FenceCondition,
    pub resolved: bool,
}

#[derive(Debug, Clone)]
pub enum FenceCondition {
    AllArrived(Vec<VesselId>),
    ValueEquals(u16, u8),
    ConfidenceAbove(f32),
    Quorum(usize),
}

pub fn fence_create(fabric: &mut MemoryFabric, condition: FenceCondition) -> FenceId {
    let id = fabric.next_fence_id;
    fabric.next_fence_id += 1;

    let waiting = match &condition {
        FenceCondition::AllArrived(vessels) => vessels.clone(),
        _ => vec![],
    };

    fabric.fences.insert(
        id,
        Fence {
            waiting_vessels: waiting.clone(),
            condition,
            resolved: false,
        },
    );

    id
}

pub fn fence_signal(fabric: &mut MemoryFabric, vessel_id: VesselId, fence_id: FenceId) {
    if let Some(fence) = fabric.fences.get_mut(&fence_id) {
        if fence.resolved {
            return;
        }
        fence.waiting_vessels.retain(|v| *v != vessel_id);
    }
}

pub fn fence_wait(fabric: &mut MemoryFabric, vessel_id: VesselId, fence_id: FenceId) -> bool {
    let fence = match fabric.fences.get_mut(&fence_id) {
        Some(f) => f,
        None => return false,
    };

    if fence.resolved {
        return true;
    }

    let met = match &fence.condition {
        FenceCondition::AllArrived(vessels) => {
            fence.waiting_vessels.is_empty()
        }
        FenceCondition::ValueEquals(addr, expected) => {
            fabric.cells.get(addr).map(|c| c.value == *expected).unwrap_or(false)
        }
        FenceCondition::ConfidenceAbove(threshold) => {
            // Simplified: treat as always met for threshold > 0
            *threshold <= 0.0
        }
        FenceCondition::Quorum(count) => {
            let arrived = match &fence.condition {
                FenceCondition::AllArrived(vessels) => vessels.len() - fence.waiting_vessels.len(),
                _ => 0,
            };
            arrived >= *count
        }
    };

    if met {
        fence.resolved = true;
    }

    met
}

pub fn memory_barrier(_fabric: &mut MemoryFabric, _vessel_id: VesselId) {
    // In a real implementation this would flush write buffers and
    // ensure ordering. For this in-memory fabric, writes are immediately
    // visible, so this is a no-op but exists for the API contract.
}
