use super::dinterface::{decode_answer_id, decode_prompt, decode_array};
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use ton_client::encoding::decode_abi_bigint;
use crate::helpers::TonClient;
use crate::debot::term_signing_box::TerminalSigningBox;
use tokio::sync::RwLock;

const ID: &'static str = "c13024e101c95e71afb1f5fa6d72f633d51e721de0320d73dfd6121a54e4d40a";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "get",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"},
				{"name":"possiblePublicKeys","type":"uint256[]"}
			],
			"outputs": [
				{"name":"handle","type":"uint32"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct SigningBoxInput {
    handles: RwLock<Vec<TerminalSigningBox>>,
    client: TonClient,
}
impl SigningBoxInput {
    pub fn new(client: TonClient) -> Self {
        Self { handles: RwLock::new(vec![]), client }
    }

    async fn get(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let possible_keys = decode_array(
            args,
            "possiblePublicKeys",
            |elem| {
                decode_abi_bigint(elem.as_str()?).ok()?;
                Some(elem.as_str().unwrap().to_string())
            }
        )?;
        println!("{}", prompt);
        let signing_box = TerminalSigningBox::new(self.client.clone(), possible_keys).await?;
        let handle = signing_box.handle();
        self.handles.write().await.push(signing_box);
        Ok((answer_id, json!({ "handle": handle.0})))
    }
}

#[async_trait::async_trait]
impl DebotInterface for SigningBoxInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "get" => self.get(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}