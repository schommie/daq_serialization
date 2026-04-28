use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Celsius(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Rpm(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Volts(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct NewtonMeters(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Percent(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct MeasurementValue(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct FaultSeverity(pub f32);
