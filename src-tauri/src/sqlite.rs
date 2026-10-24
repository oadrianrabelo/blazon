//! Minimal read-only access to SQLite through winsqlite3.dll, which ships with
//! Windows 10 and later, so importing history adds nothing to the app's size.
//! Browsers keep their databases locked while running, so files are copied
//! (with their WAL journal) to a temporary folder before being opened.

use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    fs,
    path::Path,
};

#[link(name = "winsqlite3")]
extern "system" {
    fn sqlite3_open_v2(
        filename: *const c_char,
        db: *mut *mut c_void,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(db: *mut c_void) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut c_void,
        sql: *const c_char,
        n: c_int,
        stmt: *mut *mut c_void,
        tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_step(stmt: *mut c_void) -> c_int;
    fn sqlite3_finalize(stmt: *mut c_void) -> c_int;
    fn sqlite3_column_count(stmt: *mut c_void) -> c_int;
    fn sqlite3_column_type(stmt: *mut c_void, col: c_int) -> c_int;
    fn sqlite3_column_int64(stmt: *mut c_void, col: c_int) -> i64;
    fn sqlite3_column_text(stmt: *mut c_void, col: c_int) -> *const u8;
    fn sqlite3_column_blob(stmt: *mut c_void, col: c_int) -> *const c_void;
    fn sqlite3_column_bytes(stmt: *mut c_void, col: c_int) -> c_int;
}

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_OPEN_READONLY: c_int = 0x1;
const SQLITE_INTEGER: c_int = 1;
const SQLITE_BLOB: c_int = 4;
const SQLITE_NULL: c_int = 5;

#[derive(Debug, Clone)]
pub enum Cell {
    Int(i64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl Cell {
    pub fn text(&self) -> String {
        match self {
            Cell::Text(s) => s.clone(),
            Cell::Int(i) => i.to_string(),
            Cell::Blob(b) => String::from_utf8_lossy(b).into_owned(),
            Cell::Null => String::new(),
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Cell::Blob(b) => b.clone(),
            Cell::Text(s) => s.as_bytes().to_vec(),
            _ => vec![],
        }
    }

    pub fn int(&self) -> i64 {
        match self {
            Cell::Int(i) => *i,
            Cell::Text(s) => s.parse().unwrap_or(0),
            Cell::Blob(_) | Cell::Null => 0,
        }
    }
}

/// Runs `sql` on a private copy of the database at `path`.
pub fn query(path: &Path, sql: &str) -> Result<Vec<Vec<Cell>>, String> {
    let dir = std::env::temp_dir().join(format!("blazon-import-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let copy = dir.join("db.sqlite");
    fs::copy(path, &copy).map_err(|_| {
        "Não deu para ler o arquivo do navegador. Feche o navegador e tente de novo.".to_string()
    })?;
    let wal = path.with_file_name(format!(
        "{}-wal",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    if wal.is_file() {
        let _ = fs::copy(&wal, dir.join("db.sqlite-wal"));
    }
    let result = run(&copy, sql);
    let _ = fs::remove_dir_all(&dir);
    result
}

fn run(path: &Path, sql: &str) -> Result<Vec<Vec<Cell>>, String> {
    let name = CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    let sql = CString::new(sql).map_err(|e| e.to_string())?;
    let mut rows = vec![];
    unsafe {
        let mut db = std::ptr::null_mut();
        if sqlite3_open_v2(
            name.as_ptr(),
            &mut db,
            SQLITE_OPEN_READONLY,
            std::ptr::null(),
        ) != SQLITE_OK
        {
            sqlite3_close(db);
            return Err("Banco de dados do navegador inválido.".into());
        }
        let mut stmt = std::ptr::null_mut();
        if sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()) != SQLITE_OK {
            sqlite3_close(db);
            return Err("Formato de banco de dados desconhecido.".into());
        }
        let cols = sqlite3_column_count(stmt);
        while sqlite3_step(stmt) == SQLITE_ROW {
            let mut row = Vec::with_capacity(cols as usize);
            for c in 0..cols {
                row.push(match sqlite3_column_type(stmt, c) {
                    SQLITE_NULL => Cell::Null,
                    SQLITE_INTEGER => Cell::Int(sqlite3_column_int64(stmt, c)),
                    SQLITE_BLOB => {
                        let p = sqlite3_column_blob(stmt, c) as *const u8;
                        let n = sqlite3_column_bytes(stmt, c).max(0) as usize;
                        if p.is_null() {
                            Cell::Blob(vec![])
                        } else {
                            Cell::Blob(std::slice::from_raw_parts(p, n).to_vec())
                        }
                    }
                    _ => {
                        let p = sqlite3_column_text(stmt, c);
                        if p.is_null() {
                            Cell::Null
                        } else {
                            Cell::Text(
                                CStr::from_ptr(p as *const c_char)
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                        }
                    }
                });
            }
            rows.push(row);
        }
        sqlite3_finalize(stmt);
        sqlite3_close(db);
    }
    Ok(rows)
}
