use logenough::logdata::LogData;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

const LOG_LINE_COUNT: usize = 1000000;

fn main() {
    let mem_size_buffer = 30_000;
    let data_size = size_of::<Mutex<LogData<LOG_LINE_COUNT>>>();
    let from_fn_multiplier = if cfg!(debug_assertions) { 10 } else { 2 };

    let data = thread::Builder::new()
        .name("child thread".into())
        .stack_size(data_size * from_fn_multiplier + mem_size_buffer)
        .spawn(|| Arc::new(LogData::<LOG_LINE_COUNT>::new()))
        .unwrap()
        .join()
        .unwrap();

    println!("{:?}", data.data[0].lock())
}
