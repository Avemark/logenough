use parking_lot::{Mutex, MutexGuard};

pub struct Logline {
    pub bytes_read: usize,
    pub buffer: [u8; 508],
}

impl std::fmt::Display for Logline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(&self.buffer[..self.bytes_read]).fmt(f)
    }
}

pub struct LockedLogline {
    line: Mutex<Logline>,
}

impl std::fmt::Display for LockedLogline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.line.lock().fmt(f)
    }
}

impl LockedLogline {
    pub fn lock(&self) -> MutexGuard<'_, Logline> {
        self.line.lock()
    }

    pub fn new() -> Self {
        Self {
            line: Mutex::new(Logline {
                bytes_read: 0usize,
                buffer: [0u8; 508],
            }),
        }
    }
}
