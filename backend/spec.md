# Rust SSR Web Dashboard – Full Technical Specification

> Single-user, password-protected Rust-based SSR dashboard for key management and logs

---

## Overview

A minimal **Rust SSR dashboard** that:
- Requires a single password on first load (no admin accounts or multi-user system).
- Landing page is `/logs` (no separate dashboard).
- Allows managing npub keys, policies, groups, invitations, and viewing logs.
- Uses **SSR** pages only with **Handlebars templates**.
- Optional SSE for live logs updates.

### Tech Stack

| Component       | Choice |
|----------------|--------|
| Language        | Rust |
| Web Framework   | Axum or Actix-web |
| Templates       | Handlebars-Rust or Askama |
| Database        | PostgreSQL (via sqlx) |
| Sessions        | Cookie-based session (signed) |
| Realtime        | SSE for logs updates |
| Styling         | Minimal CSS, dark-first |
| JS              | Minimal, only SSE + copy buttons if needed |

---

## Authentication Flow

- **Single password** stored as Argon2 hash in a config file or `.env`.
- **First load:** prompt to create password (setup mode).
- **Login:** validate password, set signed session cookie.
- **Session TTL:** 24 hours - cookies expire after 24 hours.
- **Session Revalidation:** Login within the 24-hour window revalidates and extends the token duration for another 24 hours.
- **Middleware:** blocks all routes except `/login` if not authenticated.

---

## Pages (SSR UI Specs)

### 1. `/login` (Setup / Login)
- Full-screen centered card.
- Fields: password input, submit button.
- If first load: "Set App Password".
- On success: redirect to `/logs`.
- Error: small inline red message.

### 2. `/logs` (Home / Landing page)
- Header: Title "Logs" + toolbar (Live mode toggle, Export CSV/JSON).
- Table columns: Timestamp, npub/source, Action, Result, IP.
- Infinite scroll or pagination for older logs.
- SSE live updates; fade-in new rows.
- Empty state: “No logs yet. Activity will appear here.”

### 3. `/logs/:id`
- View details of a single log entry:
  - File Name, Action, Peer ID, Timestamp, Duration, Size, Status/Error messages.
- Navigation: Back to `/logs`.

### 4. `/keys`
- Header: "Keys" + [Add Key] button.
- Table columns: Status (🟢/⚫), npub, NIP-05 profile, Policy, Group, Expiry, Actions.
- Actions: Enable/Disable, Edit, Delete.

### 5. `/keys/add`
- Modal / separate page.
- Fields: Manual npub / NIP-05, optional expiry, assign to policy/group.
- Fetch profile button for NIP-05.
- Submit → adds to DB and returns to `/keys`.

### 6. `/policies`
- List of policies: Name, Active Days, Time Window, Expiry, Actions (Edit/Delete).
- Add/Edit policy form: Name, weekdays, time start/end, expiry days.

### 7. `/groups`
- List of groups: Name, Keys Count, Active status, Actions (Manage/Delete).
- Manage group: assign/remove keys, view assigned keys.

### 8. `/invites`
- Table: Token, Expiry, Max Uses, Uses, Enabled, Actions (Disable/Copy link).
- Form to create new invite: expiry date/time, max uses, optional comment.

### 9. `/settings`
- Sections: Security (change password), Portal defaults (invite expiry, default policy), Log retention, Theme toggle (dark/light).
- Vertical card layout per category, Save button.

---

## Directory Structure

```
src/
├── main.rs
├── config.rs         # config loader, password hash
├── auth.rs           # session middleware, login logic
├── db.rs             # database pool & migrations
├── routes/
│    ├── login.rs
│    ├── logs.rs
│    ├── keys.rs
│    ├── policies.rs
│    ├── groups.rs
│    ├── invites.rs
│    ├── settings.rs
├── models/
│    ├── key.rs
│    ├── policy.rs
│    ├── group.rs
│    ├── invite.rs
│    └── log.rs
├── sse.rs            # SSE streaming for logs
├── templates/
│    ├── layout.hbs
│    ├── login.hbs
│    ├── logs.hbs
│    ├── log_detail.hbs
│    ├── keys.hbs
│    ├── keys_add.hbs
│    ├── policies.hbs
│    ├── policies_new.hbs
│    ├── groups.hbs
│    ├── groups_view.hbs
│    ├── invites.hbs
│    └── settings.hbs
├── utils.rs          # helpers, formatting functions
└── middleware.rs     # auth checks, logging middleware
```

---

## Database Schema (PostgreSQL)

### `keys`
| Column | Type | Notes |
|--------|------|-------|
| id | UUID (PK) | |
| npub | TEXT | Public key |
| nip05 | TEXT | Optional |
| profile_name | TEXT | Derived from NIP-05 |
| status | INTEGER | 1=enabled, 0=disabled |
| policy_id | UUID | FK → policies.id |
| group_id | UUID | FK → groups.id |
| expires_at | TIMESTAMP | Optional expiry |
| created_at | TIMESTAMP | Timestamp |

### `policies`
| Column | Type | Notes |
|--------|------|------|
| id | UUID (PK) | |
| name | TEXT | |
| active_days | TEXT | CSV e.g., "mon,tue,wed" |
| time_start | TIME | HH:MM |
| time_end | TIME | HH:MM |
| expiry_days | INTEGER | Optional forced expiry |
| created_at | TIMESTAMP | |

### `groups`
| Column | Type | Notes |
|--------|------|------|
| id | UUID (PK) | |
| name | TEXT | |
| created_at | TIMESTAMP | |

### `invites`
| Column | Type | Notes |
|--------|------|------|
| id | UUID (PK) | |
| token | TEXT | Unique |
| expires_at | TIMESTAMP | |
| max_uses | INTEGER | |
| uses | INTEGER | |
| created_at | TIMESTAMP | |

### `logs`
| Column | Type | Notes |
|--------|------|------|
| id | UUID (PK) | |
| key_id | UUID | FK → keys.id |
| timestamp | TIMESTAMP | |
| action | TEXT | |
| result | TEXT | "success" / "denied" |
| ip_address | INET | |

---

## Routes Summary

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/login` | Login form | No |
| POST | `/login` | Validate password | No |
| GET | `/logout` | End session | Yes |
| GET | `/logs` | Logs list | Yes |
| GET | `/logs/:id` | Log detail | Yes |
| GET | `/logs/stream` | SSE feed | Yes |
| GET | `/keys` | List keys | Yes |
| GET | `/keys/add` | Add key form | Yes |
| POST | `/keys/add` | Create key | Yes |
| POST | `/keys/:id/toggle` | Enable/Disable | Yes |
| POST | `/keys/:id/delete` | Delete key | Yes |
| GET | `/policies` | List policies | Yes |
| GET | `/policies/new` | Add policy form | Yes |
| POST | `/policies/new` | Create policy | Yes |
| GET | `/groups` | List groups | Yes |
| GET | `/groups/:id` | Manage group | Yes |
| POST | `/groups/new` | Create group | Yes |
| POST | `/groups/:id/add-key` | Assign key | Yes |
| GET | `/invites` | List invites | Yes |
| POST | `/invites/new` | Create invite | Yes |
| GET | `/settings` | Settings form | Yes |
| POST | `/settings` | Save settings | Yes |

---

## SSE / Live Logs

- `/logs/stream` returns `text/event-stream`
- On new DB entry, server pushes via SSE
- Client JS appends new rows to logs table
- Pause/resume toggle available

---

## Minimal JS

| Feature | Required? |
|---------|------------|
| SSE Live Logs | ✅ |
| Copy invite links | ✅ optional |
| Modals / add-key | ✅ optional |
| Pagination | ❌ server-side |
| Theme toggle | ✅ optional |

---

## Error Pages

- 401: Unauthorized (wrong password)
- 404: Not Found
- 500: Internal Error

---

## Settings Table

| Field | Notes |
|-------|------|
| log_retention_days | Purge logs after X days |
| invite_default_expiry | Default expiry for new invites |
| key_default_policy | Default policy assigned to keys |
| theme_default | "dark" or "light" |

---

## Notes

- No SPA: all pages SSR
- Minimal design, dark-first
- Focus on logs page as landing page
- Optional SSE enhances logs view
- Single password for access, no accounts

