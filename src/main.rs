use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::{Datelike, NaiveDate, Utc};
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
    ics_url: String,
}

#[tokio::main]
async fn main() {
    // Tu peux surcharger l'URL via une variable d'environnement si besoin
    let ics_url = std::env::var("ICS_URL").unwrap_or_else(|_| {
        "http://zimbra.inria.fr/home/olivier.barais@irisa.fr/Calendar.ics".to_string()
    });

    // Client HTTP partagé pour réutiliser les connexions
    let state = AppState {
        client: Arc::new(Client::new()),
        ics_url,
    };

    let app = Router::new()
        .route("/export/:months", get(export_calendar))
        .with_state(state);
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Service démarré sur http://localhost:{}", port);
    println!("Exemple d'accès : http://localhost:{}/export/3", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Handler qui intercepte la requête web et traite l'ICS
async fn export_calendar(
    Path(months): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    
    // 1. Récupération du calendrier d'origine
    let response = state
        .client
        .get(&state.ics_url)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    let ics_content = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Filtrage du calendrier
    let filtered_ics = filter_ics(&ics_content, months);

    // 3. Renvoi au format officiel iCalendar
    let mut res = filtered_ics.into_response();
    res.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::header::HeaderValue::from_static("text/calendar; charset=utf-8"),
    );
    Ok(res)
}

/// Moteur de filtrage léger du format ICS
fn filter_ics(ics_content: &str, months_to_keep: u32) -> String {
    let mut output = String::with_capacity(ics_content.len());
    
    let now = Utc::now().naive_utc().date();
    // Calcule la date limite (Aujourd'hui + X mois)
    let window_end = now
        .checked_add_months(chrono::Months::new(months_to_keep))
        .unwrap_or(now);

    let mut current_event = String::new();
    let mut in_event = false;
    
    // Variables pour stocker les métadonnées de l'événement en cours d'analyse
    let mut has_rrule = false;
    let mut until_date = None;
    let mut dtstart = None;
    let mut dtend = None;

    for line in ics_content.lines() {
        if line.starts_with("BEGIN:VEVENT") {
            in_event = true;
            current_event.clear();
            current_event.push_str(line);
            current_event.push_str("\r\n");
            
            has_rrule = false;
            until_date = None;
            dtstart = None;
            dtend = None;
        } else if line.starts_with("END:VEVENT") {
            current_event.push_str(line);
            current_event.push_str("\r\n");
            
            // Décision : Doit-on conserver cet événement ?
            let keep = if has_rrule {
                // S'il est récurrent, on le garde TOUJOURS sauf s'il a une fin (UNTIL) qui est déjà passée.
                if let Some(until) = until_date {
                    until >= now
                } else {
                    true // Récurrence infinie
                }
            } else {
                // RDV ponctuel : on vérifie qu'il chevauche notre fenêtre [now, window_end]
                let start = dtstart.unwrap_or(now);
                let end = dtend.unwrap_or(start);
                
                start <= window_end && end >= now
            };

            if keep {
                output.push_str(&current_event);
            }
            in_event = false;
        } else if in_event {
            current_event.push_str(line);
            current_event.push_str("\r\n");

            // Extraction sommaire des dates pour notre logique
            if line.starts_with("RRULE") {
                has_rrule = true;
                if let Some(idx) = line.find("UNTIL=") {
                    until_date = extract_date(&line[idx..]);
                }
            } else if line.starts_with("DTSTART") {
                dtstart = extract_date(line);
            } else if line.starts_with("DTEND") {
                dtend = extract_date(line);
            }
        } else {
            // Tout ce qui n'est pas un VEVENT (l'entête, les VTIMEZONE, le footer) est gardé tel quel
            output.push_str(line);
            output.push_str("\r\n");
        }
    }

    output
}

/// Cherche intelligemment les 8 premiers chiffres consécutifs d'une chaîne (format YYYYMMDD)
fn extract_date(s: &str) -> Option<NaiveDate> {
    let mut digits = String::new();
    let mut in_digits = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            in_digits = true;
        } else if in_digits {
            if digits.len() >= 8 {
                break;
            } else {
                digits.clear();
                in_digits = false;
            }
        }
    }
    
    if digits.len() >= 8 {
        let y = digits[0..4].parse().ok()?;
        let m = digits[4..6].parse().ok()?;
        let d = digits[6..8].parse().ok()?;
        NaiveDate::from_ymd_opt(y, m, d)
    } else {
        None
    }
}