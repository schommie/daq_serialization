use crate::{Celsius, Device, FaultSeverity, MeasurementValue, Percent, Volts};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum BmsMessage {
    Voltages {
        source: Device,
        readings: BmsVoltageReadings,
    },
    Temperatures {
        source: Device,
        readings: BmsTemperatureReadings,
    },
    Balancing {
        source: Device,
        active_cell: u8,
        duty_cycle: Percent,
    },
    Faults {
        source: Device,
        code: u32,
        severity: FaultSeverity,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BmsVoltageReadings {
    pub pack: Volts,
    pub min_cell: Volts,
    pub max_cell: Volts,
    pub average_cell: Volts,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BmsTemperatureReadings {
    pub min: Celsius,
    pub max: Celsius,
    pub average: Celsius,
}
