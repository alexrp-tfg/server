use crate::media::{MockMediaRepository, MockStorageService};
use crate::users::{MockLoginTokenService, MockUserRepository};
use lib::api::http_server::AppState;
use lib::users::domain::LoginTokenService;
use std::sync::Arc;

/// Creates an AppState for testing with optional custom implementations
///
/// # Arguments
/// * `user_repo` - Optional custom user repository, uses default if None
/// * `token_service` - Optional custom login token service, uses default if None
/// * `media_repo` - Optional custom media repository, uses default if None
/// * `storage_service` - Optional custom storage service, uses default if None
///
/// # Example
/// ```
/// // Default state
/// let state = create_test_app_state(None, None, None, None);
///
/// // Custom user repo only
/// let state = create_test_app_state(Some(custom_user_repo), None, None, None);
///
/// // Custom user repo and token service
/// let state = create_test_app_state(Some(custom_user_repo), Some(custom_token_service), None, None);
/// ```
pub fn create_test_app_state(
    user_repo: Option<MockUserRepository>,
    token_service: Option<Arc<dyn LoginTokenService>>,
    media_repo: Option<MockMediaRepository>,
    storage_service: Option<MockStorageService>,
) -> AppState {
    AppState {
        user_repository: Arc::new(user_repo.unwrap_or_default()),
        login_token_service: token_service.unwrap_or(Arc::new(MockLoginTokenService::default())),
        media_repository: Arc::new(media_repo.unwrap_or_default()),
        storage_service: Arc::new(storage_service.unwrap_or_default()),
    }
}

/// Creates a default AppState for testing (convenience function)
///
/// This is equivalent to `create_test_app_state(None, None, None, None)`
/// but more readable when you just need defaults.
pub fn create_default_test_app_state() -> AppState {
    create_test_app_state(None, None, None, None)
}
