use std::fs::File;
use std::io::{ErrorKind, Read};

pub(super) fn load_access_token() -> String {
    match std::env::var("CI_TEST_TOKEN") {
        Ok(v) => v,
        Err(_) => {
            let mut file = match File::open(".access_token") {
                Ok(v) => v,
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => panic!("Unable to find file .access_token"),

                    ErrorKind::PermissionDenied => {
                        panic!("Permission denied when opening file .access_token")
                    }

                    _ => panic!("Unable to open file due to unexpected error {:?}", e),
                },
            };
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                panic!("Unable to read file contents due to error {:?}", e);
            }
            contents
        }
    }
}
