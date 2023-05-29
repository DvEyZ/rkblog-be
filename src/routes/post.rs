use mongodb::{bson::{doc}, Collection};
use rocket::{State, serde::json::Json, futures::TryStreamExt, http::Status, response::status::Created};
use crate::{models::{post::{ PostStoreModel, PostReadBriefModel, PostReadFullModel, PostWriteModel}, 
    user::{UserStoreModel, UserPermissionLevel}}, middlewares::auth::{AuthorizeToken, UserAuthorization}};
use crate::errors::ApiError;

type PostsResponse = Result<Json<Vec<PostReadBriefModel>>, ApiError>;
type PostResponse = Result<Json<PostReadFullModel>, ApiError>;
type PostResponseCreated = Result<Created<Json<PostReadFullModel>>, ApiError>;

#[get("/")]
pub async fn list(
    db :&State<Collection<PostStoreModel>>, 
    ref_users :&State<Collection<UserStoreModel>>,
    _auth :AuthorizeToken<UserAuthorization>
) -> PostsResponse {
    let mut results = match db.find(None, None).await {
        Ok(posts) => posts,
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()} )
    };

    let mut posts :Vec<PostReadBriefModel> = vec![];
    while let Ok(Some(i)) = results.try_next().await {
        posts.push(i.brief(&ref_users).await?);
    } 
        
    Ok(Json(posts))
}

#[get("/<title>")]
pub async fn get<'a>(
    db :&State<Collection<PostStoreModel>>, 
    ref_users :&State<Collection<UserStoreModel>>,
    _auth :AuthorizeToken<UserAuthorization>,
    title :&'a str
) -> PostResponse {
    let post = match db.find_one(doc!{"title": title}, None).await {
        Ok(maybe_post) => match maybe_post {
            Some(post) => post,
            None => return Err(ApiError{status: Status::NotFound, message: format!("Post {} not found.", title)})
        },
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    };

    Ok(Json(post.to(&ref_users).await?))
}

#[post("/", data="<post>")]
pub async fn create(
    db :&State<Collection<PostStoreModel>>, 
    ref_users :&State<Collection<UserStoreModel>>,
    auth :AuthorizeToken<UserAuthorization>,
    post :Json<PostWriteModel>
) -> PostResponseCreated {
    match db.find_one(doc!{"title": &post.0.title}, None).await {
        Ok(maybe_post) => if let Some(_thing) = maybe_post {
            return Err(ApiError{ status: Status::NotFound, message: format!("Post {} already exists.", &post.0.title)})
        }
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    };

    let new_post = PostStoreModel::new(post.0, &ref_users, &auth.claim.name).await?;

    match db.insert_one(&new_post, None).await {
        Ok(_ok) => return Ok(Created::new(format!("{}", &new_post.title)).body(Json(new_post.to(&ref_users).await?))),
        Err(e) => return Err(ApiError{status: Status::InternalServerError, message: e.to_string()})
    };
}

#[put("/<title>", data="<post>")]
pub async fn update<'a>(
    db :&State<Collection<PostStoreModel>>, 
    ref_users :&State<Collection<UserStoreModel>>,
    auth :AuthorizeToken<UserAuthorization>,
    title :&'a str, 
    post :Json<PostWriteModel>
) -> PostResponse {
    let origin_post = match db.find_one(doc!{"title": title}, None).await {
        Ok(maybe_post) => match maybe_post {
            Some(post) => post,
            None => return Err(ApiError{ status: Status::NotFound, message: format!("Post {} not found.", title)})
        }
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    };

    let author = PostStoreModel::query_author(&ref_users, doc!{"_id": &origin_post.author}).await?;

    if &author.name != &auth.claim.name {
        if auth.claim.permissions != UserPermissionLevel::Admin {
            return Err(ApiError { status: Status::Forbidden, message: "You don't have permission to modify this resource.".to_string() })
        }
    }

    let replace_post = PostStoreModel::from(post.0, origin_post._id, &ref_users, &auth.claim.name).await?;

    match db.replace_one(doc!{"_id": &origin_post._id}, &replace_post, None).await {
        Ok(_ok) => Ok(Json(replace_post.to(&ref_users).await?)),
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    }
}

#[delete("/<title>")]
pub async fn delete<'a>(
    db :&State<Collection<PostStoreModel>>,
    ref_users :&State<Collection<UserStoreModel>>,
    auth :AuthorizeToken<UserAuthorization>,
    title :&'a str
) -> PostResponse {
    let post = match db.find_one(doc!{"title": title}, None).await {
        Ok(maybe_post) => match maybe_post {
            Some(post) => post,
            None => return Err(ApiError{ status: Status::NotFound, message: format!("Post {} not found.", title)})
        },
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    };

    let author = PostStoreModel::query_author(&ref_users, doc!{"_id": &post.author}).await?;

    if &author.name != &auth.claim.name {
        if auth.claim.permissions != UserPermissionLevel::Admin {
            return Err(ApiError { status: Status::Forbidden, message: "You don't have permission to delete this resource.".to_string() })
        }
    }

    match db.delete_one(doc!{"title": title}, None).await {
        Ok(_ok) => Ok(Json(post.to(&ref_users).await?)),
        Err(e) => return Err(ApiError{ status: Status::InternalServerError, message: e.to_string()})
    }
}