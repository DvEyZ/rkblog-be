use mongodb::{Collection, bson::doc};
use rocket::{State, serde::json::Json, http::Status, futures::TryStreamExt, response::status::{Created}};
use crate::{models::{user::{UserStoreModel, UserReadFullModel, UserWriteModel, UserPermissionLevel, UserReadBriefModel}, post::PostStoreModel}, 
    errors::ApiError, 
    middlewares::auth::{AuthorizeToken, UserAuthorization, AdminPermissionAuthorization}};

type UsersResponse = Result<Json<Vec<UserReadBriefModel>>, ApiError>;
type UserResponse = Result<Json<UserReadFullModel>, ApiError>;
type UserResponseCreated = Result<Created<Json<UserReadFullModel>>, ApiError>;

#[get("/")]
pub async fn list(
    db :&State<Collection<UserStoreModel>>,
    _auth :AuthorizeToken<UserAuthorization>
) -> UsersResponse {
    let mut results = match db.find(None, None).await {
        Ok(users) => users,
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    let mut users = vec![];
    while let Ok(Some(user)) = results.try_next().await {
        users.push(user.brief())
    };

    Ok(Json(users))
}

#[get("/<name>")]
pub async fn get<'a>(
    db :&State<Collection<UserStoreModel>>, 
    _auth: AuthorizeToken<UserAuthorization>,
    name :&'a str
) -> UserResponse {
    let user = match db.find_one(doc! {"name": name}, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => user,
            None => return Err(ApiError { status: Status::NotFound, message: format!("User {} not found.", name) })
        }
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    Ok(Json(user.to()))
}

#[post("/", data="<user>")]
pub async fn create(
    db :&State<Collection<UserStoreModel>>, 
    _auth: AuthorizeToken<AdminPermissionAuthorization>,
    user :Json<UserWriteModel>
) -> UserResponseCreated {
    match db.find_one(doc! {"name": &user.0.name}, None).await {
        Ok(maybe_user) => if let Some(_thing) = maybe_user {
            return Err(ApiError { status: Status::Conflict, message: format!("User {} already exists.", &user.0.name) })
        }
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    }
    
    let new_user = UserStoreModel::new(user.0);

    match db.insert_one(&new_user, None).await {
        Ok(_ok) => return Ok(Created::new(format!("{}", &new_user.name)).body(Json(new_user.to()))),
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    }
}

#[put("/<name>", data="<user>")]
pub async fn update<'a>(
    db :&State<Collection<UserStoreModel>>, 
    auth :AuthorizeToken<UserAuthorization>,
    name :&'a str, 
    user :Json<UserWriteModel>
) -> UserResponse {
    let origin_user = match db.find_one(doc! {"name": name}, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => user,
            None => return Err(ApiError { status: Status::NotFound, message: format!("User {} not found.", name) })
        }
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    if &origin_user.name != &auth.claim.name {
        if auth.claim.permissions != UserPermissionLevel::Admin {
            return Err(ApiError { status: Status::Forbidden, message: "You don't have permission to modify this resource.".to_string() })
        }
    }
    
    let replace_user = UserStoreModel::from(user.0, origin_user._id);

    match db.replace_one(doc! {"_id": &origin_user._id}, &replace_user, None).await {
        Ok(_ok) => return Ok(Json(replace_user.to())),
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    }
}

#[delete("/<name>")]
pub async fn delete<'a>(
    db :&State<Collection<UserStoreModel>>, 
    post_ref :&State<Collection<PostStoreModel>>,
    _auth :AuthorizeToken<AdminPermissionAuthorization>,
    name :&'a str
) -> UserResponse {
    let user = match db.find_one(doc! {"name": name}, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => user,
            None => return Err(ApiError { status: Status::NotFound, message: format!("User {} not found.", name) })
        }
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    };

    match db.delete_one(doc! {"name": name}, None).await {
        Ok(_ok) => {
            // Delete all posts of the deleted user
            match post_ref.delete_many(doc!{"author": user._id}, None).await {
                Ok(_ok) => return Ok(Json(user.to())),
                Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
            }
        },
        Err(e) => return Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
    }
}