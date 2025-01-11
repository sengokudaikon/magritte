#[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
pub type RwLock<T: ?Sized> = std::sync::RwLock<T>;

#[cfg(feature = "rt-tokio")]
pub type RwLock<T: ?Sized> = tokio::sync::RwLock<T>;

#[cfg(feature = "rt-async-std")]
pub type RwLock<T: ?Sized> = async_std::sync::RwLock<T>;