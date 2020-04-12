use std::process;

use super::helpers::do_nothing;
use crate::session::SavedSession;

use anyhow::Result;
use log::{debug, error};
use nom::character::is_alphabetic;
use nom::{named, tag_no_case, take_till};
use ruma_client::{api::r0::message::create_message_event, HttpsClient};
use ruma_events::{
    room::message::{MessageEventContent, TextMessageEventContent},
    EventType,
};
use ruma_identifiers::RoomId;
use uom::si::f64::*;
use uom::si::length::{centimeter, foot, inch, kilometer, meter, mile};
use uom::si::mass::{kilogram, pound};
use uom::si::thermodynamic_temperature::{degree_celsius, degree_fahrenheit};
use uom::si::velocity::{kilometer_per_hour, mile_per_hour};

pub(super) async fn unit_conversion(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if text.relates_to == None && text.formatted_body == None {
        named!(strip_tag, tag_no_case!("!convert "));
        let message = match strip_tag(&text.body.as_bytes()) {
            Ok(v) => v.0,
            Err(e) => {
                error!("{:?}", e);
                process::exit(52)
            }
        };
        named!(split_parts, take_till!(is_alphabetic));
        let (quantity, unit) = match split_parts(message) {
            Ok(v) => (Some(v.1.to_vec()), Some(v.0.to_vec())),
            Err(e) => {
                error!("{:?}", e);
                (None, None)
            }
        };

        let quantity = match quantity {
            Some(q) => match String::from_utf8(q) {
                Ok(v) => match v.trim().parse::<f64>() {
                    Ok(n) => Some(n),
                    Err(e) => {
                        error!("{:?}", e);
                        None
                    }
                },
                Err(e) => {
                    error!("{:?}", e);
                    None
                }
            },
            None => {
                error!("No quantity provided!");
                None
            }
        };

        let unit = match unit {
            Some(u) => match String::from_utf8(u) {
                Ok(t) => Some(t),
                Err(e) => {
                    error!("{:?}", e);
                    None
                }
            },
            None => {
                error!("No unit provided!");
                None
            }
        };

        match quantity {
            Some(quantity) => match unit {
                Some(unit) => {
                    match unit.trim().to_lowercase().as_ref() {
                        // length
                        "cm" => {
                            let length = Length::new::<centimeter>(quantity);
                            send_converted_value(
                                length.get::<inch>(),
                                "in",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "m" => {
                            let length = Length::new::<meter>(quantity);
                            send_converted_value(
                                length.get::<foot>(),
                                "ft",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "km" => {
                            let length = Length::new::<kilometer>(quantity);
                            send_converted_value(
                                length.get::<mile>(),
                                "mi",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "in" => {
                            let length = Length::new::<inch>(quantity);
                            send_converted_value(
                                length.get::<centimeter>(),
                                "cm",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "ft" => {
                            let length = Length::new::<foot>(quantity);
                            send_converted_value(
                                length.get::<meter>(),
                                "m",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "mi" => {
                            let length = Length::new::<mile>(quantity);
                            send_converted_value(
                                length.get::<kilometer>(),
                                "km",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        // temperature
                        "c" => {
                            let temperature =
                                ThermodynamicTemperature::new::<degree_celsius>(quantity);
                            send_converted_value(
                                temperature.get::<degree_fahrenheit>(),
                                "f",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "f" => {
                            let temperature =
                                ThermodynamicTemperature::new::<degree_fahrenheit>(quantity);
                            send_converted_value(
                                temperature.get::<degree_celsius>(),
                                "c",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        // weight
                        "kg" => {
                            let mass = Mass::new::<kilogram>(quantity);
                            send_converted_value(
                                mass.get::<pound>(),
                                "lbs",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "lbs" => {
                            let mass = Mass::new::<pound>(quantity);
                            send_converted_value(
                                mass.get::<kilogram>(),
                                "kg",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        // velocity
                        "km/h" | "kmh" | "kph" | "kmph" => {
                            let velocity = Velocity::new::<kilometer_per_hour>(quantity);
                            send_converted_value(
                                velocity.get::<mile_per_hour>(),
                                "mph",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        "mph" => {
                            let velocity = Velocity::new::<mile_per_hour>(quantity);
                            send_converted_value(
                                velocity.get::<kilometer_per_hour>(),
                                "km/h",
                                room_id,
                                client,
                                session,
                            )
                            .await
                        }
                        _ => {
                            debug!(
                                "Attempted unknown conversion for unit {}",
                                unit
                            );
                            do_nothing().await
                        }
                    }
                }
                None => do_nothing().await,
            },
            None => do_nothing().await,
        }
    } else {
        do_nothing().await
    }
}

async fn send_converted_value(
    converted_quantity: f64,
    converted_unit: &str,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    let response = client
        .request(create_message_event::Request {
            room_id: room_id.clone(), // INVESTIGATE: Does this really need to be cloned?
            event_type: EventType::RoomMessage,
            txn_id: session.next_txn_id(),
            data: MessageEventContent::Text(TextMessageEventContent {
                body: format!("{:.2}{}", converted_quantity, converted_unit),
                format: None,
                formatted_body: None,
                relates_to: None,
            }),
        })
        .await;
    match response {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{:?}", e);
            process::exit(48)
        }
    }
}
