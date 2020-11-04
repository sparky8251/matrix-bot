mod github;
mod message;

use github::pull_request as pull_request_fn;
use message::message as message_fn;
use rocket::Rocket;

pub fn register_handlers(rocket: Rocket) -> Rocket {
    rocket
        .mount("/message", routes![message_fn])
        .mount("/pull_request", routes![pull_request_fn])
}
