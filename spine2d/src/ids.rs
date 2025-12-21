use std::sync::atomic::{AtomicU32, Ordering};

// Match upstream runtimes: IDs are process-global, monotonically increasing counters used for
// per-timeline property gating (eg. deform/sequence).
static NEXT_VERTEX_ATTACHMENT_ID: AtomicU32 = AtomicU32::new(0);
static NEXT_SEQUENCE_ID: AtomicU32 = AtomicU32::new(0);

pub(crate) fn next_vertex_attachment_id() -> u32 {
    NEXT_VERTEX_ATTACHMENT_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn next_sequence_id() -> u32 {
    NEXT_SEQUENCE_ID.fetch_add(1, Ordering::Relaxed)
}
