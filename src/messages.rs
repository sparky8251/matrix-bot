use ruma::{RoomId, UserId};

#[derive(Debug)]
pub struct MatrixMessage {
    pub room_id: Box<RoomId>,
    pub message: MatrixMessageType,
    // pub resp: Responder<MatrixMessageResult>,
}

#[derive(Debug)]
pub enum MatrixMessageType {
    Invite(MatrixInviteMessage),
    Text(String),
    Notice(String),
    FormattedText(MatrixFormattedMessage),
    FormattedNotice(MatrixFormattedMessage),
}

#[derive(Debug)]
pub enum MatrixInviteType {
    Accept,
    Reject,
}

#[derive(Debug)]
pub struct MatrixFormattedMessage {
    pub plain_text: String,
    pub formatted_text: Option<String>,
}

#[derive(Debug)]
pub struct MatrixInviteMessage {
    pub kind: MatrixInviteType,
    pub sender: Box<UserId>,
}

// #[derive(Debug)]
// pub enum MatrixMessageResult {
//     Sent,
//     FailedToSend,
// }

// pub type Responder<T> = oneshot::Sender<T>;
