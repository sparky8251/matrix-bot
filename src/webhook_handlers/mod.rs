mod github;
mod message;

use github::event as event_fn;
use message::message as message_fn;
use rocket::Rocket;

pub fn register_handlers(rocket: Rocket) -> Rocket {
    rocket
        .mount("/message", routes![message_fn])
        .mount("/event", routes![event_fn])
}
