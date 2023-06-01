use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection};
use rocket::{serde::{Serialize, Deserialize}, http::Status};

use crate::errors::ApiError;

use super::user::{UserReadBriefModel, UserStoreModel};

#[derive(Deserialize)]
pub struct PostWriteModel {
    pub title :String,
    pub content :String,
}

#[derive(Serialize)]
pub struct PostReadBriefModel {
    pub _id :String,
    pub title :String,
    pub author :String
}

#[derive(Serialize)]
pub struct PostReadFullModel {
    pub _id :String,
    pub title :String,
    pub content :String,
    pub author :UserReadBriefModel
}

#[derive(Serialize,Deserialize)]
pub struct PostStoreModel {
    pub _id :ObjectId,
    pub title :String,
    pub content :String,
    pub author :ObjectId
}

impl PostStoreModel {
    pub async fn query_author(user_ref :&Collection<UserStoreModel>, filter :Document) -> Result<UserStoreModel, ApiError> {
        match user_ref.find_one(filter, None).await {
            Ok(maybe_user) => match maybe_user {
                Some(user) => Ok(user),
                None => Err(ApiError { status: Status::NotFound, message: format!("User not found.") })
            }
            Err(e) => Err(ApiError { status: Status::InternalServerError, message: e.to_string() })
        }
    }

    pub async fn new(post :PostWriteModel, user_ref :&Collection<UserStoreModel>, author :&str) -> Result<Self, ApiError> {
        let author = Self::query_author(user_ref, doc!{"name": author}).await?;

        Ok(Self {
            _id: ObjectId::new(),
            title: post.title,
            content: post.content,
            author: author._id
        })
    }

    pub async fn from(post :PostWriteModel, id :ObjectId, user_ref :&Collection<UserStoreModel>, author :&str) -> Result<Self, ApiError> {
        let author = Self::query_author(user_ref, doc!{"name": author}).await?;

        Ok(Self {
            _id: id,
            title: post.title,
            content: post.content,
            author: author._id
        })
    }

    pub async fn brief(self, user_ref :&Collection<UserStoreModel>) -> Result<PostReadBriefModel, ApiError> {
        let author = Self::query_author(user_ref, doc!{"_id": &self.author}).await?;

        Ok(PostReadBriefModel {
            _id: self._id.to_hex(),
            title: self.title,
            author: author.name
        })
    }

    pub async fn to(self, user_ref :&Collection<UserStoreModel>) -> Result<PostReadFullModel, ApiError> {
        let author = Self::query_author(user_ref, doc!{"_id": &self.author}).await?;

        Ok(PostReadFullModel {
            _id: self._id.to_hex(),
            title: self.title,
            content: self.content,
            author: author.brief()
        })
    }
}