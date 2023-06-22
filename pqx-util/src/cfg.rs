//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/06/04 09:19:42 Sunday
//! brief:

use std::fs::File;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::PqxUtilResult;

pub fn read_yaml<P, T>(path: P) -> PqxUtilResult<T>
where
    P: AsRef<str>,
    T: DeserializeOwned,
{
    let f = File::open(path.as_ref())?;
    let d: T = serde_yaml::from_reader(f)?;

    Ok(d)
}

pub fn read_json<P, T>(path: P) -> PqxUtilResult<T>
where
    P: AsRef<str>,
    T: DeserializeOwned,
{
    let f = File::open(path.as_ref())?;
    let d: T = serde_json::from_reader(f)?;

    Ok(d)
}

// ================================================================================================
// Path
// ================================================================================================

pub fn current_dir() -> PqxUtilResult<PathBuf> {
    let dir = std::env::current_dir()?;

    Ok(dir)
}

pub fn parent_dir(pb: PathBuf) -> PqxUtilResult<PathBuf> {
    let dir = pb.parent().ok_or("parent path fail")?.to_owned();

    Ok(dir)
}

pub fn join_dir<L, R>(lhs: L, rhs: R) -> PqxUtilResult<PathBuf>
where
    L: AsRef<Path>,
    R: AsRef<Path>,
{
    Ok(lhs.as_ref().join(rhs))
}

pub fn get_cur_dir_file(filename: &str) -> PqxUtilResult<PathBuf> {
    Ok(join_dir(current_dir()?, filename)?)
}
