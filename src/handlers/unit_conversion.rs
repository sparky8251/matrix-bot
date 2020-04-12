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

        macro_rules! convert_unit {
            (
                $unit:expr, $quantity:expr,
                $(
                    $unit_ty:ident {
                        $( ( $from_str:expr, $to_str:expr, $from_ty:ty, $to_ty:ty ) ),*
                        $(,)?
                    }
                )*
                _ => {
                    $($default_code:tt)*
                }
            ) => {
                match $unit {
                    $(
                        $(
                            $from_str => {
                                let unit_value = $unit_ty::new::<$from_ty>($quantity);
                                send_converted_value(
                                    unit_value.get::<$to_ty>(),
                                    $to_str,
                                    room_id,
                                    client,
                                    session,
                                )
                                .await
                            }
                        )*
                    )*
                    _ => {
                        $($default_code)*
                    }
                }
            }
        }

        match quantity {
            Some(quantity) => match unit {
                Some(unit) => {
                    let unit = unit.trim().to_lowercase();
                    let unit = unit.as_str();
                    convert_unit!(unit, quantity,
                        Length {
                            ("cm", "in", centimeter, inch),
                            ("m", "ft", meter, foot),
                            ("km", "mi", kilometer, mile),
                            ("in", "cm", inch, centimeter),
                            ("ft", "m", foot, meter),
                            ("mi", "km", mile, kilometer),
                        }
                        ThermodynamicTemperature {
                            ("c", "f", degree_celsius, degree_fahrenheit),
                            ("f", "c", degree_fahrenheit, degree_celsius),
                        }
                        Mass {
                            ("kg", "lbs", kilogram, pound),
                            ("lbs", "kg", pound, kilogram),
                        }
                        Velocity {
                            ("km/h", "mph", kilometer_per_hour, mile_per_hour),
                            ("kmh", "mph", kilometer_per_hour, mile_per_hour),
                            ("kph", "mph", kilometer_per_hour, mile_per_hour),
                            ("kmph", "mph", kilometer_per_hour, mile_per_hour),
                            ("mph", "km/h", mile_per_hour, kilometer_per_hour),
                        }
                        _ => {
                            debug!(
                                "Attempted unknown conversion for unit {}",
                                unit.trim().to_lowercase()
                            );
                            do_nothing().await
                        }
                    )
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
