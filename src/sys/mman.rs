use {Error, Result, NixPath};
use errno::Errno;
use fcntl::OFlag;
use libc::{self, c_int, c_void, size_t, off_t};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

libc_bitflags!{
    /// Desired memory protection of a memory mapping.
    pub struct ProtFlags: libc::c_int {
        /// Pages cannot be accessed.
        PROT_NONE;
        /// Pages can be read.
        PROT_READ;
        /// Pages can be written.
        PROT_WRITE;
        /// Pages can be executed
        PROT_EXEC;
        /// Apply protection up to the end of a mapping that grows upwards.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        PROT_GROWSDOWN;
        /// Apply protection down to the beginning of a mapping that grows downwards.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        PROT_GROWSUP;
    }
}

libc_bitflags!{
    /// Additional parameters for `mmap()`.
    pub struct MapFlags: c_int {
        /// Compatibility flag. Ignored.
        MAP_FILE;
        /// Share this mapping. Mutually exclusive with `MAP_PRIVATE`.
        MAP_SHARED;
        /// Create a private copy-on-write mapping. Mutually exclusive with `MAP_SHARED`.
        MAP_PRIVATE;
        /// Place the mapping at exactly the address specified in `addr`.
        MAP_FIXED;
        /// Synonym for `MAP_ANONYMOUS`.
        MAP_ANON;
        /// The mapping is not backed by any file.
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
        MAP_ANONYMOUS;
        /// Put the mapping into the first 2GB of the process address space.
        #[cfg(any(all(any(target_os = "android", target_os = "linux"),
                      any(target_arch = "x86", target_arch = "x86_64")),
                  all(target_os = "linux", target_env = "musl", any(target_arch = "x86", target_pointer_width = "64")),
                  all(target_os = "freebsd", target_pointer_width = "64")))]
        MAP_32BIT;
        /// Used for stacks; indicates to the kernel that the mapping should extend downward in memory.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_GROWSDOWN;
        /// Compatibility flag. Ignored.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_DENYWRITE;
        /// Compatibility flag. Ignored.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_EXECUTABLE;
        /// Mark the mmaped region to be locked in the same way as `mlock(2)`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_LOCKED;
        /// Do not reserve swap space for this mapping.
        ///
        /// This was removed in FreeBSD 11.
        #[cfg(not(target_os = "freebsd"))]
        MAP_NORESERVE;
        /// Populate page tables for a mapping.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_POPULATE;
        /// Only meaningful when used with `MAP_POPULATE`. Don't perform read-ahead.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_NONBLOCK;
        /// Allocate the mapping using "huge pages."
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_HUGETLB;
        /// Lock the mapped region into memory as with `mlock(2)`.
        #[cfg(target_os = "netbsd")]
        MAP_WIRED;
        /// Causes dirtied data in the specified range to be flushed to disk only when necessary.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MAP_NOSYNC;
        /// Rename private pages to a file.
        ///
        /// This was removed in FreeBSD 11.
        #[cfg(any(target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd"))]
        MAP_RENAME;
        /// Region may contain semaphores.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
        MAP_HASSEMAPHORE;
        /// Region grows down, like a stack.
        #[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
        MAP_STACK;
        /// Pages in this mapping are not retained in the kernel's memory cache.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MAP_NOCACHE;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MAP_JIT;
    }
}

libc_enum!{
    /// Usage information for a range of memory to allow for performance optimizations by the kernel.
    ///
    /// Used by [`madvise`](./fn.madvise.html).
    #[repr(i32)]
    pub enum MmapAdvise {
        /// No further special treatment. This is the default.
        MADV_NORMAL,
        /// Expect random page references.
        MADV_RANDOM,
        /// Expect sequential page references.
        MADV_SEQUENTIAL,
        /// Expect access in the near future.
        MADV_WILLNEED,
        /// Do not expect access in the near future.
        MADV_DONTNEED,
        /// Free up a given range of pages and its associated backing store.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_REMOVE,
        /// Do not make pages in this range available to the child after a `fork(2)`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DONTFORK,
        /// Undo the effect of `MADV_DONTFORK`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DOFORK,
        /// Poison the given pages.
        ///
        /// Subsequent references to those pages are treated like hardware memory corruption.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_HWPOISON,
        /// Enable Kernel Samepage Merging (KSM) for the given pages.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_MERGEABLE,
        /// Undo the effect of `MADV_MERGEABLE`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_UNMERGEABLE,
        /// Preserve the memory of each page but offline the original page.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_SOFT_OFFLINE,
        /// Enable Transparent Huge Pages (THP) for pages in the given range.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_HUGEPAGE,
        /// Undo the effect of `MADV_HUGEPAGE`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_NOHUGEPAGE,
        /// Exclude the given range from a core dump.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DONTDUMP,
        /// Undo the effect of an earlier `MADV_DONTDUMP`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DODUMP,
        /// Specify that the application no longer needs the pages in the given range.
        MADV_FREE,
        /// Request that the system not flush the current range to disk unless it needs to.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_NOSYNC,
        /// Undoes the effects of `MADV_NOSYNC` for any future pages dirtied within the given range.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_AUTOSYNC,
        /// Region is not included in a core file.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_NOCORE,
        /// Include region in a core file
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_CORE,
        #[cfg(any(target_os = "freebsd"))]
        MADV_PROTECT,
        /// Invalidate the hardware page table for the given region.
        #[cfg(target_os = "dragonfly")]
        MADV_INVAL,
        /// Set the offset of the page directory page to `value` for the virtual page table.
        #[cfg(target_os = "dragonfly")]
        MADV_SETMAP,
        /// Indicates that the application will not need the data in the given range.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_ZERO_WIRED_PAGES,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_FREE_REUSABLE,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_FREE_REUSE,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_CAN_REUSE,
    }
}

libc_bitflags!{
    /// Configuration flags for `msync`.
    pub struct MsFlags: c_int {
        /// Schedule an update but return immediately.
        MS_ASYNC;
        /// Invalidate all cached data.
        MS_INVALIDATE;
        /// Invalidate pages, but leave them mapped.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MS_KILLPAGES;
        /// Deactivate pages, but leave them mapped.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MS_DEACTIVATE;
        /// Perform an update and wait for it to complete.
        MS_SYNC;
    }
}

pub unsafe fn mlock(addr: *const c_void, length: size_t) -> Result<()> {
    Errno::result(libc::mlock(addr, length)).map(drop)
}

pub unsafe fn munlock(addr: *const c_void, length: size_t) -> Result<()> {
    Errno::result(libc::munlock(addr, length)).map(drop)
}

/// Calls to mmap are inherently unsafe, so they must be made in an unsafe block. Typically
/// a higher-level abstraction will hide the unsafe interactions with the mmap'd region.
pub unsafe fn mmap(addr: *mut c_void, length: size_t, prot: ProtFlags, flags: MapFlags, fd: RawFd, offset: off_t) -> Result<*mut c_void> {
    let ret = libc::mmap(addr, length, prot.bits(), flags.bits(), fd, offset);

    if ret == libc::MAP_FAILED {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(ret)
    }
}

pub unsafe fn munmap(addr: *mut c_void, len: size_t) -> Result<()> {
    Errno::result(libc::munmap(addr, len)).map(drop)
}

pub unsafe fn madvise(addr: *mut c_void, length: size_t, advise: MmapAdvise) -> Result<()> {
    Errno::result(libc::madvise(addr, length, advise as i32)).map(drop)
}

pub unsafe fn msync(addr: *mut c_void, length: size_t, flags: MsFlags) -> Result<()> {
    Errno::result(libc::msync(addr, length, flags.bits())).map(drop)
}

#[cfg(not(target_os = "android"))]
pub fn shm_open<P: ?Sized + NixPath>(name: &P, flag: OFlag, mode: Mode) -> Result<RawFd> {
    let ret = try!(name.with_nix_path(|cstr| {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::c_uint)
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::mode_t)
        }
    }));

    Errno::result(ret)
}

#[cfg(not(target_os = "android"))]
pub fn shm_unlink<P: ?Sized + NixPath>(name: &P) -> Result<()> {
    let ret = try!(name.with_nix_path(|cstr| {
        unsafe { libc::shm_unlink(cstr.as_ptr()) }
    }));

    Errno::result(ret).map(drop)
}
