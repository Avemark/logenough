use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let interrupted = Arc::new(AtomicBool::new(true));

    let got_int = interrupted.clone();
    ctrlc::set_handler(move || {
        got_int.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler")
}
