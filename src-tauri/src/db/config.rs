use std::fs;
use std::env::home_dir;
use std::path::PathBuf;

pub fn get_app_data_dir() -> Result<PathBuf, std::io::Error> {
    let home_dir = home_dir().expect("Could not find home directory.");
    
    let app_dir = home_dir.join(".floaty");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    Ok(app_dir)
}

pub fn get_database_path() -> Result<String, std::io::Error> {
    let app_dir = get_app_data_dir()?;
    let db_path = app_dir.join("app.db");
    Ok(db_path.to_string_lossy().to_string())
}
