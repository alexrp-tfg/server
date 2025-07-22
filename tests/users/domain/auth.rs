use lib::users::domain::{Claims, Role, Token};
// Auth domain tests
use serde_json;

#[test]
fn test_claims_struct_serde() {
    let claims = Claims {
        sub: "user-id".to_string(),
        username: "alice".to_string(),
        role: Role::Admin,
        exp: 123456,
    };
    let json = serde_json::to_string(&claims).unwrap();
    let de: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(claims, de);
}

#[test]
fn test_token_struct_serde() {
    let token = Token("sometoken".to_string());
    let json = serde_json::to_string(&token).unwrap();
    let de: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(token, de);
} 
