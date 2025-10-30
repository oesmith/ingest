use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub enum BodyType {
    Cars,
    Motorcycles,
    #[serde(rename = "Buses and coaches")]
    Buses,
    #[serde(rename = "Light goods vehicles")]
    LightGoods,
    #[serde(rename = "Heavy goods vehicles")]
    HeavyGoods,
    #[serde(rename = "Other vehicles")]
    Other,
}

#[derive(Deserialize, Debug, Clone)]
pub enum FuelType {
    #[serde(alias = "PETROL")]
    Petrol,
    #[serde(alias = "DIESEL")]
    Diesel,
    #[serde(alias = "GAS")]
    Gas,
    #[serde(
        rename = "Battery electric",
        alias = "BATTERY ELECTRIC"
    )]
    BEV,
    #[serde(
        rename = "Hybrid electric (petrol)",
        alias = "Hybrid electric (Petrol)",
        alias = "HYBRID ELECTRIC (PETROL)"
    )]
    PetrolHybrid,
    #[serde(
        rename = "Hybrid electric (diesel)",
        alias = "Hybrid electric (Diesel)",
        alias = "HYBRID ELECTRIC (DIESEL)"
    )]
    DieselHybrid,
    #[serde(
        rename = "Plug-in hybrid electric (petrol)",
        alias = "Plug-in hybrid electric (Petrol)",
        alias = "PLUG-IN HYBRID ELECTRIC (PETROL)"
    )]
    PetrolPluginHybrid,
    #[serde(
        rename = "Plug-in hybrid electric (diesel)",
        alias = "Plug-in hybrid electric (Diesel)",
        alias = "PLUG-IN HYBRID ELECTRIC (DIESEL)"
    )]
    DieselPluginHybrid,
    #[serde(rename = "Fuel cell electric", alias = "FUEL CELL ELECTRIC")]
    FuelCell,
    #[serde(rename = "Range extended electric", alias = "RANGE EXTENDED ELECTRIC")]
    RangeExtender,
    #[serde(rename = "Other fuel types", alias = "Other", alias = "OTHER FUEL TYPES")]
    Other,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum LicenceStatus {
    Licensed,
    SORN,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OptionalNumber {
    Count(i32),
    Flag(FlagType),
}

#[derive(Deserialize, Debug, Clone)]
pub enum FlagType {
    #[serde(rename = "[x]")]
    NotAvailable,
    #[serde(rename = "[z]")]
    NotApplicable,
}

#[derive(Debug, Clone)]
pub struct VehicleIdentity<'a> {
    pub make: &'a str,
    pub generic_model: &'a str,
    pub model: &'a str,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Veh0120 {
    #[serde(rename = "BodyType")]
    pub body_type: BodyType,

    #[serde(rename = "Make")]
    pub make: String,

    #[serde(rename = "GenModel")]
    pub generic_model: String,

    #[serde(rename = "Model")]
    pub model: String,

    #[serde(rename = "Fuel")]
    pub fuel: FuelType,

    #[serde(rename = "LicenceStatus")]
    pub licence_status: LicenceStatus,

    #[serde(flatten)]
    pub extra: HashMap<String, i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Veh0124 {
    #[serde(rename = "BodyType")]
    pub body_type: BodyType,

    #[serde(rename = "Make")]
    pub make: String,

    #[serde(rename = "GenModel")]
    pub generic_model: String,

    #[serde(rename = "Model")]
    pub model: String,

    #[serde(rename = "YearFirstUsed")]
    pub first_used: OptionalNumber,

    #[serde(rename = "YearManufacture")]
    pub manufactured: OptionalNumber,

    #[serde(rename = "LicenceStatus")]
    pub licence_status: LicenceStatus,

    #[serde(flatten)]
    pub extra: HashMap<String, OptionalNumber>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Veh0160 {
    #[serde(rename = "BodyType")]
    pub body_type: BodyType,

    #[serde(rename = "Make")]
    pub make: String,

    #[serde(rename = "GenModel")]
    pub generic_model: String,

    #[serde(rename = "Model")]
    pub model: String,

    #[serde(rename = "Fuel")]
    pub fuel: FuelType,

    #[serde(flatten)]
    pub extra: HashMap<String, i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Veh0220 {
    #[serde(rename = "BodyType")]
    pub body_type: BodyType,

    #[serde(rename = "Make")]
    pub make: String,

    #[serde(rename = "GenModel")]
    pub generic_model: String,

    #[serde(rename = "Model")]
    pub model: String,

    #[serde(rename = "Fuel")]
    pub fuel: FuelType,

    #[serde(rename = "EngineSizeSimple")]
    pub engine_size_simple: OptionalNumber,

    #[serde(rename = "EngineSizeDesc")]
    pub engine_size_desc: String,

    #[serde(rename = "LicenceStatus")]
    pub licence_status: LicenceStatus,

    #[serde(flatten)]
    pub extra: HashMap<String, i32>,
}

pub trait HasIdentity {
    fn identity(&self) -> VehicleIdentity;
}

impl HasIdentity for Veh0120 {
    fn identity(&self) -> VehicleIdentity {
        VehicleIdentity { make: &self.make, generic_model: &self.generic_model, model: &self.model }
    }
}

impl HasIdentity for Veh0124 {
    fn identity(&self) -> VehicleIdentity {
        VehicleIdentity { make: &self.make, generic_model: &self.generic_model, model: &self.model }
    }
}

impl HasIdentity for Veh0160 {
    fn identity(&self) -> VehicleIdentity {
        VehicleIdentity { make: &self.make, generic_model: &self.generic_model, model: &self.model }
    }
}

impl HasIdentity for Veh0220 {
    fn identity(&self) -> VehicleIdentity {
        VehicleIdentity { make: &self.make, generic_model: &self.generic_model, model: &self.model }
    }
}
