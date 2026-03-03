# Tor.Web.Capture

A Rust application for capturing web pages via TOR with an integrated web interface.

## Features

- **Integrated TOR Client** - Uses Arti (Rust TOR implementation) for native TOR connectivity
- **Web Interface** - Dynamic HTMX-powered dashboard for managing captures
- **Screenshot + HTML Capture** - Full page screenshots and HTML source via headless Chrome
- **IoT Bot User-Agents** - Shodan, Censys, ZGrab, Masscan, Nmap, and 10+ others
- **Scheduled Captures** - Cron-based scheduling for automated captures
- **Google Drive Upload** - OAuth2 and Service Account support
- **SQLite Storage** - Local database for targets, captures, and configuration
- **Security-First** - DNS via TOR, WebRTC disabled, circuit isolation

## Requirements

- Rust 1.75+
- Chromium or Google Chrome
- Linux (tested on Ubuntu 22.04+)

## Installation

```bash
# Clone the repository
git clone https://github.com/youruser/Tor.Web.Capture.git
cd Tor.Web.Capture

# Build
cargo build --release

# Run
cargo run --release
# Or directly:
./target/release/tor.web.capture
```

## Configuration

Configuration file: `config/default.toml`

```toml
[web]
bind_address = "127.0.0.1"
port = 8080

[tor]
enabled = true
data_dir = "./data/tor"
new_circuit_per_capture = true

[capture]
storage_path = "./data/captures"
max_concurrent_captures = 3
default_viewport_width = 1920
default_viewport_height = 1080

[storage]
database_path = "./data/tor-capture.db"

[gdrive]
enabled = false
auto_upload = false
```

## Usage

1. Start the application:
   ```bash
   cargo run --release
   ```

2. Open your browser to `http://127.0.0.1:8080`

3. Add a target URL to capture

4. Click "Capture" or set up a schedule

## Architecture

```mermaid
graph TB
    subgraph "Tor.Web.Capture"
        subgraph "Frontend"
            Browser["Browser<br/>HTMX"]
        end

        subgraph "Web Layer"
            Web["tor-capture-web<br/>Axum + Templates"]
        end

        subgraph "Core Services"
            Browser_Crate["tor-capture-browser<br/>Chrome CDP"]
            Scheduler["tor-capture-scheduler<br/>Cron Jobs"]
            Storage["tor-capture-storage<br/>SQLite + r2d2"]
        end

        subgraph "Network Layer"
            Network["tor-capture-network<br/>Arti Client"]
        end

        subgraph "External Services"
            GDrive["tor-capture-gdrive<br/>OAuth2 Upload"]
        end

        subgraph "Shared"
            Core["tor-capture-core<br/>Models • Errors • Config"]
        end
    end

    subgraph "TOR Network"
        Guard["Guard Node"]
        Middle["Middle Node"]
        Exit["Exit Node"]
    end

    subgraph "External"
        Target["Target Website"]
        GoogleDrive["Google Drive"]
    end

    Browser <--> Web
    Web --> Browser_Crate
    Web --> Scheduler
    Web --> Storage
    Browser_Crate --> Network
    Scheduler --> Network
    Network --> Guard
    Guard --> Middle
    Middle --> Exit
    Exit --> Target
    Storage --> GDrive
    GDrive --> GoogleDrive

    Core -.-> Web
    Core -.-> Browser_Crate
    Core -.-> Network
    Core -.-> Storage
    Core -.-> Scheduler
    Core -.-> GDrive
```

## Capture Flow

```mermaid
sequenceDiagram
    participant User as User (Browser)
    participant Web as Axum Router
    participant Chrome as Chrome CDP
    participant TOR as TOR SOCKS5
    participant Target as Target Website
    participant DB as SQLite
    participant GDrive as Google Drive

    User->>Web: POST /targets/{id}/capture
    Web->>Chrome: capture_page()
    Chrome->>TOR: Connect via SOCKS5
    TOR->>Target: Request (anonymized)
    Target-->>TOR: Response
    TOR-->>Chrome: HTML + Assets
    Chrome-->>Chrome: Take Screenshot
    Chrome-->>Web: CaptureResult
    Web->>DB: Save Capture

    alt Auto-upload enabled
        Web->>GDrive: Upload Screenshot
        GDrive-->>Web: File ID
        Web->>DB: Update with GDrive ID
    end

    Web-->>User: HTMX Partial (DOM Update)
```

## Crate Dependencies

```mermaid
graph BT
    Core["tor-capture-core<br/>(models, errors, config)"]

    Network["tor-capture-network<br/>(Arti TOR)"]
    Browser["tor-capture-browser<br/>(Chrome)"]
    Storage["tor-capture-storage<br/>(SQLite)"]
    Scheduler["tor-capture-scheduler<br/>(Cron)"]
    GDrive["tor-capture-gdrive<br/>(OAuth2)"]

    Web["tor-capture-web<br/>(Axum + HTMX)"]
    Main["src/main.rs<br/>(binary)"]

    Network --> Core
    Browser --> Core
    Storage --> Core
    Scheduler --> Core
    GDrive --> Core

    Web --> Network
    Web --> Browser
    Web --> Storage
    Web --> Scheduler
    Web --> GDrive

    Main --> Web
```

## Project Structure

```
Tor.Web.Capture/
├── src/main.rs                   # Entry point
├── config/default.toml           # Configuration
├── static/                       # CSS, JS assets
├── data/                         # Database, captures, TOR data
└── crates/
    ├── tor-capture-core/         # Models, errors, config
    ├── tor-capture-network/      # Arti TOR client wrapper
    ├── tor-capture-browser/      # Chrome headless capture
    ├── tor-capture-storage/      # SQLite repositories
    ├── tor-capture-gdrive/       # Google Drive integration
    ├── tor-capture-scheduler/    # Cron job scheduler
    └── tor-capture-web/          # Axum web server + HTMX
```

## IoT User-Agents

The following scanner user-agents are available:

| Scanner | Category |
|---------|----------|
| Shodan | IoT Scanner |
| Censys | Security Scanner |
| ZGrab | Security Scanner |
| Masscan | IoT Scanner |
| Nmap NSE | Security Scanner |
| BinaryEdge | IoT Scanner |
| FOFA | IoT Scanner |
| ZoomEye | IoT Scanner |
| GreyNoise | Security Scanner |
| Shadowserver | Security Scanner |
| SecurityTrails | Security Scanner |
| Onyphe | Security Scanner |
| IPinfo | Security Scanner |

## API Endpoints

### Web Interface (HTMX)

| Method | Route | Description |
|--------|-------|-------------|
| GET | `/` | Dashboard |
| GET | `/targets` | List targets |
| POST | `/targets` | Create target |
| POST | `/targets/{id}/capture` | Trigger capture |
| GET | `/captures` | List captures |
| GET | `/captures/{id}/screenshot` | Download screenshot |
| GET | `/schedules` | List schedules |
| GET | `/settings` | Settings page |

### REST API (JSON)

| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/status` | Application status |
| GET | `/api/v1/targets` | List all targets |
| POST | `/api/v1/targets` | Create target |
| GET | `/api/v1/captures` | List captures |
| GET | `/api/v1/user-agents` | List user agents |

## Security

### Embedded TOR Client

The application embeds its own TOR client via **Arti** (pure Rust TOR implementation). No external TOR daemon required.

```mermaid
graph LR
    subgraph "Tor.Web.Capture"
        Chrome["Chrome Headless"]
        Arti["Arti TOR Client<br/>SOCKS5 127.0.0.1:9050"]
    end

    subgraph "TOR Network"
        Guard["Guard Node"]
        Middle["Middle Node"]
        Exit["Exit Node"]
    end

    Target["Target Website"]

    Chrome -->|"--proxy-server=socks5://..."| Arti
    Arti --> Guard
    Guard --> Middle
    Middle --> Exit
    Exit --> Target
```

### Traffic Isolation

```mermaid
graph TB
    subgraph "Protected Traffic (via TOR)"
        Captures["Web Captures"]
        DNS["DNS Resolution"]
        PageContent["Page Content"]
    end

    subgraph "Direct Traffic (no TOR)"
        WebUI["Web UI (localhost:8080)"]
        GDrive["Google Drive Upload"]
        Bootstrap["TOR Bootstrap"]
    end

    subgraph "Chrome Hardening"
        Proxy["--proxy-server=socks5"]
        DNSRule["--host-resolver-rules=MAP * ~NOTFOUND"]
        NoWebRTC["--disable-webrtc"]
        Incognito["--incognito"]
        NoCache["--disable-application-cache"]
    end

    Captures --> Proxy
    DNS --> DNSRule
    PageContent --> NoWebRTC
```

### Anti-Leak Protections

| Protection | Chrome Flag | Purpose |
|------------|-------------|---------|
| **SOCKS5 Proxy** | `--proxy-server=socks5://127.0.0.1:9050` | Route all traffic via TOR |
| **DNS via TOR** | `--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost` | Prevent DNS leaks |
| **WebRTC Disabled** | `--disable-webrtc` + `--disable-features=WebRTC` | Prevent IP leaks |
| **No Cache** | `--disable-application-cache` + `--aggressive-cache-discard` | No persistent data |
| **Incognito** | `--incognito` | No cookies/history |
| **Circuit Isolation** | New circuit per capture (configurable) | Prevent correlation |

### What Goes Through TOR

| Traffic Type | Via TOR | Notes |
|--------------|---------|-------|
| Web captures (screenshot/HTML) | Yes | All target requests |
| DNS resolution | Yes | Forced via SOCKS5 |
| Page assets (JS, CSS, images) | Yes | All resources |
| Web UI (localhost) | No | Local only |
| Google Drive uploads | No | Direct connection |
| TOR bootstrap | No | Initial consensus download |

## License

MIT License
