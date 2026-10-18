# UI Plan — Majeh's Email Browserr

> Created 2026-10-18. Goal: replace the plain placeholder UI with a modern, dark-themed,
> responsive email client UI (Gmail/Outlook-class look), using **mock data only**
> (real API integration is Phase 2 and stays out of scope here).

## Design Reference

- **Desktop (≥ 901px):** 3-pane layout — dark left sidebar, email list, reading pane,
  with a slim top bar (logo, global search, profile).
- **Mobile (≤ 900px):** single-column list → reading pane flow, hamburger drawer for the
  sidebar, bottom tab navigation (Mailbox / Settings / About).
- **Theme:** dark navy glassmorphism-adjacent (per GEMINI.md Phase 3 "Aesthetics"),
  accent blue `#3b82f6`, rounded cards, subtle borders.

## Layout Structure

```
.app-container (column)
├── TopBar            logo · search · profile        (hamburger on mobile)
├── .app-body (row)
│   ├── Sidebar       Accounts · Folders · Labels · Settings/About · Quote
│   │                 (off-canvas drawer on mobile)
│   └── main.content
│       ├── [Mailbox tab]  .mail-layout = EmailList | ReadingPane
│       ├── [Settings tab] settings cards
│       └── [About tab]    about card
└── BottomNavigation  (mobile only)
```

## Components (src/components/)

| Component | Responsibility |
|---|---|
| `top_bar` | Logo, search input (live filter), hamburger (mobile), profile chip |
| `sidebar` | **Accounts list w/ unread badges + "Add Account"**, folder list (Inbox 275, Starred, Snoozed, Sent, Drafts, Archived, Trash), labels (Work/Personal/Newsletters), Compose button, Settings/About links, footer quote |
| `email_list` | Inbox header + unread count, filter tabs (All / Unread / Starred), email rows (avatar, sender, subject, preview, time, unread dot, star toggle) |
| `reading_pane` | Header (sender, address, time, actions), mock email body, Reply/Reply All/Forward footer, back button (mobile) |
| `navigation` | Existing bottom tab bar — visible on mobile only |
| `settings` | Rows per design: Account, Notifications (toggle), Appearance, Email Sync, Manage Labels, Privacy & Security, Connected Apps, Sign Out |
| `about` | Logo, app name, tagline, version, link rows (Features, Open Source, Report a Bug, Contact, Privacy Policy, Terms of Service) |
| `icons` | Tiny inline-SVG icon set (Feather-style paths) shared by all components |

## State (signals in `app.rs`)

- `active_tab: Tab` — Mailbox / Settings / About
- `emails: Vec<Email>` (signal) — mock store; star/unread mutations update it
- `selected_id: Option<usize>` — reading pane target (None → placeholder on desktop, list on mobile)
- `folder: Folder` — active sidebar folder
- `filter: EmailFilter` — All / Unread / Starred tabs
- `query: String` — live search over sender/subject/preview
- `drawer_open: bool` — mobile sidebar drawer
- `notifications: bool` — settings toggle

## Mock Data (`src/data.rs`)

- `Account { name, email, initial, color, unread }` — Gmail Personal (245), Outlook Work (27), Yahoo Backup (3), Custom IMAP (0)
- `Email { id, sender, address, subject, preview, time, avatar, color, unread, starred, folder, body }` — ~10 rows (Notion, GitHub, Firebase, Linear, Google, Outlook Team, Dribbble, Figma, Apple, Vercel)
- `Folder` enum with display names + icons

## Styling (`style.css`, external, sectioned)

1. **Base** — CSS vars (colors, radii), reset, scrollbar styling
2. **Layout** — app shell, top bar, sidebar, mail grid
3. **Sidebar** — accounts, folders, labels, compose, quote
4. **Email list** — filter tabs, rows, avatars, stars
5. **Reading pane** — header, body, action footer
6. **Settings / About** — card + row styles
7. **Bottom nav** — mobile tab bar
8. **Responsive** — `@media (max-width: 900px)`: drawer sidebar, single-column mail flow, mobile-only elements (hamburger, back button, bottom nav), touch-friendly targets

## Acceptance Criteria

- [x] Desktop 3-pane matches reference (dark sidebar w/ accounts + badges, list w/ filters, reading pane)
- [x] Mobile: list → reading pane, hamburger drawer, bottom nav, settings/about as full-screen lists
- [x] Live search + folder/filter/star interactions work with mock data
- [x] `trunk build` clean; no inline CSS in index.html
- [ ] Real auth/API integration — explicitly deferred to Phase 2
