#[cfg(windows)]
mod platform {
    use std::io;
    use std::mem::size_of;
    use std::os::windows::io::AsRawHandle;
    use std::ptr::{null, null_mut};

    use std::ffi::c_void;
    use std::process::Child;

    type Handle = *mut c_void;
    type Bool = i32;
    type Dword = u32;

    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: Dword = 0x0000_2000;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct JobObjectBasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: Dword,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: Dword,
        affinity: usize,
        priority_class: Dword,
        scheduling_class: Dword,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct JobObjectExtendedLimitInformation {
        basic_limit_information: JobObjectBasicLimitInformation,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(lp_job_attributes: *mut c_void, lp_name: *const u16) -> Handle;
        fn SetInformationJobObject(
            job: Handle,
            job_object_information_class: i32,
            job_object_information: *mut c_void,
            job_object_information_length: Dword,
        ) -> Bool;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> Bool;
        fn CloseHandle(handle: Handle) -> Bool;
    }

    #[derive(Debug)]
    pub struct JobObject {
        handle: Handle,
    }

    impl JobObject {
        pub fn new() -> io::Result<Self> {
            unsafe {
                let handle = CreateJobObjectW(null_mut(), null());
                if handle.is_null() {
                    return Err(io::Error::last_os_error());
                }

                let mut info = JobObjectExtendedLimitInformation::default();
                info.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                let ok = SetInformationJobObject(
                    handle,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                    &mut info as *mut _ as *mut c_void,
                    size_of::<JobObjectExtendedLimitInformation>() as Dword,
                );
                if ok == 0 {
                    let error = io::Error::last_os_error();
                    CloseHandle(handle);
                    return Err(io::Error::new(
                        error.kind(),
                        format!("设置 Job Object 限制失败: {error}"),
                    ));
                }

                Ok(Self { handle })
            }
        }

        pub fn attach_child(&self, child: &Child) -> io::Result<()> {
            unsafe {
                let ok = AssignProcessToJobObject(self.handle, child.as_raw_handle() as Handle);
                if ok == 0 {
                    let error = io::Error::last_os_error();
                    return Err(io::Error::new(
                        error.kind(),
                        format!("绑定子进程到 Job Object 失败: {error}"),
                    ));
                }
            }
            Ok(())
        }
    }

    impl Drop for JobObject {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use std::io;
    use std::process::Child;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct JobObject;

    impl JobObject {
        pub fn new() -> io::Result<Self> {
            Ok(Self)
        }

        pub fn attach_child(&self, _child: &Child) -> io::Result<()> {
            Ok(())
        }
    }
}

#[allow(unused_imports)]
pub use platform::JobObject;

#[cfg(test)]
mod tests {
    use super::JobObject;

    #[cfg(not(windows))]
    #[test]
    fn job_object_is_a_no_op_on_non_windows_platforms() {
        let job = JobObject::new().expect("create no-op job object");
        assert!(matches!(job.attach_child(&dummy_child()), Ok(())));
    }

    #[cfg(not(windows))]
    fn dummy_child() -> std::process::Child {
        let mut command = std::process::Command::new("sh");
        command.arg("-c").arg("exit 0");
        command.spawn().expect("spawn dummy child")
    }
}
