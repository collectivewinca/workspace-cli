pub mod types;
pub mod list;
pub mod create;
pub mod update;
pub mod delete;

// Re-export commonly used types and functions
pub use types::{
    Event,
    EventDateTime,
    Attendee,
    Organizer,
    EventList,
    CalendarList,
    CalendarListEntry,
    EventReminders,
    ReminderOverride,
};

pub use list::{
    list_events,
    list_calendars,
    get_event,
    ListEventsParams,
    query_free_busy,
    FreeBusyParams,
    FreeBusyResponse,
    CalendarFreeBusy,
    TimePeriod,
};

pub use create::{
    create_event,
    CreateEventParams,
};

pub use update::{
    update_event,
    UpdateEventParams,
};

pub use delete::delete_event;
