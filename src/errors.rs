use std::io::Cursor;
use rocket::{http::{Status, ContentType}, Response, serde::json::serde_json::json};

#[derive(Debug)]
pub struct ApiError {
    pub status: Status,
    pub message: String
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for ApiError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        let body = json!({
            "message": self.message
        }).to_string();

        Response::build()
            .sized_body(body.len(), Cursor::new(body))
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[catch(401)]
pub fn unauthorized(_ :&rocket::Request) -> ApiError {
    ApiError { 
        status: Status::Unauthorized, 
        message: String::from("You need to authenticate to access this resource.")
    }
}

#[catch(403)]
pub fn forbidden(_ :&rocket::Request) -> ApiError {
    ApiError { 
        status: Status::Unauthorized, 
        message: String::from("You don't have permission to access this resource.")
    }
}