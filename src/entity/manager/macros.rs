#[macro_export]
macro_rules! impl_entity_flush {
    ($($entity:ty),* $(,)?) => {
        impl $crate::entity::manager::EntityManager {
            #[doc(hidden)]
            pub async fn flush_entity_state(&self, state: &mut $crate::entity::manager::unsafe_em::TypeErasedState) -> anyhow::Result<()>
            where
                $($entity: $crate::TableTrait + $crate::HasId + $crate::RecordType + Sized + $crate::entity_crud::SurrealCrud<$entity>,)*
            {
                $(
                    // Safety: TypeErasedState ensures type safety through TypeId checks
                    if let Some(state) = unsafe { state.as_mut::<$entity>() } {
                        // Handle new entities
                        for entity in state.new.drain(..) {
                            <$entity as $crate::entity_crud::SurrealCrud<$entity>>::insert(&entity)?.execute(self.db()).await?;
                        }

                        // Handle removed entities
                        for entity in state.removed.drain(..) {
                            <$entity as $crate::entity_crud::SurrealCrud<$entity>>::delete(&entity)?.execute(self.db()).await?;
                        }

                        // Handle dirty entities
                        for id in state.dirty.drain() {
                            if let Some(entity) = state.managed.get(&id) {
                                <$entity as $crate::entity_crud::SurrealCrud<$entity>>::update(entity)?.execute(self.db()).await?;
                            }
                        }
                        return Ok(());
                    }
                )*

                Err(anyhow::anyhow!("Unknown entity type"))
            }
        }
    };
}
