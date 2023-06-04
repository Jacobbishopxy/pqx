//! file: cfg.rs
//! author: Jacob Xie
//! date: 2023/06/04 09:19:42 Sunday
//! brief:

use std::fs::File;

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
