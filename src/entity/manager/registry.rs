use crate::{HasId, TableTrait};
use anyhow::Result;
use inventory::{collect, iter};
use std::any::TypeId;
use super::macros::EntityFlusherRegistration;

/// Registration for entity proxy generation and type-safe state management
#[derive(Clone)]
pub struct EntityProxyRegistration {
    pub type_id: TypeId,
}

impl EntityProxyRegistration {
    pub const fn new<T: TableTrait + HasId + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }
}

inventory::collect!(EntityProxyRegistration);
inventory::collect!(EntityFlusherRegistration);

pub fn get_registered_entities() -> Vec<TypeId> {
    iter::<EntityProxyRegistration>()
        .map(|reg| reg.type_id)
        .collect()
}

pub fn get_flusher_for_type(type_id: TypeId) -> Option<&'static dyn super::macros::EntityStateFlusher> {
    iter::<EntityFlusherRegistration>()
        .find(|reg| (reg.get_flusher)().type_id() == type_id)
        .map(|reg| (reg.get_flusher)())
}
