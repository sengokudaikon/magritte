pub enum Credentials {
    Root {
        user: String,
        pass: String,
    },
    Namespace {
        user: String,
        pass: String,
        ns: String,
    },
    Database {
        user: String,
        pass: String,
        ns: String,
        db: String,
    },
}
