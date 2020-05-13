use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref NO_BANG: Regex = Regex::new(r"(^[^!])").unwrap();
}
lazy_static! {
    pub static ref SINGLE_UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        ^!convert                                   # The tag from line start
        (?:[[\t\v\f\r ][:blank:][^\n]])*?           # Any amount of whitespace
        ([[:digit:]]+(?:\.[[:digit:]]+)?)           # The number to convert, will only allow 1 period for floating points (captured)
        (?:[[\t\v\f\r ][:blank:][^\n]])*?           # Any amount of white space
        (°{0,1}[[:alpha:]]+/{0,1}[[:alpha:]]{0,})   # The unit to convert from including potential / (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    pub static ref UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        ([[:digit:]]+(?:\.[[:digit:]]+)?)           # The number to convert, will only allow 1 period for floating points (captured)
        (?:[[[\t\v\f\r ]][:blank:]])*?              # Any amount of white space
        (°{0,1}[[:alpha:]]+/{0,1}[[:alpha:]]{0,})   # The unit to convert from including potential / (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    pub static ref CODE_TAG: Regex = Regex::new(r"(?s)(<code>.*</code>)*").unwrap();
}
lazy_static! {
    pub static ref PRE_TAG: Regex = Regex::new(r"(?s)(<pre>.*</pre>)*").unwrap();
}
lazy_static! {
    // TODO: fix regex so it will match '!roll 22 90' just as well as '!roll 22' but never '!roll 22 90 120'
    pub static ref ROLL: Regex = Regex::new(r"(^!roll\s*?[[:digit]][\s+?[[:digit:]]]x{0,1}?)").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bang_test_bang_at_front() {
        assert_eq!(false, NO_BANG.is_match("!convert"));
    }
    #[test]
    fn no_bang_test_bang_at_end() {
        assert_eq!(true, NO_BANG.is_match("convert!"));
    }
    #[test]
    fn no_bang_test_bang_after_starting_space() {
        assert_eq!(true, NO_BANG.is_match(" !convert"));
    }
    #[test]
    fn no_bang_test_no_bang() {
        assert_eq!(true, NO_BANG.is_match("hoogaboogawooga"));
    }
    #[test]
    fn single_unit_conversion_test_match() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22km"))
    }
    #[test]
    fn single_unit_conversion_test_match_float() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2km"))
    }
    #[test]
    fn single_unit_conversion_test_match_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 km"))
    }
    #[test]
    fn single_unit_conversion_test_match_float_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2 km"))
    }
    #[test]
    fn single_unit_conversion_test_match_forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_float_forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_float_forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2 km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_digit() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6ft"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_digit_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6 ft"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_digit_forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_digit_forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6 km/h"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_letter() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22c"))
    }
    #[test]
    fn single_unit_conversion_test_match_single_letter_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 c"))
    }
    #[test]
    fn single_unit_conversion_test_capture() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_float() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_float_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_forwardslash() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_forwardslash_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_forwardslash_float() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_forwardslash_float_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_digit() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_digit_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_digit_forwardslash() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_digit_forwardslash_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_letter() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22c") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "c"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_unit_conversion_test_capture_single_letter_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 c") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "c"), (quantity, unit.as_str()))
    }
    #[test]
    fn unit_conversion_test_match_single() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22km from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_forwardslash() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are going 22km/h right now")
        )
    }
    #[test]
    fn unit_conversion_test_match_single_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22.2km from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_float_forwardslash() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are going 22.2km/h right now")
        )
    }
    #[test]
    fn unit_conversion_test_match_single_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22 km from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22.2 km from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_end() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22km"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_end_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22.2km"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_end_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22 km"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_end_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22.2 km"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_beginning() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22km away from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_beginning_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22.2km away from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_beginning_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22 km away from me"))
    }
    #[test]
    fn unit_conversion_test_match_single_at_beginning_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22.2 km away from me"))
    }
    #[test]
    fn unit_conversion_test_match_multiple() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are 22km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_float() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2km from me. at 22.2kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_float_mixed() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22 km from me. at 22 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_float_multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22.2 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_float_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22.2kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_mixed_float_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are 22km from me. at 22 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_at_both_ends() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22km away from me. itll take 1 hour at 22kmph")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_at_both_ends_multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22 km away from me. itll take 1 hour at 22 kmph")
        )
    }
    #[test]
    fn unit_conversion_test_match_multiple_at_both_ends_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22 km away from me. itll take 1 hour at 22kmph")
        )
    }
}
