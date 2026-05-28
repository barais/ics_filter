# ICS Calendar Filter 📅✂️

A lightweight, ultra-fast, and memory-efficient web service written in **Rust**. It fetches a remote iCalendar (`.ics`) file and dynamically filters the events to keep only the appointments spanning the next `X` months. 

It was initially designed to clean up heavy remote calendars (like Zimbra servers) to easily sync them with mobile devices or Google Calendar/Outlook, without keeping years of past events.

## ✨ Features

- **Blazing Fast & Low Footprint:** Written in Rust using `axum`. Consumes ~5MB of RAM.
- **Smart Filtering:** Handles regular events and recurring events (`RRULE`). Recurring events are kept unless their end date (`UNTIL`) is already in the past.
- **Zero Data Loss:** It doesn't use a strict parser that strips custom tags. It parses the file block by block, meaning all custom provider tags (like `X-ZIMBRA-...` or `X-MICROSOFT-...`) and timezones (`VTIMEZONE`) are kept intact.
- **Dynamic Range:** Pass the number of months you want to keep directly in the URL (e.g., `/export/3` for 3 months).

## 🛠️ Prerequisites

You need to have **Rust** and **Cargo** installed on your system. 
If you don't have them yet, you can install them via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## ⚙️ Environment Variables

The application behavior can be customized using environment variables.

| Variable  | Description | Default Value |
| :--- | :--- | :--- |
| `ICS_URL` | The absolute URL of the remote source `.ics` calendar you want to filter. | `http://zimbra.inria.fr/home/olivier.barais@irisa.fr/Calendar.ics` |
| `PORT` | The port on which the server will listen.| `8080` |

*(Note: The server listens on port `8080` by default).*

## 🚀 How to Compile and Run

### Local Development / Testing

To test the application locally:

```bash
# Export the environment variable (optional, falls back to default if omitted)
export ICS_URL="https://your-server.com/path/to/calendar.ics"

# Run the project
cargo run
```

### Production Build

For production, you should compile the project in `release` mode to get a highly optimized binary.

```bash
# 1. Compile the binary
cargo build --release

# 2. The compiled binary is now available in the target/release folder
# You can run it directly:
ICS_URL="https://your-server.com/path/to/calendar.ics" ./target/release/calendar_filter
```

## 🌐 Usage

Once the server is running, you can fetch your filtered calendar by accessing the `/export/:months` endpoint. 

Replace `:months` with the number of upcoming months you want to keep.

**Examples:**
- Fetch events for the next **3 months**:
  ```bash
  curl http://localhost:8080/export/3 > filtered_calendar.ics
  ```
- Fetch events for the next **6 months**:
  ```bash
  curl http://localhost:8080/export/6 > filtered_calendar.ics
  ```

If your service is deployed behind a reverse proxy (like Nginx, Traefik, or Caddy) under the domain `calendarfiltre.barais.fr`, you simply add this URL to your calendar app (Google Calendar, Apple Calendar, Thunderbird):
`https://calendarfiltre.barais.fr/export/3`

## 🧠 How the filtering logic works

1. **Past Events**: If an event (`VEVENT`) started and ended in the past, it is removed.
2. **Future Events**: If an event falls within the window `[Today -> Today + X months]`, it is kept.
3. **Recurring Events (`RRULE`)**: If an event repeats indefinitely, it is kept. If it repeats but has an `UNTIL` date that is already in the past, it is removed.
4. **Metadata**: Headers, footers, and `VTIMEZONE` blocks are strictly preserved so the calendar remains valid and keeps the right timezones.

## 📄 License

MIT License

### Quelques conseils supplémentaires :
- Si tu as besoin d'ajouter une variable d'environnement pour le **PORT** (plutôt que de le figer sur `8080`), tu peux modifier la ligne `let addr = SocketAddr::from(([0, 0, 0, 0], 8080));` dans ton code `main.rs` par :
  ```rust
  let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
  let addr = SocketAddr::from(([0, 0, 0, 0], port));
  ```
  *(Et tu pourras rajouter `PORT` dans le tableau des variables d'environnement de ce README !)*

  