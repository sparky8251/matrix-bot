use std::fmt;

use log::{debug, error, trace};
use uom::si::f64::*;
use uom::si::length::{centimeter, foot, inch, kilometer, meter, mile};
use uom::si::mass::{kilogram, pound};
use uom::si::thermodynamic_temperature::{degree_celsius, degree_fahrenheit};
use uom::si::velocity::{kilometer_per_hour, mile_per_hour};

#[derive(Debug)]
pub struct ConvertedUnit {
    from: String,
    to: String,
}

impl fmt::Display for ConvertedUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.from, self.to)
    }
}

pub fn convert_unit(conversions: Vec<(String, String)>) -> Option<Vec<ConvertedUnit>> {
    let mut result = Vec::new();
    let mut working_data = Vec::new();

    for conversion in conversions {
        let (quantity, unit) = conversion;
        match quantity.parse::<f64>() {
            Ok(v) => working_data.push((unit, v)),
            Err(e) => {
                error!(
                    "Quantity unable to be parsed. Error is {:?}, quantity is {:?}",
                    e, quantity
                );
            }
        }
    }

    if working_data.len() == 0 {
        trace!("No units to convert after parsing quanitity to f64");
        return None;
    }

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
                            let from = format!("{:.2}{}", $quantity, $from_str);
                            let to = format!("{:.2}{}", converted_quantity, $to_str);
                            result.push(ConvertedUnit {
                                from,
                                to
                            });
                        }
                    )*
                )*
                _ => {
                    $($default_code)*
                }
            }
        }
    }
    for (unit, quantity) in working_data {
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
        );
    }

    if result.len() > 0 {
        Some(result)
    } else {
        trace!("No units converted");
        None
    }
}
