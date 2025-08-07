pub mod utils;

// User tests
mod users {
    // Application layer tests
    pub mod application {
        pub mod commands {
            mod test_create_user;
            mod test_login;
        }
    }

    // Domain layer tests
    pub mod domain {
        mod auth;
        mod roles;
        mod user;
        mod user_repository;
    }

    // Infrastructure layer tests
    pub mod infrastructure {
        mod test_jwt_token_service;
        mod test_mappers;
        mod test_models;
    }

    pub mod integration {
        mod test_user_endpoints;
    }

    pub mod mocks;
    pub use mocks::*;
}

mod media {
    pub mod application {
        pub mod commands {
            mod test_delete_media;
            mod test_upload_media;
        }
    }

    pub mod integration {
        mod test_media_endpoints;
    }

    pub mod mocks;
    pub use mocks::*;
}
