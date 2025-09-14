//! Snapshot functionality for consistent reads

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A snapshot provides a consistent read view of the database
/// at a specific sequence number
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Sequence number at which this snapshot was taken
    sequence_number: u64,
    /// Unique snapshot ID
    id: u64,
}

static SNAPSHOT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl Snapshot {
    /// Create a new snapshot
    pub fn new(sequence_number: u64) -> Self {
        let id = SNAPSHOT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            sequence_number,
            id,
        }
    }

    /// Get the sequence number for this snapshot
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Get the unique ID for this snapshot
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Check if a sequence number is visible in this snapshot
    pub fn is_visible(&self, seq_num: u64) -> bool {
        seq_num <= self.sequence_number
    }
}

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Snapshot {}

impl std::hash::Hash for Snapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Snapshot list manages all active snapshots
pub struct SnapshotList {
    /// List of active snapshots sorted by sequence number
    snapshots: std::sync::RwLock<Vec<Arc<Snapshot>>>,
}

impl SnapshotList {
    /// Create a new snapshot list
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Add a snapshot to the list
    pub fn add(&self, snapshot: Arc<Snapshot>) {
        let mut snapshots = self.snapshots.write().unwrap();

        // Insert in sorted order by sequence number
        let pos = snapshots
            .binary_search_by_key(&snapshot.sequence_number(), |s| s.sequence_number())
            .unwrap_or_else(|pos| pos);

        snapshots.insert(pos, snapshot);
    }

    /// Remove a snapshot from the list
    pub fn remove(&self, snapshot: &Snapshot) -> bool {
        let mut snapshots = self.snapshots.write().unwrap();

        if let Some(pos) = snapshots.iter().position(|s| s.id() == snapshot.id()) {
            snapshots.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the oldest snapshot sequence number
    pub fn oldest(&self) -> Option<u64> {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.first().map(|s| s.sequence_number())
    }

    /// Get all snapshot sequence numbers
    pub fn all(&self) -> Vec<u64> {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.iter().map(|s| s.sequence_number()).collect()
    }

    /// Check if any snapshots exist
    pub fn is_empty(&self) -> bool {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.is_empty()
    }

    /// Get the count of active snapshots
    pub fn count(&self) -> usize {
        let snapshots = self.snapshots.read().unwrap();
        snapshots.len()
    }

    /// Clear all snapshots
    pub fn clear(&self) {
        let mut snapshots = self.snapshots.write().unwrap();
        snapshots.clear();
    }
}

impl Default for SnapshotList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new(100);
        assert_eq!(snapshot.sequence_number(), 100);
        assert!(snapshot.id() > 0);
        assert!(snapshot.is_visible(50));
        assert!(snapshot.is_visible(100));
        assert!(!snapshot.is_visible(150));
    }

    #[test]
    fn test_snapshot_equality() {
        let snapshot1 = Snapshot::new(100);
        let snapshot2 = Snapshot::new(100);
        let snapshot3 = snapshot1.clone();

        assert_ne!(snapshot1, snapshot2); // Different IDs
        assert_eq!(snapshot1, snapshot3); // Same ID
    }

    #[test]
    fn test_snapshot_list() {
        let list = SnapshotList::new();
        assert!(list.is_empty());
        assert_eq!(list.count(), 0);
        assert!(list.oldest().is_none());

        // Add snapshots
        let snap1 = Arc::new(Snapshot::new(50));
        let snap2 = Arc::new(Snapshot::new(100));
        let snap3 = Arc::new(Snapshot::new(75));

        list.add(snap1.clone());
        list.add(snap2.clone());
        list.add(snap3.clone());

        assert!(!list.is_empty());
        assert_eq!(list.count(), 3);
        assert_eq!(list.oldest(), Some(50));

        // Check order
        let all_seq_nums = list.all();
        assert_eq!(all_seq_nums, vec![50, 75, 100]);

        // Remove snapshot
        assert!(list.remove(&snap3));
        assert_eq!(list.count(), 2);
        assert!(!list.remove(&snap3)); // Already removed

        // Clear all
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn test_snapshot_visibility() {
        let snapshot = Snapshot::new(100);

        assert!(snapshot.is_visible(1));
        assert!(snapshot.is_visible(50));
        assert!(snapshot.is_visible(100));
        assert!(!snapshot.is_visible(101));
        assert!(!snapshot.is_visible(200));
    }

    #[test]
    fn test_snapshot_unique_ids() {
        let snap1 = Snapshot::new(100);
        let snap2 = Snapshot::new(100);

        assert_ne!(snap1.id(), snap2.id());
        assert_ne!(snap1, snap2);
    }
}