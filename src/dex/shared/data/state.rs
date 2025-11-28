use model::models;
use repository::repositories;

#[derive(Clone)]
pub struct AppState {
    pub repository: repositories::Repositories,
    pub model: models::Models,
}

impl AppState {
    pub fn new(
        repository: repositories::Repositories,
        model: models::Models,
    ) -> Self {
        Self { repository, model }
    }
}