pub mod progress;

/// The default amount of parallelism to use.
const DEFAULT_PARALLELISM: usize = 12;

/// Gets the default amount of parallelism to use.
///
/// If the value could not be obtained, defaults to ``crate::constants::DEFAULT_PARALLELISM``
pub fn available_parallelism() -> usize {
    if let Ok(available) = std::thread::available_parallelism() {
        available.get()
    } else {
        DEFAULT_PARALLELISM
    }
}