use crate::*;
use std::collections::HashSet;

pub fn fabric_read(
    fabric: &MemoryFabric,
    addr: u16,
    reader_id: VesselId,
) -> Result<u8, FabricError> {
    let cell = fabric.cells.get(&addr).ok_or(FabricError::AddressNotMapped(addr))?;
    if !cell.permissions.can_read(reader_id) {
        return Err(FabricError::PermissionDenied {
            addr,
            vessel: reader_id,
            operation: "read",
        });
    }
    Ok(cell.value)
}

pub fn fabric_write(
    fabric: &mut MemoryFabric,
    addr: u16,
    value: u8,
    writer_id: VesselId,
) -> Result<(), FabricError> {
    let cell = fabric.cells.get_mut(&addr).ok_or(FabricError::AddressNotMapped(addr))?;
    if !cell.permissions.can_write(writer_id) {
        return Err(FabricError::PermissionDenied {
            addr,
            vessel: writer_id,
            operation: "write",
        });
    }
    cell.value = value;
    cell.version += 1;
    cell.last_modified += 1;
    fabric.write_order.push((addr, writer_id, cell.version));
    Ok(())
}

pub fn fabric_alloc_region(
    fabric: &mut MemoryFabric,
    size: u16,
    owner: VesselId,
    region_type: RegionType,
) -> u16 {
    // Find lowest available base address
    let used: Vec<(u16, u16)> = fabric.regions.iter().map(|r| (r.base, r.size)).collect();
    let mut base: u16 = 0;
    'outer: loop {
        for &(b, s) in &used {
            if base >= b && base < b + s {
                base = b + s;
                continue 'outer;
            }
        }
        break;
    }

    // Create cells
    let permissions = match region_type {
        RegionType::Private => CellPermissions::owner_only(owner),
        RegionType::Shared => CellPermissions::shared_readable(owner),
        RegionType::Broadcast | RegionType::Heap => CellPermissions::fully_shared(owner),
        _ => CellPermissions::owner_only(owner),
    };

    for i in 0..size {
        fabric.cells.insert(
            base + i,
            MemoryCell {
                value: 0,
                owner,
                permissions: permissions.clone(),
                last_modified: 0,
                version: 0,
            },
        );
    }

    fabric.regions.push(MemoryRegion {
        base,
        size,
        owner,
        fabric_type: region_type,
    });

    base
}

pub fn fabric_free_region(fabric: &mut MemoryFabric, base: u16) -> Result<(), FabricError> {
    let idx = fabric
        .regions
        .iter()
        .position(|r| r.base == base)
        .ok_or(FabricError::RegionNotFound(base))?;

    let region = fabric.regions.remove(idx);
    for i in 0..region.size {
        fabric.cells.remove(&(base + i));
    }
    Ok(())
}

pub fn fabric_subscribe(
    fabric: &mut MemoryFabric,
    vessel_id: VesselId,
    addr: u16,
    range: u16,
    callback_type: CallbackType,
) {
    let subs = fabric.subscriptions.entry(vessel_id).or_default();
    subs.push(Subscription { addr, range, callback_type });
}

pub fn fabric_broadcast_write(
    fabric: &mut MemoryFabric,
    addr: u16,
    value: u8,
    broadcaster_id: VesselId,
) -> Vec<Notification> {
    let old_value = fabric.cells.get(&addr).map(|c| c.value).unwrap_or(0);

    if fabric_write(fabric, addr, value, broadcaster_id).is_err() {
        return vec![];
    }

    let cell = fabric.cells.get(&addr).unwrap();
    let mut notifications = Vec::new();

    let mut notified: HashSet<VesselId> = HashSet::new();

    for (vid, subs) in &fabric.subscriptions {
        for sub in subs {
            let in_range = addr >= sub.addr && addr < sub.addr + sub.range;
            let triggered = match sub.callback_type {
                CallbackType::OnChange => in_range && old_value != value,
                CallbackType::OnWrite => in_range,
                CallbackType::OnThreshold(t) => in_range && value >= t,
            };
            if triggered && !notified.contains(vid) {
                notifications.push(Notification {
                    addr,
                    old_value,
                    new_value: value,
                    writer: broadcaster_id,
                    version: cell.version,
                });
                notified.insert(*vid);
            }
        }
    }

    notifications
}
