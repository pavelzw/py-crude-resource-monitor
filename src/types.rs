use py_spy::Pid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonLine {
    pub stacktraces: Vec<StackTrace>,
    pub resources: ProcessResources,
    pub index: usize,
    pub time: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResources {
    pub cpu: f32,
    pub memory: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResources {
    pub memory: u64,
    pub cpu: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub thread_resources: HashMap<u64, ThreadResources>,
}

// the following structs are `Deserialize`-able wrappers for py-spy structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrace {
    pub pid: Pid,
    pub thread_id: u64,
    pub thread_name: Option<String>,
    pub os_thread_id: Option<u64>,
    pub active: bool,
    pub owns_gil: bool,
    pub frames: Vec<Frame>,
    pub process_info: Option<ProcessInfo>,
}

impl From<py_spy::StackTrace> for StackTrace {
    fn from(trace: py_spy::StackTrace) -> Self {
        StackTrace {
            pid: trace.pid,
            thread_id: trace.thread_id,
            thread_name: trace.thread_name,
            os_thread_id: trace.os_thread_id,
            active: trace.active,
            owns_gil: trace.owns_gil,
            frames: trace.frames.into_iter().map(|f| f.into()).collect(),
            process_info: trace.process_info.map(|p| (*p).clone().into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub name: String,
    pub filename: String,
    pub module: Option<String>,
    pub short_filename: Option<String>,
    pub line: i32,
    pub locals: Option<Vec<LocalVariable>>,
    pub is_entry: bool,
}

impl From<py_spy::Frame> for Frame {
    fn from(frame: py_spy::Frame) -> Self {
        Frame {
            name: frame.name,
            filename: frame.filename,
            module: frame.module,
            short_filename: frame.short_filename,
            line: frame.line,
            locals: frame
                .locals
                .map(|locals| locals.into_iter().map(|l| l.into()).collect()),
            is_entry: frame.is_entry,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVariable {
    pub name: String,
    pub addr: usize,
    pub arg: bool,
    pub repr: Option<String>,
}

impl From<py_spy::stack_trace::LocalVariable> for LocalVariable {
    fn from(local: py_spy::stack_trace::LocalVariable) -> Self {
        LocalVariable {
            name: local.name,
            addr: local.addr,
            arg: local.arg,
            repr: local.repr,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub command_line: String,
    pub parent: Option<Box<ProcessInfo>>,
}

impl From<py_spy::stack_trace::ProcessInfo> for ProcessInfo {
    fn from(info: py_spy::stack_trace::ProcessInfo) -> Self {
        ProcessInfo {
            pid: info.pid,
            command_line: info.command_line,
            parent: info.parent.map(|p| Box::new((*p).into())),
        }
    }
}
