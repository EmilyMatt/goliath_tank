use goliath_common::core::{Client, Vehicle};
use goliath_common::dev::NaiveDb;
use std::collections::HashMap;

#[derive(Default)]
pub struct CacheDb {
    vehicles: HashMap<String, Vehicle>,
    clients: HashMap<String, Client>,
}

impl CacheDb {
    pub fn new() -> Self {
        Self {
            vehicles: HashMap::from([(
                "EmilyVehicle".to_string(),
                Vehicle {
                    id: "EmilyVehicle".to_string(),
                    secret_key: "EmilyVehicleSecret".to_string(),
                },
            )]),
            clients: HashMap::from([(
                "EmilyClient".to_string(),
                Client {
                    id: "EmilyClient".to_string(),
                    secret_key: "EmilyClientSecret".to_string(),
                },
            )]),
        }
    }
}

impl NaiveDb for CacheDb {
    fn get_client(&self, id: &str) -> Option<&Client> {
        self.clients.get(id)
    }

    fn get_client_mut(&mut self, id: &str) -> Option<&mut Client> {
        self.clients.get_mut(id)
    }

    fn get_vehicle(&self, id: &str) -> Option<&Vehicle> {
        self.vehicles.get(id)
    }

    fn get_vehicle_mut(&mut self, id: &str) -> Option<&mut Vehicle> {
        self.vehicles.get_mut(id)
    }
}
