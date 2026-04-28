use crate::{Celsius, Device, MeasurementValue, Rpm};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const TEMPERATURE_SAMPLE_COUNT: usize = 15;
pub const IMU_SAMPLE_COUNT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum DaqMessage {
    Temperature {
        source: Device,
        samples: [TemperatureSample; TEMPERATURE_SAMPLE_COUNT],
    },
    WheelSpeed {
        source: Device,
        rpm: Rpm,
    },
    Imu {
        source: Device,
        samples: [ImuSample; IMU_SAMPLE_COUNT],
    },
    #[serde(rename = "tbd")]
    Tbd {
        source: Device,
        value: MeasurementValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemperatureSample {
    pub tire: Celsius,
    pub brake: Celsius,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImuSample {
    pub acceleration: Acceleration,
    pub angular_acceleration: AngularAcceleration,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AngularAcceleration {
    pub rho: f32,
    pub theta: f32,
    pub phi: f32,
}
