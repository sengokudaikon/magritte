use std::any::TypeId;
use std::sync::OnceLock;

pub trait EntityStateFlusher: Send + Sync {
    fn type_id(&self) -> TypeId;
    fn flush(&self, state: *mut (), db: crate::SurrealDB) -> anyhow::Result<()>;
}

pub struct EntityFlusherRegistration {
    pub get_flusher: fn() -> &'static dyn EntityStateFlusher,
}

#[macro_export]
macro_rules! impl_entity_flush {
    ($entity:ty) => {
        const _: () = {
            #[doc(hidden)]
            pub struct EntityFlusher<T>(std::marker::PhantomData<T>);

            impl EntityFlusher<$entity> {
                const fn new() -> Self {
                    Self(std::marker::PhantomData)
                }
            }

            impl $crate::entity::manager::EntityStateFlusher for EntityFlusher<$entity> {
                fn type_id(&self) -> std::any::TypeId {
                    std::any::TypeId::of::<$entity>()
                }

                fn flush(&self, state_ptr: *mut (), db: $crate::SurrealDB) -> anyhow::Result<()> {
                    // Safety: We verify the type through TypeId before casting
                    let state = unsafe { &mut *(state_ptr as *mut $crate::entity::manager::EntityState<$entity>) };
                    let rt = tokio::runtime::Handle::current();
                    
                    // Handle new entities
                    for entity in state.new.drain(..) {
                        rt.block_on(
                            <$entity as $crate::entity_crud::SurrealCrud<$entity>>::insert(&entity)?
                                .execute(db.clone())
                        )?;
                    }

                    // Handle removed entities
                    for entity in state.removed.drain(..) {
                        rt.block_on(
                            <$entity as $crate::entity_crud::SurrealCrud<$entity>>::delete(&entity)?
                                .execute(db.clone())
                        )?;
                    }

                    // Handle dirty entities
                    for id in state.dirty.drain() {
                        if let Some(entity) = state.managed.get(&id) {
                            rt.block_on(
                                <$entity as $crate::entity_crud::SurrealCrud<$entity>>::update(entity)?
                                    .execute(db.clone())
                            )?;
                        }
                    }
                    Ok(())
                }
            }

            static FLUSHER: std::sync::OnceLock<EntityFlusher<$entity>> = std::sync::OnceLock::new();

            fn get_flusher() -> &'static dyn $crate::entity::manager::EntityStateFlusher {
                FLUSHER.get_or_init(|| EntityFlusher::<$entity>::new())
            }

            inventory::submit! {
                $crate::entity::manager::EntityFlusherRegistration {
                    get_flusher: get_flusher,
                }
            }
        };
    };
}