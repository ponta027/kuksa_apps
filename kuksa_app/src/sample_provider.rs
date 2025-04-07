use kuksa::*;
mod kuksa_util;

use std::{thread, time};
struct DisplayDatapoint(proto::v1::Datapoint);
use databroker_proto::kuksa::val as proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KuksaClient::new(kuksa_common::to_uri(String::from("127.0.0.1:55555"))?);
    let mut speed: u32 = 0;

    loop {
        kuksa_util::handle_publish_command("Vehicle.Speed", &speed.to_string(), &mut client)
            .await?;
        thread::sleep(time::Duration::from_secs(10));

        let is_open_val = kuksa_util::handle_gettarget_command(
            "Vehicle.Cabin.Door.Row1.PassengerSide.IsOpen",
            &mut client,
        )
        .await?;
        println!("{:?}", is_open_val.0.value.unwrap());
        speed += 1;
    }

    Ok(())
}
