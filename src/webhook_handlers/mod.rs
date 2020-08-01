mod message;

use message::*;
use rocket::Rocket;

pub fn register_handlers(rocket: Rocket) -> Rocket {
    rocket.mount("/message", routes![message])
}
