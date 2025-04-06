use ansi_term::Color;
use kuksa::*;
use std::fmt;
use std::{thread, time};
mod kuksa_util;

struct DisplayDatapoint(proto::v1::Datapoint);
use databroker_proto::kuksa::val as proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start");
    let mut client = KuksaClient::new(kuksa_common::to_uri(String::from("127.0.0.1:55555"))?);

    kuksa_util::handle_actuate_command(
        "Vehicle.Cabin.Door.Row1.PassengerSide.IsOpen",
        "true",
        &mut client,
    )
    .await?;

    let result =
        kuksa_util::handle_get_command(vec![String::from("Vehicle.Speed")], &mut client).await?;
    println!("Get:{}", result);
    let signals = kuksa_util::get_signals(vec![String::from("Vehicle.Speed")], &mut client).await?;
    println!("{:?}", signals);

    let subscription_nbr = 1;
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
                            } else {
                                break;
                            }
                        }
                        Err(err) => {
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
    loop {
        thread::sleep(time::Duration::from_secs(10));
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
