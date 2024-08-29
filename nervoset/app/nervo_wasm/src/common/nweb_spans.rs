use tracing::{info_span, Span};

const CHAT: &str = "nweb-chat";
const SEND_MSG: &str = "nweb-send-msg";

pub fn nweb_chat_span() -> Span {
    info_span!(CHAT)
}

pub fn nweb_send_msg_span() -> Span {
    info_span!(SEND_MSG)
}
