use databroker_proto::kuksa::val as proto;
use kuksa::*;
use prost_types::Timestamp;
use std::collections::HashMap;
use std::fmt;
use std::string::ParseError;
use std::time::SystemTime;

#[derive(Debug)]
pub struct DisplayDatapoint(pub proto::v1::Datapoint);

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
                    }
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
                        println!("publish:OK");
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
                    return Ok(DisplayDatapoint(val).to_string());
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
    let datapoint_val: DisplayDatapoint;
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

/**
 */
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
            let _meta_data = entry.metadata;
            result.push(DisplayDatapoint(val));
        }
    }
    result
}

pub async fn get_target_value(
    path: &str,
    client: &mut KuksaClient,
) -> Result<Vec<DisplayDatapoint>, Box<dyn std::error::Error>> {
    let args = path;
    let paths: Vec<String> = args
        .split_whitespace()
        .map(|path| path.to_owned())
        .collect();
    let mut datapoint_vals: Vec<DisplayDatapoint> = vec![];
    match client.get_target_values(paths).await {
        Ok(data_entries) => {
            for entry in data_entries {
                if let Some(val) = entry.actuator_target {
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
                    datapoint_vals.push(DisplayDatapoint(val.clone()));
                } else {
                    println!("{} is not an actuator.", entry.path);
                }
            }
        }
        Err(kuksa_common::ClientError::Status(err)) => {}
        Err(kuksa_common::ClientError::Connection(msg)) => {}
        Err(kuksa_common::ClientError::Function(msg)) => {}
    }
    Ok(datapoint_vals)
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
