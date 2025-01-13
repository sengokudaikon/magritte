use crate::entity::manager::EntityState;
use crate::{HasId, TableTrait};
use std::any::TypeId;
use anyhow::{Result, anyhow};
use magritte_query::database::SurrealDB;

/// Raw pointer wrapper for type-erased but safe access
pub struct TypeErasedState {
    ptr: *mut (),
    drop: unsafe fn(*mut ()),
    type_id: TypeId,
}

impl TypeErasedState {
    pub fn new<T>(state: EntityState<T>) -> Self 
    where 
        T: TableTrait + HasId + 'static,
    {
        unsafe fn drop_impl<T: TableTrait>(ptr: *mut ()) {
            drop(Box::from_raw(ptr as *mut EntityState<T>));
        }

        let boxed = Box::new(state);
        Self {
            ptr: Box::into_raw(boxed) as *mut (),
            drop: drop_impl::<T>,
            type_id: TypeId::of::<T>(),
        }
    }

    /// # Safety
    ///
    /// The caller must ensure the type is correct
    pub unsafe fn as_ref<T: TableTrait>(&self) -> Option<&EntityState<T>> {
        if TypeId::of::<T>() == self.type_id {
            Some(&*(self.ptr as *const EntityState<T>))
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must ensure the type is correct
    pub unsafe fn as_mut<T: TableTrait>(&mut self) -> Option<&mut EntityState<T>> {
        if TypeId::of::<T>() == self.type_id {
            Some(&mut *(self.ptr as *mut EntityState<T>))
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must ensure the type is registered and compatible
    pub unsafe fn flush_with_db(&mut self, db: &SurrealDB) -> Result<()> {
        let flusher = super::registry::get_flusher_for_type(self.type_id)
            .ok_or_else(|| anyhow!("No flusher found for type"))?;
        flusher.flush(self.ptr, db)
    }
}

impl Drop for TypeErasedState {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.ptr) }
    }
}

// Safety: The TypeErasedState ensures proper type checking and memory management
unsafe impl Send for TypeErasedState {}
unsafe impl Sync for TypeErasedState {} 