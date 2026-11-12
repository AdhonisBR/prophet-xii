use prophet_standalone::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::{atomic::AtomicU64, Arc},
};
struct Counter;
thread_local! {static ENABLED:Cell<bool>=const{Cell::new(false)};static COUNT:Cell<usize>=const{Cell::new(0)};}
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ENABLED.with(|v| {
            if v.get() {
                COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ENABLED.with(|v| {
            if v.get() {
                COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static ALLOC: Counter = Counter;
#[test]
fn native_callback_has_no_allocations_including_control_and_telemetry() {
    let (mut tx, rx) = rtrb::RingBuffer::new(512);
    let (tt, mut tr) = rtrb::RingBuffer::new(4);
    let mut audio = AudioProcessor::new(48000.0, rx, tt, Arc::new(AtomicU64::new(0)));
    let mut out = [0.0f32; 512];
    for n in 48..60 {
        tx.push(Command::Midi {
            status: 0x90,
            data1: n,
            data2: 100,
        })
        .unwrap();
    }
    COUNT.with(|c| c.set(0));
    ENABLED.with(|v| v.set(true));
    for i in 0..200 {
        if i == 50 {
            tx.push(Command::Preset(1)).unwrap();
        }
        if i == 100 {
            tx.push(Command::Param { id: 63, value: 4.0 }).unwrap();
        }
        audio.process(&mut out, 2);
        while tr.pop().is_ok() {}
    }
    ENABLED.with(|v| v.set(false));
    assert_eq!(COUNT.with(|v| v.get()), 0);
    assert!(out.iter().all(|v| v.is_finite()));
}
