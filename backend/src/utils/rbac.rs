use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::utils::auth::AuthenticatedUser;

pub enum Role {
    User,
    Admin,
}

pub struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RoleGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleGuardMiddleware {
            service: Rc::new(service),
            role: self.role.clone(),
        }))
    }
}

pub struct RoleGuardMiddleware<S> {
    service: Rc<S>,
    role: Role,
}

impl<S, B> Service<ServiceRequest> for RoleGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let role = self.role.clone();

        Box::pin(async move {
            let user = req.extensions().get::<AuthenticatedUser>();

            match (user, role) {
                (Some(user), Role::Admin) if user.is_admin => service.call(req).await,
                (Some(_), Role::User) => service.call(req).await,
                _ => {
                    let (req, _) = req.into_parts();
                    let res = HttpResponse::Forbidden()
                        .json("Insufficient permissions")
                        .into_body();
                    Ok(ServiceResponse::new(req, res))
                }
            }
        })
    }
}
