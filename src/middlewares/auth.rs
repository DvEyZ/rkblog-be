use std::marker::PhantomData;

use rocket::{request::{Outcome, FromRequest}, http::Status, State, outcome::Outcome::{Success}};

use crate::{models::user::{UserAuthClaimsModel, UserPermissionLevel}};

pub struct SecretKeyWrapper {
    pub key :rocket::config::SecretKey
}

pub trait Authorize {
    fn authorize(claim :&UserAuthClaimsModel) -> bool;
}
pub struct UserAuthorization {}
pub struct AdminPermissionAuthorization {}

impl Authorize for UserAuthorization {
    fn authorize(_claim :&UserAuthClaimsModel) -> bool {
        true
    }
}

impl Authorize for AdminPermissionAuthorization {
    fn authorize(claim :&UserAuthClaimsModel) -> bool {
        claim.permissions == UserPermissionLevel::Admin
    }
}

pub struct AuthorizeToken <T :Authorize> {
    token_type :PhantomData<T>,
    pub claim :UserAuthClaimsModel
}

#[rocket::async_trait]
impl<T :Authorize, 'r> FromRequest<'r> for AuthorizeToken<T> {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match request.headers().get_one("Authorization") {
            Some(token) => {
                let mut spl = token.split(' ');
                if let Some(name) = spl.nth(0) {
                    if name != "Bearer" {
                        return Outcome::Failure((Status::BadRequest, ()))
                    }
                };
                match spl.nth(0) {
                    Some(token) => token,
                    None => return Outcome::Failure((Status::BadRequest, ()))
                }
            },
            None => return Outcome::Failure((Status::Unauthorized, ()))
        };

        let secret = match request.guard::<&State<SecretKeyWrapper>>().await.map(|conf| &conf.key) {
            Success(key) => key,
            _ => return Outcome::Failure((Status::InternalServerError, ()))
        };

        let claim = match jsonwebtoken::decode::<UserAuthClaimsModel>(
            token, 
            &jsonwebtoken::DecodingKey::from_secret(secret.to_string().as_bytes()), 
            &jsonwebtoken::Validation::default()
        ) {
            Ok(claim) => claim.claims,
            Err(_e) => return Outcome::Failure((Status::Forbidden, ()))
        };
        
        if &claim.exp < &jsonwebtoken::get_current_timestamp() {
            return Outcome::Failure((Status::Unauthorized, ()))
        }

        if !T::authorize(&claim) {
            return Outcome::Failure((Status::Forbidden, ()))
        }

        return Outcome::Success(AuthorizeToken {claim, token_type: PhantomData::default()});
    }
}