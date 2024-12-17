use crate::types::ReturnType;

pub trait Returns {
    fn return_type_mut(&mut self) -> &mut Option<ReturnType>;
    fn return_(mut self, return_type: ReturnType) -> Self
    where
        Self: Sized,
    {
        *self.return_type_mut() = Some(return_type);
        self
    }
    /// Specify which fields to return from mutations (CREATE, UPDATE, DELETE)
    /// ```rust,ignore /// # use magritte::QB;
    /// # use surrealdb::Surreal;
    /// # async fn example(db: Surreal<surrealdb::engine::any::Any>) {
    /// let query = QB::<Person>::create(db)
    ///     .content(person)?
    ///     .return_fields(&["id", "name", "created_at"])
    ///     .build();
    /// # }
    /// ```
    fn return_fields(mut self, fields: &[&str]) -> Self
    where
        Self: Sized,
    {
        *self.return_type_mut() = Some(ReturnType::Fields(
            fields.iter().map(|&s| s.to_string()).collect(),
        ));
        self
    }
}
