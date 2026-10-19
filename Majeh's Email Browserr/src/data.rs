//! Mock data models for the UI. Phase 2 will replace this with real
//! Gmail/Outlook API data behind the same shapes.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Folder {
    Inbox,
    Starred,
    Snoozed,
    Sent,
    Drafts,
    Archived,
    Trash,
}

impl Folder {
    pub fn label(&self) -> &'static str {
        match self {
            Folder::Inbox => "Inbox",
            Folder::Starred => "Starred",
            Folder::Snoozed => "Snoozed",
            Folder::Sent => "Sent",
            Folder::Drafts => "Drafts",
            Folder::Archived => "Archived",
            Folder::Trash => "Trash",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Folder::Inbox => "inbox",
            Folder::Starred => "star",
            Folder::Snoozed => "clock",
            Folder::Sent => "send",
            Folder::Drafts => "file",
            Folder::Archived => "archive",
            Folder::Trash => "trash",
        }
    }

    pub const ALL: [Folder; 7] = [
        Folder::Inbox,
        Folder::Starred,
        Folder::Snoozed,
        Folder::Sent,
        Folder::Drafts,
        Folder::Archived,
        Folder::Trash,
    ];
}

#[derive(Clone)]
pub struct Account {
    pub name: String,
    pub email: String,
    pub initial: String,
    pub color: String,
    pub unread: u32,
}

#[derive(Clone)]
pub struct Email {
    pub id: usize,
    /// Gmail message ID (empty for mock data) — used for cache dedupe.
    pub msg_id: String,
    pub sender: String,
    pub address: String,
    /// Raw `To` header value (empty for mock data).
    pub to_field: String,
    pub subject: String,
    pub preview: String,
    pub time: String,
    /// Epoch milliseconds from Gmail `internalDate` (0 for mock data).
    pub timestamp_ms: i64,
    pub initial: String,
    pub color: String,
    pub unread: bool,
    pub starred: bool,
    pub folder: Folder,
    pub body: Vec<String>,
}

/// Session-global counter for unique `Email.id` values (fetched emails get
/// ids that never collide with the mock ids or with other accounts).
pub fn next_email_id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1000);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Format an epoch-millis timestamp like the Gmail web UI:
/// "10:24 AM" (today), "Yesterday", "Mar 8" (this year), "3/8/24" (older).
pub fn format_email_time(ms: i64) -> String {
    if ms <= 0 {
        return String::new();
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    if date.get_time().is_nan() {
        return String::new();
    }
    let now = js_sys::Date::new_0();

    // Time-of-day part.
    let h = date.get_hours();
    let m = date.get_minutes();
    let ampm = if h < 12 { "AM" } else { "PM" };
    let h12 = match h {
        0 => 12,
        1..=12 => h,
        _ => h - 12,
    };
    let clock = format!("{}:{:02} {}", h12, m, ampm);

    // Day comparison (local time).
    let same_day = date.get_full_year() == now.get_full_year()
        && date.get_month() == now.get_month()
        && date.get_date() == now.get_date();
    if same_day {
        return clock;
    }
    let yesterday = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
        now.get_time() - 86_400_000.0,
    ));
    if date.get_full_year() == yesterday.get_full_year()
        && date.get_month() == yesterday.get_month()
        && date.get_date() == yesterday.get_date()
    {
        return "Yesterday".to_string();
    }
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = MONTHS.get(date.get_month() as usize).copied().unwrap_or("?");
    if date.get_full_year() == now.get_full_year() {
        format!("{} {}", month, date.get_date())
    } else {
        let yy = date.get_full_year() % 100;
        format!("{}/{}/{}", date.get_month() + 1, date.get_date(), yy)
    }
}

/// A single in-flight or finished email-fetch job (one account + one date
/// range). Rendered by the fetch panel as its own progress bar.
#[derive(Clone, Debug, PartialEq)]
pub struct FetchTask {
    /// Unique per account+range: `{email}|{from}|{to}` (from/to empty = full inbox).
    pub key: String,
    pub account: String,
    /// Human-readable range label, e.g. `2024-01-01 → 2024-02-01`.
    pub label: String,
    /// Total messages reported by Gmail (0 = still listing).
    pub total: usize,
    /// Messages fetched so far.
    pub fetched: usize,
    pub status: TaskStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Running,
    Done,
    Error(String),
}

impl FetchTask {
    /// "Fetching 20 emails for x@gmail.com — 12/20"
    pub fn progress_text(&self) -> String {
        match &self.status {
            TaskStatus::Error(e) => format!("Failed to fetch {}: {e}", self.account),
            TaskStatus::Done => {
                if self.total > 0 {
                    format!("Fetched {} emails for {} — done", self.fetched, self.account)
                } else {
                    format!("Fetched 0 emails for {} — done", self.account)
                }
            }
            TaskStatus::Running => {
                if self.total > 0 {
                    format!(
                        "Fetching {} emails for {} — {}/{}{}",
                        self.total,
                        self.account,
                        self.fetched,
                        self.total,
                        if self.label.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", self.label)
                        }
                    )
                } else {
                    format!(
                        "Fetching emails for {}{}…",
                        self.account,
                        if self.label.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", self.label)
                        }
                    )
                }
            }
        }
    }

    /// 0.0–1.0 progress; `None` while the total is unknown.
    pub fn fraction(&self) -> Option<f32> {
        if self.total == 0 {
            None
        } else {
            Some((self.fetched as f32 / self.total as f32).clamp(0.0, 1.0))
        }
    }
}

pub fn accounts() -> Vec<Account> {
    vec![
        Account { name: "Gmail Personal".into(), email: "majeh@gmail.com".into(), initial: "G".into(), color: "#ea4335".into(), unread: 245 },
        Account { name: "Outlook Work".into(), email: "majeh@company.com".into(), initial: "O".into(), color: "#0078d4".into(), unread: 27 },
        Account { name: "Yahoo Backup".into(), email: "majeh@yahoo.com".into(), initial: "Y".into(), color: "#6001d2".into(), unread: 3 },
        Account { name: "Custom IMAP".into(), email: "mail@majeh.io".into(), initial: "I".into(), color: "#2d8a4e".into(), unread: 0 },
    ]
}

macro_rules! email {
    ($id:expr, $sender:expr, $addr:expr, $subj:expr, $prev:expr, $time:expr, $init:expr, $color:expr, $unread:expr, $starred:expr, [$($p:expr),+]) => {
        Email {
            id: $id,
            msg_id: String::new(),
            sender: $sender.into(), address: $addr.into(),
            to_field: String::new(),
            subject: $subj.into(), preview: $prev.into(),
            time: $time.into(), timestamp_ms: 0,
            initial: $init.into(), color: $color.into(), unread: $unread, starred: $starred,
            folder: Folder::Inbox, body: vec![$($p.into()),+],
        }
    };
}

pub fn emails() -> Vec<Email> {
    vec![
        email!(0, "Notion", "team@notion.so", "Your weekly digest is ready",
            "Here's what you accomplished this week with Notion...", "10:24 AM", "N", "#e8e8e8", true, false,
            ["Your weekly digest is ready — here's what you accomplished this week with Notion.",
             "12 pages edited · 4 new ideas · 8 tasks completed · 2 team mentions",
             "Highlights: Product roadmap updated · Design system improvements · Team collaboration increased by 40%.",
             "Keep building, The Notion Team"]),
        email!(1, "GitHub", "notifications@github.com", "Pull request merged",
            "Re: impl email sync performance — your PR was merged into main.", "9:41 AM", "G", "#24292f", true, false,
            ["Your pull request \"impl email sync performance\" was merged into main.",
             "Repo: ZMajeh/projects · PR #42 · merged by majeh",
             "Thanks for the contribution!"]),
        email!(2, "Firebase", "no-reply@firebase.google.com", "Build completed successfully",
            "Your app is live on Firebase Hosting 🎉", "8:17 AM", "F", "#ff9100", true, true,
            ["Your app is live on Firebase Hosting! 🎉",
             "Project: majehs-email-browserr · Channel: live",
             "Deployment completed in 42s. No errors detected."]),
        email!(3, "Linear", "no-reply@linear.app", "New comment on issue #2847",
            "@majehi commented: \"This looks great!\"", "Yesterday", "L", "#5e6ad2", false, false,
            ["@majehi commented on issue #2847:",
             "\"This looks great! Let's ship it in the next release.\""]),
        email!(4, "Google", "no-reply@accounts.google.com", "Security alert",
            "A new sign-in was detected on Mac...", "Yesterday", "G", "#4285f4", false, false,
            ["A new sign-in was detected on a Mac device.",
             "If this was you, you don't need to do anything.",
             "If not, check your account activity and change your password immediately."]),
        email!(5, "Outlook Team", "no-reply@outlook.com", "Welcome to your new Outlook",
            "Get the most out of your account with these tips...", "Yesterday", "O", "#0078d4", false, false,
            ["Welcome to your new Outlook! Get the most out of your account with these tips.",
             "Organize your inbox with focused categories, schedule emails, and more."]),
        email!(6, "Dribbble", "no-reply@dribbble.com", "Your shot was liked!",
            "Nice work! Your design is getting attention...", "Mar 8", "D", "#ea4c89", false, true,
            ["Nice work! Your design is getting attention.",
             "23 people liked your shot in the last 24 hours."]),
        email!(7, "Figma", "no-reply@figma.com", "Team file updated",
            "Majeh edited Email Browser UI Kit...", "Mar 8", "F", "#a259ff", false, false,
            ["Majeh edited \"Email Browser UI Kit\" 12 minutes ago.",
             "Open the file to see what changed."]),
        email!(8, "Apple", "no-reply@apple.com", "Your Apple ID was used to sign in",
            "Your Apple ID was used to sign in to iCloud...", "Mar 7", "A", "#555555", false, false,
            ["Your Apple ID was used to sign in to iCloud from a new device.",
             "Device: MacBook Pro · Location: Manila, PH"]),
        email!(9, "Vercel", "no-reply@vercel.com", "Deployment completed",
            "Your production deployment was deployed to...", "Mar 7", "V", "#000000", false, false,
            ["Your production deployment was deployed successfully.",
             "Build time: 38s · Region: sin1 · Status: Ready"]),
    ]
}

