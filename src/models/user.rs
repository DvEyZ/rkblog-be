use mongodb::bson::oid::ObjectId;
use rocket::serde::{Serialize, Deserialize};
use sha256::digest;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum UserPermissionLevel {
    User, Admin
}

#[derive(Deserialize)]
pub struct UserWriteModel {
    pub name :String,
    pub password :String,
    pub permissions :UserPermissionLevel,
    
    pub bio :String
}

#[derive(Serialize, Deserialize)]
pub struct UserStoreModel {
    pub _id :ObjectId,
    pub name :String,
    pub password_hash :String,
    pub permissions :UserPermissionLevel,

    pub bio :String
}

#[derive(Serialize)]
pub struct  UserReadBriefModel {
    pub _id :String,
    pub name :String,
    pub permissions :UserPermissionLevel
}

#[derive(Serialize)]
pub struct UserReadFullModel {
    pub _id :String,
    pub name :String,
    pub permissions :UserPermissionLevel,

    pub bio :String
}

#[derive(Deserialize)]
pub struct UserAuthModel {
    pub name :String,
    pub password :String
}

#[derive(Serialize)]
pub struct UserAuthResponseModel {
    pub token :String
}

#[derive(Serialize,Deserialize)]
pub struct UserAuthClaimsModel {
    pub exp :u64,
    pub _id :String,
    pub name :String,
    pub permissions :UserPermissionLevel,
}

impl UserStoreModel {
    pub fn new(user :UserWriteModel) -> Self {
        Self {
            _id: ObjectId::new(),
            name: user.name,
            password_hash: digest(user.password),
            permissions: user.permissions,

            bio: user.bio
        }
    }

    pub fn from(user :UserWriteModel, id :ObjectId) -> Self {
        Self {
            _id: id,
            name: user.name,
            password_hash: digest(user.password),
            permissions: user.permissions,

            bio: user.bio
        }
    }

    pub fn brief(self) -> UserReadBriefModel {
        UserReadBriefModel {
            _id: self._id.to_hex(),
            name: self.name,
            permissions: self.permissions
        }
    }

    pub fn to(self) -> UserReadFullModel {
        UserReadFullModel {
            _id: self._id.to_hex(),
            name: self.name, 
            permissions: self.permissions,

            bio: self.bio
        }
    }

    pub fn authenticate(&self, password :&str) -> bool {
        digest(password) == self.password_hash
    }
}