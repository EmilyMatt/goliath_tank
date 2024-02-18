use crate::core::{Client, Vehicle};

pub trait NaiveDb {
    fn get_client(&self, id: &str) -> Option<&Client>;

    fn get_client_mut(&mut self, id: &str) -> Option<&mut Client>;

    fn get_vehicle(&self, id: &str) -> Option<&Vehicle>;

    fn get_vehicle_mut(&mut self, id: &str) -> Option<&mut Vehicle>;
}
