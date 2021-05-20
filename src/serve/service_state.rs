use std::process::Child;

use log::info;

use super::{ProtoBinder, Service};

pub(crate) struct ServiceState<'a, P: ProtoBinder> {
    // todo(tmfink): reap child processes
    child_procs: Vec<Child>,
    pub(crate) service: &'a Service,
    pub(crate) proto_binder: P,
}

impl<'a, P: ProtoBinder> ServiceState<'a, P> {
    pub(crate) fn new(service: &'a Service, proto_binder: P) -> Self {
        Self {
            service,
            proto_binder,
            child_procs: Vec::new(),
        }
    }
    pub(crate) fn add_child(&mut self, child: Child) {
        self.child_procs.push(child)
    }

    pub(crate) fn _children_count(&self) -> usize {
        self.child_procs.len()
    }

    /// Reap child processes that have exited
    pub(crate) fn try_reap_children(&mut self) {
        let mut new_children = Vec::new();
        for mut child in self.child_procs.drain(..) {
            match child.try_wait() {
                Ok(Some(status)) => info!(
                    "service {:?} child exited with status {}",
                    self.service.name, status
                ),
                Ok(None) => new_children.push(child),
                Err(err) => info!(
                    "failed to wait for service {:?} child: {}",
                    self.service.name, err
                ),
            }
        }
        self.child_procs = new_children;
    }
}
