const MAX_NUM_PROCESSES: usize = 256;

struct Process;

static PROCESSES: [Option<Process>; MAX_NUM_PROCESSES] =
    [const { Option::<Process>::None }; MAX_NUM_PROCESSES];

impl Drop for Process {
    fn drop(&mut self) {
        todo!()
    }
}
