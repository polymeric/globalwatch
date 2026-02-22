use serde::{Deserialize, Serialize};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FeedEvent {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub label: String,
    pub headline: String,
    pub source: String,
    pub severity: String, // "low" | "medium" | "high"
    pub category: String, // "weather" | "news"
}

// ── Sources config ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SourcesFile {
    sources: Vec<SourceConfig>,
}

#[derive(Debug, Deserialize)]
struct SourceConfig {
    id: String,
    #[serde(default)]
    enabled: bool,
    #[serde(rename = "feedType")]
    feed_type: String,
    url: String,
    #[serde(default)]
    category: String,
}

fn load_sources(app: &tauri::AppHandle) -> Vec<SourceConfig> {
    // Look for sources.config.json next to the app binary / in CWD (dev mode).
    let candidates = [
        std::env::current_dir()
            .ok()
            .map(|d| d.join("sources.config.json")),
        tauri::Manager::path(app).app_data_dir().ok().map(|d| d.join("sources.config.json")),
    ];

    for path in candidates.into_iter().flatten() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<SourcesFile>(&text) {
                Ok(sf) => {
                    log::info!("[feeds] loaded {} sources from {}", sf.sources.len(), path.display());
                    return sf.sources.into_iter().filter(|s| s.enabled).collect();
                }
                Err(e) => log::warn!("[feeds] failed to parse sources.config.json: {e}"),
            }
        }
    }

    // Fallback — always fetch the two free default sources.
    log::info!("[feeds] no sources.config.json found, using built-in defaults");
    vec![
        SourceConfig {
            id: "gdacs".into(),
            enabled: true,
            feed_type: "rss".into(),
            url: "https://www.gdacs.org/xml/rss.xml".into(),
            category: "weather".into(),
        },
        SourceConfig {
            id: "noaa-alerts".into(),
            enabled: true,
            feed_type: "atom".into(),
            url: "https://alerts.weather.gov/cap/us.php?x=1".into(),
            category: "weather".into(),
        },
    ]
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn fetch_all(app: &tauri::AppHandle) -> Vec<FeedEvent> {
    let sources = load_sources(app);
    if sources.is_empty() {
        return vec![];
    }

    // Fan-out all fetches concurrently.
    let futures: Vec<_> = sources
        .into_iter()
        .map(|s| async move { fetch_source(s).await })
        .collect();

    let results = futures::future::join_all(futures).await;
    let mut events = Vec::new();
    for result in results {
        match result {
            Ok(mut e) => events.append(&mut e),
            Err(e) => log::warn!("[feeds] source error: {e}"),
        }
    }
    log::info!("[feeds] total events: {}", events.len());
    events
}

async fn fetch_source(src: SourceConfig) -> Result<Vec<FeedEvent>, String> {
    let xml = get_text(&src.url).await?;
    let category = if src.category.is_empty() { "news".to_owned() } else { src.category };

    let events = match src.id.as_str() {
        "gdacs" => parse_gdacs(&xml),
        "noaa-alerts" => parse_noaa(&xml),
        _ => parse_rss_news(&xml, &src.id, &category),
    };

    log::info!("[feeds] {}: {} events", src.id, events.len());
    Ok(events)
}

async fn get_text(url: &str) -> Result<String, String> {
    reqwest::Client::new()
        .get(url)
        .header("User-Agent", "globalwatch/0.1")
        .send()
        .await
        .map_err(|e| format!("{url}: request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("{url}: body read failed: {e}"))
}

// ── GDACS RSS (geo:lat / geo:long embedded) ───────────────────────────────────

fn parse_gdacs(xml: &str) -> Vec<FeedEvent> {
    let mut events = Vec::new();
    for (i, item) in xml_blocks(xml, "item").into_iter().enumerate() {
        let Some(lat) = xml_tag(&item, "geo:lat").and_then(|s| s.trim().parse().ok()) else {
            continue;
        };
        let Some(lon) = xml_tag(&item, "geo:long").and_then(|s| s.trim().parse().ok()) else {
            continue;
        };

        let title = xml_tag(&item, "title")
            .map(|s| strip_cdata(s.trim()).to_owned())
            .unwrap_or_default();

        let alert = xml_tag(&item, "gdacs:alertlevel")
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();

        let severity = match alert.as_str() {
            "red" => "high",
            "orange" => "medium",
            _ => "low",
        };

        events.push(FeedEvent {
            id: format!("gdacs-{i}"),
            lat,
            lon,
            label: title.clone(),
            headline: title,
            source: "GDACS".into(),
            severity: severity.into(),
            category: "weather".into(),
        });
    }
    events
}

// ── NOAA CAP/Atom (polygon centroid) ─────────────────────────────────────────

fn parse_noaa(xml: &str) -> Vec<FeedEvent> {
    let mut events = Vec::new();
    for (i, entry) in xml_blocks(xml, "entry").into_iter().enumerate().take(60) {
        let location = xml_tag(&entry, "cap:polygon")
            .as_deref()
            .and_then(|p| polygon_centroid(p.trim()))
            .or_else(|| {
                xml_tag(&entry, "cap:circle")
                    .as_deref()
                    .and_then(|c| circle_center(c.trim()))
            });

        let Some((lat, lon)) = location else { continue };

        let title = xml_tag(&entry, "title")
            .map(|s| strip_cdata(s.trim()).to_owned())
            .unwrap_or_default();

        let area = xml_tag(&entry, "cap:areaDesc")
            .map(|s| {
                let s = strip_cdata(s.trim());
                s.split(';').next().unwrap_or(s).trim().to_owned()
            })
            .unwrap_or_else(|| title.clone());

        let sev = xml_tag(&entry, "cap:severity")
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();

        let severity = match sev.as_str() {
            "extreme" | "severe" => "high",
            "moderate" => "medium",
            _ => "low",
        };

        events.push(FeedEvent {
            id: format!("noaa-{i}"),
            lat,
            lon,
            label: area,
            headline: title,
            source: "NOAA Alerts".into(),
            severity: severity.into(),
            category: "weather".into(),
        });
    }
    events
}

// ── Generic news RSS/Atom (geocoded from headline text) ───────────────────────

fn parse_rss_news(xml: &str, source_id: &str, category: &str) -> Vec<FeedEvent> {
    // Atom feeds use <entry>, RSS feeds use <item>.
    let block_tag = if xml.contains("<entry>") || xml.contains("<entry ") {
        "entry"
    } else {
        "item"
    };

    let source_label = source_id_to_label(source_id);
    let mut events = Vec::new();

    for (i, block) in xml_blocks(xml, block_tag).into_iter().enumerate().take(40) {
        let title = xml_tag(&block, "title")
            .map(|s| strip_cdata(s.trim()).to_owned())
            .unwrap_or_default();

        if title.is_empty() {
            continue;
        }

        // Try to find a geographic location in the title (+ first part of description).
        let description = xml_tag(&block, "description")
            .or_else(|| xml_tag(&block, "summary"))
            .map(|s| strip_cdata(s.trim()).to_owned())
            .unwrap_or_default();

        // Search title first; fall back to first 120 chars of description.
        let search_text = format!("{title} {}", &description[..description.len().min(120)]);

        let Some((lat, lon, place)) = geocode_from_text(&search_text) else {
            continue; // No recognised location — skip this item.
        };

        events.push(FeedEvent {
            id: format!("{source_id}-{i}"),
            lat,
            lon,
            label: place,
            headline: title,
            source: source_label.clone(),
            severity: "low".into(),
            category: category.to_owned(),
        });
    }
    events
}

fn source_id_to_label(id: &str) -> String {
    match id {
        "bbc-world" => "BBC World",
        "aljazeera" => "Al Jazeera",
        "reuters-world" => "Reuters",
        "france24" => "France 24",
        "dw-world" => "DW World",
        other => other,
    }
    .to_owned()
}

// ── Geocoder ──────────────────────────────────────────────────────────────────

/// Search `text` for known country / city / region names.
/// Returns `(lat, lon, matched_place_name)` for the first (longest) match found.
fn geocode_from_text(text: &str) -> Option<(f64, f64, String)> {
    // Sorted longest-first so "South Sudan" beats "Sudan", etc.
    for &(name, lat, lon) in GEO_LOOKUP {
        // Case-insensitive word-boundary match.
        if text_contains_place(text, name) {
            return Some((lat, lon, name.to_owned()));
        }
    }
    None
}

/// True if `text` contains `place` as a whole word (case-insensitive).
fn text_contains_place(text: &str, place: &str) -> bool {
    let text_lower = text.to_lowercase();
    let place_lower = place.to_lowercase();
    if let Some(idx) = text_lower.find(&place_lower) {
        let before = idx > 0 && text_lower.as_bytes()[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + place_lower.len();
        let after = after_idx < text_lower.len()
            && text_lower.as_bytes()[after_idx].is_ascii_alphanumeric();
        !before && !after
    } else {
        false
    }
}

/// (name, lat, lon) — sorted longest name first to prefer specific matches.
static GEO_LOOKUP: &[(&str, f64, f64)] = &[
    // ── Regions & multi-word countries (must come before shorter prefixes) ──
    ("Democratic Republic of Congo", -4.038, 21.759),
    ("Dominican Republic", 18.736, -70.163),
    ("Papua New Guinea", -6.315, 143.956),
    ("Equatorial Guinea", 1.651, 10.268),
    ("Guinea-Bissau", 11.804, -15.180),
    ("United Arab Emirates", 23.424, 53.848),
    ("United States", 37.090, -95.713),
    ("United Kingdom", 51.509, -0.118),
    ("South Korea", 35.908, 127.767),
    ("North Korea", 40.339, 127.510),
    ("South Sudan", 6.877, 31.307),
    ("Sri Lanka", 7.873, 80.772),
    ("Saudi Arabia", 23.886, 45.079),
    ("Ivory Coast", 7.540, -5.547),
    ("Burkina Faso", 12.364, -1.532),
    ("Côte d'Ivoire", 7.540, -5.547),
    ("Central African Republic", 6.611, 20.939),
    ("Bosnia and Herzegovina", 43.916, 17.679),
    ("Trinidad and Tobago", 10.692, -61.223),
    ("El Salvador", 13.794, -88.897),
    ("Western Sahara", 24.215, -12.886),
    ("New Zealand", -40.900, 174.886),
    ("North Macedonia", 41.609, 21.745),
    ("Sierra Leone", 8.460, -11.779),
    ("Timor-Leste", -8.874, 125.728),
    ("Puerto Rico", 18.221, -66.590),
    ("Costa Rica", 9.749, -83.753),
    ("Czech Republic", 49.818, 15.473),
    ("East Africa", -1.0, 37.0),
    ("West Africa", 12.0, -2.0),
    ("Middle East", 29.0, 42.0),
    ("Southeast Asia", 5.0, 110.0),
    ("Central Asia", 45.0, 65.0),
    ("Horn of Africa", 7.0, 44.0),
    ("Sub-Saharan Africa", 0.0, 25.0),
    ("Latin America", -15.0, -60.0),
    ("Eastern Europe", 52.0, 30.0),
    // ── Countries ────────────────────────────────────────────────────────────
    ("Afghanistan", 33.939, 67.710),
    ("Albania", 41.153, 20.168),
    ("Algeria", 28.034, 1.659),
    ("Angola", -11.202, 17.874),
    ("Argentina", -38.416, -63.617),
    ("Armenia", 40.070, 45.038),
    ("Australia", -25.275, 133.775),
    ("Austria", 47.516, 14.550),
    ("Azerbaijan", 40.143, 47.577),
    ("Bahrain", 26.067, 50.558),
    ("Bangladesh", 23.685, 90.356),
    ("Belarus", 53.709, 27.954),
    ("Belgium", 50.504, 4.469),
    ("Bolivia", -16.291, -63.589),
    ("Brazil", -14.235, -51.925),
    ("Bulgaria", 42.734, 25.486),
    ("Cambodia", 12.565, 104.991),
    ("Cameroon", 3.848, 11.502),
    ("Canada", 56.131, -106.347),
    ("Chad", 15.454, 18.732),
    ("Chile", -35.676, -71.543),
    ("China", 35.861, 104.195),
    ("Colombia", 4.571, -74.297),
    ("Congo", -0.228, 15.827),
    ("Croatia", 45.100, 15.202),
    ("Cuba", 21.521, -77.781),
    ("Ecuador", -1.832, -78.183),
    ("Egypt", 26.820, 30.802),
    ("Ethiopia", 9.145, 40.489),
    ("France", 46.227, 2.214),
    ("Gabon", -0.804, 11.610),
    ("Georgia", 42.315, 43.357),
    ("Germany", 51.165, 10.451),
    ("Ghana", 7.946, -1.023),
    ("Greece", 39.074, 21.824),
    ("Guatemala", 15.784, -90.230),
    ("Guinea", 9.946, -9.697),
    ("Haiti", 18.971, -72.285),
    ("Honduras", 15.200, -86.242),
    ("Hungary", 47.163, 19.503),
    ("India", 20.594, 78.963),
    ("Indonesia", -0.789, 113.921),
    ("Iran", 32.427, 53.688),
    ("Iraq", 33.223, 43.679),
    ("Ireland", 53.418, -8.244),
    ("Israel", 31.046, 34.851),
    ("Italy", 41.874, 12.568),
    ("Jamaica", 18.110, -77.297),
    ("Japan", 36.205, 138.252),
    ("Jordan", 30.586, 36.238),
    ("Kazakhstan", 48.020, 66.923),
    ("Kenya", -0.023, 37.906),
    ("Kosovo", 42.603, 20.903),
    ("Kuwait", 29.311, 47.482),
    ("Kyrgyzstan", 41.205, 74.776),
    ("Laos", 19.856, 102.495),
    ("Lebanon", 33.854, 35.862),
    ("Libya", 26.335, 17.228),
    ("Madagascar", -18.767, 46.869),
    ("Malaysia", 4.211, 101.976),
    ("Mali", 17.570, -3.996),
    ("Mexico", 23.634, -102.552),
    ("Moldova", 47.412, 28.370),
    ("Morocco", 31.791, -7.092),
    ("Mozambique", -18.665, 35.530),
    ("Myanmar", 16.871, 96.083),
    ("Namibia", -22.957, 18.490),
    ("Nepal", 28.394, 84.124),
    ("Netherlands", 52.133, 5.291),
    ("Nicaragua", 12.865, -85.207),
    ("Niger", 17.607, 8.081),
    ("Nigeria", 9.082, 8.675),
    ("Norway", 60.472, 8.469),
    ("Oman", 21.513, 55.923),
    ("Pakistan", 30.375, 69.345),
    ("Palestine", 31.952, 35.233),
    ("Panama", 8.538, -80.782),
    ("Paraguay", -23.443, -58.444),
    ("Peru", -9.190, -75.015),
    ("Philippines", 12.880, 121.774),
    ("Poland", 51.919, 19.145),
    ("Portugal", 39.400, -8.224),
    ("Qatar", 25.354, 51.184),
    ("Romania", 45.943, 24.967),
    ("Russia", 61.524, 105.318),
    ("Rwanda", -1.940, 29.874),
    ("Senegal", 14.497, -14.452),
    ("Serbia", 44.017, 21.006),
    ("Slovakia", 48.669, 19.699),
    ("Somalia", 5.152, 46.199),
    ("Spain", 40.464, -3.750),
    ("Sudan", 12.862, 30.217),
    ("Sweden", 60.128, 18.644),
    ("Switzerland", 46.818, 8.228),
    ("Syria", 34.802, 38.996),
    ("Taiwan", 23.698, 120.961),
    ("Tajikistan", 38.861, 71.276),
    ("Tanzania", -6.369, 34.889),
    ("Thailand", 15.870, 100.993),
    ("Tunisia", 33.887, 9.537),
    ("Turkey", 38.964, 35.243),
    ("Turkmenistan", 38.970, 59.556),
    ("Uganda", 1.374, 32.290),
    ("Ukraine", 48.379, 31.165),
    ("Uruguay", -32.523, -55.765),
    ("Uzbekistan", 41.377, 64.585),
    ("Venezuela", 6.424, -66.590),
    ("Vietnam", 14.058, 108.277),
    ("Yemen", 15.552, 48.516),
    ("Zambia", -13.133, 27.849),
    ("Zimbabwe", -19.015, 29.155),
    // ── Cities (major news datelines) ────────────────────────────────────────
    ("Kyiv", 50.450, 30.523),
    ("Moscow", 55.751, 37.618),
    ("Beijing", 39.904, 116.407),
    ("Washington", 38.907, -77.037),
    ("London", 51.507, -0.128),
    ("Paris", 48.857, 2.352),
    ("Berlin", 52.520, 13.405),
    ("Gaza", 31.501, 34.466),
    ("Jerusalem", 31.769, 35.217),
    ("Tel Aviv", 32.085, 34.782),
    ("Tehran", 35.690, 51.388),
    ("Kabul", 34.516, 69.177),
    ("Islamabad", 33.684, 73.048),
    ("Nairobi", -1.286, 36.818),
    ("Cairo", 30.044, 31.236),
    ("Tripoli", 32.899, 13.179),
    ("Khartoum", 15.500, 32.560),
    ("Mogadishu", 2.047, 45.343),
    ("Sanaa", 15.369, 44.191),
    ("Damascus", 33.510, 36.292),
    ("Baghdad", 33.343, 44.401),
    ("Beirut", 33.890, 35.501),
    ("Riyadh", 24.688, 46.724),
    ("Ankara", 39.920, 32.854),
    ("Istanbul", 41.013, 28.977),
    ("Taipei", 25.033, 121.565),
    ("Seoul", 37.566, 126.978),
    ("Pyongyang", 39.019, 125.755),
    ("Tokyo", 35.690, 139.692),
    ("Jakarta", -6.208, 106.845),
    ("Manila", 14.599, 120.984),
    ("Rangoon", 16.866, 96.195),
    ("Yangon", 16.866, 96.195),
    ("Caracas", 10.480, -66.904),
    ("Havana", 23.136, -82.359),
    ("Bogotá", 4.711, -74.072),
    ("Addis Ababa", 9.032, 38.747),
    ("Kinshasa", -4.322, 15.322),
    ("Bamako", 12.650, -8.000),
    ("Ouagadougou", 12.372, -1.526),
];

// ── XML helpers ───────────────────────────────────────────────────────────────

fn xml_blocks(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find(&open) {
        let slice = &rest[start..];
        let Some(tag_end) = slice.find('>') else { break };
        let content_start = tag_end + 1;
        let Some(end) = slice[content_start..].find(&close) else { break };
        out.push(slice[content_start..content_start + end].to_owned());
        rest = &slice[content_start + end + close.len()..];
    }
    out
}

fn xml_tag(block: &str, tag: &str) -> Option<String> {
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut rest = block;
    loop {
        let idx = rest.find(&open_prefix)?;
        let after_name = &rest[idx + open_prefix.len()..];
        let next = after_name.chars().next()?;
        if matches!(next, '>' | ' ' | '/' | '\n' | '\r' | '\t') {
            let tag_end = idx + open_prefix.len() + after_name.find('>')? + 1;
            let end = rest[tag_end..].find(&close)?;
            return Some(rest[tag_end..tag_end + end].to_owned());
        }
        rest = &rest[idx + open_prefix.len()..];
    }
}

fn strip_cdata(s: &str) -> &str {
    s.strip_prefix("<![CDATA[")
        .and_then(|s| s.strip_suffix("]]>"))
        .unwrap_or(s)
}

fn polygon_centroid(polygon: &str) -> Option<(f64, f64)> {
    let points: Vec<(f64, f64)> = polygon
        .split_whitespace()
        .filter_map(|pair| {
            let mut it = pair.split(',');
            Some((it.next()?.trim().parse().ok()?, it.next()?.trim().parse().ok()?))
        })
        .collect();
    if points.is_empty() {
        return None;
    }
    let n = points.len() as f64;
    Some((
        points.iter().map(|p| p.0).sum::<f64>() / n,
        points.iter().map(|p| p.1).sum::<f64>() / n,
    ))
}

fn circle_center(circle: &str) -> Option<(f64, f64)> {
    let pair = circle.split_whitespace().next()?;
    let mut it = pair.split(',');
    Some((it.next()?.trim().parse().ok()?, it.next()?.trim().parse().ok()?))
}
