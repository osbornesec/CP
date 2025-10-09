//! Utility functions for the `CPInfo` parser

pub mod conversions;

use crate::error::{CpinfoError, Result};
use std::sync::{Mutex, MutexGuard};

/// Acquire a lock on the given mutex, translating a poisoned mutex into `CpinfoError::mutex_poisoned()`.
///
/// # Returns
///
/// `Ok(MutexGuard<'_, T>)` containing the guard if the lock was acquired, `Err(CpinfoError::mutex_poisoned())` if the mutex is poisoned.
///
/// # Examples
///
/// ```
/// use std::sync::Mutex;
/// use cpinfo_parser::utils::safe_mutex_lock;
///
/// let data = Mutex::new(42);
/// let guard = safe_mutex_lock(&data).expect("mutex should not be poisoned");
/// assert_eq!(*guard, 42);
/// ```
#[inline]
pub fn safe_mutex_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    return match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(_poison_err) => Err(CpinfoError::mutex_poisoned()),
    };
}