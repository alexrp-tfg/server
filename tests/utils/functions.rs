use password_hash::rand_core::OsRng;

pub fn hash_password(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher};
    use password_hash::SaltString;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
