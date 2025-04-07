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

    println!("{:?}", signals.get(0).unwrap().0.value);

    subscribe_data(&mut client, vec!["Vehicle.Speed"]).await;
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

fn subscribe_action(subscribe_resp: std::option::Option<kuksa::proto::v1::SubscribeResponse>) {
    println!("subscribe:{:?}", subscribe_resp);

    let subscription_nbr = 1;
    let sub_disp = format!("[{subscription_nbr}]");
    let sub_disp_pad = " ".repeat(sub_disp.len());
    let sub_disp_color = format!("{}", Color::White.dimmed().paint(&sub_disp));

    if let Some(resp) = subscribe_resp {
        for update in resp.updates {
            if let Some(entry) = update.entry {
                //subscribe:Some(SubscribeResponse { updates: [EntryUpdate { entry: Some(DataEntry { path: "Vehicle.Speed", value: Some(D
                //atapoint { timestamp: Some(Timestamp { seconds: 1744037974, nanos: 563389959 }), value: Some(Float(37.0)) }), actuator_
                //target: None, metadata: Some(Metadata { data_type: Unspecified, entry_type: Unspecified, description: None, comment: No
                //ne, deprecation: None, unit: Some("km/h"), value_restriction: None, entry_specific: None }) }), fields: [Value] }] })

                use std::fmt::Write;
                let mut output = String::new();
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
                    println!("{}", output);
                }
            }
        }
    }
}

/** */
async fn subscribe_data(client: &mut KuksaClient, target: Vec<&str>) {
    println!("start subscribe");
    let subscription_nbr = 1;
    match client.subscribe(target).await {
        Ok(mut subscription) => {
            println!("subscribe OK");
            tokio::spawn(async move {
                let sub_disp = format!("[{subscription_nbr}]");
                let sub_disp_pad = " ".repeat(sub_disp.len());
                let sub_disp_color = format!("{}", Color::White.dimmed().paint(&sub_disp));

                loop {
                    match subscription.message().await {
                        Ok(subscribe_resp) => {
                            subscribe_action(subscribe_resp.clone());
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
}
