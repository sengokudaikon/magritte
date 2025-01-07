#[cfg(not(any(feature = "rt-may", feature = "rt-tokio", feature = "rt-glommio", feature = "rt-async-std")))]
pub type RwLock<T: ?Sized> = std::sync::RwLock<T>;

#[cfg(feature = "rt-may")]
pub type RwLock<T: ?Sized> = may::sync::RwLock<T>;

#[cfg(feature = "rt-tokio")]
pub type RwLock<T: ?Sized> = tokio::sync::RwLock<T>;

#[cfg(feature = "rt-glommio")]
pub type RwLock<T: ?Sized> = glommio::sync::RwLock<T>;

#[cfg(feature = "rt-async-std")]
pub type RwLock<T: ?Sized> = async_std::sync::RwLock<T>;