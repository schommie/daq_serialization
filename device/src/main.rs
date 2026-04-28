use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const HOST_WS_URL: &str = "ws://127.0.0.1:9002";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Bms,
    Vcu,
    Raspi,
    NodeFL,
    NodeFR,
    NodeRL,
    NodeRR,
    NodeDash,
    NodeRideHeight,
    NodePDMTB,
    NodePDMDASH,
    NodePDMPCBPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "system", content = "message", rename_all = "lowercase")]
pub enum WsMessage {
    Daq(DaqMessage),
    Bms(BmsMessage),
    Vcu(VcuMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum DaqMessage {
    Temperature {
        device: Device,
        values: HashMap<String, f32>,
    },
    WheelSpeed {
        device: Device,
        values: HashMap<String, f32>,
    },
    Imu {
        device: Device,
        values: HashMap<String, f32>,
    },
    #[serde(rename = "tbd")]
    Tbd {
        device: Device,
        values: HashMap<String, f32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum BmsMessage {
    Voltages {
        device: Device,
        values: HashMap<String, f32>,
    },
    Temperatures {
        device: Device,
        values: HashMap<String, f32>,
    },
    Balancing {
        device: Device,
        values: HashMap<String, f32>,
    },
    Faults {
        device: Device,
        values: HashMap<String, f32>,
    },
    SetValue {
        device: Device,
        values: HashMap<String, f32>,
    },
    Reset {
        device: Device,
    },
    Ping {
        device: Device,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "camelCase")]
pub enum VcuMessage {
    TorqueRequest {
        device: Device,
        values: HashMap<String, f32>,
    },
    SetValue {
        device: Device,
        values: HashMap<String, f32>,
    },
    Reset {
        device: Device,
    },
    Ping {
        device: Device,
    },
}

impl WsMessage {
    pub fn to_ws_message(&self) -> Message {
        let json = serde_json::to_string(self).expect("WsMessage should always serialize");
        Message::text(json)
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("WsMessage should always serialize")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("connecting to host at {HOST_WS_URL}");
    let (ws, _) = connect_async(HOST_WS_URL).await?;
    println!("connected. type raw json and press enter to send it.");
    println!(
        r#"example: {{"system":"daq","message":{{"frame":"temperature","device":"nodefl","values":{{"rpm":42.0}}}}}}"#
    );

    let (mut ws_tx, mut ws_rx) = ws.split();

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    println!("host -> device raw: {text}");

                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_message) => {
                            println!(
                                "host -> device deserialized:\n{}",
                                ws_message.to_pretty_json()
                            );
                            handle_ws_message(&ws_message);
                        }
                        Err(_) => println!("host -> device did not match WsMessage"),
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("host closed the websocket");
                    break;
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("receive error: {error}");
                    break;
                }
            }
        }
    });

    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();

        if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
            break;
        }

        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("test") {
            let test_data = HashMap::from([("test".to_string(), 42.0)]);

            let ws_message = WsMessage::Daq(DaqMessage::Temperature {
                device: Device::NodeFL,
                values: test_data,
            });

            println!("sending test daq message:\n{}", ws_message.to_pretty_json());
            ws_tx.send(ws_message.to_ws_message()).await?;
        } else if line.eq_ignore_ascii_case("test2") {
            let test_data = HashMap::from([("test2".to_string(), 42.0)]);

            let ws_message = WsMessage::Bms(BmsMessage::SetValue {
                device: Device::Bms,
                values: test_data,
            });

            println!("sending test bms message:\n{}", ws_message.to_pretty_json());
            ws_tx.send(ws_message.to_ws_message()).await?;
        } else {
            ws_tx.send(Message::text(line.to_owned())).await?;
        }
    }

    let _ = ws_tx.send(Message::Close(None)).await;
    let _ = rx_thread.await;

    Ok(())
}

fn handle_ws_message(message: &WsMessage) {
    match message {
        WsMessage::Daq(message) => handle_daq_message(message),
        WsMessage::Bms(message) => handle_bms_message(message),
        WsMessage::Vcu(message) => handle_vcu_message(message),
    }
}

fn handle_daq_message(message: &DaqMessage) {
    println!("host -> device daq message: {message:?}");
}

fn handle_bms_message(message: &BmsMessage) {
    println!("host -> device bms message: {message:?}");
}

fn handle_vcu_message(message: &VcuMessage) {
    println!("host -> device vcu message: {message:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_daq_message_with_system_router() {
        let message = WsMessage::Daq(DaqMessage::Temperature {
            device: Device::NodeFL,
            values: HashMap::from([("celsius".to_string(), 23.5)]),
        });

        let json = serde_json::to_value(&message).expect("message should serialize");

        assert_eq!(json["system"], "daq");
        assert_eq!(json["message"]["frame"], "temperature");
        assert_eq!(json["message"]["device"], "nodefl");
        assert_eq!(json["message"]["values"]["celsius"], 23.5);
    }

    #[test]
    fn deserializes_bms_message_without_command_or_telemetry_type() {
        let json = r#"{
            "system": "bms",
            "message": {
                "frame": "setValue",
                "device": "bms",
                "values": { "target": 12.0 }
            }
        }"#;

        let message: WsMessage = serde_json::from_str(json).expect("message should deserialize");

        match message {
            WsMessage::Bms(BmsMessage::SetValue { device, values }) => {
                assert!(matches!(device, Device::Bms));
                assert_eq!(values["target"], 12.0);
            }
            other => panic!("expected BMS setValue message, got {other:?}"),
        }
    }
}
