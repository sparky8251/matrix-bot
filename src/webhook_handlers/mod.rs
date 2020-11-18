mod github;
mod message;

use github::ping as ping_fn;
use message::message as message_fn;
use rocket::Rocket;

pub fn register_handlers(rocket: Rocket) -> Rocket {
    rocket
        .mount("/message", routes![message_fn])
        .mount("/ping", routes![ping_fn])
}
