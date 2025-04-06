use databroker_proto::kuksa::val as proto;
use kuksa::*;
use prost_types::Timestamp;
use std::collections::HashMap;
use std::string::ParseError;
use std::time::SystemTime;

#[derive(Debug)]
pub struct DisplayDatapoint(proto::v1::Datapoint);

pub async fn handle_actuate_command(
    path: &str,
    input: &str,
    client: &mut KuksaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("path:{}", path);
    let datapoint_entries = match client.get_metadata(vec![path]).await {
        Ok(data_entries) => Some(data_entries),
        _ => {
            println!("ERROR datapoint_entry");
            None
        }
    };
    if let Some(entries) = datapoint_entries {
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

    println!("actuate Finished!");
    //
    Ok(())
}

pub async fn handle_publish_command(
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

pub async fn get_signals(
    paths: Vec<String>,
    client: &mut KuksaClient,
) -> Result<Vec<DisplayDatapoint>, Box<dyn std::error::Error>> {
    let mut ret: Vec<DisplayDatapoint> = vec![];
    println!("{:?}", paths);
    match client.get_current_values(paths).await {
        Ok(data_entries) => {
            ret = convert_entries(data_entries);
        }
        _ => {}
    }
    Ok(ret)
}
fn convert_entries(data_entries: Vec<DataEntry>) -> Vec<DisplayDatapoint> {
    println!("convert_entries");
    let mut result: Vec<DisplayDatapoint> = vec![];
    for entry in data_entries {
        if let Some(val) = entry.value {
            let path = entry.path;
            let meta_data = entry.metadata;
            println!("{:?}", val);
            result.push(DisplayDatapoint(val));
        }
    }

    //vec![String::from("sample")]
    result
}

pub async fn handle_get_command(
    paths: Vec<String>,
    client: &mut KuksaClient,
) -> Result<String, Box<dyn std::error::Error>> {
    match client.get_current_values(paths).await {
        Ok(data_entries) => {
            for entry in data_entries {
                if let Some(val) = entry.value {
                    println!(
                        "{}: {:?} {}",
                        entry.path,
                        val,
                        //DisplayDatapoint(val),
                        entry
                            .metadata
                            .and_then(|meta| meta.unit)
                            .map(|unit| unit.to_string())
                            .unwrap_or_else(|| "".to_string())
                    );
                    return Ok(String::from("100"));
                } else {
                    println!("{}: NotAvailable", entry.path);
                }
            }
        }
        Err(kuksa_common::ClientError::Status(err)) => {}
        Err(kuksa_common::ClientError::Connection(msg)) => {}
        Err(kuksa_common::ClientError::Function(msg)) => {}
    }

    return Ok(String::from(""));
}

pub async fn handle_gettarget_command(
    path: &str,
    client: &mut KuksaClient,
) -> Result<DisplayDatapoint, Box<dyn std::error::Error>> {
    let args = path;
    let paths: Vec<String> = args
        .split_whitespace()
        .map(|path| path.to_owned())
        .collect();
    let mut datapoint_val: DisplayDatapoint;
    match client.get_target_values(paths).await {
        Ok(data_entries) => {
            for entry in data_entries {
                if let Some(val) = entry.actuator_target {
                    datapoint_val = DisplayDatapoint(val.clone());
                    println!(
                        "{}: {:?} {}",
                        entry.path,
                        val,
                        entry
                            .metadata
                            .and_then(|meta| meta.unit)
                            .map(|unit| unit.to_string())
                            .unwrap_or_else(|| "".to_string())
                    );
                    return Ok(datapoint_val);
                } else {
                    println!("{} is not an actuator.", entry.path);
                }
            }
        }
        Err(kuksa_common::ClientError::Status(err)) => {}
        Err(kuksa_common::ClientError::Connection(msg)) => {}
        Err(kuksa_common::ClientError::Function(msg)) => {}
    }
    Err("TEST ERROR".into())
}
