mod no_capture {
    use crate::regex::*;

    #[test]
    fn single() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22km from me"))
    }
    #[test]
    fn single_forwardslash() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are going 22km/h right now")
        )
    }
    #[test]
    fn single_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22.2km from me"))
    }
    #[test]
    fn single_float_forwardslash() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are going 22.2km/h right now")
        )
    }
    #[test]
    fn single_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22 km from me"))
    }
    #[test]
    fn single_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("you are 22.2 km from me"))
    }
    #[test]
    fn single_at_end() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22km"))
    }
    #[test]
    fn single_at_end_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22.2km"))
    }
    #[test]
    fn single_at_end_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22 km"))
    }
    #[test]
    fn single_at_end_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("its 22.2 km"))
    }
    #[test]
    fn single_at_beginning() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22km away from me"))
    }
    #[test]
    fn single_at_beginning_float() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22.2km away from me"))
    }
    #[test]
    fn single_at_beginning_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22 km away from me"))
    }
    #[test]
    fn single_at_beginning_float_space() {
        assert_eq!(true, UNIT_CONVERSION.is_match("22.2 km away from me"))
    }
    #[test]
    fn multiple() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are 22km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_float() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2km from me. at 22.2kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_float_mixed() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22 km from me. at 22 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_float_multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22.2 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_float_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22.2kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_mixed_float_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION
                .is_match("you are 22.2 km from me. at 22kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("you are 22km from me. at 22 kmph itll be 1 hour to get here")
        )
    }
    #[test]
    fn multiple_at_both_ends() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22km away from me. itll take 1 hour at 22kmph")
        )
    }
    #[test]
    fn multiple_at_both_ends_multiple_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22 km away from me. itll take 1 hour at 22 kmph")
        )
    }
    #[test]
    fn multiple_at_both_ends_mixed_space() {
        assert_eq!(
            true,
            UNIT_CONVERSION.is_match("22 km away from me. itll take 1 hour at 22kmph")
        )
    }
}

mod capture {
    use crate::regex::*;
    use ruma::UserId;

    #[test]
    fn capture_formatted_username_for_ban_handler() {
        let input_string = "The other one had the mention in the nice tag like when I do this <a href=\"https://matrix.to/#/@sparky:matrix.possumlodge.me\">sparky</a>";
        let actual_username = UserId::parse("@sparky:matrix.possumlodge.me").unwrap();

        let captured_username = FORMATTED_USERNAME
            .captures_iter(input_string)
            .next()
            .unwrap();
        let captured_username = UserId::parse(&captured_username[0]).unwrap();

        assert_eq!(actual_username, captured_username);

        let input_string =
            "!ban <a href=\"https://matrix.to/#/@danoneil:matrix.org\">danoneil</a> Spam";
        let actual_username = UserId::parse("@danoneil:matrix.org").unwrap();

        let captured_username = FORMATTED_USERNAME
            .captures_iter(input_string)
            .next()
            .unwrap();
        let captured_username = UserId::parse(&captured_username[0]).unwrap();

        assert_eq!(actual_username, captured_username);
    }
}
