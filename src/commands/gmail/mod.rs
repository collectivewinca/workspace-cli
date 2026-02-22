pub mod types;
pub mod list;
pub mod get;
pub mod send;
pub mod delete;
pub mod trash;
pub mod labels;
pub mod modify;
pub mod filters;

// Re-export main types and functions for convenience
pub use types::{
    Message,
    MessagePayload,
    MessagePart,
    MessageBody,
    Header,
    ListMessagesResponse,
    MessageRef,
    SendMessageRequest,
};

pub use list::{list_messages, ListParams};
pub use get::{get_message, extract_body, get_header, list_attachments, get_attachment, download_attachment, AttachmentInfo, AttachmentData};
pub use send::{send_message, create_draft, ComposeParams, Attachment, load_attachment, extract_forward_metadata, build_forward_body, ForwardMetadata};
pub use delete::{delete_message, batch_delete};
pub use trash::{trash_message, untrash_message};
pub use labels::{list_labels, get_label, modify_labels, add_labels, remove_labels, Label, ListLabelsResponse};
pub use modify::{mark_read, mark_unread, star_message, unstar_message, archive_message, move_to_inbox};
pub use filters::{Filter, FilterCriteria, FilterAction, ListFiltersResponse, list_filters, get_filter, create_filter, delete_filter, build_filter};
