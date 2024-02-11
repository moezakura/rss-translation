use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct Request {
    q: Vec<String>,
    target: String,
}

pub struct TranslatResult {
    pub translated: String,
    pub raw_text: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Translated {
    translatedText: String,
}

#[derive(Serialize, Deserialize)]
struct ReponseData {
    translations: Vec<Translated>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    data: ReponseData,
}

#[derive(Clone)]
pub struct TranslateProvider {
    project_id: String,
    service_account_json: String,
    access_token: String,
    access_token_expires_at: i64,
}

pub struct TranslateProviderInitConfig {
    pub project_id: String,
    pub service_account_json: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ServiceAccountFile {
    project_id: String,
    private_key_id: String,
    private_key: String,
    client_email: String,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_x509_cert_url: String,
}

#[derive(Serialize, Deserialize)]
struct GoogleJwt {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

impl TranslateProvider {
    pub fn new(init_config: TranslateProviderInitConfig) -> TranslateProvider {
        TranslateProvider {
            project_id: init_config.project_id,
            service_account_json: init_config.service_account_json,
            access_token: "".to_string(),
            access_token_expires_at: 0,
        }
    }

    async fn get_access_token(&mut self) -> Result<String, Box<dyn Error>> {
        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        if self.access_token_expires_at.clone() > now_sec {
            return Ok(self.access_token.clone());
        }

        let service_account_json_path = &self.service_account_json.clone();
        let service_account_json = std::fs::read_to_string(service_account_json_path)?;
        let service_account_info =
            serde_json::from_str::<ServiceAccountFile>(&service_account_json)?;

        let now = std::time::SystemTime::now();
        let iat = now.duration_since(std::time::UNIX_EPOCH)?.as_secs();
        let exp = iat + 3600;
        let jwt_payload = GoogleJwt {
            iss: service_account_info.clone().client_email,
            scope: "https://www.googleapis.com/auth/cloud-translation".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            exp: exp as i64,
            iat: iat as i64,
        };
        let jwt = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &jwt_payload,
            &jsonwebtoken::EncodingKey::from_rsa_pem(service_account_info.private_key.as_bytes())?,
        )?;

        let endpoint = "https://oauth2.googleapis.com/token";
        let client = reqwest::Client::new();
        let response = client
            .post(endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;
        let response_body = response.text().await?;
        let parsed_response: serde_json::Value = serde_json::from_str(&response_body)?;
        let token = parsed_response["access_token"].as_str().unwrap();

        self.access_token = token.to_string();
        self.access_token_expires_at = exp as i64;

        Ok(token.to_string())
    }

    pub async fn translate(
        &mut self,
        target_strs: Vec<String>,
        to: String,
    ) -> Result<Vec<TranslatResult>, Box<dyn Error>> {
        //
        let endpoint = "https://translation.googleapis.com/language/translate/v2";

        let project_id = &self.project_id.clone();
        let api_key = self.get_access_token().await?;

        let request_json = Request {
            q: target_strs.clone(),
            target: to,
        };

        let client = reqwest::Client::new();

        let response = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("x-goog-user-project", project_id)
            .body(serde_json::to_string(&request_json)?)
            .send()
            .await?;
        let response_body = response.text().await?;

        let parsed_response: Response = serde_json::from_str(&response_body)?;
        let translated: Vec<String> = parsed_response
            .data
            .translations
            .iter()
            .map(|t| t.translatedText.clone())
            .collect();

        let result = target_strs
            .iter()
            .zip(translated.iter())
            .map(|(raw, translated)| TranslatResult {
                translated: translated.clone(),
                raw_text: raw.clone(),
            })
            .collect();

        Ok(result)
    }
}
