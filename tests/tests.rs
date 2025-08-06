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
        mod user_repository;
        mod user;
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
            mod test_upload_media;
        }
    }
    
    pub mod mocks;
    pub use mocks::*;
}
