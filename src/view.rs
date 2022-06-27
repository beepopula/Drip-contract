use std::{collections::HashMap};

use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_coe_map(&self) -> HashMap<String, U128> {
        let keys = self.coe_map.keys_as_vector().clone();
        let mut coe_map: HashMap<String, U128> = HashMap::new();
        for key in keys.iter() {
            coe_map.insert(key.clone(), self.coe_map.get(&key).unwrap().into());
        }
        coe_map
    }
}