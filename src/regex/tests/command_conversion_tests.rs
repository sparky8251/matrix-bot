mod no_capture {
    use crate::regex::*;
    #[test]
    fn standard() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22km"))
    }
    #[test]
    fn float() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22.2km"))
    }
    #[test]
    fn space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22 km"))
    }
    #[test]
    fn float_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22.2 km"))
    }
    #[test]
    fn forwardslash() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22km/h"))
    }
    #[test]
    fn forwardslash_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22 km/h"))
    }
    #[test]
    fn float_forwardslash() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22.2km/h"))
    }
    #[test]
    fn float_forwardslash_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22.2 km/h"))
    }
    #[test]
    fn single_digit() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 6ft"))
    }
    #[test]
    fn single_digit_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 6 ft"))
    }
    #[test]
    fn single_digit_forwardslash() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 6km/h"))
    }
    #[test]
    fn single_digit_forwardslash_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 6 km/h"))
    }
    #[test]
    fn single_letter() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22c"))
    }
    #[test]
    fn single_letter_space() {
        assert_eq!(true, UNIT_CONVERSION_COMMAND.is_match("!convert 22 c"))
    }
}