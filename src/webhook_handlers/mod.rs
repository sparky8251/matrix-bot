mod message;

use message::message as message_fn;
use rocket::Rocket;

pub fn register_handlers(rocket: Rocket) -> Rocket {
    rocket.mount("/message", routes![message_fn])
}
