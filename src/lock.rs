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
    use std::io::{Read, Seek, SeekFrom, Write};

    use crate::lock::runtime_lockfile;

    pub struct GlobalLock {
        lock: fs::File,
    }

    impl GlobalLock {
        pub fn new() -> std::io::Result<Self> {
            let path = runtime_lockfile();

            let mut lock = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .append(false)
                .create(true)
                .open(&path)?;

            lock.lock()?;

            let len = lock.metadata()?.len();

            // If there's a previous pid entry lock file then the corresponding process is killed
            if len > 0 {
                let mut buf = [0; 4];
                lock.read_exact(&mut buf)?;

                let prev_pid: i32 = u32::from_le_bytes(buf) as i32;
                eprintln!("{prev_pid}");

                if unsafe { libc::kill(prev_pid, libc::SIGTERM) } != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }

            lock.set_len(0)?;
            lock.seek(SeekFrom::Start(0))?;

            let pid = std::process::id();
            lock.write_all(&pid.to_le_bytes())?;

            lock.unlock()?;

            Ok(Self { lock })
        }
    }

    impl Drop for GlobalLock {
        fn drop(&mut self) {
            // Clear the pid
            let _ = self.lock.lock();
            let _ = self.lock.set_len(0);
            let _ = self.lock.unlock();
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod _impl {
    pub struct GlobalLock;

    impl GlobalLock {
        pub fn new() -> std::io::Result<Self> {
            Ok(Self)
        }
    }
}
