use super::helpers::do_nothing;
use crate::regex::SINGLE_UNIT_CONVERSION;
use crate::session::SavedSession;

use anyhow::Result;
use log::{debug, error, trace};
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};
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
) -> Result<(), anyhow::Error> {
    if text.relates_to == None && text.formatted_body == None {
        let mut quantity = "".to_string();
        let mut unit = "".to_string();
        for cap in SINGLE_UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
            trace!(
                "Capture: {:?} => Quantity: {:?} => Unit: {:?}",
                &cap[0],
                quantity,
                unit
            );
            break; // Should not be possible to match multiple times, but just in case we only want the first match
        }
        let quantity = match quantity.parse::<f64>() {
            Ok(v) => v,
            Err(e) => {
                error!("Quantity unable to be parsed. Error is {:?}", e);
                return Ok(());
            }
        };
        let unit = unit;

        trace!("{:?} => {:?}", quantity, unit);

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

        match convert_unit!(unit.as_str(), quantity,
            Length {
                ("cm", "in", centimeter, inch),
                ("m", "ft", meter, foot),
                ("km", "mi", kilometer, mile),
                ("in", "cm", inch, centimeter),
                ("ft", "m", foot, meter),
                ("mi", "km", mile, kilometer),
                ("mile", "km", mile, kilometer),
                ("miles", "km", mile, kilometer),
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
                    "Attempted unknown conversion for unit {:?}",
                    unit.trim().to_lowercase()
                );
                do_nothing().await
            }
        ) {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
    do_nothing().await
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
            data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: format!("{:.2}{}", converted_quantity, converted_unit),
                relates_to: None,
            })),
        })
        .await;
    match response {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{:?}", e);
            Ok(())
        }
    }
}
