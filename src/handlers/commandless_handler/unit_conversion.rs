use crate::regex::{CODE_TAG, PRE_TAG, UNIT_CONVERSION};
use crate::Storage;

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

pub async fn unit_conversion(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
) -> Result<()> {
    let mut conversions = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = CODE_TAG.replace_all(&v, "");
            trace!("Cleaned text after code tag is {:?}", clean_text);
            let clean_text = PRE_TAG.replace_all(&clean_text, "");
            trace!("Cleaned text after pre tag is {:?}", clean_text);
            if UNIT_CONVERSION.is_match(&clean_text) {
                for cap in UNIT_CONVERSION.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    conversions.push((cap[1].to_string(), cap[2].to_string()))
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return Ok(());
            }
        }
        None => {
            for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
                conversions.push((cap[1].to_string(), cap[2].to_string()))
            }
        }
    }
    let conversions = conversions;
    let mut result = String::new();
    for conversion in conversions {
        let (quantity, unit) = conversion;
        let quantity = match quantity.parse::<f64>() {
            Ok(v) => v,
            Err(e) => {
                error!("Quantity unable to be parsed. Error is {:?}", e);
                return Ok(());
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
                                    let converted_quantity = unit_value.get::<$to_ty>();
                                    let from_string = format!("{:.2}{} => ", $quantity, $from_str);
                                    let to_string = format!("{:.2}{}\n", converted_quantity, $to_str);
                                    result.push_str(from_string.as_str());
                                    result.push_str(to_string.as_str());
                                }
                            )*
                        )*
                        _ => {
                            $($default_code)*
                        }
                    }
                }
            }
        convert_unit!(unit.as_str(), quantity,
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
                ("°c", "°f", degree_celsius, degree_fahrenheit),
                ("f", "c", degree_fahrenheit, degree_celsius),
                ("°f", "°c", degree_fahrenheit, degree_celsius),
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
                unit.trim().to_lowercase());
            }
        )
    }

    if result.trim().to_string() != "" {
        let response = client
            .request(create_message_event::Request {
                room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                event_type: EventType::RoomMessage,
                txn_id: storage.next_txn_id(),
                data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                    body: result.trim().to_string(),
                    relates_to: None,
                })),
            })
            .await;
        match response {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("{:?}", e);
                return Ok(());
            }
        }
    } else {
        trace!("Nothing left after trimming result, doing nothing");
        return Ok(());
    }
}
