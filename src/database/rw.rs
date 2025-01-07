#[cfg(not(any(feature = "may", feature = "tokio", feature = "glommio", feature = "async_std")))]
pub type RwLock<T: ?Sized> = std::sync::RwLock<T>;

#[cfg(feature = "may")]
pub type RwLock<T: ?Sized> = may::sync::RwLock<T>;

#[cfg(feature = "tokio")]
pub type RwLock<T: ?Sized> = tokio::sync::RwLock<T>;

#[cfg(feature = "glommio")]
pub type RwLock<T: ?Sized> = glommio::sync::RwLock<T>;

#[cfg(feature = "async_std")]
pub type RwLock<T: ?Sized> = async_std::sync::RwLock<T>;