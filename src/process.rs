use spin::mutex::spin::SpinMutex;

const MAX_NUM_PROCESSES: usize = 256;

#[repr(C, align(4096))]
struct Process {
    owning_process: Option<usize>,
    saved_register_state: SpinMutex<[u64; 31]>,
}

static PROCESSES: SpinMutex<[Option<Process>; MAX_NUM_PROCESSES]> =
    SpinMutex::new([const { Option::<Process>::None }; MAX_NUM_PROCESSES]);

impl Drop for Process {
    fn drop(&mut self) {
        todo!()
    }
}

impl Process {
    pub fn create_independent() -> Process {
        todo!()
    }

    pub fn is_executing(&self) -> bool {
        self.saved_register_state.is_locked()
    }

    pub fn try_execute(&self) -> Result<!, ()> {
        todo!()
    }
}
