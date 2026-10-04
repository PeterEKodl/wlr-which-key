pub use _impl::GlobalLock;

use std::env;
use std::path::PathBuf;

pub fn runtime_lockfile() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR").map_or_else(
        || {
            let uid = unsafe { libc::geteuid() };

            ["/tmp", &format!("wlr-which-key-{uid}.lock")]
                .iter()
                .collect()
        },
        |runtime_dir| {
            [&runtime_dir, std::ffi::OsStr::new("wlr-which-key.lock")]
                .iter()
                .collect()
        },
    )
}

#[cfg(target_os = "linux")]
mod _impl {
    use std::fs;

    use crate::lock::runtime_lockfile;

    pub struct GlobalLock {
        lock: fs::File,
    }

    impl GlobalLock {
        pub fn try_lock() -> std::io::Result<Self> {
            let path = runtime_lockfile();

            let lock = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(true)
                .open(path)?;

            lock.try_lock()?;

            Ok(Self { lock })
        }
    }

    impl Drop for GlobalLock {
        fn drop(&mut self) {
            let _ = self.lock.unlock();
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod _impl {
    pub struct GlobalLock;

    impl GlobalLock {
        pub fn try_lock() -> std::io::Result<Self> {
            Ok(Self)
        }
    }
}
