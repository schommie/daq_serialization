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
pub enum SystemType {
    Daq,
    Bms,
    Vcu,
}

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
#[serde(rename_all = "camelCase")]
pub enum Frame {
    Voltages,
    Temperatures,
    Balancing,
    Faults,
    Temperature,
    WheelSpeed,
    Imu,
    TorqueRequest,
    #[serde(rename = "tbd")]
    Tbd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    SetValue,
    Reset,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    Telemetry {
        system: SystemType,
        frame: Frame,
        device: Device,
        values: HashMap<String, f32>,
    },
    Command {
        system: SystemType,
        command: Command,
        device: Device,
        values: HashMap<String, f32>,
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
        r#"example: {{"type":"telemetry","system":"daq","frame":"temperature","device":"nodefl","values":{{"rpm":42.0}}}}"#
    );

    let (mut ws_tx, mut ws_rx) = ws.split();

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    println!("host -> device raw: {text}");

                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_message) => println!(
                            "host -> device deserialized:\n{}",
                            ws_message.to_pretty_json()
                        ),
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

            let ws_message = WsMessage::Telemetry {
                system: SystemType::Daq,
                frame: Frame::Temperature,
                device: Device::NodeFL,
                values: test_data,
            };

            println!("sending test telemetry:\n{}", ws_message.to_pretty_json());
            ws_tx.send(ws_message.to_ws_message()).await?;
        } else if line.eq_ignore_ascii_case("test2") {
            let test_data = HashMap::from([("test2".to_string(), 42.0)]);

            let ws_message = WsMessage::Command {
                system: SystemType::Bms,
                command: Command::SetValue,
                device: Device::Bms,
                values: test_data,
            };

            println!("sending test command:\n{}", ws_message.to_pretty_json());
            ws_tx.send(ws_message.to_ws_message()).await?;
        } else {
            ws_tx.send(Message::text(line.to_owned())).await?;
        }
    }

    let _ = ws_tx.send(Message::Close(None)).await;
    let _ = rx_thread.await;

    Ok(())
}
