use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

pub struct Task {
    // So we can have a way to specify which task should be woken
    id: TaskId,
    // Pinned, heap-allocated, dynamically dispatched future with empty type ()
    // () because only side effects matter, like printing to screen
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

// These traits are needed as key type of BTreeMap
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

use core::sync::atomic::{AtomicU64, Ordering};

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
