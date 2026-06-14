use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ChatPermissions: i64 {
        const SEND_MESSAGES = 1 << 0;
        const ADD_MEMBERS = 1 << 1;
        const PIN_MESSAGES = 1 << 2;
        const SEND_VIDEO_MESSAGES = 1 << 3;
        const SEND_VOICE_MESSAGES = 1 << 4;
        const SEND_FILES = 1 << 5;
        const CREATE_POLLS = 1 << 6;
        const CHANGE_GROUP_INFO = 1 << 7;

        const DELETE_MESSAGES = 1 << 8;
        const MANAGE_MEMBERS = 1 << 9;
        const MANAGE_INVITE_LINKS = 1 << 10;
        const ADD_ADMINS = 1 << 11;

        const DM_CHAT = Self::SEND_MESSAGES.bits() | Self::SEND_VIDEO_MESSAGES.bits()
            | Self::SEND_VOICE_MESSAGES.bits() | Self::DELETE_MESSAGES.bits()
            | Self::SEND_FILES.bits() | Self::PIN_MESSAGES.bits();

        const DEFAULT_GROUP_DM_MEMBER = Self::SEND_MESSAGES.bits() | Self::SEND_VIDEO_MESSAGES.bits()
            | Self::SEND_VOICE_MESSAGES.bits() | Self::ADD_MEMBERS.bits()
            | Self::SEND_FILES.bits() | Self::PIN_MESSAGES.bits()
            | Self::CREATE_POLLS.bits() | Self::CHANGE_GROUP_INFO.bits();
    }
}

impl std::fmt::Display for ChatPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits();
        if bits == Self::SEND_MESSAGES.bits() {
            write!(f, "SEND_MESSAGES")
        } else if bits == Self::ADD_MEMBERS.bits() {
            write!(f, "ADD_MEMBERS")
        } else if bits == Self::PIN_MESSAGES.bits() {
            write!(f, "PIN_MESSAGES")
        } else if bits == Self::SEND_VIDEO_MESSAGES.bits() {
            write!(f, "SEND_VIDEO_MESSAGES")
        } else if bits == Self::SEND_VOICE_MESSAGES.bits() {
            write!(f, "SEND_VOICE_MESSAGES")
        } else if bits == Self::SEND_FILES.bits() {
            write!(f, "SEND_FILES")
        } else if bits == Self::CREATE_POLLS.bits() {
            write!(f, "CREATE_POLLS")
        } else if bits == Self::CHANGE_GROUP_INFO.bits() {
            write!(f, "CHANGE_GROUP_INFO")
        } else if bits == Self::DELETE_MESSAGES.bits() {
            write!(f, "DELETE_MESSAGES")
        } else if bits == Self::MANAGE_MEMBERS.bits() {
            write!(f, "MANAGE_MEMBERS")
        } else if bits == Self::MANAGE_INVITE_LINKS.bits() {
            write!(f, "MANAGE_INVITE_LINKS")
        } else if bits == Self::ADD_ADMINS.bits() {
            write!(f, "ADD_ADMINS")
        } else {
            write!(f, "{:?}", self)
        }
    }
}
