use mongodb::{Client, options::ClientOptions, error::Error, Collection};

use crate::models::{post::PostStoreModel, user::UserStoreModel};

pub struct Db {
    pub posts :Collection<PostStoreModel>,
    pub users :Collection<UserStoreModel>
}

pub async fn connect(uri :&str, db :&str) -> Result<Db, Error> {
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let db = client.database(db);

    let posts = db.collection::<PostStoreModel>("Post");
    let users = db.collection::<UserStoreModel>("User");

    Ok(Db { posts, users })
}