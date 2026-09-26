#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CallFrame {
    pub function_id: u32,
    pub ip: usize,
    pub stack_base: usize,
}
