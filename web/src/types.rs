#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: i64,
    pub clerk_id: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}
