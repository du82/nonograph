//! science.rs — scrape content/ for post dates and generate {year}-calendar.html heatmap pages
//!
//! Run with:
//!   rustc science.rs -o science && ./science
//! or:
//!   cargo script science.rs   (if you have cargo-script installed)
//!
//! Dependencies: only the standard library.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Date parsing
// ---------------------------------------------------------------------------

/// A simple date value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if month < 1 || month > 12 || day < 1 || day > 31 {
            return None;
        }
        Some(Self { year, month, day })
    }

    fn month_name(&self) -> &'static str {
        MONTH_NAMES[(self.month - 1) as usize]
    }

    /// Day-of-week: 0 = Sunday … 6 = Saturday (Tomohiko Sakamoto algorithm)
    fn weekday(&self) -> u32 {
        let (y, m, d) = (self.year, self.month as i32, self.day as i32);
        let t: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if m < 3 { y - 1 } else { y };
        ((y + y / 4 - y / 100 + y / 400 + t[(m - 1) as usize] + d).rem_euclid(7)) as u32
    }

    /// Is the year a leap year?
    fn is_leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Number of days in a given month.
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    /// Iterate over every day in a year.
    fn all_days_in_year(year: i32) -> Vec<Date> {
        let mut days = Vec::new();
        for month in 1u32..=12 {
            for day in 1..=Self::days_in_month(year, month) {
                days.push(Date::new(year, month, day).unwrap());
            }
        }
        days
    }
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const MONTH_NAMES_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// ---------------------------------------------------------------------------
// Try to extract a date from the *first non-empty line* of a markdown file.
//
// Supported formats (all seen in the corpus):
//   "March 20, 2026"
//   "April 16, 2026 | some author"
//   "March 28, 2026 | some source"
// ---------------------------------------------------------------------------
fn parse_prose_date(line: &str) -> Option<Date> {
    // Strip everything after a pipe character
    let line = line.split('|').next().unwrap_or("").trim();

    // Expected: "<MonthName> <day>, <year>"
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }
    let month_idx = MONTH_NAMES_LONG
        .iter()
        .position(|&m| m.eq_ignore_ascii_case(parts[0]))?;
    let day: u32 = parts[1].trim_end_matches(',').parse().ok()?;
    let year: i32 = parts[2].trim().parse().ok()?;
    Date::new(year, (month_idx + 1) as u32, day)
}

// ---------------------------------------------------------------------------
// Try to extract a date from a *filename* (stem only, without extension).
//
// Supported patterns (most-specific first):
//   ends with -MM-DD-YYYY   → year/month/day explicit
//   ends with -YYYY-MM-DD   → year/month/day explicit (ISO-ish)
//   ends with -MM-DD        → month/day, year inferred from previous match or skipped
//   ends with -MM-YYYY      → month + year, day = 1
//
// Strategy: find the last occurrence of a date-like numeric suffix.
// ---------------------------------------------------------------------------
fn parse_filename_date(stem: &str) -> Option<Date> {
    // Collect all '-'-separated tokens in reverse
    let tokens: Vec<&str> = stem.split('-').collect();
    let n = tokens.len();
    if n < 2 {
        return None;
    }

    // Helper: parse a token as an integer
    let num = |t: &str| -> Option<u32> { t.parse::<u32>().ok() };

    // Try patterns starting from the end of the token list

    // Pattern: …-MM-DD-YYYY  (last three tokens)
    if n >= 3 {
        if let (Some(a), Some(b), Some(c)) =
            (num(tokens[n - 3]), num(tokens[n - 2]), num(tokens[n - 1]))
        {
            // MM-DD-YYYY
            if c > 1000 && a >= 1 && a <= 12 && b >= 1 && b <= 31 {
                return Date::new(c as i32, a, b);
            }
            // YYYY-MM-DD
            if a > 1000 && b >= 1 && b <= 12 && c >= 1 && c <= 31 {
                return Date::new(a as i32, b, c);
            }
        }
    }

    // Pattern: …-MM-DD  (last two tokens, no year — use current filename heuristic)
    if n >= 2 {
        if let (Some(a), Some(b)) = (num(tokens[n - 2]), num(tokens[n - 1])) {
            if a >= 1 && a <= 12 && b >= 1 && b <= 31 {
                // Year unknown from filename alone; caller should prefer prose date.
                // Return None here so prose date takes priority; year-less filenames
                // are handled below with a fallback year search.
                let _ = (a, b); // suppress unused warning
            }
        }
    }

    None
}

/// Full date extraction for a file: tries prose first, then filename patterns
/// including a two-token MM-DD pattern with a fallback year derived from
/// *any* four-digit year found elsewhere in the filename.
fn extract_date(path: &Path) -> Option<Date> {
    // --- 1. Try first non-empty line of file ---
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Some(d) = parse_prose_date(trimmed) {
                    return Some(d);
                }
                break; // only check first non-empty line
            }
        }
    }

    // --- 2. Try filename ---
    let stem = path.file_stem()?.to_string_lossy();
    let stem = stem.as_ref();

    // Try three-token patterns first
    if let Some(d) = parse_filename_date(stem) {
        return Some(d);
    }

    // Try two-token MM-DD with year extracted from anywhere in filename
    let tokens: Vec<&str> = stem.split('-').collect();
    let n = tokens.len();
    let num = |t: &str| -> Option<u32> { t.parse::<u32>().ok() };

    // Find a four-digit year anywhere in the tokens
    let year_from_filename: Option<i32> = tokens
        .iter()
        .find_map(|t| t.parse::<i32>().ok().filter(|&y| y > 1000 && y < 3000));

    if n >= 2 {
        if let (Some(a), Some(b)) = (num(tokens[n - 2]), num(tokens[n - 1])) {
            if a >= 1 && a <= 12 && b >= 1 && b <= 31 {
                if let Some(year) = year_from_filename {
                    return Date::new(year, a, b);
                }
            }
        }
    }

    // Fallback: single-token month-day at end with year elsewhere (e.g. name-04-15 when year in middle)
    // Already covered above.

    None
}

// ---------------------------------------------------------------------------
// Heatmap level: 1–5 based on post count relative to the max for the year
// ---------------------------------------------------------------------------
fn level(count: usize, max: usize) -> u8 {
    if count == 0 || max == 0 {
        return 0;
    }
    let ratio = count as f64 / max as f64;
    if ratio <= 0.20 {
        1
    } else if ratio <= 0.40 {
        2
    } else if ratio <= 0.60 {
        3
    } else if ratio <= 0.80 {
        4
    } else {
        5
    }
}

// ---------------------------------------------------------------------------
// HTML generation
// ---------------------------------------------------------------------------

/// Render one year's grid rows into `html`. `max_count` is passed in so all
/// years share the same heat scale.
fn render_year_grid(
    html: &mut String,
    year: i32,
    counts: &HashMap<Date, Vec<String>>,
    max_count: usize,
) {
    let all_days = Date::all_days_in_year(year);
    let total_posts: usize = counts.values().map(|v| v.len()).sum();
    let active_days = counts.len();

    // Year label + stats row
    html.push_str(&format!("        <div class=\"year-row\">\n"));
    html.push_str(&format!(
        "            <div class=\"year-label\">{}</div>\n",
        year
    ));
    html.push_str("            <div class=\"year-body\">\n");
    html.push_str("                <div class=\"wrapper\">\n");
    html.push_str("                    <div class=\"labels\">\n");
    html.push_str("                        <div>Sun</div>\n");
    html.push_str("                        <div>Mon</div>\n");
    html.push_str("                        <div>Tue</div>\n");
    html.push_str("                        <div>Wed</div>\n");
    html.push_str("                        <div>Thu</div>\n");
    html.push_str("                        <div>Fri</div>\n");
    html.push_str("                        <div>Sat</div>\n");
    html.push_str("                    </div>\n");
    html.push_str("                    <div class=\"grid\">\n");

    // Leading empty cells so Jan 1 lands on the right weekday column
    let jan1 = Date::new(year, 1, 1).unwrap();
    let start_offset = jan1.weekday();
    for _ in 0..start_offset {
        html.push_str("                        <div class=\"day empty\"></div>\n");
    }

    for date in &all_days {
        let count = counts.get(date).map(|v| v.len()).unwrap_or(0);
        let lv = level(count, max_count);

        let tip = if count == 0 {
            format!(
                "{} {}, {}: No posts",
                MONTH_NAMES[(date.month - 1) as usize],
                date.day,
                date.year
            )
        } else {
            format!(
                "{} {}, {}: {} post{}",
                MONTH_NAMES[(date.month - 1) as usize],
                date.day,
                date.year,
                count,
                if count == 1 { "" } else { "s" }
            )
        };

        let tip = tip.replace('"', "&quot;");

        let class = if lv == 0 {
            "day".to_string()
        } else {
            format!("day l{}", lv)
        };

        html.push_str(&format!(
            "                        <div class=\"{}\" data-tip=\"{}\"></div>\n",
            class, tip
        ));
    }

    html.push_str("                    </div>\n"); // .grid
    html.push_str("                </div>\n"); // .wrapper
    let avg = if active_days > 0 {
        format!("{:.1}", total_posts as f64 / active_days as f64)
    } else {
        "0.0".to_string()
    };
    html.push_str(&format!(
        "                <p class=\"year-stats\">{} posts &nbsp;·&nbsp; {} active days &nbsp;·&nbsp; {} avg posts/active day</p>\n",
        total_posts, active_days, avg
    ));
    html.push_str("            </div>\n"); // .year-body
    html.push_str("        </div>\n"); // .year-row
}

fn generate_html(by_year: &HashMap<i32, HashMap<Date, Vec<String>>>) -> String {
    let mut years: Vec<i32> = by_year.keys().cloned().collect();
    years.sort();

    let grand_total: usize = by_year
        .values()
        .flat_map(|m| m.values())
        .map(|v| v.len())
        .sum();

    // Global max so all year rows use the same heat scale
    let global_max = by_year
        .values()
        .flat_map(|m| m.values())
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    let mut html = String::new();

    // ---- Head ----
    html.push_str("<!doctype html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("    <head>\n");
    html.push_str("        <meta charset=\"UTF-8\" />\n");
    html.push_str(
        "        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n",
    );
    html.push_str("        <title>Writing Activity Heatmap</title>\n");
    html.push_str(
        r#"        <style>
            :root {
                --bg: #ffffff;
                --text: #111827;
                --muted: #6b7280;
                --empty: #f3f4f6;
                --l1: #e5e7eb;
                --l2: #d1d5db;
                --l3: #9ca3af;
                --l4: #6b7280;
                --l5: #374151;
                --size: 12px;
                --gap: 3px;
            }

            body {
                font-family:
                    system-ui,
                    -apple-system,
                    sans-serif;
                background: var(--bg);
                color: var(--text);
                margin: 0;
                padding: 2rem;
                box-sizing: border-box;
            }

            h2 {
                margin: 0 0 0.25rem;
                font-weight: 600;
                font-size: 1.25rem;
            }
            p.sub {
                margin: 0 0 1.5rem;
                color: var(--muted);
                font-size: 0.875rem;
            }

            .wrapper {
                display: flex;
                gap: 0.5rem;
                overflow-x: auto;
                padding-bottom: 1rem;
                border-bottom: 1px solid var(--l2);
                align-items: start;
            }

            .labels {
                display: grid;
                grid-template-rows: repeat(7, var(--size));
                gap: var(--gap);
                font-size: 0.6rem;
                color: var(--muted);
                height: calc(7 * var(--size) + 6 * var(--gap));
            }
            .labels div {
                height: var(--size);
                display: flex;
                align-items: center;
            }

            .grid {
                display: grid;
                grid-template-rows: repeat(7, var(--size));
                grid-auto-columns: var(--size);
                grid-auto-flow: column;
                gap: var(--gap);
            }

            .day {
                width: var(--size);
                height: var(--size);
                border-radius: 2px;
                background: var(--empty);
                position: relative;
                cursor: pointer;
                transition:
                    transform 0.1s ease,
                    box-shadow 0.1s ease;
            }
            .day:hover {
                transform: scale(1.4);
                box-shadow: 0 0 0 2px var(--l4);
                z-index: 2;
            }

            .day::after {
                content: attr(data-tip);
                position: absolute;
                bottom: 100%;
                left: 50%;
                transform: translateX(-50%) translateY(-4px);
                background: var(--text);
                color: #fff;
                padding: 4px 6px;
                border-radius: 4px;
                font-size: 0.65rem;
                white-space: nowrap;
                opacity: 0;
                pointer-events: none;
                transition: opacity 0.15s ease;
                z-index: 10;
            }
            .day:hover::after {
                opacity: 1;
            }

            .day.l1 { background: var(--l1); }
            .day.l2 { background: var(--l2); }
            .day.l3 { background: var(--l3); }
            .day.l4 { background: var(--l4); }
            .day.l5 { background: var(--l5); }
            .day.empty {
                background: transparent;
                cursor: default;
            }

            .legend {
                display: flex;
                align-items: center;
                gap: 0.4rem;
                margin-top: 1rem;
                font-size: 0.75rem;
                color: var(--muted);
            }
            .legend .box {
                width: var(--size);
                height: var(--size);
                border-radius: 2px;
            }

            .year-row {
                display: flex;
                align-items: start;
                gap: 1rem;
                margin-bottom: 1.5rem;
            }

            .year-label {
                font-size: 0.8rem;
                font-weight: 700;
                color: var(--muted);
                width: 2.5rem;
                padding-top: 2px;
                flex-shrink: 0;
                text-align: right;
            }

            .year-body {
                flex: 1;
            }

            .year-stats {
                margin: 0.4rem 0 0;
                font-size: 0.75rem;
                color: var(--muted);
            }

            .legend {
                display: flex;
                align-items: center;
                gap: 0.4rem;
                margin-top: 0.5rem;
                font-size: 0.75rem;
                color: var(--muted);
            }
            .legend .box {
                width: var(--size);
                height: var(--size);
                border-radius: 2px;
            }
        </style>
    </head>
"#,
    );

    // ---- Body ----
    html.push_str("    <body>\n");
    html.push_str("        <h2>Writing &amp; Publishing Activity</h2>\n");
    html.push_str(&format!(
        "        <p class=\"sub\">{} total posts across all years</p>\n",
        grand_total
    ));

    // ---- One row per year, oldest first ----
    for year in &years {
        let counts = &by_year[year];
        render_year_grid(&mut html, *year, counts, global_max);
    }

    // ---- Legend ----
    html.push_str("        <div class=\"legend\">\n");
    html.push_str("            <span>Less</span>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--empty)\"></div>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--l1)\"></div>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--l2)\"></div>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--l3)\"></div>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--l4)\"></div>\n");
    html.push_str("            <div class=\"box\" style=\"background: var(--l5)\"></div>\n");
    html.push_str("            <span>More</span>\n");
    html.push_str("        </div>\n");

    html.push_str("    </body>\n");
    html.push_str("</html>\n");
    html
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let collected: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}…", collected)
    } else {
        collected
    }
}

// ---------------------------------------------------------------------------
// Extract a human-readable title from a markdown file.
// Looks for the first `# Heading` line; falls back to the filename stem.
// ---------------------------------------------------------------------------
fn extract_title(path: &Path) -> String {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return trimmed[2..].trim().to_string();
            }
        }
    }
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let content_dir = Path::new("content");
    if !content_dir.exists() {
        eprintln!(
            "Error: 'content/' directory not found. Run this script from the nonograph root."
        );
        std::process::exit(1);
    }

    // Map: year -> (date -> list of post titles)
    let mut by_year: HashMap<i32, HashMap<Date, Vec<String>>> = HashMap::new();
    let mut skipped = 0usize;

    let entries = match fs::read_dir(content_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error reading content/: {}", err);
            std::process::exit(1);
        }
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "md").unwrap_or(false))
        .collect();
    files.sort();

    for path in &files {
        match extract_date(path) {
            Some(date) => {
                let title = extract_title(path);
                by_year
                    .entry(date.year)
                    .or_default()
                    .entry(date)
                    .or_default()
                    .push(title);
            }
            None => {
                skipped += 1;
                eprintln!("  [skip] could not determine date for: {}", path.display());
            }
        }
    }

    if by_year.is_empty() {
        println!("No dated posts found in content/.");
        return;
    }

    // Print summary
    println!("\n=== Post Activity Summary ===\n");
    let mut years: Vec<i32> = by_year.keys().cloned().collect();
    years.sort();

    for year in &years {
        let year_counts = &by_year[year];
        let total: usize = year_counts.values().map(|v| v.len()).sum();
        let active_days = year_counts.len();
        let max = year_counts.values().map(|v| v.len()).max().unwrap_or(0);

        // Find busiest days
        let mut sorted: Vec<(&Date, usize)> =
            year_counts.iter().map(|(d, v)| (d, v.len())).collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

        println!("Year {}:", year);
        println!("  Total posts : {}", total);
        println!("  Active days : {}", active_days);
        println!("  Busiest day : {} post(s)", max);
        println!("  Top days:");
        for (date, count) in sorted.iter().take(5) {
            println!(
                "    {} {:>2}, {} — {} post(s)",
                MONTH_NAMES_LONG[(date.month - 1) as usize],
                date.day,
                date.year,
                count,
            );
        }
        println!();
    }

    if skipped > 0 {
        println!("(Skipped {} file(s) with no parseable date)\n", skipped);
    }

    // Generate single combined calendar
    let html = generate_html(&by_year);
    let filename = "calendar.html";
    match fs::write(filename, &html) {
        Ok(_) => println!("Generated: {}", filename),
        Err(err) => eprintln!("Failed to write {}: {}", filename, err),
    }

    println!("\nDone.");
}
