use std::path::{Path, PathBuf};

use anyhow::{bail, Result};


use rusqlite::{
    types::{Value as SqlValue}, ToSql,
};

use super::debugdb::DebugData;

pub trait Extract {
    fn vto_string(&self) -> Result<String>;
    fn vto_i64(&self) -> Result<i64>;
}

impl Extract for SqlValue {
    fn vto_string(&self) -> Result<String> {
        match self {
            SqlValue::Text(s) => Ok(s.clone()),
            _ => bail!("expected a string"),
        }
    }
    fn vto_i64(&self) -> Result<i64> {
        match self {
            SqlValue::Integer(i) => Ok(*i),
            _ => bail!("expected a number"),
        }
    }
}

impl DebugData {
    pub fn guess_cc65_dir(&self) -> Result<Option<PathBuf>> {
        let ans = self.query_db(&[], "select name from file")?;
        for row in ans {
            if let SqlValue::Text(s) = &row[0] {
                let path = Path::new(s);
                if path.is_absolute() {
                    if let Some(p) = path.parent() {
                        if p.ends_with("include") {
                            return Ok(Some(p.parent().unwrap().to_path_buf()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn find_file(&self, file: &Path) -> Result<Option<PathBuf>> {
        // find a file somewhere
        if file.is_absolute() {
            if file.exists() {
                return Ok(Some(file.to_path_buf()));
            }
        } else {
            if let Some(cc65) = &self.cc65_dir {
                let mut p = PathBuf::new();
                p.push(cc65);
                p.push("libsrc");
                p.push(file);
                if p.exists() {
                    return Ok(Some(p));
                }
            }
            if file.exists() {
                return Ok(Some(file.canonicalize()?));
            }
        }
        Ok(None)
    }
    // general purpose query function
    pub fn query_db(&self, params: &[&dyn ToSql], query: &str) -> Result<Vec<Vec<SqlValue>>> {
        let mut stmt = self.conn.prepare_cached(query)?;
        let cols = stmt.column_count();

        for (i, p) in params.iter().enumerate() {
            stmt.raw_bind_parameter(i + 1, p)?;
        }
        let mut rows = stmt.raw_query();
        let mut result = Vec::new();
        while let Some(r) = rows.next()? {
            let mut row_vec = Vec::new();
            for i in 0..cols {
                let val = r.get::<usize, SqlValue>(i).unwrap();
                row_vec.push(val);
            }
            result.push(row_vec);
        }

        Ok(result)
    }
}
