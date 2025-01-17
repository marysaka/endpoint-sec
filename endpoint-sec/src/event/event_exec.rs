//! [`EventExec`]

use std::ffi::OsStr;
use std::iter::FusedIterator;

#[cfg(feature = "macos_13_0_0")]
use endpoint_sec_sys::{cpu_subtype_t, cpu_type_t};
use endpoint_sec_sys::{
    es_event_exec_t, es_exec_arg, es_exec_arg_count, es_exec_env, es_exec_env_count,
    es_string_token_t,
};
#[cfg(feature = "macos_11_0_0")]
use endpoint_sec_sys::{es_exec_fd, es_exec_fd_count, es_fd_t, ShouldNotBeNull};

#[cfg(feature = "macos_10_15_1")]
use crate::File;
use crate::Process;

/// A process execution event.
#[doc(alias = "es_event_exec_t")]
pub struct EventExec<'a> {
    /// The raw reference.
    pub(crate) raw: &'a es_event_exec_t,

    /// The version of the message.
    pub(crate) version: u32,
}

/// Describe an open file descriptor.
#[doc(alias = "es_fd_t")]
#[cfg(feature = "macos_11_0_0")]
pub struct Fd<'a>(pub(crate) &'a es_fd_t);

impl<'a> EventExec<'a> {
    /// The new process that is being executed.
    #[inline(always)]
    pub fn target(&self) -> Process<'_> {
        // Safety: 'a tied to self, object obtained through ES
        Process::new(unsafe { self.raw.target() }, self.version)
    }

    /// The exec path passed up to dyld, before symlink resolution. This is the path argument
    /// to `execve(2)` or `posix_spawn(2)`, or the interpreter from the shebang line for scripts run
    /// through the shell script image activator.
    ///
    /// Present on version 7 and later.
    #[inline(always)]
    #[cfg(feature = "macos_13_3_0")]
    pub fn dyld_exec_path(&self) -> Option<&'a OsStr> {
        if self.version >= 7 {
            // Safety: 'a tied to self, object obtained through ES
            Some(unsafe { self.raw.dyld_exec_path.as_os_str() })
        } else {
            None
        }
    }

    /// Script being executed by interpreter (if present) on version 2 and later, otherwise None.
    ///
    /// **Warning**: This only work if a script was executed directly and not as an argument to the
    /// interpreter (e.g. `./foo.sh` not `/bin/sh ./foo.sh`)
    #[cfg(feature = "macos_10_15_1")]
    #[inline(always)]
    pub fn script(&self) -> Option<File<'_>> {
        if self.version >= 2 {
            // Safety: Safe as we check the version before accessing the field.
            let script_ptr = unsafe { self.raw.anon_0.anon_0.script };

            // Safety: Safe as File cannot outlive self and ES is supposed to give us an aligned
            // and valid pointer if non-null.
            Some(File::new(unsafe { script_ptr.as_ref()? }))
        } else {
            None
        }
    }

    /// Current working directory at exec time (if present) on version 3 and later, otherwise None.
    #[inline(always)]
    #[cfg(feature = "macos_10_15_4")]
    pub fn cwd(&self) -> Option<File<'_>> {
        if self.version >= 3 {
            // Safety: Safe as File cannot outlive self and as we check the version before accessing the field.
            Some(File::new(unsafe { self.raw.anon_0.anon_0.cwd.as_ref() }))
        } else {
            None
        }
    }

    /// Highest open file descriptor after the exec completed (if present) on version 4 and later, otherwise None.
    #[inline(always)]
    #[cfg(feature = "macos_11_0_0")]
    pub fn last_fd(&self) -> Option<i32> {
        if self.version >= 4 {
            // Safety: Safe as we check the version before accessing the field.
            Some(unsafe { self.raw.anon_0.anon_0.last_fd })
        } else {
            None
        }
    }

    /// Get the number of arguments associated to the [`EventExec`].
    #[inline(always)]
    pub fn arg_count(&self) -> u32 {
        // Safety: Safe as raw is a reference and therefore cannot be null and the data comes from
        // ES: if it's not valid, there isn't anything we can do to detect it
        unsafe { es_exec_arg_count(self.raw) }
    }

    /// Get the number of environment variables associated to the [`EventExec`].
    #[inline(always)]
    pub fn env_count(&self) -> u32 {
        // Safety: Safe as raw is a reference and therefore cannot be null and the data comes from
        // ES: if it's not valid, there isn't anything we can do to detect it
        unsafe { es_exec_env_count(self.raw) }
    }

    /// Get the number of file descriptors associated to the [`EventExec`].
    #[inline(always)]
    #[cfg(feature = "macos_11_0_0")]
    pub fn fd_count(&self) -> u32 {
        // Safety: Safe as raw is a reference and therefore cannot be null and the data comes from
        // ES: if it's not valid, there isn't anything we can do to detect it
        unsafe { es_exec_fd_count(self.raw) }
    }

    /// Get the argument at the specified position on the associated [`EventExec`].
    #[inline(always)]
    pub fn arg(&self, index: u32) -> Option<&'a OsStr> {
        self.args().nth(index as _)
    }

    /// Get the environment variable at the specified position on the associated [`EventExec`].
    #[inline(always)]
    pub fn env(&self, index: u32) -> Option<&'a OsStr> {
        self.envs().nth(index as _)
    }

    /// Get the file descriptor at the specified position on the associated [`EventExec`].
    #[inline(always)]
    #[cfg(feature = "macos_11_0_0")]
    pub fn fd(&self, index: u32) -> Option<Fd<'a>> {
        self.fds().nth(index as _)
    }

    /// Iterator over the arguments
    #[inline(always)]
    pub fn args<'event>(&'event self) -> ExecArgs<'event, 'a> {
        ExecArgs::new(self)
    }

    /// Iterator over the environment
    #[inline(always)]
    pub fn envs<'event>(&'event self) -> ExecEnvs<'event, 'a> {
        ExecEnvs::new(self)
    }

    /// Iterator over the file descriptors
    #[inline(always)]
    #[cfg(feature = "macos_11_0_0")]
    pub fn fds<'event>(&'event self) -> ExecFds<'event, 'a> {
        ExecFds::new(self)
    }

    /// CPU type of the executable image which is being executed, present on version 6 or later.
    #[inline(always)]
    #[cfg(feature = "macos_13_0_0")]
    pub fn image_cputype(&self) -> Option<cpu_type_t> {
        if self.version >= 6 {
            // Safety: Safe as we check the version before accessing the field.
            Some(unsafe { self.raw.anon_0.anon_0.image_cputype })
        } else {
            None
        }
    }

    /// CPU subtype of the executable image, present on version 6 or later.
    #[inline(always)]
    #[cfg(feature = "macos_13_0_0")]
    pub fn image_cpusubtype(&self) -> Option<cpu_subtype_t> {
        if self.version >= 6 {
            // Safety: Safe as we check the version before accessing the field.
            Some(unsafe { self.raw.anon_0.anon_0.image_cpusubtype })
        } else {
            None
        }
    }

    /// Collect the argument for debug
    fn all_args(&self) -> Vec<&'a OsStr> {
        self.args().collect()
    }
}

// Safety: safe to send across threads: does not contain any interior mutability nor depend on current thread state
unsafe impl Send for EventExec<'_> {}

// This will expose all arguments, env variables and file descriptors.
impl_debug_eq_hash_with_functions!(EventExec<'a> with version;
    #[cfg(feature = "macos_11_0_0")]
    cwd,
    all_args,
    #[cfg(feature = "macos_10_15_1")]
    script,
    target,
    #[cfg(feature = "macos_11_0_0")]
    last_fd,
    arg_count,
    env_count,
    #[cfg(feature = "macos_11_0_0")]
    fd_count,
    #[cfg(feature = "macos_13_0_0")]
    image_cputype,
    #[cfg(feature = "macos_13_0_0")]
    image_cpusubtype,
);

#[cfg(feature = "macos_11_0_0")]
impl<'a> Fd<'a> {
    /// File descriptor number
    #[inline(always)]
    pub fn fd(&self) -> i32 {
        self.0.fd
    }

    /// File descriptor type, as libproc fdtype
    #[inline(always)]
    pub fn fdtype(&self) -> u32 {
        self.0.fdtype
    }

    /// Unique id of the pipe for correlation with other file descriptors pointing to the same or
    /// other end of the same pipe.
    ///
    /// **Note**: This is only valid when `fdtype == PROX_FDTYPE_PIPE`, otherwise this return None.
    #[inline(always)]
    pub fn pipe_id(&self) -> Option<u64> {
        // Safety: 'a tied to self, object obtained through ES
        unsafe { self.0.pipe() }.map(|x| x.pipe_id)
    }
}

#[cfg(feature = "macos_11_0_0")]
// Safety: safe to send across threads: does not contain any interior mutability nor depend on current thread state
unsafe impl Send for Fd<'_> {}

#[cfg(feature = "macos_11_0_0")]
impl_debug_eq_hash_with_functions!(Fd<'a>; fd, fdtype, pipe_id);

/// Generate an iterator implementation for a component of [`EventExec`]
///
/// Safety:
///
/// - `raw_element_func` will be called like this: `raw_element_func(&es_event_exec, valid index)`,
///   it must be safe to call under these conditions
/// - `raw_to_wrapped` will be called with the result of the preceding operation like this:
///   `raw_to_wrapped(raw_token)`. This token COULD be null if `raw_element_func` can return `null`
///   when called in the conditions described above. Usually Apple documents that if the event is
///   a valid pointer and the index is correct, the function cannot return `null` and that calling
///   outside the bounds is undefined behaviour.
macro_rules! make_exec_iterator {
    ($(#[$enum_doc:meta])+ $name:ident with $element_count: ident; $item: ty; $raw_element_func: ident, $raw_to_wrapped: ident $(,)?) => {
        $(#[$enum_doc])*
        pub struct $name<'event, 'raw> {
            /// Wrapped event
            ev: &'event EventExec<'raw>,
            /// Element count. When `current >= count`, the iterator is done and will only return
            /// `None` for all subsequent calls to `next`.
            count: u32,
            /// A call to `next` will yield element `current`
            current: u32,
        }

        impl $name<'_, '_> {
            /// New iterator from event
            fn new<'ev, 'raw>(ev: &'ev EventExec<'raw>) -> $name<'ev, 'raw> {
                $name {
                    ev,
                    count: ev.$element_count(),
                    current: 0,
                }
            }
        }

        impl<'raw> Iterator for $name<'_, 'raw> {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current < self.count {
                    // Safety: Safe as raw is a reference and therefore cannot be null
                    let raw_token = unsafe { $raw_element_func(self.ev.raw, self.current) };

                    self.current = self.current.saturating_add(1);
                    // Safety: Safe as we ensure the lifetime is rebound correctly in our wrappers
                    Some(unsafe { $raw_to_wrapped(raw_token) })
                } else {
                    None
                }
            }

            #[inline(always)]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.current = n.min(u32::MAX as usize) as u32;
                self.next()
            }

            #[inline(always)]
            fn last(mut self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                self.current = self.count.saturating_sub(1);
                self.next()
            }

            #[inline(always)]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len();
                (len, Some(len))
            }

            #[inline(always)]
            fn count(mut self) -> usize {
                let len = self.len();
                self.current = self.count;
                len
            }
        }

        impl ExactSizeIterator for $name<'_, '_> {
            #[inline(always)]
            fn len(&self) -> usize {
                // Casting to usize if ok: all macOS machines are 64 bits now so a u32 will always
                // fit into a usize
                self.count.saturating_sub(self.current) as _
            }
        }

        impl FusedIterator for $name<'_, '_> {}
    };
}

/// Wrapper for the `.as_ref()` call on `es_string_token_t` with lifetime extension.
///
/// # Safety
///
/// This is a horrible horrible hack. Apple documents that the `es_string_token_t` returned by
/// both [`es_exec_env`] and [`es_exec_arg`] are zero-allocation when in bounds and that the
/// returned string token must not outlive the original event, which it cannot do in our
/// iterator so it's safe. Thanks Rust for references and the borrow checker.
unsafe fn as_os_str<'a>(x: es_string_token_t) -> &'a OsStr {
    // Safety: this is only called inside the iterator where `'a` will be the lifetime of `&mut self`
    unsafe { &*(x.as_os_str() as *const _) }
}

make_exec_iterator!(
    /// Iterator over the arguments of an [`EventExec`]
    ExecArgs with arg_count;
    &'raw OsStr;
    es_exec_arg,
    as_os_str,
);

make_exec_iterator!(
    /// Iterator over the environment of an [`EventExec`]
    ExecEnvs with env_count;
    &'raw OsStr;
    es_exec_env,
    as_os_str,
);

/// Helper to declare the [`ExecFds`] iterator
///
/// Safety: safe if `fd` is aligned, non-null of the correct type
#[cfg(feature = "macos_11_0_0")]
unsafe fn make_fd<'a>(fd: ShouldNotBeNull<es_fd_t>) -> Fd<'a> {
    // Safety: see above
    unsafe { Fd(fd.as_ref()) }
}

#[cfg(feature = "macos_11_0_0")]
make_exec_iterator!(
    /// Iterator over the file descriptors of an [`EventExec`]
    ExecFds with fd_count;
    Fd<'raw>;
    es_exec_fd,
    make_fd,
);
