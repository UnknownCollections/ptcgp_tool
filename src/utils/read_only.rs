#[repr(transparent)]
/// A transparent wrapper that provides read-only access to an inner value.
///
/// This wrapper encapsulates a value of type `T` and exposes only immutable
/// access via the `Deref` trait. The underlying value can be retrieved by consuming
/// the wrapper using the [`take`] method.
///
/// # Field
///
/// * `0` - The encapsulated value.
pub struct ReadOnly<T>(#[doc = "The encapsulated value."] T);

impl<T> ReadOnly<T> {
    /// Creates a new `ReadOnly` instance that wraps the provided value.
    ///
    /// # Arguments
    ///
    /// * `inner` - The value to be encapsulated.
    pub fn new(inner: T) -> Self {
        ReadOnly(inner)
    }

    /// Consumes the `ReadOnly` wrapper and returns the inner value.
    ///
    /// This method removes the read-only wrapper, yielding ownership of the encapsulated value.
    pub fn take(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for ReadOnly<T> {
    type Target = T;

    /// Returns an immutable reference to the encapsulated value.
    ///
    /// This allows the `ReadOnly` wrapper to be used in contexts where an immutable
    /// reference to `T` is required.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implements `Send` for `ReadOnly<T>`.
///
/// # Safety
///
/// It is safe to send `ReadOnly<T>` across threads because the wrapper is transparent.
/// This implementation assumes that sending `T` to another thread does not violate any invariants.
unsafe impl<T> Send for ReadOnly<T> {}

/// Implements `Sync` for `ReadOnly<T>`.
///
/// # Safety
///
/// It is safe to share references to `ReadOnly<T>` across threads because the wrapper
/// only provides immutable access. This implementation assumes that sharing `T` immutably
/// across threads does not violate any invariants.
unsafe impl<T> Sync for ReadOnly<T> {}
