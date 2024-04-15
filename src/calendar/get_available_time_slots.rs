use crate::routes::get_service_duration_by_id;
use actix_web::web;
use chrono::{DateTime, Duration, Utc};
use google_calendar3::{
    api::{FreeBusyRequest, FreeBusyRequestItem},
    hyper, hyper_rustls, oauth2,
    oauth2::ServiceAccountAuthenticator,
    CalendarHub,
};
use log::{error, info};
use sqlx::PgPool;
use std::default::Default;
use std::error::Error;

const SERVICE_ACCOUNT_KEY_FILE: &str = "calendar-json.googleapis.com.json";
const CALENDAR_ID: &str = "bbbb.iam.gserviceaccount.com";

pub async fn get_available_time_slots(
    db_pool: web::Data<PgPool>,
    service_id: i64,
) -> Result<Vec<String>, Box<dyn Error>> {
    let service_duration_minutes = get_service_duration_by_id(db_pool, service_id).await?;
    let service_duration = Duration::minutes(service_duration_minutes.into());

    let sa_key = oauth2::read_service_account_key(SERVICE_ACCOUNT_KEY_FILE).await?;
    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;
    let hub = CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    let request = FreeBusyRequest {
        time_min: Some(Utc::now().into()),
        time_max: Some((Utc::now() + Duration::days(7)).into()),
        items: Some(vec![FreeBusyRequestItem {
            id: Some(CALENDAR_ID.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let (response_status, response_data) = hub.freebusy().query(request).doit().await?;
    if !response_status.status().is_success() {
        error!("Failed to fetch freebusy info: {:?}", response_status);
        return Err("Failed to fetch freebusy information".into());
    }

    let freebusy_response = response_data;

    let mut available_slots: Vec<String> = Vec::new();
    if let Some(calendars) = freebusy_response.calendars {
        for (_, calendar) in calendars {
            if let Some(busy_intervals) = calendar.busy {
                let mut last_end_time: DateTime<Utc> = Utc::now();

                for busy_interval in busy_intervals {
                    let start: DateTime<Utc> = busy_interval.start.unwrap_or(last_end_time);
                    let end: DateTime<Utc> = busy_interval.end.unwrap_or(start);

                    let diff = start - last_end_time;

                    if diff >= service_duration {
                        let slot_start = last_end_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                        let slot_end = start.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                        available_slots.push(format!("{} - {}", slot_start, slot_end));
                    }
                    last_end_time = end;
                }
            }
        }
    }

    info!("Available slots: {:?}", available_slots);
    Ok(available_slots)
}
