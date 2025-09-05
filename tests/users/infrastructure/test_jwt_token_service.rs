use lib::users::{
    domain::{Claims, LoginTokenService, Role, user::UserLoginError},
    infrastructure::jwt_token_service::{JwtTokenConfig, JwtTokenService},
};
use uuid::Uuid;

fn test_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4(),
        username: "alice".to_string(),
        role: Role::User,
        exp: 9999999999,
    }
}

#[test]
fn test_create_token_success() {
    let config = JwtTokenConfig {
        secret_key: "testsecret".to_string(),
    };
    let service = JwtTokenService::new(config.clone());
    let claims = test_claims();
    let token = service.create_token(claims.clone());
    assert!(token.is_ok());
    let token = token.unwrap();
    // Should be able to validate the token
    let claims2 = service.validate_token(&token.0);
    assert!(claims2.is_ok());
    assert_eq!(claims2.unwrap().username, claims.username);
}

#[test]
fn test_validate_token_invalid() {
    let config = JwtTokenConfig {
        secret_key: "testsecret".to_string(),
    };
    let service = JwtTokenService::new(config);
    let result = service.validate_token("not_a_real_token");
    assert!(matches!(result, Err(UserLoginError::InvalidToken)));
}
