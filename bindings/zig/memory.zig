//! NVMS Zig Memory Allocator
//!
//! High-performance, no-hidden-allocation memory allocator
//! for use in performance-critical NVMS paths.
//!
//! # Features
//!
//! - No hidden allocations (unlike std.heap)
//! - C-compatible ABI for Go/Rust interop
//! - Memory pooling for reduced fragmentation
//! - Zero-overhead for common allocation patterns
//!
//! # Build
//!
//! ```bash
//! zig build
//! ```

const std = @import("std");

// Export configuration
pub const NVMS_ALLOCATOR_PAGE_SIZE = 4096;
pub const NVMS_ALLOCATOR_MAX_POOL_SIZE = 64 * 1024 * 1024; // 64MB

/// NVMS Memory Pool
pub const MemoryPool = struct {
    arena: std.heap.ArenaAllocator,
    total_allocated: u64,
    peak_allocated: u64,

    const Self = @This();

    /// Create a new memory pool
    pub fn init() Self {
        return Self{
            .arena = std.heap.ArenaAllocator.init(std.heap.page_allocator),
            .total_allocated = 0,
            .peak_allocated = 0,
        };
    }

    /// Allocate memory
    pub fn alloc(self: *Self, size: usize) ?[]u8 {
        const result = self.arena.allocator().alloc(u8, size) catch return null;
        self.total_allocated += size;
        if (self.total_allocated > self.peak_allocated) {
            self.peak_allocated = self.total_allocated;
        }
        return result;
    }

    /// Allocate aligned memory
    pub fn allocAligned(self: *Self, size: usize, alignment: usize) ?[*]u8 {
        const result = self.arena.allocator().alignedAlloc(u8, alignment, size) catch return null;
        self.total_allocated += size;
        if (self.total_allocated > self.peak_allocated) {
            self.peak_allocated = self.total_allocated;
        }
        return result.ptr;
    }

    /// Free all memory
    pub fn deinit(self: *Self) void {
        self.arena.deinit();
    }

    /// Get stats
    pub fn getStats(self: *const Self) Stats {
        return Stats{
            .total_allocated = self.total_allocated,
            .peak_allocated = self.peak_allocated,
        };
    }

    pub const Stats = struct {
        total_allocated: u64,
        peak_allocated: u64,
    };
};

/// Thread-safe memory pool with mutex
pub const ThreadSafePool = struct {
    inner: MemoryPool,
    mutex: std.Thread.Mutex,

    const Self = @This();

    pub fn init() Self {
        return Self{
            .inner = MemoryPool.init(),
            .mutex = std.Thread.Mutex{},
        };
    }

    pub fn alloc(self: *Self, size: usize) ?[]u8 {
        self.mutex.lock();
        defer self.mutex.unlock();
        return self.inner.alloc(size);
    }

    pub fn deinit(self: *Self) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        self.inner.deinit();
    }
};

// C-export functions for Go/Rust interop
export fn nvms_zig_alloc(size: usize) ?[*]u8 {
    const pool = std.heap.page_allocator;
    const result = pool.alloc(u8, size) catch return null;
    return result.ptr;
}

export fn nvms_zig_free(ptr: [*]u8, size: usize) void {
    const pool = std.heap.page_allocator;
    pool.free(ptr[0..size]);
}

export fn nvms_zig_alloc_aligned(size: usize, alignment: usize) ?[*]u8 {
    const pool = std.heap.page_allocator;
    const result = pool.alignedAlloc(u8, alignment, size) catch return null;
    return result.ptr;
}

// Memory statistics
export fn nvms_zig_get_alloc_count() u64 {
    return 0; // TODO: Implement with atomic counter
}

export fn nvms_zig_get_total_bytes() u64 {
    return 0; // TODO: Implement with atomic counter
}

test "MemoryPool basic allocation" {
    var pool = MemoryPool.init();
    defer pool.deinit();

    const data = pool.alloc(100);
    try std.testing.expect(data != null);
    try std.testing.expect(data.?.len == 100);
}

test "MemoryPool alignment" {
    var pool = MemoryPool.init();
    defer pool.deinit();

    const ptr = pool.allocAligned(100, 4096);
    try std.testing.expect(ptr != null);
    try std.testing.expect(@mod(@intFromPtr(ptr.?), 4096) == 0);
}

test "MemoryPool stats" {
    var pool = MemoryPool.init();
    defer pool.deinit();

    _ = try pool.alloc(100);
    const stats = pool.getStats();
    try std.testing.expect(stats.total_allocated >= 100);
}
