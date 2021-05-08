use std::collections::HashSet;

use log::debug;

use crate::service::Service;

#[derive(Debug, Default)]
pub struct Config {
    services: Vec<Service>,
    service_names: HashSet<String>,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn has_service(&self, name: &str) -> bool {
        self.service_names.contains(name)
    }

    pub fn add_service(&mut self, service: Service) -> crate::Result<()> {
        debug!("Adding service {:?}", &service);

        // We should have already done this check
        assert!(!self.has_service(&service.name));

        self.service_names.insert(service.name.clone());
        self.services.push(service);
        Ok(())
    }

    pub fn services(&self) -> &[Service] {
        &self.services
    }
}
