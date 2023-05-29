use mongodb::{Collection, bson::doc};
use rocket::{State, serde::json::{Json}, http::Status};

use crate::{models::user::{UserStoreModel, UserAuthModel, UserAuthResponseModel, UserAuthClaimsModel}, errors::ApiError, 
middlewares::auth::SecretKeyWrapper};

type AuthResponse = Result<Json<UserAuthResponseModel>, ApiError>;

#[post("/", data="<auth>")]
pub async fn get_token(
    db :&State<Collection<UserStoreModel>>, 
    secret :&State<SecretKeyWrapper>,
    auth :Json<UserAuthModel>
) -> AuthResponse {
    let user = match db.find_one(doc!{"name": &auth.0.name}, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => user,
            None => return Err(ApiError { status: Status::NotFound, message: format!("User {} not found.", &auth.0.name) })
        }
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    if !user.authenticate(&auth.password) {
        return Err(ApiError { status: Status::Forbidden, message: "Invalid password.".to_string() })
    }

    let claims = UserAuthClaimsModel {
        exp: jsonwebtoken::get_current_timestamp() + 3600,
        _id: user._id.to_hex(),
        name: user.name,
        permissions: user.permissions
    };

    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(), 
        &claims, 
        &jsonwebtoken::EncodingKey::from_secret(&secret.key.to_string().as_bytes())
    ) {
        Ok(token) => token,
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    Ok(Json(UserAuthResponseModel {token}))
}