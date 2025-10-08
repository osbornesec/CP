//! Utility functions for the `CPInfo` parser

pub mod conversions;

use crate::error::{CpinfoError, Result};
use std::sync::{Mutex, MutexGuard};

/// Safe mutex access with proper error handling
///
/// This function provides centralized error handling for mutex operations,
/// converting mutex poisoning errors into proper `CpinfoError::MutexPoisoned` variants.
///
/// # Arguments
/// * `mutex` - The mutex to lock
///
/// # Errors
///
/// Returns a `CpinfoError::MutexPoisoned` if the mutex is poisoned.
///
/// # Examples
/// ```
/// use std::sync::Mutex;
/// use cpinfo_parser::utils::safe_mutex_lock;
///
/// let data = Mutex::new(42);
/// match safe_mutex_lock(&data) {
///     Ok(guard) => println!("Value: {}", *guard),
///     Err(e) => eprintln!("Mutex error: {}", e),
/// }
/// ```
#[inline]
pub fn safe_mutex_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>> {
    return match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(_poison_err) => return Err(CpinfoError::mutex_poisoned()),
    };
}
