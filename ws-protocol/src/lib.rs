pub mod bms;
pub mod daq;
pub mod device;
pub mod units;
pub mod vcu;

pub use bms::*;
pub use daq::*;
pub use device::*;
pub use units::*;
pub use vcu::*;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(
    tag = "system",
    content = "message",
    rename_all = "lowercase",
    deny_unknown_fields
)]
pub enum Message {
    Daq(DaqMessage),
    Bms(BmsMessage),
    Vcu(VcuMessage),
}

impl Message {
    pub fn encode_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn decode_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Message should always serialize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_daq_temperature_message() {
        let message = Message::Daq(DaqMessage::Temperature {
            source: Device::NodeFL,
            samples: [TemperatureSample {
                tire: Celsius(23.5),
                brake: Celsius(24.0),
            }; TEMPERATURE_SAMPLE_COUNT],
        });

        let json = serde_json::to_value(&message).expect("message should serialize");

        assert_eq!(json["system"], "daq");
        assert_eq!(json["message"]["frame"], "temperature");
        assert_eq!(json["message"]["source"], "nodefl");
        assert_eq!(json["message"]["samples"][0]["tire"], 23.5);
        assert_eq!(json["message"]["samples"][0]["brake"], 24.0);
    }

    #[test]
    fn deserializes_bms_set_value_message() {
        let json = r#"{
            "system": "bms",
            "message": {
                "frame": "setValue",
                "source": "bms",
                "target": 12.0
            }
        }"#;

        let message = Message::decode_json(json).expect("message should deserialize");

        match message {
            Message::Bms(BmsMessage::SetValue { source, target }) => {
                assert!(matches!(source, Device::Bms));
                assert_eq!(target, MeasurementValue(12.0));
            }
            other => panic!("expected BMS setValue message, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_sample_fields() {
        let json = r#"{
            "system": "daq",
            "message": {
                "frame": "temperature",
                "source": "nodefl",
                "samples": [
                    { "sensorA": 23.5, "sensorB": 24.0, "extra": 1.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 },
                    { "sensorA": 23.5, "sensorB": 24.0 }
                ]
            }
        }"#;

        assert!(Message::decode_json(json).is_err());
    }
}
