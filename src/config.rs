use serde::{Deserialize, Serialize};

use crate::{
    ddr4::{DDR4Org, Speed},
    memory::MappingType,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub channels: usize,
    pub ranks: usize,
    pub ddr4_org: DDR4Org,
    pub ddr4_speed: Speed,
    pub mapping_type: MappingType,
}

impl Config {
    pub fn from_toml_path(path: &str) -> Self {
        toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: 1,
            ranks: 1,
            mapping_type: MappingType::ChRaBaRoCo,
            ddr4_org: DDR4Org::DDR4_4Gb_x8,
            ddr4_speed: Speed::DDR4_2400R,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn dum_config() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        println!("{}", toml);
    }
}
