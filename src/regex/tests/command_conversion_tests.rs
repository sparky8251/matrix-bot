mod no_capture {
    use crate::regex::*;
    #[test]
    fn standard() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22km"))
    }
    #[test]
    fn float() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2km"))
    }
    #[test]
    fn space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 km"))
    }
    #[test]
    fn float_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2 km"))
    }
    #[test]
    fn forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22km/h"))
    }
    #[test]
    fn forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 km/h"))
    }
    #[test]
    fn float_forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2km/h"))
    }
    #[test]
    fn float_forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22.2 km/h"))
    }
    #[test]
    fn single_digit() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6ft"))
    }
    #[test]
    fn single_digit_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6 ft"))
    }
    #[test]
    fn single_digit_forwardslash() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6km/h"))
    }
    #[test]
    fn single_digit_forwardslash_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 6 km/h"))
    }
    #[test]
    fn single_letter() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22c"))
    }
    #[test]
    fn single_letter_space() {
        assert_eq!(true, SINGLE_UNIT_CONVERSION.is_match("!convert 22 c"))
    }
}

mod capture {
    use crate::regex::*;
    #[test]
    fn standard() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn float() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn float_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn forwardslash() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn forwardslash_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn forwardslash_float() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn forwardslash_float_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22.2 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.2, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_digit() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_digit_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6 km") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_digit_forwardslash() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_digit_forwardslash_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 6 km/h") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((6.0, "km/h"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_letter() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22c") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "c"), (quantity, unit.as_str()))
    }
    #[test]
    fn single_letter_space() {
        let (mut quantity, mut unit) = (String::new(), String::new());
        for cap in SINGLE_UNIT_CONVERSION.captures_iter("!convert 22 c") {
            quantity = cap[1].to_string();
            unit = cap[2].to_string();
        }
        let quantity = quantity.parse::<f64>().unwrap();
        assert_eq!((22.0, "c"), (quantity, unit.as_str()))
    }
}
