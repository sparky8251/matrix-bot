//! Handler for the unit conversion command

use crate::helpers::convert_unit;
use crate::helpers::MatrixNoticeResponse;
use crate::messages::{MatrixMessage, MatrixMessageType};
use crate::regex::UNIT_CONVERSION;
use ruma::{
    events::room::message::{Relation, TextMessageEventContent},
    RoomId,
};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

/// Command based unit conversion handler that will parse, generate a response body, and send it
pub(super) async fn unit_conversion_handler(
    text: &TextMessageEventContent,
    relates_to: Option<&Relation>,
    room_id: &RoomId,
    send: &mut Sender<MatrixMessage>,
) {
    if relates_to.is_none() && text.formatted.is_none() {
        let conversions: Vec<(String, String)> = UNIT_CONVERSION.captures_iter(&text.body.to_lowercase())
            .map(|cap| (cap[1].to_string(), cap[2].to_string()))
            .collect();
        let result = match convert_unit(conversions) {
            Some(v) => v,
            None => {
                debug!("No convertable units found. No reply will be constructed.");
                return;
            }
        };
        let mut response = MatrixNoticeResponse::default();
        response.set_unit_conversions(result);
        if send
            .send(MatrixMessage {
                room_id: Some(room_id.to_owned()),
                message: MatrixMessageType::Notice(response.to_string()),
            })
            .await
            .is_err()
        {
            error!("Channel closed. Unable to send message.");
        }
    }
}
