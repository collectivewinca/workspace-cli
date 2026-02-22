use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Event, EventDateTime, Attendee, EventReminders, ReminderOverride};

pub struct CreateEventParams {
    pub calendar_id: String,
    pub summary: String,
    pub start: String,  // RFC3339 or YYYY-MM-DD
    pub end: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub attendees: Option<Vec<String>>,
    pub time_zone: Option<String>,
    /// Recurrence rule (e.g., "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR")
    pub recurrence: Option<String>,
    /// Reminders (e.g., "email:30,popup:10" for email 30 min before and popup 10 min before)
    pub reminders: Option<String>,
}

pub async fn create_event(client: &ApiClient, params: CreateEventParams) -> Result<Event> {
    let is_all_day = !params.start.contains('T');

    let start = if is_all_day {
        EventDateTime {
            date: Some(params.start.clone()),
            date_time: None,
            time_zone: None,  // All-day events should not have timezone
        }
    } else {
        EventDateTime {
            date: None,
            date_time: Some(params.start.clone()),
            time_zone: params.time_zone.clone(),
        }
    };

    let end = if is_all_day {
        EventDateTime {
            date: Some(params.end.clone()),
            date_time: None,
            time_zone: None,  // All-day events should not have timezone
        }
    } else {
        EventDateTime {
            date: None,
            date_time: Some(params.end.clone()),
            time_zone: params.time_zone.clone(),
        }
    };

    let attendees: Vec<Attendee> = params.attendees
        .unwrap_or_default()
        .into_iter()
        .map(|email| Attendee {
            email,
            optional: false,
            response_status: None,
        })
        .collect();

    // Parse recurrence rule
    let recurrence = params.recurrence.map(|r| vec![r]);

    // Parse reminders (format: "email:30,popup:10")
    let reminders = params.reminders.map(|r| {
        let overrides: Vec<ReminderOverride> = r.split(',')
            .filter_map(|part| {
                let mut parts = part.trim().split(':');
                let method = parts.next()?;
                let minutes = parts.next()?.parse().ok()?;
                Some(ReminderOverride {
                    method: method.to_string(),
                    minutes,
                })
            })
            .collect();
        EventReminders {
            use_default: false,
            overrides,
        }
    });

    let event = Event {
        id: None,
        summary: Some(params.summary),
        description: params.description,
        location: params.location,
        start: Some(start),
        end: Some(end),
        status: None,
        attendees,
        organizer: None,
        html_link: None,
        created: None,
        updated: None,
        recurrence,
        reminders,
    };

    let path = format!("/calendars/{}/events", urlencoding::encode(&params.calendar_id));
    client.post(&path, &event).await
}
