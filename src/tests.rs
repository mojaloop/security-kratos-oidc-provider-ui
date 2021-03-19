use rocket::http::Status;
use rocket::local::Client;
use rocket_contrib::templates::Template;
use wiremock::matchers::{method, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[async_std::test]
async fn test_healthz() {
    let rocket = super::rocket();
    let client = Client::new(rocket).expect("valid rocket instance");
    let response = client.get("/healthz").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[async_std::test]
async fn requests_registration_with_flow_id() {
    let mock_server = MockServer::start().await;

    // this is the flow ID we'll include in our request and require be sent to our mock
    let flow_id = "2";

    Mock::given(method("GET"))
        .and(query_param("id", flow_id))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{
                "methods": {
                    "oidc": {
                        "config": {
                            "action": "/",
                            "method": "POST",
                            "fields": []
                        }
                    }
                }
            }"#,
        ))
        .expect(1)
        .named("Registration Endpoint")
        .mount(&mock_server)
        .await;

    let rocket = super::rocket().manage(super::RegistrationEndpoint(mock_server.uri()));
    let client = Client::new(rocket).expect("valid rocket instance");
    let response = client.get(format!("/?flow={}", flow_id)).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[async_std::test]
async fn error_messages_cause_error_response() {
    let mock_server = MockServer::start().await;

    let error_text = "A Very Specific Error";

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            "{}{}{}",
            r#"{
                "methods": {
                    "oidc": {
                        "config": {
                            "action": "/",
                            "method": "POST",
                            "messages": [{
                                "id": 1,
                                "type": "error",
                                "text": ""#,
            error_text,
            r#""
                            }],
                            "fields": []
                        }
                    }
                }
            }"#
        )))
        .mount(&mock_server)
        .await;

    let rocket = super::rocket().manage(super::RegistrationEndpoint(mock_server.uri()));
    let client = Client::new(rocket).expect("valid rocket instance");
    let mut response = client.get("/?flow=2").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    assert!(response.body_string().unwrap().contains(error_text));
}

#[test]
fn forms_work() -> Result<(), serde_json::Error> {
    let rocket = super::rocket();
    let action =
        "http://127.0.0.1:4433/self-service/methods/oidc/auth/838895fb-bb05-4075-8a7e-1cd22ca42017";
    let token =
        "GTZrak1TBiIfyyaFajmrp/A6dS4Gh3NpMPzQjJ9ChSS7d4ODojfibHcfgNTX/9ivwci/V1K6GuRY/f4b545NUA==";
    let example1 = format!(
        "{}{}{}{}{}",
        r#"{
        "id": "838895fb-bb05-4075-8a7e-1cd22ca42017",
        "type": "browser",
        "expires_at": "2020-09-05T08:24:36.4797386Z",
        "issued_at": "2020-09-05T08:14:36.4797386Z",
        "request_url": "http://127.0.0.1:4433/self-service/registration/browser",
        "methods": {
          "oidc": {
            "method": "oidc",
            "config": {
              "action": ""#,
        action,
        r#"",
              "method": "POST",
              "fields": [
                {
                  "name": "csrf_token",
                  "type": "hidden",
                  "required": true,
                  "value": ""#,
        token,
        r#""
                },
                {
                  "name": "provider",
                  "type": "submit",
                  "value": "github"
                }
              ]
            }
          }
        }
      }"#
    );
    let kratos1: super::KratosRegistration = serde_json::from_str(&example1)?;
    let output1 = Template::show(&rocket, "form", &kratos1).unwrap();
    println!("{}", output1);
    assert!(output1.contains(action));
    assert!(output1.contains(token));

    let example2 = r#"{
        "id": "838895fb-bb05-4075-8a7e-1cd22ca42017",
        "type": "browser",
        "expires_at": "2020-09-05T08:24:36.4797386Z",
        "issued_at": "2020-09-05T08:14:36.4797386Z",
        "request_url": "http://127.0.0.1:4433/self-service/registration/browser",
        "methods": {
          "oidc": {
            "method": "oidc",
            "config": {
                "action": "http://127.0.0.1:4433/self-service/methods/oidc/auth/2e87577f-1209-407c-b523-4727d7bbdbd4",
                "method": "POST",
                "messages": [
                  {
                    "id": 4000000,
                    "type": "error",
                    "text": "Authentication failed because no id_token was returned. Please accept the \"openid\" permission and try again."
                  }
                ],
                "fields": [
                  {
                    "name": "csrf_token",
                    "type": "hidden",
                    "required": true,
                    "value": "O0e8KWxx1zZeNEgUOxNFvV8dVMrK63MXoxU102lj9n0nXcsaDWXhPn0TiZG3Nk+nLi/pT1wjTUsHniLRKOpp3w=="
                  },
                  {
                    "name": "provider",
                    "type": "submit",
                    "value": "github"
                  }
                ]
              }
          }
        }
      }"#;
    let kratos2: super::KratosRegistration = serde_json::from_str(&example2)?;
    let _output2 = Template::show(&rocket, "form", &kratos2).unwrap();
    Ok(())
}
