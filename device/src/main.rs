use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsFrame;

use ws_protocol::*;

const HOST_WS_URL: &str = "ws://127.0.0.1:9002";

fn to_ws_frame(message: &ws_protocol::Message) -> WsFrame {
    let json = message
        .encode_json()
        .expect("Message should always serialize");
    WsFrame::text(json)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("connecting to host at {HOST_WS_URL}");
    let (ws, _) = connect_async(HOST_WS_URL).await?;
    println!("connected. type raw json and press enter to send it.");
    println!(
        r#"example: {{"system":"daq","message":{{"frame":"wheelSpeed","source":"nodefl","rpm":42.0}}}}"#
    );

    let (mut ws_tx, mut ws_rx) = ws.split();

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(WsFrame::Text(text)) => {
                    //println!("host -> device raw: {text}");

                    match ws_protocol::Message::decode_json(&text) {
                        Ok(ws_message) => {
                            //println!("host -> device deserialized:\n{}", ws_message.to_pretty_json());
                            handle_ws_message(&ws_message);
                        }
                        Err(_) => println!("host -> device did not match Message"),
                    }
                }
                Ok(WsFrame::Close(_)) => {
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
            let ws_message = ws_protocol::Message::Daq(DaqMessage::Temperature {
                source: Device::NodeFL,
                samples: [TemperatureSample {
                    tire: Celsius(23.5),
                    brake: Celsius(24.0),
                }; TEMPERATURE_SAMPLE_COUNT],
            });

            println!("sending test daq message:\n{}", ws_message.to_pretty_json());
            ws_tx.send(to_ws_frame(&ws_message)).await?;
        } else if line.eq_ignore_ascii_case("test2") {
            let ws_message = ws_protocol::Message::Bms(BmsMessage::SetValue {
                source: Device::Bms,
                target: MeasurementValue(42.0),
            });

            println!("sending test bms message:\n{}", ws_message.to_pretty_json());
            ws_tx.send(to_ws_frame(&ws_message)).await?;
        } else {
            ws_tx.send(WsFrame::text(line.to_owned())).await?;
        }
    }

    let _ = ws_tx.send(WsFrame::Close(None)).await;
    let _ = rx_thread.await;

    Ok(())
}

fn handle_ws_message(message: &ws_protocol::Message) {
    match message {
        ws_protocol::Message::Daq(message) => handle_daq_message(message),
        ws_protocol::Message::Bms(message) => handle_bms_message(message),
        ws_protocol::Message::Vcu(message) => handle_vcu_message(message),
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
