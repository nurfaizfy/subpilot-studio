use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub original_language: String,
    pub target_language: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub output_folder: String,
    pub source_video: String,
    pub thumbnail: String,
    pub last_modified: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssStyle {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Fontname")]
    pub fontname: String,
    #[serde(rename = "Fontsize")]
    pub fontsize: String,
    #[serde(rename = "PrimaryColour")]
    pub primary_colour: String,
    #[serde(rename = "SecondaryColour")]
    pub secondary_colour: String,
    #[serde(rename = "OutlineColour")]
    pub outline_colour: String,
    #[serde(rename = "BackColour")]
    pub back_colour: String,
    #[serde(rename = "Bold")]
    pub bold: String,
    #[serde(rename = "Italic")]
    pub italic: String,
    #[serde(rename = "Underline")]
    pub underline: String,
    #[serde(rename = "StrikeOut")]
    pub strikeout: String,
    #[serde(rename = "ScaleX")]
    pub scale_x: String,
    #[serde(rename = "ScaleY")]
    pub scale_y: String,
    #[serde(rename = "Spacing")]
    pub spacing: String,
    #[serde(rename = "Angle")]
    pub angle: String,
    #[serde(rename = "BorderStyle")]
    pub border_style: String,
    #[serde(rename = "Outline")]
    pub outline: String,
    #[serde(rename = "Shadow")]
    pub shadow: String,
    #[serde(rename = "Alignment")]
    pub alignment: String,
    #[serde(rename = "MarginL")]
    pub margin_l: String,
    #[serde(rename = "MarginR")]
    pub margin_r: String,
    #[serde(rename = "MarginV")]
    pub margin_v: String,
    #[serde(rename = "Encoding")]
    pub encoding: String,
}

pub fn init_db(app_dir: &PathBuf) -> Result<Connection> {
    let db_path = app_dir.join("subpilot.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            project_type TEXT NOT NULL,
            original_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            season INTEGER,
            episode INTEGER,
            output_folder TEXT NOT NULL,
            source_video TEXT NOT NULL,
            thumbnail TEXT NOT NULL,
            last_modified TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS dictionary (
            id TEXT PRIMARY KEY,
            original TEXT NOT NULL,
            translation TEXT NOT NULL,
            type TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM dictionary", [], |row| row.get(0))?;
    if count == 0 {
        let defaults = vec![
            ("先生", "Sensei", "Honorific", 10),
            ("先輩", "Senpai", "Honorific", 10),
            ("後輩", "Kouhai", "Honorific", 10),
            ("さん", "-san", "Honorific", 5),
            ("くん", "-kun", "Honorific", 5),
            ("ちゃん", "-chan", "Honorific", 5),
            ("社長", "Presiden", "Business", 8),
            ("課長", "Manajer", "Business", 8),
            ("部長", "Kepala Bagian", "Business", 8),
        ];

        for (orig, trans, dtype, prio) in defaults {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO dictionary (id, original, translation, type, priority) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, orig, trans, dtype, prio],
            )?;
        }
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            api_url TEXT NOT NULL,
            default_model TEXT NOT NULL,
            is_fallback INTEGER NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;

    let provider_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM ai_providers", [], |row| row.get(0))?;
    if provider_count == 0 {
        let defaults = vec![
            (
                "gemini",
                "Gemini",
                "https://generativelanguage.googleapis.com/v1beta/models",
                "gemini-2.5-flash",
                0,
                10,
            ),
            (
                "openai",
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                "gpt-4o-mini",
                1,
                9,
            ),
            (
                "openrouter",
                "OpenRouter",
                "https://openrouter.ai/api/v1/chat/completions",
                "google/gemini-flash-2.5",
                1,
                8,
            ),
            (
                "deepseek",
                "DeepSeek",
                "https://api.deepseek.com/v1/chat/completions",
                "deepseek-chat",
                1,
                7,
            ),
            (
                "ollama",
                "Ollama",
                "http://localhost:11434/api/chat",
                "llama3",
                1,
                6,
            ),
            (
                "lmstudio",
                "LM Studio",
                "http://localhost:1234/v1/chat/completions",
                "local-model",
                1,
                5,
            ),
        ];

        for (id, name, url, model, fallback, prio) in defaults {
            conn.execute(
                "INSERT INTO ai_providers (id, name, api_url, default_model, is_fallback, priority) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id.to_string(), name, url, model, fallback, prio],
            )?;
        }
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS ass_styles (
            name TEXT PRIMARY KEY,
            fontname TEXT NOT NULL,
            fontsize TEXT NOT NULL,
            primary_colour TEXT NOT NULL,
            secondary_colour TEXT NOT NULL,
            outline_colour TEXT NOT NULL,
            back_colour TEXT NOT NULL,
            bold TEXT NOT NULL,
            italic TEXT NOT NULL,
            underline TEXT NOT NULL,
            strikeout TEXT NOT NULL,
            scale_x TEXT NOT NULL,
            scale_y TEXT NOT NULL,
            spacing TEXT NOT NULL,
            angle TEXT NOT NULL,
            border_style TEXT NOT NULL,
            outline TEXT NOT NULL,
            shadow TEXT NOT NULL,
            alignment TEXT NOT NULL,
            margin_l TEXT NOT NULL,
            margin_r TEXT NOT NULL,
            margin_v TEXT NOT NULL,
            encoding TEXT NOT NULL
        )",
        [],
    )?;

    let style_count: i64 = conn.query_row("SELECT COUNT(*) FROM ass_styles", [], |row| row.get(0))?;
    if style_count == 0 {
        conn.execute(
            "INSERT INTO ass_styles (
                name, fontname, fontsize, primary_colour, secondary_colour, outline_colour, back_colour,
                bold, italic, underline, strikeout, scale_x, scale_y, spacing, angle, border_style,
                outline, shadow, alignment, margin_l, margin_r, margin_v, encoding
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
            )",
            params![
                "Default", "Arial", "48", "&H00FFFFFF", "&H000000FF", "&H00000000", "&H00000000",
                "0", "0", "0", "0", "100", "100", "0", "0", "1", "2", "2", "2", "10", "10", "10", "1"
            ],
        )?;
    }

    Ok(conn)
}

pub fn create_project(conn: &Connection, mut project: Project) -> Result<Project> {
    project.id = Uuid::new_v4().to_string();
    project.last_modified = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO projects (id, name, project_type, original_language, target_language, season, episode, output_folder, source_video, thumbnail, last_modified)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            project.id,
            project.name,
            project.project_type,
            project.original_language,
            project.target_language,
            project.season,
            project.episode,
            project.output_folder,
            project.source_video,
            project.thumbnail,
            project.last_modified
        ],
    )?;

    Ok(project)
}

pub fn update_project(conn: &Connection, mut project: Project) -> Result<Project> {
    project.last_modified = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE projects SET 
            name = ?1, 
            project_type = ?2, 
            original_language = ?3, 
            target_language = ?4, 
            season = ?5, 
            episode = ?6, 
            output_folder = ?7, 
            source_video = ?8, 
            thumbnail = ?9, 
            last_modified = ?10
         WHERE id = ?11",
        params![
            project.name,
            project.project_type,
            project.original_language,
            project.target_language,
            project.season,
            project.episode,
            project.output_folder,
            project.source_video,
            project.thumbnail,
            project.last_modified,
            project.id
        ],
    )?;

    Ok(project)
}

pub fn get_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT id, name, project_type, original_language, target_language, season, episode, output_folder, source_video, thumbnail, last_modified FROM projects ORDER BY last_modified DESC")?;
    let project_iter = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            project_type: row.get(2)?,
            original_language: row.get(3)?,
            target_language: row.get(4)?,
            season: row.get(5)?,
            episode: row.get(6)?,
            output_folder: row.get(7)?,
            source_video: row.get(8)?,
            thumbnail: row.get(9)?,
            last_modified: row.get(10)?,
        })
    })?;

    let mut projects = Vec::new();
    for project in project_iter {
        projects.push(project?);
    }

    Ok(projects)
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn duplicate_project(conn: &Connection, id: &str) -> Result<Project> {
    let mut stmt = conn.prepare("SELECT id, name, project_type, original_language, target_language, season, episode, output_folder, source_video, thumbnail, last_modified FROM projects WHERE id = ?1")?;
    let mut project = stmt.query_row(params![id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            project_type: row.get(2)?,
            original_language: row.get(3)?,
            target_language: row.get(4)?,
            season: row.get(5)?,
            episode: row.get(6)?,
            output_folder: row.get(7)?,
            source_video: row.get(8)?,
            thumbnail: row.get(9)?,
            last_modified: row.get(10)?,
        })
    })?;

    project.name = format!("{} (Copy)", project.name);
    create_project(conn, project)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    pub id: String,
    pub original: String,
    pub translation: String,
    pub entry_type: String,
    pub priority: i32,
}

pub fn get_dictionary(conn: &Connection) -> Result<Vec<DictionaryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, original, translation, type, priority FROM dictionary ORDER BY priority DESC",
    )?;
    let entry_iter = stmt.query_map([], |row| {
        Ok(DictionaryEntry {
            id: row.get(0)?,
            original: row.get(1)?,
            translation: row.get(2)?,
            entry_type: row.get(3)?,
            priority: row.get(4)?,
        })
    })?;

    let mut entries = Vec::new();
    for entry in entry_iter {
        entries.push(entry?);
    }
    Ok(entries)
}

pub fn add_dictionary_entry(
    conn: &Connection,
    mut entry: DictionaryEntry,
) -> Result<DictionaryEntry> {
    entry.id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO dictionary (id, original, translation, type, priority) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entry.id, entry.original, entry.translation, entry.entry_type, entry.priority],
    )?;
    Ok(entry)
}

pub fn update_dictionary_entry(
    conn: &Connection,
    entry: DictionaryEntry,
) -> Result<DictionaryEntry> {
    conn.execute(
        "UPDATE dictionary SET original = ?1, translation = ?2, type = ?3, priority = ?4 WHERE id = ?5",
        params![entry.original, entry.translation, entry.entry_type, entry.priority, entry.id],
    )?;
    Ok(entry)
}

pub fn delete_dictionary_entry(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM dictionary WHERE id = ?1", params![id])?;
    Ok(())
}


pub fn get_ass_styles(conn: &Connection) -> Result<Vec<AssStyle>> {
    let mut stmt = conn.prepare("SELECT name, fontname, fontsize, primary_colour, secondary_colour, outline_colour, back_colour, bold, italic, underline, strikeout, scale_x, scale_y, spacing, angle, border_style, outline, shadow, alignment, margin_l, margin_r, margin_v, encoding FROM ass_styles")?;
    let style_iter = stmt.query_map([], |row| {
        Ok(AssStyle {
            name: row.get(0)?,
            fontname: row.get(1)?,
            fontsize: row.get(2)?,
            primary_colour: row.get(3)?,
            secondary_colour: row.get(4)?,
            outline_colour: row.get(5)?,
            back_colour: row.get(6)?,
            bold: row.get(7)?,
            italic: row.get(8)?,
            underline: row.get(9)?,
            strikeout: row.get(10)?,
            scale_x: row.get(11)?,
            scale_y: row.get(12)?,
            spacing: row.get(13)?,
            angle: row.get(14)?,
            border_style: row.get(15)?,
            outline: row.get(16)?,
            shadow: row.get(17)?,
            alignment: row.get(18)?,
            margin_l: row.get(19)?,
            margin_r: row.get(20)?,
            margin_v: row.get(21)?,
            encoding: row.get(22)?,
        })
    })?;

    let mut styles = Vec::new();
    for style in style_iter {
        styles.push(style?);
    }
    Ok(styles)
}

pub fn save_ass_style(conn: &Connection, style: AssStyle) -> Result<AssStyle> {
    conn.execute(
        "INSERT INTO ass_styles (
            name, fontname, fontsize, primary_colour, secondary_colour, outline_colour, back_colour,
            bold, italic, underline, strikeout, scale_x, scale_y, spacing, angle, border_style,
            outline, shadow, alignment, margin_l, margin_r, margin_v, encoding
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
        )
        ON CONFLICT(name) DO UPDATE SET
            fontname=excluded.fontname,
            fontsize=excluded.fontsize,
            primary_colour=excluded.primary_colour,
            secondary_colour=excluded.secondary_colour,
            outline_colour=excluded.outline_colour,
            back_colour=excluded.back_colour,
            bold=excluded.bold,
            italic=excluded.italic,
            underline=excluded.underline,
            strikeout=excluded.strikeout,
            scale_x=excluded.scale_x,
            scale_y=excluded.scale_y,
            spacing=excluded.spacing,
            angle=excluded.angle,
            border_style=excluded.border_style,
            outline=excluded.outline,
            shadow=excluded.shadow,
            alignment=excluded.alignment,
            margin_l=excluded.margin_l,
            margin_r=excluded.margin_r,
            margin_v=excluded.margin_v,
            encoding=excluded.encoding",
        params![
            style.name, style.fontname, style.fontsize, style.primary_colour, style.secondary_colour, style.outline_colour, style.back_colour,
            style.bold, style.italic, style.underline, style.strikeout, style.scale_x, style.scale_y, style.spacing, style.angle, style.border_style,
            style.outline, style.shadow, style.alignment, style.margin_l, style.margin_r, style.margin_v, style.encoding
        ],
    )?;
    Ok(style)
}

pub fn delete_ass_style(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM ass_styles WHERE name = ?1", params![name])?;
    Ok(())
}

