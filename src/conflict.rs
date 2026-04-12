use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    LastWriterWins,
    TrustWeighted,
    QuorumVote,
    Reject,
}

#[derive(Debug, Clone)]
pub enum Resolution {
    Accepted { value: u8, vessel: VesselId },
    Rejected { reason: String },
}

#[derive(Debug, Clone)]
pub struct MemoryConflict {
    pub addr: u16,
    pub candidates: Vec<ConflictCandidate>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ConflictCandidate {
    pub vessel: VesselId,
    pub value: u8,
    pub version: u32,
    pub timestamp: u64,
}

pub fn detect_conflicts(fabric: &MemoryFabric) -> Vec<MemoryConflict> {
    // Detect addresses written by multiple vessels in recent write order
    let mut addr_writers: HashMap<u16, Vec<(VesselId, u8, u32, u64)>> = HashMap::new();

    // Look at the last N writes for conflicts (same addr, different writers)
    for &(addr, vessel, version) in &fabric.write_order {
        let cell = match fabric.cells.get(&addr) {
            Some(c) => c,
            None => continue,
        };
        let entry = addr_writers.entry(addr).or_default();
        // Only count unique vessel writes
        if !entry.iter().any(|(v, _, _, _)| *v == vessel) {
            entry.push((vessel, cell.value, version, cell.last_modified));
        }
    }

    let mut conflicts = Vec::new();
    for (addr, writers) in addr_writers {
        if writers.len() > 1 {
            conflicts.push(MemoryConflict {
                addr,
                candidates: writers
                    .into_iter()
                    .map(|(vessel, value, version, timestamp)| ConflictCandidate {
                        vessel,
                        value,
                        version,
                        timestamp,
                    })
                    .collect(),
                timestamp: 0,
            });
        }
    }

    conflicts
}

pub fn resolve_conflict(
    conflict: &MemoryConflict,
    strategy: ConflictResolution,
    trust_scores: &HashMap<VesselId, f32>,
) -> Resolution {
    match strategy {
        ConflictResolution::LastWriterWins => {
            let best = conflict
                .candidates
                .iter()
                .max_by_key(|c| c.timestamp)
                .unwrap();
            Resolution::Accepted {
                value: best.value,
                vessel: best.vessel,
            }
        }
        ConflictResolution::TrustWeighted => {
            let best = conflict
                .candidates
                .iter()
                .max_by(|a, b| {
                    let ta = trust_scores.get(&a.vessel).copied().unwrap_or(0.0);
                    let tb = trust_scores.get(&b.vessel).copied().unwrap_or(0.0);
                    ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            Resolution::Accepted {
                value: best.value,
                vessel: best.vessel,
            }
        }
        ConflictResolution::QuorumVote => {
            // Simplified: pick the value with most votes (candidates with same value)
            let mut value_counts: HashMap<u8, usize> = HashMap::new();
            let mut value_vessel: HashMap<u8, VesselId> = HashMap::new();
            for c in &conflict.candidates {
                *value_counts.entry(c.value).or_insert(0) += 1;
                value_vessel.entry(c.value).or_insert(c.vessel);
            }
            let (value, _) = value_counts.into_iter().max_by_key(|(_, count)| *count).unwrap();
            Resolution::Accepted {
                value,
                vessel: value_vessel[&value],
            }
        }
        ConflictResolution::Reject => Resolution::Rejected {
            reason: "Conflict rejected by policy".to_string(),
        },
    }
}
