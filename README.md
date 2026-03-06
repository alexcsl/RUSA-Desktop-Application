# RUSA — Remote Universe Space Administration

A desktop application for managing operations across a remote space administration agency. Built with **Tauri**, **SvelteKit**, and **Rust**, backed by **Supabase** (PostgreSQL).

## Repository Structure

```
├── rusa/                  # Desktop application (Tauri + SvelteKit + Rust)
│   ├── src/               # SvelteKit frontend
│   ├── src-tauri/         # Rust backend (Tauri)
│   ├── supabase/          # Database migrations & seed data
│   └── static/            # Static assets
│
└── Diagram_SUBMIT/        # UML diagrams & documentation
    ├── Activity/           # Activity diagrams
    ├── MLSD/               # Multi-Level Sequence Diagrams
    ├── Main.vpp            # Visual Paradigm project
    ├── RUSA - Class Diagram.png
    ├── RUSA - Use Case Diagram.png
    └── RUSA_Use_Case_Description.xlsx
```

## Subsystems

The application is organized into **12 subsystems**, each serving a distinct division within the space administration:

| # | Subsystem | Description |
|---|-----------|-------------|
| 01 | **Engineers** | Task management, progress reports, inventory tracking |
| 02 | **Data Analysts** | Statistical analysis requests, data processing pipelines |
| 03 | **Security Teams** | Security reports, secure messaging, incident management |
| 04 | **Scientists** | Research proposals, experiment tracking, peer review |
| 05 | **Astronauts** | Mission management, journals, broadcast requests |
| 06 | **Exoplanet Settlers** | Settler relocation, habitat management |
| 07 | **Psychiatry** | Mental health assessments, counseling records |
| 08 | **Space Station Settlers** | Station operations, life support monitoring |
| 09 | **Medical** | Medical records, health reports, treatment logs |
| 10 | **Sanitary** | Sanitation schedules, expenditure tracking |
| 11 | **Directors** | Division oversight with 13 specialized director roles |
| 12 | **Administrator** | System-wide administration, audit logs, personnel management |

## Tech Stack

- **Frontend:** SvelteKit 5, TypeScript, Vite
- **Backend:** Rust (Tauri 2)
- **Database:** PostgreSQL via Supabase
- **Cache:** Redis
- **Auth:** JWT + bcrypt
- **Desktop:** Tauri 2 (cross-platform)

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/)
- [Tauri CLI](https://tauri.app/)
- Supabase project (or local PostgreSQL + Redis)

### Installation

```bash
cd rusa
npm install
```

### Development

```bash
cd rusa
npm run tauri dev
```

### Build

```bash
cd rusa
npm run tauri build
```

## Diagrams

The `Diagram_SUBMIT/` folder contains all UML documentation:

- **Activity Diagrams** — Workflow visualizations for key use cases
- **MLSD** — Multi-Level Sequence Diagrams
- **Class Diagram** — System class structure
- **Use Case Diagram** — Actor–system interactions
- **Use Case Descriptions** — Detailed specifications (Excel)
