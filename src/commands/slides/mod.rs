pub mod types;
pub mod get;
pub mod create;
pub mod update;

// Re-export commonly used types
pub use types::{
    Presentation,
    Page,
    PageElement,
    Shape,
    Table,
    TextContent,
    TextElement,
};

// Re-export key functions
pub use get::{
    get_presentation,
    get_page,
    extract_all_text,
    extract_page_text,
    get_slide_text,
    get_summary,
};

// Re-export create functions
pub use create::create_presentation;

// Re-export update functions
pub use update::{
    add_slide,
    add_text,
    BatchUpdateResponse,
    SLIDE_LAYOUTS,
};
