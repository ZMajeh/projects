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
    pub name: &'static str,
    pub email: &'static str,
    pub initial: &'static str,
    pub color: &'static str,
    pub unread: u32,
}

#[derive(Clone)]
pub struct Email {
    pub id: usize,
    pub sender: &'static str,
    pub address: &'static str,
    pub subject: &'static str,
    pub preview: &'static str,
    pub time: &'static str,
    pub initial: &'static str,
    pub color: &'static str,
    pub unread: bool,
    pub starred: bool,
    pub folder: Folder,
    pub body: &'static [&'static str],
}

pub fn accounts() -> Vec<Account> {
    vec![
        Account { name: "Gmail Personal", email: "majeh@gmail.com", initial: "G", color: "#ea4335", unread: 245 },
        Account { name: "Outlook Work", email: "majeh@company.com", initial: "O", color: "#0078d4", unread: 27 },
        Account { name: "Yahoo Backup", email: "majeh@yahoo.com", initial: "Y", color: "#6001d2", unread: 3 },
        Account { name: "Custom IMAP", email: "mail@majeh.io", initial: "I", color: "#2d8a4e", unread: 0 },
    ]
}

macro_rules! email {
    ($id:expr, $sender:expr, $addr:expr, $subj:expr, $prev:expr, $time:expr, $init:expr, $color:expr, $unread:expr, $starred:expr, [$($p:expr),+]) => {
        Email {
            id: $id, sender: $sender, address: $addr, subject: $subj, preview: $prev,
            time: $time, initial: $init, color: $color, unread: $unread, starred: $starred,
            folder: Folder::Inbox, body: &[$($p),+],
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

