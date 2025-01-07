pub trait Transactional {
    fn is_transaction(&self) -> bool;
    fn in_transaction(&mut self) -> &mut bool;

    fn begin_transaction(mut self) -> Self
    where
        Self: Sized,
    {
        *self.in_transaction() = true;
        self
    }
}
