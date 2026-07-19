use crate::ServiceType;
use super::{control_service, get_service_status, ServiceAction};
use crate::ServiceStatus;

pub async fn start() -> Result<(), String> {
    control_service(&ServiceType::FileZilla, ServiceAction::Start).await
}

pub async fn stop() -> Result<(), String> {
    control_service(&ServiceType::FileZilla, ServiceAction::Stop).await
}

pub async fn status() -> Result<ServiceStatus, String> {
    get_service_status(&ServiceType::FileZilla).await
}
