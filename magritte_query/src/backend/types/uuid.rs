use rand::{thread_rng, Rng};
use uuid::{Uuid, Bytes};
use crate::types::{RecordType, SurrealId};

impl<T: RecordType> From<Uuid> for SurrealId<T> {
    fn from(uuid: Uuid) -> Self {
        SurrealId::new(uuid.to_string())
    }
}

impl<T: RecordType> TryFrom<SurrealId<T>> for Uuid {
    type Error = uuid::Error;

    fn try_from(id: SurrealId<T>) -> Result<Self, Self::Error> {
        Uuid::parse_str(id.id().to_string().as_str())
    }
}


impl<T: RecordType> SurrealId<T> {
    pub fn gen_v4() -> Self {
        Self::from(Uuid::new_v4())
    }

    pub fn gen_v6() -> Self {
        let mut rng = thread_rng();
        let node_id: [u8; 6] = rng.gen();
        Self::from(Uuid::now_v6(&node_id))
    }

    pub fn as_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(self.id().to_string().as_str()).ok()
    }
}
