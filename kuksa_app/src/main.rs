use kuksa::*;
use prost_types::Timestamp;
use std::collections::HashMap;
use std::fmt;
use std::string::ParseError;
use std::time::SystemTime;

use ansi_term::Color;
use std::{thread, time};

struct DisplayDatapoint(proto::v1::Datapoint);
use databroker_proto::kuksa::val as proto;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: String = "127.0.0.1:55555".to_string();
    let mut client = KuksaClient::new(kuksa_common::to_uri(url)?);

    handle_publish_command("Vehicle.Speed", "200", &mut client).await?;
    handle_actuate_command(
        "Vehicle.Cabin.Door.Row1.PassengerSide.IsOpen",
        "true",
        &mut client,
    )
    .await?;

    let mut subscription_nbr = 1;
    let input: String = "Vehicle.Speed".to_string();
    match client.subscribe(vec![&input]).await {
        Ok(mut subscription) => {
            //            let iface = interface.clone();
            tokio::spawn(async move {
                let sub_disp = format!("[{subscription_nbr}]");
                let sub_disp_pad = " ".repeat(sub_disp.len());
                let sub_disp_color = format!("{}", Color::White.dimmed().paint(&sub_disp));

                loop {
                    match subscription.message().await {
                        Ok(subscribe_resp) => {
                            println!("{:?}", subscribe_resp);
                            if let Some(resp) = subscribe_resp {
                                // Build output before writing it
                                // (to avoid interleaving confusion)
                                use std::fmt::Write;
                                let mut output = String::new();
                                let mut first_line = true;
                                for update in resp.updates {
                                    if first_line {
                                        first_line = false;
                                        write!(output, "{} ", &sub_disp_color,).unwrap();
                                    } else {
                                        write!(output, "{} ", &sub_disp_pad,).unwrap();
                                    }
                                    if let Some(entry) = update.entry {
                                        if let Some(value) = entry.value {
                                            writeln!(
                                                output,
                                                "{}: {} {}",
                                                entry.path,
                                                DisplayDatapoint(value),
                                                entry
                                                    .metadata
                                                    .and_then(|meta| meta.unit)
                                                    .map(|unit| unit.to_string())
                                                    .unwrap_or_else(|| "".to_string())
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                            //                                write!(iface, "{output}").unwrap();
                            } else {
                                //                                writeln!(
                                //                                    iface,
                                //                                    "{} {}",
                                //                                    Color::Red.dimmed().paint(&sub_disp),
                                //                                    Color::White
                                //                                        .dimmed()
                                //                                        .paint("Server gone. Subscription stopped"),
                                //                                )
                                //                                .unwrap();
                                break;
                            }
                        }
                        Err(err) => {
                            //                            write!(
                            //                                iface,
                            //                                "{} {}",
                            //                                &sub_disp_color,
                            //                                Color::Red.dimmed().paint(format!("Channel error: {err}"))
                            //                            )
                            //                            .unwrap();
                            break;
                        }
                    }
                }
            });
        }
        Err(kuksa_common::ClientError::Status(status)) => println!("{:?}", &status),
        Err(kuksa_common::ClientError::Connection(msg)) => println!("{:?}", msg),
        Err(kuksa_common::ClientError::Function(msg)) => {
            println!("{}", format_args!("Error {msg:?}"))
        }
    }
    let five_sec = time::Duration::from_secs(30);
    thread::sleep(five_sec);
    Ok(())
}
async fn handle_actuate_command(
    path: &str,
    input: &str,
    client: &mut KuksaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("oath:{}", path);
    let datapoint_entries = match client.get_metadata(vec![path]).await {
        Ok(data_entries) => Some(data_entries),
        _ => {
            println!("ERROR datapoint_entry");
            None
        }
    };
    if let Some(entries) = datapoint_entries {
        println!("datapoint_entry");
        for entry in entries {
            println!("entry:{:?}", entry);
            if let Some(metadata) = entry.metadata {
                let value = input.parse::<bool>().unwrap();
                let data_value: Result<kuksa::proto::v1::datapoint::Value, ParseError> =
                    Ok(proto::v1::datapoint::Value::Bool(value));
                //                let data_value = try_into_data_value(
                //                    value,
                //                    proto::v1::DataType::try_from(metadata.data_type).unwrap(),
                //                );
                if data_value.is_err() {
                    println!(
                        "Could not parse \"{value}\" as {:?}",
                        proto::v1::DataType::try_from(metadata.data_type).unwrap()
                    );
                    continue;
                }

                if metadata.entry_type != proto::v1::EntryType::Actuator as i32 {
                    println!("{}", format!("{} is not an actuator.", path));
                    //cli::print_error("actuate", format!("{} is not an actuator.", path))?;
                    continue;
                }

                let ts = Timestamp::from(SystemTime::now());
                let datapoints = HashMap::from([(
                    path.to_string(),
                    proto::v1::Datapoint {
                        timestamp: Some(ts),
                        value: Some(data_value.unwrap()),
                    },
                )]);

                println!("datapoints:{:?}", datapoints);
                match client.set_target_values(datapoints).await {
                    Ok(_) => println!("OK:actuate"),
                    Err(ClientError::Status(status)) => println!("ERROR:actuate{:?}", &status),
                    Err(ClientError::Connection(msg)) => println!("ERROR:actuate{:?}", msg),
                    Err(ClientError::Function(msg)) => {
                        println!("actuate{:?}", format_args!("Error {msg:?}"))
                    } //                    Ok(_) => cli::print_resp_ok("actuate")?,
                      //                    Err(ClientError::Status(status)) => cli::print_resp_err("actuate", &status)?,
                      //                    Err(ClientError::Connection(msg)) => cli::print_error("actuate", msg)?,
                      //                    Err(ClientError::Function(msg)) => {
                      //                        cli::print_resp_err_fmt("actuate", format_args!("Error {msg:?}"))?
                      //                    }
                }
            }
        }
    }

    println!("Hello, world!");
    //
    Ok(())
}

async fn handle_publish_command(
    path: &str,
    input: &str,
    client: &mut KuksaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let datapoint_entries = match client.get_metadata(vec![path]).await {
        Ok(data_entries) => Some(data_entries),
        Err(kuksa_common::ClientError::Status(status)) => {
            println!("metadata:{:?}", &status);
            None
        }
        Err(kuksa_common::ClientError::Connection(msg)) => {
            println!("metadata:{:?}", msg);
            None
        }
        Err(kuksa_common::ClientError::Function(msg)) => {
            println!("publish:{}", format_args!("Error {msg:?}"));
            None
        }
    };

    if let Some(entries) = datapoint_entries {
        for entry in entries {
            if let Some(metadata) = entry.metadata {
                let value = input.parse::<f32>().unwrap();
                let data_value: Result<kuksa::proto::v1::datapoint::Value, ParseError> =
                    Ok(proto::v1::datapoint::Value::Float(value));

                //                let data_value = try_into_data_value(
                //                    value,
                //                    proto::v1::DataType::try_from(metadata.data_type).unwrap(),
                //                );
                if data_value.is_err() {
                    println!(
                        "Could not parse \"{}\" as {:?}",
                        value,
                        proto::v1::DataType::try_from(metadata.data_type).unwrap()
                    );
                    continue;
                }
                let ts = Timestamp::from(SystemTime::now());
                let datapoints = HashMap::from([(
                    path.to_string().clone(),
                    proto::v1::Datapoint {
                        timestamp: Some(ts),
                        value: Some(data_value.unwrap()),
                    },
                )]);

                match client.set_current_values(datapoints).await {
                    Ok(_) => {
                        println!("publish");
                    }
                    Err(kuksa_common::ClientError::Status(status)) => {
                        println!("publish:{:?}", &status)
                    }
                    Err(kuksa_common::ClientError::Connection(msg)) => {
                        println!("publish:{:?}", msg)
                    }
                    Err(kuksa_common::ClientError::Function(msg)) => {
                        println!("publish:{}", format_args!("Error {msg:?}"));
                    }
                }
            }
        }
    }

    Ok(())
}
fn display_array<T>(f: &mut fmt::Formatter<'_>, array: &[T]) -> fmt::Result
where
    T: fmt::Display,
{
    f.write_str("[")?;
    let real_delimiter = ", ";
    let mut delimiter = "";
    for value in array {
        write!(f, "{delimiter}")?;
        delimiter = real_delimiter;
        write!(f, "{value}")?;
    }
    f.write_str("]")
}

impl fmt::Display for DisplayDatapoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.value {
            Some(value) => match value {
                proto::v1::datapoint::Value::Bool(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::Int32(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::Int64(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::Uint32(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::Uint64(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::Float(value) => f.pad(&format!("{value:.2}")),
                proto::v1::datapoint::Value::Double(value) => f.pad(&format!("{value}")),
                proto::v1::datapoint::Value::String(value) => f.pad(&format!("'{value}'")),
                proto::v1::datapoint::Value::StringArray(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::BoolArray(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::Int32Array(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::Int64Array(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::Uint32Array(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::Uint64Array(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::FloatArray(array) => display_array(f, &array.values),
                proto::v1::datapoint::Value::DoubleArray(array) => display_array(f, &array.values),
            },
            None => f.pad("None"),
        }
    }
}
