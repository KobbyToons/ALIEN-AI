use std::collections::HashSet;
use std::sync::RwLock;

pub struct CancellationManager {
    cancelled_ids: RwLock<HashSet<String>>,
    global_cancelled: std::sync::atomic::AtomicBool,
}

impl CancellationManager {
    pub fn new() -> Self {
        Self {
            cancelled_ids: RwLock::new(HashSet::new()),
            global_cancelled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn cancel(&self, id: Option<String>) {
        self.global_cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(id) = id {
            if let Ok(mut set) = self.cancelled_ids.write() {
                set.insert(id);
            }
        }
    }

    pub fn reset_global(&self) {
        self.global_cancelled.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self, id: &str) -> bool {
        if self.global_cancelled.load(std::sync::atomic::Ordering::SeqCst) {
            return true;
        }
        if let Ok(set) = self.cancelled_ids.read() {
            set.contains(id)
        } else {
            false
        }
    }

    pub fn clear(&self, id: &str) {
        if let Ok(mut set) = self.cancelled_ids.write() {
            set.remove(id);
        }
    }
}
