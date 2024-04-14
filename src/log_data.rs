use crate::reference::Reference;
use std::sync::{Arc, Mutex};

// Maximum size UDP Datagram guaranteed to be delivered as a single packet.
// UDP Header = 8 bytes.
// IP Header Max = 60 bytes.
// Minimum allowed Network buffer reassembly size limit = 576 bytes.
// 576 - 60 - 8 = 508
pub type BufferLine = Mutex<[u8; 508]>;

#[derive(Debug)]
pub struct Logdata<const N: usize> {
    pub data: Arc<[BufferLine; N]>,
    pub reference: Arc<Reference<N>>,
}

impl<const N: usize> Clone for Logdata<N> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            reference: Arc::clone(&self.reference),
        }
    }
}
#[allow(dead_code)]
impl<const N: usize> Logdata<N> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(core::array::from_fn(|_| Mutex::new([0u8; 508]))),
            reference: Arc::new(Reference::<N>::new(0)),
        }
    }
}
