pub struct User {
    pub id: i64,
    pub email: String,
    pub role: UserRole,
}

pub enum UserRole {
    Admin,
    User,
}

impl User {}
