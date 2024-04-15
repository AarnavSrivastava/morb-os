use alloc::alloc::{GlobalAlloc, Layout};
use lazy_static::lazy_static;
use spin::Mutex;
use core::{mem, ptr::{self, NonNull}};
use super::{Locked, HEAP_SIZE};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}