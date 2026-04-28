use crate::{Device, MeasurementValue, NewtonMeters};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum VcuMessage {
    TorqueRequest {
        source: Device,
        torque: NewtonMeters,
    },
    SetValue {
        source: Device,
        target: MeasurementValue,
    },
    Reset {
        source: Device,
    },
    Ping {
        source: Device,
    },
}
