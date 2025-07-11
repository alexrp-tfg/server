use utoipa::{openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}, Modify};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let security_scheme = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        );

        if let Some(components) = &mut openapi.components {
            // Add security scheme to existing components
            components.security_schemes.insert("bearer_auth".to_string(), security_scheme);
        } else {
            // Create new components with security scheme
            openapi.components = Some(
                utoipa::openapi::ComponentsBuilder::new()
                    .security_scheme("bearer_auth", security_scheme)
                    .build(),
            );
        }
    }
}
