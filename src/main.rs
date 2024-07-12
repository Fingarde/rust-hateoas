use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde::ser::SerializeSeq;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Link {
    rel: String,
    href: String,
}

#[derive(Serialize, Deserialize)]
struct HateoasResponse<T> {
    data: T,
    #[serde(rename = "_links", skip_serializing_if = "Vec::is_empty")]
    links: Vec<Link>,
}

impl<T> IntoResponse for HateoasResponse<T> where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let json = Json(self);
        json.into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct HateoasData<T> {
    #[serde(flatten)]
    inner: T,
    #[serde(rename = "_links", skip_serializing_if = "Vec::is_empty")]
    links: Vec<Link>,
}

trait IntoHateoasData {
    fn into_hateoas_data(self) -> HateoasData<Self> where Self: Sized;
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}





#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    // extract only the groups name
    #[serde(serialize_with = "serialize_groups")]
    groups: Vec<Group>,
}

fn serialize_groups<S>(groups: &Vec<Group>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(groups.len()))?;
    for group in groups {
        seq.serialize_element(&group.name)?;
    }
    seq.end()
}

#[derive(Serialize, Deserialize)]
struct Group {
    name: String,
}

impl IntoHateoasData for Group {
    fn into_hateoas_data(self) -> HateoasData<Self> {
        HateoasData {
            links: vec![
                Link {
                    rel: "self".to_string(),
                    href: format!("/groups/{}", self.name),
                },
                Link {
                    rel: "users".to_string(),
                    href: format!("/groups/{}/users", self.name),
                },
            ],
            inner: self,
        }
    }
}


impl IntoHateoasData for User {
    fn into_hateoas_data(self) -> HateoasData<Self> {
        HateoasData {
            links: vec![
                Link {
                    rel: "self".to_string(),
                    href: format!("/users/{}", self.id),
                },
                Link {
                    rel: "profile".to_string(),
                    href: format!("/users/{}/profile", self.id),
                },
                Link {
                    rel: "groups".to_string(),
                    href: format!("/users/{}/groups", self.id),
                },
            ],
            inner: self,
        }
    }
}



async fn root() -> impl IntoResponse {
    let response = HateoasResponse {
        links: vec![Link {
            rel: "self".to_string(),
            href: "/users".to_string(),
        }],
        data: vec![
            User {
                id: Uuid::new_v4(),
                name: "tibertra".to_string(),
                email: "timothe.bertrand@uca.fr".to_string(),
                password: "tibertra".to_string(),
                groups: vec![
                    Group {
                        name: "admin".to_string(),
                    },
                ]
            }.into_hateoas_data()
        ],
    };

    response
}
