use log::{debug, info};
use py_spy::{Config, PythonSpy, StackTrace};
use std::collections::HashMap;

pub struct SpyHelper {
    spies: HashMap<py_spy::Pid, PythonSpy>,
}

impl SpyHelper {
    pub fn new(root: py_spy::Pid) -> anyhow::Result<Self> {
        let mut helper = SpyHelper {
            spies: HashMap::new(),
        };
        helper.track_process(root)?;

        Ok(helper)
    }

    pub fn any_live(&self) -> bool {
        !self.spies.is_empty()
    }

    pub fn refresh(&mut self) {
        let mut to_remove = Vec::new();
        let mut new_processes = Vec::new();

        for spy in self.spies.values() {
            if let Ok(children) = spy.process.child_processes() {
                for (child, _) in children {
                    if self.spies.contains_key(&child) {
                        continue;
                    }
                    new_processes.push(child);
                }
            }
            if let Err(e) = spy.process.exe() {
                info!("Tracked process exited: {}", e);
                to_remove.push(spy.pid);
            }
        }

        // Clean up exited processes
        for pid in to_remove {
            self.spies.remove(&pid);
        }

        // Add new processes
        for pid in new_processes {
            if let Err(e) = self.track_process(pid) {
                info!("Error tracking process {}: {}", pid, e);
            }
            info!("Tracking new process {}", pid);
        }

        debug!("Tracking {} processes", self.spies.len());
    }

    fn track_process(&mut self, pid: py_spy::Pid) -> anyhow::Result<()> {
        let spy = PythonSpy::new(
            pid,
            &Config {
                native: true,
                ..Default::default()
            },
        )?;

        self.spies.insert(pid, spy);

        Ok(())
    }

    pub fn get_stacktraces(&mut self) -> HashMap<py_spy::Pid, Vec<StackTrace>> {
        let mut all_traces = HashMap::new();

        for spy in self.spies.values_mut() {
            let process_traces = spy.get_stack_traces();
            if let Err(e) = process_traces {
                info!("Sample error {}: {}", spy.pid, e);
                continue;
            }
            all_traces.insert(spy.pid, process_traces.unwrap());
        }

        all_traces
    }
}
